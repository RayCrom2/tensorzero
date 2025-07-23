use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::OnceLock;
use tokio::time::Instant;
use url::Url;
use uuid::Uuid;

use crate::cache::ModelProviderRequest;
use crate::endpoints::inference::InferenceCredentials;
use crate::error::{Error, ErrorDetails};
use crate::inference::types::batch::StartBatchProviderInferenceResponse;
use crate::inference::types::batch::{BatchRequestRow, PollBatchInferenceResponse};
use crate::inference::types::{
    current_timestamp, ContentBlock, ContentBlockOutput, FinishReason, Latency,
    ModelInferenceRequest, PeekableProviderInferenceResponseStream, ProviderInferenceResponse,
    RequestMessage, Text, Usage,
};
use crate::inference::InferenceProvider;
use crate::model::{build_creds_caching_default, Credential, CredentialLocation, ModelProvider};
use crate::providers::helpers::inject_extra_request_data_and_send;

pub const PROVIDER_TYPE: &str = "nvidia_nim";

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[cfg_attr(test, ts(export))]
pub struct NvidiaNimProvider {
    model_name: String,
    api_base: Url,
    #[serde(skip)]
    #[cfg_attr(test, ts(skip))]
    credentials: NvidiaNimCredentials,
}

pub static DEFAULT_CREDENTIALS: OnceLock<NvidiaNimCredentials> = OnceLock::new();

fn default_api_key_location() -> CredentialLocation {
    CredentialLocation::Env("NVIDIA_API_KEY".to_string())
}

#[derive(Clone, Debug)]
pub enum NvidiaNimCredentials {
    Static(SecretString),
    Dynamic(String),
    None,
}

impl TryFrom<Credential> for NvidiaNimCredentials {
    type Error = Error;

    fn try_from(credential: Credential) -> Result<Self, Self::Error> {
        match credential {
            Credential::Static(key) => Ok(NvidiaNimCredentials::Static(key)),
            Credential::FileContents(key) => Ok(NvidiaNimCredentials::Static(key)),
            Credential::Dynamic(key_name) => Ok(NvidiaNimCredentials::Dynamic(key_name)),
            Credential::None | Credential::Missing => Ok(NvidiaNimCredentials::None),
            _ => Err(Error::new(ErrorDetails::Config {
                message: "Invalid api_key_location for NVIDIA NIM provider".to_string(),
            })),
        }
    }
}

impl NvidiaNimProvider {
    pub fn new(
        model_name: String,
        api_base: Url,
        api_key_location: Option<CredentialLocation>,
    ) -> Result<Self, Error> {
        let credentials = build_creds_caching_default(
            api_key_location,
            default_api_key_location(),
            PROVIDER_TYPE,
            &DEFAULT_CREDENTIALS,
        )?;

        Ok(NvidiaNimProvider {
            model_name,
            api_base,
            credentials,
        })
    }

    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    fn make_body(&self, request: ModelProviderRequest<'_>) -> Result<Value, Error> {
        let mut body = json!({
            "model": self.model_name,
            "messages": self.format_messages(&request.request.messages)?,
            "stream": request.request.stream,
        });

        // Add inference parameters if present
        if let Some(temperature) = request.request.temperature {
            body["temperature"] = json!(temperature);
        }
        if let Some(max_tokens) = request.request.max_tokens {
            body["max_tokens"] = json!(max_tokens);
        }
        if let Some(top_p) = request.request.top_p {
            body["top_p"] = json!(top_p);
        }

        Ok(body)
    }

    fn format_messages(&self, messages: &[RequestMessage]) -> Result<Vec<Value>, Error> {
        messages
            .iter()
            .map(|msg| {
                Ok(json!({
                    "role": match msg.role {
                        crate::inference::types::Role::User => "user",
                        crate::inference::types::Role::Assistant => "assistant",
                    },
                    "content": self.format_content_blocks(&msg.content)?
                }))
            })
            .collect()
    }

    fn format_content_blocks(&self, content: &[ContentBlock]) -> Result<String, Error> {
        // For now, just concatenate text content
        // You would expand this to handle images, tools, etc. based on NIM capabilities
        let text = content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::Text(text) => Some(text.text.as_str()),
                _ => None, // Handle other content types as needed
            })
            .collect::<Vec<_>>()
            .join(" ");

        Ok(text)
    }

    fn parse_response(
        &self,
        request: &ModelInferenceRequest,
        raw_request: String,
        raw_response: String,
        latency: Latency,
    ) -> Result<ProviderInferenceResponse, Error> {
        let response: NimResponse = serde_json::from_str(&raw_response).map_err(|e| {
            Error::new(ErrorDetails::InferenceServer {
                message: format!("Error parsing NIM response: {e}"),
                provider_type: PROVIDER_TYPE.to_string(),
                raw_request: Some(raw_request.clone()),
                raw_response: Some(raw_response.clone()),
            })
        })?;

        let content = response
            .choices
            .into_iter()
            .filter_map(|choice| choice.message.content)
            .map(|text| ContentBlockOutput::Text(Text { text }))
            .collect();

        Ok(ProviderInferenceResponse {
            id: Uuid::now_v7(),
            created: current_timestamp(),
            output: content,
            system: request.system.clone(),
            input_messages: request.messages.clone(),
            raw_request,
            raw_response,
            usage: Usage {
                input_tokens: response.usage.prompt_tokens,
                output_tokens: response.usage.completion_tokens,
            },
            latency,
            finish_reason: Some(FinishReason::Stop), // Adapt based on NIM response
        })
    }
}

#[derive(Debug, Deserialize)]
struct NimResponse {
    choices: Vec<NimChoice>,
    usage: NimUsage,
}

#[derive(Debug, Deserialize)]
struct NimChoice {
    message: NimMessage,
}

#[derive(Debug, Deserialize)]
struct NimMessage {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NimUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

impl InferenceProvider for NvidiaNimProvider {
    async fn infer<'a>(
        &'a self,
        request: ModelProviderRequest<'a>,
        http_client: &'a reqwest::Client,
        dynamic_api_keys: &'a InferenceCredentials,
        model_provider: &'a ModelProvider,
    ) -> Result<ProviderInferenceResponse, Error> {
        let request_body = self.make_body(request)?;
        let request_url = self.api_base.join("chat/completions").map_err(|e| {
            Error::new(ErrorDetails::Config {
                message: format!("Invalid API base URL: {e}"),
            })
        })?;

        let api_key = self.credentials.get_api_key(dynamic_api_keys)?;
        let start_time = Instant::now();

        let mut request_builder = http_client.post(request_url);

        if let Some(api_key) = api_key {
            request_builder = request_builder.bearer_auth(api_key.expose_secret());
        }

        let (res, raw_request) = inject_extra_request_data_and_send(
            PROVIDER_TYPE,
            &request.request.extra_body,
            &request.request.extra_headers,
            model_provider,
            request.model_name,
            request_body,
            request_builder,
        )
        .await?;

        if res.status().is_success() {
            let raw_response = res.text().await.map_err(|e| {
                Error::new(ErrorDetails::InferenceServer {
                    message: format!("Error parsing response: {e}"),
                    provider_type: PROVIDER_TYPE.to_string(),
                    raw_request: Some(raw_request.clone()),
                    raw_response: None,
                })
            })?;

            let latency = Latency::NonStreaming {
                response_time: start_time.elapsed(),
            };

            self.parse_response(request.request, raw_request, raw_response, latency)
        } else {
            let error_response = res.text().await.map_err(|e| {
                Error::new(ErrorDetails::InferenceServer {
                    message: format!("Error parsing error response: {e}"),
                    provider_type: PROVIDER_TYPE.to_string(),
                    raw_request: Some(raw_request.clone()),
                    raw_response: None,
                })
            })?;

            Err(Error::new(ErrorDetails::InferenceServer {
                message: format!("NIM API error: {error_response}"),
                provider_type: PROVIDER_TYPE.to_string(),
                raw_request: Some(raw_request),
                raw_response: Some(error_response),
            }))
        }
    }

    async fn infer_stream<'a>(
        &'a self,
        _request: ModelProviderRequest<'a>,
        _http_client: &'a reqwest::Client,
        _dynamic_api_keys: &'a InferenceCredentials,
        _model_provider: &'a ModelProvider,
    ) -> Result<(PeekableProviderInferenceResponseStream, String), Error> {
        // Implement streaming if NIM supports it
        Err(Error::new(
            ErrorDetails::UnsupportedModelProviderForBatchInference {
                provider_type: PROVIDER_TYPE.to_string(),
            },
        ))
    }

    async fn start_batch_inference<'a>(
        &'a self,
        _requests: &'a [ModelInferenceRequest<'_>],
        _client: &'a reqwest::Client,
        _dynamic_api_keys: &'a InferenceCredentials,
    ) -> Result<StartBatchProviderInferenceResponse, Error> {
        // Implement batch inference if NIM supports it
        Err(Error::new(
            ErrorDetails::UnsupportedModelProviderForBatchInference {
                provider_type: PROVIDER_TYPE.to_string(),
            },
        ))
    }

    async fn poll_batch_inference<'a>(
        &'a self,
        _batch_request: &'a BatchRequestRow<'a>,
        _http_client: &'a reqwest::Client,
        _dynamic_api_keys: &'a InferenceCredentials,
    ) -> Result<PollBatchInferenceResponse, Error> {
        Err(Error::new(
            ErrorDetails::UnsupportedModelProviderForBatchInference {
                provider_type: PROVIDER_TYPE.to_string(),
            },
        ))
    }
}

impl NvidiaNimCredentials {
    fn get_api_key<'a>(
        &'a self,
        dynamic_api_keys: &'a InferenceCredentials,
    ) -> Result<Option<&'a SecretString>, Error> {
        match self {
            NvidiaNimCredentials::Static(api_key) => Ok(Some(api_key)),
            NvidiaNimCredentials::Dynamic(key_name) => match dynamic_api_keys.get(key_name) {
                Some(api_key) => Ok(Some(api_key)),
                None => Err(Error::new(ErrorDetails::ApiKeyMissing {
                    provider_name: PROVIDER_TYPE.to_string(),
                })),
            },
            NvidiaNimCredentials::None => Ok(None),
        }
    }
}

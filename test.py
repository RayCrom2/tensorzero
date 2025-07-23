import os

from dotenv import load_dotenv
from tensorzero import TensorZeroGateway

# Load the .env file
load_dotenv()

# Access the environment variable
url = os.getenv("TENSORZERO_CLICKHOUSE_URL")

with TensorZeroGateway.build_embedded(
    clickhouse_url=url,
    config_file="config/tensorzero.toml",
) as client:
    response = client.inference(
        function_name="generate_short_story",
        input={
            "messages": [
                {
                    "role": "user",
                    "content": "Write a two sentence short story about the sun.",
                }
            ]
        },
    )

print(response)

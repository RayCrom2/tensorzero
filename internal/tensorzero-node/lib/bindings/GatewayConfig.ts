// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { ExportConfig } from "./ExportConfig";
import type { ObservabilityConfig } from "./ObservabilityConfig";

export type GatewayConfig = {
  bind_address: string | null;
  observability: ObservabilityConfig;
  debug: boolean;
  /**
   * If `true`, allow minijinja to read from the filesystem (within the tree of the config file) for '{% include %}'
   * Defaults to `false`
   */
  enable_template_filesystem_access: boolean;
  export: ExportConfig;
  base_path: string | null;
};

import { XyNginCConfig } from "./types";

/**
 * Validates the plugin configuration.
 *
 * @param config - The configuration to validate.
 * @throws Error if the configuration is invalid.
 */
export function validateConfig(config: XyNginCConfig): void {
  if (!config.domains || config.domains.length === 0) {
    throw new Error(
      "[XyNginC] Configuration error: 'domains' array cannot be empty",
    );
  }

  for (const domain of config.domains) {
    if (!domain.domain || typeof domain.domain !== "string") {
      throw new Error(
        "[XyNginC] Configuration error: 'domain' must be a non-empty string",
      );
    }

    if (
      !domain.port ||
      typeof domain.port !== "number" ||
      domain.port < 1 ||
      domain.port > 65535
    ) {
      throw new Error(
        `[XyNginC] Configuration error: 'port' must be between 1-65535 for ${domain.domain}`,
      );
    }

    if (domain.ssl && !domain.email) {
      throw new Error(
        `[XyNginC] Configuration error: 'email' is required when SSL is enabled for ${domain.domain}`,
      );
    }

    // Set default host to localhost if not provided
    if (!domain.host) {
      domain.host = "localhost";
    }

    // Set default maxBodySize if not provided
    if (!domain.maxBodySize) {
      domain.maxBodySize = "20M";
    }
  }
}

import { __strl__ } from "strulink";
import { XyNginCConfig } from "../types";
import { Interface } from "reliant-type";

/**
 * Validates the plugin configuration.
 *
 * @param config - The configuration to validate.
 * @throws Error if the configuration is invalid.
 */
export function validateConfig(config: XyNginCConfig): void {
  const d = config.domains;

  if (!d || d.length === 0) {
    throw new Error(
      "[XyNginC] Configuration error: 'domains' array cannot be empty",
    );
  }

  for (const d2 of d) {
    if (!d2.domain || typeof d2.domain !== "string") {
      throw new Error(
        "[XyNginC] Configuration error: 'domain' must be a non-empty string",
      );
    }
    const vd = __strl__.checkUrl(d2.domain, { requireProtocol: false });
    if (!vd.isValid) {
      const EDOMAIN = vd?.validationDetails?.domain;
      if (EDOMAIN) {
        console.error("[XYNGINC::STRULINK:EDOMAIN]: ", EDOMAIN);
        throw new Error("[XYNGINC::STRULINK:EDOMAIN]" + vd?.cause);
      }
      throw new Error(vd?.cause);
    }

    if (
      !d2.port ||
      typeof d2.port !== "number" ||
      d2.port < 1 ||
      d2.port > 65535
    ) {
      throw new Error(
        `[XyNginC] Configuration error: 'port' must be between 1-65535 for ${d2.domain}`,
      );
    }

    if (d2.ssl && !d2.email) {
      throw new Error(
        `[XyNginC] Configuration error: 'email' is required when SSL is enabled for ${d2.domain}`,
      );
    }
    const emailInterface = Interface({
      email: "email",
    });
    if (d2.email) {
      const vmail = emailInterface.safeParse({ email: d2.email });
      if (!vmail.success) {
        console.error("EVMAIL: ", vmail.errors.join("\n"));
        `[XyNginC] Configuration error: The provided email address (${d2.email}) is not valid for the domain "${d2.domain}".`;
      }
    }

    // Set default host to localhost if not provided
    if (!d2.host) {
      d2.host = "localhost";
    }

    // Set default maxBodySize if not provided
    if (!d2.maxBodySize) {
      d2.maxBodySize = "20M";
    }
  }
}

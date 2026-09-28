/**
 * XyPriss Default Server Configuration
 *
 * Configured for a standard, single-instance server operation.
 */

import { __sys__ } from "xypriss";
import { manifest } from "./manifest";
import XNCP from "xynginc";

export const serverConfigs: ServerOptions = {
  plugins: {
    register: [
      XNCP({
        domains: [], // for testing
      }),
    ],
  },
  /**
   * Standard single-server configuration.
   * Configured for high-performance standalone operation.
   */
  server: {
    autoKillConflict: true,
    port: __sys__.vars.__port__,
    serviceName: manifest.name,
    xems: {
      secret: "TEST_12345678901234567890123456789012",
      path: "/tmp/xems",
    },
  },
};

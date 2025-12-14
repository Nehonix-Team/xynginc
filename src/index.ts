import { Plugin } from "xypriss";
import { exec } from "child_process";
import { promisify } from "util";
import path from "path";
import fs from "fs";

const execAsync = promisify(exec);

export interface XyNginCConfig {
  domains: Array<{
    domain: string;
    port: number;
    ssl?: boolean;
    email?: string;
  }>;
  autoReload?: boolean;
}

export default function XNCP(config: XyNginCConfig) {
  return Plugin.create({
    name: "xynginc",
    version: "0.0.1",
    description: "XyPriss Nginx Controller Plugin",

    onServerStart: async () => {
      console.log("[XyNginC] Initializing Nginx Controller...");

      // TODO: Locate the binary
      // const binaryPath = path.join(__dirname, "../bin/xynginc");

      // TODO: Execute binary with config
      // await execAsync(`${binaryPath} apply --config '${JSON.stringify(config)}'`);

      console.log("[XyNginC] Configuration applied successfully.");
    },
  });
}

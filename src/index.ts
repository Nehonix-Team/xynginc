import { Plugin } from "xypriss";
import { exec } from "child_process";
import { promisify } from "util";
import path from "path";
import fs from "fs";
import os from "os";
import https from "https";

const execAsync = promisify(exec);

export interface XyNginCDomainConfig {
  domain: string;
  port: number;
  ssl?: boolean;
  email?: string;
}

export interface XyNginCConfig {
  domains: XyNginCDomainConfig[];
  autoReload?: boolean;
}

interface XyNginCPluginOptions extends XyNginCConfig {
  /** Path to the xynginc binary (auto-detected if not provided) */
  binaryPath?: string;
  /** Auto-download binary if not found (default: true) */
  autoDownload?: boolean;
  /** GitHub release version to download (default: "latest") */
  version?: string;
}

const BINARY_NAME = "xynginc";
const GITHUB_REPO = "Nehonix-Team/xynginc";
const BINARY_DIR = path.join(__dirname, "../bin");

export default function XNCP(options: XyNginCPluginOptions) {
  const {
    domains,
    autoReload = true,
    binaryPath,
    autoDownload = true,
    version = "latest",
  } = options;

  return Plugin.create({
    name: "xynginc",
    version: "1.0.0",
    description: "XyPriss Nginx Controller - Automatic Nginx & SSL management",

    onRegister: async () => {
      console.log("[XyNginC] 🔧 Registering plugin...");

      // Validate config
      validateConfig({ domains, autoReload });
    },

    onServerStart: async (server) => {
      console.log("[XyNginC] 🚀 Initializing Nginx Controller...");

      try {
        // 1. Ensure binary exists
        const binary = await ensureBinary(binaryPath, autoDownload, version);
        console.log(`[XyNginC] ✓ Binary located: ${binary}`);

        // 2. Check system requirements
        console.log("[XyNginC] 🔍 Checking system requirements...");
        await checkRequirements(binary);

        // 3. Apply configuration
        console.log("[XyNginC] 📋 Applying configuration...");
        await applyConfig(binary, { domains, auto_reload: autoReload });

        console.log("[XyNginC] ✅ Configuration applied successfully!");

        // Expose CLI helper methods on server
        server.xynginc = {
          addDomain: (
            domain: string,
            port: number,
            ssl = false,
            email?: string
          ) => addDomain(binary, domain, port, ssl, email),
          removeDomain: (domain: string) => removeDomain(binary, domain),
          listDomains: () => listDomains(binary),
          reload: () => reloadNginx(binary),
          test: () => testNginx(binary),
          status: () => getStatus(binary),
        };

        console.log("[XyNginC] 💡 Server methods available: server.xynginc.*");
      } catch (error) {
        console.error("[XyNginC] ❌ Failed to initialize:", error);
        throw error;
      }
    },

    onServerStop: async () => {
      console.log("[XyNginC] 👋 Shutting down Nginx Controller...");
    },
  });
}

/**
 * Validate the plugin configuration
 */
function validateConfig(config: XyNginCConfig): void {
  if (!config.domains || config.domains.length === 0) {
    throw new Error(
      "[XyNginC] Configuration error: 'domains' array cannot be empty"
    );
  }

  for (const domain of config.domains) {
    if (!domain.domain || typeof domain.domain !== "string") {
      throw new Error(
        "[XyNginC] Configuration error: 'domain' must be a non-empty string"
      );
    }

    if (
      !domain.port ||
      typeof domain.port !== "number" ||
      domain.port < 1 ||
      domain.port > 65535
    ) {
      throw new Error(
        `[XyNginC] Configuration error: 'port' must be between 1-65535 for ${domain.domain}`
      );
    }

    if (domain.ssl && !domain.email) {
      throw new Error(
        `[XyNginC] Configuration error: 'email' is required when SSL is enabled for ${domain.domain}`
      );
    }
  }
}

/**
 * Ensure the binary exists (locate or download)
 */
async function ensureBinary(
  customPath: string | undefined,
  autoDownload: boolean,
  version: string
): Promise<string> {
  // 1. Try custom path
  if (customPath && fs.existsSync(customPath)) {
    return customPath;
  }

  // 2. Try PATH
  try {
    const { stdout } = await execAsync("which xynginc");
    const globalPath = stdout.trim();
    if (globalPath && fs.existsSync(globalPath)) {
      return globalPath;
    }
  } catch {
    // Not in PATH
  }

  // 3. Try local bin directory
  const localPath = path.join(BINARY_DIR, BINARY_NAME);
  if (fs.existsSync(localPath)) {
    return localPath;
  }

  // 4. Auto-download if enabled
  if (autoDownload) {
    console.log("[XyNginC] 📥 Binary not found, downloading...");
    return await downloadBinary(version);
  }

  throw new Error(
    "[XyNginC] Binary not found. Install xynginc or set 'autoDownload: true'"
  );
}

/**
 * Download the binary from GitHub releases
 */
async function downloadBinary(version: string): Promise<string> {
  const platform = os.platform();
  const arch = os.arch();

  if (platform !== "linux") {
    throw new Error(
      `[XyNginC] Unsupported platform: ${platform}. Only Linux is supported.`
    );
  }

  const binaryName = `${BINARY_NAME}-${platform}-${arch}`;
  const downloadUrl =
    version === "latest"
      ? `https://github.com/${GITHUB_REPO}/releases/latest/download/${binaryName}`
      : `https://github.com/${GITHUB_REPO}/releases/download/${version}/${binaryName}`;

  console.log(`[XyNginC] 📦 Downloading from: ${downloadUrl}`);

  // Create bin directory
  if (!fs.existsSync(BINARY_DIR)) {
    fs.mkdirSync(BINARY_DIR, { recursive: true });
  }

  const localPath = path.join(BINARY_DIR, BINARY_NAME);

  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(localPath);

    https
      .get(downloadUrl, (response) => {
        if (response.statusCode === 302 || response.statusCode === 301) {
          // Follow redirect
          https
            .get(response.headers.location!, (redirectResponse) => {
              redirectResponse.pipe(file);
              file.on("finish", () => {
                file.close();
                fs.chmodSync(localPath, 0o755); // Make executable
                console.log("[XyNginC] ✓ Binary downloaded successfully");
                resolve(localPath);
              });
            })
            .on("error", reject);
        } else {
          response.pipe(file);
          file.on("finish", () => {
            file.close();
            fs.chmodSync(localPath, 0o755); // Make executable
            console.log("[XyNginC] ✓ Binary downloaded successfully");
            resolve(localPath);
          });
        }
      })
      .on("error", (err) => {
        fs.unlinkSync(localPath); // Delete partial file
        reject(new Error(`Failed to download binary: ${err.message}`));
      });
  });
}

/**
 * Check system requirements using the binary
 */
async function checkRequirements(binaryPath: string): Promise<void> {
  try {
    const { stdout, stderr } = await execAsync(`sudo ${binaryPath} check`);
    console.log(stdout);
    if (stderr) console.error(stderr);
  } catch (error: any) {
    throw new Error(`System requirements check failed: ${error.message}`);
  }
}

/**
 * Apply configuration using the binary
 */
async function applyConfig(
  binaryPath: string,
  config: { domains: XyNginCDomainConfig[]; auto_reload: boolean }
): Promise<void> {
  const configJson = JSON.stringify(config);

  try {
    // Pass config via stdin to avoid shell escaping issues
    const { stdout, stderr } = await execAsync(
      `echo '${configJson}' | sudo ${binaryPath} apply --config -`
    );
    console.log(stdout);
    if (stderr) console.error(stderr);
  } catch (error: any) {
    throw new Error(`Failed to apply configuration: ${error.message}`);
  }
}

/**
 * Add a domain using the binary
 */
async function addDomain(
  binaryPath: string,
  domain: string,
  port: number,
  ssl: boolean,
  email?: string
): Promise<void> {
  const sslFlag = ssl ? "--ssl" : "";
  const emailFlag = email ? `--email ${email}` : "";

  try {
    const { stdout } = await execAsync(
      `sudo ${binaryPath} add --domain ${domain} --port ${port} ${sslFlag} ${emailFlag}`
    );
    console.log(stdout);
  } catch (error: any) {
    throw new Error(`Failed to add domain: ${error.message}`);
  }
}

/**
 * Remove a domain using the binary
 */
async function removeDomain(binaryPath: string, domain: string): Promise<void> {
  try {
    const { stdout } = await execAsync(`sudo ${binaryPath} remove ${domain}`);
    console.log(stdout);
  } catch (error: any) {
    throw new Error(`Failed to remove domain: ${error.message}`);
  }
}

/**
 * List all configured domains
 */
async function listDomains(binaryPath: string): Promise<string[]> {
  try {
    const { stdout } = await execAsync(`sudo ${binaryPath} list`);
    // Parse output to extract domain names
    const lines = stdout.split("\n").filter((line) => line.includes(" - "));
    return lines.map((line) => line.trim().split(" - ")[0]);
  } catch (error: any) {
    throw new Error(`Failed to list domains: ${error.message}`);
  }
}

/**
 * Reload Nginx
 */
async function reloadNginx(binaryPath: string): Promise<void> {
  try {
    const { stdout } = await execAsync(`sudo ${binaryPath} reload`);
    console.log(stdout);
  } catch (error: any) {
    throw new Error(`Failed to reload Nginx: ${error.message}`);
  }
}

/**
 * Test Nginx configuration
 */
async function testNginx(binaryPath: string): Promise<boolean> {
  try {
    await execAsync(`sudo ${binaryPath} test`);
    return true;
  } catch {
    return false;
  }
}

/**
 * Get status
 */
async function getStatus(binaryPath: string): Promise<string> {
  try {
    const { stdout } = await execAsync(`sudo ${binaryPath} status`);
    return stdout;
  } catch (error: any) {
    throw new Error(`Failed to get status: ${error.message}`);
  }
}

// Re-export types
export type { XyNginCPluginOptions };

// Named exports for direct usage
export const XyNginC = XNCP;

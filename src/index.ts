/*
 * This code contains proprietary source code from NEHONIX
 * Copyright © 2025 NEHONIX - www.nehonix.com
 * Licensed under NEHONIX Open Source License (NOSL) v1.0
 */

import { Plugin } from "xypriss";
import { exec, spawn } from "child_process";
import { promisify } from "util";
import path from "path";
import fs from "fs";
import os from "os";
import https from "https";
import { Logger } from "./logger";
import {
  XyNginCConfig,
  XyNginCDomainConfig,
  XyNginCPluginOptions,
} from "./types";

const execAsync = promisify(exec);

const BINARY_BASE_NAME = "xynginc";
const GITHUB_REPO = "Nehonix-Team/xynginc";
const BINARY_DIR = path.join(__dirname, "../bin");

function getBinaryName(): string {
  const platform = os.platform();
  const arch = os.arch();
  const ext = platform === "win32" ? ".exe" : "";
  return `${BINARY_BASE_NAME}-${platform}-${arch}${ext}`;
}

/**
 * XyNginC Plugin for XyPriss.
 * Automates Nginx and SSL management for your server.
 *
 * @param options - Plugin configuration options.
 * @returns A XyPriss Plugin instance.
 */
export default function XNCP(options: XyNginCPluginOptions) {
  const {
    domains,
    autoReload = true,
    binaryPath,
    autoDownload = true,
    version = "latest",
    installRequirements = true,
    sudoPassword,
  } = options;

  const getSudo = () => {
    const pwd = sudoPassword || process.env.SUDO_PASSWORD;
    if (pwd) {
      return `echo '${pwd}' | sudo -S`;
    }
    return "sudo";
  };

  return Plugin.create({
    name: "xynginc",
    version: "1.0.7",
    description: "XyPriss Nginx Controller - Automatic Nginx & SSL management",

    onRegister: async () => {
      Logger.info("[XyNginC] Registering plugin...");

      // Validate config
      validateConfig({ domains, autoReload });
    },

    onServerStart: async (server) => {
      Logger.info("[XyNginC] Initializing Nginx Controller...");

      try {
        // 1. Ensure binary exists
        const binary = await ensureBinary(binaryPath, autoDownload, version);
        Logger.success(`[XyNginC] ✓ Binary located: ${binary}`);

        // 2. Check system requirements
        Logger.info("[XyNginC] Checking system requirements...");

        // Check if requirements are satisfied
        const requirementsOk = await checkRequirements(binary, getSudo());

        // Install requirements if enabled and needed
        if (!requirementsOk && installRequirements) {
          Logger.info(
            "[XyNginC] Requirements missing, installing automatically...",
          );
          await installRequirementsHandler(binary, getSudo());
          Logger.info("[XyNginC] Requirements installed, re-checking...");
          await checkRequirements(binary, getSudo());
        } else if (!requirementsOk) {
          throw new Error(
            "[XyNginC] System requirements not satisfied. Install with 'installRequirements: true' or run: sudo xynginc install",
          );
        }

        // 3. Apply configuration
        Logger.info("[XyNginC] Applying configuration...");
        await applyConfig(
          binary,
          { domains, auto_reload: autoReload },
          getSudo(),
        );

        Logger.success("[XyNginC] Configuration applied successfully!");

        // Expose CLI helper methods on server
        server.xynginc = {
          addDomain: (
            domain: string,
            port: number,
            ssl = false,
            email?: string,
            maxBodySize?: string,
          ) =>
            addDomain(binary, domain, port, ssl, email, maxBodySize, getSudo()),
          removeDomain: (domain: string) =>
            removeDomain(binary, domain, getSudo()),
          listDomains: () => listDomains(binary, getSudo()),
          reload: () => reloadNginx(binary, getSudo()),
          test: () => testNginx(binary, getSudo()),
          status: () => getStatus(binary, getSudo()),
          installRequirements: () =>
            installRequirementsHandler(binary, getSudo()),
        };

        Logger.info("[XyNginC] Server methods available: server.xynginc.*");
      } catch (error) {
        Logger.error(`[XyNginC] ✖ Failed to initialize: ${error}`);
        throw error;
      }
    },

    onServerStop: async () => {
      Logger.info("[XyNginC] Shutting down Nginx Controller...");
    },
  });
}

/**
 * Validates the plugin configuration.
 *
 * @param config - The configuration to validate.
 * @throws Error if the configuration is invalid.
 */
function validateConfig(config: XyNginCConfig): void {
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

/**
 * Ensures the xynginc binary exists.
 * It checks the custom path, the system PATH, and the local bin directory.
 * If not found and autoDownload is enabled, it downloads the binary.
 *
 * @param customPath - Optional custom path to the binary.
 * @param autoDownload - Whether to download the binary if missing.
 * @param version - The version to download.
 * @returns The absolute path to the binary.
 */
async function ensureBinary(
  customPath: string | undefined,
  autoDownload: boolean,
  version: string,
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
  const localPath = path.join(BINARY_DIR, getBinaryName());
  if (fs.existsSync(localPath)) {
    return localPath;
  }

  // Also try default un-suffixed binary if someone manually placed it
  const defaultLocalPath = path.join(BINARY_DIR, "xynginc");
  if (fs.existsSync(defaultLocalPath)) {
    return defaultLocalPath;
  }

  // 4. Auto-download if enabled
  if (autoDownload) {
    Logger.info("[XyNginC] Binary not found, downloading...");
    return await downloadBinary(version);
  }

  throw new Error(
    "[XyNginC] Binary not found. Install xynginc or set 'autoDownload: true'",
  );
}

/**
 * Downloads the xynginc binary from GitHub releases.
 *
 * @param version - The version to download (e.g., "latest" or "v1.4.5").
 * @returns The path to the downloaded binary.
 */
async function downloadBinary(version: string): Promise<string> {
  const platform = os.platform();

  if (platform !== "linux" && platform !== "win32" && platform !== "darwin") {
    Logger.warn(
      `[XyNginC] Unusual platform detected: ${platform}. Attempting to download anyway.`,
    );
  }

  const binaryName = getBinaryName();
  const downloadUrl =
    version === "latest"
      ? `https://github.com/${GITHUB_REPO}/releases/latest/download/${binaryName}`
      : `https://github.com/${GITHUB_REPO}/releases/download/${version}/${binaryName}`;

  Logger.info(`[XyNginC] Downloading from: ${downloadUrl}`);

  // Create bin directory
  if (!fs.existsSync(BINARY_DIR)) {
    fs.mkdirSync(BINARY_DIR, { recursive: true });
  }

  const localPath = path.join(BINARY_DIR, binaryName);

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
                Logger.success("[XyNginC] ✓ Binary downloaded successfully");
                resolve(localPath);
              });
            })
            .on("error", reject);
        } else {
          response.pipe(file);
          file.on("finish", () => {
            file.close();
            fs.chmodSync(localPath, 0o755); // Make executable
            Logger.success("[XyNginC] ✓ Binary downloaded successfully");
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
 * Checks if system requirements (Nginx, Certbot) are satisfied using the binary.
 *
 * @param binaryPath - Path to the xynginc binary.
 * @param sudoCmd - Sudo prefix to use.
 * @returns True if requirements are met, false otherwise.
 */
async function checkRequirements(
  binaryPath: string,
  sudoCmd: string,
): Promise<boolean> {
  try {
    const { stdout, stderr } = await execAsync(
      `${sudoCmd} ${binaryPath} check`,
    );
    Logger.info(stdout.trim());
    if (stderr) Logger.error(stderr.trim());
    return true;
  } catch (error: any) {
    Logger.warn(`[XyNginC] System requirements check failed: ${error.message}`);
    return false;
  }
}

/**
 * Installs system requirements using the binary in interactive mode.
 *
 * @param binaryPath - Path to the xynginc binary.
 * @param sudoCmd - Sudo prefix to use.
 */
async function installRequirementsHandler(
  binaryPath: string,
  sudoCmd: string,
): Promise<void> {
  return new Promise((resolve, reject) => {
    Logger.info("[XyNginC] Launching interactive installer...");
    Logger.info("[XyNginC] Please respond to any prompts in the terminal.");

    // Handle process environmental logic if running via non-interactive sudo -S
    const cmd = sudoCmd.includes("-S")
      ? `${sudoCmd} ${binaryPath} install`
      : `sudo ${binaryPath} install`;

    // Spawn the process with inherited stdio for full interactivity
    const installProcess = spawn(cmd, {
      stdio: "inherit", // This allows the subprocess to use the parent's stdin/stdout/stderr
      shell: true,
    });

    installProcess.on("close", (code) => {
      if (code === 0) {
        Logger.success(
          "[XyNginC] ✓ System requirements installed successfully",
        );
        resolve();
      } else {
        reject(new Error(`Installation failed with exit code ${code}`));
      }
    });

    installProcess.on("error", (error) => {
      reject(new Error(`Failed to start installation: ${error.message}`));
    });
  });
}

/**
 * Applies the configuration using the xynginc binary.
 *
 * @param binaryPath - Path to the xynginc binary.
 * @param config - The configuration object.
 * @param sudoCmd - Sudo prefix to use.
 */
async function applyConfig(
  binaryPath: string,
  config: { domains: XyNginCDomainConfig[]; auto_reload: boolean },
  sudoCmd: string,
): Promise<void> {
  // Map camelCase to snake_case for Rust core
  const mappedConfig = {
    auto_reload: config.auto_reload,
    domains: config.domains.map((d) => ({
      domain: d.domain,
      port: d.port,
      ssl: d.ssl,
      email: d.email,
      host: d.host,
      max_body_size: d.maxBodySize,
    })),
  };

  const configJson = JSON.stringify(mappedConfig);

  try {
    // Test nginx BEFORE applying new config
    Logger.info("[XyNginC] Testing current nginx config...");
    const testResult = await testNginx(binaryPath, sudoCmd);
    if (!testResult) {
      Logger.warn(
        "[XyNginC] ⚠️  Current nginx config has errors. Attempting to fix...",
      );
    }

    // Pass config via stdin to avoid shell escaping issues
    const { stdout, stderr } = await execAsync(
      `echo '${configJson}' | ${sudoCmd} ${binaryPath} apply --config -`,
    );
    Logger.info(stdout.trim());
    if (stderr) Logger.error(stderr.trim());
  } catch (error: any) {
    // If it fails, show more helpful error
    Logger.error(`[XyNginC] Failed to apply configuration: ${error.message}`);
    Logger.info("[XyNginC] Try running: sudo nginx -t");
    Logger.info("[XyNginC] Check: /etc/nginx/sites-enabled/");
    throw new Error(`Failed to apply configuration: ${error.message}`);
  }
}

/**
 * Adds a new domain configuration using the binary.
 *
 * @param binaryPath - Path to the xynginc binary.
 * @param domain - The domain name.
 * @param port - The backend port.
 * @param ssl - Whether to enable SSL.
 * @param email - Optional email for SSL.
 * @param maxBodySize - Optional maximum body size.
 * @param sudoCmd - Sudo prefix to use.
 */
async function addDomain(
  binaryPath: string,
  domain: string,
  port: number,
  ssl: boolean,
  email?: string,
  maxBodySize?: string,
  sudoCmd: string = "sudo",
): Promise<void> {
  const sslFlag = ssl ? "--ssl" : "";
  const emailFlag = email ? `--email ${email}` : "";
  const maxBodySizeFlag = maxBodySize ? `--max-body-size ${maxBodySize}` : "";

  try {
    const { stdout } = await execAsync(
      `${sudoCmd} ${binaryPath} add --domain ${domain} --port ${port} ${sslFlag} ${emailFlag} ${maxBodySizeFlag}`,
    );
    Logger.info(stdout.trim());
  } catch (error: any) {
    throw new Error(`Failed to add domain: ${error.message}`);
  }
}

/**
 * Removes a domain configuration using the binary.
 *
 * @param binaryPath - Path to the xynginc binary.
 * @param domain - The domain name to remove.
 * @param sudoCmd - Sudo prefix to use.
 */
async function removeDomain(
  binaryPath: string,
  domain: string,
  sudoCmd: string = "sudo",
): Promise<void> {
  try {
    const { stdout } = await execAsync(
      `${sudoCmd} ${binaryPath} remove ${domain}`,
    );
    Logger.info(stdout.trim());
  } catch (error: any) {
    throw new Error(`Failed to remove domain: ${error.message}`);
  }
}

/**
 * Lists all configured domains.
 *
 * @param binaryPath - Path to the xynginc binary.
 * @param sudoCmd - Sudo prefix to use.
 * @returns A list of domain names.
 */
async function listDomains(
  binaryPath: string,
  sudoCmd: string = "sudo",
): Promise<string[]> {
  try {
    const { stdout } = await execAsync(`${sudoCmd} ${binaryPath} list`);
    // Parse output to extract domain names
    const lines = stdout.split("\n").filter((line) => line.includes(" - "));
    return lines.map((line) => line.trim().split(" - ")[0]);
  } catch (error: any) {
    throw new Error(`Failed to list domains: ${error.message}`);
  }
}

/**
 * Reloads the Nginx service using the binary.
 *
 * @param binaryPath - Path to the xynginc binary.
 * @param sudoCmd - Sudo prefix to use.
 */
async function reloadNginx(
  binaryPath: string,
  sudoCmd: string = "sudo",
): Promise<void> {
  try {
    const { stdout } = await execAsync(`${sudoCmd} ${binaryPath} reload`);
    Logger.info(stdout.trim());
  } catch (error: any) {
    throw new Error(`Failed to reload Nginx: ${error.message}`);
  }
}

/**
 * Tests the Nginx configuration validity using the binary.
 *
 * @param binaryPath - Path to the xynginc binary.
 * @param sudoCmd - Sudo prefix to use.
 * @returns True if the configuration is valid.
 */
async function testNginx(
  binaryPath: string,
  sudoCmd: string = "sudo",
): Promise<boolean> {
  try {
    await execAsync(`${sudoCmd} ${binaryPath} test`);
    return true;
  } catch {
    return false;
  }
}

/**
 * Gets the status of managed sites using the binary.
 *
 * @param binaryPath - Path to the xynginc binary.
 * @param sudoCmd - Sudo prefix to use.
 * @returns The status output.
 */
async function getStatus(
  binaryPath: string,
  sudoCmd: string = "sudo",
): Promise<string> {
  try {
    const { stdout } = await execAsync(`${sudoCmd} ${binaryPath} status`);
    return stdout;
  } catch (error: any) {
    throw new Error(`Failed to get status: ${error.message}`);
  }
}

// Re-export types
export type { XyNginCConfig, XyNginCDomainConfig, XyNginCPluginOptions };

// Named exports for direct usage
export const XyNginC = XNCP;

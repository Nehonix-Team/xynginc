import { Logger } from "./logger";
import { spawn } from "child_process";
import { XyNginCDomainConfig } from "./types";
import { execAsync } from "./execAsync";
import { XStringify } from "xypriss-security";

/**
 * Installs system requirements using the binary in interactive mode.
 *
 * @param binaryPath - Path to the xynginc binary.
 * @param sudoCmd - Sudo prefix to use.
 */
export async function installRequirementsHandler(
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
export async function applyConfig(
  binaryPath: string,
  config: {
    domains: XyNginCDomainConfig[];
    auto_reload: boolean;
    auto_fix_firewall: boolean;
  },
  sudoCmd: string,
): Promise<void> {
  // Map camelCase to snake_case for Go core
  const mappedConfig = {
    auto_reload: config.auto_reload,
    autofix_firewall: config.auto_fix_firewall,
    domains: config.domains.map((d) => ({
      domain: d.domain,
      port: d.port,
      ssl: d.ssl,
      email: d.email,
      host: d.host,
      max_body_size: d.maxBodySize,
    })),
  };

  const configJson = XStringify(mappedConfig, {
    pureRaw: true,
    maxDepth: 100,
    maxLength: 2000000, // 2MB limit
    truncateStrings: 1000000, // 1MB limit per string (pour le HTML)
    reportCircularPath: true,
  });

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
    // If using sudo -S, we must pass both the password and the JSON in the same pipe
    if (sudoCmd.includes("-S")) {
      const pwdMatch = sudoCmd.match(/echo '(.*)' \| sudo -S/);
      if (pwdMatch) {
        const pwd = pwdMatch[1];
        await execStream(
          `(echo '${pwd}'; echo '${configJson}') | sudo -S ${binaryPath} apply --config -`,
        );
      } else {
        await execStream(
          `echo '${configJson}' | ${sudoCmd} ${binaryPath} apply --config -`,
        );
      }
    } else {
      await execStream(
        `echo '${configJson}' | ${sudoCmd} ${binaryPath} apply --config -`,
      );
    }
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
export async function addDomain(
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
export async function removeDomain(
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
export async function listDomains(
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
export async function reloadNginx(
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
export async function testNginx(
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
export async function getStatus(
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

/**
 * Checks if system requirements (Nginx, Certbot) are satisfied using the binary.
 *
 * @param binaryPath - Path to the xynginc binary.
 * @param sudoCmd - Sudo prefix to use.
 * @returns True if requirements are met, false otherwise.
 */
export async function checkRequirements(
  binaryPath: string,
  sudoCmd: string,
): Promise<boolean> {
  try {
    Logger.info("[XyNginC] Checking system requirements...");
    const cmd = `${sudoCmd} ${binaryPath} check`;
    Logger.info(`[XyNginC] Running: ${cmd}`);
    await execStream(cmd);
    Logger.info("[XyNginC] System requirements checked successfully!");
    return true;
  } catch (error: any) {
    Logger.warn(`[XyNginC] System requirements check failed: ${error.message}`);
    return false;
  }
}

/**
 * Executes a shell command and streams its stdout/stderr line-by-line in real-time,
 * preserving ANSI color codes printed by the Go binary.
 */
export function execStream(command: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const child = spawn(command, { shell: true });
    let stderrData = "";

    child.stdout.on("data", (data) => {
      // Print raw data so inner ANSI codes from Go are preserved directly in PM2
      process.stdout.write(data);
    });

    child.stderr.on("data", (data) => {
      const msg = data.toString();
      // Filter out the interactive sudo prompt from stderr output
      if (!msg.includes("[sudo] password for")) {
        process.stdout.write(data); // Write to stdout to avoid PM2 red-error highlighting
      }
      stderrData += msg;
    });

    child.on("close", (code) => {
      if (code === 0) resolve();
      else reject(new Error(`Command failed with code ${code}.`));
    });
  });
}

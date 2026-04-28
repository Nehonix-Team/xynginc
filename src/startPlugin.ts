import { PluginServer } from ".";
import { ensureBinary } from "./mods/ensureBinary";
import { Logger } from "./mods/logger";
import {
  addDomain,
  applyConfig,
  checkRequirements,
  getStatus,
  installRequirementsHandler,
  listDomains,
  reloadNginx,
  removeDomain,
  testNginx,
} from "./mods/requirements";
import { XyNginCDomainConfig } from "./types";

const getSudo = (sudoPassword: string) => {
  // Attempt multiple ways to get the password, including XyPriss internal env if somehow exposed
  const envPwd = __sys__.__env__.get("SUDO_PASSWORD");
  // || (global as any).__sys__?.$env?.("SUDO_PASSWORD");
  const pwd = sudoPassword || envPwd;

  if (pwd) {
    if (sudoPassword) {
      Logger.info(
        `[XyNginC] Using sudo password provided via plugin options(${sudoPassword.slice(0, 2)}***).`,
      );
    } else {
      Logger.info(
        "[XyNginC] Using sudo password injected from environment variables.",
      );
    }
    return `echo '${pwd}' | sudo -S`;
  }

  Logger.warn(
    "[XyNginC] ⚠️ No sudo password provided. Falling back to non-interactive mode (sudo -n).",
  );
  Logger.warn(
    "[XyNginC] ⚠️ If the command requires a password, it will fail immediately instead of hanging.",
  );

  // Return non-interactive sudo to prevent infinite blocking/hanging
  return "sudo -n";
};

export async function startXNCPlugin(
  server: PluginServer,
  options: {
    binaryPath?: string;
    autoDownload: boolean;
    version: string;
    domains: XyNginCDomainConfig[];
    autoReload: boolean;
    autoFixFirewall: boolean;
    installRequirements: boolean;
    sudoPassword: string;
  },
) {
  const {
    binaryPath,
    autoDownload,
    version,
    domains,
    autoReload,
    autoFixFirewall,
    installRequirements,
    sudoPassword,
  } = options;
  Logger.info("[XyNginC] Initializing Nginx Controller...");
  if (!sudoPassword) {
    Logger.warn(
      "[XyNginC] To ensure optimal performance and prevent potential issues, it is recommended to provide your system sudo password.",
    );
    Logger.info(
      "[XyNginC] Security notice: The password is used solely for privileged communication with your operating system. It is never stored, logged, or transmitted externally.",
    );
  }

  try {
    // 1. Ensure binary exists
    const binary = await ensureBinary(binaryPath, autoDownload, version);
    Logger.success(`[XyNginC] ✓ Binary located: ${binary}`);

    __sys__.vars.update({
      xynginc: {
        binary,
      },
    });

    // 2. Check system requirements
    Logger.info("[XyNginC] Checking system requirements...");

    // Check if requirements are satisfied
    const requirementsOk = await checkRequirements(
      binary,
      getSudo(sudoPassword),
    );

    // Install requirements if enabled and needed
    if (!requirementsOk && installRequirements) {
      Logger.info(
        "[XyNginC] Requirements missing, installing automatically...",
      );
      await installRequirementsHandler(binary, getSudo(sudoPassword));
      Logger.info("[XyNginC] Requirements installed, re-checking...");
      await checkRequirements(binary, getSudo(sudoPassword));
    } else if (!requirementsOk) {
      throw new Error(
        "[XyNginC] System requirements not satisfied. Install with 'installRequirements: true' or run: sudo xynginc install",
      );
    }

    // 3. Apply configuration
    Logger.info("[XyNginC] Applying configuration...");
    await applyConfig(
      binary,
      {
        domains,
        auto_reload: autoReload,
        auto_fix_firewall: autoFixFirewall,
      },
      getSudo(sudoPassword),
    );

    Logger.success("[XyNginC] Configuration applied successfully!");

    // Expose CLI helper methods on server
    const sUtil = {
      addDomain: (
        domain: string,
        port: number,
        ssl = false,
        email?: string,
        maxBodySize?: string,
      ) =>
        addDomain(
          binary,
          domain,
          port,
          ssl,
          email,
          maxBodySize,
          getSudo(sudoPassword),
        ),
      removeDomain: (domain: string) =>
        removeDomain(binary, domain, getSudo(sudoPassword)),
      listDomains: () => listDomains(binary, getSudo(sudoPassword)),
      reload: () => reloadNginx(binary, getSudo(sudoPassword)),
      test: () => testNginx(binary, getSudo(sudoPassword)),
      status: () => getStatus(binary, getSudo(sudoPassword)),
      installRequirements: () =>
        installRequirementsHandler(binary, getSudo(sudoPassword)),
    };
    (server as any).app.xynginc = sUtil;
    (server as any).xynginc = sUtil;

    Logger.info("[XyNginC] Server methods available: server.xynginc.*");
  } catch (error) {
    Logger.error(`[XyNginC] ✖ Failed to initialize: ${error}`);
    throw error;
  }
}

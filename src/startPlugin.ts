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
import { pkg } from "./pkg";
import { XyNginCDomainConfig } from "./types";


const getSudo = (sudoPassword?: string) => {
  // 1. Direct option passed by the user
  if (sudoPassword) {
    Logger.info(
      `[XyNginC] Using sudo password provided via plugin options(${sudoPassword.slice(0, 2)}***).`,
    );
    return `echo '${sudoPassword}' | sudo -S`;
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

  console.log("root: ", __sys__.plugins.get(pkg.name))

  try {
    // 1. Ensure binary exists
    const binary = await ensureBinary(binaryPath, autoDownload, version);
    Logger.success(`[XyNginC] ✓ Binary located: ${binary}`);

    __sys__.vars.update({
      xynginc: {
        binary,
      },
    });

    // 2. Resolve sudo command asynchronously
    const sudoCmd = await getSudo(sudoPassword);

    // 3. Check system requirements
    Logger.info("[XyNginC] Checking system requirements...");

    // Check if requirements are satisfied
    const requirementsOk = await checkRequirements(
      binary,
      sudoCmd,
    );

    // Install requirements if enabled and needed
    if (!requirementsOk && installRequirements) {
      Logger.info(
        "[XyNginC] Requirements missing, installing automatically...",
      );
      await installRequirementsHandler(binary, sudoCmd);
      Logger.info("[XyNginC] Requirements installed, re-checking...");
      await checkRequirements(binary, sudoCmd);
    } else if (!requirementsOk) {
      throw new Error(
        "[XyNginC] System requirements not satisfied. Install with 'installRequirements: true' or run: sudo xynginc install",
      );
    }

    // 4. Apply configuration
    Logger.info("[XyNginC] Applying configuration...");
    await applyConfig(
      binary,
      {
        domains,
        auto_reload: autoReload,
        auto_fix_firewall: autoFixFirewall,
      },
      sudoCmd,
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
          sudoCmd,
        ),
      removeDomain: (domain: string) =>
        removeDomain(binary, domain, sudoCmd),
      listDomains: () => listDomains(binary, sudoCmd),
      reload: () => reloadNginx(binary, sudoCmd),
      test: () => testNginx(binary, sudoCmd),
      status: () => getStatus(binary, sudoCmd),
      installRequirements: () =>
        installRequirementsHandler(binary, sudoCmd),
    };
    (server as any).app.xynginc = sUtil;
    (server as any).xynginc = sUtil;

    Logger.info("[XyNginC] Server methods available: server.xynginc.*");
  } catch (error) {
    Logger.error(`[XyNginC] ✖ Failed to initialize: ${error}`);
    throw error;
  }
}

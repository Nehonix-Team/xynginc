/*
 * This code contains proprietary source code from NEHONIX
 * Copyright © 2025 NEHONIX - www.nehonix.com
 * Licensed under NEHONIX Open Source License (NOSL) v1.0
 */

import { Plugin } from "xypriss";
import { Logger } from "./mods/logger";
import {
  XyNginCConfig,
  XyNginCDomainConfig,
  XyNginCPluginOptions,
} from "./types";
import { validateConfig } from "./mods/validateConfig";
import { startXNCPlugin } from "./startPlugin";

export type PlC = Parameters<typeof Plugin.create>[0];
export type PluginServer = Parameters<NonNullable<PlC["onServerStart"]>>[0];

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
    autoFixFirewall = false,
    binaryPath,
    autoDownload = true,
    version = "latest",
    installRequirements = true,
    sudoPassword,
  } = options;

  const pkg = Plugin.manifest<{
    name: string;
    version: string;
    description: string;
  }>(__sys__);

  return Plugin.create(
    {
      name: pkg.name,
      version: pkg.version,
      description: pkg.description,

      onRegister: async (_server) => {
        Logger.info("[XyNginC] Registering plugin...");
        validateConfig({ domains, autoReload, autoFixFirewall });
      },
      onServerStart: async (_server: PluginServer) => {
        // Validate config
        await startXNCPlugin(_server, {
          autoDownload,
          autoReload,
          autoFixFirewall,
          binaryPath: binaryPath,
          version,
          domains,
          installRequirements,
          sudoPassword: sudoPassword || "",
        });
      },

      onServerStop: async () => {
        Logger.info("[XyNginC] Shutting down Nginx Controller...");
      },
    },
    __sys__.__root__,
  );
}

// Re-export types
export type { XyNginCConfig, XyNginCDomainConfig, XyNginCPluginOptions };

// Named exports for direct usage
export const XyNginC = XNCP;

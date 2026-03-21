/*
 * This code contains proprietary source code from NEHONIX
 * Copyright © 2025 NEHONIX - www.nehonix.com
 * Licensed under NEHONIX Open Source License (NOSL) v1.0
 */

import { Plugin } from "xypriss";
import { Logger } from "./logger";
import {
  XyNginCConfig,
  XyNginCDomainConfig,
  XyNginCPluginOptions,
} from "./types";
import { validateConfig } from "./validateConfig";
import { startXNCPlugin } from "./startPlugin";

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

  return Plugin.create({
    name: "xynginc",
    version: "1.0.81",
    description: "XyPriss Nginx Controller - Automatic Nginx & SSL management",

    onRegister: async (server) => {
      Logger.info("[XyNginC] Registering plugin...");
      validateConfig({ domains, autoReload, autoFixFirewall });
    },

    onServerStart: async (_server) => {
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
  });
}

// Re-export types
export type { XyNginCConfig, XyNginCDomainConfig, XyNginCPluginOptions };

// Named exports for direct usage
export const XyNginC = XNCP;

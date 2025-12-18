/*
 * This code contains proprietary source code from NEHONIX
 * Copyright © 2025 NEHONIX - www.nehonix.com
 * Licensed under NEHONIX Open Source License (NOSL) v1.0
 */

/**
 * Configuration for a single domain managed by XyNginC.
 */
export interface XyNginCDomainConfig {
  /**
   * The domain name to manage (e.g., "api.example.com").
   */
  domain: string;

  /**
   * The local port where the application is running.
   */
  port: number;

  /**
   * Whether to enable SSL/TLS using Let's Encrypt.
   * @default false
   */
  ssl?: boolean;

  /**
   * The email address for Let's Encrypt notifications.
   * Required if `ssl` is true.
   */
  email?: string;

  /**
   * The backend host to proxy to.
   * @default "localhost"
   */
  host?: string;

  /**
   * Maximum allowed size for client request body (e.g., "20M", "100M").
   * @default "20M"
   */
  maxBodySize?: string;
}

/**
 * Global configuration for the XyNginC plugin.
 */
export interface XyNginCConfig {
  /**
   * List of domains to be managed by Nginx.
   */
  domains: XyNginCDomainConfig[];

  /**
   * Whether to automatically reload Nginx when configuration changes.
   * @default true
   */
  autoReload?: boolean;
}

/**
 * Options for initializing the XyNginC plugin.
 */
export interface XyNginCPluginOptions extends XyNginCConfig {
  /**
   * Custom path to the xynginc binary.
   * If not provided, the plugin will attempt to auto-detect it in the PATH or local bin directory.
   */
  binaryPath?: string;

  /**
   * Whether to automatically download the xynginc binary if it's not found.
   * @default true
   */
  autoDownload?: boolean;

  /**
   * Specific version of the xynginc binary to download from GitHub releases.
   * @default "latest"
   */
  version?: string;

  /**
   * Whether to automatically install system requirements (Nginx, Certbot, etc.) if they are missing.
   * @default true
   */
  installRequirements?: boolean;
}

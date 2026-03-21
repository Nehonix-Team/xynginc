package engine

import (
	"encoding/json"
	"fmt"
	"io"
	"net"
	"os"

	"xynginc/backup"
	"xynginc/logger"
	"xynginc/models"
	"xynginc/ssl"
)

func ApplyConfig(configPath string, noBackup bool, force bool) error {
	logger.Step("> Applying configuration...")

	var configContent []byte
	var err error

	if configPath == "-" {
		logger.Info("> Reading from stdin...")
		configContent, err = io.ReadAll(os.Stdin)
		if err != nil {
			return fmt.Errorf("failed to read stdin: %v", err)
		}
	} else {
		configContent, err = os.ReadFile(configPath)
		if err != nil {
			return fmt.Errorf("failed to read config file: %v", err)
		}
	}

	var config models.Config
	if err := json.Unmarshal(configContent, &config); err != nil {
		return fmt.Errorf("invalid JSON config: %v", err)
	}

	logger.Success(fmt.Sprintf("✓ Config parsed: %d domain(s)", len(config.Domains)))

	if !noBackup {
		logger.Step("\n> Creating backup...")
		if err := backup.CreateBackup(); err != nil {
			return err
		}
	}

	logger.Step("\n> Checking for broken configurations...")
	brokenConfigs, err := detectBrokenConfigs()
	if err != nil {
		return err
	}

	if len(brokenConfigs) > 0 {
		logger.Warning(fmt.Sprintf("⚠️  Found %d broken configuration(s)", len(brokenConfigs)))
		for _, broken := range brokenConfigs {
			logger.Info(fmt.Sprintf("   - %s", broken))
		}

		logger.Step("> Cleaning broken configurations...")
		for _, broken := range brokenConfigs {
			_ = removeConfigFiles(broken)
		}
		logger.Success("✓ Cleanup complete")
	} else {
		logger.Success("✓ No broken configurations found")
	}

	logger.Step("\n> Installing main nginx configuration...")
	if err := ensureNginxMainConfigExists(); err != nil {
		return err
	}

	// NEW: Test and fix nginx EARLY to ensure modules are installed before first reload
	logger.Step("> Verifying nginx modules and configuration...")
	if err := testNginxWithAutofix(); err != nil {
		logger.Warning(fmt.Sprintf("⚠️  Nginx auto-fix failed: %v", err))
		// We continue anyway, maybe the domain loop will fix it or the final test will catch it
	}

	if err := ensureErrorPagesExist(nil); err != nil {
		return err
	}

	for _, domainConfig := range config.Domains {
		logger.Step(fmt.Sprintf("\n🌐 Processing: %s", domainConfig.Domain))

		if configExists(domainConfig.Domain) {
			logger.Info("> Configuration already exists, will be overwritten")
		}

		isIP := net.ParseIP(domainConfig.Domain) != nil

		if domainConfig.SSL {
			if isIP {
				logger.Warning(fmt.Sprintf("⚠️  SSL requested for IP address '%s', but Let's Encrypt does not support IP addresses.", domainConfig.Domain))
				logger.Warning("   Falling back to HTTP for this domain.")

				domainConfig.SSL = false
				if err := generateNginxConfig(&domainConfig); err != nil {
					return err
				}
				if err := enableSite(domainConfig.Domain); err != nil {
					return err
				}
			} else {
				logger.Info("> SSL requested - generating temporary HTTP configuration first")

				tempConfig := domainConfig
				tempConfig.SSL = false

				if err := generateNginxConfig(&tempConfig); err != nil {
					return err
				}
				if err := enableSite(tempConfig.Domain); err != nil {
					return err
				}

				logger.Info("> Reloading nginx for certbot validation...")
				if err := ReloadNginx(); err != nil {
					return err
				}

				if err := ssl.SetupSSL(&domainConfig); err != nil {
					logger.Error(fmt.Sprintf("❌ SSL setup failed for %s: %v", domainConfig.Domain, err))
					logger.Warning("   ⚠️  Falling back to HTTP only for this domain due to SSL error.")

					httpConfig := domainConfig
					httpConfig.SSL = false
					if err := generateNginxConfig(&httpConfig); err != nil {
						return err
					}
					if err := enableSite(httpConfig.Domain); err != nil {
						return err
					}
				} else {
					logger.Info("> Generating final HTTPS configuration...")
					if err := generateNginxConfig(&domainConfig); err != nil {
						return err
					}
					if err := enableSite(domainConfig.Domain); err != nil {
						return err
					}
				}
			}
		} else {
			if err := generateNginxConfig(&domainConfig); err != nil {
				return err
			}
			if err := enableSite(domainConfig.Domain); err != nil {
				return err
			}
		}
	}

	logger.Step("\n> Testing nginx configuration...")
	if err := testNginxWithAutofix(); err == nil {
		logger.Success("✓ Configuration is valid")
	} else {
		if force {
			logger.Warning("⚠️  Configuration test failed but --force is enabled")
			logger.Warning(fmt.Sprintf("   Error: %v", err))
		} else {
			logger.Error("❌ Configuration test failed!")
			logger.Error(fmt.Sprintf("   %v", err))
			logger.Step("\n🔄 Rolling back changes...")

			if !noBackup {
				if rbErr := backup.RestoreLatestBackup(); rbErr != nil {
					return fmt.Errorf("configuration test failed and rollback failed: %v", rbErr)
				}
			}

			return fmt.Errorf("configuration test failed. Changes have been rolled back")
		}
	}

	if config.AutoReload {
		logger.Step("\n🔄 Auto-reload enabled")
		if err := ReloadNginx(); err != nil {
			return err
		}
	}

	logger.Success("\n✅ Configuration applied successfully!")
	return nil
}

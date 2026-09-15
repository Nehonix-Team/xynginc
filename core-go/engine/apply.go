package engine

import (
	"encoding/json"
	"fmt"
	"io"
	"net"
	"os"

	"xynginc/backup"
	"xynginc/firewall"
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

	// NEW: Auto-fix firewall if requested
	if config.AutoFixFirewall {
		if err := firewall.CheckAndFixFirewall(); err != nil {
			logger.Warning(fmt.Sprintf("⚠️  Firewall auto-fix failed: %v", err))
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

	// 1. Group domains needing new SSL certificates
	var sslDomainsToSetup []*models.DomainConfig

	for i := range config.Domains {
		domainConfig := &config.Domains[i]
		isIP := net.ParseIP(domainConfig.Domain) != nil

		if domainConfig.SSL && !isIP {
			certPath := fmt.Sprintf("/etc/letsencrypt/live/%s/fullchain.pem", domainConfig.Domain)
			if _, err := os.Stat(certPath); err != nil {
				sslDomainsToSetup = append(sslDomainsToSetup, domainConfig)
			}
		}
	}

	// 2. If there are domains needing new SSL certs, set up temporary HTTP configs & single reload
	if len(sslDomainsToSetup) > 0 {
		logger.Step(fmt.Sprintf("\n🔒 Preparing SSL setup for %d domain(s)...", len(sslDomainsToSetup)))

		for _, domainConfig := range sslDomainsToSetup {
			tempConfig := *domainConfig
			tempConfig.SSL = false
			if err := generateNginxConfig(&tempConfig); err != nil {
				return err
			}
			if err := enableSite(tempConfig.Domain); err != nil {
				return err
			}
		}

		logger.Info("> Reloading nginx once for certbot validation...")
		if err := ReloadNginx(); err != nil {
			return err
		}

		// Execute grouped SSL setup (1 single certbot call for all domains!)
		_ = ssl.SetupGroupedSSL(sslDomainsToSetup)
	}

	// 3. Generate final Nginx configurations for all domains
	logger.Step("\n🌐 Generating final Nginx domain configurations...")
	for i := range config.Domains {
		domainConfig := config.Domains[i]
		isIP := net.ParseIP(domainConfig.Domain) != nil

		if domainConfig.SSL && !isIP {
			certPath := fmt.Sprintf("/etc/letsencrypt/live/%s/fullchain.pem", domainConfig.Domain)
			if _, err := os.Stat(certPath); err == nil {
				if err := generateNginxConfig(&domainConfig); err != nil {
					return err
				}
			} else {
				logger.Warning(fmt.Sprintf("⚠️  Falling back to HTTP only for %s (SSL cert not available)", domainConfig.Domain))
				domainConfig.SSL = false
				if err := generateNginxConfig(&domainConfig); err != nil {
					return err
				}
			}
		} else {
			if domainConfig.SSL && isIP {
				logger.Warning(fmt.Sprintf("⚠️  SSL requested for IP address '%s', falling back to HTTP.", domainConfig.Domain))
				domainConfig.SSL = false
			}
			if err := generateNginxConfig(&domainConfig); err != nil {
				return err
			}
		}

		if err := enableSite(domainConfig.Domain); err != nil {
			return err
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

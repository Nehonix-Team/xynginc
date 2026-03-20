package engine

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/fatih/color"

	"xynginc/backup"
	"xynginc/constants"
	"xynginc/logger"
	"xynginc/models"
	"xynginc/ssl"
)

func ListDomains() error {
	logger.Step("Configured domains:\n")

	entries, err := os.ReadDir(constants.NginxSitesAvailable)
	if err != nil {
		return fmt.Errorf("failed to read sites-available: %v", err)
	}

	count := 0
	for _, entry := range entries {
		name := entry.Name()

		if name != "default" {
			enabledPath := filepath.Join(constants.NginxSitesEnabled, name)
			enabled := false
			if _, err := os.Stat(enabledPath); !os.IsNotExist(err) {
				enabled = true
			}

			status := color.New(color.Reset).Sprint("◯ disabled")
			if enabled {
				status = color.New(color.FgGreen).Sprint("✓ enabled")
			}
			fmt.Printf("   %s - %s\n", name, status)
			count++
		}
	}

	if count == 0 {
		logger.Info("   (no domains configured)")
	}

	return nil
}

func AddDomain(domain string, port uint16, useSSL bool, email *string, host *string, maxBodySize *string) error {
	if useSSL && email == nil {
		return fmt.Errorf("email is required when SSL is enabled")
	}

	cfgEmail := ""
	if email != nil {
		cfgEmail = *email
	}

	cfgHost := "localhost"
	if host != nil {
		cfgHost = *host
	}

	cfgMaxBodySize := "20M"
	if maxBodySize != nil {
		cfgMaxBodySize = *maxBodySize
	}

	config := &models.DomainConfig{
		Domain:      domain,
		Port:        port,
		SSL:         useSSL,
		Email:       cfgEmail,
		Host:        cfgHost,
		MaxBodySize: cfgMaxBodySize,
	}

	logger.Step(fmt.Sprintf("Adding domain: %s", domain))

	if err := backup.CreateBackup(); err != nil {
		return err
	}

	if err := generateNginxConfig(config); err != nil {
		return err
	}
	if err := enableSite(domain); err != nil {
		return err
	}

	if useSSL {
		if err := ssl.SetupSSL(config); err != nil {
			return err
		}
	}

	if err := TestNginx(); err != nil {
		return err
	}
	if err := ReloadNginx(); err != nil {
		return err
	}

	logger.Success(fmt.Sprintf("✅ Domain %s added successfully!", domain))
	return nil
}

func RemoveDomain(domain string) error {
	logger.Step(fmt.Sprintf("Removing domain: %s", domain))

	if err := backup.CreateBackup(); err != nil {
		return err
	}

	if err := removeConfigFiles(domain); err != nil {
		return err
	}

	if err := TestNginx(); err != nil {
		return err
	}
	if err := ReloadNginx(); err != nil {
		return err
	}

	logger.Success(fmt.Sprintf("✅ Domain %s removed successfully!", domain))
	return nil
}

func enableSite(domain string) error {
	availablePath := filepath.Join(constants.NginxSitesAvailable, domain)
	enabledPath := filepath.Join(constants.NginxSitesEnabled, domain)

	if _, err := os.Stat(enabledPath); !os.IsNotExist(err) {
		if err := os.Remove(enabledPath); err != nil {
			return fmt.Errorf("failed to remove existing symlink: %v", err)
		}
	}

	if err := os.Symlink(availablePath, enabledPath); err != nil {
		return fmt.Errorf("failed to create symlink: %v", err)
	}

	logger.Success("   ✓ Site enabled")
	return nil
}

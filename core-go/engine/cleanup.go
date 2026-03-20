package engine

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	"xynginc/constants"
	"xynginc/logger"
)

func detectBrokenConfigs() ([]string, error) {
	var broken []string

	cmd := exec.Command("nginx", "-t")
	out, err := cmd.CombinedOutput()
	if err != nil {
		stderr := string(out)

		for _, line := range strings.Split(stderr, "\n") {
			if strings.Contains(line, "in /etc/nginx/sites-enabled/") {
				start := strings.Index(line, "/etc/nginx/sites-enabled/")
				if start != -1 {
					domainEnd := strings.Index(line[start+25:], ":")
					if domainEnd != -1 {
						domain := line[start+25 : start+25+domainEnd]
						if !contains(broken, domain) {
							broken = append(broken, domain)
						}
					}
				}
			} else if strings.Contains(line, "cannot load certificate") {
				start := strings.Index(line, "/etc/letsencrypt/live/")
				if start != -1 {
					end := strings.Index(line[start+22:], "/")
					if end != -1 {
						domain := line[start+22 : start+22+end]
						configPath := filepath.Join(constants.NginxSitesEnabled, domain)
						if _, err := os.Stat(configPath); !os.IsNotExist(err) && !contains(broken, domain) {
							broken = append(broken, domain)
						}
					}
				}
			}
		}
	}

	return broken, nil
}

func CleanBrokenConfigs(dryRun bool) error {
	logger.Step("🧹 Cleaning broken configurations...\n")

	broken, err := detectBrokenConfigs()
	if err != nil {
		return err
	}

	if len(broken) == 0 {
		logger.Success("✓ No broken configurations found")
		return nil
	}

	logger.Warning(fmt.Sprintf("Found %d broken configuration(s):", len(broken)))
	for _, domain := range broken {
		logger.Info(fmt.Sprintf("   - %s", domain))
	}

	if dryRun {
		logger.Warning("\nDry run mode: no changes made")
		return nil
	}

	logger.Step("\n>  Removing broken configurations...")
	for _, domain := range broken {
		if err := removeConfigFiles(domain); err != nil {
			logger.Warning(fmt.Sprintf("   ⚠️  Failed to remove %s: %v", domain, err))
		} else {
			logger.Success(fmt.Sprintf("   ✓ Removed: %s", domain))
		}
	}

	logger.Success("\n✅ Cleanup complete!")
	return nil
}

func removeConfigFiles(domain string) error {
	availablePath := filepath.Join(constants.NginxSitesAvailable, domain)
	enabledPath := filepath.Join(constants.NginxSitesEnabled, domain)

	if _, err := os.Stat(enabledPath); !os.IsNotExist(err) {
		if err := os.Remove(enabledPath); err != nil {
			return fmt.Errorf("failed to remove symlink: %v", err)
		}
	}

	if _, err := os.Stat(availablePath); !os.IsNotExist(err) {
		if err := os.Remove(availablePath); err != nil {
			return fmt.Errorf("failed to remove config: %v", err)
		}
	}

	return nil
}

func contains(slice []string, item string) bool {
	for _, a := range slice {
		if a == item {
			return true
		}
	}
	return false
}

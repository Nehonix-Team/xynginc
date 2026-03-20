package ssl

import (
	"fmt"
	"os"
	"os/exec"
	"strings"

	"xynginc/logger"
	"xynginc/models"
)

func checkCertbotNginxPlugin() bool {
	cmd := exec.Command("certbot", "plugins", "--text")
	out, err := cmd.CombinedOutput()
	if err == nil {
		pluginsText := string(out)
		return strings.Contains(pluginsText, "nginx") || strings.Contains(pluginsText, "* nginx")
	}
	return false
}

func installCertbotNginxPlugin() error {
	logger.Warning("⚠️  Certbot nginx plugin not found. Installing...")

	cmd := exec.Command("apt-get", "install", "-y", "python3-certbot-nginx")
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	err := cmd.Run()
	if err != nil {
		return fmt.Errorf("failed to install python3-certbot-nginx package: %v", err)
	}

	logger.Success("✓ Certbot nginx plugin installed")
	return nil
}

func SetupSSL(config *models.DomainConfig) error {
	logger.Step(fmt.Sprintf("> Setting up SSL for %s...", config.Domain))

	if config.Email == "" {
		return fmt.Errorf("email required for SSL")
	}

	if !checkCertbotNginxPlugin() {
		if err := installCertbotNginxPlugin(); err != nil {
			return err
		}
	}

	args := []string{
		"certonly",
		"--nginx",
		"-d", config.Domain,
		"--email", config.Email,
		"--agree-tos",
		"--non-interactive",
	}

	cmd := exec.Command("certbot", args...)
	out, err := cmd.CombinedOutput()
	if err != nil {
		stderrText := string(out)

		if strings.Contains(stderrText, "does not appear to be installed") || strings.Contains(stderrText, "nginx plugin") {
			logger.Warning("⚠️  Certbot nginx plugin error detected. Attempting to fix...")
			if err := installCertbotNginxPlugin(); err != nil {
				return err
			}

			logger.Step("> Retrying SSL certificate request...")
			retryCmd := exec.Command("certbot", args...)
			retryOut, retryErr := retryCmd.CombinedOutput()
			if retryErr != nil {
				return fmt.Errorf("certbot failed after plugin installation:\n%s", string(retryOut))
			}
		} else {
			return fmt.Errorf("certbot failed:\n%s", stderrText)
		}
	}

	logger.Success("✓ SSL certificate obtained")
	return nil
}

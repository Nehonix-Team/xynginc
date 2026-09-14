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

	certPath := fmt.Sprintf("/etc/letsencrypt/live/%s/fullchain.pem", config.Domain)
	if _, err := os.Stat(certPath); err == nil {
		logger.Success(fmt.Sprintf("✓ SSL certificate already exists for %s", config.Domain))
		return nil
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
		"--agree-tos",
		"--non-interactive",
	}

	if config.Email != "" {
		args = append(args, "--email", config.Email)
	} else {
		args = append(args, "--register-unsafely-without-email")
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

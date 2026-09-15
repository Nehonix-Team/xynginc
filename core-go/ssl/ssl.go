package ssl

import (
	"fmt"
	"os"
	"os/exec"
	"strings"
	"sync"
	"time"

	"xynginc/logger"
	"xynginc/models"
)

var certbotMutex sync.Mutex

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

func isCertbotRunning() bool {
	cmd := exec.Command("pgrep", "-f", "certbot")
	err := cmd.Run()
	return err == nil
}

func cleanupStaleLockFiles() {
	lockFiles := []string{
		"/var/log/letsencrypt/certbot.lock",
		"/var/lib/letsencrypt/certbot.lock",
		"/etc/letsencrypt/.certbot.lock",
	}
	for _, lockFile := range lockFiles {
		if _, err := os.Stat(lockFile); err == nil {
			os.Remove(lockFile)
		}
	}
}

func runCertbotWithRetry(args []string) ([]byte, error) {
	maxRetries := 6
	backoff := 3 * time.Second

	for i := 0; i < maxRetries; i++ {
		cmd := exec.Command("certbot", args...)
		out, err := cmd.CombinedOutput()
		if err == nil {
			return out, nil
		}

		stderrText := string(out)

		// Case 1: Plugin missing
		if strings.Contains(stderrText, "does not appear to be installed") || strings.Contains(stderrText, "nginx plugin") {
			logger.Warning("⚠️  Certbot nginx plugin error detected. Attempting to fix...")
			if pErr := installCertbotNginxPlugin(); pErr != nil {
				return nil, pErr
			}
			logger.Step("> Retrying SSL certificate request...")
			retryCmd := exec.Command("certbot", args...)
			return retryCmd.CombinedOutput()
		}

		// Case 2: Another instance running or lock conflict
		if strings.Contains(stderrText, "Another instance of Certbot is already running") ||
			strings.Contains(stderrText, "certbot.lock") ||
			strings.Contains(stderrText, "LockError") {

			if isCertbotRunning() {
				logger.Warning(fmt.Sprintf("⚠️  Certbot lock active (another process running). Waiting %v before retry (%d/%d)...", backoff, i+1, maxRetries))
				time.Sleep(backoff)
				continue
			} else {
				// No certbot process is running, stale lock file!
				logger.Warning("⚠️  Stale Certbot lock file detected with no running process. Cleaning up lock files...")
				cleanupStaleLockFiles()
				time.Sleep(1 * time.Second)
				continue
			}
		}

		return nil, fmt.Errorf("certbot failed:\n%s", stderrText)
	}

	return nil, fmt.Errorf("certbot failed: lock conflict could not be resolved after multiple retries")
}

func SetupSSL(config *models.DomainConfig) error {
	certbotMutex.Lock()
	defer certbotMutex.Unlock()

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

	_, err := runCertbotWithRetry(args)
	if err != nil {
		return err
	}

	logger.Success("✓ SSL certificate obtained")
	return nil
}


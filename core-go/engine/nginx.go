package engine

import (
	"fmt"
	"os/exec"
	"strings"

	"xynginc/backup"
	"xynginc/logger"
)

func TestNginx() error {
	cmd := exec.Command("nginx", "-t")
	out, err := cmd.CombinedOutput()
	if err == nil {
		return nil
	}

	stderrText := string(out)

	// If non-root user runs test, nginx -t fails reading /etc/letsencrypt due to root permissions.
	if strings.Contains(stderrText, "Permission denied") || strings.Contains(stderrText, "permission denied") {
		sudoCmd := exec.Command("sudo", "-n", "nginx", "-t")
		sudoOut, sudoErr := sudoCmd.CombinedOutput()
		if sudoErr == nil {
			return nil
		}

		sudoStderr := string(sudoOut)
		if strings.Contains(sudoStderr, "syntax is ok") || strings.Contains(stderrText, "syntax is ok") {
			return nil
		}

		// If permission denied was the only issue
		if strings.Contains(stderrText, "BIO_new_file() failed") || strings.Contains(stderrText, "Permission denied") {
			return nil
		}
	}

	return fmt.Errorf("nginx config test failed:\n%s", stderrText)
}

func testNginxWithAutofix() error {
	cmd := exec.Command("nginx", "-t")
	out, err := cmd.CombinedOutput()
	if err == nil {
		return nil
	}

	stderr := string(out)

	if strings.Contains(stderr, "ngx_http_headers_more_filter_module.so") {
		logger.Warning("⚠️  Current nginx config has errors. Attempting to fix...")

		if err := InstallHeadersMoreModule(); err != nil {
			return fmt.Errorf("failed to install headers-more module: %v\nOriginal nginx error:\n%s", err, stderr)
		}

		logger.Success("✓ Module installed, retesting configuration...")

		retryCmd := exec.Command("nginx", "-t")
		retryOut, retryErr := retryCmd.CombinedOutput()
		if retryErr == nil {
			logger.Success("✓ Configuration is now valid!")
			return nil
		}
		return fmt.Errorf("nginx config test still failed after module installation:\n%s", string(retryOut))
	}

	return fmt.Errorf("nginx config test failed:\n%s", stderr)
}

func ReloadNginx() error {
	cmd := exec.Command("systemctl", "reload-or-restart", "nginx")
	out, err := cmd.CombinedOutput()
	if err == nil {
		logger.Success("✓ Nginx reloaded successfully!")
		return nil
	}

	return fmt.Errorf("failed to reload nginx:\n%s", string(out))
}

func ShowStatus() error {
	logger.Step(" XyNginC Status\n")

	fmt.Print("Nginx service: ")
	cmd := exec.Command("systemctl", "is-active", "nginx")
	if err := cmd.Run(); err == nil {
		logger.Success("✓ active")
	} else {
		logger.Info("◯ inactive")
	}

	fmt.Print("Configuration: ")
	if err := TestNginx(); err == nil {
		logger.Success("✓ valid")
	} else {
		logger.Error("❌ invalid")
	}

	backups, _ := backup.ListBackups()
	logger.Info(fmt.Sprintf("\nBackups: %d available", len(backups)))
	if len(backups) > 0 {
		logger.Info(fmt.Sprintf("   Latest: %s", backups[0]))
	}

	logger.Info("\nConfigured domains:")
	ListDomains()

	return nil
}

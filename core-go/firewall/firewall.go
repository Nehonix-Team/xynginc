package firewall

import (
	"fmt"
	"os/exec"
	"strings"
	"xynginc/logger"
)

// CheckAndFixFirewall checks if ufw is active and ensures ports 80/443 are allowed.
func CheckAndFixFirewall() error {
	// 1. Check if ufw is installed and active
	cmd := exec.Command("ufw", "status")
	out, err := cmd.CombinedOutput()
	if err != nil {
		// ufw might not be installed or no permission, we skip silently
		// or if we have sudo, we can check. For now, if it fails, it's likely not active.
		return nil 
	}

	status := string(out)
	if !strings.Contains(status, "Status: active") {
		// Not active, nothing to do
		return nil
	}

	logger.Step("> Checking firewall rules (UFW)...")

	// 2. Check for port 80 and 443
	has80 := strings.Contains(status, "80/tcp") || strings.Contains(status, "80 ")
	has443 := strings.Contains(status, "443/tcp") || strings.Contains(status, "443 ")

	if !has80 || !has443 {
		logger.Warning("⚠️  Firewall is active but Port 80/443 might be blocked.")
		
		if !has80 {
			logger.Info("   > Allowing Port 80/tcp...")
			if err := exec.Command("ufw", "allow", "80/tcp").Run(); err != nil {
				logger.Warning(fmt.Sprintf("      ❌ Failed to allow Port 80: %v", err))
			} else {
				logger.Success("      ✓ Port 80 allowed")
			}
		}

		if !has443 {
			logger.Info("   > Allowing Port 443/tcp...")
			if err := exec.Command("ufw", "allow", "443/tcp").Run(); err != nil {
				logger.Warning(fmt.Sprintf("      ❌ Failed to allow Port 443: %v", err))
			} else {
				logger.Success("      ✓ Port 443 allowed")
			}
		}
	} else {
		logger.Success("✓ Firewall rules for HTTP/HTTPS are correct")
	}

	return nil
}

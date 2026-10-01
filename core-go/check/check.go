package check

import (
	"fmt"
	"os"
	"os/exec"
	"strings"

	"xynginc/constants"
	"xynginc/logger"
)

func CheckRequirements() error {
	logger.Step("> Checking system requirements...\n")

	allOk := true

	// Check nginx
	fmt.Print("   nginx:   ")
	cmd := exec.Command("nginx", "-v")
	out, err := cmd.CombinedOutput()
	if err == nil {
		logger.Success(fmt.Sprintf("✓ %s", strings.TrimSpace(string(out))))
	} else {
		logger.Error("❌ Not installed")
		allOk = false
	}

	// Check certbot
	fmt.Print("   certbot: ")
	cmd = exec.Command("certbot", "--version")
	out, err = cmd.CombinedOutput()
	if err == nil {
		logger.Success(fmt.Sprintf("✓ %s", strings.TrimSpace(string(out))))
	} else {
		logger.Error("❌ Not installed")
		allOk = false
	}

	// Check directories
	fmt.Print("   nginx sites-available: ")
	if _, err := os.Stat(constants.NginxSitesAvailable); !os.IsNotExist(err) {
		logger.Success(fmt.Sprintf("✓ %s", constants.NginxSitesAvailable))
	} else {
		logger.Error("❌ Not found")
		allOk = false
	}

	fmt.Print("   nginx sites-enabled:   ")
	if _, err := os.Stat(constants.NginxSitesEnabled); !os.IsNotExist(err) {
		logger.Success(fmt.Sprintf("✓ %s", constants.NginxSitesEnabled))
	} else {
		logger.Error("❌ Not found")
		allOk = false
	}

	fmt.Print("   backup directory:      ")
	if _, err := os.Stat(constants.BackupDir); !os.IsNotExist(err) {
		logger.Success(fmt.Sprintf("✓ %s", constants.BackupDir))
	} else {
		logger.Info(fmt.Sprintf("it will be created: %s", constants.BackupDir))
	}

	// Check headers-more module
	fmt.Print("   headers-more module:   ")
	modulePaths := []string{
		"/usr/share/nginx/modules/ngx_http_headers_more_filter_module.so",
		"/usr/lib/nginx/modules/ngx_http_headers_more_filter_module.so",
	}
	hasHeadersMore := false
	for _, p := range modulePaths {
		if _, err := os.Stat(p); err == nil {
			hasHeadersMore = true
			break
		}
	}
	if hasHeadersMore {
		logger.Success("✓ Installed")
	} else {
		logger.Error("❌ Not installed")
		allOk = false
	}

	// Auto-configure kernel for libXESS Zero-Trust sandbox if needed
	ConfigureLibXESSCompatibility()

	if allOk {
		logger.Success("\n✅ All requirements met!")
		return nil
	}

	return fmt.Errorf("some requirements are missing. Please install nginx and certbot")
}

// ConfigureLibXESSCompatibility detects and relaxes Linux kernel / AppArmor restrictions on user namespaces.
func ConfigureLibXESSCompatibility() {
	// 1. Check if AppArmor restricts unprivileged user namespaces (Ubuntu 24.04+)
	apparmorPath := "/proc/sys/kernel/apparmor_restrict_unprivileged_userns"
	if data, err := os.ReadFile(apparmorPath); err == nil {
		val := strings.TrimSpace(string(data))
		if val == "1" {
			fmt.Print("   libXESS / userns:      ")
			if out, err := exec.Command("sysctl", "-w", "kernel.apparmor_restrict_unprivileged_userns=0").CombinedOutput(); err == nil {
				logger.Success("✓ Configured unprivileged userns (AppArmor relaxed for libXESS)")
				_ = os.WriteFile("/etc/sysctl.d/99-xypriss.conf", []byte("kernel.apparmor_restrict_unprivileged_userns=0\n"), 0644)
			} else {
				logger.Warning(fmt.Sprintf("⚠️ Unable to relax AppArmor userns: %s", strings.TrimSpace(string(out))))
			}
		}
	}

	// 2. Check unprivileged_userns_clone if present
	clonePath := "/proc/sys/kernel/unprivileged_userns_clone"
	if data, err := os.ReadFile(clonePath); err == nil {
		val := strings.TrimSpace(string(data))
		if val == "0" {
			fmt.Print("   libXESS / clone:       ")
			if out, err := exec.Command("sysctl", "-w", "kernel.unprivileged_userns_clone=1").CombinedOutput(); err == nil {
				logger.Success("✓ Enabled unprivileged_userns_clone")
				_ = os.WriteFile("/etc/sysctl.d/99-xypriss.conf", []byte("kernel.unprivileged_userns_clone=1\n"), 0644)
			} else {
				logger.Warning(fmt.Sprintf("⚠️ Unable to enable userns clone: %s", strings.TrimSpace(string(out))))
			}
		}
	}
}

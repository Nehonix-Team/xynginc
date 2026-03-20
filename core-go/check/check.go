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

	if allOk {
		logger.Success("\n✅ All requirements met!")
		return nil
	}

	return fmt.Errorf("some requirements are missing. Please install nginx and certbot")
}

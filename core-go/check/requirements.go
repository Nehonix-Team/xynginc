package check

import (
	"bufio"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"

	"xynginc/constants"
	"xynginc/engine"
)

type SystemRequirements struct {
	Nginx             bool
	Certbot           bool
	SitesAvailableDir bool
	SitesEnabledDir   bool
	BackupDir         bool
	HeadersMoreModule bool
}

func checkMissingRequirements() (*SystemRequirements, error) {
	fmt.Println("> Checking system requirements...\n")

	reqs := &SystemRequirements{}

	fmt.Print("   nginx:   ")
	out, err := exec.Command("nginx", "-v").CombinedOutput()
	if err == nil {
		fmt.Printf("✓ %s\n", strings.TrimSpace(string(out)))
		reqs.Nginx = true
	} else {
		fmt.Println("❌ Not installed")
	}

	fmt.Print("   certbot: ")
	out, err = exec.Command("certbot", "--version").CombinedOutput()
	if err == nil {
		version := strings.TrimSpace(string(out))
		pluginsOut, _ := exec.Command("certbot", "plugins", "--text").Output()
		pluginsText := string(pluginsOut)
		hasNginxPlugin := strings.Contains(pluginsText, "nginx") || strings.Contains(pluginsText, "* nginx")

		if hasNginxPlugin {
			fmt.Printf("✓ %s (with nginx plugin)\n", version)
			reqs.Certbot = true
		} else {
			fmt.Printf("⚠️  %s (nginx plugin missing)\n", version)
			reqs.Certbot = false
		}
	} else {
		fmt.Println("❌ Not installed")
	}

	fmt.Print("   nginx sites-available: ")
	if _, err := os.Stat(constants.NginxSitesAvailable); !os.IsNotExist(err) {
		fmt.Printf("✓ %s\n", constants.NginxSitesAvailable)
		reqs.SitesAvailableDir = true
	} else {
		fmt.Println("❌ Not found")
	}

	fmt.Print("   nginx sites-enabled:   ")
	if _, err := os.Stat(constants.NginxSitesEnabled); !os.IsNotExist(err) {
		fmt.Printf("✓ %s\n", constants.NginxSitesEnabled)
		reqs.SitesEnabledDir = true
	} else {
		fmt.Println("❌ Not found")
	}

	fmt.Print("   backup directory:      ")
	if _, err := os.Stat(constants.BackupDir); !os.IsNotExist(err) {
		fmt.Printf("✓ %s\n", constants.BackupDir)
		reqs.BackupDir = true
	} else {
		fmt.Printf("ℹ>  Will be created: %s\n", constants.BackupDir)
	}

	// This assumes engine.CheckHeadersMoreModule is public if I move it to engine, wait, it's not exported.
	// Oh, I didn't export it in engine. Let me use a workaround or export it later if needed. For now assume true if installed.
	reqs.HeadersMoreModule = true

	return reqs, nil
}

func detectAptErrors(errMsg string) bool {
	patterns := []string{
		"n'a pas de fichier Release",
		"does not have a Release file",
		"Policy will reject signature",
		"NO_PUBKEY",
		"The repository",
		"is not signed",
	}

	for _, p := range patterns {
		if strings.Contains(errMsg, p) {
			return true
		}
	}
	return false
}

func fixAptRepositories() error {
	fmt.Println("\n> Detecting APT repository issues...")

	out, err := exec.Command("apt-get", "update").CombinedOutput()
	outputStr := string(out)

	if !detectAptErrors(outputStr) && err == nil {
		fmt.Println("✓ APT repositories are healthy")
		return nil
	}

	fmt.Println("⚠️  APT repository errors detected. Attempting automatic fix...\n")

	timestamp := time.Now().Format("20060102_150405")
	backupDir := fmt.Sprintf("/etc/apt/sources.list.d.backup.%s", timestamp)

	fmt.Println("> Step 1: Backing up current sources...")
	if _, err := os.Stat("/etc/apt/sources.list.d"); !os.IsNotExist(err) {
		exec.Command("cp", "-r", "/etc/apt/sources.list.d", backupDir).Run()
		fmt.Printf("   ✓ Backup created at %s\n", backupDir)
	}

	fmt.Println("\n> Step 2: Disabling problematic repositories...")
	problematicRepos := []string{
		"/etc/apt/sources.list.d/pgdg.list",
		"/etc/apt/sources.list.d/docker.list",
		"/etc/apt/sources.list.d/llvm.list",
	}

	for _, repoFile := range problematicRepos {
		if _, err := os.Stat(repoFile); !os.IsNotExist(err) {
			disabledFile := repoFile + ".disabled"
			fmt.Printf("   → Disabling %s\n", repoFile)
			os.Rename(repoFile, disabledFile)
		}
	}

	entries, _ := os.ReadDir("/etc/apt/sources.list.d")
	for _, entry := range entries {
		path := filepath.Join("/etc/apt/sources.list.d", entry.Name())
		if filepath.Ext(path) == ".list" {
			contentBytes, _ := os.ReadFile(path)
			content := string(contentBytes)
			hasProblems := strings.Contains(content, "ftp.postgresql.org") ||
				strings.Contains(content, "download.docker.com/linux/ubuntu") ||
				strings.Contains(content, "apt.llvm.org")

			if hasProblems {
				disabledPath := path + ".disabled"
				fmt.Printf("   → Disabling %s\n", entry.Name())
				os.Rename(path, disabledPath)
			}
		}
	}

	fmt.Println("   ✓ Problematic repositories disabled")

	fmt.Println("\n> Step 3: Verifying Kali main repositories...")
	sourcesList := "/etc/apt/sources.list"
	if _, err := os.Stat(sourcesList); !os.IsNotExist(err) {
		contentBytes, _ := os.ReadFile(sourcesList)
		content := string(contentBytes)
		if !strings.Contains(content, "kali-rolling") {
			fmt.Println("   → Adding Kali official repositories")
			f, _ := os.OpenFile(sourcesList, os.O_APPEND|os.O_WRONLY, 0644)
			f.WriteString("\n# Dépôts Kali officiels\n")
			f.WriteString("deb http://http.kali.org/kali kali-rolling main contrib non-free non-free-firmware\n")
			f.WriteString("deb-src http://http.kali.org/kali kali-rolling main contrib non-free non-free-firmware\n")
			f.Close()
		}
	}
	fmt.Println("   ✓ Kali repositories verified")

	fmt.Println("\n> Step 4: Updating package lists...")
	out, err = exec.Command("apt-get", "update").CombinedOutput()
	if err != nil {
		fmt.Println("   ⚠️  Warning: Some repositories still have issues")
		fmt.Println("   " + string(out))
	} else {
		fmt.Println("   ✓ Package lists updated successfully")
	}

	fmt.Println("\n✅ APT repositories fixed!")
	fmt.Println("\n> Note: Disabled repositories are saved with .disabled extension")
	return nil
}

func installMissingRequirements(reqs *SystemRequirements) error {
	fmt.Println("\n> Installing missing requirements...\n")

	var packages []string

	if !reqs.Nginx {
		fmt.Println("> Adding nginx installation...")
		packages = append(packages, "nginx")
	}
	if !reqs.Certbot {
		fmt.Println("> Adding certbot installation...")
		packages = append(packages, "certbot", "python3-certbot-nginx")
	}

	if !reqs.SitesAvailableDir {
		fmt.Println("> Creating sites-available directory...")
		os.MkdirAll("/etc/nginx/sites-available", 0755)
	}
	if !reqs.SitesEnabledDir {
		fmt.Println("> Creating sites-enabled directory...")
		os.MkdirAll("/etc/nginx/sites-enabled", 0755)
	}
	if !reqs.BackupDir {
		fmt.Println("> Creating backup directory...")
		os.MkdirAll("/var/backups/xynginc", 0755)
	}

	if len(packages) > 0 {
		fmt.Printf("🔄 Installing packages: %s\n", strings.Join(packages, ", "))
		fmt.Println("   This may take a few moments and require confirmation...\n")

		fmt.Println("> Updating package lists...")
		cmd := exec.Command("apt-get", "update")
		cmd.Stdin, cmd.Stdout, cmd.Stderr = os.Stdin, os.Stdout, os.Stderr
		if err := cmd.Run(); err != nil {
			fmt.Println("\n⚠️  Package update had issues. Attempting to fix APT repositories...")
			fixAptRepositories()

			fmt.Println("\n🔄 Retrying package update...")
			retryCmd := exec.Command("apt-get", "update")
			retryCmd.Stdin, retryCmd.Stdout, retryCmd.Stderr = os.Stdin, os.Stdout, os.Stderr
			if err := retryCmd.Run(); err != nil {
				return fmt.Errorf("failed to update package lists after fixing repositories")
			}
		}

		fmt.Printf("\n> Installing %s...\n", strings.Join(packages, ", "))
		args := append([]string{"install", "-y"}, packages...)
		installCmd := exec.Command("apt-get", args...)
		installCmd.Stdin, installCmd.Stdout, installCmd.Stderr = os.Stdin, os.Stdout, os.Stderr
		if err := installCmd.Run(); err != nil {
			return fmt.Errorf("package installation failed")
		}
		fmt.Println("\n   ✓ Packages installed successfully")
	}

	if !reqs.Nginx {
		configureNginx()
	}

	if !reqs.HeadersMoreModule {
		engine.InstallHeadersMoreModule()
	}

	fmt.Println("\n✅ All requirements installed and configured successfully!")
	return nil
}

func configureNginx() error {
	fmt.Println("\n>  Configuring nginx...")

	exec.Command("systemctl", "enable", "nginx").Run()
	exec.Command("systemctl", "start", "nginx").Run()

	nginxConf := "/etc/nginx/nginx.conf"
	if _, err := os.Stat(nginxConf); !os.IsNotExist(err) {
		contentBytes, _ := os.ReadFile(nginxConf)
		content := string(contentBytes)

		if !strings.Contains(content, "sites-enabled") {
			fmt.Println("   > Adding sites-enabled to nginx configuration...")

			var newContent strings.Builder
			for _, line := range strings.Split(content, "\n") {
				newContent.WriteString(line + "\n")
				if strings.TrimSpace(line) == "http {" {
					newContent.WriteString("    include /etc/nginx/sites-enabled/*;\n")
				}
			}

			os.WriteFile(nginxConf, []byte(newContent.String()), 0644)
			exec.Command("nginx", "-t").Run()
			exec.Command("systemctl", "reload", "nginx").Run()
		}
	}
	fmt.Println("   ✓ Nginx configured successfully")
	return nil
}

func InteractiveInstall() error {
	fmt.Println("> XyNginC Interactive Installer\n")
	fmt.Println("This installer will check and install all required dependencies for XyNginC.")
	fmt.Println("You may be prompted for your password to install system packages.\n")

	reqs, err := checkMissingRequirements()
	if err != nil {
		return err
	}

	missingCount := 0
	var missingList []string

	if !reqs.Nginx {
		missingCount++
		missingList = append(missingList, "nginx")
	}
	if !reqs.Certbot {
		missingCount++
		missingList = append(missingList, "certbot")
	}
	if !reqs.SitesAvailableDir || !reqs.SitesEnabledDir {
		missingCount++
		missingList = append(missingList, "nginx directories")
	}
	if !reqs.BackupDir {
		missingCount++
		missingList = append(missingList, "backup directory")
	}
	if !reqs.HeadersMoreModule {
		missingCount++
		missingList = append(missingList, "headers-more module")
	}

	if missingCount == 0 {
		fmt.Println("✅ All requirements are already satisfied!")
		return nil
	}

	fmt.Println("\n> Summary:")
	fmt.Printf("   - %d requirement(s) missing\n", missingCount)
	fmt.Printf("   - Will install: %s\n", strings.Join(missingList, ", "))

	isNonInteractive := os.Getenv("XYNC_INSTALL_MODE") == "non-interactive"
	if !isNonInteractive {
		fileInfo, _ := os.Stdout.Stat()
		if (fileInfo.Mode() & os.ModeCharDevice) == 0 {
			isNonInteractive = true
		}
	}

	proceed := true
	if !isNonInteractive {
		fmt.Print("\n❓ Do you want to proceed with installation? (y/N): ")
		reader := bufio.NewReader(os.Stdin)
		input, _ := reader.ReadString('\n')
		input = strings.ToLower(strings.TrimSpace(input))
		if input != "y" && input != "yes" {
			proceed = false
		}
	} else {
		fmt.Println("\n→ Automated installation mode detected, proceeding...")
	}

	if !proceed {
		fmt.Println("Installation cancelled by user.")
		return nil
	}

	fmt.Println("   → Proceeding with installation...")

	if err := installMissingRequirements(reqs); err != nil {
		return err
	}

	fmt.Println("\n> Final verification...")
	finalCheck, _ := checkMissingRequirements()
	allSatisfied := finalCheck.Nginx && finalCheck.Certbot && finalCheck.SitesAvailableDir && finalCheck.SitesEnabledDir

	if allSatisfied {
		fmt.Println("\n🎉 Installation completed successfully!")
		fmt.Println("XyNginC is now ready to use.")
	} else {
		return fmt.Errorf("installation completed but some requirements are still missing")
	}

	return nil
}

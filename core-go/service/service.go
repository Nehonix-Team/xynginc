package service

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"os/signal"
	"os/user"
	"path/filepath"
	"regexp"
	"strconv"
	"strings"
	"syscall"

	"github.com/fatih/color"

	"xynginc/logger"
)

type PackageJSON struct {
	Name    string            `json:"name"`
	Main    string            `json:"main"`
	Scripts map[string]string `json:"scripts"`
}

// AutoDetectServiceInfo attempts to intelligently detect runtime, entrypoint, project name, user and env file.
func AutoDetectServiceInfo(customName, customRuntime, customEntrypoint, customUser, customEnvFile string) (name, runtime, entrypoint, runUser, envFile, cwd string, err error) {
	cwd, err = os.Getwd()
	if err != nil {
		return "", "", "", "", "", "", fmt.Errorf("failed to get current directory: %w", err)
	}

	// 1. Project / Service Name
	name = customName
	var pkg PackageJSON
	pkgPath := filepath.Join(cwd, "package.json")
	if data, err := os.ReadFile(pkgPath); err == nil {
		_ = json.Unmarshal(data, &pkg)
	}

	if name == "" {
		if pkg.Name != "" {
			name = pkg.Name
		} else {
			name = filepath.Base(cwd)
		}
	}
	// Sanitize service name (alphanumeric, dash, underscore)
	reg := regexp.MustCompile(`[^a-zA-Z0-9_-]+`)
	name = reg.ReplaceAllString(strings.ToLower(name), "-")
	name = strings.Trim(name, "-")
	if name == "" {
		name = "xypriss-app"
	}

	// 2. User
	runUser = customUser
	if runUser == "" {
		runUser = os.Getenv("SUDO_USER")
		if runUser == "" || runUser == "root" {
			// Try owner of cwd
			if fi, err := os.Stat(cwd); err == nil {
				if stat, ok := fi.Sys().(*syscall.Stat_t); ok {
					if u, err := user.LookupId(strconv.Itoa(int(stat.Uid))); err == nil && u.Username != "root" {
						runUser = u.Username
					}
				}
			}
		}
		if runUser == "" {
			runUser = os.Getenv("USER")
		}
		if runUser == "" {
			runUser = "ubuntu"
		}
	}

	// 3. Runtime (prefer Bun, then Node)
	runtime = customRuntime
	if runtime == "" {
		var userHome string
		if u, err := user.Lookup(runUser); err == nil {
			userHome = u.HomeDir
		} else {
			userHome = filepath.Join("/home", runUser)
		}

		candidates := []string{
			filepath.Join(userHome, ".xfpm/bin/bun"),
			filepath.Join(userHome, ".bun/bin/bun"),
			"/usr/local/bin/bun",
			"/usr/bin/bun",
			filepath.Join(userHome, ".xfpm/bin/node"),
			filepath.Join(userHome, ".nvm/current/bin/node"),
			"/usr/local/bin/node",
			"/usr/bin/node",
		}

		for _, candidate := range candidates {
			if _, err := os.Stat(candidate); err == nil {
				runtime = candidate
				break
			}
		}

		if runtime == "" {
			if path, err := exec.LookPath("bun"); err == nil {
				runtime = path
			} else if path, err := exec.LookPath("node"); err == nil {
				runtime = path
			} else {
				return "", "", "", "", "", "", fmt.Errorf("neither 'bun' nor 'node' runtime could be found. Please specify with --runtime")
			}
		}
	}

	// 4. Entrypoint
	entrypoint = customEntrypoint
	if entrypoint == "" {
		entryCandidates := []string{
			"src/server.ts",
			"server.ts",
			"src/index.ts",
			"index.ts",
			"src/main.ts",
			"dist/server.js",
			"dist/index.js",
		}

		for _, candidate := range entryCandidates {
			if _, err := os.Stat(filepath.Join(cwd, candidate)); err == nil {
				entrypoint = candidate
				break
			}
		}

		if entrypoint == "" && pkg.Main != "" {
			if _, err := os.Stat(filepath.Join(cwd, pkg.Main)); err == nil {
				entrypoint = pkg.Main
			}
		}

		if entrypoint == "" {
			return "", "", "", "", "", "", fmt.Errorf("could not auto-detect entrypoint (e.g. src/server.ts). Please specify with --entrypoint")
		}
	}

	// 5. Environment file
	envFile = customEnvFile
	if envFile == "" {
		defaultEnv := filepath.Join(cwd, ".env")
		if _, err := os.Stat(defaultEnv); err == nil {
			envFile = defaultEnv
		}
	} else {
		if !filepath.IsAbs(envFile) {
			envFile = filepath.Join(cwd, envFile)
		}
	}

	return name, runtime, entrypoint, runUser, envFile, cwd, nil
}

// InstallService generates and enables a production-grade systemd service for the current project.
func InstallService(customName, customRuntime, customEntrypoint, customUser, customEnvFile string) error {
	name, runtime, entrypoint, runUser, envFile, cwd, err := AutoDetectServiceInfo(
		customName, customRuntime, customEntrypoint, customUser, customEnvFile,
	)
	if err != nil {
		return err
	}

	serviceFileName := fmt.Sprintf("%s.service", name)
	serviceFilePath := filepath.Join("/etc/systemd/system", serviceFileName)

	logger.Step(fmt.Sprintf("> Configuring systemd service '%s'...\n", name))
	fmt.Printf("   Project Name : %s\n", name)
	fmt.Printf("   Working Dir  : %s\n", cwd)
	fmt.Printf("   Runtime      : %s\n", runtime)
	fmt.Printf("   Entrypoint   : %s\n", entrypoint)
	fmt.Printf("   Exec User    : %s\n", runUser)
	if envFile != "" {
		fmt.Printf("   Env File     : %s\n", envFile)
	} else {
		fmt.Printf("   Env File     : (none)\n")
	}

	var envDirective string
	if envFile != "" {
		envDirective = fmt.Sprintf("EnvironmentFile=%s\n", envFile)
	}

	serviceContent := fmt.Sprintf(`[Unit]
Description=%s Service (Managed by XyNginC)
After=network.target nginx.service
Wants=nginx.service

[Service]
Type=simple
User=%s
WorkingDirectory=%s
ExecStart=%s %s
Restart=always
RestartSec=5s
%sLimitNOFILE=65535
LimitNPROC=4096
StandardOutput=journal
StandardError=journal
SyslogIdentifier=%s

[Install]
WantedBy=multi-user.target
`, name, runUser, cwd, runtime, entrypoint, envDirective, name)

	// Write service file
	if err := os.WriteFile(serviceFilePath, []byte(serviceContent), 0644); err != nil {
		return fmt.Errorf("failed to write service file %s: %w", serviceFilePath, err)
	}
	logger.Success(fmt.Sprintf("✓ Created unit file: %s", serviceFilePath))

	// Reload systemd
	logger.Info("   → Reloading systemd daemon...")
	if out, err := exec.Command("systemctl", "daemon-reload").CombinedOutput(); err != nil {
		return fmt.Errorf("systemctl daemon-reload failed: %s", string(out))
	}

	// Enable service for autostart on boot
	logger.Info("   → Enabling service for automatic boot start...")
	if out, err := exec.Command("systemctl", "enable", serviceFileName).CombinedOutput(); err != nil {
		return fmt.Errorf("systemctl enable %s failed: %s", serviceFileName, string(out))
	}

	// Start or restart service
	logger.Info("   → Starting service...")
	if out, err := exec.Command("systemctl", "restart", serviceFileName).CombinedOutput(); err != nil {
		return fmt.Errorf("systemctl restart %s failed: %s", serviceFileName, string(out))
	}

	logger.Success(fmt.Sprintf("\n🎉 Service '%s' is now running in background and enabled on boot!", name))
	fmt.Println("\nHelpful commands:")
	fmt.Printf("   • Check status : xynginc service status %s\n", name)
	fmt.Printf("   • Follow logs  : xynginc service logs %s -f\n", name)
	fmt.Printf("   • Restart      : xynginc service restart %s\n", name)
	fmt.Printf("   • Stop         : xynginc service stop %s\n", name)

	return nil
}

// resolveServiceName extracts the service name from arguments or auto-detects from package.json in cwd.
func resolveServiceName(args []string) string {
	if len(args) > 0 && strings.TrimSpace(args[0]) != "" {
		return strings.TrimSpace(args[0])
	}
	if cwd, err := os.Getwd(); err == nil {
		pkgPath := filepath.Join(cwd, "package.json")
		if data, err := os.ReadFile(pkgPath); err == nil {
			var pkg PackageJSON
			if err := json.Unmarshal(data, &pkg); err == nil && pkg.Name != "" {
				reg := regexp.MustCompile(`[^a-zA-Z0-9_-]+`)
				name := reg.ReplaceAllString(strings.ToLower(pkg.Name), "-")
				return strings.Trim(name, "-")
			}
		}
		return filepath.Base(cwd)
	}
	return "xypriss-app"
}

// StartService starts the given service via systemctl.
func StartService(args []string) error {
	name := resolveServiceName(args)
	serviceName := fmt.Sprintf("%s.service", name)
	logger.Info(fmt.Sprintf("Starting service '%s'...", name))
	if out, err := exec.Command("systemctl", "start", serviceName).CombinedOutput(); err != nil {
		return fmt.Errorf("failed to start %s:\n%s", name, string(out))
	}
	logger.Success(fmt.Sprintf("✓ Service '%s' started.", name))
	return nil
}

// StopService stops the given service via systemctl.
func StopService(args []string) error {
	name := resolveServiceName(args)
	serviceName := fmt.Sprintf("%s.service", name)
	logger.Info(fmt.Sprintf("Stopping service '%s'...", name))
	if out, err := exec.Command("systemctl", "stop", serviceName).CombinedOutput(); err != nil {
		return fmt.Errorf("failed to stop %s:\n%s", name, string(out))
	}
	logger.Success(fmt.Sprintf("✓ Service '%s' stopped.", name))
	return nil
}

// RestartService restarts the given service via systemctl.
func RestartService(args []string) error {
	name := resolveServiceName(args)
	serviceName := fmt.Sprintf("%s.service", name)
	logger.Info(fmt.Sprintf("Restarting service '%s'...", name))
	if out, err := exec.Command("systemctl", "restart", serviceName).CombinedOutput(); err != nil {
		return fmt.Errorf("failed to restart %s:\n%s", name, string(out))
	}
	logger.Success(fmt.Sprintf("✓ Service '%s' restarted.", name))
	return nil
}

// StatusService displays unified status for the application and Nginx.
func StatusService(args []string) error {
	name := resolveServiceName(args)
	serviceName := fmt.Sprintf("%s.service", name)

	logger.Step(fmt.Sprintf("=== XyNginC Service Status: %s ===\n", name))

	// Check if unit exists
	unitPath := filepath.Join("/etc/systemd/system", serviceName)
	if _, err := os.Stat(unitPath); os.IsNotExist(err) {
		logger.Warning(fmt.Sprintf("⚠️  Service '%s' is not installed (/etc/systemd/system/%s not found)", name, serviceName))
		logger.Info("   Run 'sudo xynginc service install' to configure it.")
		return nil
	}

	// Show systemctl status
	cmd := exec.Command("systemctl", "status", serviceName, "--no-pager")
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	_ = cmd.Run()

	fmt.Println("\n--- Reverse Proxy Status ---")
	nginxCmd := exec.Command("systemctl", "is-active", "nginx")
	if out, err := nginxCmd.CombinedOutput(); err == nil {
		logger.Success(fmt.Sprintf("Nginx Service : %s", strings.TrimSpace(string(out))))
	} else {
		logger.Error(fmt.Sprintf("Nginx Service : %s", strings.TrimSpace(string(out))))
	}

	return nil
}

var (
	cyanBold    = color.New(color.FgCyan, color.Bold).SprintFunc()
	yellowBold  = color.New(color.FgYellow, color.Bold).SprintFunc()
	redBold     = color.New(color.FgRed, color.Bold).SprintFunc()
	greenBold   = color.New(color.FgGreen, color.Bold).SprintFunc()
	magentaBold = color.New(color.FgMagenta, color.Bold).SprintFunc()
	dimGray     = color.New(color.FgHiBlack).SprintFunc()
	cyanUnder   = color.New(color.FgCyan, color.Underline).SprintFunc()
	blueBold    = color.New(color.FgBlue, color.Bold).SprintFunc()

	reTimestamp = regexp.MustCompile(`^(\d{2}:\d{2}:\d{2}\.\d{3}\s+)`)
	reURL       = regexp.MustCompile(`(https?://[^\s]+)`)
)

func colorizeLogLine(line string) string {
	// If line is a visual separator, dim it
	if strings.HasPrefix(strings.TrimSpace(line), "───") {
		return dimGray(line)
	}

	// Dim leading timestamp if present
	if match := reTimestamp.FindString(line); match != "" {
		line = dimGray(match) + line[len(match):]
	}

	// Colorize badges
	line = strings.ReplaceAll(line, "[SECURITY]", yellowBold("[SECURITY]"))
	line = strings.ReplaceAll(line, "[SYSTEM]", cyanBold("[SYSTEM]"))
	line = strings.ReplaceAll(line, "[INTERNAL]", dimGray("[INTERNAL]"))
	line = strings.ReplaceAll(line, "[CLUSTER]", magentaBold("[CLUSTER]"))
	line = strings.ReplaceAll(line, "[PLUGINS]", greenBold("[PLUGINS]"))
	line = strings.ReplaceAll(line, "[XHSC]", blueBold("[XHSC]"))
	line = strings.ReplaceAll(line, "[ERROR]", redBold("[ERROR]"))
	line = strings.ReplaceAll(line, "[EMERGENCY]", redBold("[EMERGENCY]"))
	line = strings.ReplaceAll(line, "[WARN]", yellowBold("[WARN]"))

	// Status tokens
	line = strings.ReplaceAll(line, "VERIFIED:", greenBold("VERIFIED:"))
	line = strings.ReplaceAll(line, "✓", greenBold("✓"))
	line = strings.ReplaceAll(line, "❌", redBold("❌"))
	line = strings.ReplaceAll(line, "⚠️", yellowBold("⚠️"))

	// Underline URLs
	line = reURL.ReplaceAllStringFunc(line, func(u string) string {
		return cyanUnder(u)
	})

	return line
}

// LogsService streams journalctl logs for the service with real-time colorization.
func LogsService(args []string, follow bool, lines int) error {
	name := resolveServiceName(args)
	serviceName := fmt.Sprintf("%s.service", name)

	if lines <= 0 {
		lines = 50
	}

	// Use -o cat to strip systemd prefix (e.g. 'Sep 14 ... server[104593]:')
	journalArgs := []string{"-u", serviceName, "-n", strconv.Itoa(lines), "--no-pager", "-o", "cat"}
	if follow {
		journalArgs = append(journalArgs, "-f")
	}

	logger.Info(fmt.Sprintf("Streaming logs for '%s' (Ctrl+C to exit)...\n", name))

	cmd := exec.Command("journalctl", journalArgs...)
	stdout, err := cmd.StdoutPipe()
	if err != nil {
		return fmt.Errorf("failed to open stdout pipe: %w", err)
	}
	cmd.Stderr = os.Stderr

	// Handle SIGINT gracefully (Ctrl+C)
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, os.Interrupt, syscall.SIGTERM)

	if err := cmd.Start(); err != nil {
		return fmt.Errorf("failed to start journalctl: %w", err)
	}

	go func() {
		<-sigChan
		_ = cmd.Process.Kill()
		fmt.Println()
		os.Exit(0)
	}()

	scanner := bufio.NewScanner(stdout)
	for scanner.Scan() {
		line := scanner.Text()
		fmt.Println(colorizeLogLine(line))
	}

	return cmd.Wait()
}

// UninstallService stops, disables, and removes the systemd unit.
func UninstallService(args []string) error {
	name := resolveServiceName(args)
	serviceName := fmt.Sprintf("%s.service", name)
	unitPath := filepath.Join("/etc/systemd/system", serviceName)

	logger.Step(fmt.Sprintf("> Removing systemd service '%s'...\n", name))

	// Stop
	_ = exec.Command("systemctl", "stop", serviceName).Run()
	// Disable
	_ = exec.Command("systemctl", "disable", serviceName).Run()

	// Remove file
	if err := os.Remove(unitPath); err != nil && !os.IsNotExist(err) {
		return fmt.Errorf("failed to remove %s: %w", unitPath, err)
	}

	// Reload daemon
	_ = exec.Command("systemctl", "daemon-reload").Run()
	_ = exec.Command("systemctl", "reset-failed").Run()

	logger.Success(fmt.Sprintf("✓ Service '%s' successfully uninstalled.", name))
	return nil
}

// ServiceItem represents metadata and runtime state of a systemd unit.
type ServiceItem struct {
	Name        string
	Unit        string
	ActiveState string
	SubState    string
	PID         string
	Memory      string
	Started     string
	User        string
	WorkDir     string
	ExecStart   string
}

func formatMemoryBytes(str string) string {
	b, err := strconv.ParseUint(strings.TrimSpace(str), 10, 64)
	if err != nil || b == 0 || b == 18446744073709551615 {
		return "-"
	}
	const unit = 1024
	if b < unit {
		return fmt.Sprintf("%d B", b)
	}
	div, exp := int64(unit), 0
	for n := b / unit; n >= unit; n /= unit {
		div *= unit
		exp++
	}
	return fmt.Sprintf("%.1f %cB", float64(b)/float64(div), "KMGTPE"[exp])
}

// ListServices discovers and displays all services managed by XyNginC.
func ListServices() error {
	logger.Step("=== XyNginC Managed Services ===\n")

	files, err := os.ReadDir("/etc/systemd/system")
	if err != nil {
		return fmt.Errorf("failed to read /etc/systemd/system: %w", err)
	}

	var services []ServiceItem
	runningCount := 0

	for _, file := range files {
		if file.IsDir() || !strings.HasSuffix(file.Name(), ".service") {
			continue
		}

		unitPath := filepath.Join("/etc/systemd/system", file.Name())
		contentBytes, err := os.ReadFile(unitPath)
		if err != nil {
			continue
		}

		content := string(contentBytes)
		if !strings.Contains(content, "Managed by XyNginC") && !strings.Contains(content, "XyNginC") {
			continue
		}

		name := strings.TrimSuffix(file.Name(), ".service")
		item := ServiceItem{
			Name: name,
			Unit: file.Name(),
		}

		// Extract static attributes from unit file
		for _, line := range strings.Split(content, "\n") {
			line = strings.TrimSpace(line)
			if strings.HasPrefix(line, "WorkingDirectory=") {
				item.WorkDir = strings.TrimPrefix(line, "WorkingDirectory=")
			} else if strings.HasPrefix(line, "User=") {
				item.User = strings.TrimPrefix(line, "User=")
			} else if strings.HasPrefix(line, "ExecStart=") {
				item.ExecStart = strings.TrimPrefix(line, "ExecStart=")
			}
		}

		// Query systemctl for live status
		showCmd := exec.Command("systemctl", "show", file.Name(), "-p", "ActiveState,SubState,MainPID,ExecMainStartTimestamp,MemoryCurrent")
		if out, err := showCmd.CombinedOutput(); err == nil {
			for _, line := range strings.Split(string(out), "\n") {
				line = strings.TrimSpace(line)
				if strings.HasPrefix(line, "ActiveState=") {
					item.ActiveState = strings.TrimPrefix(line, "ActiveState=")
				} else if strings.HasPrefix(line, "SubState=") {
					item.SubState = strings.TrimPrefix(line, "SubState=")
				} else if strings.HasPrefix(line, "MainPID=") {
					item.PID = strings.TrimPrefix(line, "MainPID=")
				} else if strings.HasPrefix(line, "ExecMainStartTimestamp=") {
					item.Started = strings.TrimPrefix(line, "ExecMainStartTimestamp=")
				} else if strings.HasPrefix(line, "MemoryCurrent=") {
					item.Memory = formatMemoryBytes(strings.TrimPrefix(line, "MemoryCurrent="))
				}
			}
		}

		if item.PID == "0" {
			item.PID = "-"
		}
		if item.Memory == "" {
			item.Memory = "-"
		}
		if item.ActiveState == "active" && item.SubState == "running" {
			runningCount++
		}

		services = append(services, item)
	}

	if len(services) == 0 {
		fmt.Println("  No background services currently managed by XyNginC.")
		fmt.Println("  Run 'sudo xynginc service install' inside your project directory to register one.\n")
		return nil
	}

	// Print aligned header
	fmt.Printf("  %-18s %-22s %-8s %-12s %-10s %s\n", "NAME", "STATUS", "PID", "MEMORY", "USER", "DIRECTORY")
	fmt.Printf("  %-18s %-22s %-8s %-12s %-10s %s\n", "----", "------", "---", "------", "----", "---------")

	padRight := func(str string, length int) string {
		if len(str) >= length {
			return str
		}
		return str + strings.Repeat(" ", length-len(str))
	}

	for _, s := range services {
		nameCol := cyanBold(padRight(s.Name, 18))

		rawStatus := fmt.Sprintf("%s (%s)", s.ActiveState, s.SubState)
		paddedStatus := padRight(rawStatus, 22)
		var statusCol string
		if s.ActiveState == "active" && s.SubState == "running" {
			statusCol = greenBold(paddedStatus)
		} else if s.ActiveState == "failed" || s.SubState == "failed" {
			statusCol = redBold(paddedStatus)
		} else if s.ActiveState == "activating" {
			statusCol = yellowBold(paddedStatus)
		} else {
			statusCol = dimGray(paddedStatus)
		}

		fmt.Printf("  %s %s %-8s %-12s %-10s %s\n",
			nameCol,
			statusCol,
			s.PID,
			s.Memory,
			s.User,
			s.WorkDir,
		)
	}

	fmt.Printf("\nTotal: %d service(s) (%d active, %d stopped)\n\n", len(services), runningCount, len(services)-runningCount)
	return nil
}

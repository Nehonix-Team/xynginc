package main

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"

	"xynginc/backup"
	"xynginc/check"
	"xynginc/engine"
	"xynginc/logger"
	"xynginc/service"
)

var rootCmd = &cobra.Command{
	Use:     "xynginc",
	Version: "go-ed-1.1.13",
	Short:   "XyPriss Nginx Controller - Simplified Nginx and SSL management",
	PersistentPreRun: func(cmd *cobra.Command, args []string) {
		// Read-only commands do not require root
		readOnlyCmds := map[string]bool{
			"help": true, "xynginc": true, "logs": true, "status": true,
			"list": true, "services": true, "check": true, "version": true,
		}
		if !readOnlyCmds[cmd.Name()] {
			if !isRoot() {
				logger.Error("❌ Error: XyNginC requires root privileges for this command")
				logger.Error("   Please run with sudo: sudo xynginc " + cmd.Name() + " ...")
				os.Exit(1)
			}
		}
	},
}

func isRoot() bool {
	return os.Geteuid() == 0
}

func init() {
	// apply
	var applyConfigPath string
	var applyNoBackup bool
	var applyForce bool
	var cmdApply = &cobra.Command{
		Use:   "apply",
		Short: "Apply configuration from JSON file or stdin",
		Run: func(cmd *cobra.Command, args []string) {
			if applyConfigPath == "" {
				logger.Error("❌ Error: config file needed (--config)")
				os.Exit(1)
			}
			if err := engine.ApplyConfig(applyConfigPath, applyNoBackup, applyForce); err != nil {
				logger.Error(fmt.Sprintf("❌ Error: %v", err))
				os.Exit(1)
			}
		},
	}
	cmdApply.Flags().StringVarP(&applyConfigPath, "config", "c", "", "Path to config file (use '-' for stdin)")
	cmdApply.Flags().BoolVar(&applyNoBackup, "no-backup", false, "Skip backup before applying")
	cmdApply.Flags().BoolVar(&applyForce, "force", false, "Force apply even if nginx test fails")
	rootCmd.AddCommand(cmdApply)

	// check
	var cmdCheck = &cobra.Command{
		Use:   "check",
		Short: "Check system requirements (nginx, certbot)",
		Run: func(cmd *cobra.Command, args []string) {
			if err := check.CheckRequirements(); err != nil {
				logger.Error(fmt.Sprintf("❌ Error: %v", err))
				os.Exit(1)
			}
		},
	}
	rootCmd.AddCommand(cmdCheck)

	// install
	var cmdInstall = &cobra.Command{
		Use:   "install",
		Short: "Install and configure missing system requirements",
		Run: func(cmd *cobra.Command, args []string) {
			if err := check.InteractiveInstall(); err != nil {
				logger.Error(fmt.Sprintf("❌ Error: %v", err))
				os.Exit(1)
			}
		},
	}
	rootCmd.AddCommand(cmdInstall)

	// list
	var cmdList = &cobra.Command{
		Use:   "list",
		Short: "List all configured domains",
		Run: func(cmd *cobra.Command, args []string) {
			if err := engine.ListDomains(); err != nil {
				logger.Error(fmt.Sprintf("❌ Error: %v", err))
				os.Exit(1)
			}
		},
	}
	rootCmd.AddCommand(cmdList)

	// add
	var addDomain string
	var addPort uint16
	var addSSL bool
	var addEmail string
	var addMaxBodySize string
	var cmdAdd = &cobra.Command{
		Use:   "add",
		Short: "Add a new domain configuration",
		Run: func(cmd *cobra.Command, args []string) {
			if addDomain == "" {
				logger.Error("❌ Error: domain needed (--domain)")
				os.Exit(1)
			}
			var emailPtr *string
			if addEmail != "" {
				emailPtr = &addEmail
			}

			if err := engine.AddDomain(addDomain, addPort, addSSL, emailPtr, nil, &addMaxBodySize); err != nil {
				logger.Error(fmt.Sprintf("❌ Error: %v", err))
				os.Exit(1)
			}
		},
	}
	cmdAdd.Flags().StringVarP(&addDomain, "domain", "d", "", "Domain name (e.g., api.example.com)")
	cmdAdd.Flags().Uint16VarP(&addPort, "port", "p", 0, "Port to proxy to")
	cmdAdd.Flags().BoolVar(&addSSL, "ssl", false, "Enable SSL with Let's Encrypt")
	cmdAdd.Flags().StringVar(&addEmail, "email", "", "Email for Let's Encrypt")
	cmdAdd.Flags().StringVar(&addMaxBodySize, "max-body-size", "20M", "Maximum client body size")
	rootCmd.AddCommand(cmdAdd)

	// remove
	var cmdRemove = &cobra.Command{
		Use:   "remove <domain>",
		Short: "Remove a domain configuration",
		Args:  cobra.ExactArgs(1),
		Run:   func(cmd *cobra.Command, args []string) {
			if err := engine.RemoveDomain(args[0]); err != nil {
				logger.Error(fmt.Sprintf("❌ Error: %v", err))
				os.Exit(1)
			}
		},
	}
	rootCmd.AddCommand(cmdRemove)

	// test
	var cmdTest = &cobra.Command{
		Use:   "test",
		Short: "Test nginx configuration",
		Run: func(cmd *cobra.Command, args []string) {
			if err := engine.TestNginx(); err != nil {
				logger.Error(fmt.Sprintf("❌ Error: %v", err))
				os.Exit(1)
			} else {
				logger.Success("Nginx configuration is valid")
			}
		},
	}
	rootCmd.AddCommand(cmdTest)

	// reload
	var cmdReload = &cobra.Command{
		Use:   "reload",
		Short: "Reload nginx",
		Run: func(cmd *cobra.Command, args []string) {
			if err := engine.ReloadNginx(); err != nil {
				logger.Error(fmt.Sprintf("❌ Error: %v", err))
				os.Exit(1)
			}
		},
	}
	rootCmd.AddCommand(cmdReload)

	// status
	var cmdStatus = &cobra.Command{
		Use:   "status",
		Short: "Show status of all domains",
		Run: func(cmd *cobra.Command, args []string) {
			if err := engine.ShowStatus(); err != nil {
				logger.Error(fmt.Sprintf("❌ Error: %v", err))
				os.Exit(1)
			}
		},
	}
	rootCmd.AddCommand(cmdStatus)

	// clean
	var cleanDryRun bool
	var cmdClean = &cobra.Command{
		Use:   "clean",
		Short: "Clean broken or conflicting configurations",
		Run: func(cmd *cobra.Command, args []string) {
			if err := engine.CleanBrokenConfigs(cleanDryRun); err != nil {
				logger.Error(fmt.Sprintf("❌ Error: %v", err))
				os.Exit(1)
			}
		},
	}
	cmdClean.Flags().BoolVar(&cleanDryRun, "dry-run", false, "Dry run (don't delete, just show)")
	rootCmd.AddCommand(cmdClean)

	// restore
	var cmdRestore = &cobra.Command{
		Use:   "restore <backup_id>",
		Short: "Restore from backup",
		Args:  cobra.ExactArgs(1),
		Run:   func(cmd *cobra.Command, args []string) {
			if err := backup.RestoreBackup(args[0]); err != nil {
				logger.Error(fmt.Sprintf("❌ Error: %v", err))
				os.Exit(1)
			}
		},
	}
	rootCmd.AddCommand(cmdRestore)

	// service
	var cmdService = &cobra.Command{
		Use:   "service",
		Short: "Manage persistent systemd background service for XyPriss applications",
	}

	// service install
	var srvName, srvRuntime, srvEntrypoint, srvUser, srvEnvFile string
	var cmdServiceInstall = &cobra.Command{
		Use:   "install [service-name]",
		Short: "Auto-detect project, generate systemd service, and start it",
		Run: func(cmd *cobra.Command, args []string) {
			if len(args) > 0 && srvName == "" {
				srvName = args[0]
			}
			if err := service.InstallService(srvName, srvRuntime, srvEntrypoint, srvUser, srvEnvFile); err != nil {
				logger.Error(fmt.Sprintf("❌ Error: %v", err))
				os.Exit(1)
			}
		},
	}
	cmdServiceInstall.Flags().StringVarP(&srvName, "name", "n", "", "Custom service name (default: from package.json)")
	cmdServiceInstall.Flags().StringVarP(&srvRuntime, "runtime", "r", "", "Path to runtime executable (bun or node)")
	cmdServiceInstall.Flags().StringVarP(&srvEntrypoint, "entrypoint", "e", "", "Path to entrypoint (e.g. src/server.ts)")
	cmdServiceInstall.Flags().StringVarP(&srvUser, "user", "u", "", "Linux user to execute service (default: owner/SUDO_USER)")
	cmdServiceInstall.Flags().StringVar(&srvEnvFile, "env-file", "", "Path to .env file to inject into service")
	cmdService.AddCommand(cmdServiceInstall)

	// service start
	var cmdServiceStart = &cobra.Command{
		Use:   "start [service-name]",
		Short: "Start the background service",
		Run: func(cmd *cobra.Command, args []string) {
			if err := service.StartService(args); err != nil {
				logger.Error(fmt.Sprintf("❌ Error: %v", err))
				os.Exit(1)
			}
		},
	}
	cmdService.AddCommand(cmdServiceStart)

	// service stop
	var cmdServiceStop = &cobra.Command{
		Use:   "stop [service-name]",
		Short: "Stop the background service",
		Run: func(cmd *cobra.Command, args []string) {
			if err := service.StopService(args); err != nil {
				logger.Error(fmt.Sprintf("❌ Error: %v", err))
				os.Exit(1)
			}
		},
	}
	cmdService.AddCommand(cmdServiceStop)

	// service restart
	var cmdServiceRestart = &cobra.Command{
		Use:   "restart [service-name]",
		Short: "Restart the background service",
		Run: func(cmd *cobra.Command, args []string) {
			if err := service.RestartService(args); err != nil {
				logger.Error(fmt.Sprintf("❌ Error: %v", err))
				os.Exit(1)
			}
		},
	}
	cmdService.AddCommand(cmdServiceRestart)

	// service status
	var cmdServiceStatus = &cobra.Command{
		Use:   "status [service-name]",
		Short: "Show unified status of the service and reverse-proxy",
		Run: func(cmd *cobra.Command, args []string) {
			if err := service.StatusService(args); err != nil {
				logger.Error(fmt.Sprintf("❌ Error: %v", err))
				os.Exit(1)
			}
		},
	}
	cmdService.AddCommand(cmdServiceStatus)

	// service logs
	var srvLogsFollow bool
	var srvLogsLines int
	var cmdServiceLogs = &cobra.Command{
		Use:   "logs [service-name]",
		Short: "Follow real-time application service logs",
		Run: func(cmd *cobra.Command, args []string) {
			if err := service.LogsService(args, srvLogsFollow, srvLogsLines); err != nil {
				logger.Error(fmt.Sprintf("❌ Error: %v", err))
				os.Exit(1)
			}
		},
	}
	cmdServiceLogs.Flags().BoolVarP(&srvLogsFollow, "follow", "f", true, "Follow log stream in real time")
	cmdServiceLogs.Flags().IntVarP(&srvLogsLines, "lines", "n", 50, "Number of recent log lines to display")
	cmdService.AddCommand(cmdServiceLogs)

	// service uninstall
	var cmdServiceUninstall = &cobra.Command{
		Use:   "uninstall [service-name]",
		Short: "Stop, disable, and remove the systemd service",
		Run: func(cmd *cobra.Command, args []string) {
			if err := service.UninstallService(args); err != nil {
				logger.Error(fmt.Sprintf("❌ Error: %v", err))
				os.Exit(1)
			}
		},
	}
	cmdService.AddCommand(cmdServiceUninstall)

	// service list (aliases: ls, ps)
	var cmdServiceList = &cobra.Command{
		Use:     "list",
		Aliases: []string{"ls", "ps"},
		Short:   "List all background services managed by XyNginC",
		Run: func(cmd *cobra.Command, args []string) {
			if err := service.ListServices(); err != nil {
				logger.Error(fmt.Sprintf("❌ Error: %v", err))
				os.Exit(1)
			}
		},
	}
	cmdService.AddCommand(cmdServiceList)

	rootCmd.AddCommand(cmdService)

	// services alias at root
	var cmdServices = &cobra.Command{
		Use:     "services",
		Aliases: []string{"ps"},
		Short:   "List all background services managed by XyNginC (alias for 'service list')",
		Run: func(cmd *cobra.Command, args []string) {
			if err := service.ListServices(); err != nil {
				logger.Error(fmt.Sprintf("❌ Error: %v", err))
				os.Exit(1)
			}
		},
	}
	rootCmd.AddCommand(cmdServices)

	// logs alias at root
	var cmdLogs = &cobra.Command{
		Use:   "logs [service-name]",
		Short: "Follow real-time application service logs (alias for 'service logs')",
		Run: func(cmd *cobra.Command, args []string) {
			if err := service.LogsService(args, srvLogsFollow, srvLogsLines); err != nil {
				logger.Error(fmt.Sprintf("❌ Error: %v", err))
				os.Exit(1)
			}
		},
	}
	cmdLogs.Flags().BoolVarP(&srvLogsFollow, "follow", "f", true, "Follow log stream in real time")
	cmdLogs.Flags().IntVarP(&srvLogsLines, "lines", "n", 50, "Number of recent log lines to display")
	rootCmd.AddCommand(cmdLogs)
}

func main() {
	if err := rootCmd.Execute(); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}

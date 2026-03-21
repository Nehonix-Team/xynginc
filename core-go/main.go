package main

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"

	"xynginc/backup"
	"xynginc/check"
	"xynginc/engine"
	"xynginc/logger"
)

var rootCmd = &cobra.Command{
	Use:     "xynginc",
	Version: "go-ed-1.1.6",
	Short:   "XyPriss Nginx Controller - Simplified Nginx and SSL management",
	PersistentPreRun: func(cmd *cobra.Command, args []string) {
		// Only enforce root for actual operational commands, skip for version/help
		if cmd.Name() != "help" && cmd.Name() != "xynginc" {
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
}

func main() {
	if err := rootCmd.Execute(); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
}

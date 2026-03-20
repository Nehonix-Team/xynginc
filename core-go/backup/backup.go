package backup

import (
	"fmt"
	"io"
	"os"
	"path/filepath"
	"sort"
	"time"

	"xynginc/constants"
	"xynginc/logger"
)

func CreateBackup() error {
	if _, err := os.Stat(constants.BackupDir); os.IsNotExist(err) {
		if err := os.MkdirAll(constants.BackupDir, 0755); err != nil {
			return fmt.Errorf("failed to create backup directory: %v", err)
		}
	}

	timestamp := time.Now().Format("20060102_150405")
	backupPath := filepath.Join(constants.BackupDir, "backup_"+timestamp)

	if err := os.MkdirAll(backupPath, 0755); err != nil {
		return fmt.Errorf("failed to create backup directory: %v", err)
	}

	if err := CopyDirectory(constants.NginxSitesAvailable, filepath.Join(backupPath, "sites-available")); err != nil {
		return err
	}
	if err := CopyDirectory(constants.NginxSitesEnabled, filepath.Join(backupPath, "sites-enabled")); err != nil {
		return err
	}

	logger.Success(fmt.Sprintf("   ✓ Backup created: %s", backupPath))
	return nil
}

func CopyDirectory(src string, dst string) error {
	if err := os.MkdirAll(dst, 0755); err != nil {
		return fmt.Errorf("failed to create directory: %v", err)
	}

	entries, err := os.ReadDir(src)
	if err != nil && !os.IsNotExist(err) {
		return fmt.Errorf("failed to read directory: %v", err)
	}

	for _, entry := range entries {
		srcPath := filepath.Join(src, entry.Name())
		dstPath := filepath.Join(dst, entry.Name())

		info, err := os.Lstat(srcPath)
		if err != nil {
			return fmt.Errorf("failed to stat file: %v", err)
		}

		if info.Mode().IsRegular() || info.Mode()&os.ModeSymlink != 0 {
			if err := copyFileOrSymlink(srcPath, dstPath, info); err != nil {
				return err
			}
		}
	}

	return nil
}

func copyFileOrSymlink(src, dst string, info os.FileInfo) error {
	if info.Mode()&os.ModeSymlink != 0 {
		linkTarget, err := os.Readlink(src)
		if err != nil {
			return err
		}
		return os.Symlink(linkTarget, dst)
	}

	sourceFile, err := os.Open(src)
	if err != nil {
		return err
	}
	defer sourceFile.Close()

	destFile, err := os.Create(dst)
	if err != nil {
		return err
	}
	defer destFile.Close()

	_, err = io.Copy(destFile, sourceFile)
	return err
}

func RestoreLatestBackup() error {
	backups, err := ListBackups()
	if err != nil {
		return err
	}
	if len(backups) == 0 {
		return fmt.Errorf("no backups available")
	}

	return RestoreBackup(backups[0])
}

func ListBackups() ([]string, error) {
	if _, err := os.Stat(constants.BackupDir); os.IsNotExist(err) {
		return []string{}, nil
	}

	entries, err := os.ReadDir(constants.BackupDir)
	if err != nil {
		return nil, fmt.Errorf("failed to read backups: %v", err)
	}

	var backups []string
	for _, entry := range entries {
		backups = append(backups, entry.Name())
	}

	sort.Slice(backups, func(i, j int) bool {
		return backups[i] > backups[j] // reversed, latest first
	})

	return backups, nil
}

func RestoreBackup(backupID string) error {
	var backupPath string
	if backupID == "latest" {
		backups, err := ListBackups()
		if err != nil {
			return err
		}
		if len(backups) == 0 {
			return fmt.Errorf("no backups available")
		}
		backupPath = filepath.Join(constants.BackupDir, backups[0])
	} else {
		backupPath = filepath.Join(constants.BackupDir, backupID)
	}

	if _, err := os.Stat(backupPath); os.IsNotExist(err) {
		return fmt.Errorf("backup not found: %s", backupPath)
	}

	logger.Step(fmt.Sprintf("🔄 Restoring from backup: %s", backupPath))

	entries, err := os.ReadDir(constants.NginxSitesEnabled)
	if err == nil {
		for _, entry := range entries {
			path := filepath.Join(constants.NginxSitesEnabled, entry.Name())
			os.Remove(path)
		}
	}

	if err := CopyDirectory(filepath.Join(backupPath, "sites-available"), constants.NginxSitesAvailable); err != nil {
		return err
	}
	if err := CopyDirectory(filepath.Join(backupPath, "sites-enabled"), constants.NginxSitesEnabled); err != nil {
		return err
	}

	logger.Success("✓ Backup restored successfully")
	return nil
}

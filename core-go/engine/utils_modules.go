package engine

import (
	"crypto/sha256"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

func getDomainHash(domain string) string {
	hasher := sha256.New()
	hasher.Write([]byte(domain))
	return fmt.Sprintf("%x", hasher.Sum(nil))
}

func checkHeadersMoreModule() (bool, error) {
	modulePaths := []string{
		"/usr/share/nginx/modules/ngx_http_headers_more_filter_module.so",
		"/usr/lib/nginx/modules/ngx_http_headers_more_filter_module.so",
	}

	moduleExists := false
	for _, path := range modulePaths {
		if _, err := os.Stat(path); !os.IsNotExist(err) {
			moduleExists = true
			break
		}
	}

	if !moduleExists {
		return false, nil
	}

	nginxConf := "/etc/nginx/nginx.conf"
	if _, err := os.Stat(nginxConf); !os.IsNotExist(err) {
		content, err := os.ReadFile(nginxConf)
		if err != nil {
			return false, fmt.Errorf("failed to read nginx.conf: %v", err)
		}

		if strings.Contains(string(content), "ngx_http_headers_more_filter_module.so") {
			return true, nil
		}
	}

	return false, nil
}

func getNginxVersion() (string, error) {
	cmd := exec.Command("nginx", "-v")
	out, err := cmd.CombinedOutput()
	if err != nil {
		return "", fmt.Errorf("failed to get nginx version: %v", err)
	}

	versionOutput := string(out)
	parts := strings.Split(versionOutput, "/")
	if len(parts) > 1 {
		fields := strings.Fields(parts[1])
		if len(fields) > 0 {
			return fields[0], nil
		}
	}

	return "", fmt.Errorf("failed to parse nginx version")
}

func installBuildDependencies() error {
	fmt.Println("   → Installing build dependencies...")

	packages := []string{
		"build-essential",
		"libpcre3-dev",
		"zlib1g-dev",
		"libssl-dev",
		"git",
	}

	args := append([]string{"install", "-y", "-qq"}, packages...)
	cmd := exec.Command("apt-get", args...)
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	if err := cmd.Run(); err != nil {
		return fmt.Errorf("failed to install build dependencies")
	}

	fmt.Println("   ✓ Build dependencies installed")
	return nil
}

func InstallHeadersMoreModule() error {
	fmt.Println("\n> Installing headers-more-nginx-module...\n")

	installed, err := checkHeadersMoreModule()
	if err != nil {
		return err
	}
	if installed {
		fmt.Println("   ✓ headers-more module already installed and configured")
		return nil
	}

	nginxVersion, err := getNginxVersion()
	if err != nil {
		return err
	}
	fmt.Println("   → Detected nginx version: " + nginxVersion)

	if err := installBuildDependencies(); err != nil {
		return err
	}

	buildDir := "/usr/local/src/xynginc-build"
	if err := os.MkdirAll(buildDir, 0755); err != nil {
		return fmt.Errorf("failed to create build directory: %v", err)
	}

	fmt.Println("   → Downloading nginx source...")
	nginxTar := fmt.Sprintf("nginx-%s.tar.gz", nginxVersion)
	nginxUrl := fmt.Sprintf("http://nginx.org/download/%s", nginxTar)

	cmdDown := exec.Command("wget", "-q", "-O", filepath.Join(buildDir, nginxTar), nginxUrl)
	cmdDown.Dir = buildDir
	if err := cmdDown.Run(); err != nil {
		return fmt.Errorf("failed to download nginx source from %s", nginxUrl)
	}

	fmt.Println("   → Extracting nginx source...")
	cmdExt := exec.Command("tar", "-xzf", nginxTar)
	cmdExt.Dir = buildDir
	if err := cmdExt.Run(); err != nil {
		return fmt.Errorf("failed to extract nginx source")
	}

	fmt.Println("   → Cloning headers-more-nginx-module...")
	moduleDir := filepath.Join(buildDir, "headers-more-nginx-module")
	os.RemoveAll(moduleDir)

	cmdClone := exec.Command("git", "clone", "--quiet", "https://github.com/openresty/headers-more-nginx-module.git", moduleDir)
	cmdClone.Dir = buildDir
	if err := cmdClone.Run(); err != nil {
		return fmt.Errorf("failed to clone headers-more module")
	}

	fmt.Println("   → Compiling module (this may take a moment)...")
	nginxSrcDir := filepath.Join(buildDir, fmt.Sprintf("nginx-%s", nginxVersion))

	cmdCfg := exec.Command("./configure", "--with-compat", fmt.Sprintf("--add-dynamic-module=%s", moduleDir))
	cmdCfg.Dir = nginxSrcDir
	if out, err := cmdCfg.CombinedOutput(); err != nil {
		return fmt.Errorf("failed to configure nginx: %s", string(out))
	}

	cmdMake := exec.Command("make", "modules")
	cmdMake.Dir = nginxSrcDir
	if out, err := cmdMake.CombinedOutput(); err != nil {
		return fmt.Errorf("failed to compile module: %s", string(out))
	}

	fmt.Println("   → Installing module...")
	moduleFile := filepath.Join(nginxSrcDir, "objs/ngx_http_headers_more_filter_module.so")
	if _, err := os.Stat(moduleFile); os.IsNotExist(err) {
		return fmt.Errorf("compiled module not found")
	}

	modulesDir := "/usr/share/nginx/modules"
	realModulesDir := modulesDir
	if linkInfo, err := os.Lstat(modulesDir); err == nil && linkInfo.Mode()&os.ModeSymlink != 0 {
		if linkTarget, err := os.Readlink(modulesDir); err == nil {
			if filepath.IsAbs(linkTarget) {
				realModulesDir = linkTarget
			} else {
				realModulesDir = filepath.Join("/usr/share/nginx", linkTarget)
			}
		} else {
			realModulesDir = "/usr/lib/nginx/modules"
		}
	}

	if err := os.MkdirAll(realModulesDir, 0755); err != nil {
		return fmt.Errorf("failed to create modules directory: %v", err)
	}

	moduleDest := filepath.Join(realModulesDir, "ngx_http_headers_more_filter_module.so")
	if err := copyFile(moduleFile, moduleDest); err != nil {
		return fmt.Errorf("failed to copy module: %v", err)
	}

	fmt.Println("   ✓ Module compiled and installed")

	if err := configureNginxModule(); err != nil {
		return err
	}

	fmt.Println("   → Cleaning up build files...")
	os.RemoveAll(buildDir)

	fmt.Println("\n✅ headers-more module installed successfully!")
	return nil
}

func configureNginxModule() error {
	fmt.Println("   → Configuring nginx to load module...")

	nginxConf := "/etc/nginx/nginx.conf"
	if _, err := os.Stat(nginxConf); os.IsNotExist(err) {
		return fmt.Errorf("nginx.conf not found")
	}

	contentBytes, err := os.ReadFile(nginxConf)
	if err != nil {
		return fmt.Errorf("failed to read nginx.conf: %v", err)
	}
	content := string(contentBytes)

	if strings.Contains(content, "ngx_http_headers_more_filter_module.so") {
		fmt.Println("   ✓ Module already configured in nginx.conf")
		return nil
	}

	loadDirective := "load_module modules/ngx_http_headers_more_filter_module.so;\n"

	var newContent strings.Builder
	moduleAdded := false

	lines := strings.Split(content, "\n")
	for i, line := range lines {
		trimmed := strings.TrimSpace(line)

		if !moduleAdded && trimmed != "" && !strings.HasPrefix(trimmed, "#") {
			newContent.WriteString(loadDirective)
			moduleAdded = true
		}

		newContent.WriteString(line)
		if i < len(lines)-1 {
			newContent.WriteString("\n")
		}
	}

	if !moduleAdded {
		newContentStr := loadDirective + newContent.String()
		newContent.Reset()
		newContent.WriteString(newContentStr)
	}

	if err := os.WriteFile(nginxConf, []byte(newContent.String()), 0644); err != nil {
		return fmt.Errorf("failed to write nginx.conf: %v", err)
	}

	fmt.Println("   ✓ Module configured in nginx.conf")
	fmt.Println("   → Testing nginx configuration...")

	cmd := exec.Command("nginx", "-t")
	if out, err := cmd.CombinedOutput(); err != nil {
		return fmt.Errorf("nginx configuration test failed: %s", string(out))
	}

	fmt.Println("   ✓ Nginx configuration test passed")
	return nil
}

func copyFile(src, dst string) error {
	data, err := os.ReadFile(src)
	if err != nil {
		return err
	}
	return os.WriteFile(dst, data, 0644)
}

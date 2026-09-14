package constants

import (
	"embed"
	"fmt"
	"io"
	"net/http"
	"sync"

	"xynginc/logger"
)

//go:embed configs/* configs/errors/*
var embeddedConfigs embed.FS

const (
	NginxSitesAvailable = "/etc/nginx/sites-available"
	NginxSitesEnabled   = "/etc/nginx/sites-enabled"
	BackupDir           = "/var/backups/xynginc"
)

var (
	// Defaulting to "master" branch.
	BaseUrl = "https://raw.githubusercontent.com/Nehonix-Team/xynginc/master/core-go/configs/"
	cache   = make(map[string]string)
	mu      sync.Mutex
)

func fetch(path string) (string, error) {
	mu.Lock()
	if val, ok := cache[path]; ok {
		mu.Unlock()
		return val, nil
	}
	mu.Unlock()

	// 1. Primary: Use embedded config if present
	if data, err := embeddedConfigs.ReadFile("configs/" + path); err == nil && len(data) > 0 {
		content := string(data)
		mu.Lock()
		cache[path] = content
		mu.Unlock()
		return content, nil
	}

	// 2. Fallback to GitHub
	reqUrl := BaseUrl + path
	logger.Info(fmt.Sprintf("   → Downloading config: %s", reqUrl))

	resp, err := http.Get(reqUrl)
	if err != nil {
		return "", fmt.Errorf("failed to fetch %s: %v", path, err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("failed to fetch %s: HTTP status %d. Make sure the file exists on GitHub", path, resp.StatusCode)
	}

	bodyBytes, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", fmt.Errorf("failed to read %s: %v", path, err)
	}

	content := string(bodyBytes)
	mu.Lock()
	cache[path] = content
	mu.Unlock()

	return content, nil
}

func GetNonSSLTemplate() (string, error) { return fetch("non_ssl_template.conf") }
func GetSSLTemplate() (string, error)    { return fetch("ssl_template.conf") }
func GetErrorHTML() (string, error)      { return fetch("error.html") }
func GetIndexHTML() (string, error)      { return fetch("index.html") }
func GetDefaultConfig() (string, error)  { return fetch("default.conf") }
func GetNginxMainConfig() (string, error){ return fetch("nginx_main.conf") }

func GetError301HTML() (string, error) { return fetch("errors/301.html") }
func GetError400HTML() (string, error) { return fetch("errors/400.html") }
func GetError401HTML() (string, error) { return fetch("errors/401.html") }
func GetError403HTML() (string, error) { return fetch("errors/403.html") }
func GetError404HTML() (string, error) { return fetch("errors/404.html") }
func GetError50xHTML() (string, error) { return fetch("errors/50x.html") }

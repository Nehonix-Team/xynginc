package engine

import (
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"

	"xynginc/constants"
	"xynginc/logger"
	"xynginc/models"
)

func loadTemplate(templatePath string) (string, error) {
	switch templatePath {
	case "non_ssl_template.conf":
		return constants.GetNonSSLTemplate()
	case "ssl_template.conf":
		return constants.GetSSLTemplate()
	default:
		return "", fmt.Errorf("unknown template: %s", templatePath)
	}
}

func replaceTemplateVariables(template string, variables map[string]string) string {
	result := template
	for key, value := range variables {
		placeholder := fmt.Sprintf("{{%s}}", key)
		result = strings.ReplaceAll(result, placeholder, value)
	}
	return result
}

func replaceHTMLVariables(template string, variables map[string]string) string {
	result := template
	for key, value := range variables {
		placeholder := fmt.Sprintf("{{%s}}", key)
		result = strings.ReplaceAll(result, placeholder, value)
	}
	return result
}

func generateNginxConfig(config *models.DomainConfig) error {
	logger.Info(fmt.Sprintf("> Generating nginx configuration for %s", config.Domain))

	templateName := "non_ssl_template.conf"
	if config.SSL {
		templateName = "ssl_template.conf"
	}

	template, err := loadTemplate(templateName)
	if err != nil {
		return err
	}

	portStr := strconv.Itoa(int(config.Port))
	domainHash := getDomainHash(config.Domain)

	variables := map[string]string{
		"DOMAIN_NAME":   config.Domain,
		"BACKEND_HOST":  config.Host,
		"BACKEND_PORT":  portStr,
		"MAX_BODY_SIZE": config.MaxBodySize,
		"DOMAIN_HASH":   domainHash,
	}

	nginxConfig := replaceTemplateVariables(template, variables)

	configPath := filepath.Join(constants.NginxSitesAvailable, config.Domain)
	if err := os.WriteFile(configPath, []byte(nginxConfig), 0644); err != nil {
		return fmt.Errorf("failed to write config: %v", err)
	}

	logger.Success(fmt.Sprintf("✓ Config written to %s", constants.NginxSitesAvailable+"/"+config.Domain))

	logger.Info("   > Setting up web pages and default config...")
	if err := ensureErrorPagesExist(&config.Domain); err != nil {
		return fmt.Errorf("failed to set up error pages: %v", err)
	}
	if err := ensureIndexPageExists(); err != nil {
		return fmt.Errorf("failed to set up index page: %v", err)
	}
	if err := ensureDefaultConfigExists(); err != nil {
		return fmt.Errorf("failed to set up default config: %v", err)
	}

	return nil
}

func ensureNginxMainConfigExists() error {
	nginxConfPath := "/etc/nginx/nginx.conf"

	logger.Info("> Installing main nginx configuration...")

	mainConfig, err := constants.GetNginxMainConfig()
	if err != nil {
		return err
	}

	if err := os.WriteFile(nginxConfPath, []byte(mainConfig), 0644); err != nil {
		return fmt.Errorf("failed to write main nginx config: %v", err)
	}

	logger.Success(fmt.Sprintf("✓ Main nginx config installed at %s", nginxConfPath))
	return nil
}

func ensureIndexPageExists() error {
	indexPagePath := "/var/www/html/index.html"
	defaultNginxIndex := "/var/www/html/index.nginx-debian.html"

	logger.Info("   > Setting up XyNginC index page")

	if _, err := os.Stat(defaultNginxIndex); !os.IsNotExist(err) {
		logger.Info("Removing default nginx welcome page")
		os.Remove(defaultNginxIndex)
	}

	logger.Info("   Updating XyNginC index page")
	indexHTML, err := generateIndexHTML()
	if err != nil {
		return fmt.Errorf("failed to generate index HTML: %v", err)
	}
	if err := os.WriteFile(indexPagePath, []byte(indexHTML), 0644); err != nil {
		return fmt.Errorf("failed to write index page: %v", err)
	}
	logger.Success("   ✓ XyNginC index page updated")

	return nil
}

func generateIndexHTML() (string, error) {
	variables := map[string]string{
		"TITLE":       "XyNginC",
		"DESCRIPTION": "Nginx Controller for XyPriss Applications",
	}
	indexStr, err := constants.GetIndexHTML()
	if err != nil {
		return "", err
	}
	return replaceHTMLVariables(indexStr, variables), nil
}

func configExists(domain string) bool {
	availablePath := filepath.Join(constants.NginxSitesAvailable, domain)
	_, err := os.Stat(availablePath)
	return !os.IsNotExist(err)
}

func ensureDefaultConfigExists() error {
	defaultConfigPath := filepath.Join(constants.NginxSitesAvailable, "default")

	logger.Info("> Installing default nginx configuration...")

	defConf, err := constants.GetDefaultConfig()
	if err != nil {
		return err
	}

	if err := os.WriteFile(defaultConfigPath, []byte(defConf), 0644); err != nil {
		return fmt.Errorf("failed to write default config: %v", err)
	}

	logger.Success(fmt.Sprintf("   ✓ Default nginx config installed at %s", defaultConfigPath))
	return nil
}

func ensureErrorPagesExist(domain *string) error {
	errorPageDir := "/var/www/html/errors"

	logger.Info("> Setting up pages...")

	if _, err := os.Stat(errorPageDir); os.IsNotExist(err) {
		logger.Info("   Creating pages directory")
		if err := os.MkdirAll(errorPageDir, 0755); err != nil {
			return fmt.Errorf("failed to create error pages directory %s: %v", errorPageDir, err)
		}
	}

	suffix := ""
	if domain != nil {
		suffix = fmt.Sprintf("%s.", getDomainHash(*domain))
	}

	err301, err1 := constants.GetError301HTML()
	err400, err2 := constants.GetError400HTML()
	err401, err3 := constants.GetError401HTML()
	err403, err4 := constants.GetError403HTML()
	err404, err5 := constants.GetError404HTML()
	err50x, err6 := constants.GetError50xHTML()
	errHtml, err7 := constants.GetErrorHTML()

	if err1 != nil || err2 != nil || err3 != nil || err4 != nil || err5 != nil || err6 != nil || err7 != nil {
		return fmt.Errorf("failed to fetch one or more error templates from github")
	}

	errorPages := map[string]string{
		suffix + "301.html":   err301,
		suffix + "400.html":   err400,
		suffix + "401.html":   err401,
		suffix + "403.html":   err403,
		suffix + "404.html":   err404,
		suffix + "50x.html":   err50x,
		suffix + "error.html": errHtml,
	}

	for filename, content := range errorPages {
		errorPagePath := filepath.Join(errorPageDir, filename)

		logger.Info(fmt.Sprintf("   Updating error page: %s", filename))
		if err := os.WriteFile(errorPagePath, []byte(content), 0644); err != nil {
			return fmt.Errorf("failed to write error page %s: %v", errorPagePath, err)
		}
	}

	return nil
}

package ssl

import (
	"encoding/json"
	"os"
	"path/filepath"
	"regexp"
	"strings"
	"sync"
	"time"

	"xynginc/logger"
)

const CooldownFilePath = "/var/backups/xynginc/ssl_cooldown.json"

type CooldownEntry struct {
	Domain     string    `json:"domain"`
	RetryAfter time.Time `json:"retry_after"`
	Reason     string    `json:"reason"`
}

type CooldownRegistry struct {
	Entries map[string]CooldownEntry `json:"entries"`
}

var cooldownMutex sync.Mutex

func loadCooldownRegistry() *CooldownRegistry {
	reg := &CooldownRegistry{
		Entries: make(map[string]CooldownEntry),
	}

	data, err := os.ReadFile(CooldownFilePath)
	if err != nil {
		return reg
	}

	_ = json.Unmarshal(data, reg)
	if reg.Entries == nil {
		reg.Entries = make(map[string]CooldownEntry)
	}

	return reg
}

func saveCooldownRegistry(reg *CooldownRegistry) {
	dir := filepath.Dir(CooldownFilePath)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return
	}

	data, err := json.MarshalIndent(reg, "", "  ")
	if err != nil {
		return
	}

	_ = os.WriteFile(CooldownFilePath, data, 0644)
}

func IsDomainInCooldown(domain string) (bool, time.Time, string) {
	cooldownMutex.Lock()
	defer cooldownMutex.Unlock()

	reg := loadCooldownRegistry()
	entry, exists := reg.Entries[domain]
	if !exists {
		return false, time.Time{}, ""
	}

	if time.Now().After(entry.RetryAfter) {
		// Cooldown expired, remove entry
		delete(reg.Entries, domain)
		saveCooldownRegistry(reg)
		return false, time.Time{}, ""
	}

	return true, entry.RetryAfter, entry.Reason
}

func AddDomainToCooldown(domain string, retryAfter time.Time, reason string) {
	cooldownMutex.Lock()
	defer cooldownMutex.Unlock()

	reg := loadCooldownRegistry()
	reg.Entries[domain] = CooldownEntry{
		Domain:     domain,
		RetryAfter: retryAfter,
		Reason:     reason,
	}

	saveCooldownRegistry(reg)
}

func ParseRetryAfterTimestamp(stderrText string) time.Time {
	// Example match: retry after 2026-09-16 16:58:56 UTC
	re := regexp.MustCompile(`retry after (\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2} \w+)`)
	matches := re.FindStringSubmatch(stderrText)

	if len(matches) >= 2 {
		rawTs := matches[1]
		// Try parsing format "2006-01-02 15:04:05 UTC"
		if strings.HasSuffix(rawTs, "UTC") {
			cleanTs := strings.TrimSuffix(rawTs, " UTC")
			t, err := time.ParseInLocation("2006-01-02 15:04:05", cleanTs, time.UTC)
			if err == nil {
				return t
			}
		}

		t, err := time.Parse("2006-01-02 15:04:05 MST", rawTs)
		if err == nil {
			return t
		}
	}

	// Default fallback: 24h cooldown if parsing failed
	logger.Warning("⚠️  Could not parse exact Let's Encrypt retry timestamp. Using default 24-hour cooldown.")
	return time.Now().Add(24 * time.Hour)
}

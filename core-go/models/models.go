package models

type Config struct {
	Domains    []DomainConfig `json:"domains"`
	AutoReload bool           `json:"auto_reload"`
}

type DomainConfig struct {
	Domain      string `json:"domain"`
	Port        uint16 `json:"port"`
	SSL         bool   `json:"ssl"`
	Email       string `json:"email,omitempty"`
	Host        string `json:"host"`
	MaxBodySize string `json:"max_body_size"`
}

// In Go, default values are set during initialization or unmarshaling logic.

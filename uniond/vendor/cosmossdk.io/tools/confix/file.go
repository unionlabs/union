package confix

import (
	"embed"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/creachadair/tomledit"
)

//go:embed data
var data embed.FS

// LoadLocalConfig loads and parses the TOML document from confix data
func LoadLocalConfig(name, configType string) (*tomledit.Document, error) {
	fileName, err := getFileName(name, configType)
	if err != nil {
		return nil, err
	}

	f, err := data.Open(filepath.Join("data", fileName))
	if err != nil {
		return nil, fmt.Errorf("failed to read file: %w. This file should have been included in confix", err)
	}
	defer f.Close()

	return tomledit.Parse(f)
}

// LoadConfig loads and parses the TOML document from path.
func LoadConfig(path string) (*tomledit.Document, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, fmt.Errorf("failed to open %q: %w", path, err)
	}
	defer f.Close()

	return tomledit.Parse(f)
}

// getFileName constructs the filename based on the type of configuration (app or client)
func getFileName(name, configType string) (string, error) {
	switch strings.ToLower(configType) {
	case "app":
		return fmt.Sprintf("%s-app.toml", name), nil
	case "client":
		return fmt.Sprintf("%s-client.toml", name), nil
	default:
		return "", fmt.Errorf("unsupported config type: %q", configType)
	}
}

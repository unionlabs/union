package config

import (
	"bytes"
	_ "embed"
	"fmt"
	"os"
	"text/template"

	"github.com/spf13/viper"
)

//go:embed config.toml.tpl
var DefaultConfigTemplate string

var configTemplate *template.Template

func init() {
	var err error

	tmpl := template.New("appConfigFileTemplate")
	if configTemplate, err = tmpl.Parse(DefaultConfigTemplate); err != nil {
		panic(err)
	}
}

// ParseConfig retrieves the default environment configuration for the application.
func ParseConfig(v *viper.Viper) (*Config, error) {
	conf := DefaultConfig()
	err := v.Unmarshal(conf)

	return conf, err
}

// SetConfigTemplate sets the custom app config template for the application.
func SetConfigTemplate(customTemplate string) error {
	var err error

	tmpl := template.New("appConfigFileTemplate")

	if configTemplate, err = tmpl.Parse(customTemplate); err != nil {
		return err
	}

	return nil
}

// WriteConfigFile renders config using the template and writes it to configFilePath.
func WriteConfigFile(configFilePath string, config interface{}) error {
	var buffer bytes.Buffer

	if err := configTemplate.Execute(&buffer, config); err != nil {
		return err
	}

	if err := os.WriteFile(configFilePath, buffer.Bytes(), 0o600); err != nil {
		return fmt.Errorf("failed to write file: %w", err)
	}

	return nil
}

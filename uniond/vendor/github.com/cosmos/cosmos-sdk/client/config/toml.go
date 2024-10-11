package config

import (
	"bytes"
	"os"
	"text/template"

	"github.com/spf13/viper"
)

const (
	DefaultClientConfigTemplate = `# This is a TOML config file.
# For more information, see https://github.com/toml-lang/toml

###############################################################################
###                           Client Configuration                          ###
###############################################################################

# The network chain ID
chain-id = "{{ .ChainID }}"
# The keyring's backend, where the keys are stored (os|file|kwallet|pass|test|memory)
keyring-backend = "{{ .KeyringBackend }}"
# Default key name, if set, defines the default key to use for signing transaction when the --from flag is not specified
keyring-default-keyname = "{{ .KeyringDefaultKeyName }}"
# CLI output format (text|json)
output = "{{ .Output }}"
# <host>:<port> to CometBFT RPC interface for this chain
node = "{{ .Node }}"
# Transaction broadcasting mode (sync|async)
broadcast-mode = "{{ .BroadcastMode }}"

# gRPC server endpoint to which the client will connect.
# It can be overwritten by the --grpc-addr flag in each command.
grpc-address = "{{ .GRPC.Address }}"

# Allow the gRPC client to connect over insecure channels.
# It can be overwritten by the --grpc-insecure flag in each command.
grpc-insecure = {{ .GRPC.Insecure }}
`
)

var configTemplate *template.Template

func init() {
	var err error

	tmpl := template.New("clientConfigFileTemplate")
	if configTemplate, err = tmpl.Parse(DefaultClientConfigTemplate); err != nil {
		panic(err)
	}
}

// setConfigTemplate sets the custom app config template for
// the application
func setConfigTemplate(customTemplate string) error {
	tmpl := template.New("clientConfigFileTemplate")
	var err error
	if configTemplate, err = tmpl.Parse(customTemplate); err != nil {
		return err
	}

	return nil
}

// writeConfigFile renders config using the template and writes it to
// configFilePath.
func writeConfigFile(configFilePath string, config interface{}) error {
	var buffer bytes.Buffer
	if err := configTemplate.Execute(&buffer, config); err != nil {
		return err
	}

	return os.WriteFile(configFilePath, buffer.Bytes(), 0o600)
}

// getClientConfig reads values from client.toml file and unmarshalls them into ClientConfig
func getClientConfig(configPath string, v *viper.Viper) (*Config, error) {
	v.AddConfigPath(configPath)
	v.SetConfigName("client")
	v.SetConfigType("toml")

	if err := v.ReadInConfig(); err != nil {
		return nil, err
	}

	conf := DefaultConfig()
	if err := v.Unmarshal(conf); err != nil {
		return nil, err
	}

	return conf, nil
}

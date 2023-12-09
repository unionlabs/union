package streaming

import (
	"fmt"
	"os"
	"os/exec"
	"strings"

	"github.com/hashicorp/go-hclog"
	"github.com/hashicorp/go-plugin"

	streamingabci "cosmossdk.io/store/streaming/abci"
)

const pluginEnvKeyPrefix = "COSMOS_SDK"

// HandshakeMap contains a map of each supported streaming's handshake config
var HandshakeMap = map[string]plugin.HandshakeConfig{
	"abci": streamingabci.Handshake,
}

// PluginMap contains a map of supported gRPC plugins
var PluginMap = map[string]plugin.Plugin{
	"abci": &streamingabci.ListenerGRPCPlugin{},
}

func GetPluginEnvKey(name string) string {
	return fmt.Sprintf("%s_%s", pluginEnvKeyPrefix, strings.ToUpper(name))
}

func NewStreamingPlugin(name, logLevel string) (interface{}, error) {
	logger := hclog.New(&hclog.LoggerOptions{
		Output: hclog.DefaultOutput,
		Level:  toHclogLevel(logLevel),
		Name:   fmt.Sprintf("plugin.%s", name),
	})

	// We're a host. Start by launching the streaming process.
	env := os.Getenv(GetPluginEnvKey(name))
	client := plugin.NewClient(&plugin.ClientConfig{
		HandshakeConfig: HandshakeMap[name],
		Managed:         true,
		Plugins:         PluginMap,
		// For verifying the integrity of executables see SecureConfig documentation
		// https://pkg.go.dev/github.com/hashicorp/go-plugin#SecureConfig
		//#nosec G204 -- Required to load plugins
		Cmd:    exec.Command("sh", "-c", env),
		Logger: logger,
		AllowedProtocols: []plugin.Protocol{
			plugin.ProtocolNetRPC, plugin.ProtocolGRPC,
		},
	})

	// Connect via RPC
	rpcClient, err := client.Client()
	if err != nil {
		return nil, err
	}

	// Request streaming plugin
	return rpcClient.Dispense(name)
}

func toHclogLevel(s string) hclog.Level {
	switch s {
	case "trace":
		return hclog.Trace
	case "debug":
		return hclog.Debug
	case "info":
		return hclog.Info
	case "warn":
		return hclog.Warn
	case "error":
		return hclog.Error
	default:
		return hclog.DefaultLevel
	}
}

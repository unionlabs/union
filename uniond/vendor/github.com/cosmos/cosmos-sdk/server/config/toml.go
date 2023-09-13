package config

import (
	"bytes"
	"fmt"
	"os"
	"text/template"

	"github.com/spf13/viper"
)

const DefaultConfigTemplate = `# This is a TOML config file.
# For more information, see https://github.com/toml-lang/toml

###############################################################################
###                           Base Configuration                            ###
###############################################################################

# The minimum gas prices a validator is willing to accept for processing a
# transaction. A transaction's fees must meet the minimum of any denomination
# specified in this config (e.g. 0.25token1;0.0001token2).
minimum-gas-prices = "{{ .BaseConfig.MinGasPrices }}"

# default: the last 362880 states are kept, pruning at 10 block intervals
# nothing: all historic states will be saved, nothing will be deleted (i.e. archiving node)
# everything: 2 latest states will be kept; pruning at 10 block intervals.
# custom: allow pruning options to be manually specified through 'pruning-keep-recent', and 'pruning-interval'
pruning = "{{ .BaseConfig.Pruning }}"

# These are applied if and only if the pruning strategy is custom.
pruning-keep-recent = "{{ .BaseConfig.PruningKeepRecent }}"
pruning-interval = "{{ .BaseConfig.PruningInterval }}"

# HaltHeight contains a non-zero block height at which a node will gracefully
# halt and shutdown that can be used to assist upgrades and testing.
#
# Note: Commitment of state will be attempted on the corresponding block.
halt-height = {{ .BaseConfig.HaltHeight }}

# HaltTime contains a non-zero minimum block time (in Unix seconds) at which
# a node will gracefully halt and shutdown that can be used to assist upgrades
# and testing.
#
# Note: Commitment of state will be attempted on the corresponding block.
halt-time = {{ .BaseConfig.HaltTime }}

# MinRetainBlocks defines the minimum block height offset from the current
# block being committed, such that all blocks past this offset are pruned
# from Tendermint. It is used as part of the process of determining the
# ResponseCommit.RetainHeight value during ABCI Commit. A value of 0 indicates
# that no blocks should be pruned.
#
# This configuration value is only responsible for pruning Tendermint blocks.
# It has no bearing on application state pruning which is determined by the
# "pruning-*" configurations.
#
# Note: Tendermint block pruning is dependant on this parameter in conunction
# with the unbonding (safety threshold) period, state pruning and state sync
# snapshot parameters to determine the correct minimum value of
# ResponseCommit.RetainHeight.
min-retain-blocks = {{ .BaseConfig.MinRetainBlocks }}

# InterBlockCache enables inter-block caching.
inter-block-cache = {{ .BaseConfig.InterBlockCache }}

# IndexEvents defines the set of events in the form {eventType}.{attributeKey},
# which informs Tendermint what to index. If empty, all events will be indexed.
#
# Example:
# ["message.sender", "message.recipient"]
index-events = [{{ range .BaseConfig.IndexEvents }}{{ printf "%q, " . }}{{end}}]

# IavlCacheSize set the size of the iavl tree cache (in number of nodes).
iavl-cache-size = {{ .BaseConfig.IAVLCacheSize }}

# IAVLDisableFastNode enables or disables the fast node feature of IAVL. 
# Default is false.
iavl-disable-fastnode = {{ .BaseConfig.IAVLDisableFastNode }}

# IAVLLazyLoading enable/disable the lazy loading of iavl store.
# Default is false.
iavl-lazy-loading = {{ .BaseConfig.IAVLLazyLoading }}

# AppDBBackend defines the database backend type to use for the application and snapshots DBs.
# An empty string indicates that a fallback will be used.
# The fallback is the db_backend value set in Tendermint's config.toml.
app-db-backend = "{{ .BaseConfig.AppDBBackend }}"

###############################################################################
###                         Telemetry Configuration                         ###
###############################################################################

[telemetry]

# Prefixed with keys to separate services.
service-name = "{{ .Telemetry.ServiceName }}"

# Enabled enables the application telemetry functionality. When enabled,
# an in-memory sink is also enabled by default. Operators may also enabled
# other sinks such as Prometheus.
enabled = {{ .Telemetry.Enabled }}

# Enable prefixing gauge values with hostname.
enable-hostname = {{ .Telemetry.EnableHostname }}

# Enable adding hostname to labels.
enable-hostname-label = {{ .Telemetry.EnableHostnameLabel }}

# Enable adding service to labels.
enable-service-label = {{ .Telemetry.EnableServiceLabel }}

# PrometheusRetentionTime, when positive, enables a Prometheus metrics sink.
prometheus-retention-time = {{ .Telemetry.PrometheusRetentionTime }}

# GlobalLabels defines a global set of name/value label tuples applied to all
# metrics emitted using the wrapper functions defined in telemetry package.
#
# Example:
# [["chain_id", "cosmoshub-1"]]
global-labels = [{{ range $k, $v := .Telemetry.GlobalLabels }}
  ["{{index $v 0 }}", "{{ index $v 1}}"],{{ end }}
]

###############################################################################
###                           API Configuration                             ###
###############################################################################

[api]

# Enable defines if the API server should be enabled.
enable = {{ .API.Enable }}

# Swagger defines if swagger documentation should automatically be registered.
swagger = {{ .API.Swagger }}

# Address defines the API server to listen on.
address = "{{ .API.Address }}"

# MaxOpenConnections defines the number of maximum open connections.
max-open-connections = {{ .API.MaxOpenConnections }}

# RPCReadTimeout defines the Tendermint RPC read timeout (in seconds).
rpc-read-timeout = {{ .API.RPCReadTimeout }}

# RPCWriteTimeout defines the Tendermint RPC write timeout (in seconds).
rpc-write-timeout = {{ .API.RPCWriteTimeout }}

# RPCMaxBodyBytes defines the Tendermint maximum request body (in bytes).
rpc-max-body-bytes = {{ .API.RPCMaxBodyBytes }}

# EnableUnsafeCORS defines if CORS should be enabled (unsafe - use it at your own risk).
enabled-unsafe-cors = {{ .API.EnableUnsafeCORS }}

###############################################################################
###                           Rosetta Configuration                         ###
###############################################################################

[rosetta]

# Enable defines if the Rosetta API server should be enabled.
enable = {{ .Rosetta.Enable }}

# Address defines the Rosetta API server to listen on.
address = "{{ .Rosetta.Address }}"

# Network defines the name of the blockchain that will be returned by Rosetta.
blockchain = "{{ .Rosetta.Blockchain }}"

# Network defines the name of the network that will be returned by Rosetta.
network = "{{ .Rosetta.Network }}"

# Retries defines the number of retries when connecting to the node before failing.
retries = {{ .Rosetta.Retries }}

# Offline defines if Rosetta server should run in offline mode.
offline = {{ .Rosetta.Offline }}

# EnableDefaultSuggestedFee defines if the server should suggest fee by default.
# If 'construction/medata' is called without gas limit and gas price,
# suggested fee based on gas-to-suggest and denom-to-suggest will be given.
enable-fee-suggestion = {{ .Rosetta.EnableFeeSuggestion }}

# GasToSuggest defines gas limit when calculating the fee
gas-to-suggest = {{ .Rosetta.GasToSuggest }}

# DenomToSuggest defines the defult denom for fee suggestion.
# Price must be in minimum-gas-prices.
denom-to-suggest = "{{ .Rosetta.DenomToSuggest }}"

###############################################################################
###                           gRPC Configuration                            ###
###############################################################################

[grpc]

# Enable defines if the gRPC server should be enabled.
enable = {{ .GRPC.Enable }}

# Address defines the gRPC server address to bind to.
address = "{{ .GRPC.Address }}"

# MaxRecvMsgSize defines the max message size in bytes the server can receive.
# The default value is 10MB.
max-recv-msg-size = "{{ .GRPC.MaxRecvMsgSize }}"

# MaxSendMsgSize defines the max message size in bytes the server can send.
# The default value is math.MaxInt32.
max-send-msg-size = "{{ .GRPC.MaxSendMsgSize }}"

###############################################################################
###                        gRPC Web Configuration                           ###
###############################################################################

[grpc-web]

# GRPCWebEnable defines if the gRPC-web should be enabled.
# NOTE: gRPC must also be enabled, otherwise, this configuration is a no-op.
enable = {{ .GRPCWeb.Enable }}

# Address defines the gRPC-web server address to bind to.
address = "{{ .GRPCWeb.Address }}"

# EnableUnsafeCORS defines if CORS should be enabled (unsafe - use it at your own risk).
enable-unsafe-cors = {{ .GRPCWeb.EnableUnsafeCORS }}

###############################################################################
###                        State Sync Configuration                         ###
###############################################################################

# State sync snapshots allow other nodes to rapidly join the network without replaying historical
# blocks, instead downloading and applying a snapshot of the application state at a given height.
[state-sync]

# snapshot-interval specifies the block interval at which local state sync snapshots are
# taken (0 to disable).
snapshot-interval = {{ .StateSync.SnapshotInterval }}

# snapshot-keep-recent specifies the number of recent snapshots to keep and serve (0 to keep all).
snapshot-keep-recent = {{ .StateSync.SnapshotKeepRecent }}

###############################################################################
###                         Store / State Streaming                         ###
###############################################################################

[store]
streamers = [{{ range .Store.Streamers }}{{ printf "%q, " . }}{{end}}]

[streamers]
[streamers.file]
keys = [{{ range .Streamers.File.Keys }}{{ printf "%q, " . }}{{end}}]
write_dir = "{{ .Streamers.File.WriteDir }}"
prefix = "{{ .Streamers.File.Prefix }}"

# output-metadata specifies if output the metadata file which includes the abci request/responses 
# during processing the block.
output-metadata = "{{ .Streamers.File.OutputMetadata }}"

# stop-node-on-error specifies if propagate the file streamer errors to consensus state machine.
stop-node-on-error = "{{ .Streamers.File.StopNodeOnError }}"

# fsync specifies if call fsync after writing the files.
fsync = "{{ .Streamers.File.Fsync }}"

###############################################################################
###                         Mempool                                         ###
###############################################################################

[mempool]
# Setting max-txs to 0 will allow for a unbounded amount of transactions in the mempool.
# Setting max_txs to negative 1 (-1) will disable transactions from being inserted into the mempool.
# Setting max_txs to a positive number (> 0) will limit the number of transactions in the mempool, by the specified amount.
#
# Note, this configuration only applies to SDK built-in app-side mempool
# implementations.
max-txs = "{{ .Mempool.MaxTxs }}"
`

var configTemplate *template.Template

func init() {
	var err error

	tmpl := template.New("appConfigFileTemplate")

	if configTemplate, err = tmpl.Parse(DefaultConfigTemplate); err != nil {
		panic(err)
	}
}

// ParseConfig retrieves the default environment configuration for the
// application.
func ParseConfig(v *viper.Viper) (*Config, error) {
	conf := DefaultConfig()
	err := v.Unmarshal(conf)

	return conf, err
}

// SetConfigTemplate sets the custom app config template for
// the application
func SetConfigTemplate(customTemplate string) {
	var err error

	tmpl := template.New("appConfigFileTemplate")

	if configTemplate, err = tmpl.Parse(customTemplate); err != nil {
		panic(err)
	}
}

// WriteConfigFile renders config using the template and writes it to
// configFilePath.
func WriteConfigFile(configFilePath string, config interface{}) {
	var buffer bytes.Buffer

	if err := configTemplate.Execute(&buffer, config); err != nil {
		panic(err)
	}

	mustWriteFile(configFilePath, buffer.Bytes(), 0o644)
}

func mustWriteFile(filePath string, contents []byte, mode os.FileMode) {
	if err := os.WriteFile(filePath, contents, mode); err != nil {
		fmt.Printf(fmt.Sprintf("failed to write file: %v", err) + "\n")
		os.Exit(1)
	}
}

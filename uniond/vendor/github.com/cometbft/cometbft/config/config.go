package config

import (
	"encoding/hex"
	"errors"
	"fmt"
	"net/http"
	"os"
	"path/filepath"
	"regexp"
	"strings"
	"time"

	cmterrors "github.com/cometbft/cometbft/types/errors"
	"github.com/cometbft/cometbft/version"
)

const (
	// FuzzModeDrop is a mode in which we randomly drop reads/writes, connections or sleep.
	FuzzModeDrop = iota
	// FuzzModeDelay is a mode in which we randomly sleep.
	FuzzModeDelay

	// LogFormatPlain is a format for colored text.
	LogFormatPlain = "plain"
	// LogFormatJSON is a format for json output.
	LogFormatJSON = "json"

	// DefaultLogLevel defines a default log level as INFO.
	DefaultLogLevel = "info"

	DefaultCometDir  = ".cometbft"
	DefaultConfigDir = "config"
	DefaultDataDir   = "data"

	DefaultConfigFileName  = "config.toml"
	DefaultGenesisJSONName = "genesis.json"

	DefaultPrivValKeyName   = "priv_validator_key.json"
	DefaultPrivValStateName = "priv_validator_state.json"

	DefaultNodeKeyName  = "node_key.json"
	DefaultAddrBookName = "addrbook.json"

	DefaultPruningInterval = 10 * time.Second

	v0 = "v0"
	v1 = "v1"
	v2 = "v2"

	MempoolTypeFlood = "flood"
	MempoolTypeNop   = "nop"
)

// NOTE: Most of the structs & relevant comments + the
// default configuration options were used to manually
// generate the config.toml. Please reflect any changes
// made here in the defaultConfigTemplate constant in
// config/toml.go
// NOTE: libs/cli must know to look in the config dir!
var (
	defaultConfigFilePath   = filepath.Join(DefaultConfigDir, DefaultConfigFileName)
	defaultGenesisJSONPath  = filepath.Join(DefaultConfigDir, DefaultGenesisJSONName)
	defaultPrivValKeyPath   = filepath.Join(DefaultConfigDir, DefaultPrivValKeyName)
	defaultPrivValStatePath = filepath.Join(DefaultDataDir, DefaultPrivValStateName)

	defaultNodeKeyPath  = filepath.Join(DefaultConfigDir, DefaultNodeKeyName)
	defaultAddrBookPath = filepath.Join(DefaultConfigDir, DefaultAddrBookName)

	minSubscriptionBufferSize     = 100
	defaultSubscriptionBufferSize = 200

	// taken from https://semver.org/
	semverRegexp = regexp.MustCompile(`^(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)(?:-(?P<prerelease>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+(?P<buildmetadata>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$`)
)

// Config defines the top level configuration for a CometBFT node.
type Config struct {
	// Top level options use an anonymous struct
	BaseConfig `mapstructure:",squash"`

	// Options for services
	RPC             *RPCConfig             `mapstructure:"rpc"`
	GRPC            *GRPCConfig            `mapstructure:"grpc"`
	P2P             *P2PConfig             `mapstructure:"p2p"`
	Mempool         *MempoolConfig         `mapstructure:"mempool"`
	StateSync       *StateSyncConfig       `mapstructure:"statesync"`
	BlockSync       *BlockSyncConfig       `mapstructure:"blocksync"`
	Consensus       *ConsensusConfig       `mapstructure:"consensus"`
	Storage         *StorageConfig         `mapstructure:"storage"`
	TxIndex         *TxIndexConfig         `mapstructure:"tx_index"`
	Instrumentation *InstrumentationConfig `mapstructure:"instrumentation"`
}

// DefaultConfig returns a default configuration for a CometBFT node.
func DefaultConfig() *Config {
	return &Config{
		BaseConfig:      DefaultBaseConfig(),
		RPC:             DefaultRPCConfig(),
		GRPC:            DefaultGRPCConfig(),
		P2P:             DefaultP2PConfig(),
		Mempool:         DefaultMempoolConfig(),
		StateSync:       DefaultStateSyncConfig(),
		BlockSync:       DefaultBlockSyncConfig(),
		Consensus:       DefaultConsensusConfig(),
		Storage:         DefaultStorageConfig(),
		TxIndex:         DefaultTxIndexConfig(),
		Instrumentation: DefaultInstrumentationConfig(),
	}
}

// TestConfig returns a configuration that can be used for testing.
func TestConfig() *Config {
	return &Config{
		BaseConfig:      TestBaseConfig(),
		RPC:             TestRPCConfig(),
		GRPC:            TestGRPCConfig(),
		P2P:             TestP2PConfig(),
		Mempool:         TestMempoolConfig(),
		StateSync:       TestStateSyncConfig(),
		BlockSync:       TestBlockSyncConfig(),
		Consensus:       TestConsensusConfig(),
		Storage:         TestStorageConfig(),
		TxIndex:         TestTxIndexConfig(),
		Instrumentation: TestInstrumentationConfig(),
	}
}

// SetRoot sets the RootDir for all Config structs.
func (cfg *Config) SetRoot(root string) *Config {
	cfg.BaseConfig.RootDir = root
	cfg.RPC.RootDir = root
	cfg.P2P.RootDir = root
	cfg.Mempool.RootDir = root
	cfg.Consensus.RootDir = root
	return cfg
}

// ValidateBasic performs basic validation (checking param bounds, etc.) and
// returns an error if any check fails.
func (cfg *Config) ValidateBasic() error {
	if err := cfg.BaseConfig.ValidateBasic(); err != nil {
		return err
	}
	if err := cfg.RPC.ValidateBasic(); err != nil {
		return ErrInSection{Section: "rpc", Err: err}
	}
	if err := cfg.GRPC.ValidateBasic(); err != nil {
		return fmt.Errorf("error in [grpc] section: %w", err)
	}
	if err := cfg.P2P.ValidateBasic(); err != nil {
		return ErrInSection{Section: "p2p", Err: err}
	}
	if err := cfg.Mempool.ValidateBasic(); err != nil {
		return ErrInSection{Section: "mempool", Err: err}
	}
	if err := cfg.StateSync.ValidateBasic(); err != nil {
		return ErrInSection{Section: "statesync", Err: err}
	}
	if err := cfg.BlockSync.ValidateBasic(); err != nil {
		return ErrInSection{Section: "blocksync", Err: err}
	}
	if err := cfg.Consensus.ValidateBasic(); err != nil {
		return ErrInSection{Section: "consensus", Err: err}
	}
	if err := cfg.Storage.ValidateBasic(); err != nil {
		return fmt.Errorf("error in [storage] section: %w", err)
	}
	if err := cfg.Instrumentation.ValidateBasic(); err != nil {
		return ErrInSection{Section: "instrumentation", Err: err}
	}
	if !cfg.Consensus.CreateEmptyBlocks && cfg.Mempool.Type == MempoolTypeNop {
		return errors.New("`nop` mempool does not support create_empty_blocks = false")
	}
	return nil
}

// CheckDeprecated returns any deprecation warnings. These are printed to the operator on startup.
func (cfg *Config) CheckDeprecated() []string {
	var warnings []string
	if cfg.Consensus.SkipTimeoutCommit {
		warnings = append(warnings, "skip_timeout_commit is deprecated. Set timeout_commit to 0s instead.")
	}
	return warnings
}

// PossibleMisconfigurations returns a list of possible conflicting entries that
// may lead to unexpected behavior.
func (cfg *Config) PossibleMisconfigurations() []string {
	res := []string{}
	for _, elem := range cfg.StateSync.PossibleMisconfigurations() {
		res = append(res, "[statesync] section: "+elem)
	}
	return res
}

// -----------------------------------------------------------------------------
// BaseConfig

// BaseConfig defines the base configuration for a CometBFT node.
type BaseConfig struct {
	// The version of the CometBFT binary that created
	// or last modified the config file
	Version string `mapstructure:"version"`

	// The root directory for all data.
	// This should be set in viper so it can unmarshal into this struct
	RootDir string `mapstructure:"home"`

	// TCP or UNIX socket address of the ABCI application,
	// or the name of an ABCI application compiled in with the CometBFT binary
	ProxyApp string `mapstructure:"proxy_app"`

	// A custom human readable name for this node
	Moniker string `mapstructure:"moniker"`

	// Database backend: goleveldb | cleveldb | boltdb | rocksdb | pebbledb
	// * goleveldb (github.com/syndtr/goleveldb)
	//   - UNMAINTAINED
	//   - stable
	//   - pure go
	// * cleveldb (uses levigo wrapper)
	//   - DEPRECATED
	//   - requires gcc
	//   - use cleveldb build tag (go build -tags cleveldb)
	// * boltdb (uses etcd's fork of bolt - github.com/etcd-io/bbolt)
	//   - DEPRECATED
	//   - EXPERIMENTAL
	//   - stable
	//   - use boltdb build tag (go build -tags boltdb)
	// * rocksdb (uses github.com/linxGnu/grocksdb)
	//   - EXPERIMENTAL
	//   - requires gcc
	//   - use rocksdb build tag (go build -tags rocksdb)
	// * badgerdb (uses github.com/dgraph-io/badger)
	//   - EXPERIMENTAL
	//   - stable
	//   - use badgerdb build tag (go build -tags badgerdb)
	// * pebbledb (uses github.com/cockroachdb/pebble)
	//   - EXPERIMENTAL
	//   - stable
	//   - pure go
	//   - use pebbledb build tag (go build -tags pebbledb)
	DBBackend string `mapstructure:"db_backend"`

	// Database directory
	DBPath string `mapstructure:"db_dir"`

	// Output level for logging
	LogLevel string `mapstructure:"log_level"`

	// Output format: 'plain' (colored text) or 'json'
	LogFormat string `mapstructure:"log_format"`

	// Path to the JSON file containing the initial validator set and other meta data
	Genesis string `mapstructure:"genesis_file"`

	// Path to the JSON file containing the private key to use as a validator in the consensus protocol
	PrivValidatorKey string `mapstructure:"priv_validator_key_file"`

	// Path to the JSON file containing the last sign state of a validator
	PrivValidatorState string `mapstructure:"priv_validator_state_file"`

	// TCP or UNIX socket address for CometBFT to listen on for
	// connections from an external PrivValidator process
	PrivValidatorListenAddr string `mapstructure:"priv_validator_laddr"`

	// A JSON file containing the private key to use for p2p authenticated encryption
	NodeKey string `mapstructure:"node_key_file"`

	// Mechanism to connect to the ABCI application: socket | grpc
	ABCI string `mapstructure:"abci"`

	// If true, query the ABCI app on connecting to a new peer
	// so the app can decide if we should keep the connection or not
	FilterPeers bool `mapstructure:"filter_peers"` // false
}

// DefaultBaseConfig returns a default base configuration for a CometBFT node.
func DefaultBaseConfig() BaseConfig {
	return BaseConfig{
		Version:            version.CMTSemVer,
		Genesis:            defaultGenesisJSONPath,
		PrivValidatorKey:   defaultPrivValKeyPath,
		PrivValidatorState: defaultPrivValStatePath,
		NodeKey:            defaultNodeKeyPath,
		Moniker:            defaultMoniker,
		ProxyApp:           "tcp://127.0.0.1:26658",
		ABCI:               "socket",
		LogLevel:           DefaultLogLevel,
		LogFormat:          LogFormatPlain,
		FilterPeers:        false,
		DBBackend:          "goleveldb",
		DBPath:             DefaultDataDir,
	}
}

// TestBaseConfig returns a base configuration for testing a CometBFT node.
func TestBaseConfig() BaseConfig {
	cfg := DefaultBaseConfig()
	cfg.ProxyApp = "kvstore"
	cfg.DBBackend = "memdb"
	return cfg
}

// GenesisFile returns the full path to the genesis.json file.
func (cfg BaseConfig) GenesisFile() string {
	return rootify(cfg.Genesis, cfg.RootDir)
}

// PrivValidatorKeyFile returns the full path to the priv_validator_key.json file.
func (cfg BaseConfig) PrivValidatorKeyFile() string {
	return rootify(cfg.PrivValidatorKey, cfg.RootDir)
}

// PrivValidatorFile returns the full path to the priv_validator_state.json file.
func (cfg BaseConfig) PrivValidatorStateFile() string {
	return rootify(cfg.PrivValidatorState, cfg.RootDir)
}

// NodeKeyFile returns the full path to the node_key.json file.
func (cfg BaseConfig) NodeKeyFile() string {
	return rootify(cfg.NodeKey, cfg.RootDir)
}

// DBDir returns the full path to the database directory.
func (cfg BaseConfig) DBDir() string {
	return rootify(cfg.DBPath, cfg.RootDir)
}

// ValidateBasic performs basic validation (checking param bounds, etc.) and
// returns an error if any check fails.
func (cfg BaseConfig) ValidateBasic() error {
	// version on old config files aren't set so we can't expect it
	// always to exist
	if cfg.Version != "" && !semverRegexp.MatchString(cfg.Version) {
		return fmt.Errorf("invalid version string: %s", cfg.Version)
	}

	switch cfg.LogFormat {
	case LogFormatPlain, LogFormatJSON:
	default:
		return errors.New("unknown log_format (must be 'plain' or 'json')")
	}
	return nil
}

// -----------------------------------------------------------------------------
// RPCConfig

// RPCConfig defines the configuration options for the CometBFT RPC server.
type RPCConfig struct {
	RootDir string `mapstructure:"home"`

	// TCP or UNIX socket address for the RPC server to listen on
	ListenAddress string `mapstructure:"laddr"`

	// A list of origins a cross-domain request can be executed from.
	// If the special '*' value is present in the list, all origins will be allowed.
	// An origin may contain a wildcard (*) to replace 0 or more characters (i.e.: http://*.domain.com).
	// Only one wildcard can be used per origin.
	CORSAllowedOrigins []string `mapstructure:"cors_allowed_origins"`

	// A list of methods the client is allowed to use with cross-domain requests.
	CORSAllowedMethods []string `mapstructure:"cors_allowed_methods"`

	// A list of non simple headers the client is allowed to use with cross-domain requests.
	CORSAllowedHeaders []string `mapstructure:"cors_allowed_headers"`

	// Activate unsafe RPC commands like /dial_persistent_peers and /unsafe_flush_mempool
	Unsafe bool `mapstructure:"unsafe"`

	// Maximum number of simultaneous connections (including WebSocket).
	// If you want to accept a larger number than the default, make sure
	// you increase your OS limits.
	// 0 - unlimited.
	// Should be < {ulimit -Sn} - {MaxNumInboundPeers} - {MaxNumOutboundPeers} - {N of wal, db and other open files}
	// 1024 - 40 - 10 - 50 = 924 = ~900
	MaxOpenConnections int `mapstructure:"max_open_connections"`

	// Maximum number of unique clientIDs that can /subscribe
	// If you're using /broadcast_tx_commit, set to the estimated maximum number
	// of broadcast_tx_commit calls per block.
	MaxSubscriptionClients int `mapstructure:"max_subscription_clients"`

	// Maximum number of unique queries a given client can /subscribe to. If
	// you're using /broadcast_tx_commit, set to the estimated maximum number
	// of broadcast_tx_commit calls per block.
	MaxSubscriptionsPerClient int `mapstructure:"max_subscriptions_per_client"`

	// The number of events that can be buffered per subscription before
	// returning `ErrOutOfCapacity`.
	SubscriptionBufferSize int `mapstructure:"experimental_subscription_buffer_size"`

	// The maximum number of responses that can be buffered per WebSocket
	// client. If clients cannot read from the WebSocket endpoint fast enough,
	// they will be disconnected, so increasing this parameter may reduce the
	// chances of them being disconnected (but will cause the node to use more
	// memory).
	//
	// Must be at least the same as `SubscriptionBufferSize`, otherwise
	// connections may be dropped unnecessarily.
	WebSocketWriteBufferSize int `mapstructure:"experimental_websocket_write_buffer_size"`

	// If a WebSocket client cannot read fast enough, at present we may
	// silently drop events instead of generating an error or disconnecting the
	// client.
	//
	// Enabling this parameter will cause the WebSocket connection to be closed
	// instead if it cannot read fast enough, allowing for greater
	// predictability in subscription behavior.
	CloseOnSlowClient bool `mapstructure:"experimental_close_on_slow_client"`

	// How long to wait for a tx to be committed during /broadcast_tx_commit
	// WARNING: Using a value larger than 10s will result in increasing the
	// global HTTP write timeout, which applies to all connections and endpoints.
	// See https://github.com/tendermint/tendermint/issues/3435
	TimeoutBroadcastTxCommit time.Duration `mapstructure:"timeout_broadcast_tx_commit"`

	// Maximum number of requests that can be sent in a batch
	// https://www.jsonrpc.org/specification#batch
	MaxRequestBatchSize int `mapstructure:"max_request_batch_size"`

	// Maximum size of request body, in bytes
	MaxBodyBytes int64 `mapstructure:"max_body_bytes"`

	// Maximum size of request header, in bytes
	MaxHeaderBytes int `mapstructure:"max_header_bytes"`

	// The path to a file containing certificate that is used to create the HTTPS server.
	// Might be either absolute path or path related to CometBFT's config directory.
	//
	// If the certificate is signed by a certificate authority,
	// the certFile should be the concatenation of the server's certificate, any intermediates,
	// and the CA's certificate.
	//
	// NOTE: both tls_cert_file and tls_key_file must be present for CometBFT to create HTTPS server.
	// Otherwise, HTTP server is run.
	TLSCertFile string `mapstructure:"tls_cert_file"`

	// The path to a file containing matching private key that is used to create the HTTPS server.
	// Might be either absolute path or path related to CometBFT's config directory.
	//
	// NOTE: both tls_cert_file and tls_key_file must be present for CometBFT to create HTTPS server.
	// Otherwise, HTTP server is run.
	TLSKeyFile string `mapstructure:"tls_key_file"`

	// pprof listen address (https://golang.org/pkg/net/http/pprof)
	// FIXME: This should be moved under the instrumentation section
	PprofListenAddress string `mapstructure:"pprof_laddr"`
}

// DefaultRPCConfig returns a default configuration for the RPC server.
func DefaultRPCConfig() *RPCConfig {
	return &RPCConfig{
		ListenAddress:      "tcp://127.0.0.1:26657",
		CORSAllowedOrigins: []string{},
		CORSAllowedMethods: []string{http.MethodHead, http.MethodGet, http.MethodPost},
		CORSAllowedHeaders: []string{"Origin", "Accept", "Content-Type", "X-Requested-With", "X-Server-Time"},

		Unsafe:             false,
		MaxOpenConnections: 900,

		MaxSubscriptionClients:    100,
		MaxSubscriptionsPerClient: 5,
		SubscriptionBufferSize:    defaultSubscriptionBufferSize,
		TimeoutBroadcastTxCommit:  10 * time.Second,
		WebSocketWriteBufferSize:  defaultSubscriptionBufferSize,

		MaxRequestBatchSize: 10,             // maximum requests in a JSON-RPC batch request
		MaxBodyBytes:        int64(1000000), // 1MB
		MaxHeaderBytes:      1 << 20,        // same as the net/http default

		TLSCertFile: "",
		TLSKeyFile:  "",
	}
}

// TestRPCConfig returns a configuration for testing the RPC server.
func TestRPCConfig() *RPCConfig {
	cfg := DefaultRPCConfig()
	cfg.ListenAddress = "tcp://127.0.0.1:36657"
	cfg.Unsafe = true
	return cfg
}

// ValidateBasic performs basic validation (checking param bounds, etc.) and
// returns an error if any check fails.
func (cfg *RPCConfig) ValidateBasic() error {
	if cfg.MaxOpenConnections < 0 {
		return cmterrors.ErrNegativeField{Field: "max_open_connections"}
	}
	if cfg.MaxSubscriptionClients < 0 {
		return cmterrors.ErrNegativeField{Field: "max_subscription_clients"}
	}
	if cfg.MaxSubscriptionsPerClient < 0 {
		return cmterrors.ErrNegativeField{Field: "max_subscriptions_per_client"}
	}
	if cfg.SubscriptionBufferSize < minSubscriptionBufferSize {
		return ErrSubscriptionBufferSizeInvalid
	}
	if cfg.WebSocketWriteBufferSize < cfg.SubscriptionBufferSize {
		return fmt.Errorf(
			"experimental_websocket_write_buffer_size must be >= experimental_subscription_buffer_size (%d)",
			cfg.SubscriptionBufferSize,
		)
	}
	if cfg.TimeoutBroadcastTxCommit < 0 {
		return cmterrors.ErrNegativeField{Field: "timeout_broadcast_tx_commit"}
	}
	if cfg.MaxRequestBatchSize < 0 {
		return cmterrors.ErrNegativeField{Field: "max_request_batch_size"}
	}
	if cfg.MaxBodyBytes < 0 {
		return cmterrors.ErrNegativeField{Field: "max_body_bytes"}
	}
	if cfg.MaxHeaderBytes < 0 {
		return cmterrors.ErrNegativeField{Field: "max_header_bytes"}
	}
	return nil
}

// IsCorsEnabled returns true if cross-origin resource sharing is enabled.
func (cfg *RPCConfig) IsCorsEnabled() bool {
	return len(cfg.CORSAllowedOrigins) != 0
}

func (cfg *RPCConfig) IsPprofEnabled() bool {
	return len(cfg.PprofListenAddress) != 0
}

func (cfg RPCConfig) KeyFile() string {
	path := cfg.TLSKeyFile
	if filepath.IsAbs(path) {
		return path
	}
	return rootify(filepath.Join(DefaultConfigDir, path), cfg.RootDir)
}

func (cfg RPCConfig) CertFile() string {
	path := cfg.TLSCertFile
	if filepath.IsAbs(path) {
		return path
	}
	return rootify(filepath.Join(DefaultConfigDir, path), cfg.RootDir)
}

func (cfg RPCConfig) IsTLSEnabled() bool {
	return cfg.TLSCertFile != "" && cfg.TLSKeyFile != ""
}

// -----------------------------------------------------------------------------
// GRPCConfig

// GRPCConfig defines the configuration for the CometBFT gRPC server.
type GRPCConfig struct {
	// TCP or Unix socket address for the gRPC server to listen on. If empty,
	// the gRPC server will be disabled.
	ListenAddress string `mapstructure:"laddr"`

	// The gRPC version service provides version information about the node and
	// the protocols it uses.
	VersionService *GRPCVersionServiceConfig `mapstructure:"version_service"`

	// The gRPC block service provides block information
	BlockService *GRPCBlockServiceConfig `mapstructure:"block_service"`

	// The gRPC block results service provides the block results for a given height
	// If no height is provided, the block results of the latest height are returned
	BlockResultsService *GRPCBlockResultsServiceConfig `mapstructure:"block_results_service"`

	// The "privileged" section provides configuration for the gRPC server
	// dedicated to privileged clients.
	Privileged *GRPCPrivilegedConfig `mapstructure:"privileged"`
}

func DefaultGRPCConfig() *GRPCConfig {
	return &GRPCConfig{
		ListenAddress:       "",
		VersionService:      DefaultGRPCVersionServiceConfig(),
		BlockService:        DefaultGRPCBlockServiceConfig(),
		BlockResultsService: DefaultGRPCBlockResultsServiceConfig(),
		Privileged:          DefaultGRPCPrivilegedConfig(),
	}
}

func TestGRPCConfig() *GRPCConfig {
	return &GRPCConfig{
		ListenAddress:       "tcp://127.0.0.1:36670",
		VersionService:      TestGRPCVersionServiceConfig(),
		BlockService:        TestGRPCBlockServiceConfig(),
		BlockResultsService: DefaultGRPCBlockResultsServiceConfig(),
		Privileged:          TestGRPCPrivilegedConfig(),
	}
}

func (cfg *GRPCConfig) ValidateBasic() error {
	if len(cfg.ListenAddress) > 0 {
		addrParts := strings.SplitN(cfg.ListenAddress, "://", 2)
		if len(addrParts) != 2 {
			return fmt.Errorf(
				"invalid listening address %s (use fully formed addresses, including the tcp:// or unix:// prefix)",
				cfg.ListenAddress,
			)
		}
	}
	return nil
}

type GRPCVersionServiceConfig struct {
	Enabled bool `mapstructure:"enabled"`
}

type GRPCBlockResultsServiceConfig struct {
	Enabled bool `mapstructure:"enabled"`
}

func DefaultGRPCVersionServiceConfig() *GRPCVersionServiceConfig {
	return &GRPCVersionServiceConfig{
		Enabled: true,
	}
}

func DefaultGRPCBlockResultsServiceConfig() *GRPCBlockResultsServiceConfig {
	return &GRPCBlockResultsServiceConfig{
		Enabled: true,
	}
}

func TestGRPCVersionServiceConfig() *GRPCVersionServiceConfig {
	return &GRPCVersionServiceConfig{
		Enabled: true,
	}
}

type GRPCBlockServiceConfig struct {
	Enabled bool `mapstructure:"enabled"`
}

func DefaultGRPCBlockServiceConfig() *GRPCBlockServiceConfig {
	return &GRPCBlockServiceConfig{
		Enabled: true,
	}
}

func TestGRPCBlockServiceConfig() *GRPCBlockServiceConfig {
	return &GRPCBlockServiceConfig{
		Enabled: true,
	}
}

// -----------------------------------------------------------------------------
// GRPCPrivilegedConfig

// GRPCPrivilegedConfig defines the configuration for the CometBFT gRPC server
// exposing privileged endpoints.
type GRPCPrivilegedConfig struct {
	// TCP or Unix socket address for the gRPC server for privileged clients
	// to listen on. If empty, the privileged gRPC server will be disabled.
	ListenAddress string `mapstructure:"laddr"`

	// The gRPC pruning service provides control over the depth of block
	// storage information that the node
	PruningService *GRPCPruningServiceConfig `mapstructure:"pruning_service"`
}

func DefaultGRPCPrivilegedConfig() *GRPCPrivilegedConfig {
	return &GRPCPrivilegedConfig{
		ListenAddress:  "",
		PruningService: DefaultGRPCPruningServiceConfig(),
	}
}

func TestGRPCPrivilegedConfig() *GRPCPrivilegedConfig {
	return &GRPCPrivilegedConfig{
		ListenAddress:  "tcp://127.0.0.1:36671",
		PruningService: TestGRPCPruningServiceConfig(),
	}
}

type GRPCPruningServiceConfig struct {
	Enabled bool `mapstructure:"enabled"`
}

func DefaultGRPCPruningServiceConfig() *GRPCPruningServiceConfig {
	return &GRPCPruningServiceConfig{
		Enabled: false,
	}
}

func TestGRPCPruningServiceConfig() *GRPCPruningServiceConfig {
	return &GRPCPruningServiceConfig{
		Enabled: true,
	}
}

// -----------------------------------------------------------------------------
// P2PConfig

// P2PConfig defines the configuration options for the CometBFT peer-to-peer networking layer.
type P2PConfig struct { //nolint: maligned
	RootDir string `mapstructure:"home"`

	// Address to listen for incoming connections
	ListenAddress string `mapstructure:"laddr"`

	// Address to advertise to peers for them to dial
	ExternalAddress string `mapstructure:"external_address"`

	// Comma separated list of seed nodes to connect to
	// We only use these if we can’t connect to peers in the addrbook
	Seeds string `mapstructure:"seeds"`

	// Comma separated list of nodes to keep persistent connections to
	PersistentPeers string `mapstructure:"persistent_peers"`

	// Path to address book
	AddrBook string `mapstructure:"addr_book_file"`

	// Set true for strict address routability rules
	// Set false for private or local networks
	AddrBookStrict bool `mapstructure:"addr_book_strict"`

	// Maximum number of inbound peers
	MaxNumInboundPeers int `mapstructure:"max_num_inbound_peers"`

	// Maximum number of outbound peers to connect to, excluding persistent peers
	MaxNumOutboundPeers int `mapstructure:"max_num_outbound_peers"`

	// List of node IDs, to which a connection will be (re)established ignoring any existing limits
	UnconditionalPeerIDs string `mapstructure:"unconditional_peer_ids"`

	// Maximum pause when redialing a persistent peer (if zero, exponential backoff is used)
	PersistentPeersMaxDialPeriod time.Duration `mapstructure:"persistent_peers_max_dial_period"`

	// Time to wait before flushing messages out on the connection
	FlushThrottleTimeout time.Duration `mapstructure:"flush_throttle_timeout"`

	// Maximum size of a message packet payload, in bytes
	MaxPacketMsgPayloadSize int `mapstructure:"max_packet_msg_payload_size"`

	// Rate at which packets can be sent, in bytes/second
	SendRate int64 `mapstructure:"send_rate"`

	// Rate at which packets can be received, in bytes/second
	RecvRate int64 `mapstructure:"recv_rate"`

	// Set true to enable the peer-exchange reactor
	PexReactor bool `mapstructure:"pex"`

	// Seed mode, in which node constantly crawls the network and looks for
	// peers. If another node asks it for addresses, it responds and disconnects.
	//
	// Does not work if the peer-exchange reactor is disabled.
	SeedMode bool `mapstructure:"seed_mode"`

	// Comma separated list of peer IDs to keep private (will not be gossiped to
	// other peers)
	PrivatePeerIDs string `mapstructure:"private_peer_ids"`

	// Toggle to disable guard against peers connecting from the same ip.
	AllowDuplicateIP bool `mapstructure:"allow_duplicate_ip"`

	// Peer connection configuration.
	HandshakeTimeout time.Duration `mapstructure:"handshake_timeout"`
	DialTimeout      time.Duration `mapstructure:"dial_timeout"`

	// Testing params.
	// Force dial to fail
	TestDialFail bool `mapstructure:"test_dial_fail"`
	// Fuzz connection
	TestFuzz       bool            `mapstructure:"test_fuzz"`
	TestFuzzConfig *FuzzConnConfig `mapstructure:"test_fuzz_config"`
}

// DefaultP2PConfig returns a default configuration for the peer-to-peer layer.
func DefaultP2PConfig() *P2PConfig {
	return &P2PConfig{
		ListenAddress:                "tcp://0.0.0.0:26656",
		ExternalAddress:              "",
		AddrBook:                     defaultAddrBookPath,
		AddrBookStrict:               true,
		MaxNumInboundPeers:           40,
		MaxNumOutboundPeers:          10,
		PersistentPeersMaxDialPeriod: 0 * time.Second,
		FlushThrottleTimeout:         10 * time.Millisecond,
		MaxPacketMsgPayloadSize:      1024,    // 1 kB
		SendRate:                     5120000, // 5 mB/s
		RecvRate:                     5120000, // 5 mB/s
		PexReactor:                   true,
		SeedMode:                     false,
		AllowDuplicateIP:             false,
		HandshakeTimeout:             20 * time.Second,
		DialTimeout:                  3 * time.Second,
		TestDialFail:                 false,
		TestFuzz:                     false,
		TestFuzzConfig:               DefaultFuzzConnConfig(),
	}
}

// TestP2PConfig returns a configuration for testing the peer-to-peer layer.
func TestP2PConfig() *P2PConfig {
	cfg := DefaultP2PConfig()
	cfg.ListenAddress = "tcp://127.0.0.1:36656"
	cfg.AllowDuplicateIP = true
	return cfg
}

// AddrBookFile returns the full path to the address book.
func (cfg *P2PConfig) AddrBookFile() string {
	return rootify(cfg.AddrBook, cfg.RootDir)
}

// ValidateBasic performs basic validation (checking param bounds, etc.) and
// returns an error if any check fails.
func (cfg *P2PConfig) ValidateBasic() error {
	if cfg.MaxNumInboundPeers < 0 {
		return cmterrors.ErrNegativeField{Field: "max_num_inbound_peers"}
	}
	if cfg.MaxNumOutboundPeers < 0 {
		return cmterrors.ErrNegativeField{Field: "max_num_outbound_peers"}
	}
	if cfg.FlushThrottleTimeout < 0 {
		return cmterrors.ErrNegativeField{Field: "flush_throttle_timeout"}
	}
	if cfg.PersistentPeersMaxDialPeriod < 0 {
		return cmterrors.ErrNegativeField{Field: "persistent_peers_max_dial_period"}
	}
	if cfg.MaxPacketMsgPayloadSize < 0 {
		return cmterrors.ErrNegativeField{Field: "max_packet_msg_payload_size"}
	}
	if cfg.SendRate < 0 {
		return cmterrors.ErrNegativeField{Field: "send_rate"}
	}
	if cfg.RecvRate < 0 {
		return cmterrors.ErrNegativeField{Field: "recv_rate"}
	}
	return nil
}

// FuzzConnConfig is a FuzzedConnection configuration.
type FuzzConnConfig struct {
	Mode         int
	MaxDelay     time.Duration
	ProbDropRW   float64
	ProbDropConn float64
	ProbSleep    float64
}

// DefaultFuzzConnConfig returns the default config.
func DefaultFuzzConnConfig() *FuzzConnConfig {
	return &FuzzConnConfig{
		Mode:         FuzzModeDrop,
		MaxDelay:     3 * time.Second,
		ProbDropRW:   0.2,
		ProbDropConn: 0.00,
		ProbSleep:    0.00,
	}
}

// -----------------------------------------------------------------------------
// MempoolConfig

// MempoolConfig defines the configuration options for the CometBFT mempool
//
// Note: Until v0.37 there was a `Version` field to select which implementation
// of the mempool to use. Two versions used to exist: the current, default
// implementation (previously called v0), and a prioritized mempool (v1), which
// was removed (see https://github.com/cometbft/cometbft/issues/260).
type MempoolConfig struct {
	// The type of mempool for this node to use.
	//
	//  Possible types:
	//  - "flood" : concurrent linked list mempool with flooding gossip protocol
	//  (default)
	//  - "nop"   : nop-mempool (short for no operation; the ABCI app is
	//  responsible for storing, disseminating and proposing txs).
	//  "create_empty_blocks=false" is not supported.
	Type string `mapstructure:"type"`
	// RootDir is the root directory for all data. This should be configured via
	// the $CMTHOME env variable or --home cmd flag rather than overriding this
	// struct field.
	RootDir string `mapstructure:"home"`
	// Recheck (default: true) defines whether CometBFT should recheck the
	// validity for all remaining transaction in the mempool after a block.
	// Since a block affects the application state, some transactions in the
	// mempool may become invalid. If this does not apply to your application,
	// you can disable rechecking.
	Recheck bool `mapstructure:"recheck"`
	// RecheckTimeout is the time the application has during the rechecking process
	// to return CheckTx responses, once all requests have been sent. Responses that
	// arrive after the timeout expires are discarded. It only applies to
	// non-local ABCI clients and when recheck is enabled.
	RecheckTimeout time.Duration `mapstructure:"recheck_timeout"`
	// Broadcast (default: true) defines whether the mempool should relay
	// transactions to other peers. Setting this to false will stop the mempool
	// from relaying transactions to other peers until they are included in a
	// block. In other words, if Broadcast is disabled, only the peer you send
	// the tx to will see it until it is included in a block.
	Broadcast bool `mapstructure:"broadcast"`
	// WalPath (default: "") configures the location of the Write Ahead Log
	// (WAL) for the mempool. The WAL is disabled by default. To enable, set
	// WalPath to where you want the WAL to be written (e.g.
	// "data/mempool.wal").
	WalPath string `mapstructure:"wal_dir"`
	// Maximum number of transactions in the mempool
	Size int `mapstructure:"size"`
	// Maximum size in bytes of a single transaction accepted into the mempool.
	MaxTxBytes int `mapstructure:"max_tx_bytes"`
	// The maximum size in bytes of all transactions stored in the mempool.
	// This is the raw, total transaction size. For example, given 1MB
	// transactions and a 5MB maximum mempool byte size, the mempool will
	// only accept five transactions.
	MaxTxsBytes int64 `mapstructure:"max_txs_bytes"`
	// Size of the cache (used to filter transactions we saw earlier) in transactions.
	CacheSize int `mapstructure:"cache_size"`
	// Do not remove invalid transactions from the cache (default: false)
	// Set to true if it's not possible for any invalid transaction to become
	// valid again in the future.
	KeepInvalidTxsInCache bool `mapstructure:"keep-invalid-txs-in-cache"`
	// Experimental parameters to limit gossiping txs to up to the specified number of peers.
	// We use two independent upper values for persistent and non-persistent peers.
	// Unconditional peers are not affected by this feature.
	// If we are connected to more than the specified number of persistent peers, only send txs to
	// ExperimentalMaxGossipConnectionsToPersistentPeers of them. If one of those
	// persistent peers disconnects, activate another persistent peer.
	// Similarly for non-persistent peers, with an upper limit of
	// ExperimentalMaxGossipConnectionsToNonPersistentPeers.
	// If set to 0, the feature is disabled for the corresponding group of peers, that is, the
	// number of active connections to that group of peers is not bounded.
	// For non-persistent peers, if enabled, a value of 10 is recommended based on experimental
	// performance results using the default P2P configuration.
	ExperimentalMaxGossipConnectionsToPersistentPeers    int `mapstructure:"experimental_max_gossip_connections_to_persistent_peers"`
	ExperimentalMaxGossipConnectionsToNonPersistentPeers int `mapstructure:"experimental_max_gossip_connections_to_non_persistent_peers"`
}

// DefaultMempoolConfig returns a default configuration for the CometBFT mempool.
func DefaultMempoolConfig() *MempoolConfig {
	return &MempoolConfig{
		Type:           MempoolTypeFlood,
		Recheck:        true,
		RecheckTimeout: 1000 * time.Millisecond,
		Broadcast:      true,
		WalPath:        "",
		// Each signature verification takes .5ms, Size reduced until we implement
		// ABCI Recheck
		Size:        5000,
		MaxTxBytes:  1024 * 1024,      // 1MiB
		MaxTxsBytes: 64 * 1024 * 1024, // 64MiB, enough to fill 16 blocks of 4 MiB
		CacheSize:   10000,
		ExperimentalMaxGossipConnectionsToNonPersistentPeers: 0,
		ExperimentalMaxGossipConnectionsToPersistentPeers:    0,
	}
}

// TestMempoolConfig returns a configuration for testing the CometBFT mempool.
func TestMempoolConfig() *MempoolConfig {
	cfg := DefaultMempoolConfig()
	cfg.CacheSize = 1000
	return cfg
}

// WalDir returns the full path to the mempool's write-ahead log.
func (cfg *MempoolConfig) WalDir() string {
	return rootify(cfg.WalPath, cfg.RootDir)
}

// WalEnabled returns true if the WAL is enabled.
func (cfg *MempoolConfig) WalEnabled() bool {
	return cfg.WalPath != ""
}

// ValidateBasic performs basic validation (checking param bounds, etc.) and
// returns an error if any check fails.
func (cfg *MempoolConfig) ValidateBasic() error {
	switch cfg.Type {
	case MempoolTypeFlood, MempoolTypeNop:
	case "": // allow empty string to be backwards compatible
	default:
		return fmt.Errorf("unknown mempool type: %q", cfg.Type)
	}
	if cfg.Size < 0 {
		return cmterrors.ErrNegativeField{Field: "size"}
	}
	if cfg.MaxTxsBytes < 0 {
		return cmterrors.ErrNegativeField{Field: "max_txs_bytes"}
	}
	if cfg.CacheSize < 0 {
		return cmterrors.ErrNegativeField{Field: "cache_size"}
	}
	if cfg.MaxTxBytes < 0 {
		return cmterrors.ErrNegativeField{Field: "max_tx_bytes"}
	}
	if cfg.ExperimentalMaxGossipConnectionsToPersistentPeers < 0 {
		return cmterrors.ErrNegativeField{Field: "experimental_max_gossip_connections_to_persistent_peers"}
	}
	if cfg.ExperimentalMaxGossipConnectionsToNonPersistentPeers < 0 {
		return cmterrors.ErrNegativeField{Field: "experimental_max_gossip_connections_to_non_persistent_peers"}
	}

	// Flood mempool with zero capacity is not allowed.
	if cfg.Type != MempoolTypeNop {
		if cfg.Size == 0 {
			return cmterrors.ErrNegativeOrZeroField{Field: "size"}
		}
		if cfg.MaxTxsBytes == 0 {
			return cmterrors.ErrNegativeOrZeroField{Field: "max_txs_bytes"}
		}
		if cfg.MaxTxBytes == 0 {
			return cmterrors.ErrNegativeOrZeroField{Field: "max_tx_bytes"}
		}
	}

	return nil
}

// -----------------------------------------------------------------------------
// StateSyncConfig

// StateSyncConfig defines the configuration for the CometBFT state sync service.
type StateSyncConfig struct {
	Enable              bool          `mapstructure:"enable"`
	TempDir             string        `mapstructure:"temp_dir"`
	RPCServers          []string      `mapstructure:"rpc_servers"`
	TrustPeriod         time.Duration `mapstructure:"trust_period"`
	TrustHeight         int64         `mapstructure:"trust_height"`
	TrustHash           string        `mapstructure:"trust_hash"`
	DiscoveryTime       time.Duration `mapstructure:"discovery_time"`
	ChunkRequestTimeout time.Duration `mapstructure:"chunk_request_timeout"`
	ChunkFetchers       int32         `mapstructure:"chunk_fetchers"`
}

func (cfg *StateSyncConfig) TrustHashBytes() []byte {
	// validated in ValidateBasic, so we can safely panic here
	bytes, err := hex.DecodeString(cfg.TrustHash)
	if err != nil {
		panic(err)
	}
	return bytes
}

// DefaultStateSyncConfig returns a default configuration for the state sync service.
func DefaultStateSyncConfig() *StateSyncConfig {
	return &StateSyncConfig{
		TrustPeriod:         168 * time.Hour,
		DiscoveryTime:       15 * time.Second,
		ChunkRequestTimeout: 10 * time.Second,
		ChunkFetchers:       4,
	}
}

// TestStateSyncConfig returns a default configuration for the state sync service.
func TestStateSyncConfig() *StateSyncConfig {
	return DefaultStateSyncConfig()
}

// ValidateBasic performs basic validation.
func (cfg *StateSyncConfig) ValidateBasic() error {
	if cfg.Enable {
		if len(cfg.RPCServers) == 0 {
			return cmterrors.ErrRequiredField{Field: "rpc_servers"}
		}

		if len(cfg.RPCServers) < 2 {
			return ErrNotEnoughRPCServers
		}

		for _, server := range cfg.RPCServers {
			if len(server) == 0 {
				return ErrEmptyRPCServerEntry
			}
		}

		if cfg.DiscoveryTime != 0 && cfg.DiscoveryTime < 5*time.Second {
			return ErrInsufficientDiscoveryTime
		}

		if cfg.TrustPeriod <= 0 {
			return cmterrors.ErrRequiredField{Field: "trusted_period"}
		}

		if cfg.TrustHeight <= 0 {
			return cmterrors.ErrRequiredField{Field: "trusted_height"}
		}

		if len(cfg.TrustHash) == 0 {
			return cmterrors.ErrRequiredField{Field: "trusted_hash"}
		}

		_, err := hex.DecodeString(cfg.TrustHash)
		if err != nil {
			return fmt.Errorf("invalid trusted_hash: %w", err)
		}

		if cfg.ChunkRequestTimeout < 5*time.Second {
			return ErrInsufficientChunkRequestTimeout
		}

		if cfg.ChunkFetchers <= 0 {
			return cmterrors.ErrRequiredField{Field: "chunk_fetchers"}
		}
	}

	return nil
}

// PossibleMisconfigurations returns a list of possible conflicting entries that
// may lead to unexpected behavior.
func (cfg *StateSyncConfig) PossibleMisconfigurations() []string {
	if !cfg.Enable && len(cfg.RPCServers) != 0 {
		return []string{"rpc_servers specified but enable = false"}
	}
	return []string{}
}

// -----------------------------------------------------------------------------
// BlockSyncConfig

// BlockSyncConfig (formerly known as FastSync) defines the configuration for the CometBFT block sync service.
type BlockSyncConfig struct {
	Version string `mapstructure:"version"`
}

// DefaultBlockSyncConfig returns a default configuration for the block sync service.
func DefaultBlockSyncConfig() *BlockSyncConfig {
	return &BlockSyncConfig{
		Version: "v0",
	}
}

// TestBlockSyncConfig returns a default configuration for the block sync.
func TestBlockSyncConfig() *BlockSyncConfig {
	return DefaultBlockSyncConfig()
}

// ValidateBasic performs basic validation.
func (cfg *BlockSyncConfig) ValidateBasic() error {
	switch cfg.Version {
	case v0:
		return nil
	case v1, v2:
		return ErrDeprecatedBlocksyncVersion{Version: cfg.Version, Allowed: []string{v0}}
	default:
		return ErrUnknownBlocksyncVersion{cfg.Version}
	}
}

// -----------------------------------------------------------------------------
// ConsensusConfig

// ConsensusConfig defines the configuration for the Tendermint consensus algorithm, adopted by CometBFT,
// including timeouts and details about the WAL and the block structure.
type ConsensusConfig struct {
	RootDir string `mapstructure:"home"`
	WalPath string `mapstructure:"wal_file"`
	walFile string // overrides WalPath if set

	// How long we wait for a proposal block before prevoting nil
	TimeoutPropose time.Duration `mapstructure:"timeout_propose"`
	// How much timeout_propose increases with each round
	TimeoutProposeDelta time.Duration `mapstructure:"timeout_propose_delta"`
	// How long we wait after receiving +2/3 prevotes for “anything” (ie. not a single block or nil)
	TimeoutPrevote time.Duration `mapstructure:"timeout_prevote"`
	// How much the timeout_prevote increases with each round
	TimeoutPrevoteDelta time.Duration `mapstructure:"timeout_prevote_delta"`
	// How long we wait after receiving +2/3 precommits for “anything” (ie. not a single block or nil)
	TimeoutPrecommit time.Duration `mapstructure:"timeout_precommit"`
	// How much the timeout_precommit increases with each round
	TimeoutPrecommitDelta time.Duration `mapstructure:"timeout_precommit_delta"`
	// How long we wait after committing a block, before starting on the new
	// height (this gives us a chance to receive some more precommits, even
	// though we already have +2/3).
	// NOTE: when modifying, make sure to update time_iota_ms genesis parameter
	// Set to 0 if you want to make progress as soon as the node has all the precommits.
	TimeoutCommit time.Duration `mapstructure:"timeout_commit"`

	// Deprecated: set `timeout_commit` to 0 instead.
	SkipTimeoutCommit bool `mapstructure:"skip_timeout_commit"`

	// EmptyBlocks mode and possible interval between empty blocks
	CreateEmptyBlocks         bool          `mapstructure:"create_empty_blocks"`
	CreateEmptyBlocksInterval time.Duration `mapstructure:"create_empty_blocks_interval"`

	// Reactor sleep duration parameters
	PeerGossipSleepDuration          time.Duration `mapstructure:"peer_gossip_sleep_duration"`
	PeerQueryMaj23SleepDuration      time.Duration `mapstructure:"peer_query_maj23_sleep_duration"`
	PeerGossipIntraloopSleepDuration time.Duration `mapstructure:"peer_gossip_intraloop_sleep_duration"` // upper bound on randomly selected values

	DoubleSignCheckHeight int64 `mapstructure:"double_sign_check_height"`
}

// DefaultConsensusConfig returns a default configuration for the consensus service.
func DefaultConsensusConfig() *ConsensusConfig {
	return &ConsensusConfig{
		WalPath:                          filepath.Join(DefaultDataDir, "cs.wal", "wal"),
		TimeoutPropose:                   3000 * time.Millisecond,
		TimeoutProposeDelta:              500 * time.Millisecond,
		TimeoutPrevote:                   1000 * time.Millisecond,
		TimeoutPrevoteDelta:              500 * time.Millisecond,
		TimeoutPrecommit:                 1000 * time.Millisecond,
		TimeoutPrecommitDelta:            500 * time.Millisecond,
		TimeoutCommit:                    1000 * time.Millisecond,
		SkipTimeoutCommit:                false,
		CreateEmptyBlocks:                true,
		CreateEmptyBlocksInterval:        0 * time.Second,
		PeerGossipSleepDuration:          100 * time.Millisecond,
		PeerQueryMaj23SleepDuration:      2000 * time.Millisecond,
		PeerGossipIntraloopSleepDuration: 0 * time.Second,
		DoubleSignCheckHeight:            int64(0),
	}
}

// TestConsensusConfig returns a configuration for testing the consensus service.
func TestConsensusConfig() *ConsensusConfig {
	cfg := DefaultConsensusConfig()
	cfg.TimeoutPropose = 40 * time.Millisecond
	cfg.TimeoutProposeDelta = 1 * time.Millisecond
	cfg.TimeoutPrevote = 10 * time.Millisecond
	cfg.TimeoutPrevoteDelta = 1 * time.Millisecond
	cfg.TimeoutPrecommit = 10 * time.Millisecond
	cfg.TimeoutPrecommitDelta = 1 * time.Millisecond
	// NOTE: when modifying, make sure to update time_iota_ms (testGenesisFmt) in toml.go
	cfg.TimeoutCommit = 0
	cfg.SkipTimeoutCommit = true //nolint:staiccheck
	cfg.PeerGossipSleepDuration = 5 * time.Millisecond
	cfg.PeerQueryMaj23SleepDuration = 250 * time.Millisecond
	cfg.DoubleSignCheckHeight = int64(0)
	return cfg
}

// WaitForTxs returns true if the consensus should wait for transactions before entering the propose step.
func (cfg *ConsensusConfig) WaitForTxs() bool {
	return !cfg.CreateEmptyBlocks || cfg.CreateEmptyBlocksInterval > 0
}

func timeoutTime(baseTimeout, timeoutDelta time.Duration, round int32) time.Duration {
	timeout := baseTimeout.Nanoseconds() + timeoutDelta.Nanoseconds()*int64(round)
	return time.Duration(timeout) * time.Nanosecond
}

// Propose returns the amount of time to wait for a proposal.
func (cfg *ConsensusConfig) Propose(round int32) time.Duration {
	return timeoutTime(cfg.TimeoutPropose, cfg.TimeoutProposeDelta, round)
}

// Prevote returns the amount of time to wait for straggler votes after receiving any +2/3 prevotes.
func (cfg *ConsensusConfig) Prevote(round int32) time.Duration {
	return timeoutTime(cfg.TimeoutPrevote, cfg.TimeoutPrevoteDelta, round)
}

// Precommit returns the amount of time to wait for straggler votes after receiving any +2/3 precommits.
func (cfg *ConsensusConfig) Precommit(round int32) time.Duration {
	return timeoutTime(cfg.TimeoutPrecommit, cfg.TimeoutPrecommitDelta, round)
}

// Commit returns the amount of time to wait for straggler votes after receiving +2/3 precommits
// for a single block (ie. a commit).
func (cfg *ConsensusConfig) Commit(t time.Time) time.Time {
	return t.Add(cfg.TimeoutCommit)
}

// WalFile returns the full path to the write-ahead log file.
func (cfg *ConsensusConfig) WalFile() string {
	if cfg.walFile != "" {
		return cfg.walFile
	}
	return rootify(cfg.WalPath, cfg.RootDir)
}

// SetWalFile sets the path to the write-ahead log file.
func (cfg *ConsensusConfig) SetWalFile(walFile string) {
	cfg.walFile = walFile
}

// ValidateBasic performs basic validation (checking param bounds, etc.) and
// returns an error if any check fails.
func (cfg *ConsensusConfig) ValidateBasic() error {
	if cfg.TimeoutPropose < 0 {
		return cmterrors.ErrNegativeField{Field: "timeout_propose"}
	}
	if cfg.TimeoutProposeDelta < 0 {
		return cmterrors.ErrNegativeField{Field: "timeout_propose_delta"}
	}
	if cfg.TimeoutPrevote < 0 {
		return cmterrors.ErrNegativeField{Field: "timeout_prevote"}
	}
	if cfg.TimeoutPrevoteDelta < 0 {
		return cmterrors.ErrNegativeField{Field: "timeout_prevote_delta"}
	}
	if cfg.TimeoutPrecommit < 0 {
		return cmterrors.ErrNegativeField{Field: "timeout_precommit"}
	}
	if cfg.TimeoutPrecommitDelta < 0 {
		return cmterrors.ErrNegativeField{Field: "timeout_precommit_delta"}
	}
	if cfg.TimeoutCommit < 0 {
		return cmterrors.ErrNegativeField{Field: "timeout_commit"}
	}
	if cfg.CreateEmptyBlocksInterval < 0 {
		return cmterrors.ErrNegativeField{Field: "create_empty_blocks_interval"}
	}
	if cfg.PeerGossipSleepDuration < 0 {
		return cmterrors.ErrNegativeField{Field: "peer_gossip_sleep_duration"}
	}
	if cfg.PeerQueryMaj23SleepDuration < 0 {
		return cmterrors.ErrNegativeField{Field: "peer_query_maj23_sleep_duration"}
	}
	if cfg.DoubleSignCheckHeight < 0 {
		return cmterrors.ErrNegativeField{Field: "double_sign_check_height"}
	}
	return nil
}

// -----------------------------------------------------------------------------
// StorageConfig

// StorageConfig allows more fine-grained control over certain storage-related
// behavior.
type StorageConfig struct {
	// Set to false to ensure ABCI responses are persisted. ABCI responses are
	// required for `/block_results` RPC queries, and to reindex events in the
	// command-line tool.
	DiscardABCIResponses bool `mapstructure:"discard_abci_responses"`
	// Configuration related to storage pruning.
	Pruning *PruningConfig `mapstructure:"pruning"`
	// Compaction on pruning - enable or disable in-process compaction.
	// If the DB backend supports it, this will force the DB to compact
	// the database levels and save on storage space. Setting this to true
	// is most beneficial when used in combination with pruning as it will
	// phyisically delete the entries marked for deletion.
	// false by default (forcing compaction is disabled).
	Compact bool `mapstructure:"compact"`
	// Compaction interval - number of blocks to try explicit compaction on.
	// This parameter should be tuned depending on the number of items
	// you expect to delete between two calls to forced compaction.
	// If your retain height is 1 block, it is too much of an overhead
	// to try compaction every block. But it should also not be a very
	// large multiple of your retain height as it might occur bigger overheads.
	// 1000 by default.
	CompactionInterval int64 `mapstructure:"compaction_interval"`

	// The representation of keys in the database.
	// The current representation of keys in Comet's stores is considered to be v1
	// Users can experiment with a different layout by setting this field to v2.
	// Not that this is an experimental feature and switching back from v2 to v1
	// is not supported by CometBFT.
	ExperimentalKeyLayout string `mapstructure:"experimental_db_key_layout"`
}

// DefaultStorageConfig returns the default configuration options relating to
// CometBFT storage optimization.
func DefaultStorageConfig() *StorageConfig {
	return &StorageConfig{
		DiscardABCIResponses:  false,
		Pruning:               DefaultPruningConfig(),
		Compact:               false,
		CompactionInterval:    1000,
		ExperimentalKeyLayout: "v1",
	}
}

// TestStorageConfig returns storage configuration that can be used for
// testing.
func TestStorageConfig() *StorageConfig {
	return &StorageConfig{
		DiscardABCIResponses: false,
		Pruning:              TestPruningConfig(),
	}
}

func (cfg *StorageConfig) ValidateBasic() error {
	if err := cfg.Pruning.ValidateBasic(); err != nil {
		return fmt.Errorf("error in [pruning] section: %w", err)
	}
	if cfg.ExperimentalKeyLayout != "v1" && cfg.ExperimentalKeyLayout != "v2" {
		return fmt.Errorf("unsupported version of DB Key layout, expected v1 or v2, got %s", cfg.ExperimentalKeyLayout)
	}
	return nil
}

// -----------------------------------------------------------------------------
// TxIndexConfig
// Remember that Event has the following structure:
// type: [
//
//	key: value,
//	...
//
// ]
//
// CompositeKeys are constructed by `type.key`
// TxIndexConfig defines the configuration for the transaction indexer,
// including composite keys to index.
type TxIndexConfig struct {
	// What indexer to use for transactions
	//
	// Options:
	//   1) "null"
	//   2) "kv" (default) - the simplest possible indexer,
	//      backed by key-value storage (defaults to levelDB; see DBBackend).
	//   3) "psql" - the indexer services backed by PostgreSQL.
	Indexer string `mapstructure:"indexer"`

	// The PostgreSQL connection configuration, the connection format:
	// postgresql://<user>:<password>@<host>:<port>/<db>?<opts>
	PsqlConn string `mapstructure:"psql-conn"`

	// The PostgreSQL table that stores indexed blocks.
	TableBlocks string `mapstructure:"table_blocks"`
	// The PostgreSQL table that stores indexed transaction results.
	TableTxResults string `mapstructure:"table_tx_results"`
	// The PostgreSQL table that stores indexed events.
	TableEvents string `mapstructure:"table_events"`
	// The PostgreSQL table that stores indexed attributes.
	TableAttributes string `mapstructure:"table_attributes"`
}

// DefaultTxIndexConfig returns a default configuration for the transaction indexer.
func DefaultTxIndexConfig() *TxIndexConfig {
	return &TxIndexConfig{
		Indexer: "kv",
	}
}

// TestTxIndexConfig returns a default configuration for the transaction indexer.
func TestTxIndexConfig() *TxIndexConfig {
	return DefaultTxIndexConfig()
}

// -----------------------------------------------------------------------------
// InstrumentationConfig

// InstrumentationConfig defines the configuration for metrics reporting.
type InstrumentationConfig struct {
	// When true, Prometheus metrics are served under /metrics on
	// PrometheusListenAddr.
	// Check out the documentation for the list of available metrics.
	Prometheus bool `mapstructure:"prometheus"`

	// Address to listen for Prometheus collector(s) connections.
	PrometheusListenAddr string `mapstructure:"prometheus_listen_addr"`

	// Maximum number of simultaneous connections.
	// If you want to accept a larger number than the default, make sure
	// you increase your OS limits.
	// 0 - unlimited.
	MaxOpenConnections int `mapstructure:"max_open_connections"`

	// Instrumentation namespace.
	Namespace string `mapstructure:"namespace"`
}

// DefaultInstrumentationConfig returns a default configuration for metrics
// reporting.
func DefaultInstrumentationConfig() *InstrumentationConfig {
	return &InstrumentationConfig{
		Prometheus:           false,
		PrometheusListenAddr: ":26660",
		MaxOpenConnections:   3,
		Namespace:            "cometbft",
	}
}

// TestInstrumentationConfig returns a default configuration for metrics
// reporting.
func TestInstrumentationConfig() *InstrumentationConfig {
	return DefaultInstrumentationConfig()
}

// ValidateBasic performs basic validation (checking param bounds, etc.) and
// returns an error if any check fails.
func (cfg *InstrumentationConfig) ValidateBasic() error {
	if cfg.MaxOpenConnections < 0 {
		return cmterrors.ErrNegativeField{Field: "max_open_connections"}
	}
	return nil
}

func (cfg *InstrumentationConfig) IsPrometheusEnabled() bool {
	return cfg.Prometheus && cfg.PrometheusListenAddr != ""
}

// -----------------------------------------------------------------------------
// Utils

// helper function to make config creation independent of root dir.
func rootify(path, root string) string {
	if filepath.IsAbs(path) {
		return path
	}
	return filepath.Join(root, path)
}

// -----------------------------------------------------------------------------
// Moniker

var defaultMoniker = getDefaultMoniker()

// getDefaultMoniker returns a default moniker, which is the host name. If runtime
// fails to get the host name, "anonymous" will be returned.
func getDefaultMoniker() string {
	moniker, err := os.Hostname()
	if err != nil {
		moniker = "anonymous"
	}
	return moniker
}

// -----------------------------------------------------------------------------
// PruningConfig

type PruningConfig struct {
	// The time period between automated background pruning operations.
	Interval time.Duration `mapstructure:"interval"`
	// Data companion-related pruning configuration.
	DataCompanion *DataCompanionPruningConfig `mapstructure:"data_companion"`
}

func DefaultPruningConfig() *PruningConfig {
	return &PruningConfig{
		Interval:      DefaultPruningInterval,
		DataCompanion: DefaultDataCompanionPruningConfig(),
	}
}

func TestPruningConfig() *PruningConfig {
	return &PruningConfig{
		Interval:      DefaultPruningInterval,
		DataCompanion: TestDataCompanionPruningConfig(),
	}
}

func (cfg *PruningConfig) ValidateBasic() error {
	if cfg.Interval <= 0 {
		return errors.New("interval must be > 0")
	}
	if err := cfg.DataCompanion.ValidateBasic(); err != nil {
		return fmt.Errorf("error in [data_companion] section: %w", err)
	}
	return nil
}

// -----------------------------------------------------------------------------
// DataCompanionPruningConfig

type DataCompanionPruningConfig struct {
	// Whether automatic pruning respects values set by the data companion.
	// Disabled by default. All other parameters in this section are ignored
	// when this is disabled.
	//
	// If disabled, only the application retain height will influence block
	// pruning (but not block results pruning). Only enabling this at a later
	// stage will potentially mean that blocks below the application-set retain
	// height at the time will not be available to the data companion.
	Enabled bool `mapstructure:"enabled"`
	// The initial value for the data companion block retain height if the data
	// companion has not yet explicitly set one. If the data companion has
	// already set a block retain height, this is ignored.
	InitialBlockRetainHeight int64 `mapstructure:"initial_block_retain_height"`
	// The initial value for the data companion block results retain height if
	// the data companion has not yet explicitly set one. If the data companion
	// has already set a block results retain height, this is ignored.
	InitialBlockResultsRetainHeight int64 `mapstructure:"initial_block_results_retain_height"`
}

func DefaultDataCompanionPruningConfig() *DataCompanionPruningConfig {
	return &DataCompanionPruningConfig{
		Enabled:                         false,
		InitialBlockRetainHeight:        0,
		InitialBlockResultsRetainHeight: 0,
	}
}

func TestDataCompanionPruningConfig() *DataCompanionPruningConfig {
	return &DataCompanionPruningConfig{
		Enabled:                         false,
		InitialBlockRetainHeight:        0,
		InitialBlockResultsRetainHeight: 0,
	}
}

func (cfg *DataCompanionPruningConfig) ValidateBasic() error {
	if !cfg.Enabled {
		return nil
	}
	if cfg.InitialBlockRetainHeight < 0 {
		return errors.New("initial_block_retain_height cannot be negative")
	}
	if cfg.InitialBlockResultsRetainHeight < 0 {
		return errors.New("initial_block_results_retain_height cannot be negative")
	}
	return nil
}

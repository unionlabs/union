package config

import (
	"encoding/hex"
	"errors"
	"fmt"
	"net/http"
	"os"
	"path/filepath"
	"time"
)

const (
	// FuzzModeDrop is a mode in which we randomly drop reads/writes, connections or sleep
	FuzzModeDrop = iota
	// FuzzModeDelay is a mode in which we randomly sleep
	FuzzModeDelay

	// LogFormatPlain is a format for colored text
	LogFormatPlain = "plain"
	// LogFormatJSON is a format for json output
	LogFormatJSON = "json"

	// DefaultLogLevel defines a default log level as INFO.
	DefaultLogLevel = "info"

	// Mempool versions. V1 is prioritized mempool (deprecated), v0 is regular mempool.
	// Default is v0.
	MempoolV0 = "v0"
	MempoolV1 = "v1"
)

// NOTE: Most of the structs & relevant comments + the
// default configuration options were used to manually
// generate the config.toml. Please reflect any changes
// made here in the defaultConfigTemplate constant in
// config/toml.go
// NOTE: libs/cli must know to look in the config dir!
var (
	DefaultTendermintDir = ".cometbft"
	defaultConfigDir     = "config"
	defaultDataDir       = "data"

	defaultConfigFileName  = "config.toml"
	defaultGenesisJSONName = "genesis.json"

	defaultPrivValKeyName   = "priv_validator_key.json"
	defaultPrivValStateName = "priv_validator_state.json"

	defaultNodeKeyName  = "node_key.json"
	defaultAddrBookName = "addrbook.json"

	defaultConfigFilePath   = filepath.Join(defaultConfigDir, defaultConfigFileName)
	defaultGenesisJSONPath  = filepath.Join(defaultConfigDir, defaultGenesisJSONName)
	defaultPrivValKeyPath   = filepath.Join(defaultConfigDir, defaultPrivValKeyName)
	defaultPrivValStatePath = filepath.Join(defaultDataDir, defaultPrivValStateName)

	defaultNodeKeyPath  = filepath.Join(defaultConfigDir, defaultNodeKeyName)
	defaultAddrBookPath = filepath.Join(defaultConfigDir, defaultAddrBookName)

	minSubscriptionBufferSize     = 100
	defaultSubscriptionBufferSize = 200
)

// Config defines the top level configuration for a CometBFT node
type Config struct {
	// Top level options use an anonymous struct
	BaseConfig `mapstructure:",squash"`

	// Options for services
	RPC       *RPCConfig       `mapstructure:"rpc"`
	P2P       *P2PConfig       `mapstructure:"p2p"`
	Mempool   *MempoolConfig   `mapstructure:"mempool"`
	StateSync *StateSyncConfig `mapstructure:"statesync"`
	BlockSync *BlockSyncConfig `mapstructure:"blocksync"`
	//TODO(williambanfield): remove this field once v0.37 is released.
	// https://github.com/tendermint/tendermint/issues/9279
	DeprecatedFastSyncConfig map[interface{}]interface{} `mapstructure:"fastsync"`
	Consensus                *ConsensusConfig            `mapstructure:"consensus"`
	Storage                  *StorageConfig              `mapstructure:"storage"`
	TxIndex                  *TxIndexConfig              `mapstructure:"tx_index"`
	Instrumentation          *InstrumentationConfig      `mapstructure:"instrumentation"`
}

// DefaultConfig returns a default configuration for a CometBFT node
func DefaultConfig() *Config {
	return &Config{
		BaseConfig:      DefaultBaseConfig(),
		RPC:             DefaultRPCConfig(),
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

// TestConfig returns a configuration that can be used for testing
func TestConfig() *Config {
	return &Config{
		BaseConfig:      TestBaseConfig(),
		RPC:             TestRPCConfig(),
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

// SetRoot sets the RootDir for all Config structs
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
		return fmt.Errorf("error in [rpc] section: %w", err)
	}
	if err := cfg.P2P.ValidateBasic(); err != nil {
		return fmt.Errorf("error in [p2p] section: %w", err)
	}
	if err := cfg.Mempool.ValidateBasic(); err != nil {
		return fmt.Errorf("error in [mempool] section: %w", err)
	}
	if err := cfg.StateSync.ValidateBasic(); err != nil {
		return fmt.Errorf("error in [statesync] section: %w", err)
	}
	if err := cfg.BlockSync.ValidateBasic(); err != nil {
		return fmt.Errorf("error in [blocksync] section: %w", err)
	}
	if err := cfg.Consensus.ValidateBasic(); err != nil {
		return fmt.Errorf("error in [consensus] section: %w", err)
	}
	if err := cfg.Instrumentation.ValidateBasic(); err != nil {
		return fmt.Errorf("error in [instrumentation] section: %w", err)
	}
	return nil
}

func (cfg *Config) CheckDeprecated() []string {
	var warnings []string
	if cfg.Mempool.Version == MempoolV1 {
		warnings = append(warnings, "prioritized mempool detected. This version of the mempool will be removed in the next major release.")
	}
	if cfg.Mempool.TTLNumBlocks != 0 {
		warnings = append(warnings, "prioritized mempool key detected. This key, together with this version of the mempool, will be removed in the next major release.")
	}
	if cfg.Mempool.TTLDuration != 0 {
		warnings = append(warnings, "prioritized mempool key detected. This key, together with this version of the mempool, will be removed in the next major release.")
	}
	if !cfg.BaseConfig.BlockSyncMode {
		warnings = append(warnings, "disabled block_sync key detected. BlockSync will be enabled unconditionally in the next major release and this key will be removed.")
	}
	if cfg.DeprecatedFastSyncConfig != nil {
		warnings = append(warnings, "[fastsync] table detected. This section has been renamed to [blocksync]. The values in this deprecated section will be disregarded.")
	}
	if cfg.BaseConfig.DeprecatedFastSyncMode != nil {
		warnings = append(warnings, "fast_sync key detected. This key has been renamed to block_sync. The value of this deprecated key will be disregarded.")
	}
	return warnings
}

//-----------------------------------------------------------------------------
// BaseConfig

// BaseConfig defines the base configuration for a CometBFT node
type BaseConfig struct { //nolint: maligned
	// chainID is unexposed and immutable but here for convenience
	chainID string

	// The root directory for all data.
	// This should be set in viper so it can unmarshal into this struct
	RootDir string `mapstructure:"home"`

	// TCP or UNIX socket address of the ABCI application,
	// or the name of an ABCI application compiled in with the CometBFT binary
	ProxyApp string `mapstructure:"proxy_app"`

	// A custom human readable name for this node
	Moniker string `mapstructure:"moniker"`

	// If this node is many blocks behind the tip of the chain, Blocksync
	// allows them to catchup quickly by downloading blocks in parallel
	// and verifying their commits
	// Deprecated: BlockSync will be enabled unconditionally in the next major release.
	BlockSyncMode bool `mapstructure:"block_sync"`

	//TODO(williambanfield): remove this field once v0.37 is released.
	// https://github.com/tendermint/tendermint/issues/9279
	DeprecatedFastSyncMode interface{} `mapstructure:"fast_sync"`

	// Database backend: goleveldb | cleveldb | boltdb | rocksdb
	// * goleveldb (github.com/syndtr/goleveldb - most popular implementation)
	//   - pure go
	//   - stable
	// * cleveldb (uses levigo wrapper)
	//   - fast
	//   - requires gcc
	//   - use cleveldb build tag (go build -tags cleveldb)
	// * boltdb (uses etcd's fork of bolt - github.com/etcd-io/bbolt)
	//   - EXPERIMENTAL
	//   - may be faster is some use-cases (random reads - indexer)
	//   - use boltdb build tag (go build -tags boltdb)
	// * rocksdb (uses github.com/tecbot/gorocksdb)
	//   - EXPERIMENTAL
	//   - requires gcc
	//   - use rocksdb build tag (go build -tags rocksdb)
	// * badgerdb (uses github.com/dgraph-io/badger)
	//   - EXPERIMENTAL
	//   - use badgerdb build tag (go build -tags badgerdb)
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

// DefaultBaseConfig returns a default base configuration for a CometBFT node
func DefaultBaseConfig() BaseConfig {
	return BaseConfig{
		Genesis:            defaultGenesisJSONPath,
		PrivValidatorKey:   defaultPrivValKeyPath,
		PrivValidatorState: defaultPrivValStatePath,
		NodeKey:            defaultNodeKeyPath,
		Moniker:            defaultMoniker,
		ProxyApp:           "tcp://127.0.0.1:26658",
		ABCI:               "socket",
		LogLevel:           DefaultLogLevel,
		LogFormat:          LogFormatPlain,
		BlockSyncMode:      true,
		FilterPeers:        false,
		DBBackend:          "goleveldb",
		DBPath:             "data",
	}
}

// TestBaseConfig returns a base configuration for testing a CometBFT node
func TestBaseConfig() BaseConfig {
	cfg := DefaultBaseConfig()
	cfg.chainID = "cometbft_test"
	cfg.ProxyApp = "kvstore"
	cfg.BlockSyncMode = false
	cfg.DBBackend = "memdb"
	return cfg
}

func (cfg BaseConfig) ChainID() string {
	return cfg.chainID
}

// GenesisFile returns the full path to the genesis.json file
func (cfg BaseConfig) GenesisFile() string {
	return rootify(cfg.Genesis, cfg.RootDir)
}

// PrivValidatorKeyFile returns the full path to the priv_validator_key.json file
func (cfg BaseConfig) PrivValidatorKeyFile() string {
	return rootify(cfg.PrivValidatorKey, cfg.RootDir)
}

// PrivValidatorFile returns the full path to the priv_validator_state.json file
func (cfg BaseConfig) PrivValidatorStateFile() string {
	return rootify(cfg.PrivValidatorState, cfg.RootDir)
}

// NodeKeyFile returns the full path to the node_key.json file
func (cfg BaseConfig) NodeKeyFile() string {
	return rootify(cfg.NodeKey, cfg.RootDir)
}

// DBDir returns the full path to the database directory
func (cfg BaseConfig) DBDir() string {
	return rootify(cfg.DBPath, cfg.RootDir)
}

// ValidateBasic performs basic validation (checking param bounds, etc.) and
// returns an error if any check fails.
func (cfg BaseConfig) ValidateBasic() error {
	switch cfg.LogFormat {
	case LogFormatPlain, LogFormatJSON:
	default:
		return errors.New("unknown log_format (must be 'plain' or 'json')")
	}
	return nil
}

//-----------------------------------------------------------------------------
// RPCConfig

// RPCConfig defines the configuration options for the CometBFT RPC server
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

	// TCP or UNIX socket address for the gRPC server to listen on
	// NOTE: This server only supports /broadcast_tx_commit
	GRPCListenAddress string `mapstructure:"grpc_laddr"`

	// Maximum number of simultaneous connections.
	// Does not include RPC (HTTP&WebSocket) connections. See max_open_connections
	// If you want to accept a larger number than the default, make sure
	// you increase your OS limits.
	// 0 - unlimited.
	GRPCMaxOpenConnections int `mapstructure:"grpc_max_open_connections"`

	// Activate unsafe RPC commands like /dial_persistent_peers and /unsafe_flush_mempool
	Unsafe bool `mapstructure:"unsafe"`

	// Maximum number of simultaneous connections (including WebSocket).
	// Does not include gRPC connections. See grpc_max_open_connections
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

	// Maximum number of unique queries a given client can /subscribe to
	// If you're using GRPC (or Local RPC client) and /broadcast_tx_commit, set
	// to the estimated maximum number of broadcast_tx_commit calls per block.
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
	PprofListenAddress string `mapstructure:"pprof_laddr"`
}

// DefaultRPCConfig returns a default configuration for the RPC server
func DefaultRPCConfig() *RPCConfig {
	return &RPCConfig{
		ListenAddress:          "tcp://127.0.0.1:26657",
		CORSAllowedOrigins:     []string{},
		CORSAllowedMethods:     []string{http.MethodHead, http.MethodGet, http.MethodPost},
		CORSAllowedHeaders:     []string{"Origin", "Accept", "Content-Type", "X-Requested-With", "X-Server-Time"},
		GRPCListenAddress:      "",
		GRPCMaxOpenConnections: 900,

		Unsafe:             false,
		MaxOpenConnections: 900,

		MaxSubscriptionClients:    100,
		MaxSubscriptionsPerClient: 5,
		SubscriptionBufferSize:    defaultSubscriptionBufferSize,
		TimeoutBroadcastTxCommit:  10 * time.Second,
		WebSocketWriteBufferSize:  defaultSubscriptionBufferSize,

		MaxBodyBytes:   int64(1000000), // 1MB
		MaxHeaderBytes: 1 << 20,        // same as the net/http default

		TLSCertFile: "",
		TLSKeyFile:  "",
	}
}

// TestRPCConfig returns a configuration for testing the RPC server
func TestRPCConfig() *RPCConfig {
	cfg := DefaultRPCConfig()
	cfg.ListenAddress = "tcp://127.0.0.1:36657"
	cfg.GRPCListenAddress = "tcp://127.0.0.1:36658"
	cfg.Unsafe = true
	return cfg
}

// ValidateBasic performs basic validation (checking param bounds, etc.) and
// returns an error if any check fails.
func (cfg *RPCConfig) ValidateBasic() error {
	if cfg.GRPCMaxOpenConnections < 0 {
		return errors.New("grpc_max_open_connections can't be negative")
	}
	if cfg.MaxOpenConnections < 0 {
		return errors.New("max_open_connections can't be negative")
	}
	if cfg.MaxSubscriptionClients < 0 {
		return errors.New("max_subscription_clients can't be negative")
	}
	if cfg.MaxSubscriptionsPerClient < 0 {
		return errors.New("max_subscriptions_per_client can't be negative")
	}
	if cfg.SubscriptionBufferSize < minSubscriptionBufferSize {
		return fmt.Errorf(
			"experimental_subscription_buffer_size must be >= %d",
			minSubscriptionBufferSize,
		)
	}
	if cfg.WebSocketWriteBufferSize < cfg.SubscriptionBufferSize {
		return fmt.Errorf(
			"experimental_websocket_write_buffer_size must be >= experimental_subscription_buffer_size (%d)",
			cfg.SubscriptionBufferSize,
		)
	}
	if cfg.TimeoutBroadcastTxCommit < 0 {
		return errors.New("timeout_broadcast_tx_commit can't be negative")
	}
	if cfg.MaxBodyBytes < 0 {
		return errors.New("max_body_bytes can't be negative")
	}
	if cfg.MaxHeaderBytes < 0 {
		return errors.New("max_header_bytes can't be negative")
	}
	return nil
}

// IsCorsEnabled returns true if cross-origin resource sharing is enabled.
func (cfg *RPCConfig) IsCorsEnabled() bool {
	return len(cfg.CORSAllowedOrigins) != 0
}

func (cfg RPCConfig) KeyFile() string {
	path := cfg.TLSKeyFile
	if filepath.IsAbs(path) {
		return path
	}
	return rootify(filepath.Join(defaultConfigDir, path), cfg.RootDir)
}

func (cfg RPCConfig) CertFile() string {
	path := cfg.TLSCertFile
	if filepath.IsAbs(path) {
		return path
	}
	return rootify(filepath.Join(defaultConfigDir, path), cfg.RootDir)
}

func (cfg RPCConfig) IsTLSEnabled() bool {
	return cfg.TLSCertFile != "" && cfg.TLSKeyFile != ""
}

//-----------------------------------------------------------------------------
// P2PConfig

// P2PConfig defines the configuration options for the CometBFT peer-to-peer networking layer
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

	// UPNP port forwarding
	UPNP bool `mapstructure:"upnp"`

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
	// FUzz connection
	TestFuzz       bool            `mapstructure:"test_fuzz"`
	TestFuzzConfig *FuzzConnConfig `mapstructure:"test_fuzz_config"`
}

// DefaultP2PConfig returns a default configuration for the peer-to-peer layer
func DefaultP2PConfig() *P2PConfig {
	return &P2PConfig{
		ListenAddress:                "tcp://0.0.0.0:26656",
		ExternalAddress:              "",
		UPNP:                         false,
		AddrBook:                     defaultAddrBookPath,
		AddrBookStrict:               true,
		MaxNumInboundPeers:           40,
		MaxNumOutboundPeers:          10,
		PersistentPeersMaxDialPeriod: 0 * time.Second,
		FlushThrottleTimeout:         100 * time.Millisecond,
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

// TestP2PConfig returns a configuration for testing the peer-to-peer layer
func TestP2PConfig() *P2PConfig {
	cfg := DefaultP2PConfig()
	cfg.ListenAddress = "tcp://127.0.0.1:36656"
	cfg.FlushThrottleTimeout = 10 * time.Millisecond
	cfg.AllowDuplicateIP = true
	return cfg
}

// AddrBookFile returns the full path to the address book
func (cfg *P2PConfig) AddrBookFile() string {
	return rootify(cfg.AddrBook, cfg.RootDir)
}

// ValidateBasic performs basic validation (checking param bounds, etc.) and
// returns an error if any check fails.
func (cfg *P2PConfig) ValidateBasic() error {
	if cfg.MaxNumInboundPeers < 0 {
		return errors.New("max_num_inbound_peers can't be negative")
	}
	if cfg.MaxNumOutboundPeers < 0 {
		return errors.New("max_num_outbound_peers can't be negative")
	}
	if cfg.FlushThrottleTimeout < 0 {
		return errors.New("flush_throttle_timeout can't be negative")
	}
	if cfg.PersistentPeersMaxDialPeriod < 0 {
		return errors.New("persistent_peers_max_dial_period can't be negative")
	}
	if cfg.MaxPacketMsgPayloadSize < 0 {
		return errors.New("max_packet_msg_payload_size can't be negative")
	}
	if cfg.SendRate < 0 {
		return errors.New("send_rate can't be negative")
	}
	if cfg.RecvRate < 0 {
		return errors.New("recv_rate can't be negative")
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

//-----------------------------------------------------------------------------
// MempoolConfig

// MempoolConfig defines the configuration options for the CometBFT mempool
type MempoolConfig struct {
	// Mempool version to use:
	//  1) "v0" - (default) FIFO mempool.
	//  2) "v1" - prioritized mempool (deprecated; will be removed in the next release).
	Version string `mapstructure:"version"`
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
	// Limit the total size of all txs in the mempool.
	// This only accounts for raw transactions (e.g. given 1MB transactions and
	// max_txs_bytes=5MB, mempool will only accept 5 transactions).
	MaxTxsBytes int64 `mapstructure:"max_txs_bytes"`
	// Size of the cache (used to filter transactions we saw earlier) in transactions
	CacheSize int `mapstructure:"cache_size"`
	// Do not remove invalid transactions from the cache (default: false)
	// Set to true if it's not possible for any invalid transaction to become
	// valid again in the future.
	KeepInvalidTxsInCache bool `mapstructure:"keep-invalid-txs-in-cache"`
	// Maximum size of a single transaction
	// NOTE: the max size of a tx transmitted over the network is {max_tx_bytes}.
	MaxTxBytes int `mapstructure:"max_tx_bytes"`
	// Maximum size of a batch of transactions to send to a peer
	// Including space needed by encoding (one varint per transaction).
	// XXX: Unused due to https://github.com/tendermint/tendermint/issues/5796
	MaxBatchBytes int `mapstructure:"max_batch_bytes"`

	// TTLDuration, if non-zero, defines the maximum amount of time a transaction
	// can exist for in the mempool.
	//
	// Note, if TTLNumBlocks is also defined, a transaction will be removed if it
	// has existed in the mempool at least TTLNumBlocks number of blocks or if it's
	// insertion time into the mempool is beyond TTLDuration.
	//
	// Deprecated: Only used by priority mempool, which will be removed in the
	// next major release.
	TTLDuration time.Duration `mapstructure:"ttl-duration"`

	// TTLNumBlocks, if non-zero, defines the maximum number of blocks a transaction
	// can exist for in the mempool.
	//
	// Note, if TTLDuration is also defined, a transaction will be removed if it
	// has existed in the mempool at least TTLNumBlocks number of blocks or if
	// it's insertion time into the mempool is beyond TTLDuration.
	//
	// Deprecated: Only used by priority mempool, which will be removed in the
	// next major release.
	TTLNumBlocks int64 `mapstructure:"ttl-num-blocks"`
}

// DefaultMempoolConfig returns a default configuration for the CometBFT mempool
func DefaultMempoolConfig() *MempoolConfig {
	return &MempoolConfig{
		Version:   MempoolV0,
		Recheck:   true,
		Broadcast: true,
		WalPath:   "",
		// Each signature verification takes .5ms, Size reduced until we implement
		// ABCI Recheck
		Size:         5000,
		MaxTxsBytes:  1024 * 1024 * 1024, // 1GB
		CacheSize:    10000,
		MaxTxBytes:   1024 * 1024, // 1MB
		TTLDuration:  0 * time.Second,
		TTLNumBlocks: 0,
	}
}

// TestMempoolConfig returns a configuration for testing the CometBFT mempool
func TestMempoolConfig() *MempoolConfig {
	cfg := DefaultMempoolConfig()
	cfg.CacheSize = 1000
	return cfg
}

// WalDir returns the full path to the mempool's write-ahead log
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
	if cfg.Size < 0 {
		return errors.New("size can't be negative")
	}
	if cfg.MaxTxsBytes < 0 {
		return errors.New("max_txs_bytes can't be negative")
	}
	if cfg.CacheSize < 0 {
		return errors.New("cache_size can't be negative")
	}
	if cfg.MaxTxBytes < 0 {
		return errors.New("max_tx_bytes can't be negative")
	}
	return nil
}

//-----------------------------------------------------------------------------
// StateSyncConfig

// StateSyncConfig defines the configuration for the CometBFT state sync service
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

// DefaultStateSyncConfig returns a default configuration for the state sync service
func DefaultStateSyncConfig() *StateSyncConfig {
	return &StateSyncConfig{
		TrustPeriod:         168 * time.Hour,
		DiscoveryTime:       15 * time.Second,
		ChunkRequestTimeout: 10 * time.Second,
		ChunkFetchers:       4,
	}
}

// TestStateSyncConfig returns a default configuration for the state sync service
func TestStateSyncConfig() *StateSyncConfig {
	return DefaultStateSyncConfig()
}

// ValidateBasic performs basic validation.
func (cfg *StateSyncConfig) ValidateBasic() error {
	if cfg.Enable {
		if len(cfg.RPCServers) == 0 {
			return errors.New("rpc_servers is required")
		}

		if len(cfg.RPCServers) < 2 {
			return errors.New("at least two rpc_servers entries is required")
		}

		for _, server := range cfg.RPCServers {
			if len(server) == 0 {
				return errors.New("found empty rpc_servers entry")
			}
		}

		if cfg.DiscoveryTime != 0 && cfg.DiscoveryTime < 5*time.Second {
			return errors.New("discovery time must be 0s or greater than five seconds")
		}

		if cfg.TrustPeriod <= 0 {
			return errors.New("trusted_period is required")
		}

		if cfg.TrustHeight <= 0 {
			return errors.New("trusted_height is required")
		}

		if len(cfg.TrustHash) == 0 {
			return errors.New("trusted_hash is required")
		}

		_, err := hex.DecodeString(cfg.TrustHash)
		if err != nil {
			return fmt.Errorf("invalid trusted_hash: %w", err)
		}

		if cfg.ChunkRequestTimeout < 5*time.Second {
			return errors.New("chunk_request_timeout must be at least 5 seconds")
		}

		if cfg.ChunkFetchers <= 0 {
			return errors.New("chunk_fetchers is required")
		}
	}

	return nil
}

//-----------------------------------------------------------------------------
// BlockSyncConfig

// BlockSyncConfig (formerly known as FastSync) defines the configuration for the CometBFT block sync service
type BlockSyncConfig struct {
	Version string `mapstructure:"version"`
}

// DefaultBlockSyncConfig returns a default configuration for the block sync service
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
	case "v0":
		return nil
	case "v1", "v2":
		return fmt.Errorf("blocksync version %s has been deprecated. Please use v0 instead", cfg.Version)
	default:
		return fmt.Errorf("unknown blocksync version %s", cfg.Version)
	}
}

//-----------------------------------------------------------------------------
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
	TimeoutCommit time.Duration `mapstructure:"timeout_commit"`

	// Make progress as soon as we have all the precommits (as if TimeoutCommit = 0)
	SkipTimeoutCommit bool `mapstructure:"skip_timeout_commit"`

	// EmptyBlocks mode and possible interval between empty blocks
	CreateEmptyBlocks         bool          `mapstructure:"create_empty_blocks"`
	CreateEmptyBlocksInterval time.Duration `mapstructure:"create_empty_blocks_interval"`

	// Reactor sleep duration parameters
	PeerGossipSleepDuration     time.Duration `mapstructure:"peer_gossip_sleep_duration"`
	PeerQueryMaj23SleepDuration time.Duration `mapstructure:"peer_query_maj23_sleep_duration"`

	DoubleSignCheckHeight int64 `mapstructure:"double_sign_check_height"`
}

// DefaultConsensusConfig returns a default configuration for the consensus service
func DefaultConsensusConfig() *ConsensusConfig {
	return &ConsensusConfig{
		WalPath:                     filepath.Join(defaultDataDir, "cs.wal", "wal"),
		TimeoutPropose:              3000 * time.Millisecond,
		TimeoutProposeDelta:         500 * time.Millisecond,
		TimeoutPrevote:              1000 * time.Millisecond,
		TimeoutPrevoteDelta:         500 * time.Millisecond,
		TimeoutPrecommit:            1000 * time.Millisecond,
		TimeoutPrecommitDelta:       500 * time.Millisecond,
		TimeoutCommit:               1000 * time.Millisecond,
		SkipTimeoutCommit:           false,
		CreateEmptyBlocks:           true,
		CreateEmptyBlocksInterval:   0 * time.Second,
		PeerGossipSleepDuration:     100 * time.Millisecond,
		PeerQueryMaj23SleepDuration: 2000 * time.Millisecond,
		DoubleSignCheckHeight:       int64(0),
	}
}

// TestConsensusConfig returns a configuration for testing the consensus service
func TestConsensusConfig() *ConsensusConfig {
	cfg := DefaultConsensusConfig()
	cfg.TimeoutPropose = 40 * time.Millisecond
	cfg.TimeoutProposeDelta = 1 * time.Millisecond
	cfg.TimeoutPrevote = 10 * time.Millisecond
	cfg.TimeoutPrevoteDelta = 1 * time.Millisecond
	cfg.TimeoutPrecommit = 10 * time.Millisecond
	cfg.TimeoutPrecommitDelta = 1 * time.Millisecond
	// NOTE: when modifying, make sure to update time_iota_ms (testGenesisFmt) in toml.go
	cfg.TimeoutCommit = 10 * time.Millisecond
	cfg.SkipTimeoutCommit = true
	cfg.PeerGossipSleepDuration = 5 * time.Millisecond
	cfg.PeerQueryMaj23SleepDuration = 250 * time.Millisecond
	cfg.DoubleSignCheckHeight = int64(0)
	return cfg
}

// WaitForTxs returns true if the consensus should wait for transactions before entering the propose step
func (cfg *ConsensusConfig) WaitForTxs() bool {
	return !cfg.CreateEmptyBlocks || cfg.CreateEmptyBlocksInterval > 0
}

// Propose returns the amount of time to wait for a proposal
func (cfg *ConsensusConfig) Propose(round int32) time.Duration {
	return time.Duration(
		cfg.TimeoutPropose.Nanoseconds()+cfg.TimeoutProposeDelta.Nanoseconds()*int64(round),
	) * time.Nanosecond
}

// Prevote returns the amount of time to wait for straggler votes after receiving any +2/3 prevotes
func (cfg *ConsensusConfig) Prevote(round int32) time.Duration {
	return time.Duration(
		cfg.TimeoutPrevote.Nanoseconds()+cfg.TimeoutPrevoteDelta.Nanoseconds()*int64(round),
	) * time.Nanosecond
}

// Precommit returns the amount of time to wait for straggler votes after receiving any +2/3 precommits
func (cfg *ConsensusConfig) Precommit(round int32) time.Duration {
	return time.Duration(
		cfg.TimeoutPrecommit.Nanoseconds()+cfg.TimeoutPrecommitDelta.Nanoseconds()*int64(round),
	) * time.Nanosecond
}

// Commit returns the amount of time to wait for straggler votes after receiving +2/3 precommits
// for a single block (ie. a commit).
func (cfg *ConsensusConfig) Commit(t time.Time) time.Time {
	return t.Add(cfg.TimeoutCommit)
}

// WalFile returns the full path to the write-ahead log file
func (cfg *ConsensusConfig) WalFile() string {
	if cfg.walFile != "" {
		return cfg.walFile
	}
	return rootify(cfg.WalPath, cfg.RootDir)
}

// SetWalFile sets the path to the write-ahead log file
func (cfg *ConsensusConfig) SetWalFile(walFile string) {
	cfg.walFile = walFile
}

// ValidateBasic performs basic validation (checking param bounds, etc.) and
// returns an error if any check fails.
func (cfg *ConsensusConfig) ValidateBasic() error {
	if cfg.TimeoutPropose < 0 {
		return errors.New("timeout_propose can't be negative")
	}
	if cfg.TimeoutProposeDelta < 0 {
		return errors.New("timeout_propose_delta can't be negative")
	}
	if cfg.TimeoutPrevote < 0 {
		return errors.New("timeout_prevote can't be negative")
	}
	if cfg.TimeoutPrevoteDelta < 0 {
		return errors.New("timeout_prevote_delta can't be negative")
	}
	if cfg.TimeoutPrecommit < 0 {
		return errors.New("timeout_precommit can't be negative")
	}
	if cfg.TimeoutPrecommitDelta < 0 {
		return errors.New("timeout_precommit_delta can't be negative")
	}
	if cfg.TimeoutCommit < 0 {
		return errors.New("timeout_commit can't be negative")
	}
	if cfg.CreateEmptyBlocksInterval < 0 {
		return errors.New("create_empty_blocks_interval can't be negative")
	}
	if cfg.PeerGossipSleepDuration < 0 {
		return errors.New("peer_gossip_sleep_duration can't be negative")
	}
	if cfg.PeerQueryMaj23SleepDuration < 0 {
		return errors.New("peer_query_maj23_sleep_duration can't be negative")
	}
	if cfg.DoubleSignCheckHeight < 0 {
		return errors.New("double_sign_check_height can't be negative")
	}
	return nil
}

//-----------------------------------------------------------------------------
// StorageConfig

// StorageConfig allows more fine-grained control over certain storage-related
// behavior.
type StorageConfig struct {
	// Set to false to ensure ABCI responses are persisted. ABCI responses are
	// required for `/block_results` RPC queries, and to reindex events in the
	// command-line tool.
	DiscardABCIResponses bool `mapstructure:"discard_abci_responses"`
}

// DefaultStorageConfig returns the default configuration options relating to
// CometBFT storage optimization.
func DefaultStorageConfig() *StorageConfig {
	return &StorageConfig{
		DiscardABCIResponses: false,
	}
}

// TestStorageConfig returns storage configuration that can be used for
// testing.
func TestStorageConfig() *StorageConfig {
	return &StorageConfig{
		DiscardABCIResponses: false,
	}
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

//-----------------------------------------------------------------------------
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
		return errors.New("max_open_connections can't be negative")
	}
	return nil
}

//-----------------------------------------------------------------------------
// Utils

// helper function to make config creation independent of root dir
func rootify(path, root string) string {
	if filepath.IsAbs(path) {
		return path
	}
	return filepath.Join(root, path)
}

//-----------------------------------------------------------------------------
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

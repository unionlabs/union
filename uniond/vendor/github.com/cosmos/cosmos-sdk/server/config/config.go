package config

import (
	"fmt"
	"math"

	"github.com/spf13/viper"

	pruningtypes "cosmossdk.io/store/pruning/types"

	"github.com/cosmos/cosmos-sdk/telemetry"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

const (
	defaultMinGasPrices = ""

	// DefaultAPIAddress defines the default address to bind the API server to.
	DefaultAPIAddress = "tcp://localhost:1317"

	// DefaultGRPCAddress defines the default address to bind the gRPC server to.
	DefaultGRPCAddress = "localhost:9090"

	// DefaultGRPCMaxRecvMsgSize defines the default gRPC max message size in
	// bytes the server can receive.
	DefaultGRPCMaxRecvMsgSize = 1024 * 1024 * 10

	// DefaultGRPCMaxSendMsgSize defines the default gRPC max message size in
	// bytes the server can send.
	DefaultGRPCMaxSendMsgSize = math.MaxInt32
)

// BaseConfig defines the server's basic configuration
type BaseConfig struct {
	// The minimum gas prices a validator is willing to accept for processing a
	// transaction. A transaction's fees must meet the minimum of any denomination
	// specified in this config (e.g. 0.25token1;0.0001token2).
	MinGasPrices string `mapstructure:"minimum-gas-prices"`

	// The maximum amount of gas a grpc/Rest query may consume.
	// If set to 0, it is unbounded.
	QueryGasLimit uint64 `mapstructure:"query-gas-limit"`

	Pruning           string `mapstructure:"pruning"`
	PruningKeepRecent string `mapstructure:"pruning-keep-recent"`
	PruningInterval   string `mapstructure:"pruning-interval"`

	// HaltHeight contains a non-zero block height at which a node will gracefully
	// halt and shutdown that can be used to assist upgrades and testing.
	//
	// Note: Commitment of state will be attempted on the corresponding block.
	HaltHeight uint64 `mapstructure:"halt-height"`

	// HaltTime contains a non-zero minimum block time (in Unix seconds) at which
	// a node will gracefully halt and shutdown that can be used to assist
	// upgrades and testing.
	//
	// Note: Commitment of state will be attempted on the corresponding block.
	HaltTime uint64 `mapstructure:"halt-time"`

	// MinRetainBlocks defines the minimum block height offset from the current
	// block being committed, such that blocks past this offset may be pruned
	// from CometBFT. It is used as part of the process of determining the
	// ResponseCommit.RetainHeight value during ABCI Commit. A value of 0 indicates
	// that no blocks should be pruned.
	//
	// This configuration value is only responsible for pruning CometBFT blocks.
	// It has no bearing on application state pruning which is determined by the
	// "pruning-*" configurations.
	//
	// Note: CometBFT block pruning is dependent on this parameter in conjunction
	// with the unbonding (safety threshold) period, state pruning and state sync
	// snapshot parameters to determine the correct minimum value of
	// ResponseCommit.RetainHeight.
	MinRetainBlocks uint64 `mapstructure:"min-retain-blocks"`

	// InterBlockCache enables inter-block caching.
	InterBlockCache bool `mapstructure:"inter-block-cache"`

	// IndexEvents defines the set of events in the form {eventType}.{attributeKey},
	// which informs CometBFT what to index. If empty, all events will be indexed.
	IndexEvents []string `mapstructure:"index-events"`

	// IavlCacheSize set the size of the iavl tree cache.
	IAVLCacheSize uint64 `mapstructure:"iavl-cache-size"`

	// IAVLDisableFastNode enables or disables the fast sync node.
	IAVLDisableFastNode bool `mapstructure:"iavl-disable-fastnode"`

	// AppDBBackend defines the type of Database to use for the application and snapshots databases.
	// An empty string indicates that the CometBFT config's DBBackend value should be used.
	AppDBBackend string `mapstructure:"app-db-backend"`
}

// APIConfig defines the API listener configuration.
type APIConfig struct {
	// Enable defines if the API server should be enabled.
	Enable bool `mapstructure:"enable"`

	// Swagger defines if swagger documentation should automatically be registered.
	Swagger bool `mapstructure:"swagger"`

	// EnableUnsafeCORS defines if CORS should be enabled (unsafe - use it at your own risk)
	EnableUnsafeCORS bool `mapstructure:"enabled-unsafe-cors"`

	// Address defines the API server to listen on
	Address string `mapstructure:"address"`

	// MaxOpenConnections defines the number of maximum open connections
	MaxOpenConnections uint `mapstructure:"max-open-connections"`

	// RPCReadTimeout defines the CometBFT RPC read timeout (in seconds)
	RPCReadTimeout uint `mapstructure:"rpc-read-timeout"`

	// RPCWriteTimeout defines the CometBFT RPC write timeout (in seconds)
	RPCWriteTimeout uint `mapstructure:"rpc-write-timeout"`

	// RPCMaxBodyBytes defines the CometBFT maximum request body (in bytes)
	RPCMaxBodyBytes uint `mapstructure:"rpc-max-body-bytes"`

	// TODO: TLS/Proxy configuration.
	//
	// Ref: https://github.com/cosmos/cosmos-sdk/issues/6420
}

// GRPCConfig defines configuration for the gRPC server.
type GRPCConfig struct {
	// Enable defines if the gRPC server should be enabled.
	Enable bool `mapstructure:"enable"`

	// Address defines the API server to listen on
	Address string `mapstructure:"address"`

	// MaxRecvMsgSize defines the max message size in bytes the server can receive.
	// The default value is 10MB.
	MaxRecvMsgSize int `mapstructure:"max-recv-msg-size"`

	// MaxSendMsgSize defines the max message size in bytes the server can send.
	// The default value is math.MaxInt32.
	MaxSendMsgSize int `mapstructure:"max-send-msg-size"`
}

// StateSyncConfig defines the state sync snapshot configuration.
type StateSyncConfig struct {
	// SnapshotInterval sets the interval at which state sync snapshots are taken.
	// 0 disables snapshots.
	SnapshotInterval uint64 `mapstructure:"snapshot-interval"`

	// SnapshotKeepRecent sets the number of recent state sync snapshots to keep.
	// 0 keeps all snapshots.
	SnapshotKeepRecent uint32 `mapstructure:"snapshot-keep-recent"`
}

// MempoolConfig defines the configurations for the SDK built-in app-side mempool
// implementations.
type MempoolConfig struct {
	// MaxTxs defines the behavior of the mempool. A negative value indicates
	// the mempool is disabled entirely, zero indicates that the mempool is
	// unbounded in how many txs it may contain, and a positive value indicates
	// the maximum amount of txs it may contain.
	MaxTxs int `mapstructure:"max-txs"`
}

// State Streaming configuration
type (
	// StreamingConfig defines application configuration for external streaming services
	StreamingConfig struct {
		ABCI ABCIListenerConfig `mapstructure:"abci"`
	}
	// ABCIListenerConfig defines application configuration for ABCIListener streaming service
	ABCIListenerConfig struct {
		Keys          []string `mapstructure:"keys"`
		Plugin        string   `mapstructure:"plugin"`
		StopNodeOnErr bool     `mapstructure:"stop-node-on-err"`
	}
)

// Config defines the server's top level configuration
type Config struct {
	BaseConfig `mapstructure:",squash"`

	// Telemetry defines the application telemetry configuration
	Telemetry telemetry.Config `mapstructure:"telemetry"`
	API       APIConfig        `mapstructure:"api"`
	GRPC      GRPCConfig       `mapstructure:"grpc"`
	StateSync StateSyncConfig  `mapstructure:"state-sync"`
	Streaming StreamingConfig  `mapstructure:"streaming"`
	Mempool   MempoolConfig    `mapstructure:"mempool"`
}

// SetMinGasPrices sets the validator's minimum gas prices.
func (c *Config) SetMinGasPrices(gasPrices sdk.DecCoins) {
	c.MinGasPrices = gasPrices.String()
}

// GetMinGasPrices returns the validator's minimum gas prices based on the set configuration.
func (c *Config) GetMinGasPrices() sdk.DecCoins {
	if c.MinGasPrices == "" {
		return sdk.DecCoins{}
	}

	gasPrices, err := sdk.ParseDecCoins(c.MinGasPrices)
	if err != nil {
		panic(fmt.Sprintf("invalid minimum gas prices: %v", err))
	}

	return gasPrices
}

// DefaultConfig returns server's default configuration.
func DefaultConfig() *Config {
	return &Config{
		BaseConfig: BaseConfig{
			MinGasPrices:        defaultMinGasPrices,
			QueryGasLimit:       0,
			InterBlockCache:     true,
			Pruning:             pruningtypes.PruningOptionDefault,
			PruningKeepRecent:   "0",
			PruningInterval:     "0",
			MinRetainBlocks:     0,
			IndexEvents:         make([]string, 0),
			IAVLCacheSize:       781250,
			IAVLDisableFastNode: false,
			AppDBBackend:        "",
		},
		Telemetry: telemetry.Config{
			Enabled:      false,
			GlobalLabels: [][]string{},
		},
		API: APIConfig{
			Enable:             false,
			Swagger:            false,
			Address:            DefaultAPIAddress,
			MaxOpenConnections: 1000,
			RPCReadTimeout:     10,
			RPCMaxBodyBytes:    1000000,
		},
		GRPC: GRPCConfig{
			Enable:         true,
			Address:        DefaultGRPCAddress,
			MaxRecvMsgSize: DefaultGRPCMaxRecvMsgSize,
			MaxSendMsgSize: DefaultGRPCMaxSendMsgSize,
		},
		StateSync: StateSyncConfig{
			SnapshotInterval:   0,
			SnapshotKeepRecent: 2,
		},
		Streaming: StreamingConfig{
			ABCI: ABCIListenerConfig{
				Keys:          []string{},
				StopNodeOnErr: true,
			},
		},
		Mempool: MempoolConfig{
			MaxTxs: -1,
		},
	}
}

// GetConfig returns a fully parsed Config object.
func GetConfig(v *viper.Viper) (Config, error) {
	conf := DefaultConfig()
	if err := v.Unmarshal(conf); err != nil {
		return Config{}, fmt.Errorf("error extracting app config: %w", err)
	}
	return *conf, nil
}

// ValidateBasic returns an error if min-gas-prices field is empty in BaseConfig. Otherwise, it returns nil.
func (c Config) ValidateBasic() error {
	if c.BaseConfig.MinGasPrices == "" {
		return sdkerrors.ErrAppConfig.Wrap("set min gas price in app.toml or flag or env variable")
	}
	if c.Pruning == pruningtypes.PruningOptionEverything && c.StateSync.SnapshotInterval > 0 {
		return sdkerrors.ErrAppConfig.Wrapf(
			"cannot enable state sync snapshots with '%s' pruning setting", pruningtypes.PruningOptionEverything,
		)
	}

	return nil
}

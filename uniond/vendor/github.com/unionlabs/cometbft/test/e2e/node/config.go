package main

import (
	"fmt"
	"time"

	"github.com/BurntSushi/toml"

	"github.com/cometbft/cometbft/test/e2e/app"
	cmterrors "github.com/cometbft/cometbft/types/errors"
)

// Config is the application configuration.
type Config struct {
	ChainID          string                      `toml:"chain_id"`
	Listen           string                      `toml:"listen"`
	Protocol         string                      `toml:"protocol"`
	Dir              string                      `toml:"dir"`
	Mode             string                      `toml:"mode"`
	PersistInterval  uint64                      `toml:"persist_interval"`
	SnapshotInterval uint64                      `toml:"snapshot_interval"`
	RetainBlocks     uint64                      `toml:"retain_blocks"`
	ValidatorUpdates map[string]map[string]uint8 `toml:"validator_update"`
	PrivValServer    string                      `toml:"privval_server"`
	PrivValKey       string                      `toml:"privval_key"`
	PrivValState     string                      `toml:"privval_state"`
	KeyType          string                      `toml:"key_type"`

	PrepareProposalDelay time.Duration `toml:"prepare_proposal_delay"`
	ProcessProposalDelay time.Duration `toml:"process_proposal_delay"`
	CheckTxDelay         time.Duration `toml:"check_tx_delay"`
	FinalizeBlockDelay   time.Duration `toml:"finalize_block_delay"`
	VoteExtensionDelay   time.Duration `toml:"vote_extension_delay"`

	VoteExtensionSize          uint  `toml:"vote_extension_size"`
	VoteExtensionsEnableHeight int64 `toml:"vote_extensions_enable_height"`
	VoteExtensionsUpdateHeight int64 `toml:"vote_extensions_update_height"`

	ABCIRequestsLoggingEnabled bool `toml:"abci_requests_logging_enabled"`

	ExperimentalKeyLayout string `toml:"experimental_db_key_layout"`

	Compact bool `toml:"compact"`

	CompactionInterval bool `toml:"compaction_interval"`

	DiscardABCIResponses bool `toml:"discard_abci_responses"`

	Indexer string `toml:"indexer"`

	PbtsEnableHeight int64 `toml:"pbts_enable_height"`
	PbtsUpdateHeight int64 `toml:"pbts_update_height"`
}

// App extracts out the application specific configuration parameters.
func (cfg *Config) App() *app.Config {
	return &app.Config{
		Dir:                        cfg.Dir,
		SnapshotInterval:           cfg.SnapshotInterval,
		RetainBlocks:               cfg.RetainBlocks,
		KeyType:                    cfg.KeyType,
		ValidatorUpdates:           cfg.ValidatorUpdates,
		PersistInterval:            cfg.PersistInterval,
		PrepareProposalDelay:       cfg.PrepareProposalDelay,
		ProcessProposalDelay:       cfg.ProcessProposalDelay,
		CheckTxDelay:               cfg.CheckTxDelay,
		FinalizeBlockDelay:         cfg.FinalizeBlockDelay,
		VoteExtensionDelay:         cfg.VoteExtensionDelay,
		VoteExtensionSize:          cfg.VoteExtensionSize,
		VoteExtensionsEnableHeight: cfg.VoteExtensionsEnableHeight,
		VoteExtensionsUpdateHeight: cfg.VoteExtensionsUpdateHeight,
		ABCIRequestsLoggingEnabled: cfg.ABCIRequestsLoggingEnabled,
		PbtsEnableHeight:           cfg.PbtsEnableHeight,
		PbtsUpdateHeight:           cfg.PbtsUpdateHeight,
	}
}

// LoadConfig loads the configuration from disk.
func LoadConfig(file string) (*Config, error) {
	cfg := &Config{
		Listen:          "unix:///var/run/app.sock",
		Protocol:        "socket",
		PersistInterval: 1,
	}
	_, err := toml.DecodeFile(file, &cfg)
	if err != nil {
		return nil, fmt.Errorf("failed to load config from %q: %w", file, err)
	}
	return cfg, cfg.Validate()
}

// Validate validates the configuration. We don't do exhaustive config
// validation here, instead relying on Testnet.Validate() to handle it.
//
//nolint:goconst
func (cfg Config) Validate() error {
	switch {
	case cfg.ChainID == "":
		return cmterrors.ErrRequiredField{Field: "chain_id"}
	case cfg.Listen == "" && cfg.Protocol != "builtin" && cfg.Protocol != "builtin_connsync":
		return cmterrors.ErrRequiredField{Field: "listen"}
	default:
		return nil
	}
}

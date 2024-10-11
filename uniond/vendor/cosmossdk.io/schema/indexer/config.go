package indexer

// Config species the configuration passed to an indexer initialization function.
// It includes both common configuration options related to include or excluding
// parts of the data stream as well as indexer specific options under the config
// subsection.
//
// NOTE: it is an error for an indexer to change its common options, such as adding
// or removing indexed modules, after the indexer has been initialized because this
// could result in an inconsistent state.
type Config struct {
	// Type is the name of the indexer type as registered with Register.
	Type string `mapstructure:"type" toml:"type" json:"type" comment:"The name of the registered indexer type."`

	// Config are the indexer specific config options specified by the user.
	Config interface{} `mapstructure:"config" toml:"config" json:"config,omitempty" comment:"Indexer specific configuration options."`

	// Filter is the filter configuration for the indexer.
	Filter *FilterConfig `mapstructure:"filter" toml:"filter" json:"filter,omitempty" comment:"Filter configuration for the indexer. Currently UNSUPPORTED!"`
}

// FilterConfig specifies the configuration for filtering the data stream
type FilterConfig struct {
	// ExcludeState specifies that the indexer will not receive state updates.
	ExcludeState bool `mapstructure:"exclude_state" toml:"exclude_state" json:"exclude_state" comment:"Exclude all state updates."`

	// ExcludeEvents specifies that the indexer will not receive events.
	ExcludeEvents bool `mapstructure:"exclude_events" toml:"exclude_events" json:"exclude_events" comment:"Exclude all events."`

	// ExcludeTxs specifies that the indexer will not receive transaction's.
	ExcludeTxs bool `mapstructure:"exclude_txs" toml:"exclude_txs" json:"exclude_txs" comment:"Exclude all transactions."`

	// ExcludeBlockHeaders specifies that the indexer will not receive block headers,
	// although it will still receive StartBlock and Commit callbacks, just without
	// the header data.
	ExcludeBlockHeaders bool `mapstructure:"exclude_block_headers" toml:"exclude_block_headers" json:"exclude_block_headers" comment:"Exclude all block headers."`

	Modules *ModuleFilterConfig `mapstructure:"modules" toml:"modules" json:"modules,omitempty" comment:"Module filter configuration."`
}

// ModuleFilterConfig specifies the configuration for filtering modules.
type ModuleFilterConfig struct {
	// Include specifies a list of modules whose state the indexer will
	// receive state updates for.
	// Only one of include or exclude modules should be specified.
	Include []string `mapstructure:"include" toml:"include" json:"include" comment:"List of modules to include. Only one of include or exclude should be specified."`

	// Exclude specifies a list of modules whose state the indexer will not
	// receive state updates for.
	// Only one of include or exclude modules should be specified.
	Exclude []string `mapstructure:"exclude" toml:"exclude" json:"exclude" comment:"List of modules to exclude. Only one of include or exclude should be specified."`
}

package e2e

import (
	"fmt"
	"os"
	"time"

	"github.com/BurntSushi/toml"
)

// Manifest represents a TOML testnet manifest.
type Manifest struct {
	// IPv6 uses IPv6 networking instead of IPv4. Defaults to IPv4.
	IPv6 bool `toml:"ipv6"`

	// InitialHeight specifies the initial block height, set in genesis. Defaults to 1.
	InitialHeight int64 `toml:"initial_height"`

	// InitialState is an initial set of key/value pairs for the application,
	// set in genesis. Defaults to nothing.
	InitialState map[string]string `toml:"initial_state"`

	// Validators is the initial validator set in genesis, given as node names
	// and power:
	//
	// validators = { validator01 = 10; validator02 = 20; validator03 = 30 }
	//
	// Defaults to all nodes that have mode=validator at power 100. Explicitly
	// specifying an empty set will start with no validators in genesis, and
	// the application must return the validator set in InitChain via the
	// setting validator_update.0 (see below).
	Validators *map[string]int64 `toml:"validators"`

	// ValidatorUpdates is a map of heights to validator names and their power,
	// and will be returned by the ABCI application. For example, the following
	// changes the power of validator01 and validator02 at height 1000:
	//
	// [validator_update.1000]
	// validator01 = 20
	// validator02 = 10
	//
	// Specifying height 0 returns the validator update during InitChain. The
	// application returns the validator updates as-is, i.e. removing a
	// validator must be done by returning it with power 0, and any validators
	// not specified are not changed.
	ValidatorUpdates map[string]map[string]int64 `toml:"validator_update"`

	// Nodes specifies the network nodes. At least one node must be given.
	Nodes map[string]*ManifestNode `toml:"node"`

	// Disable the peer-exchange reactor on all nodes.
	DisablePexReactor bool `toml:"disable_pex"`

	// KeyType sets the curve that will be used by validators.
	// Options are ed25519, secp256k1 and bls12381.
	KeyType string `toml:"key_type"`

	// Evidence indicates the amount of evidence that will be injected into the
	// testnet via the RPC endpoint of a random node. Default is 0
	Evidence int `toml:"evidence"`

	// ABCIProtocol specifies the protocol used to communicate with the ABCI
	// application: "unix", "tcp", "grpc", "builtin" or "builtin_connsync".
	//
	// Defaults to "builtin". "builtin" will build a complete CometBFT node
	// into the application and launch it instead of launching a separate
	// CometBFT process.
	//
	// "builtin_connsync" is basically the same as "builtin", except that it
	// uses a "connection-synchronized" local client creator, which attempts to
	// replicate the same concurrency model locally as the socket client.
	ABCIProtocol string `toml:"abci_protocol"`

	// Add artificial delays to each of the main ABCI calls to mimic computation time
	// of the application
	PrepareProposalDelay time.Duration `toml:"prepare_proposal_delay"`
	ProcessProposalDelay time.Duration `toml:"process_proposal_delay"`
	CheckTxDelay         time.Duration `toml:"check_tx_delay"`
	VoteExtensionDelay   time.Duration `toml:"vote_extension_delay"`
	FinalizeBlockDelay   time.Duration `toml:"finalize_block_delay"`

	// UpgradeVersion specifies to which version nodes need to upgrade.
	// Currently only uncoordinated upgrade is supported
	UpgradeVersion string `toml:"upgrade_version"`

	LoadTxSizeBytes   int `toml:"load_tx_size_bytes"`
	LoadTxBatchSize   int `toml:"load_tx_batch_size"`
	LoadTxConnections int `toml:"load_tx_connections"`
	LoadMaxTxs        int `toml:"load_max_txs"`

	// LogLevel specifies the log level to be set on all nodes.
	LogLevel string `toml:"log_level"`

	// LogFormat specifies the log format to be set on all nodes.
	LogFormat string `toml:"log_format"`

	// Enable or disable Prometheus metrics on all nodes.
	// Defaults to false (disabled).
	Prometheus bool `toml:"prometheus"`

	// BlockMaxBytes specifies the maximum size in bytes of a block. This
	// value will be written to the genesis file of all nodes.
	BlockMaxBytes int64 `toml:"block_max_bytes"`

	// Defines a minimum size for the vote extensions.
	VoteExtensionSize uint `toml:"vote_extension_size"`

	// VoteExtensionsEnableHeight configures the first height during which
	// the chain will use and require vote extension data to be present
	// in precommit messages.
	VoteExtensionsEnableHeight int64 `toml:"vote_extensions_enable_height"`

	// VoteExtensionsUpdateHeight configures the height at which consensus
	// param VoteExtensionsEnableHeight will be set.
	// -1 denotes it is set at genesis.
	// 0 denotes it is set at InitChain.
	VoteExtensionsUpdateHeight int64 `toml:"vote_extensions_update_height"`

	// Upper bound of sleep duration then gossipping votes and block parts
	PeerGossipIntraloopSleepDuration time.Duration `toml:"peer_gossip_intraloop_sleep_duration"`

	// Maximum number of peers to which the node gossips transactions
	ExperimentalMaxGossipConnectionsToPersistentPeers    uint `toml:"experimental_max_gossip_connections_to_persistent_peers"`
	ExperimentalMaxGossipConnectionsToNonPersistentPeers uint `toml:"experimental_max_gossip_connections_to_non_persistent_peers"`

	// Enable or disable e2e tests for CometBFT's expected behavior with respect
	// to ABCI.
	ABCITestsEnabled bool `toml:"abci_tests_enabled"`

	// Default geographical zone ID for simulating latencies, assigned to nodes that don't have a
	// specific zone assigned.
	DefaultZone string `toml:"default_zone"`

	// PbtsEnableHeight configures the first height during which
	// the chain will start using Proposer-Based Timestamps (PBTS)
	// to create and validate new blocks.
	PbtsEnableHeight int64 `toml:"pbts_enable_height"`

	// PbtsUpdateHeight configures the height at which consensus
	// param PbtsEnableHeight will be set.
	// -1 denotes it is set at genesis.
	// 0 denotes it is set at InitChain.
	PbtsUpdateHeight int64 `toml:"pbts_update_height"`
}

// ManifestNode represents a node in a testnet manifest.
type ManifestNode struct {
	// Mode specifies the type of node: "validator", "full", "light" or "seed".
	// Defaults to "validator". Full nodes do not get a signing key (a dummy key
	// is generated), and seed nodes run in seed mode with the PEX reactor enabled.
	Mode string `toml:"mode"`

	// Version specifies which version of CometBFT this node is. Specifying different
	// versions for different nodes allows for testing the interaction of different
	// node's compatibility. Note that in order to use a node at a particular version,
	// there must be a docker image of the test app tagged with this version present
	// on the machine where the test is being run.
	Version string `toml:"version"`

	// Seeds is the list of node names to use as P2P seed nodes. Defaults to none.
	Seeds []string `toml:"seeds"`

	// PersistentPeers is a list of node names to maintain persistent P2P
	// connections to. If neither seeds nor persistent peers are specified,
	// this defaults to all other nodes in the network. For light clients,
	// this relates to the providers the light client is connected to.
	PersistentPeers []string `toml:"persistent_peers"`

	// Database specifies the database backend: "goleveldb", "rocksdb",
	// "pebbledb" or "badgerdb". Defaults to "goleveldb".
	Database string `toml:"database"`

	// PrivvalProtocol specifies the protocol used to sign consensus messages:
	// "file", "unix", or "tcp". Defaults to "file". For unix and tcp, the ABCI
	// application will launch a remote signer client in a separate goroutine.
	// Only nodes with mode=validator will actually make use of this.
	PrivvalProtocol string `toml:"privval_protocol"`

	// StartAt specifies the block height at which the node will be started. The
	// runner will wait for the network to reach at least this block height.
	StartAt int64 `toml:"start_at"`

	// BlockSyncVersion specifies which version of Block Sync to use (currently
	// only "v0", the default value).
	BlockSyncVersion string `toml:"block_sync_version"`

	// StateSync enables state sync. The runner automatically configures trusted
	// block hashes and RPC servers. At least one node in the network must have
	// SnapshotInterval set to non-zero, and the state syncing node must have
	// StartAt set to an appropriate height where a snapshot is available.
	StateSync bool `toml:"state_sync"`

	// PersistInterval specifies the height interval at which the application
	// will persist state to disk. Defaults to 1 (every height), setting this to
	// 0 disables state persistence.
	PersistInterval *uint64 `toml:"persist_interval"`

	// SnapshotInterval specifies the height interval at which the application
	// will take state sync snapshots. Defaults to 0 (disabled).
	SnapshotInterval uint64 `toml:"snapshot_interval"`

	// RetainBlocks specifies the number of recent blocks to retain. Defaults to
	// 0, which retains all blocks. Must be greater that PersistInterval,
	// SnapshotInterval and EvidenceAgeHeight.
	RetainBlocks uint64 `toml:"retain_blocks"`

	// EnableCompanionPruning specifies whether or not storage pruning on the
	// node should take a data companion into account.
	EnableCompanionPruning bool `toml:"enable_companion_pruning"`

	// Perturb lists perturbations to apply to the node after it has been
	// started and synced with the network:
	//
	// disconnect: temporarily disconnects the node from the network
	// kill:       kills the node with SIGKILL then restarts it
	// pause:      temporarily pauses (freezes) the node
	// restart:    restarts the node, shutting it down with SIGTERM
	Perturb []string `toml:"perturb"`

	// SendNoLoad determines if the e2e test should send load to this node.
	// It defaults to false so unless the configured, the node will
	// receive load.
	SendNoLoad bool `toml:"send_no_load"`

	// Geographical zone ID for simulating latencies.
	Zone string `toml:"zone"`

	// ExperimentalKeyLayout sets the key representation in the DB
	ExperimentalKeyLayout string `toml:"experimental_db_key_layout"`

	// Compact triggers compaction on the DB after pruning
	Compact bool `toml:"compact"`

	// CompactionInterval sets the number of blocks at which we trigger compaction
	CompactionInterval int64 `toml:"compaction_interval"`

	// DiscardABCIResponses disables abci rsponses
	DiscardABCIResponses bool `toml:"discard_abci_responses"`

	// Indexer sets the indexer, default kv
	Indexer string `toml:"indexer"`

	// Simulated clock skew for this node
	ClockSkew time.Duration `toml:"clock_skew"`
}

// Save saves the testnet manifest to a file.
func (m Manifest) Save(file string) error {
	f, err := os.Create(file)
	if err != nil {
		return fmt.Errorf("failed to create manifest file %q: %w", file, err)
	}
	return toml.NewEncoder(f).Encode(m)
}

// LoadManifest loads a testnet manifest from a file.
func LoadManifest(file string) (Manifest, error) {
	manifest := Manifest{}
	_, err := toml.DecodeFile(file, &manifest)
	if err != nil {
		return manifest, fmt.Errorf("failed to load testnet manifest %q: %w", file, err)
	}
	return manifest, nil
}

package main

import (
	"bytes"
	"encoding/base64"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strconv"
	"strings"
	"time"

	"github.com/BurntSushi/toml"

	"github.com/cometbft/cometbft/config"
	"github.com/cometbft/cometbft/crypto/ed25519"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/p2p"
	"github.com/cometbft/cometbft/privval"
	e2e "github.com/cometbft/cometbft/test/e2e/pkg"
	"github.com/cometbft/cometbft/test/e2e/pkg/infra"
	"github.com/cometbft/cometbft/types"
)

const (
	AppAddressTCP  = "tcp://127.0.0.1:30000"
	AppAddressUNIX = "unix:///var/run/app.sock"

	PrivvalAddressTCP     = "tcp://0.0.0.0:27559"
	PrivvalAddressUNIX    = "unix:///var/run/privval.sock"
	PrivvalKeyFile        = "config/priv_validator_key.json"
	PrivvalStateFile      = "data/priv_validator_state.json"
	PrivvalDummyKeyFile   = "config/dummy_validator_key.json"
	PrivvalDummyStateFile = "data/dummy_validator_state.json"
)

// Setup sets up the testnet configuration.
func Setup(testnet *e2e.Testnet, infp infra.Provider) error {
	logger.Info("setup", "msg", log.NewLazySprintf("Generating testnet files in %#q", testnet.Dir))

	if err := os.MkdirAll(testnet.Dir, os.ModePerm); err != nil {
		return err
	}

	if err := infp.Setup(); err != nil {
		return err
	}

	genesis, err := MakeGenesis(testnet)
	if err != nil {
		return err
	}

	for _, node := range testnet.Nodes {
		nodeDir := filepath.Join(testnet.Dir, node.Name)

		dirs := []string{
			filepath.Join(nodeDir, "config"),
			filepath.Join(nodeDir, "data"),
			filepath.Join(nodeDir, "data", "app"),
		}
		for _, dir := range dirs {
			// light clients don't need an app directory
			if node.Mode == e2e.ModeLight && strings.Contains(dir, "app") {
				continue
			}
			err := os.MkdirAll(dir, 0o755)
			if err != nil {
				return err
			}
		}

		cfg, err := MakeConfig(node)
		if err != nil {
			return err
		}
		config.WriteConfigFile(filepath.Join(nodeDir, "config", "config.toml"), cfg) // panics

		appCfg, err := MakeAppConfig(node)
		if err != nil {
			return err
		}
		err = os.WriteFile(filepath.Join(nodeDir, "config", "app.toml"), appCfg, 0o644) //nolint:gosec
		if err != nil {
			return err
		}

		if node.Mode == e2e.ModeLight {
			// stop early if a light client
			continue
		}

		err = genesis.SaveAs(filepath.Join(nodeDir, "config", "genesis.json"))
		if err != nil {
			return err
		}

		err = (&p2p.NodeKey{PrivKey: node.NodeKey}).SaveAs(filepath.Join(nodeDir, "config", "node_key.json"))
		if err != nil {
			return err
		}

		(privval.NewFilePV(node.PrivvalKey,
			filepath.Join(nodeDir, PrivvalKeyFile),
			filepath.Join(nodeDir, PrivvalStateFile),
		)).Save()

		// Set up a dummy validator. CometBFT requires a file PV even when not used, so we
		// give it a dummy such that it will fail if it actually tries to use it.
		(privval.NewFilePV(ed25519.GenPrivKey(),
			filepath.Join(nodeDir, PrivvalDummyKeyFile),
			filepath.Join(nodeDir, PrivvalDummyStateFile),
		)).Save()
	}

	if testnet.Prometheus {
		if err := testnet.WritePrometheusConfig(); err != nil {
			return err
		}
	}

	return nil
}

// MakeGenesis generates a genesis document.
func MakeGenesis(testnet *e2e.Testnet) (types.GenesisDoc, error) {
	genesis := types.GenesisDoc{
		GenesisTime:     time.Now(),
		ChainID:         testnet.Name,
		ConsensusParams: types.DefaultConsensusParams(),
		InitialHeight:   testnet.InitialHeight,
	}
	// set the app version to 1
	genesis.ConsensusParams.Version.App = 1
	genesis.ConsensusParams.Evidence.MaxAgeNumBlocks = e2e.EvidenceAgeHeight
	genesis.ConsensusParams.Evidence.MaxAgeDuration = e2e.EvidenceAgeTime
	if testnet.BlockMaxBytes != 0 {
		genesis.ConsensusParams.Block.MaxBytes = testnet.BlockMaxBytes
	}
	if testnet.VoteExtensionsUpdateHeight == -1 {
		genesis.ConsensusParams.Feature.VoteExtensionsEnableHeight = testnet.VoteExtensionsEnableHeight
	}
	if testnet.PbtsUpdateHeight == -1 {
		genesis.ConsensusParams.Feature.PbtsEnableHeight = testnet.PbtsEnableHeight
	}
	for validator, power := range testnet.Validators {
		genesis.Validators = append(genesis.Validators, types.GenesisValidator{
			Name:    validator.Name,
			Address: validator.PrivvalKey.PubKey().Address(),
			PubKey:  validator.PrivvalKey.PubKey(),
			Power:   power,
		})
	}
	// The validator set will be sorted internally by CometBFT ranked by power,
	// but we sort it here as well so that all genesis files are identical.
	sort.Slice(genesis.Validators, func(i, j int) bool {
		return strings.Compare(genesis.Validators[i].Name, genesis.Validators[j].Name) == -1
	})
	if len(testnet.InitialState) > 0 {
		appState, err := json.Marshal(testnet.InitialState)
		if err != nil {
			return genesis, err
		}
		genesis.AppState = appState
	}
	return genesis, genesis.ValidateAndComplete()
}

// MakeConfig generates a CometBFT config for a node.
func MakeConfig(node *e2e.Node) (*config.Config, error) {
	cfg := config.DefaultConfig()
	cfg.Moniker = node.Name
	cfg.ProxyApp = AppAddressTCP

	cfg.RPC.ListenAddress = "tcp://0.0.0.0:26657"
	cfg.RPC.PprofListenAddress = ":6060"

	cfg.GRPC.ListenAddress = "tcp://0.0.0.0:26670"
	cfg.GRPC.VersionService.Enabled = true
	cfg.GRPC.BlockService.Enabled = true
	cfg.GRPC.BlockResultsService.Enabled = true

	cfg.P2P.ExternalAddress = fmt.Sprintf("tcp://%v", node.AddressP2P(false))
	cfg.P2P.AddrBookStrict = false

	cfg.DBBackend = node.Database
	cfg.StateSync.DiscoveryTime = 5 * time.Second
	cfg.BlockSync.Version = node.BlockSyncVersion
	cfg.Consensus.PeerGossipIntraloopSleepDuration = node.Testnet.PeerGossipIntraloopSleepDuration
	cfg.Mempool.ExperimentalMaxGossipConnectionsToNonPersistentPeers = int(node.Testnet.ExperimentalMaxGossipConnectionsToNonPersistentPeers)
	cfg.Mempool.ExperimentalMaxGossipConnectionsToPersistentPeers = int(node.Testnet.ExperimentalMaxGossipConnectionsToPersistentPeers)

	// Assume that full nodes and validators will have a data companion
	// attached, which will need access to the privileged gRPC endpoint.
	if (node.Mode == e2e.ModeValidator || node.Mode == e2e.ModeFull) && node.EnableCompanionPruning {
		cfg.Storage.Pruning.DataCompanion.Enabled = true
		cfg.Storage.Pruning.DataCompanion.InitialBlockRetainHeight = 0
		cfg.Storage.Pruning.DataCompanion.InitialBlockResultsRetainHeight = 0
		cfg.GRPC.Privileged.ListenAddress = "tcp://0.0.0.0:26671"
		cfg.GRPC.Privileged.PruningService.Enabled = true
	}

	switch node.ABCIProtocol {
	case e2e.ProtocolUNIX:
		cfg.ProxyApp = AppAddressUNIX
	case e2e.ProtocolTCP:
		cfg.ProxyApp = AppAddressTCP
	case e2e.ProtocolGRPC:
		cfg.ProxyApp = AppAddressTCP
		cfg.ABCI = "grpc"
	case e2e.ProtocolBuiltin, e2e.ProtocolBuiltinConnSync:
		cfg.ProxyApp = ""
		cfg.ABCI = ""
	default:
		return nil, fmt.Errorf("unexpected ABCI protocol setting %q", node.ABCIProtocol)
	}

	// CometBFT errors if it does not have a privval key set up, regardless of whether
	// it's actually needed (e.g. for remote KMS or non-validators). We set up a dummy
	// key here by default, and use the real key for actual validators that should use
	// the file privval.
	cfg.PrivValidatorListenAddr = ""
	cfg.PrivValidatorKey = PrivvalDummyKeyFile
	cfg.PrivValidatorState = PrivvalDummyStateFile

	switch node.Mode {
	case e2e.ModeValidator:
		switch node.PrivvalProtocol {
		case e2e.ProtocolFile:
			cfg.PrivValidatorKey = PrivvalKeyFile
			cfg.PrivValidatorState = PrivvalStateFile
		case e2e.ProtocolUNIX:
			cfg.PrivValidatorListenAddr = PrivvalAddressUNIX
		case e2e.ProtocolTCP:
			cfg.PrivValidatorListenAddr = PrivvalAddressTCP
		default:
			return nil, fmt.Errorf("invalid privval protocol setting %q", node.PrivvalProtocol)
		}
	case e2e.ModeSeed:
		cfg.P2P.SeedMode = true
		cfg.P2P.PexReactor = true
	case e2e.ModeFull, e2e.ModeLight:
		// Don't need to do anything, since we're using a dummy privval key by default.
	default:
		return nil, fmt.Errorf("unexpected mode %q", node.Mode)
	}

	if node.StateSync {
		cfg.StateSync.Enable = true
		cfg.StateSync.RPCServers = []string{}
		for _, peer := range node.Testnet.ArchiveNodes() {
			if peer.Name == node.Name {
				continue
			}
			cfg.StateSync.RPCServers = append(cfg.StateSync.RPCServers, peer.AddressRPC())
		}
		if len(cfg.StateSync.RPCServers) < 2 {
			return nil, errors.New("unable to find 2 suitable state sync RPC servers")
		}
	}

	cfg.P2P.Seeds = ""
	for _, seed := range node.Seeds {
		if len(cfg.P2P.Seeds) > 0 {
			cfg.P2P.Seeds += ","
		}
		cfg.P2P.Seeds += seed.AddressP2P(true)
	}
	cfg.P2P.PersistentPeers = ""
	for _, peer := range node.PersistentPeers {
		if len(cfg.P2P.PersistentPeers) > 0 {
			cfg.P2P.PersistentPeers += ","
		}
		cfg.P2P.PersistentPeers += peer.AddressP2P(true)
	}
	if node.Testnet.DisablePexReactor {
		cfg.P2P.PexReactor = false
	}

	if node.Testnet.LogLevel != "" {
		cfg.LogLevel = node.Testnet.LogLevel
	}

	if node.Testnet.LogFormat != "" {
		cfg.LogFormat = node.Testnet.LogFormat
	}

	if node.Prometheus {
		cfg.Instrumentation.Prometheus = true
	}

	if node.ExperimentalKeyLayout != "" {
		cfg.Storage.ExperimentalKeyLayout = node.ExperimentalKeyLayout
	}

	if node.Compact {
		cfg.Storage.Compact = node.Compact
	}

	if node.DiscardABCIResponses {
		cfg.Storage.DiscardABCIResponses = node.DiscardABCIResponses
	}

	if node.Indexer != "" {
		cfg.TxIndex.Indexer = node.Indexer
	}

	if node.CompactionInterval != 0 && node.Compact {
		cfg.Storage.CompactionInterval = node.CompactionInterval
	}
	return cfg, nil
}

// MakeAppConfig generates an ABCI application config for a node.
func MakeAppConfig(node *e2e.Node) ([]byte, error) {
	cfg := map[string]any{
		"chain_id":                      node.Testnet.Name,
		"dir":                           "data/app",
		"listen":                        AppAddressUNIX,
		"mode":                          node.Mode,
		"protocol":                      "socket",
		"persist_interval":              node.PersistInterval,
		"snapshot_interval":             node.SnapshotInterval,
		"retain_blocks":                 node.RetainBlocks,
		"key_type":                      node.PrivvalKey.Type(),
		"prepare_proposal_delay":        node.Testnet.PrepareProposalDelay,
		"process_proposal_delay":        node.Testnet.ProcessProposalDelay,
		"check_tx_delay":                node.Testnet.CheckTxDelay,
		"vote_extension_delay":          node.Testnet.VoteExtensionDelay,
		"finalize_block_delay":          node.Testnet.FinalizeBlockDelay,
		"vote_extension_size":           node.Testnet.VoteExtensionSize,
		"vote_extensions_enable_height": node.Testnet.VoteExtensionsEnableHeight,
		"vote_extensions_update_height": node.Testnet.VoteExtensionsUpdateHeight,
		"abci_requests_logging_enabled": node.Testnet.ABCITestsEnabled,
		"pbts_enable_height":            node.Testnet.PbtsEnableHeight,
		"pbts_update_height":            node.Testnet.PbtsUpdateHeight,
	}
	switch node.ABCIProtocol {
	case e2e.ProtocolUNIX:
		cfg["listen"] = AppAddressUNIX
	case e2e.ProtocolTCP:
		cfg["listen"] = AppAddressTCP
	case e2e.ProtocolGRPC:
		cfg["listen"] = AppAddressTCP
		cfg["protocol"] = "grpc"
	case e2e.ProtocolBuiltin, e2e.ProtocolBuiltinConnSync:
		delete(cfg, "listen")
		cfg["protocol"] = string(node.ABCIProtocol)
	default:
		return nil, fmt.Errorf("unexpected ABCI protocol setting %q", node.ABCIProtocol)
	}
	if node.Mode == e2e.ModeValidator {
		switch node.PrivvalProtocol {
		case e2e.ProtocolFile:
		case e2e.ProtocolTCP:
			cfg["privval_server"] = PrivvalAddressTCP
			cfg["privval_key"] = PrivvalKeyFile
			cfg["privval_state"] = PrivvalStateFile
		case e2e.ProtocolUNIX:
			cfg["privval_server"] = PrivvalAddressUNIX
			cfg["privval_key"] = PrivvalKeyFile
			cfg["privval_state"] = PrivvalStateFile
		default:
			return nil, fmt.Errorf("unexpected privval protocol setting %q", node.PrivvalProtocol)
		}
	}

	if len(node.Testnet.ValidatorUpdates) > 0 {
		validatorUpdates := map[string]map[string]int64{}
		for height, validators := range node.Testnet.ValidatorUpdates {
			updateVals := map[string]int64{}
			for node, power := range validators {
				updateVals[base64.StdEncoding.EncodeToString(node.PrivvalKey.PubKey().Bytes())] = power
			}
			validatorUpdates[strconv.FormatInt(height, 10)] = updateVals
		}
		cfg["validator_update"] = validatorUpdates
	}

	var buf bytes.Buffer
	err := toml.NewEncoder(&buf).Encode(cfg)
	if err != nil {
		return nil, fmt.Errorf("failed to generate app config: %w", err)
	}
	return buf.Bytes(), nil
}

// UpdateConfigStateSync updates the state sync config for a node.
func UpdateConfigStateSync(node *e2e.Node, height int64, hash []byte) error {
	cfgPath := filepath.Join(node.Testnet.Dir, node.Name, "config", "config.toml")

	// FIXME Apparently there's no function to simply load a config file without
	// involving the entire Viper apparatus, so we'll just resort to regexps.
	bz, err := os.ReadFile(cfgPath)
	if err != nil {
		return err
	}
	bz = regexp.MustCompile(`(?m)^trust_height =.*`).ReplaceAll(bz, []byte(fmt.Sprintf(`trust_height = %v`, height)))
	bz = regexp.MustCompile(`(?m)^trust_hash =.*`).ReplaceAll(bz, []byte(fmt.Sprintf(`trust_hash = "%X"`, hash)))
	return os.WriteFile(cfgPath, bz, 0o644) //nolint:gosec
}

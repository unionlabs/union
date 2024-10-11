// Package components defines utilities to spin up actual
// beacon node and validator processes as needed by end to end tests.
package components

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"path"
	"strconv"
	"strings"
	"syscall"

	"github.com/bazelbuild/rules_go/go/tools/bazel"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	cmdshared "github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/flags"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/sync/genesis"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/runtime/interop"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/helpers"
	e2e "github.com/prysmaticlabs/prysm/v4/testing/endtoend/params"
	e2etypes "github.com/prysmaticlabs/prysm/v4/testing/endtoend/types"
)

var _ e2etypes.ComponentRunner = (*BeaconNode)(nil)
var _ e2etypes.ComponentRunner = (*BeaconNodeSet)(nil)
var _ e2etypes.MultipleComponentRunners = (*BeaconNodeSet)(nil)
var _ e2etypes.BeaconNodeSet = (*BeaconNodeSet)(nil)

// BeaconNodeSet represents set of beacon nodes.
type BeaconNodeSet struct {
	e2etypes.ComponentRunner
	config  *e2etypes.E2EConfig
	nodes   []e2etypes.ComponentRunner
	enr     string
	ids     []string
	started chan struct{}
}

// SetENR assigns ENR to the set of beacon nodes.
func (s *BeaconNodeSet) SetENR(enr string) {
	s.enr = enr
}

// NewBeaconNodes creates and returns a set of beacon nodes.
func NewBeaconNodes(config *e2etypes.E2EConfig) *BeaconNodeSet {
	return &BeaconNodeSet{
		config:  config,
		started: make(chan struct{}, 1),
	}
}

// Start starts all the beacon nodes in set.
func (s *BeaconNodeSet) Start(ctx context.Context) error {
	if s.enr == "" {
		return errors.New("empty ENR")
	}

	// Create beacon nodes.
	nodes := make([]e2etypes.ComponentRunner, e2e.TestParams.BeaconNodeCount)
	for i := 0; i < e2e.TestParams.BeaconNodeCount; i++ {
		nodes[i] = NewBeaconNode(s.config, i, s.enr)
	}
	s.nodes = nodes

	// Wait for all nodes to finish their job (blocking).
	// Once nodes are ready passed in handler function will be called.
	return helpers.WaitOnNodes(ctx, nodes, func() {
		if s.config.UseFixedPeerIDs {
			for i := 0; i < len(nodes); i++ {
				s.ids = append(s.ids, nodes[i].(*BeaconNode).peerID)
			}
			s.config.PeerIDs = s.ids
		}
		// All nodes started, close channel, so that all services waiting on a set, can proceed.
		close(s.started)
	})
}

// Started checks whether beacon node set is started and all nodes are ready to be queried.
func (s *BeaconNodeSet) Started() <-chan struct{} {
	return s.started
}

// Pause pauses the component and its underlying process.
func (s *BeaconNodeSet) Pause() error {
	for _, n := range s.nodes {
		if err := n.Pause(); err != nil {
			return err
		}
	}
	return nil
}

// Resume resumes the component and its underlying process.
func (s *BeaconNodeSet) Resume() error {
	for _, n := range s.nodes {
		if err := n.Resume(); err != nil {
			return err
		}
	}
	return nil
}

// Stop stops the component and its underlying process.
func (s *BeaconNodeSet) Stop() error {
	for _, n := range s.nodes {
		if err := n.Stop(); err != nil {
			return err
		}
	}
	return nil
}

// PauseAtIndex pauses the component and its underlying process at the desired index.
func (s *BeaconNodeSet) PauseAtIndex(i int) error {
	if i >= len(s.nodes) {
		return errors.Errorf("provided index exceeds slice size: %d >= %d", i, len(s.nodes))
	}
	return s.nodes[i].Pause()
}

// ResumeAtIndex resumes the component and its underlying process at the desired index.
func (s *BeaconNodeSet) ResumeAtIndex(i int) error {
	if i >= len(s.nodes) {
		return errors.Errorf("provided index exceeds slice size: %d >= %d", i, len(s.nodes))
	}
	return s.nodes[i].Resume()
}

// StopAtIndex stops the component and its underlying process at the desired index.
func (s *BeaconNodeSet) StopAtIndex(i int) error {
	if i >= len(s.nodes) {
		return errors.Errorf("provided index exceeds slice size: %d >= %d", i, len(s.nodes))
	}
	return s.nodes[i].Stop()
}

// ComponentAtIndex returns the component at the provided index.
func (s *BeaconNodeSet) ComponentAtIndex(i int) (e2etypes.ComponentRunner, error) {
	if i >= len(s.nodes) {
		return nil, errors.Errorf("provided index exceeds slice size: %d >= %d", i, len(s.nodes))
	}
	return s.nodes[i], nil
}

// BeaconNode represents beacon node.
type BeaconNode struct {
	e2etypes.ComponentRunner
	config  *e2etypes.E2EConfig
	started chan struct{}
	index   int
	enr     string
	peerID  string
	cmd     *exec.Cmd
}

// NewBeaconNode creates and returns a beacon node.
func NewBeaconNode(config *e2etypes.E2EConfig, index int, enr string) *BeaconNode {
	return &BeaconNode{
		config:  config,
		index:   index,
		enr:     enr,
		started: make(chan struct{}, 1),
	}
}

func (node *BeaconNode) saveGenesis(ctx context.Context) (string, error) {
	// The deposit contract starts with an empty trie, we use the BeaconState to "pre-mine" the validator registry,
	g, err := generateGenesis(ctx)
	if err != nil {
		return "", err
	}

	root, err := g.HashTreeRoot(ctx)
	if err != nil {
		return "", err
	}
	lbhr, err := g.LatestBlockHeader().HashTreeRoot()
	if err != nil {
		return "", err
	}
	log.WithField("fork_version", g.Fork().CurrentVersion).
		WithField("latest_block_header.root", fmt.Sprintf("%#x", lbhr)).
		WithField("state_root", fmt.Sprintf("%#x", root)).
		Infof("BeaconState info")

	genesisBytes, err := g.MarshalSSZ()
	if err != nil {
		return "", err
	}
	genesisDir := path.Join(e2e.TestParams.TestPath, fmt.Sprintf("genesis/%d", node.index))
	if err := file.MkdirAll(genesisDir); err != nil {
		return "", err
	}
	genesisPath := path.Join(genesisDir, "genesis.ssz")
	return genesisPath, file.WriteFile(genesisPath, genesisBytes)
}

func (node *BeaconNode) saveConfig() (string, error) {
	cfg := params.BeaconConfig().Copy()
	cfgBytes := params.ConfigToYaml(cfg)
	cfgDir := path.Join(e2e.TestParams.TestPath, fmt.Sprintf("config/%d", node.index))
	if err := file.MkdirAll(cfgDir); err != nil {
		return "", err
	}
	cfgPath := path.Join(cfgDir, "beacon-config.yaml")
	return cfgPath, file.WriteFile(cfgPath, cfgBytes)
}

// Start starts a fresh beacon node, connecting to all passed in beacon nodes.
func (node *BeaconNode) Start(ctx context.Context) error {
	binaryPath, found := bazel.FindBinary("cmd/beacon-chain", "beacon-chain")
	if !found {
		log.Info(binaryPath)
		return errors.New("beacon chain binary not found")
	}

	config, index, enr := node.config, node.index, node.enr
	stdOutFile, err := helpers.DeleteAndCreateFile(e2e.TestParams.LogPath, fmt.Sprintf(e2e.BeaconNodeLogFileName, index))
	if err != nil {
		return err
	}
	expectedNumOfPeers := e2e.TestParams.BeaconNodeCount + e2e.TestParams.LighthouseBeaconNodeCount - 1
	if node.config.TestSync {
		expectedNumOfPeers += 1
	}
	if node.config.TestCheckpointSync {
		expectedNumOfPeers += 1
	}
	jwtPath := path.Join(e2e.TestParams.TestPath, "eth1data/"+strconv.Itoa(node.index)+"/")
	if index == 0 {
		jwtPath = path.Join(e2e.TestParams.TestPath, "eth1data/miner/")
	}
	jwtPath = path.Join(jwtPath, "geth/jwtsecret")

	genesisPath, err := node.saveGenesis(ctx)
	if err != nil {
		return err
	}
	cfgPath, err := node.saveConfig()
	if err != nil {
		return err
	}
	args := []string{
		fmt.Sprintf("--%s=%s", genesis.StatePath.Name, genesisPath),
		fmt.Sprintf("--%s=%s/eth2-beacon-node-%d", cmdshared.DataDirFlag.Name, e2e.TestParams.TestPath, index),
		fmt.Sprintf("--%s=%s", cmdshared.LogFileName.Name, stdOutFile.Name()),
		fmt.Sprintf("--%s=%s", flags.DepositContractFlag.Name, params.BeaconConfig().DepositContractAddress),
		fmt.Sprintf("--%s=%d", flags.RPCPort.Name, e2e.TestParams.Ports.PrysmBeaconNodeRPCPort+index),
		fmt.Sprintf("--%s=http://127.0.0.1:%d", flags.ExecutionEngineEndpoint.Name, e2e.TestParams.Ports.Eth1ProxyPort+index),
		fmt.Sprintf("--%s=%s", flags.ExecutionJWTSecretFlag.Name, jwtPath),
		fmt.Sprintf("--%s=%d", flags.MinSyncPeers.Name, 1),
		fmt.Sprintf("--%s=%d", cmdshared.P2PUDPPort.Name, e2e.TestParams.Ports.PrysmBeaconNodeUDPPort+index),
		fmt.Sprintf("--%s=%d", cmdshared.P2PTCPPort.Name, e2e.TestParams.Ports.PrysmBeaconNodeTCPPort+index),
		fmt.Sprintf("--%s=%d", cmdshared.P2PMaxPeers.Name, expectedNumOfPeers),
		fmt.Sprintf("--%s=%d", flags.MonitoringPortFlag.Name, e2e.TestParams.Ports.PrysmBeaconNodeMetricsPort+index),
		fmt.Sprintf("--%s=%d", flags.GRPCGatewayPort.Name, e2e.TestParams.Ports.PrysmBeaconNodeGatewayPort+index),
		fmt.Sprintf("--%s=%d", flags.ContractDeploymentBlock.Name, 0),
		fmt.Sprintf("--%s=%d", flags.MinPeersPerSubnet.Name, 0),
		fmt.Sprintf("--%s=%d", cmdshared.RPCMaxPageSizeFlag.Name, params.BeaconConfig().MinGenesisActiveValidatorCount),
		fmt.Sprintf("--%s=%s", cmdshared.BootstrapNode.Name, enr),
		fmt.Sprintf("--%s=%s", cmdshared.VerbosityFlag.Name, "debug"),
		fmt.Sprintf("--%s=%d", flags.BlockBatchLimitBurstFactor.Name, 8),
		fmt.Sprintf("--%s=%d", flags.BlobBatchLimitBurstFactor.Name, 16),
		fmt.Sprintf("--%s=%d", flags.BlobBatchLimit.Name, 256),
		fmt.Sprintf("--%s=%s", cmdshared.ChainConfigFileFlag.Name, cfgPath),
		"--" + cmdshared.ValidatorMonitorIndicesFlag.Name + "=1",
		"--" + cmdshared.ValidatorMonitorIndicesFlag.Name + "=2",
		"--" + cmdshared.ForceClearDB.Name,
		"--" + cmdshared.AcceptTosFlag.Name,
		"--" + flags.EnableDebugRPCEndpoints.Name,
	}
	if config.UsePprof {
		args = append(args, "--pprof", fmt.Sprintf("--pprofport=%d", e2e.TestParams.Ports.PrysmBeaconNodePprofPort+index))
	}
	// Only add in the feature flags if we either aren't performing a control test
	// on our features or the beacon index is a multiplier of 2 (idea is to split nodes
	// equally down the line with one group having feature flags and the other without
	// feature flags; this is to allow A-B testing on new features)
	if !config.TestFeature || index != 1 {
		args = append(args, features.E2EBeaconChainFlags...)
	}
	if config.UseBuilder {
		args = append(args, fmt.Sprintf("--%s=%s:%d", flags.MevRelayEndpoint.Name, "http://127.0.0.1", e2e.TestParams.Ports.Eth1ProxyPort+index))
	}
	args = append(args, config.BeaconFlags...)

	cmd := exec.CommandContext(ctx, binaryPath, args...) // #nosec G204 -- Safe
	// Write stderr to log files.
	stderr, err := os.Create(path.Join(e2e.TestParams.LogPath, fmt.Sprintf("beacon_node_%d_stderr.log", index)))
	if err != nil {
		return err
	}
	defer func() {
		if err := stderr.Close(); err != nil {
			log.WithError(err).Error("Failed to close stderr file")
		}
	}()
	cmd.Stderr = stderr
	log.Infof("Starting beacon chain %d with flags: %s", index, strings.Join(args, " "))
	if err = cmd.Start(); err != nil {
		return fmt.Errorf("failed to start beacon node: %w", err)
	}

	if err = helpers.WaitForTextInFile(stdOutFile, "gRPC server listening on port"); err != nil {
		return fmt.Errorf("could not find multiaddr for node %d, this means the node had issues starting: %w", index, err)
	}

	if config.UseFixedPeerIDs {
		peerId, err := helpers.FindFollowingTextInFile(stdOutFile, "Running node with peer id of ")
		if err != nil {
			return fmt.Errorf("could not find peer id: %w", err)
		}
		node.peerID = peerId
	}

	// Mark node as ready.
	close(node.started)

	node.cmd = cmd
	return cmd.Wait()
}

// Started checks whether beacon node is started and ready to be queried.
func (node *BeaconNode) Started() <-chan struct{} {
	return node.started
}

// Pause pauses the component and its underlying process.
func (node *BeaconNode) Pause() error {
	return node.cmd.Process.Signal(syscall.SIGSTOP)
}

// Resume resumes the component and its underlying process.
func (node *BeaconNode) Resume() error {
	return node.cmd.Process.Signal(syscall.SIGCONT)
}

// Stop stops the component and its underlying process.
func (node *BeaconNode) Stop() error {
	return node.cmd.Process.Kill()
}

func (node *BeaconNode) UnderlyingProcess() *os.Process {
	return node.cmd.Process
}

func generateGenesis(ctx context.Context) (state.BeaconState, error) {
	if e2e.TestParams.Eth1GenesisBlock == nil {
		return nil, errors.New("Cannot construct bellatrix block, e2e.TestParams.Eth1GenesisBlock == nil")
	}
	gb := e2e.TestParams.Eth1GenesisBlock
	t := e2e.TestParams.CLGenesisTime
	pcreds := e2e.TestParams.NumberOfExecutionCreds
	nvals := params.BeaconConfig().MinGenesisActiveValidatorCount
	version := e2etypes.GenesisFork()
	return interop.NewPreminedGenesis(ctx, t, nvals, pcreds, version, gb)
}

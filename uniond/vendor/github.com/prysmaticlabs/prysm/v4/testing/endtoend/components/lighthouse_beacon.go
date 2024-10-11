package components

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"path"
	"path/filepath"
	"strconv"
	"strings"
	"syscall"

	"github.com/bazelbuild/rules_go/go/tools/bazel"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/helpers"
	e2e "github.com/prysmaticlabs/prysm/v4/testing/endtoend/params"
	e2etypes "github.com/prysmaticlabs/prysm/v4/testing/endtoend/types"
)

var _ e2etypes.ComponentRunner = (*LighthouseBeaconNode)(nil)
var _ e2etypes.ComponentRunner = (*LighthouseBeaconNodeSet)(nil)
var _ e2etypes.MultipleComponentRunners = (*LighthouseBeaconNodeSet)(nil)
var _ e2etypes.BeaconNodeSet = (*LighthouseBeaconNodeSet)(nil)

// LighthouseBeaconNodeSet represents set of lighthouse beacon nodes.
type LighthouseBeaconNodeSet struct {
	e2etypes.ComponentRunner
	config  *e2etypes.E2EConfig
	enr     string
	started chan struct{}
	nodes   []e2etypes.ComponentRunner
}

// SetENR assigns ENR to the set of beacon nodes.
func (s *LighthouseBeaconNodeSet) SetENR(enr string) {
	s.enr = enr
}

// NewLighthouseBeaconNodes creates and returns a set of lighthouse beacon nodes.
func NewLighthouseBeaconNodes(config *e2etypes.E2EConfig) *LighthouseBeaconNodeSet {
	return &LighthouseBeaconNodeSet{
		config:  config,
		started: make(chan struct{}, 1),
	}
}

// Start starts all the beacon nodes in set.
func (s *LighthouseBeaconNodeSet) Start(ctx context.Context) error {
	if s.enr == "" {
		return errors.New("empty ENR")
	}

	// Create beacon nodes.
	nodes := make([]e2etypes.ComponentRunner, e2e.TestParams.LighthouseBeaconNodeCount)
	for i := 0; i < e2e.TestParams.LighthouseBeaconNodeCount; i++ {
		nodes[i] = NewLighthouseBeaconNode(s.config, i, s.enr)
	}
	s.nodes = nodes

	// Wait for all nodes to finish their job (blocking).
	// Once nodes are ready passed in handler function will be called.
	return helpers.WaitOnNodes(ctx, nodes, func() {
		// All nodes started, close channel, so that all services waiting on a set, can proceed.
		close(s.started)
	})
}

// Started checks whether beacon node set is started and all nodes are ready to be queried.
func (s *LighthouseBeaconNodeSet) Started() <-chan struct{} {
	return s.started
}

// Pause pauses the component and its underlying process.
func (s *LighthouseBeaconNodeSet) Pause() error {
	for _, n := range s.nodes {
		if err := n.Pause(); err != nil {
			return err
		}
	}
	return nil
}

// Resume resumes the component and its underlying process.
func (s *LighthouseBeaconNodeSet) Resume() error {
	for _, n := range s.nodes {
		if err := n.Resume(); err != nil {
			return err
		}
	}
	return nil
}

// Stop stops the component and its underlying process.
func (s *LighthouseBeaconNodeSet) Stop() error {
	for _, n := range s.nodes {
		if err := n.Stop(); err != nil {
			return err
		}
	}
	return nil
}

// PauseAtIndex pauses the component and its underlying process at the desired index.
func (s *LighthouseBeaconNodeSet) PauseAtIndex(i int) error {
	if i >= len(s.nodes) {
		return errors.Errorf("provided index exceeds slice size: %d >= %d", i, len(s.nodes))
	}
	return s.nodes[i].Pause()
}

// ResumeAtIndex resumes the component and its underlying process at the desired index.
func (s *LighthouseBeaconNodeSet) ResumeAtIndex(i int) error {
	if i >= len(s.nodes) {
		return errors.Errorf("provided index exceeds slice size: %d >= %d", i, len(s.nodes))
	}
	return s.nodes[i].Resume()
}

// StopAtIndex stops the component and its underlying process at the desired index.
func (s *LighthouseBeaconNodeSet) StopAtIndex(i int) error {
	if i >= len(s.nodes) {
		return errors.Errorf("provided index exceeds slice size: %d >= %d", i, len(s.nodes))
	}
	return s.nodes[i].Stop()
}

// ComponentAtIndex returns the component at the provided index.
func (s *LighthouseBeaconNodeSet) ComponentAtIndex(i int) (e2etypes.ComponentRunner, error) {
	if i >= len(s.nodes) {
		return nil, errors.Errorf("provided index exceeds slice size: %d >= %d", i, len(s.nodes))
	}
	return s.nodes[i], nil
}

// LighthouseBeaconNode represents a lighthouse beacon node.
type LighthouseBeaconNode struct {
	e2etypes.ComponentRunner
	config  *e2etypes.E2EConfig
	started chan struct{}
	index   int
	enr     string
	cmd     *exec.Cmd
}

// NewLighthouseBeaconNode creates and returns a lighthouse beacon node.
func NewLighthouseBeaconNode(config *e2etypes.E2EConfig, index int, enr string) *LighthouseBeaconNode {
	return &LighthouseBeaconNode{
		config:  config,
		index:   index,
		enr:     enr,
		started: make(chan struct{}, 1),
	}
}

// Start starts a fresh beacon node, connecting to all passed in beacon nodes.
func (node *LighthouseBeaconNode) Start(ctx context.Context) error {
	binaryPath, found := bazel.FindBinary("external/lighthouse", "lighthouse")
	if !found {
		log.Info(binaryPath)
		log.Error("beacon chain binary not found")
	}

	_, index, _ := node.config, node.index, node.enr
	testDir, err := node.createTestnetDir(ctx, index)
	if err != nil {
		return err
	}

	prysmNodeCount := e2e.TestParams.BeaconNodeCount
	jwtPath := path.Join(e2e.TestParams.TestPath, "eth1data/"+strconv.Itoa(node.index+prysmNodeCount)+"/")
	jwtPath = path.Join(jwtPath, "geth/jwtsecret")
	args := []string{
		"beacon_node",
		fmt.Sprintf("--datadir=%s/lighthouse-beacon-node-%d", e2e.TestParams.TestPath, index),
		fmt.Sprintf("--testnet-dir=%s", testDir),
		"--staking",
		"--enr-address=127.0.0.1",
		fmt.Sprintf("--enr-udp-port=%d", e2e.TestParams.Ports.LighthouseBeaconNodeP2PPort+index*2), // multiply by 2 because LH adds 1 for quic4 port
		fmt.Sprintf("--enr-tcp-port=%d", e2e.TestParams.Ports.LighthouseBeaconNodeP2PPort+index*2), // multiply by 2 because LH adds 1 for quic4 port
		fmt.Sprintf("--port=%d", e2e.TestParams.Ports.LighthouseBeaconNodeP2PPort+index*2),         // multiply by 2 because LH adds 1 for quic4 port
		fmt.Sprintf("--http-port=%d", e2e.TestParams.Ports.LighthouseBeaconNodeHTTPPort+index),
		fmt.Sprintf("--target-peers=%d", 10),
		fmt.Sprintf("--execution-endpoint=http://127.0.0.1:%d", e2e.TestParams.Ports.Eth1ProxyPort+prysmNodeCount+index),
		fmt.Sprintf("--jwt-secrets=%s", jwtPath),
		fmt.Sprintf("--boot-nodes=%s", node.enr),
		fmt.Sprintf("--metrics-port=%d", e2e.TestParams.Ports.LighthouseBeaconNodeMetricsPort+index),
		"--metrics",
		"--http",
		"--http-allow-sync-stalled",
		"--enable-private-discovery",
		"--debug-level=debug",
		"--suggested-fee-recipient=0x878705ba3f8bc32fcf7f4caa1a35e72af65cf766",
	}
	if node.config.UseFixedPeerIDs {
		flagVal := strings.Join(node.config.PeerIDs, ",")
		args = append(args,
			fmt.Sprintf("--trusted-peers=%s", flagVal))
	}
	if node.config.UseBuilder {
		args = append(args, fmt.Sprintf("--builder=%s:%d", "http://127.0.0.1", e2e.TestParams.Ports.Eth1ProxyPort+prysmNodeCount+index))
	}
	cmd := exec.CommandContext(ctx, binaryPath, args...) /* #nosec G204 */
	// Write stderr to log files.
	stderr, err := os.Create(path.Join(e2e.TestParams.LogPath, fmt.Sprintf("lighthouse_beacon_node_%d_stderr.log", index)))
	if err != nil {
		return err
	}
	defer func() {
		if err := stderr.Close(); err != nil {
			log.WithError(err).Error("Failed to close stderr file")
		}
	}()
	cmd.Stderr = stderr
	log.Infof("Starting lighthouse beacon chain %d with flags: %s", index, strings.Join(args[2:], " "))
	if err = cmd.Start(); err != nil {
		return fmt.Errorf("failed to start beacon node: %w", err)
	}

	if err = helpers.WaitForTextInFile(stderr, "Metrics HTTP server started"); err != nil {
		return fmt.Errorf("could not find initialization for node %d, this means the node had issues starting: %w", index, err)
	}

	// Mark node as ready.
	close(node.started)
	node.cmd = cmd

	return cmd.Wait()
}

// Started checks whether beacon node is started and ready to be queried.
func (node *LighthouseBeaconNode) Started() <-chan struct{} {
	return node.started
}

// Pause pauses the component and its underlying process.
func (node *LighthouseBeaconNode) Pause() error {
	return node.cmd.Process.Signal(syscall.SIGSTOP)
}

// Resume resumes the component and its underlying process.
func (node *LighthouseBeaconNode) Resume() error {
	return node.cmd.Process.Signal(syscall.SIGCONT)
}

// Stop stops the component and its underlying process.
func (node *LighthouseBeaconNode) Stop() error {
	return node.cmd.Process.Kill()
}

func (node *LighthouseBeaconNode) createTestnetDir(ctx context.Context, index int) (string, error) {
	testNetDir := e2e.TestParams.TestPath + fmt.Sprintf("/lighthouse-testnet-%d", index)
	configPath := filepath.Join(testNetDir, "config.yaml")
	rawYaml := params.ConfigToYaml(params.BeaconConfig())

	if err := file.MkdirAll(testNetDir); err != nil {
		return "", err
	}
	if err := file.WriteFile(configPath, rawYaml); err != nil {
		return "", err
	}
	bootPath := filepath.Join(testNetDir, "boot_enr.yaml")
	enrYaml := []byte(fmt.Sprintf("[%s]", node.enr))
	if err := file.WriteFile(bootPath, enrYaml); err != nil {
		return "", err
	}
	deployPath := filepath.Join(testNetDir, "deploy_block.txt")
	deployYaml := []byte("0")
	if err := file.WriteFile(deployPath, deployYaml); err != nil {
		return "", err
	}

	return testNetDir, node.saveGenesis(ctx, testNetDir)
}

func (node *LighthouseBeaconNode) saveGenesis(ctx context.Context, testNetDir string) error {
	// The deposit contract starts with an empty trie, we use the BeaconState to "pre-mine" the validator registry,
	g, err := generateGenesis(ctx)
	if err != nil {
		return err
	}

	root, err := g.HashTreeRoot(ctx)
	if err != nil {
		return err
	}
	lbhr, err := g.LatestBlockHeader().HashTreeRoot()
	if err != nil {
		return err
	}
	log.WithField("fork_version", g.Fork().CurrentVersion).
		WithField("latest_block_header.root", fmt.Sprintf("%#x", lbhr)).
		WithField("state_root", fmt.Sprintf("%#x", root)).
		Infof("BeaconState info")

	genesisBytes, err := g.MarshalSSZ()
	if err != nil {
		return err
	}
	genesisPath := path.Join(testNetDir, "genesis.ssz")
	return file.WriteFile(genesisPath, genesisBytes)
}

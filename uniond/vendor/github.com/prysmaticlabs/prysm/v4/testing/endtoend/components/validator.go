package components

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path"
	"path/filepath"
	"strings"
	"syscall"

	"github.com/bazelbuild/rules_go/go/tools/bazel"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/pkg/errors"
	cmdshared "github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/cmd/validator/flags"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	validatorpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/validator-client"
	"github.com/prysmaticlabs/prysm/v4/runtime/interop"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/helpers"
	e2e "github.com/prysmaticlabs/prysm/v4/testing/endtoend/params"
	e2etypes "github.com/prysmaticlabs/prysm/v4/testing/endtoend/types"
)

const DefaultFeeRecipientAddress = "0x099FB65722e7b2455043bfebF6177f1D2E9738d9"

var _ e2etypes.ComponentRunner = (*ValidatorNode)(nil)
var _ e2etypes.ComponentRunner = (*ValidatorNodeSet)(nil)
var _ e2etypes.MultipleComponentRunners = (*ValidatorNodeSet)(nil)

// ValidatorNodeSet represents set of validator nodes.
type ValidatorNodeSet struct {
	e2etypes.ComponentRunner
	config  *e2etypes.E2EConfig
	started chan struct{}
	nodes   []e2etypes.ComponentRunner
}

// NewValidatorNodeSet creates and returns a set of validator nodes.
func NewValidatorNodeSet(config *e2etypes.E2EConfig) *ValidatorNodeSet {
	return &ValidatorNodeSet{
		config:  config,
		started: make(chan struct{}, 1),
	}
}

// Start starts the configured amount of validators, also sending and mining their deposits.
func (s *ValidatorNodeSet) Start(ctx context.Context) error {
	// Always using genesis count since using anything else would be difficult to test for.
	validatorNum := int(params.BeaconConfig().MinGenesisActiveValidatorCount)
	prysmBeaconNodeNum := e2e.TestParams.BeaconNodeCount
	beaconNodeNum := prysmBeaconNodeNum + e2e.TestParams.LighthouseBeaconNodeCount
	if validatorNum%beaconNodeNum != 0 {
		return errors.New("validator count is not easily divisible by beacon node count")
	}
	validatorsPerNode := validatorNum / beaconNodeNum
	// Create validator nodes.
	nodes := make([]e2etypes.ComponentRunner, prysmBeaconNodeNum)
	for i := 0; i < prysmBeaconNodeNum; i++ {
		nodes[i] = NewValidatorNode(s.config, validatorsPerNode, i, validatorsPerNode*i)
	}
	s.nodes = nodes

	// Wait for all nodes to finish their job (blocking).
	// Once nodes are ready passed in handler function will be called.
	return helpers.WaitOnNodes(ctx, nodes, func() {
		// All nodes started, close channel, so that all services waiting on a set, can proceed.
		close(s.started)
	})
}

// Started checks whether validator node set is started and all nodes are ready to be queried.
func (s *ValidatorNodeSet) Started() <-chan struct{} {
	return s.started
}

// Pause pauses the component and its underlying process.
func (s *ValidatorNodeSet) Pause() error {
	for _, n := range s.nodes {
		if err := n.Pause(); err != nil {
			return err
		}
	}
	return nil
}

// Resume resumes the component and its underlying process.
func (s *ValidatorNodeSet) Resume() error {
	for _, n := range s.nodes {
		if err := n.Resume(); err != nil {
			return err
		}
	}
	return nil
}

// Stop stops the component and its underlying process.
func (s *ValidatorNodeSet) Stop() error {
	for _, n := range s.nodes {
		if err := n.Stop(); err != nil {
			return err
		}
	}
	return nil
}

// PauseAtIndex pauses the component and its underlying process at the desired index.
func (s *ValidatorNodeSet) PauseAtIndex(i int) error {
	if i >= len(s.nodes) {
		return errors.Errorf("provided index exceeds slice size: %d >= %d", i, len(s.nodes))
	}
	return s.nodes[i].Pause()
}

// ResumeAtIndex resumes the component and its underlying process at the desired index.
func (s *ValidatorNodeSet) ResumeAtIndex(i int) error {
	if i >= len(s.nodes) {
		return errors.Errorf("provided index exceeds slice size: %d >= %d", i, len(s.nodes))
	}
	return s.nodes[i].Resume()
}

// StopAtIndex stops the component and its underlying process at the desired index.
func (s *ValidatorNodeSet) StopAtIndex(i int) error {
	if i >= len(s.nodes) {
		return errors.Errorf("provided index exceeds slice size: %d >= %d", i, len(s.nodes))
	}
	return s.nodes[i].Stop()
}

// ComponentAtIndex returns the component at the provided index.
func (s *ValidatorNodeSet) ComponentAtIndex(i int) (e2etypes.ComponentRunner, error) {
	if i >= len(s.nodes) {
		return nil, errors.Errorf("provided index exceeds slice size: %d >= %d", i, len(s.nodes))
	}
	return s.nodes[i], nil
}

// ValidatorNode represents a validator node.
type ValidatorNode struct {
	e2etypes.ComponentRunner
	config       *e2etypes.E2EConfig
	started      chan struct{}
	validatorNum int
	index        int
	offset       int
	cmd          *exec.Cmd
}

// NewValidatorNode creates and returns a validator node.
func NewValidatorNode(config *e2etypes.E2EConfig, validatorNum, index, offset int) *ValidatorNode {
	return &ValidatorNode{
		config:       config,
		validatorNum: validatorNum,
		index:        index,
		offset:       offset,
		started:      make(chan struct{}, 1),
	}
}

func (node *ValidatorNode) saveConfig() (string, error) {
	cfg := params.BeaconConfig().Copy()
	cfgBytes := params.ConfigToYaml(cfg)
	cfgDir := path.Join(e2e.TestParams.TestPath, fmt.Sprintf("config/%d", node.index))
	if err := file.MkdirAll(cfgDir); err != nil {
		return "", err
	}
	cfgPath := path.Join(cfgDir, "validator-config.yaml")
	return cfgPath, file.WriteFile(cfgPath, cfgBytes)
}

// Start starts a validator client.
func (v *ValidatorNode) Start(ctx context.Context) error {
	validatorHexPubKeys := make([]string, 0)
	var pkg, target string
	if v.config.UsePrysmShValidator {
		pkg = ""
		target = "prysm_sh"
	} else {
		pkg = "cmd/validator"
		target = "validator"
	}
	binaryPath, found := bazel.FindBinary(pkg, target)
	if !found {
		return errors.New("validator binary not found")
	}

	config, validatorNum, index, offset := v.config, v.validatorNum, v.index, v.offset
	beaconRPCPort := e2e.TestParams.Ports.PrysmBeaconNodeRPCPort + index
	if beaconRPCPort >= e2e.TestParams.Ports.PrysmBeaconNodeRPCPort+e2e.TestParams.BeaconNodeCount {
		// Point any extra validator clients to a node we know is running.
		beaconRPCPort = e2e.TestParams.Ports.PrysmBeaconNodeRPCPort
	}

	file, err := helpers.DeleteAndCreateFile(e2e.TestParams.LogPath, fmt.Sprintf(e2e.ValidatorLogFileName, index))
	if err != nil {
		return err
	}
	gFile, err := helpers.GraffitiYamlFile(e2e.TestParams.TestPath)
	if err != nil {
		return err
	}
	_, pubs, err := interop.DeterministicallyGenerateKeys(uint64(offset), uint64(validatorNum))
	if err != nil {
		return err
	}
	for _, pub := range pubs {
		validatorHexPubKeys = append(validatorHexPubKeys, hexutil.Encode(pub.Marshal()))
	}
	proposerSettingsPathPath, err := createProposerSettingsPath(validatorHexPubKeys, index)
	if err != nil {
		return err
	}
	cfgPath, err := v.saveConfig()
	if err != nil {
		return err
	}
	args := []string{
		fmt.Sprintf("--%s=%s/eth2-val-%d", cmdshared.DataDirFlag.Name, e2e.TestParams.TestPath, index),
		fmt.Sprintf("--%s=%s", cmdshared.LogFileName.Name, file.Name()),
		fmt.Sprintf("--%s=%s", flags.GraffitiFileFlag.Name, gFile),
		fmt.Sprintf("--%s=%d", flags.MonitoringPortFlag.Name, e2e.TestParams.Ports.ValidatorMetricsPort+index),
		fmt.Sprintf("--%s=%d", flags.GRPCGatewayPort.Name, e2e.TestParams.Ports.ValidatorGatewayPort+index),
		fmt.Sprintf("--%s=localhost:%d", flags.BeaconRPCProviderFlag.Name, beaconRPCPort),

		fmt.Sprintf("--%s=%s", flags.GrpcHeadersFlag.Name, "dummy=value,foo=bar"), // Sending random headers shouldn't break anything.
		fmt.Sprintf("--%s=%s", cmdshared.VerbosityFlag.Name, "debug"),
		fmt.Sprintf("--%s=%s", flags.ProposerSettingsFlag.Name, proposerSettingsPathPath),
		fmt.Sprintf("--%s=%s", cmdshared.ChainConfigFileFlag.Name, cfgPath),
		"--" + cmdshared.ForceClearDB.Name,
		"--" + cmdshared.AcceptTosFlag.Name,
	}

	if v.config.UseBeaconRestApi {
		beaconRestApiPort := e2e.TestParams.Ports.PrysmBeaconNodeGatewayPort + index
		if beaconRestApiPort >= e2e.TestParams.Ports.PrysmBeaconNodeGatewayPort+e2e.TestParams.BeaconNodeCount {
			// Point any extra validator clients to a node we know is running.
			beaconRestApiPort = e2e.TestParams.Ports.PrysmBeaconNodeGatewayPort
		}

		args = append(args,
			fmt.Sprintf("--%s=http://localhost:%d", flags.BeaconRESTApiProviderFlag.Name, beaconRestApiPort),
			fmt.Sprintf("--%s", features.EnableBeaconRESTApi.Name))
	}

	// Only apply e2e flags to the current branch. New flags may not exist in previous release.
	if !v.config.UsePrysmShValidator {
		args = append(args, features.E2EValidatorFlags...)
	}
	if v.config.UseWeb3RemoteSigner {
		// Write the pubkeys as comma separated hex strings with 0x prefix.
		// See: https://docs.teku.consensys.net/en/latest/HowTo/External-Signer/Use-External-Signer/
		args = append(args,
			fmt.Sprintf("--%s=http://localhost:%d", flags.Web3SignerURLFlag.Name, Web3RemoteSignerPort),
			fmt.Sprintf("--%s=%s", flags.Web3SignerPublicValidatorKeysFlag.Name, strings.Join(validatorHexPubKeys, ",")))
	} else {
		// When not using remote key signer, use interop keys.
		args = append(args,
			fmt.Sprintf("--%s=%d", flags.InteropNumValidators.Name, validatorNum),
			fmt.Sprintf("--%s=%d", flags.InteropStartIndex.Name, offset),
		)
	}
	if v.config.UseBuilder {
		args = append(args, fmt.Sprintf("--%s", flags.EnableBuilderFlag.Name))
	}
	args = append(args, config.ValidatorFlags...)

	if v.config.UsePrysmShValidator {
		args = append([]string{"validator"}, args...)
		log.Warning("Using latest release validator via prysm.sh")
	}

	cmd := exec.CommandContext(ctx, binaryPath, args...) // #nosec G204 -- Safe

	// Write stdout and stderr to log files.
	stdout, err := os.Create(path.Join(e2e.TestParams.LogPath, fmt.Sprintf("validator_%d_stdout.log", index)))
	if err != nil {
		return err
	}
	stderr, err := os.Create(path.Join(e2e.TestParams.LogPath, fmt.Sprintf("validator_%d_stderr.log", index)))
	if err != nil {
		return err
	}
	defer func() {
		if err := stdout.Close(); err != nil {
			log.WithError(err).Error("Failed to close stdout file")
		}
		if err := stderr.Close(); err != nil {
			log.WithError(err).Error("Failed to close stderr file")
		}
	}()
	cmd.Stdout = stdout
	cmd.Stderr = stderr

	log.Infof("Starting validator client %d with flags: %s %s", index, binaryPath, strings.Join(args, " "))
	if err = cmd.Start(); err != nil {
		return err
	}

	// Mark node as ready.
	close(v.started)
	v.cmd = cmd

	return cmd.Wait()
}

// Started checks whether validator node is started and ready to be queried.
func (v *ValidatorNode) Started() <-chan struct{} {
	return v.started
}

// Pause pauses the component and its underlying process.
func (v *ValidatorNode) Pause() error {
	return v.cmd.Process.Signal(syscall.SIGSTOP)
}

// Resume resumes the component and its underlying process.
func (v *ValidatorNode) Resume() error {
	return v.cmd.Process.Signal(syscall.SIGCONT)
}

// Stop stops the component and its underlying process.
func (v *ValidatorNode) Stop() error {
	return v.cmd.Process.Kill()
}

func (v *ValidatorNode) UnderlyingProcess() *os.Process {
	return v.cmd.Process
}

func createProposerSettingsPath(pubkeys []string, nodeIdx int) (string, error) {
	testNetDir := e2e.TestParams.TestPath + fmt.Sprintf("/proposer-settings/validator_%d", nodeIdx)
	configPath := filepath.Join(testNetDir, "config.json")
	if len(pubkeys) == 0 {
		return "", errors.New("number of validators must be greater than 0")
	}
	config := make(map[string]*validatorpb.ProposerOptionPayload)

	for i, pubkey := range pubkeys {
		config[pubkeys[i]] = &validatorpb.ProposerOptionPayload{
			FeeRecipient: FeeRecipientFromPubkey(pubkey),
		}
	}
	proposerSettingsPayload := &validatorpb.ProposerSettingsPayload{
		ProposerConfig: config,
		DefaultConfig: &validatorpb.ProposerOptionPayload{
			FeeRecipient: DefaultFeeRecipientAddress,
		},
	}
	jsonBytes, err := json.Marshal(proposerSettingsPayload)
	if err != nil {
		return "", err
	}
	if err := file.MkdirAll(testNetDir); err != nil {
		return "", err
	}
	if err := file.WriteFile(configPath, jsonBytes); err != nil {
		return "", err
	}
	return configPath, nil
}

// FeeRecipientFromPubkey slices, from the beginning of the hex-encoded pubkey string, the 2 character 0x preamble
// plus enough hex chars to fill out the fee_recipient byte value.
func FeeRecipientFromPubkey(key string) string {
	// pubkey[:(2+fieldparams.FeeRecipientLength*2)] slicing 2 (for the 0x preamble) + 2 hex chars for each byte
	return common.HexToAddress(key[:(2 + fieldparams.FeeRecipientLength*2)]).Hex()
}

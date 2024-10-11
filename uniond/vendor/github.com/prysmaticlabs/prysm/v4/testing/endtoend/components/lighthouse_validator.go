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
	"github.com/google/uuid"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/runtime/interop"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/helpers"
	e2e "github.com/prysmaticlabs/prysm/v4/testing/endtoend/params"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/types"
	e2etypes "github.com/prysmaticlabs/prysm/v4/testing/endtoend/types"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	keystorev4 "github.com/wealdtech/go-eth2-wallet-encryptor-keystorev4"
	"golang.org/x/sync/errgroup"
)

var _ e2etypes.ComponentRunner = (*LighthouseValidatorNode)(nil)
var _ e2etypes.ComponentRunner = (*LighthouseValidatorNodeSet)(nil)
var _ e2etypes.MultipleComponentRunners = (*LighthouseValidatorNodeSet)(nil)

// LighthouseValidatorNodeSet represents set of lighthouse validator nodes.
type LighthouseValidatorNodeSet struct {
	e2etypes.ComponentRunner
	config  *e2etypes.E2EConfig
	started chan struct{}
	nodes   []e2etypes.ComponentRunner
}

// NewLighthouseValidatorNodeSet creates and returns a set of lighthouse validator nodes.
func NewLighthouseValidatorNodeSet(config *e2etypes.E2EConfig) *LighthouseValidatorNodeSet {
	return &LighthouseValidatorNodeSet{
		config:  config,
		started: make(chan struct{}, 1),
	}
}

// Start starts the configured amount of validators, also sending and mining their deposits.
func (s *LighthouseValidatorNodeSet) Start(ctx context.Context) error {
	// Always using genesis count since using anything else would be difficult to test for.
	validatorNum := int(params.BeaconConfig().MinGenesisActiveValidatorCount)
	lighthouseBeaconNum := e2e.TestParams.LighthouseBeaconNodeCount
	prysmBeaconNum := e2e.TestParams.BeaconNodeCount
	beaconNodeNum := lighthouseBeaconNum + prysmBeaconNum
	if validatorNum%beaconNodeNum != 0 {
		return errors.New("validator count is not easily divisible by beacon node count")
	}
	validatorsPerNode := validatorNum / beaconNodeNum

	// Create validator nodes.
	nodes := make([]e2etypes.ComponentRunner, lighthouseBeaconNum)
	for i := 0; i < lighthouseBeaconNum; i++ {
		offsetIdx := i + prysmBeaconNum
		nodes[i] = NewLighthouseValidatorNode(s.config, validatorsPerNode, i, validatorsPerNode*offsetIdx)
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
func (s *LighthouseValidatorNodeSet) Started() <-chan struct{} {
	return s.started
}

// Pause pauses the component and its underlying process.
func (s *LighthouseValidatorNodeSet) Pause() error {
	for _, n := range s.nodes {
		if err := n.Pause(); err != nil {
			return err
		}
	}
	return nil
}

// Resume resumes the component and its underlying process.
func (s *LighthouseValidatorNodeSet) Resume() error {
	for _, n := range s.nodes {
		if err := n.Resume(); err != nil {
			return err
		}
	}
	return nil
}

// Stop stops the component and its underlying process.
func (s *LighthouseValidatorNodeSet) Stop() error {
	for _, n := range s.nodes {
		if err := n.Stop(); err != nil {
			return err
		}
	}
	return nil
}

// PauseAtIndex pauses the component and its underlying process at the desired index.
func (s *LighthouseValidatorNodeSet) PauseAtIndex(i int) error {
	if i >= len(s.nodes) {
		return errors.Errorf("provided index exceeds slice size: %d >= %d", i, len(s.nodes))
	}
	return s.nodes[i].Pause()
}

// ResumeAtIndex resumes the component and its underlying process at the desired index.
func (s *LighthouseValidatorNodeSet) ResumeAtIndex(i int) error {
	if i >= len(s.nodes) {
		return errors.Errorf("provided index exceeds slice size: %d >= %d", i, len(s.nodes))
	}
	return s.nodes[i].Resume()
}

// StopAtIndex stops the component and its underlying process at the desired index.
func (s *LighthouseValidatorNodeSet) StopAtIndex(i int) error {
	if i >= len(s.nodes) {
		return errors.Errorf("provided index exceeds slice size: %d >= %d", i, len(s.nodes))
	}
	return s.nodes[i].Stop()
}

// ComponentAtIndex returns the component at the provided index.
func (s *LighthouseValidatorNodeSet) ComponentAtIndex(i int) (e2etypes.ComponentRunner, error) {
	if i >= len(s.nodes) {
		return nil, errors.Errorf("provided index exceeds slice size: %d >= %d", i, len(s.nodes))
	}
	return s.nodes[i], nil
}

// LighthouseValidatorNode represents a lighthouse validator node.
type LighthouseValidatorNode struct {
	e2etypes.ComponentRunner
	config       *e2etypes.E2EConfig
	started      chan struct{}
	validatorNum int
	index        int
	offset       int
	cmd          *exec.Cmd
}

// NewLighthouseValidatorNode creates and returns a lighthouse validator node.
func NewLighthouseValidatorNode(config *e2etypes.E2EConfig, validatorNum, index, offset int) *LighthouseValidatorNode {
	return &LighthouseValidatorNode{
		config:       config,
		validatorNum: validatorNum,
		index:        index,
		offset:       offset,
		started:      make(chan struct{}, 1),
	}
}

// Start starts a validator client.
func (v *LighthouseValidatorNode) Start(ctx context.Context) error {
	binaryPath, found := bazel.FindBinary("external/lighthouse", "lighthouse")
	if !found {
		log.Info(binaryPath)
		log.Error("validator binary not found")
	}

	_, _, index, _ := v.config, v.validatorNum, v.index, v.offset
	kPath := e2e.TestParams.TestPath + fmt.Sprintf("/lighthouse-validator-%d", index)
	testNetDir := e2e.TestParams.TestPath + fmt.Sprintf("/lighthouse-testnet-%d", index)
	httpPort := e2e.TestParams.Ports.LighthouseBeaconNodeHTTPPort
	// In the event we want to run a LH validator with a non LH
	// beacon node, we split half the validators to run with
	// lighthouse and the other half with prysm.
	if v.config.UseValidatorCrossClient && index%2 == 0 {
		httpPort = e2e.TestParams.Ports.PrysmBeaconNodeGatewayPort
	}
	args := []string{
		"validator_client",
		"--debug-level=debug",
		"--init-slashing-protection",
		fmt.Sprintf("--datadir=%s", kPath),
		fmt.Sprintf("--testnet-dir=%s", testNetDir),
		fmt.Sprintf("--beacon-nodes=http://localhost:%d", httpPort+index),
		"--suggested-fee-recipient=0x878705ba3f8bc32fcf7f4caa1a35e72af65cf766",
	}

	if v.config.UseBuilder {
		args = append(args, "--builder-proposals")
	}

	cmd := exec.CommandContext(ctx, binaryPath, args...) // #nosec G204 -- Safe

	// Write stderr to log files.
	stderr, err := os.Create(path.Join(e2e.TestParams.LogPath, fmt.Sprintf("lighthouse_validator_%d_stderr.log", index)))
	if err != nil {
		return err
	}
	defer func() {
		if err := stderr.Close(); err != nil {
			log.WithError(err).Error("Failed to close stderr file")
		}
	}()
	cmd.Stderr = stderr

	log.Infof("Starting lighthouse validator client %d with flags: %s %s", index, binaryPath, strings.Join(args, " "))
	if err = cmd.Start(); err != nil {
		return err
	}

	// Mark node as ready.
	close(v.started)
	v.cmd = cmd

	return cmd.Wait()
}

// Started checks whether validator node is started and ready to be queried.
func (v *LighthouseValidatorNode) Started() <-chan struct{} {
	return v.started
}

// Pause pauses the component and its underlying process.
func (v *LighthouseValidatorNode) Pause() error {
	return v.cmd.Process.Signal(syscall.SIGSTOP)
}

// Resume resumes the component and its underlying process.
func (v *LighthouseValidatorNode) Resume() error {
	return v.cmd.Process.Signal(syscall.SIGCONT)
}

// Stop stops the component and its underlying process.
func (v *LighthouseValidatorNode) Stop() error {
	return v.cmd.Process.Kill()
}

var _ types.ComponentRunner = &KeystoreGenerator{}

type KeystoreGenerator struct {
	started chan struct{}
}

func NewKeystoreGenerator() *KeystoreGenerator {
	return &KeystoreGenerator{started: make(chan struct{})}
}

func (k *KeystoreGenerator) Start(_ context.Context) error {
	validatorNum := int(params.BeaconConfig().MinGenesisActiveValidatorCount)
	lighthouseBeaconNum := e2e.TestParams.LighthouseBeaconNodeCount
	prysmBeaconNum := e2e.TestParams.BeaconNodeCount
	beaconNodeNum := lighthouseBeaconNum + prysmBeaconNum
	if validatorNum%beaconNodeNum != 0 {
		return errors.New("validator count is not easily divisible by beacon node count")
	}
	validatorsPerNode := validatorNum / beaconNodeNum

	for i := 0; i < lighthouseBeaconNum; i++ {
		offsetIdx := i + prysmBeaconNum
		_, err := setupKeystores(i, validatorsPerNode*offsetIdx, validatorsPerNode)
		if err != nil {
			return err
		}
		log.Infof("Generated lighthouse keystores from %d onwards with %d keys", validatorsPerNode*offsetIdx, validatorsPerNode)
	}
	// Mark component as ready.
	close(k.started)
	return nil
}

func (k *KeystoreGenerator) Started() <-chan struct{} {
	return k.started
}

// Pause pauses the component and its underlying process.
func (k *KeystoreGenerator) Pause() error {
	// no-op
	return nil
}

// Resume resumes the component and its underlying process.
func (k *KeystoreGenerator) Resume() error {
	// no-op
	return nil
}

// Stop stops the component and its underlying process.
func (k *KeystoreGenerator) Stop() error {
	// no-op
	return nil
}

func setupKeystores(valClientIdx, startIdx, numOfKeys int) (string, error) {
	testNetDir := e2e.TestParams.TestPath + fmt.Sprintf("/lighthouse-validator-%d", valClientIdx)
	if err := file.MkdirAll(testNetDir); err != nil {
		return "", err
	}
	secretsPath := filepath.Join(testNetDir, "secrets")
	validatorKeystorePath := filepath.Join(testNetDir, "validators")
	if err := file.MkdirAll(secretsPath); err != nil {
		return "", err
	}
	if err := file.MkdirAll(validatorKeystorePath); err != nil {
		return "", err
	}
	privKeys, pubKeys, err := interop.DeterministicallyGenerateKeys(uint64(startIdx), uint64(numOfKeys))
	if err != nil {
		return "", err
	}
	encryptor := keystorev4.New()
	// Use lighthouse's default password for their insecure keystores.
	password := "222222222222222222222222222222222222222222222222222"
	g, _ := errgroup.WithContext(context.Background())
	for i, pkey := range pubKeys {
		pubKeyBytes := pkey.Marshal()
		marshalledPriv := privKeys[i].Marshal()
		g.Go(func() error {
			cryptoFields, err := encryptor.Encrypt(marshalledPriv, password)
			if err != nil {
				return errors.Wrapf(
					err,
					"could not encrypt secret key for public key %#x",
					pubKeyBytes,
				)
			}
			id, err := uuid.NewRandom()
			if err != nil {
				return err
			}
			kStore := &keymanager.Keystore{
				Crypto:  cryptoFields,
				ID:      id.String(),
				Pubkey:  fmt.Sprintf("%x", pubKeyBytes),
				Version: encryptor.Version(),
				Name:    encryptor.Name(),
			}

			fPath := filepath.Join(secretsPath, "0x"+kStore.Pubkey)
			if err := file.WriteFile(fPath, []byte(password)); err != nil {
				return err
			}
			keystorePath := filepath.Join(validatorKeystorePath, "0x"+kStore.Pubkey)
			if err := file.MkdirAll(keystorePath); err != nil {
				return err
			}
			fPath = filepath.Join(keystorePath, "voting-keystore.json")
			encodedFile, err := json.MarshalIndent(kStore, "", "\t")
			if err != nil {
				return errors.Wrap(err, "could not marshal keystore to JSON file")
			}
			return file.WriteFile(fPath, encodedFile)
		})
	}
	return testNetDir, g.Wait()
}

func (k *KeystoreGenerator) UnderlyingProcess() *os.Process {
	return nil // No subprocess for this component.
}

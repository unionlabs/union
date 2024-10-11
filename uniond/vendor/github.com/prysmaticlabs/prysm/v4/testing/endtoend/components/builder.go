package components

import (
	"context"
	"encoding/hex"
	"fmt"
	"os"
	"path"
	"strconv"
	"strings"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/helpers"
	e2e "github.com/prysmaticlabs/prysm/v4/testing/endtoend/params"
	e2etypes "github.com/prysmaticlabs/prysm/v4/testing/endtoend/types"
	"github.com/prysmaticlabs/prysm/v4/testing/middleware/builder"
	"github.com/sirupsen/logrus"
)

// BuilderSet represents a set of builders for the validators running via a relay.
type BuilderSet struct {
	e2etypes.ComponentRunner
	started  chan struct{}
	builders []e2etypes.ComponentRunner
}

// NewBuilderSet creates and returns a set of builders.
func NewBuilderSet() *BuilderSet {
	return &BuilderSet{
		started: make(chan struct{}, 1),
	}
}

// Start starts all the builders in set.
func (s *BuilderSet) Start(ctx context.Context) error {
	totalNodeCount := e2e.TestParams.BeaconNodeCount + e2e.TestParams.LighthouseBeaconNodeCount
	nodes := make([]e2etypes.ComponentRunner, totalNodeCount)
	for i := 0; i < totalNodeCount; i++ {
		nodes[i] = NewBuilder(i)
	}
	s.builders = nodes

	// Wait for all nodes to finish their job (blocking).
	// Once nodes are ready passed in handler function will be called.
	return helpers.WaitOnNodes(ctx, nodes, func() {
		// All nodes started, close channel, so that all services waiting on a set, can proceed.
		close(s.started)
	})
}

// Started checks whether builder set is started and all builders are ready to be queried.
func (s *BuilderSet) Started() <-chan struct{} {
	return s.started
}

// Pause pauses the component and its underlying process.
func (s *BuilderSet) Pause() error {
	for _, n := range s.builders {
		if err := n.Pause(); err != nil {
			return err
		}
	}
	return nil
}

// Resume resumes the component and its underlying process.
func (s *BuilderSet) Resume() error {
	for _, n := range s.builders {
		if err := n.Resume(); err != nil {
			return err
		}
	}
	return nil
}

// Stop stops the component and its underlying process.
func (s *BuilderSet) Stop() error {
	for _, n := range s.builders {
		if err := n.Stop(); err != nil {
			return err
		}
	}
	return nil
}

// PauseAtIndex pauses the component and its underlying process at the desired index.
func (s *BuilderSet) PauseAtIndex(i int) error {
	if i >= len(s.builders) {
		return errors.Errorf("provided index exceeds slice size: %d >= %d", i, len(s.builders))
	}
	return s.builders[i].Pause()
}

// ResumeAtIndex resumes the component and its underlying process at the desired index.
func (s *BuilderSet) ResumeAtIndex(i int) error {
	if i >= len(s.builders) {
		return errors.Errorf("provided index exceeds slice size: %d >= %d", i, len(s.builders))
	}
	return s.builders[i].Resume()
}

// StopAtIndex stops the component and its underlying process at the desired index.
func (s *BuilderSet) StopAtIndex(i int) error {
	if i >= len(s.builders) {
		return errors.Errorf("provided index exceeds slice size: %d >= %d", i, len(s.builders))
	}
	return s.builders[i].Stop()
}

// ComponentAtIndex returns the component at the provided index.
func (s *BuilderSet) ComponentAtIndex(i int) (e2etypes.ComponentRunner, error) {
	if i >= len(s.builders) {
		return nil, errors.Errorf("provided index exceeds slice size: %d >= %d", i, len(s.builders))
	}
	return s.builders[i], nil
}

// Builder represents a block builder.
type Builder struct {
	e2etypes.ComponentRunner
	started chan struct{}
	index   int
	builder *builder.Builder
	cancel  func()
}

// NewBuilder creates and returns a builder.
func NewBuilder(index int) *Builder {
	return &Builder{
		started: make(chan struct{}, 1),
		index:   index,
	}
}

// Start runs a builder.
func (node *Builder) Start(ctx context.Context) error {
	f, err := os.Create(path.Join(e2e.TestParams.LogPath, "builder_"+strconv.Itoa(node.index)+".log"))
	if err != nil {
		return err
	}
	jwtPath := path.Join(e2e.TestParams.TestPath, "eth1data/"+strconv.Itoa(node.index)+"/")
	if node.index == 0 {
		jwtPath = path.Join(e2e.TestParams.TestPath, "eth1data/miner/")
	}
	jwtPath = path.Join(jwtPath, "geth/jwtsecret")
	secret, err := parseJWTSecretFromFile(jwtPath)
	if err != nil {
		return err
	}
	opts := []builder.Option{
		builder.WithDestinationAddress(fmt.Sprintf("http://127.0.0.1:%d", e2e.TestParams.Ports.Eth1AuthRPCPort+node.index)),
		builder.WithPort(e2e.TestParams.Ports.Eth1ProxyPort + node.index),
		builder.WithLogger(logrus.New()),
		builder.WithLogFile(f),
		builder.WithJwtSecret(string(secret)),
	}
	bd, err := builder.New(opts...)
	if err != nil {
		return err
	}
	log.Infof("Starting builder %d with port: %d and file %s", node.index, e2e.TestParams.Ports.Eth1ProxyPort+node.index, f.Name())

	// Set cancel into context.
	ctx, cancel := context.WithCancel(ctx)
	node.cancel = cancel
	node.builder = bd
	// Mark node as ready.
	close(node.started)
	return bd.Start(ctx)
}

// Started checks whether the builder is started and ready to be queried.
func (node *Builder) Started() <-chan struct{} {
	return node.started
}

// Pause pauses the component and its underlying process.
func (node *Builder) Pause() error {
	// no-op
	return nil
}

// Resume resumes the component and its underlying process.
func (node *Builder) Resume() error {
	// no-op
	return nil
}

// Stop kills the component and its underlying process.
func (node *Builder) Stop() error {
	node.cancel()
	return nil
}

func parseJWTSecretFromFile(jwtSecretFile string) ([]byte, error) {
	enc, err := file.ReadFileAsBytes(jwtSecretFile)
	if err != nil {
		return nil, err
	}
	strData := strings.TrimSpace(string(enc))
	if strData == "" {
		return nil, fmt.Errorf("provided JWT secret in file %s cannot be empty", jwtSecretFile)
	}
	secret, err := hex.DecodeString(strings.TrimPrefix(strData, "0x"))
	if err != nil {
		return nil, err
	}
	if len(secret) < 32 {
		return nil, errors.New("provided JWT secret should be a hex string of at least 32 bytes")
	}
	return secret, nil
}

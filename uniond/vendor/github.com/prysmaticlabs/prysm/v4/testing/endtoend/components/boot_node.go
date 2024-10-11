package components

import (
	"context"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"strings"
	"syscall"

	"github.com/bazelbuild/rules_go/go/tools/bazel"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/helpers"
	e2e "github.com/prysmaticlabs/prysm/v4/testing/endtoend/params"
	e2etypes "github.com/prysmaticlabs/prysm/v4/testing/endtoend/types"
)

var _ e2etypes.ComponentRunner = (*BootNode)(nil)

// BootNode represents boot node.
type BootNode struct {
	e2etypes.ComponentRunner
	started chan struct{}
	enr     string
	cmd     *exec.Cmd
}

// NewBootNode creates and returns boot node.
func NewBootNode() *BootNode {
	return &BootNode{
		started: make(chan struct{}, 1),
	}
}

// ENR exposes node's ENR.
func (node *BootNode) ENR() string {
	return node.enr
}

// Start starts a bootnode blocks up until ctx is cancelled.
func (node *BootNode) Start(ctx context.Context) error {
	binaryPath, found := bazel.FindBinary("tools/bootnode", "bootnode")
	if !found {
		log.Info(binaryPath)
		return errors.New("boot node binary not found")
	}

	args := []string{
		fmt.Sprintf("--discv5-port=%d", e2e.TestParams.Ports.BootNodePort),
		fmt.Sprintf("--metrics-port=%d", e2e.TestParams.Ports.BootNodeMetricsPort),
	}

	cmd := exec.CommandContext(ctx, binaryPath, args...) // #nosec G204 -- Safe
	stdErrFile, err := helpers.DeleteAndCreateFile(e2e.TestParams.LogPath, e2e.BootNodeLogFileName)
	if err != nil {
		return err
	}
	cmd.Stderr = stdErrFile
	log.Infof("Starting boot node with flags: %s", strings.Join(args[1:], " "))
	if err = cmd.Start(); err != nil {
		return fmt.Errorf("failed to start beacon node: %w", err)
	}

	if err = helpers.WaitForTextInFile(stdErrFile, "Running bootnode"); err != nil {
		return fmt.Errorf("could not find enr for bootnode, this means the bootnode had issues starting: %w", err)
	}

	node.enr, err = enrFromLogFile(stdErrFile.Name())
	if err != nil {
		return fmt.Errorf("could not get enr for bootnode: %w", err)
	}

	// Mark node as ready.
	close(node.started)
	node.cmd = cmd

	return cmd.Wait()
}

// Started checks whether a boot node is started and ready to be queried.
func (node *BootNode) Started() <-chan struct{} {
	return node.started
}

// Pause pauses the component and its underlying process.
func (node *BootNode) Pause() error {
	return node.cmd.Process.Signal(syscall.SIGSTOP)
}

// Resume resumes the component and its underlying process.
func (node *BootNode) Resume() error {
	return node.cmd.Process.Signal(syscall.SIGCONT)
}

// Stop stops the component and its underlying process.
func (node *BootNode) Stop() error {
	return node.cmd.Process.Kill()
}

func enrFromLogFile(name string) (string, error) {
	byteContent, err := os.ReadFile(name) // #nosec G304
	if err != nil {
		return "", err
	}
	contents := string(byteContent)

	searchText := "Running bootnode: "
	startIdx := strings.Index(contents, searchText)
	if startIdx == -1 {
		return "", fmt.Errorf("did not find ENR text in %s", contents)
	}
	startIdx += len(searchText)
	endIdx := strings.Index(contents[startIdx:], " prefix=bootnode")
	if endIdx == -1 {
		return "", fmt.Errorf("did not find ENR text in %s", contents)
	}
	return contents[startIdx : startIdx+endIdx-1], nil
}

package eth1

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path"
	"strconv"
	"strings"
	"syscall"

	"github.com/bazelbuild/rules_go/go/tools/bazel"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/runtime/interop"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/helpers"
	e2e "github.com/prysmaticlabs/prysm/v4/testing/endtoend/params"
	e2etypes "github.com/prysmaticlabs/prysm/v4/testing/endtoend/types"
	log "github.com/sirupsen/logrus"
)

// Node represents an ETH1 node.
type Node struct {
	e2etypes.ComponentRunner
	started chan struct{}
	index   int
	enr     string
	cmd     *exec.Cmd
}

// NewNode creates and returns ETH1 node.
func NewNode(index int, enr string) *Node {
	return &Node{
		started: make(chan struct{}, 1),
		index:   index,
		enr:     enr,
	}
}

// Start runs a non-mining ETH1 node.
// To connect to a miner and start working properly, this node should be a part of a NodeSet.
func (node *Node) Start(ctx context.Context) error {
	binaryPath, found := bazel.FindBinary("cmd/geth", "geth")
	if !found {
		return errors.New("go-ethereum binary not found")
	}

	eth1Path := path.Join(e2e.TestParams.TestPath, "eth1data/"+strconv.Itoa(node.index)+"/")
	// Clear out potentially existing dir to prevent issues.
	if _, err := os.Stat(eth1Path); !os.IsNotExist(err) {
		if err = os.RemoveAll(eth1Path); err != nil {
			return err
		}
	}

	if err := file.MkdirAll(eth1Path); err != nil {
		return err
	}
	gethJsonPath := path.Join(eth1Path, "genesis.json")

	gen := interop.GethTestnetGenesis(e2e.TestParams.Eth1GenesisTime, params.BeaconConfig())
	b, err := json.Marshal(gen)
	if err != nil {
		return err
	}

	if err := file.WriteFile(gethJsonPath, b); err != nil {
		return err
	}
	copyPath := path.Join(e2e.TestParams.LogPath, "eth1-genesis.json")
	if err := file.WriteFile(copyPath, b); err != nil {
		return err
	}

	initCmd := exec.CommandContext(ctx, binaryPath, "init", fmt.Sprintf("--datadir=%s", eth1Path), gethJsonPath) // #nosec G204 -- Safe
	initFile, err := helpers.DeleteAndCreateFile(e2e.TestParams.LogPath, "eth1-init_"+strconv.Itoa(node.index)+".log")
	if err != nil {
		return err
	}
	initCmd.Stderr = initFile
	if err = initCmd.Start(); err != nil {
		return err
	}
	if err = initCmd.Wait(); err != nil {
		return err
	}

	args := []string{
		"--nat=none", // disable nat traversal in e2e, it is failure prone and not needed
		fmt.Sprintf("--datadir=%s", eth1Path),
		fmt.Sprintf("--http.port=%d", e2e.TestParams.Ports.Eth1RPCPort+node.index),
		fmt.Sprintf("--ws.port=%d", e2e.TestParams.Ports.Eth1WSPort+node.index),
		fmt.Sprintf("--authrpc.port=%d", e2e.TestParams.Ports.Eth1AuthRPCPort+node.index),
		fmt.Sprintf("--bootnodes=%s", node.enr),
		fmt.Sprintf("--port=%d", e2e.TestParams.Ports.Eth1Port+node.index),
		fmt.Sprintf("--networkid=%d", NetworkId),
		"--http",
		"--http.api=engine,net,eth",
		"--http.addr=127.0.0.1",
		"--http.corsdomain=\"*\"",
		"--http.vhosts=\"*\"",
		"--rpc.allow-unprotected-txs",
		"--ws",
		"--ws.api=net,eth,engine",
		"--ws.addr=127.0.0.1",
		"--ws.origins=\"*\"",
		"--ipcdisable",
		"--verbosity=4",
		"--syncmode=full",
		fmt.Sprintf("--txpool.locals=%s", EthAddress),
	}

	// give the miner start a couple of tries, since the p2p networking check is flaky
	var retryErr error
	for retries := 0; retries < 3; retries++ {
		retryErr = nil
		log.Infof("Starting eth1 node %d, attempt %d with flags: %s", node.index, retries, strings.Join(args[2:], " "))
		runCmd := exec.CommandContext(ctx, binaryPath, args...) // #nosec G204 -- Safe
		errLog, err := os.Create(path.Join(e2e.TestParams.LogPath, "eth1_"+strconv.Itoa(node.index)+".log"))
		if err != nil {
			return err
		}
		runCmd.Stderr = errLog
		if err = runCmd.Start(); err != nil {
			return fmt.Errorf("failed to start eth1 chain: %w", err)
		}
		if err = helpers.WaitForTextInFile(errLog, "Started P2P networking"); err != nil {
			kerr := runCmd.Process.Kill()
			if kerr != nil {
				log.WithError(kerr).Error("error sending kill to failed node command process")
			}
			retryErr = fmt.Errorf("P2P log not found, this means the eth1 chain had issues starting: %w", err)
			continue
		}
		node.cmd = runCmd
		log.Infof("eth1 node started after %d retries", retries)
		break
	}
	if retryErr != nil {
		return retryErr
	}

	// Mark node as ready.
	close(node.started)

	return node.cmd.Wait()
}

// Started checks whether ETH1 node is started and ready to be queried.
func (node *Node) Started() <-chan struct{} {
	return node.started
}

// Pause pauses the component and its underlying process.
func (node *Node) Pause() error {
	return node.cmd.Process.Signal(syscall.SIGSTOP)
}

// Resume resumes the component and its underlying process.
func (node *Node) Resume() error {
	return node.cmd.Process.Signal(syscall.SIGCONT)
}

// Stop kills the component and its underlying process.
func (node *Node) Stop() error {
	return node.cmd.Process.Kill()
}

func (node *Node) UnderlyingProcess() *os.Process {
	return node.cmd.Process
}

package debug

import (
	"errors"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"syscall"
	"time"

	"github.com/spf13/cobra"
	"github.com/spf13/viper"

	cfg "github.com/cometbft/cometbft/config"
	"github.com/cometbft/cometbft/libs/cli"
	rpchttp "github.com/cometbft/cometbft/rpc/client/http"
)

var killCmd = &cobra.Command{
	Use:   "kill [pid] [compressed-output-file]",
	Short: "Kill a CometBFT process while aggregating and packaging debugging data",
	Long: `Kill a CometBFT process while also aggregating CometBFT process data
such as the latest node state, including consensus and networking state,
go-routine state, and the node's WAL and config information. This aggregated data
is packaged into a compressed archive.

Example:
$ cometbft debug 34255 /path/to/cmt-debug.zip`,
	Args: cobra.ExactArgs(2),
	RunE: killCmdHandler,
}

func killCmdHandler(_ *cobra.Command, args []string) error {
	pid, err := strconv.Atoi(args[0])
	if err != nil {
		return err
	}

	outFile := args[1]
	if outFile == "" {
		return errors.New("invalid output file")
	}

	rpc, err := rpchttp.New(nodeRPCAddr)
	if err != nil {
		return fmt.Errorf("failed to create new http client: %w", err)
	}

	home := viper.GetString(cli.HomeFlag)
	conf := cfg.DefaultConfig()
	conf = conf.SetRoot(home)
	cfg.EnsureRoot(conf.RootDir)

	// Create a temporary directory which will contain all the state dumps and
	// relevant files and directories that will be compressed into a file.
	tmpDir, err := os.MkdirTemp(os.TempDir(), "cometbft_debug_tmp")
	if err != nil {
		return fmt.Errorf("failed to create temporary directory: %w", err)
	}
	defer os.RemoveAll(tmpDir)

	logger.Info("getting node status...")
	if err := dumpStatus(rpc, tmpDir, "status.json"); err != nil {
		return err
	}

	logger.Info("getting node network info...")
	if err := dumpNetInfo(rpc, tmpDir, "net_info.json"); err != nil {
		return err
	}

	logger.Info("getting node consensus state...")
	if err := dumpConsensusState(rpc, tmpDir, "consensus_state.json"); err != nil {
		return err
	}

	logger.Info("copying node WAL...")
	if err := copyWAL(conf, tmpDir); err != nil {
		return err
	}

	logger.Info("copying node configuration...")
	if err := copyConfig(home, tmpDir); err != nil {
		return err
	}

	logger.Info("killing CometBFT process")
	if err := killProc(pid, tmpDir); err != nil {
		return err
	}

	logger.Info("archiving and compressing debug directory...")
	return zipDir(tmpDir, outFile)
}

// killProc attempts to kill the CometBFT process with a given PID with an
// ABORT signal which should result in a goroutine stacktrace. The PID's STDERR
// is tailed and piped to a file under the directory dir. An error is returned
// if the output file cannot be created or the tail command cannot be started.
// An error is not returned if any subsequent syscall fails.
func killProc(pid int, dir string) error {
	// pipe STDERR output from tailing the CometBFT process to a file
	//
	// NOTE: This will only work on UNIX systems.
	cmd := exec.Command("tail", "-f", fmt.Sprintf("/proc/%d/fd/2", pid)) //nolint: gosec

	outFile, err := os.Create(filepath.Join(dir, "stacktrace.out"))
	if err != nil {
		return err
	}
	defer outFile.Close()

	cmd.Stdout = outFile
	cmd.Stderr = outFile

	if err := cmd.Start(); err != nil {
		return err
	}

	// kill the underlying CometBFT process and subsequent tailing process
	go func() {
		// Killing the CometBFT process with the '-ABRT|-6' signal will result in
		// a goroutine stacktrace.
		p, err := os.FindProcess(pid)
		if err != nil {
			fmt.Fprintf(os.Stderr, "failed to find PID to kill CometBFT process: %s", err)
		} else if err = p.Signal(syscall.SIGABRT); err != nil {
			fmt.Fprintf(os.Stderr, "failed to kill CometBFT process: %s", err)
		}

		// allow some time to allow the CometBFT process to be killed
		//
		// TODO: We should 'wait' for a kill to succeed (e.g. poll for PID until it
		// cannot be found). Regardless, this should be ample time.
		time.Sleep(5 * time.Second)

		if err := cmd.Process.Kill(); err != nil {
			fmt.Fprintf(os.Stderr, "failed to kill CometBFT process output redirection: %s", err)
		}
	}()

	if err := cmd.Wait(); err != nil {
		// only return an error not invoked by a manual kill
		if _, ok := err.(*exec.ExitError); !ok {
			return err
		}
	}

	return nil
}

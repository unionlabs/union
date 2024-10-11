package debug

import (
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"time"

	"github.com/spf13/cobra"
	"github.com/spf13/viper"

	cfg "github.com/cometbft/cometbft/config"
	"github.com/cometbft/cometbft/libs/cli"
	rpchttp "github.com/cometbft/cometbft/rpc/client/http"
	cmttime "github.com/cometbft/cometbft/types/time"
)

var dumpCmd = &cobra.Command{
	Use:   "dump [output-directory]",
	Short: "Continuously poll a CometBFT process and dump debugging data into a single location",
	Long: `Continuously poll a CometBFT process and dump debugging data into a single
location at a specified frequency. At each frequency interval, an archived and compressed
file will contain node debugging information including the goroutine and heap profiles
if enabled.`,
	Args: cobra.ExactArgs(1),
	RunE: dumpCmdHandler,
}

func init() {
	dumpCmd.Flags().UintVar(
		&frequency,
		flagFrequency,
		30,
		"the frequency (seconds) in which to poll, aggregate and dump CometBFT debug data",
	)

	dumpCmd.Flags().StringVar(
		&profAddr,
		flagProfAddr,
		"",
		"the profiling server address (<host>:<port>)",
	)
}

func dumpCmdHandler(_ *cobra.Command, args []string) error {
	outDir := args[0]
	if outDir == "" {
		return errors.New("invalid output directory")
	}

	if frequency == 0 {
		return errors.New("frequency must be positive")
	}

	if _, err := os.Stat(outDir); os.IsNotExist(err) {
		if err := os.Mkdir(outDir, os.ModePerm); err != nil {
			return fmt.Errorf("failed to create output directory: %w", err)
		}
	}

	rpc, err := rpchttp.New(nodeRPCAddr)
	if err != nil {
		return fmt.Errorf("failed to create new http client: %w", err)
	}

	home := viper.GetString(cli.HomeFlag)
	conf := cfg.DefaultConfig()
	conf = conf.SetRoot(home)
	cfg.EnsureRoot(conf.RootDir)

	dumpDebugData(outDir, conf, rpc)

	ticker := time.NewTicker(time.Duration(frequency) * time.Second)
	for range ticker.C {
		dumpDebugData(outDir, conf, rpc)
	}

	return nil
}

func dumpDebugData(outDir string, conf *cfg.Config, rpc *rpchttp.HTTP) {
	start := cmttime.Now()

	tmpDir, err := os.MkdirTemp(outDir, "cometbft_debug_tmp")
	if err != nil {
		logger.Error("failed to create temporary directory", "dir", tmpDir, "error", err)
		return
	}
	defer os.RemoveAll(tmpDir)

	logger.Info("getting node status...")
	if err := dumpStatus(rpc, tmpDir, "status.json"); err != nil {
		logger.Error("failed to dump node status", "error", err)
		return
	}

	logger.Info("getting node network info...")
	if err := dumpNetInfo(rpc, tmpDir, "net_info.json"); err != nil {
		logger.Error("failed to dump node network info", "error", err)
		return
	}

	logger.Info("getting node consensus state...")
	if err := dumpConsensusState(rpc, tmpDir, "consensus_state.json"); err != nil {
		logger.Error("failed to dump node consensus state", "error", err)
		return
	}

	logger.Info("copying node WAL...")
	if err := copyWAL(conf, tmpDir); err != nil {
		logger.Error("failed to copy node WAL", "error", err)
		return
	}

	if profAddr != "" {
		logger.Info("getting node goroutine profile...")
		if err := dumpProfile(tmpDir, profAddr, "goroutine", 2); err != nil {
			logger.Error("failed to dump goroutine profile", "error", err)
			return
		}

		logger.Info("getting node heap profile...")
		if err := dumpProfile(tmpDir, profAddr, "heap", 2); err != nil {
			logger.Error("failed to dump heap profile", "error", err)
			return
		}
	}

	outFile := filepath.Join(outDir, start.Format(time.RFC3339)+".zip")
	if err := zipDir(tmpDir, outFile); err != nil {
		logger.Error("failed to create and compress archive", "file", outFile, "error", err)
	}
}

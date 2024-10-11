package debug

import (
	"os"

	"github.com/spf13/cobra"

	"github.com/cometbft/cometbft/libs/log"
)

var (
	nodeRPCAddr string
	profAddr    string
	frequency   uint

	flagNodeRPCAddr = "rpc-laddr"
	flagProfAddr    = "pprof-laddr"
	flagFrequency   = "frequency"

	logger = log.NewTMLogger(log.NewSyncWriter(os.Stdout))
)

// DebugCmd defines the root command containing subcommands that assist in
// debugging running CometBFT processes.
var DebugCmd = &cobra.Command{
	Use:   "debug",
	Short: "A utility to kill or watch a CometBFT process while aggregating debugging data",
}

func init() {
	DebugCmd.PersistentFlags().SortFlags = true
	DebugCmd.PersistentFlags().StringVar(
		&nodeRPCAddr,
		flagNodeRPCAddr,
		"tcp://localhost:26657/v1",
		"the CometBFT node's RPC address (<host>:<port>) and version",
	)

	DebugCmd.AddCommand(killCmd)
	DebugCmd.AddCommand(dumpCmd)
}

package commands

import (
	"context"
	"os"
	"os/signal"
	"syscall"

	"github.com/spf13/cobra"

	cfg "github.com/cometbft/cometbft/config"
	"github.com/cometbft/cometbft/inspect"
	"github.com/cometbft/cometbft/state"
	"github.com/cometbft/cometbft/state/indexer/block"
	"github.com/cometbft/cometbft/store"
	"github.com/cometbft/cometbft/types"
)

// InspectCmd is the command for starting an inspect server.
var InspectCmd = &cobra.Command{
	Use:   "inspect",
	Short: "Run an inspect server for investigating CometBFT state",
	Long: `
	inspect runs a subset of CometBFT's RPC endpoints that are useful for debugging
	issues with CometBFT.

	When the CometBFT detects inconsistent state, it will crash the
	CometBFT process. CometBFT will not start up while in this inconsistent state.
	The inspect command can be used to query the block and state store using CometBFT
	RPC calls to debug issues of inconsistent state.
	`,

	RunE: runInspect,
}

func init() {
	InspectCmd.Flags().
		String("rpc.laddr",
			config.RPC.ListenAddress, "RPC listenener address. Port required")
	InspectCmd.Flags().
		String("db-backend",
			config.DBBackend, "database backend: goleveldb | cleveldb | boltdb | rocksdb | badgerdb")
	InspectCmd.Flags().
		String("db-dir", config.DBPath, "database directory")
}

func runInspect(cmd *cobra.Command, _ []string) error {
	ctx, cancel := context.WithCancel(cmd.Context())
	defer cancel()

	c := make(chan os.Signal, 1)
	signal.Notify(c, syscall.SIGTERM, syscall.SIGINT)
	go func() {
		<-c
		cancel()
	}()

	blockStoreDB, err := cfg.DefaultDBProvider(&cfg.DBContext{ID: "blockstore", Config: config})
	if err != nil {
		return err
	}
	blockStore := store.NewBlockStore(blockStoreDB)
	defer blockStore.Close()

	stateDB, err := cfg.DefaultDBProvider(&cfg.DBContext{ID: "state", Config: config})
	if err != nil {
		return err
	}
	stateStore := state.NewStore(stateDB, state.StoreOptions{DiscardABCIResponses: false})
	defer stateStore.Close()

	genDoc, err := types.GenesisDocFromFile(config.GenesisFile())
	if err != nil {
		return err
	}
	txIndexer, blockIndexer, err := block.IndexerFromConfig(config, cfg.DefaultDBProvider, genDoc.ChainID)
	if err != nil {
		return err
	}
	ins := inspect.New(config.RPC, blockStore, stateStore, txIndexer, blockIndexer)

	logger.Info("starting inspect server")
	return ins.Run(ctx)
}

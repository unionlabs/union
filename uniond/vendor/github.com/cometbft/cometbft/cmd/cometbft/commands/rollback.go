package commands

import (
	"fmt"
	"path/filepath"

	"github.com/spf13/cobra"

	dbm "github.com/cometbft/cometbft-db"

	cfg "github.com/cometbft/cometbft/config"
	"github.com/cometbft/cometbft/libs/os"
	"github.com/cometbft/cometbft/state"
	"github.com/cometbft/cometbft/store"
)

var removeBlock = false

func init() {
	RollbackStateCmd.Flags().BoolVar(&removeBlock, "hard", false, "remove last block as well as state")
}

var RollbackStateCmd = &cobra.Command{
	Use:   "rollback",
	Short: "rollback CometBFT state by one height",
	Long: `
A state rollback is performed to recover from an incorrect application state transition,
when CometBFT has persisted an incorrect app hash and is thus unable to make
progress. Rollback overwrites a state at height n with the state at height n - 1.
The application should also roll back to height n - 1. If the --hard flag is not used, 
no blocks will be removed so upon restarting CometBFT the transactions in block n will be 
re-executed against the application. Using --hard will also remove block n. This can
be done multiple times.
`,
	RunE: func(cmd *cobra.Command, args []string) error {
		height, hash, err := RollbackState(config, removeBlock)
		if err != nil {
			return fmt.Errorf("failed to rollback state: %w", err)
		}

		if removeBlock {
			fmt.Printf("Rolled back both state and block to height %d and hash %X\n", height, hash)
		} else {
			fmt.Printf("Rolled back state to height %d and hash %X\n", height, hash)
		}

		return nil
	},
}

// RollbackState takes the state at the current height n and overwrites it with the state
// at height n - 1. Note state here refers to CometBFT state not application state.
// Returns the latest state height and app hash alongside an error if there was one.
func RollbackState(config *cfg.Config, removeBlock bool) (int64, []byte, error) {
	// use the parsed config to load the block and state store
	blockStore, stateStore, err := loadStateAndBlockStore(config)
	if err != nil {
		return -1, nil, err
	}
	defer func() {
		_ = blockStore.Close()
		_ = stateStore.Close()
	}()

	// rollback the last state
	return state.Rollback(blockStore, stateStore, removeBlock)
}

func loadStateAndBlockStore(config *cfg.Config) (*store.BlockStore, state.Store, error) {
	dbType := dbm.BackendType(config.DBBackend)

	if !os.FileExists(filepath.Join(config.DBDir(), "blockstore.db")) {
		return nil, nil, fmt.Errorf("no blockstore found in %v", config.DBDir())
	}

	// Get BlockStore
	blockStoreDB, err := dbm.NewDB("blockstore", dbType, config.DBDir())
	if err != nil {
		return nil, nil, err
	}
	blockStore := store.NewBlockStore(blockStoreDB)

	if !os.FileExists(filepath.Join(config.DBDir(), "state.db")) {
		return nil, nil, fmt.Errorf("no statestore found in %v", config.DBDir())
	}

	// Get StateStore
	stateDB, err := dbm.NewDB("state", dbType, config.DBDir())
	if err != nil {
		return nil, nil, err
	}
	stateStore := state.NewStore(stateDB, state.StoreOptions{
		DiscardABCIResponses: config.Storage.DiscardABCIResponses,
	})

	return blockStore, stateStore, nil
}

package pruning

import (
	"errors"
	"fmt"
	"path/filepath"

	dbm "github.com/cosmos/cosmos-db"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"

	corestore "cosmossdk.io/core/store"
	"cosmossdk.io/log"
	pruningtypes "cosmossdk.io/store/pruning/types"
	"cosmossdk.io/store/rootmulti"

	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/server"
	servertypes "github.com/cosmos/cosmos-sdk/server/types"
	"github.com/cosmos/cosmos-sdk/version"
)

const FlagAppDBBackend = "app-db-backend"

// Cmd prunes the sdk root multi store history versions based on the pruning options
// specified by command flags.
func Cmd[T servertypes.Application](appCreator servertypes.AppCreator[T]) *cobra.Command {
	cmd := &cobra.Command{
		Use:   "prune [pruning-method]",
		Short: "Prune app history states by keeping the recent heights and deleting old heights",
		Long: `Prune app history states by keeping the recent heights and deleting old heights.
The pruning option is provided via the 'pruning' argument or alternatively with '--pruning-keep-recent'

- default: the last 362880 states are kept
- nothing: all historic states will be saved, nothing will be deleted (i.e. archiving node)
- everything: 2 latest states will be kept
- custom: allow pruning options to be manually specified through 'pruning-keep-recent'

Note: When the --app-db-backend flag is not specified, the default backend type is 'goleveldb'.
Supported app-db-backend types include 'goleveldb', 'rocksdb', 'pebbledb'.`,
		Example: fmt.Sprintf("%s prune custom --pruning-keep-recent 100 --app-db-backend 'goleveldb'", version.AppName),
		Args:    cobra.RangeArgs(0, 1),
		RunE: func(cmd *cobra.Command, args []string) error {
			// bind flags to the Context's Viper so we can get pruning options.
			vp := viper.New()
			if err := vp.BindPFlags(cmd.Flags()); err != nil {
				return err
			}
			if err := vp.BindPFlags(cmd.PersistentFlags()); err != nil {
				return err
			}

			// use the first argument if present to set the pruning method
			if len(args) > 0 {
				vp.Set(server.FlagPruning, args[0])
			} else {
				vp.Set(server.FlagPruning, pruningtypes.PruningOptionDefault)
			}
			pruningOptions, err := server.GetPruningOptionsFromFlags(vp)
			if err != nil {
				return err
			}

			cmd.Printf("get pruning options from command flags, strategy: %v, keep-recent: %v\n",
				pruningOptions.Strategy,
				pruningOptions.KeepRecent,
			)

			home := vp.GetString(flags.FlagHome)
			db, err := openDB(home, server.GetAppDBBackend(vp))
			if err != nil {
				return err
			}

			logger := log.NewLogger(cmd.OutOrStdout())
			app := appCreator(logger, db, nil, vp)
			cms := app.CommitMultiStore()

			rootMultiStore, ok := cms.(*rootmulti.Store)
			if !ok {
				return errors.New("currently only support the pruning of rootmulti.Store type")
			}
			latestHeight := rootmulti.GetLatestVersion(db)
			// valid heights should be greater than 0.
			if latestHeight <= 0 {
				return fmt.Errorf("the database has no valid heights to prune, the latest height: %v", latestHeight)
			}

			pruningHeight := latestHeight - int64(pruningOptions.KeepRecent)
			cmd.Printf("pruning heights up to %v\n", pruningHeight)

			err = rootMultiStore.PruneStores(pruningHeight)
			if err != nil {
				return err
			}

			cmd.Println("successfully pruned the application root multi stores")
			return nil
		},
	}

	cmd.Flags().String(FlagAppDBBackend, "", "The type of database for application and snapshots databases")
	cmd.Flags().Uint64(server.FlagPruningKeepRecent, 0, "Number of recent heights to keep on disk (ignored if pruning is not 'custom')")
	cmd.Flags().Uint64(server.FlagPruningInterval, 10,
		`Height interval at which pruned heights are removed from disk (ignored if pruning is not 'custom'), 
		this is not used by this command but kept for compatibility with the complete pruning options`)

	return cmd
}

func openDB(rootDir string, backendType dbm.BackendType) (corestore.KVStoreWithBatch, error) {
	dataDir := filepath.Join(rootDir, "data")
	return dbm.NewDB("application", backendType, dataDir)
}

package pruning

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/spf13/cobra"
	"github.com/spf13/viper"

	dbm "github.com/cometbft/cometbft-db"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/server"
	servertypes "github.com/cosmos/cosmos-sdk/server/types"
	pruningtypes "github.com/cosmos/cosmos-sdk/store/pruning/types"
	"github.com/cosmos/cosmos-sdk/store/rootmulti"
)

const FlagAppDBBackend = "app-db-backend"

// PruningCmd prunes the sdk root multi store history versions based on the pruning options
// specified by command flags.
// Deprecated: Use Cmd instead.
func PruningCmd(appCreator servertypes.AppCreator) *cobra.Command {
	cmd := Cmd(appCreator, "")
	cmd.Flags().String(server.FlagPruning, pruningtypes.PruningOptionDefault, "Pruning strategy (default|nothing|everything|custom)")

	return cmd
}

// Cmd prunes the sdk root multi store history versions based on the pruning options
// specified by command flags.
func Cmd(appCreator servertypes.AppCreator, defaultNodeHome string) *cobra.Command {
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
		Example: "prune custom --pruning-keep-recent 100 --app-db-backend 'goleveldb'",
		Args:    cobra.RangeArgs(0, 1),
		RunE: func(cmd *cobra.Command, args []string) error {
			// bind flags to the Context's Viper so we can get pruning options.
			vp := viper.New()
			if err := vp.BindPFlags(cmd.Flags()); err != nil {
				return err
			}

			// use the first argument if present to set the pruning method
			if len(args) > 0 {
				vp.Set(server.FlagPruning, args[0])
			} else if vp.GetString(server.FlagPruning) == "" { // this differs from orignal https://github.com/cosmos/cosmos-sdk/pull/16856 for compatibility
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
			if home == "" {
				home = defaultNodeHome
			}

			db, err := openDB(home, server.GetAppDBBackend(vp))
			if err != nil {
				return err
			}

			logger := log.NewTMLogger(log.NewSyncWriter(os.Stdout))
			app := appCreator(logger, db, nil, vp)
			cms := app.CommitMultiStore()

			rootMultiStore, ok := cms.(*rootmulti.Store)
			if !ok {
				return fmt.Errorf("currently only support the pruning of rootmulti.Store type")
			}
			latestHeight := rootmulti.GetLatestVersion(db)
			// valid heights should be greater than 0.
			if latestHeight <= 0 {
				return fmt.Errorf("the database has no valid heights to prune, the latest height: %v", latestHeight)
			}

			var pruningHeights []int64
			for height := int64(1); height < latestHeight; height++ {
				if height < latestHeight-int64(pruningOptions.KeepRecent) {
					pruningHeights = append(pruningHeights, height)
				}
			}
			if len(pruningHeights) == 0 {
				cmd.Println("no heights to prune")
				return nil
			}
			cmd.Printf("pruning heights start from %v, end at %v\n", pruningHeights[0], pruningHeights[len(pruningHeights)-1])

			if err = rootMultiStore.PruneStores(false, pruningHeights); err != nil {
				return err
			}

			cmd.Println("successfully pruned the application root multi stores")
			return nil
		},
	}

	cmd.Flags().String(flags.FlagHome, defaultNodeHome, "The application home directory")
	cmd.Flags().String(FlagAppDBBackend, "", "The type of database for application and snapshots databases")
	cmd.Flags().Uint64(server.FlagPruningKeepRecent, 0, "Number of recent heights to keep on disk (ignored if pruning is not 'custom')")
	cmd.Flags().Uint64(server.FlagPruningInterval, 10,
		`Height interval at which pruned heights are removed from disk (ignored if pruning is not 'custom'), 
		this is not used by this command but kept for compatibility with the complete pruning options`)

	return cmd
}

func openDB(rootDir string, backendType dbm.BackendType) (dbm.DB, error) {
	dataDir := filepath.Join(rootDir, "data")
	return dbm.NewDB("application", backendType, dataDir)
}

package snapshot

import (
	"path/filepath"
	"strconv"

	dbm "github.com/cosmos/cosmos-db"
	"github.com/spf13/cobra"

	corestore "cosmossdk.io/core/store"
	"cosmossdk.io/log"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/server"
	servertypes "github.com/cosmos/cosmos-sdk/server/types"
)

// RestoreSnapshotCmd returns a command to restore a snapshot
func RestoreSnapshotCmd[T servertypes.Application](appCreator servertypes.AppCreator[T]) *cobra.Command {
	cmd := &cobra.Command{
		Use:   "restore <height> <format>",
		Short: "Restore app state from local snapshot",
		Long:  "Restore app state from local snapshot",
		Args:  cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			cfg := client.GetConfigFromCmd(cmd)
			viper := client.GetViperFromCmd(cmd)

			height, err := strconv.ParseUint(args[0], 10, 64)
			if err != nil {
				return err
			}
			format, err := strconv.ParseUint(args[1], 10, 32)
			if err != nil {
				return err
			}

			home := cfg.RootDir
			db, err := openDB(home, server.GetAppDBBackend(viper))
			if err != nil {
				return err
			}
			logger := log.NewLogger(cmd.OutOrStdout())
			app := appCreator(logger, db, nil, viper)

			sm := app.SnapshotManager()
			return sm.RestoreLocalSnapshot(height, uint32(format))
		},
	}
	return cmd
}

func openDB(rootDir string, backendType dbm.BackendType) (corestore.KVStoreWithBatch, error) {
	dataDir := filepath.Join(rootDir, "data")
	return dbm.NewDB("application", backendType, dataDir)
}

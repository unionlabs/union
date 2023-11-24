package snapshot

import (
	"github.com/spf13/cobra"

	"cosmossdk.io/log"

	"github.com/cosmos/cosmos-sdk/server"
	servertypes "github.com/cosmos/cosmos-sdk/server/types"
)

// ExportSnapshotCmd returns a command to take a snapshot of the application state
func ExportSnapshotCmd(appCreator servertypes.AppCreator) *cobra.Command {
	cmd := &cobra.Command{
		Use:   "export",
		Short: "Export app state to snapshot store",
		Args:  cobra.NoArgs,
		RunE: func(cmd *cobra.Command, args []string) error {
			ctx := server.GetServerContextFromCmd(cmd)

			height, err := cmd.Flags().GetInt64("height")
			if err != nil {
				return err
			}

			home := ctx.Config.RootDir
			db, err := openDB(home, server.GetAppDBBackend(ctx.Viper))
			if err != nil {
				return err
			}
			logger := log.NewLogger(cmd.OutOrStdout())
			app := appCreator(logger, db, nil, ctx.Viper)

			if height == 0 {
				height = app.CommitMultiStore().LastCommitID().Version
			}

			cmd.Printf("Exporting snapshot for height %d\n", height)

			sm := app.SnapshotManager()
			snapshot, err := sm.Create(uint64(height))
			if err != nil {
				return err
			}

			cmd.Printf("Snapshot created at height %d, format %d, chunks %d\n", snapshot.Height, snapshot.Format, snapshot.Chunks)
			return nil
		},
	}

	cmd.Flags().Int64("height", 0, "Height to export, default to latest state height")

	return cmd
}

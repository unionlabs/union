package snapshot

import (
	"github.com/spf13/cobra"

	servertypes "github.com/cosmos/cosmos-sdk/server/types"
)

// Cmd returns the snapshots group command
func Cmd[T servertypes.Application](appCreator servertypes.AppCreator[T]) *cobra.Command {
	cmd := &cobra.Command{
		Use:   "snapshots",
		Short: "Manage local snapshots",
	}
	cmd.AddCommand(
		ListSnapshotsCmd,
		RestoreSnapshotCmd(appCreator),
		ExportSnapshotCmd(appCreator),
		DumpArchiveCmd(),
		LoadArchiveCmd(),
		DeleteSnapshotCmd(),
	)
	return cmd
}

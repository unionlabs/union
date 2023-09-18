package snapshot

import (
	"strconv"

	"github.com/cosmos/cosmos-sdk/server"
	"github.com/spf13/cobra"
)

func DeleteSnapshotCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "delete <height> <format>",
		Short: "Delete a local snapshot",
		Args:  cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			ctx := server.GetServerContextFromCmd(cmd)

			height, err := strconv.ParseUint(args[0], 10, 64)
			if err != nil {
				return err
			}
			format, err := strconv.ParseUint(args[1], 10, 32)
			if err != nil {
				return err
			}

			snapshotStore, err := server.GetSnapshotStore(ctx.Viper)
			if err != nil {
				return err
			}

			return snapshotStore.Delete(height, uint32(format))
		},
	}
}

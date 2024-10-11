package snapshot

import (
	"strconv"

	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/server"
)

func DeleteSnapshotCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "delete <height> <format>",
		Short: "Delete a local snapshot",
		Args:  cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			viper := client.GetViperFromCmd(cmd)

			height, err := strconv.ParseUint(args[0], 10, 64)
			if err != nil {
				return err
			}
			format, err := strconv.ParseUint(args[1], 10, 32)
			if err != nil {
				return err
			}

			snapshotStore, err := server.GetSnapshotStore(viper)
			if err != nil {
				return err
			}

			return snapshotStore.Delete(height, uint32(format))
		},
	}
}

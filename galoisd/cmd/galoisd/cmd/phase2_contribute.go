package cmd

import (
	mpc "github.com/consensys/gnark/backend/groth16/bn254/mpcsetup"
	"github.com/spf13/cobra"
)

func Phase2ContributeCmd() *cobra.Command {
	var cmd = &cobra.Command{
		Short: "Initialize the phase 2 of the groth16 multi-party computation.",
		Use:   "mpc-phase2-contrib [phase2] [phase2Output]",
		Args:  cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			phase2Path := args[0]
			var srs2 mpc.Phase2
			err := readFrom(phase2Path, &srs2)
			if err != nil {
				return err
			}
			srs2.Contribute()
			phase2Output := args[1]
			return saveTo(phase2Output, &srs2)
		},
	}
	return cmd
}

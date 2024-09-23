package cmd

import (
	mpc "github.com/consensys/gnark/backend/groth16/bn254/mpcsetup"
	"github.com/spf13/cobra"
)

func Phase2VerifyCmd() *cobra.Command {
	var cmd = &cobra.Command{
		Short: "Verify a phase 2 contribution of the groth16 multi-party computation.",
		Use:   "mpc-phase2-verify [phase2Previous] [phase2Contrib]",
		Args:  cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			phase2Previous := args[0]
			var prev mpc.Phase2
			err := readFrom(phase2Previous, &prev)
			if err != nil {
				return err
			}
			phase2Contrib := args[1]
			var contrib mpc.Phase2
			err = readFrom(phase2Contrib, &contrib)
			if err != nil {
				return err
			}
			return mpc.VerifyPhase2(&prev, &contrib)
		},
	}
	return cmd
}

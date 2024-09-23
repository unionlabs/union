package cmd

import (
	mpc "github.com/consensys/gnark/backend/groth16/bn254/mpcsetup"
	bn254 "github.com/consensys/gnark/constraint/bn254"
	"github.com/spf13/cobra"
)

func Phase2InitCmd() *cobra.Command {
	var cmd = &cobra.Command{
		Short: "Initialize the phase 2 of the groth16 multi-party computation.",
		Use:   "mpc-phase2-init [r1cs] [phase1Final] [phase2InitOutput] [phase2EvalsOutput]",
		Args:  cobra.ExactArgs(4),
		RunE: func(cmd *cobra.Command, args []string) error {
			r1csPath := args[0]
			var r1cs bn254.R1CS
			err := readFrom(r1csPath, &r1cs)
			if err != nil {
				return err
			}
			phase1Path := args[1]
			var srs1 mpc.Phase1
			err = readFrom(phase1Path, &srs1)
			if err != nil {
				return err
			}
			srs2, evals := mpc.InitPhase2(&r1cs, &srs1)
			phase2InitPath := args[2]
			err = saveTo(phase2InitPath, &srs2)
			if err != nil {
				return err
			}
			phase2EvalsOutput := args[3]
			return saveTo(phase2EvalsOutput, &evals)
		},
	}
	return cmd
}

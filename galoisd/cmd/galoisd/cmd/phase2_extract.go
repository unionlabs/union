package cmd

import (
	"fmt"

	mpc "github.com/consensys/gnark/backend/groth16/bn254/mpcsetup"
	bn254 "github.com/consensys/gnark/constraint/bn254"
	"github.com/spf13/cobra"
)

func Phase2ExtractCmd() *cobra.Command {
	var cmd = &cobra.Command{
		Short: "Initialize the phase 2 of the groth16 multi-party computation.",
		Use:   "mpc-phase2-extract [r1cs] [phase1Final] [phase2Final] [phase2Evals] [provingKeyOutput] [verifyingKeyOutput]",
		Args:  cobra.ExactArgs(6),
		RunE: func(cmd *cobra.Command, args []string) error {
			r1csPath := args[0]
			var r1cs bn254.R1CS
			err := readFrom(r1csPath, &r1cs)
			if err != nil {
				return fmt.Errorf("failed to read r1cs: %v", err)
			}
			phase1Path := args[1]
			var srs1 mpc.Phase1
			err = readFrom(phase1Path, &srs1)
			if err != nil {
				return fmt.Errorf("failed to read phase1: %v", err)
			}
			phase2Path := args[2]
			var srs2 mpc.Phase2
			err = readFrom(phase2Path, &srs2)
			if err != nil {
				return fmt.Errorf("failed to read phase2: %v", err)
			}
			phase2EvalsPath := args[3]
			var evals mpc.Phase2Evaluations
			err = readFrom(phase2EvalsPath, &evals)
			if err != nil {
				return fmt.Errorf("failed to read phase2 evals: %v", err)
			}
			pk, vk := mpc.ExtractKeys(&r1cs, &srs1, &srs2, &evals)
			pkOutput := args[4]
			err = saveTo(pkOutput, &pk)
			if err != nil {
				return fmt.Errorf("failed to write pk: %v", err)
			}
			vkOutput := args[5]
			return saveTo(vkOutput, &vk)
		},
	}
	return cmd
}

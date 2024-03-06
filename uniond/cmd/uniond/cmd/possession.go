package cmd

import (
	"encoding/base64"
	"fmt"

	"github.com/cometbft/cometbft/crypto/bn254"
	"github.com/spf13/cobra"
)

func ProofOfPossession() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "prove-possession [private_key]",
		Short: "!!!WARNING!!! this command is unsecure, do not ever use it in production !!!",
		Long:  ``,
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			privKeyBytes, err := base64.StdEncoding.DecodeString(args[0])
			if err != nil {
				return err
			}
			privKey := bn254.PrivKey(privKeyBytes)
			sig, err := privKey.Sign(privKey.PubKey().Bytes())
			if err != nil {
				return err
			}
			fmt.Printf("%X\n", sig)
			return nil
		},
	}
	return cmd
}

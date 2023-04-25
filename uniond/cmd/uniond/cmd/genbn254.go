package cmd

import (
	"fmt"

	"github.com/cometbft/cometbft/crypto/bn254"
	cmtjson "github.com/cometbft/cometbft/libs/json"
	"github.com/cometbft/cometbft/privval"
	"github.com/spf13/cobra"
)

func GenBn254() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "genbn",
		Short: "Generate a BN254 key pair",
		Long:  `Generate a BN254 key pair`,
		Args:  cobra.ExactArgs(0),
		RunE: func(cmd *cobra.Command, args []string) error {
			privKey := bn254.GenPrivKey()
			val := privval.FilePVKey{
				Address: privKey.PubKey().Address(),
				PubKey:  privKey.PubKey(),
				PrivKey: privKey,
			}
			keyRepr, err := cmtjson.Marshal(val)
			if err != nil {
				return err
			}
			fmt.Printf("%s", keyRepr)
			return nil
		},
	}
	return cmd
}

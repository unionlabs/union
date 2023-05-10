package cmd

import (
	"encoding/hex"
	"fmt"

	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	ctypes "github.com/cosmos/cosmos-sdk/codec/types"
	commitmenttypes "github.com/cosmos/ibc-go/v7/modules/core/23-commitment/types"
	"github.com/spf13/cobra"
)

const (
	flagNodeURI = "node"
)

func GenStateProof() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "genstateproof [data] [path]",
		Short: "Generate a state proof.",
		Long:  `Generate a state proof.`,
		Args:  cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {

			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			nodeURI, err := cmd.Flags().GetString(flagNodeURI)
			if err != nil {
				return err
			}

			clientCtx.NodeURI =  nodeURI

			data := args[0];
			path := args[1];

			res, err := clientCtx.QueryABCI(abci.RequestQuery{
				Data: []byte(data),
				Path: path,
				Height: 2,
				Prove: true,
			})
			if err != nil {
				return err
			}

			fmt.Println(res.ProofOps)

			merkleProof, err := commitmenttypes.ConvertProofs(res.ProofOps)
			if err != nil {
				return err
			}

			fmt.Println(merkleProof)

			cdc := codec.NewProtoCodec(ctypes.NewInterfaceRegistry())
			proofBz, err := cdc.Marshal(&merkleProof)
			if err != nil {
				return err
			}

			fmt.Println(hex.EncodeToString(proofBz));

			return nil
		},
	}
	cmd.Flags().String(flagNodeURI, "tcp://localhost:26657", "The node URI")
	return cmd
}

package cmd

import (
	"encoding/hex"
	"fmt"
	"strconv"

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
		Use:   "genstateproof [height] [data] [path]",
		Short: "Generate a state proof.",
		Long:  `Generate a state proof.`,
		Args:  cobra.ExactArgs(3),
		RunE: func(cmd *cobra.Command, args []string) error {

			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			nodeURI, err := cmd.Flags().GetString(flagNodeURI)
			if err != nil {
				return err
			}

			clientCtx.NodeURI = nodeURI

			height, err := strconv.Atoi(args[0])
			if err != nil {
				return err
			}

			data := args[1]
			path := args[2]

			res, err := clientCtx.QueryABCI(abci.RequestQuery{
				Data:   []byte(data),
				Path:   path,
				Height: int64(height),
				Prove:  true,
			})
			if err != nil {
				return err
			}

			merkleProof, err := commitmenttypes.ConvertProofs(res.ProofOps)
			if err != nil {
				return err
			}

			cdc := codec.NewProtoCodec(ctypes.NewInterfaceRegistry())
			proofBz, err := cdc.Marshal(&merkleProof)
			if err != nil {
				return err
			}

			fmt.Println(hex.EncodeToString(proofBz))

			return nil
		},
	}
	cmd.Flags().String(flagNodeURI, "tcp://localhost:26657", "The node URI")
	return cmd
}

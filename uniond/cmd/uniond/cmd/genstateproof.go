package cmd

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"strconv"

	"cosmossdk.io/x/tx/signing"

	abci "github.com/cometbft/cometbft/abci/types"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/address"
	ctypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/gogoproto/proto"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
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

			data, err := hex.DecodeString(args[1])
			if err != nil {
				return err
			}

			path := args[2]

			res, err := clientCtx.QueryABCI(abci.RequestQuery{
				Data:   data,
				Path:   path,
				Height: int64(height),
				Prove:  true,
			})
			if err != nil {
				return err
			}

			fmt.Println(res)

			merkleProof, err := commitmenttypes.ConvertProofs(res.ProofOps)
			if err != nil {
				return err
			}

			fmt.Println(merkleProof)

			interfaceRegistry, err := ctypes.NewInterfaceRegistryWithOptions(ctypes.InterfaceRegistryOptions{
				ProtoFiles: proto.HybridResolver,
				SigningOptions: signing.Options{
					AddressCodec: address.Bech32Codec{
						Bech32Prefix: sdk.GetConfig().GetBech32AccountAddrPrefix(),
					},
					ValidatorAddressCodec: address.Bech32Codec{
						Bech32Prefix: sdk.GetConfig().GetBech32ValidatorAddrPrefix(),
					},
				},
			})
			if err != nil {
				return err
			}

			cdc := codec.NewProtoCodec(interfaceRegistry)
			proofBz, err := cdc.Marshal(&merkleProof)
			if err != nil {
				return err
			}

			proofJson, err := json.Marshal(&merkleProof)
			if err != nil {
				return err
			}

			fmt.Println("Proof for height: ", res.Height)
			fmt.Println("Proof: ", hex.EncodeToString(proofBz))
			fmt.Println("Proof JSON: ", string(proofJson))
			fmt.Println("Key: ", hex.EncodeToString(res.Key))
			fmt.Println("Value: ", hex.EncodeToString(res.Value))

			return nil
		},
	}
	cmd.Flags().String(flagNodeURI, "tcp://localhost:26657", "The node URI")
	return cmd
}

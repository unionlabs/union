package cli

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	clienttx "github.com/cosmos/cosmos-sdk/client/tx"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/types/tx"
	authclient "github.com/cosmos/cosmos-sdk/x/auth/client"
)

func GetAuxToFeeCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "aux-to-fee <aux_signed_tx.json>",
		Short: "Includes the aux signer data in the tx, broadcast the tx, and sends the tip amount to the broadcaster",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			auxSignerData := tx.AuxSignerData{}
			err = readAuxSignerData(clientCtx.Codec, &auxSignerData, args[0])
			if err != nil {
				return err
			}

			if auxSignerData.SignDoc.ChainId != clientCtx.ChainID {
				return fmt.Errorf("expected chain-id %s, got %s in aux signer data", clientCtx.ChainID, auxSignerData.SignDoc.ChainId)
			}

			f, err := clienttx.NewFactoryCLI(clientCtx, cmd.Flags())
			if err != nil {
				return err
			}

			txBuilder := clientCtx.TxConfig.NewTxBuilder()
			err = txBuilder.AddAuxSignerData(auxSignerData)
			if err != nil {
				return err
			}

			txBuilder.SetFeePayer(clientCtx.FromAddress)
			txBuilder.SetFeeAmount(f.Fees())
			txBuilder.SetGasLimit(f.Gas())

			if clientCtx.GenerateOnly {
				json, err := clientCtx.TxConfig.TxJSONEncoder()(txBuilder.GetTx())
				if err != nil {
					return err
				}
				return clientCtx.PrintString(fmt.Sprintf("%s\n", json))
			}

			err = authclient.SignTx(f, clientCtx, clientCtx.FromName, txBuilder, clientCtx.Offline, false)
			if err != nil {
				return err
			}

			txBytes, err := clientCtx.TxConfig.TxEncoder()(txBuilder.GetTx())
			if err != nil {
				return err
			}

			// broadcast to a Tendermint node
			res, err := clientCtx.BroadcastTx(txBytes)
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

func readAuxSignerData(cdc codec.Codec, auxSignerData *tx.AuxSignerData, filename string) error {
	bytes, err := os.ReadFile(filename)
	if err != nil {
		return err
	}

	return cdc.UnmarshalJSON(bytes, auxSignerData)
}

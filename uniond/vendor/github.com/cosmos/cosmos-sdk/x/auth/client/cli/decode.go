package cli

import (
	"encoding/base64"
	"encoding/hex"

	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
)

const flagHex = "hex"

// GetDecodeCommand returns the decode command to take serialized bytes and turn
// it into a JSON-encoded transaction.
func GetDecodeCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "decode [protobuf-byte-string]",
		Short: "Decode a binary encoded transaction string",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) (err error) {
			clientCtx := client.GetClientContextFromCmd(cmd)
			var txBytes []byte

			if useHex, _ := cmd.Flags().GetBool(flagHex); useHex {
				txBytes, err = hex.DecodeString(args[0])
			} else {
				txBytes, err = base64.StdEncoding.DecodeString(args[0])
			}
			if err != nil {
				return err
			}

			tx, err := clientCtx.TxConfig.TxDecoder()(txBytes)
			if err != nil {
				return err
			}

			json, err := clientCtx.TxConfig.TxJSONEncoder()(tx)
			if err != nil {
				return err
			}

			return clientCtx.PrintBytes(json)
		},
	}

	cmd.Flags().BoolP(flagHex, "x", false, "Treat input as hexadecimal instead of base64")
	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

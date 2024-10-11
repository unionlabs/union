package cli

import (
	"errors"
	"fmt"
	"strings"

	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/version"
	authclient "github.com/cosmos/cosmos-sdk/x/auth/client"
)

// GetBroadcastCommand returns the tx broadcast command.
func GetBroadcastCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "broadcast <file_path>",
		Short: "Broadcast transactions generated offline",
		Long: strings.TrimSpace(`Broadcast transactions created with the --generate-only
flag and signed with the sign command. Read a transaction from [file_path] and
broadcast it to a node. If you supply a dash (-) argument in place of an input
filename, the command reads from standard input.`),
		Example: fmt.Sprintf("%s tx broadcast <file_path>", version.AppName),
		Args:    cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			if offline, _ := cmd.Flags().GetBool(flags.FlagOffline); offline {
				return errors.New("cannot broadcast tx during offline mode")
			}

			txs, err := authclient.ReadTxsFromFile(clientCtx, args[0])
			if err != nil {
				return err
			}

			txEncoder := clientCtx.TxConfig.TxEncoder()
			for _, tx := range txs {
				txBytes, err1 := txEncoder(tx)
				if err1 != nil {
					err = errors.Join(err, err1)
					continue
				}

				res, err2 := clientCtx.BroadcastTx(txBytes)
				if err2 != nil {
					err = errors.Join(err, err2)
					continue
				}
				if res != nil {
					err3 := clientCtx.PrintProto(res)
					if err3 != nil {
						err = errors.Join(err, err3)
					}
				}
			}
			return err
		},
	}

	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

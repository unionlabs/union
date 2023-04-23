package cli

import (
	"fmt"
	"os"
	"strings"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/client/tx"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/spf13/cobra"

	"github.com/cosmos/ibc-go/v7/modules/apps/27-interchain-accounts/controller/types"
	icatypes "github.com/cosmos/ibc-go/v7/modules/apps/27-interchain-accounts/types"
)

const (
	// The controller chain channel version
	flagVersion               = "version"
	flagRelativePacketTimeout = "relative-packet-timeout"
)

func newRegisterInterchainAccountCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "register [connection-id]",
		Short: "Register an interchain account on the provided connection.",
		Long: strings.TrimSpace(`Register an account on the counterparty chain via the 
connection id from the source chain. Connection identifier should be for the source chain 
and the interchain account will be created on the counterparty chain. Callers are expected to 
provide the appropriate application version string via {version} flag. Generates a new 
port identifier using the provided owner string, binds to the port identifier and claims 
the associated capability.`),
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}

			connectionID := args[0]
			owner := clientCtx.GetFromAddress().String()
			version, err := cmd.Flags().GetString(flagVersion)
			if err != nil {
				return err
			}

			msg := types.NewMsgRegisterInterchainAccount(connectionID, owner, version)

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	cmd.Flags().String(flagVersion, "", "Controller chain channel version")
	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

func newSendTxCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "send-tx [connection-id] [path/to/packet_msg.json]",
		Short: "Send an interchain account tx on the provided connection.",
		Long: strings.TrimSpace(`Submits pre-built packet data containing messages to be executed on the host chain 
and attempts to send the packet. Packet data is provided as json, file or string. An 
appropriate relative timeoutTimestamp must be provided with flag {relative-packet-timeout}`),
		Args: cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}
			cdc := codec.NewProtoCodec(clientCtx.InterfaceRegistry)

			connectionID := args[0]
			owner := clientCtx.GetFromAddress().String()

			// attempt to unmarshal ica msg data argument
			var icaMsgData icatypes.InterchainAccountPacketData
			msgContentOrFileName := args[1]
			if err := cdc.UnmarshalJSON([]byte(msgContentOrFileName), &icaMsgData); err != nil {

				// check for file path if JSON input is not provided
				contents, err := os.ReadFile(msgContentOrFileName)
				if err != nil {
					return fmt.Errorf("neither JSON input nor path to .json file for packet data with messages were provided: %w", err)
				}

				if err := cdc.UnmarshalJSON(contents, &icaMsgData); err != nil {
					return fmt.Errorf("error unmarshalling packet data with messages file: %w", err)
				}
			}

			relativeTimeoutTimestamp, err := cmd.Flags().GetUint64(flagRelativePacketTimeout)
			if err != nil {
				return err
			}

			msg := types.NewMsgSendTx(owner, connectionID, relativeTimeoutTimestamp, icaMsgData)

			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	cmd.Flags().Uint64(flagRelativePacketTimeout, icatypes.DefaultRelativePacketTimeoutTimestamp, "Relative packet timeout in nanoseconds from now. Default is 10 minutes.")
	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

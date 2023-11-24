package cli

import (
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/client/tx"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/version"

	"github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	clientutils "github.com/cosmos/ibc-go/v8/modules/core/02-client/client/utils"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	channelutils "github.com/cosmos/ibc-go/v8/modules/core/04-channel/client/utils"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

const (
	flagPacketTimeoutHeight    = "packet-timeout-height"
	flagPacketTimeoutTimestamp = "packet-timeout-timestamp"
	flagAbsoluteTimeouts       = "absolute-timeouts"
	flagMemo                   = "memo"
)

// NewTransferTxCmd returns the command to create a NewMsgTransfer transaction
func NewTransferTxCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "transfer [src-port] [src-channel] [receiver] [amount]",
		Short: "Transfer a fungible token through IBC",
		Long: strings.TrimSpace(`Transfer a fungible token through IBC. Timeouts can be specified
as absolute or relative using the "absolute-timeouts" flag. Timeout height can be set by passing in the height string
in the form {revision}-{height} using the "packet-timeout-height" flag. Relative timeout height is added to the block
height queried from the latest consensus state corresponding to the counterparty channel. Relative timeout timestamp 
is added to the greater value of the local clock time and the block timestamp queried from the latest consensus state 
corresponding to the counterparty channel. Any timeout set to 0 is disabled.`),
		Example: fmt.Sprintf("%s tx ibc-transfer transfer [src-port] [src-channel] [receiver] [amount]", version.AppName),
		Args:    cobra.ExactArgs(4),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientTxContext(cmd)
			if err != nil {
				return err
			}
			sender := clientCtx.GetFromAddress().String()
			srcPort := args[0]
			srcChannel := args[1]
			receiver := args[2]

			coin, err := sdk.ParseCoinNormalized(args[3])
			if err != nil {
				return err
			}

			if !strings.HasPrefix(coin.Denom, "ibc/") {
				denomTrace := types.ParseDenomTrace(coin.Denom)
				coin.Denom = denomTrace.IBCDenom()
			}

			timeoutHeightStr, err := cmd.Flags().GetString(flagPacketTimeoutHeight)
			if err != nil {
				return err
			}
			timeoutHeight, err := clienttypes.ParseHeight(timeoutHeightStr)
			if err != nil {
				return err
			}

			timeoutTimestamp, err := cmd.Flags().GetUint64(flagPacketTimeoutTimestamp)
			if err != nil {
				return err
			}

			absoluteTimeouts, err := cmd.Flags().GetBool(flagAbsoluteTimeouts)
			if err != nil {
				return err
			}

			memo, err := cmd.Flags().GetString(flagMemo)
			if err != nil {
				return err
			}

			// if the timeouts are not absolute, retrieve latest block height and block timestamp
			// for the consensus state connected to the destination port/channel.
			// localhost clients must rely solely on local clock time in order to use relative timestamps.
			if !absoluteTimeouts {
				clientRes, err := channelutils.QueryChannelClientState(clientCtx, srcPort, srcChannel, false)
				if err != nil {
					return err
				}

				var clientState exported.ClientState
				if err := clientCtx.InterfaceRegistry.UnpackAny(clientRes.IdentifiedClientState.ClientState, &clientState); err != nil {
					return err
				}

				clientHeight, ok := clientState.GetLatestHeight().(clienttypes.Height)
				if !ok {
					return fmt.Errorf("invalid height type. expected type: %T, got: %T", clienttypes.Height{}, clientState.GetLatestHeight())
				}

				var consensusState exported.ConsensusState
				if clientState.ClientType() != exported.Localhost {
					consensusStateRes, err := clientutils.QueryConsensusState(clientCtx, clientRes.IdentifiedClientState.ClientId, clientHeight, false, true)
					if err != nil {
						return err
					}

					if err := clientCtx.InterfaceRegistry.UnpackAny(consensusStateRes.ConsensusState, &consensusState); err != nil {
						return err
					}
				}

				if !timeoutHeight.IsZero() {
					absoluteHeight := clientHeight
					absoluteHeight.RevisionNumber += timeoutHeight.RevisionNumber
					absoluteHeight.RevisionHeight += timeoutHeight.RevisionHeight
					timeoutHeight = absoluteHeight
				}

				// use local clock time as reference time if it is later than the
				// consensus state timestamp of the counterparty chain, otherwise
				// still use consensus state timestamp as reference.
				// for localhost clients local clock time is always used.
				if timeoutTimestamp != 0 {
					var consensusStateTimestamp uint64
					if consensusState != nil {
						consensusStateTimestamp = consensusState.GetTimestamp()
					}

					now := time.Now().UnixNano()
					if now > 0 {
						now := uint64(now)
						if now > consensusStateTimestamp {
							timeoutTimestamp = now + timeoutTimestamp
						} else {
							timeoutTimestamp = consensusStateTimestamp + timeoutTimestamp
						}
					} else {
						return errors.New("local clock time is not greater than Jan 1st, 1970 12:00 AM")
					}
				}
			}

			msg := types.NewMsgTransfer(
				srcPort, srcChannel, coin, sender, receiver, timeoutHeight, timeoutTimestamp, memo,
			)
			return tx.GenerateOrBroadcastTxCLI(clientCtx, cmd.Flags(), msg)
		},
	}

	cmd.Flags().String(flagPacketTimeoutHeight, types.DefaultRelativePacketTimeoutHeight, "Packet timeout block height. The timeout is disabled when set to 0-0.")
	cmd.Flags().Uint64(flagPacketTimeoutTimestamp, types.DefaultRelativePacketTimeoutTimestamp, "Packet timeout timestamp in nanoseconds from now. Default is 10 minutes. The timeout is disabled when set to 0.")
	cmd.Flags().Bool(flagAbsoluteTimeouts, false, "Timeout flags are used as absolute timeouts.")
	cmd.Flags().String(flagMemo, "", "Memo to be sent along with the packet.")
	flags.AddTxFlagsToCmd(cmd)

	return cmd
}

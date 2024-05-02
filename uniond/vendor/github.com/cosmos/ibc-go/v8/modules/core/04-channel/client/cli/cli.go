package cli

import (
	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"

	"github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
)

// GetQueryCmd returns the query commands for IBC channels
func GetQueryCmd() *cobra.Command {
	queryCmd := &cobra.Command{
		Use:                        types.SubModuleName,
		Short:                      "IBC channel query subcommands",
		DisableFlagParsing:         true,
		SuggestionsMinimumDistance: 2,
		RunE:                       client.ValidateCmd,
	}

	queryCmd.AddCommand(
		GetCmdQueryChannels(),
		GetCmdQueryChannel(),
		GetCmdQueryConnectionChannels(),
		GetCmdQueryChannelClientState(),
		GetCmdQueryPacketCommitment(),
		GetCmdQueryPacketCommitments(),
		GetCmdQueryPacketReceipt(),
		GetCmdQueryPacketAcknowledgement(),
		GetCmdQueryUnreceivedPackets(),
		GetCmdQueryUnreceivedAcks(),
		GetCmdQueryNextSequenceReceive(),
		GetCmdQueryNextSequenceSend(),
		GetCmdQueryUpgradeError(),
		GetCmdQueryUpgrade(),
		GetCmdChannelParams(),
	)

	return queryCmd
}

// NewTxCmd returns a CLI command handler for all x/ibc channel transaction commands.
func NewTxCmd() *cobra.Command {
	txCmd := &cobra.Command{
		Use:                        types.SubModuleName,
		Short:                      "IBC channel transaction subcommands",
		DisableFlagParsing:         true,
		SuggestionsMinimumDistance: 2,
		RunE:                       client.ValidateCmd,
	}

	txCmd.AddCommand(
		newUpgradeChannelsTxCmd(),
		newPruneAcknowledgementsTxCmd(),
	)

	return txCmd
}

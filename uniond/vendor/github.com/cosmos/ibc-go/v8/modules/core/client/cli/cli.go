package cli

import (
	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"

	ibcclient "github.com/cosmos/ibc-go/v8/modules/core/02-client"
	connection "github.com/cosmos/ibc-go/v8/modules/core/03-connection"
	channel "github.com/cosmos/ibc-go/v8/modules/core/04-channel"
	ibcexported "github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// GetTxCmd returns the transaction commands for this module
func GetTxCmd() *cobra.Command {
	ibcTxCmd := &cobra.Command{
		Use:                        ibcexported.ModuleName,
		Short:                      "IBC transaction subcommands",
		DisableFlagParsing:         true,
		SuggestionsMinimumDistance: 2,
		RunE:                       client.ValidateCmd,
	}

	ibcTxCmd.AddCommand(
		ibcclient.GetTxCmd(),
		channel.GetTxCmd(),
	)

	return ibcTxCmd
}

// GetQueryCmd returns the cli query commands for this module
func GetQueryCmd() *cobra.Command {
	// Group ibc queries under a subcommand
	ibcQueryCmd := &cobra.Command{
		Use:                        ibcexported.ModuleName,
		Short:                      "Querying commands for the IBC module",
		DisableFlagParsing:         true,
		SuggestionsMinimumDistance: 2,
		RunE:                       client.ValidateCmd,
	}

	ibcQueryCmd.AddCommand(
		ibcclient.GetQueryCmd(),
		connection.GetQueryCmd(),
		channel.GetQueryCmd(),
	)

	return ibcQueryCmd
}

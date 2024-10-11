package cli

import (
	"github.com/spf13/cobra"

	"github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
)

// GetQueryCmd returns the query commands for IBC connections
func GetQueryCmd() *cobra.Command {
	queryCmd := &cobra.Command{
		Use:                        types.SubModuleName,
		Short:                      "IBC connection query subcommands",
		DisableFlagParsing:         true,
		SuggestionsMinimumDistance: 2,
	}

	queryCmd.AddCommand(
		GetCmdQueryConnections(),
		GetCmdQueryConnection(),
		GetCmdQueryClientConnections(),
		GetCmdConnectionParams(),
	)

	return queryCmd
}

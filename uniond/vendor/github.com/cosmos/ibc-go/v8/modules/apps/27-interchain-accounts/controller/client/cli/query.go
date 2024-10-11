package cli

import (
	"fmt"

	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/version"

	"github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller/types"
)

// GetCmdQueryInterchainAccount returns the command handler for the controller submodule parameter querying.
func GetCmdQueryInterchainAccount() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "interchain-account [owner] [connection-id]",
		Short:   "Query the interchain account address for a given owner on a particular connection",
		Long:    "Query the controller submodule for the interchain account address for a given owner on a particular connection",
		Args:    cobra.ExactArgs(2),
		Example: fmt.Sprintf("%s query interchain-accounts controller interchain-account cosmos1layxcsmyye0dc0har9sdfzwckaz8sjwlfsj8zs connection-0", version.AppName),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			queryClient := types.NewQueryClient(clientCtx)
			req := &types.QueryInterchainAccountRequest{
				Owner:        args[0],
				ConnectionId: args[1],
			}

			res, err := queryClient.InterchainAccount(cmd.Context(), req)
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)

	return cmd
}

// GetCmdParams returns the command handler for the controller submodule parameter querying.
func GetCmdParams() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "params",
		Short:   "Query the current interchain-accounts controller submodule parameters",
		Long:    "Query the current interchain-accounts controller submodule parameters",
		Args:    cobra.NoArgs,
		Example: fmt.Sprintf("%s query interchain-accounts controller params", version.AppName),
		RunE: func(cmd *cobra.Command, _ []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}
			queryClient := types.NewQueryClient(clientCtx)

			res, err := queryClient.Params(cmd.Context(), &types.QueryParamsRequest{})
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res.Params)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)

	return cmd
}

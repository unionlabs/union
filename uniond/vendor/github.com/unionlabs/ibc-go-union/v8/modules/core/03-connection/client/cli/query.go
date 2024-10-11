package cli

import (
	"fmt"

	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/version"

	"github.com/cosmos/ibc-go/v8/modules/core/03-connection/client/utils"
	"github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	ibcexported "github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// GetCmdQueryConnections defines the command to query all the connection ends
// that this chain mantains.
func GetCmdQueryConnections() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "connections",
		Short:   "Query all connections",
		Long:    "Query all connections ends from a chain",
		Example: fmt.Sprintf("%s query %s %s connections", version.AppName, ibcexported.ModuleName, types.SubModuleName),
		Args:    cobra.NoArgs,
		RunE: func(cmd *cobra.Command, _ []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}
			queryClient := types.NewQueryClient(clientCtx)

			pageReq, err := client.ReadPageRequest(cmd.Flags())
			if err != nil {
				return err
			}

			req := &types.QueryConnectionsRequest{
				Pagination: pageReq,
			}

			res, err := queryClient.Connections(cmd.Context(), req)
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)
	flags.AddPaginationFlagsToCmd(cmd, "connection ends")

	return cmd
}

// GetCmdQueryConnection defines the command to query a connection end
func GetCmdQueryConnection() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "end [connection-id]",
		Short:   "Query stored connection end",
		Long:    "Query stored connection end",
		Example: fmt.Sprintf("%s query %s %s end [connection-id]", version.AppName, ibcexported.ModuleName, types.SubModuleName),
		Args:    cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}
			connectionID := args[0]
			prove, _ := cmd.Flags().GetBool(flags.FlagProve)

			connRes, err := utils.QueryConnection(clientCtx, connectionID, prove)
			if err != nil {
				return err
			}

			clientCtx = clientCtx.WithHeight(int64(connRes.ProofHeight.RevisionHeight))
			return clientCtx.PrintProto(connRes)
		},
	}

	cmd.Flags().Bool(flags.FlagProve, true, "show proofs for the query results")
	flags.AddQueryFlagsToCmd(cmd)

	return cmd
}

// GetCmdQueryClientConnections defines the command to query a client connections
func GetCmdQueryClientConnections() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "path [client-id]",
		Short:   "Query stored client connection paths",
		Long:    "Query stored client connection paths",
		Example: fmt.Sprintf("%s query  %s %s path [client-id]", version.AppName, ibcexported.ModuleName, types.SubModuleName),
		Args:    cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}
			clientID := args[0]
			prove, _ := cmd.Flags().GetBool(flags.FlagProve)

			connPathsRes, err := utils.QueryClientConnections(clientCtx, clientID, prove)
			if err != nil {
				return err
			}

			clientCtx = clientCtx.WithHeight(int64(connPathsRes.ProofHeight.RevisionHeight))
			return clientCtx.PrintProto(connPathsRes)
		},
	}

	cmd.Flags().Bool(flags.FlagProve, true, "show proofs for the query results")
	flags.AddQueryFlagsToCmd(cmd)

	return cmd
}

// GetCmdConnectionParams returns the command handler for ibc connection parameter querying.
func GetCmdConnectionParams() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "params",
		Short:   "Query the current ibc connection parameters",
		Long:    "Query the current ibc connection parameters",
		Args:    cobra.NoArgs,
		Example: fmt.Sprintf("%s query %s %s params", version.AppName, ibcexported.ModuleName, types.SubModuleName),
		RunE: func(cmd *cobra.Command, _ []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}
			queryClient := types.NewQueryClient(clientCtx)

			res, _ := queryClient.ConnectionParams(cmd.Context(), &types.QueryConnectionParamsRequest{})
			return clientCtx.PrintProto(res.Params)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)

	return cmd
}

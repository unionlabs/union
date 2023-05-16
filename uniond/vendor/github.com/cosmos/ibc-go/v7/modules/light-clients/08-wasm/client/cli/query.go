package cli

import (
	"context"
	"fmt"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/version"
	"github.com/cosmos/ibc-go/v7/modules/light-clients/08-wasm/types"
	"github.com/spf13/cobra"

	ibcexported "github.com/cosmos/ibc-go/v7/modules/core/exported"
)

// getCmdCode defines the command to query wasm code for given code ID.
func getCmdCode() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "code [code-id]",
		Short:   "Query wasm code",
		Long:    "Query wasm code for a light client wasm contract with a given code ID",
		Example: fmt.Sprintf("%s query %s wasm code [code-id]", version.AppName, ibcexported.ModuleName),
		Args:    cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}
			queryClient := types.NewQueryClient(clientCtx)

			codeID := args[0]
			req := types.QueryCodeRequest{
				CodeId: codeID,
			}

			res, err := queryClient.Code(context.Background(), &req)
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)

	return cmd
}

// getCmdCodeIDs defines the command to query all wasm code IDs.
func getCmdCodeIDs() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "code-ids",
		Short:   "Query all code IDs",
		Long:    "Query all code IDs for all deployed light client wasm contracts",
		Example: fmt.Sprintf("%s query %s wasm code-ids", version.AppName, ibcexported.ModuleName),
		Args:    cobra.ExactArgs(0),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}
			queryClient := types.NewQueryClient(clientCtx)

			pageReq, err := client.ReadPageRequest(cmd.Flags())
			if err != nil {
				return err
			}

			req := types.QueryCodeIdsRequest{
				Pagination: pageReq,
			}

			res, err := queryClient.CodeIds(context.Background(), &req)
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)
	flags.AddPaginationFlagsToCmd(cmd, "all wasm code")

	return cmd
}

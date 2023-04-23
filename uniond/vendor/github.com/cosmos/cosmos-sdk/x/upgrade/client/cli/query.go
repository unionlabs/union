package cli

import (
	"fmt"

	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/x/upgrade/types"
)

// GetQueryCmd returns the parent command for all x/upgrade CLI query commands.
func GetQueryCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   types.ModuleName,
		Short: "Querying commands for the upgrade module",
	}

	cmd.AddCommand(
		GetCurrentPlanCmd(),
		GetAppliedPlanCmd(),
		GetModuleVersionsCmd(),
	)

	return cmd
}

// GetCurrentPlanCmd returns the query upgrade plan command.
func GetCurrentPlanCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "plan",
		Short: "get upgrade plan (if one exists)",
		Long:  "Gets the currently scheduled upgrade plan, if one exists",
		Args:  cobra.ExactArgs(0),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}
			queryClient := types.NewQueryClient(clientCtx)

			params := types.QueryCurrentPlanRequest{}
			res, err := queryClient.CurrentPlan(cmd.Context(), &params)
			if err != nil {
				return err
			}

			if res.Plan == nil {
				return fmt.Errorf("no upgrade scheduled")
			}

			return clientCtx.PrintProto(res.GetPlan())
		},
	}

	flags.AddQueryFlagsToCmd(cmd)

	return cmd
}

// GetAppliedPlanCmd returns information about the block at which a completed
// upgrade was applied.
func GetAppliedPlanCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "applied [upgrade-name]",
		Short: "block header for height at which a completed upgrade was applied",
		Long: "If upgrade-name was previously executed on the chain, this returns the header for the block at which it was applied.\n" +
			"This helps a client determine which binary was valid over a given range of blocks, as well as more context to understand past migrations.",
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}
			queryClient := types.NewQueryClient(clientCtx)
			ctx := cmd.Context()
			params := types.QueryAppliedPlanRequest{Name: args[0]}
			res, err := queryClient.AppliedPlan(ctx, &params)
			if err != nil {
				return err
			}

			if res.Height == 0 {
				return fmt.Errorf("no upgrade found")
			}

			// we got the height, now let's return the headers
			node, err := clientCtx.GetNode()
			if err != nil {
				return err
			}
			headers, err := node.BlockchainInfo(ctx, res.Height, res.Height)
			if err != nil {
				return err
			}
			if len(headers.BlockMetas) == 0 {
				return fmt.Errorf("no headers returned for height %d", res.Height)
			}

			// always output json as Header is unreable in toml ([]byte is a long list of numbers)
			bz, err := clientCtx.LegacyAmino.MarshalJSONIndent(headers.BlockMetas[0], "", "  ")
			if err != nil {
				return err
			}
			return clientCtx.PrintString(fmt.Sprintf("%s\n", bz))
		},
	}

	flags.AddQueryFlagsToCmd(cmd)

	return cmd
}

// GetModuleVersionsCmd returns the module version list from state
func GetModuleVersionsCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "module_versions [optional module_name]",
		Short: "get the list of module versions",
		Long: "Gets a list of module names and their respective consensus versions.\n" +
			"Following the command with a specific module name will return only\n" +
			"that module's information.",
		Args: cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			queryClient := types.NewQueryClient(clientCtx)
			var params types.QueryModuleVersionsRequest

			if len(args) == 1 {
				params = types.QueryModuleVersionsRequest{ModuleName: args[0]}
			} else {
				params = types.QueryModuleVersionsRequest{}
			}

			res, err := queryClient.ModuleVersions(cmd.Context(), &params)
			if err != nil {
				return err
			}

			if res.ModuleVersions == nil {
				return errors.ErrNotFound
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)

	return cmd
}

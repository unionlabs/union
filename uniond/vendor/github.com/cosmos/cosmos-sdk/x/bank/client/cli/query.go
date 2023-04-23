package cli

import (
	"fmt"
	"strings"

	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/version"
	"github.com/cosmos/cosmos-sdk/x/bank/types"
)

const (
	FlagDenom = "denom"
)

// GetQueryCmd returns the parent command for all x/bank CLi query commands. The
// provided clientCtx should have, at a minimum, a verifier, Tendermint RPC client,
// and marshaler set.
func GetQueryCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:                        types.ModuleName,
		Short:                      "Querying commands for the bank module",
		DisableFlagParsing:         true,
		SuggestionsMinimumDistance: 2,
		RunE:                       client.ValidateCmd,
	}

	cmd.AddCommand(
		GetBalancesCmd(),
		GetSpendableBalancesCmd(),
		GetCmdQueryTotalSupply(),
		GetCmdDenomsMetadata(),
		GetCmdQuerySendEnabled(),
	)

	return cmd
}

func GetBalancesCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "balances [address]",
		Short: "Query for account balances by address",
		Long: strings.TrimSpace(
			fmt.Sprintf(`Query the total balance of an account or of a specific denomination.

Example:
  $ %s query %s balances [address]
  $ %s query %s balances [address] --denom=[denom]
`,
				version.AppName, types.ModuleName, version.AppName, types.ModuleName,
			),
		),
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			denom, err := cmd.Flags().GetString(FlagDenom)
			if err != nil {
				return err
			}

			queryClient := types.NewQueryClient(clientCtx)

			addr, err := sdk.AccAddressFromBech32(args[0])
			if err != nil {
				return err
			}

			pageReq, err := client.ReadPageRequest(cmd.Flags())
			if err != nil {
				return err
			}

			ctx := cmd.Context()

			if denom == "" {
				params := types.NewQueryAllBalancesRequest(addr, pageReq)

				res, err := queryClient.AllBalances(ctx, params)
				if err != nil {
					return err
				}

				return clientCtx.PrintProto(res)
			}

			params := types.NewQueryBalanceRequest(addr, denom)

			res, err := queryClient.Balance(ctx, params)
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res.Balance)
		},
	}

	cmd.Flags().String(FlagDenom, "", "The specific balance denomination to query for")
	flags.AddQueryFlagsToCmd(cmd)
	flags.AddPaginationFlagsToCmd(cmd, "all balances")

	return cmd
}

func GetSpendableBalancesCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "spendable-balances [address]",
		Short:   "Query for account spendable balances by address",
		Example: fmt.Sprintf("$ %s query %s spendable-balances [address]", version.AppName, types.ModuleName),
		Args:    cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			denom, err := cmd.Flags().GetString(FlagDenom)
			if err != nil {
				return err
			}

			queryClient := types.NewQueryClient(clientCtx)

			addr, err := sdk.AccAddressFromBech32(args[0])
			if err != nil {
				return err
			}

			pageReq, err := client.ReadPageRequest(cmd.Flags())
			if err != nil {
				return err
			}

			ctx := cmd.Context()

			if denom == "" {
				params := types.NewQuerySpendableBalancesRequest(addr, pageReq)

				res, err := queryClient.SpendableBalances(ctx, params)
				if err != nil {
					return err
				}

				return clientCtx.PrintProto(res)
			}

			params := types.NewQuerySpendableBalanceByDenomRequest(addr, denom)

			res, err := queryClient.SpendableBalanceByDenom(ctx, params)
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	cmd.Flags().String(FlagDenom, "", "The specific balance denomination to query for")
	flags.AddQueryFlagsToCmd(cmd)
	flags.AddPaginationFlagsToCmd(cmd, "spendable balances")

	return cmd
}

// GetCmdDenomsMetadata defines the cobra command to query client denomination metadata.
func GetCmdDenomsMetadata() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "denom-metadata",
		Short: "Query the client metadata for coin denominations",
		Long: strings.TrimSpace(
			fmt.Sprintf(`Query the client metadata for all the registered coin denominations

Example:
  To query for the client metadata of all coin denominations use:
  $ %s query %s denom-metadata

To query for the client metadata of a specific coin denomination use:
  $ %s query %s denom-metadata --denom=[denom]
`,
				version.AppName, types.ModuleName, version.AppName, types.ModuleName,
			),
		),
		RunE: func(cmd *cobra.Command, _ []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			denom, err := cmd.Flags().GetString(FlagDenom)
			if err != nil {
				return err
			}

			queryClient := types.NewQueryClient(clientCtx)

			if denom == "" {
				res, err := queryClient.DenomsMetadata(cmd.Context(), &types.QueryDenomsMetadataRequest{})
				if err != nil {
					return err
				}

				return clientCtx.PrintProto(res)
			}

			res, err := queryClient.DenomMetadata(cmd.Context(), &types.QueryDenomMetadataRequest{Denom: denom})
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	cmd.Flags().String(FlagDenom, "", "The specific denomination to query client metadata for")
	flags.AddQueryFlagsToCmd(cmd)

	return cmd
}

func GetCmdQueryTotalSupply() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "total",
		Short: "Query the total supply of coins of the chain",
		Args:  cobra.NoArgs,
		Long: strings.TrimSpace(
			fmt.Sprintf(`Query total supply of coins that are held by accounts in the chain.

Example:
  $ %s query %s total

To query for the total supply of a specific coin denomination use:
  $ %s query %s total --denom=[denom]
`,
				version.AppName, types.ModuleName, version.AppName, types.ModuleName,
			),
		),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			denom, err := cmd.Flags().GetString(FlagDenom)
			if err != nil {
				return err
			}

			queryClient := types.NewQueryClient(clientCtx)
			ctx := cmd.Context()

			pageReq, err := client.ReadPageRequest(cmd.Flags())
			if err != nil {
				return err
			}

			if denom == "" {
				res, err := queryClient.TotalSupply(ctx, &types.QueryTotalSupplyRequest{Pagination: pageReq})
				if err != nil {
					return err
				}

				return clientCtx.PrintProto(res)
			}

			res, err := queryClient.SupplyOf(ctx, &types.QuerySupplyOfRequest{Denom: denom})
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(&res.Amount)
		},
	}

	cmd.Flags().String(FlagDenom, "", "The specific balance denomination to query for")
	flags.AddQueryFlagsToCmd(cmd)
	flags.AddPaginationFlagsToCmd(cmd, "all supply totals")

	return cmd
}

func GetCmdQuerySendEnabled() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "send-enabled [denom1 ...]",
		Short: "Query for send enabled entries",
		Long: strings.TrimSpace(`Query for send enabled entries that have been specifically set.

To look up one or more specific denoms, supply them as arguments to this command.
To look up all denoms, do not provide any arguments.
`,
		),
		Example: strings.TrimSpace(
			fmt.Sprintf(`Getting one specific entry:
  $ %[1]s query %[2]s send-enabled foocoin

Getting two specific entries:
  $ %[1]s query %[2]s send-enabled foocoin barcoin

Getting all entries:
  $ %[1]s query %[2]s send-enabled
`,
				version.AppName, types.ModuleName,
			),
		),
		RunE: func(cmd *cobra.Command, args []string) error {
			reqPag, err := client.ReadPageRequest(client.MustFlagSetWithPageKeyDecoded(cmd.Flags()))
			if err != nil {
				return err
			}

			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			queryClient := types.NewQueryClient(clientCtx)
			req := &types.QuerySendEnabledRequest{
				Denoms:     args,
				Pagination: reqPag,
			}

			res, err := queryClient.SendEnabled(cmd.Context(), req)
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)
	flags.AddPaginationFlagsToCmd(cmd, "send enabled entries")

	return cmd
}

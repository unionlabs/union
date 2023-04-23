package cli

import (
	"fmt"
	"strings"

	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/version"
	"github.com/cosmos/cosmos-sdk/x/feegrant"
)

// GetQueryCmd returns the cli query commands for this module
func GetQueryCmd() *cobra.Command {
	feegrantQueryCmd := &cobra.Command{
		Use:                        feegrant.ModuleName,
		Short:                      "Querying commands for the feegrant module",
		DisableFlagParsing:         true,
		SuggestionsMinimumDistance: 2,
		RunE:                       client.ValidateCmd,
	}

	feegrantQueryCmd.AddCommand(
		GetCmdQueryFeeGrant(),
		GetCmdQueryFeeGrantsByGrantee(),
		GetCmdQueryFeeGrantsByGranter(),
	)

	return feegrantQueryCmd
}

// GetCmdQueryFeeGrant returns cmd to query for a grant between granter and grantee.
func GetCmdQueryFeeGrant() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "grant [granter] [grantee]",
		Args:  cobra.ExactArgs(2),
		Short: "Query details of a single grant",
		Long: strings.TrimSpace(
			fmt.Sprintf(`Query details for a grant. 
You can find the fee-grant of a granter and grantee.

Example:
$ %s query feegrant grant [granter] [grantee]
`, version.AppName),
		),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx := client.GetClientContextFromCmd(cmd)
			queryClient := feegrant.NewQueryClient(clientCtx)

			granterAddr, err := sdk.AccAddressFromBech32(args[0])
			if err != nil {
				return err
			}

			granteeAddr, err := sdk.AccAddressFromBech32(args[1])
			if err != nil {
				return err
			}

			res, err := queryClient.Allowance(
				cmd.Context(),
				&feegrant.QueryAllowanceRequest{
					Granter: granterAddr.String(),
					Grantee: granteeAddr.String(),
				},
			)
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res.Allowance)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)

	return cmd
}

// GetCmdQueryFeeGrantsByGrantee returns cmd to query for all grants for a grantee.
func GetCmdQueryFeeGrantsByGrantee() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "grants-by-grantee [grantee]",
		Args:  cobra.ExactArgs(1),
		Short: "Query all grants of a grantee",
		Long: strings.TrimSpace(
			fmt.Sprintf(`Queries all the grants for a grantee address.

Example:
$ %s query feegrant grants-by-grantee [grantee]
`, version.AppName),
		),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx := client.GetClientContextFromCmd(cmd)
			queryClient := feegrant.NewQueryClient(clientCtx)

			granteeAddr, err := sdk.AccAddressFromBech32(args[0])
			if err != nil {
				return err
			}

			pageReq, err := client.ReadPageRequest(cmd.Flags())
			if err != nil {
				return err
			}

			res, err := queryClient.Allowances(
				cmd.Context(),
				&feegrant.QueryAllowancesRequest{
					Grantee:    granteeAddr.String(),
					Pagination: pageReq,
				},
			)
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)
	flags.AddPaginationFlagsToCmd(cmd, "grants")

	return cmd
}

// GetCmdQueryFeeGrantsByGranter returns cmd to query for all grants by a granter.
func GetCmdQueryFeeGrantsByGranter() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "grants-by-granter [granter]",
		Args:  cobra.ExactArgs(1),
		Short: "Query all grants by a granter",
		Long: strings.TrimSpace(
			fmt.Sprintf(`Queries all the grants issued for a granter address.

Example:
$ %s query feegrant grants-by-granter [granter]
`, version.AppName),
		),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx := client.GetClientContextFromCmd(cmd)
			queryClient := feegrant.NewQueryClient(clientCtx)

			granterAddr, err := sdk.AccAddressFromBech32(args[0])
			if err != nil {
				return err
			}

			pageReq, err := client.ReadPageRequest(cmd.Flags())
			if err != nil {
				return err
			}

			res, err := queryClient.AllowancesByGranter(
				cmd.Context(),
				&feegrant.QueryAllowancesByGranterRequest{
					Granter:    granterAddr.String(),
					Pagination: pageReq,
				},
			)
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)
	flags.AddPaginationFlagsToCmd(cmd, "grants")

	return cmd
}

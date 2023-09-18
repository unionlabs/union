package cli

import (
	"fmt"
	"strings"

	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/version"
	"github.com/cosmos/cosmos-sdk/x/nft"
)

// Flag names and values
const (
	FlagOwner   = "owner"
	FlagClassID = "class-id"
)

// GetQueryCmd returns the cli query commands for this module
func GetQueryCmd() *cobra.Command {
	nftQueryCmd := &cobra.Command{
		Use:                        nft.ModuleName,
		Short:                      "Querying commands for the nft module",
		Long:                       "",
		DisableFlagParsing:         true,
		SuggestionsMinimumDistance: 2,
		RunE:                       client.ValidateCmd,
	}

	nftQueryCmd.AddCommand(
		GetCmdQueryClass(),
		GetCmdQueryClasses(),
		GetCmdQueryNFT(),
		GetCmdQueryNFTs(),
		GetCmdQueryOwner(),
		GetCmdQueryBalance(),
		GetCmdQuerySupply(),
	)
	return nftQueryCmd
}

// GetCmdQueryClass implements the query class command.
func GetCmdQueryClass() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "class [class-id]",
		Args:    cobra.ExactArgs(1),
		Short:   "query an NFT class based on its id",
		Example: fmt.Sprintf(`$ %s query %s class <class-id>`, version.AppName, nft.ModuleName),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}
			queryClient := nft.NewQueryClient(clientCtx)
			res, err := queryClient.Class(cmd.Context(), &nft.QueryClassRequest{
				ClassId: args[0],
			})
			if err != nil {
				return err
			}
			return clientCtx.PrintProto(res)
		},
	}
	flags.AddQueryFlagsToCmd(cmd)
	return cmd
}

// GetCmdQueryClasses implements the query classes command.
func GetCmdQueryClasses() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "classes",
		Short:   "query all NFT classes",
		Example: fmt.Sprintf(`$ %s query %s classes`, version.AppName, nft.ModuleName),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}
			queryClient := nft.NewQueryClient(clientCtx)
			pageReq, err := client.ReadPageRequest(cmd.Flags())
			if err != nil {
				return err
			}
			res, err := queryClient.Classes(cmd.Context(), &nft.QueryClassesRequest{
				Pagination: pageReq,
			})
			if err != nil {
				return err
			}
			return clientCtx.PrintProto(res)
		},
	}
	flags.AddQueryFlagsToCmd(cmd)
	flags.AddPaginationFlagsToCmd(cmd, "classes")
	return cmd
}

// GetCmdQueryNFT implements the query nft command.
func GetCmdQueryNFT() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "nft [class-id] [nft-id]",
		Args:    cobra.ExactArgs(2),
		Short:   "query an NFT based on its class and id.",
		Example: fmt.Sprintf(`$ %s query %s nft`, version.AppName, nft.ModuleName),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}
			queryClient := nft.NewQueryClient(clientCtx)
			res, err := queryClient.NFT(cmd.Context(), &nft.QueryNFTRequest{
				ClassId: args[0],
				Id:      args[1],
			})
			if err != nil {
				return err
			}
			return clientCtx.PrintProto(res)
		},
	}
	flags.AddQueryFlagsToCmd(cmd)
	return cmd
}

// GetCmdQueryNFTs implements the query nft command.
func GetCmdQueryNFTs() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "nfts",
		Short: "query all NFTs of a given class or owner address.",
		Long: strings.TrimSpace(
			fmt.Sprintf(`Query all NFTs of a given class or owner address. If owner
is set, all nfts that belong to the owner are filtered out.
Examples:
$ %s query %s nfts <class-id> --owner=<owner>
`,
				version.AppName, nft.ModuleName),
		),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}
			queryClient := nft.NewQueryClient(clientCtx)
			pageReq, err := client.ReadPageRequest(cmd.Flags())
			if err != nil {
				return err
			}

			owner, err := cmd.Flags().GetString(FlagOwner)
			if err != nil {
				return err
			}

			if len(owner) > 0 {
				if _, err := sdk.AccAddressFromBech32(owner); err != nil {
					return err
				}
			}

			classID, err := cmd.Flags().GetString(FlagClassID)
			if err != nil {
				return err
			}

			if len(owner) == 0 && len(classID) == 0 {
				return errors.ErrInvalidRequest.Wrap("must provide at least one of classID or owner")
			}

			request := &nft.QueryNFTsRequest{
				ClassId:    classID,
				Owner:      owner,
				Pagination: pageReq,
			}
			res, err := queryClient.NFTs(cmd.Context(), request)
			if err != nil {
				return err
			}
			return clientCtx.PrintProto(res)
		},
	}
	flags.AddQueryFlagsToCmd(cmd)
	flags.AddPaginationFlagsToCmd(cmd, "nfts")
	cmd.Flags().String(FlagOwner, "", "The owner of the nft")
	cmd.Flags().String(FlagClassID, "", "The class-id of the nft")
	return cmd
}

// GetCmdQueryOwner implements the query owner command.
func GetCmdQueryOwner() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "owner [class-id] [nft-id]",
		Args:    cobra.ExactArgs(2),
		Short:   "query the owner of the NFT based on its class and id.",
		Example: fmt.Sprintf(`$ %s query %s owner <class-id> <nft-id>`, version.AppName, nft.ModuleName),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}
			queryClient := nft.NewQueryClient(clientCtx)
			res, err := queryClient.Owner(cmd.Context(), &nft.QueryOwnerRequest{
				ClassId: args[0],
				Id:      args[1],
			})
			if err != nil {
				return err
			}
			return clientCtx.PrintProto(res)
		},
	}
	flags.AddQueryFlagsToCmd(cmd)
	return cmd
}

// GetCmdQueryBalance implements the query balance command.
func GetCmdQueryBalance() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "balance [owner] [class-id]",
		Args:    cobra.ExactArgs(2),
		Short:   "query the number of NFTs of a given class owned by the owner.",
		Example: fmt.Sprintf(`$ %s query %s balance <owner> <class-id>`, version.AppName, nft.ModuleName),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}
			queryClient := nft.NewQueryClient(clientCtx)
			res, err := queryClient.Balance(cmd.Context(), &nft.QueryBalanceRequest{
				ClassId: args[1],
				Owner:   args[0],
			})
			if err != nil {
				return err
			}
			return clientCtx.PrintProto(res)
		},
	}
	flags.AddQueryFlagsToCmd(cmd)
	return cmd
}

// GetCmdQuerySupply implements the query supply command.
func GetCmdQuerySupply() *cobra.Command {
	cmd := &cobra.Command{
		Use:     "supply [class-id]",
		Args:    cobra.ExactArgs(1),
		Short:   "query the number of nft based on the class.",
		Example: fmt.Sprintf(`$ %s query %s supply <class-id>`, version.AppName, nft.ModuleName),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}
			queryClient := nft.NewQueryClient(clientCtx)
			res, err := queryClient.Supply(cmd.Context(), &nft.QuerySupplyRequest{
				ClassId: args[0],
			})
			if err != nil {
				return err
			}
			return clientCtx.PrintProto(res)
		},
	}
	flags.AddQueryFlagsToCmd(cmd)
	return cmd
}

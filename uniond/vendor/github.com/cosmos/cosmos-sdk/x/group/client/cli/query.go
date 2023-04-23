package cli

import (
	"strconv"

	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/x/group"
)

// QueryCmd returns the cli query commands for the group module.
func QueryCmd(name string) *cobra.Command {
	queryCmd := &cobra.Command{
		Use:                        name,
		Short:                      "Querying commands for the group module",
		DisableFlagParsing:         true,
		SuggestionsMinimumDistance: 2,
		RunE:                       client.ValidateCmd,
	}

	queryCmd.AddCommand(
		QueryGroupInfoCmd(),
		QueryGroupPolicyInfoCmd(),
		QueryGroupMembersCmd(),
		QueryGroupsByAdminCmd(),
		QueryGroupPoliciesByGroupCmd(),
		QueryGroupPoliciesByAdminCmd(),
		QueryProposalCmd(),
		QueryProposalsByGroupPolicyCmd(),
		QueryVoteByProposalVoterCmd(),
		QueryVotesByProposalCmd(),
		QueryVotesByVoterCmd(),
		QueryGroupsByMemberCmd(),
		QueryTallyResultCmd(),
		QueryGroupsCmd(),
	)

	return queryCmd
}

// QueryGroupsByMemberCmd creates a CLI command for Query/GroupsByMember.
func QueryGroupsByMemberCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "groups-by-member [address]",
		Short: "Query for groups by member address with pagination flags",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			pageReq, err := client.ReadPageRequest(cmd.Flags())
			if err != nil {
				return err
			}

			queryClient := group.NewQueryClient(clientCtx)
			res, err := queryClient.GroupsByMember(cmd.Context(), &group.QueryGroupsByMemberRequest{
				Address:    args[0],
				Pagination: pageReq,
			})
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)
	flags.AddPaginationFlagsToCmd(cmd, "groups-by-members")
	return cmd
}

// QueryGroupInfoCmd creates a CLI command for Query/GroupInfo.
func QueryGroupInfoCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "group-info [id]",
		Short: "Query for group info by group id",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			groupID, err := strconv.ParseUint(args[0], 10, 64)
			if err != nil {
				return err
			}

			queryClient := group.NewQueryClient(clientCtx)

			res, err := queryClient.GroupInfo(cmd.Context(), &group.QueryGroupInfoRequest{
				GroupId: groupID,
			})
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res.Info)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)

	return cmd
}

// QueryGroupPolicyInfoCmd creates a CLI command for Query/GroupPolicyInfo.
func QueryGroupPolicyInfoCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "group-policy-info [group-policy-account]",
		Short: "Query for group policy info by account address of group policy",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			queryClient := group.NewQueryClient(clientCtx)

			res, err := queryClient.GroupPolicyInfo(cmd.Context(), &group.QueryGroupPolicyInfoRequest{
				Address: args[0],
			})
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res.Info)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)

	return cmd
}

// QueryGroupMembersCmd creates a CLI command for Query/GroupMembers.
func QueryGroupMembersCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "group-members [id]",
		Short: "Query for group members by group id with pagination flags",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			groupID, err := strconv.ParseUint(args[0], 10, 64)
			if err != nil {
				return err
			}

			pageReq, err := client.ReadPageRequest(cmd.Flags())
			if err != nil {
				return err
			}

			queryClient := group.NewQueryClient(clientCtx)

			res, err := queryClient.GroupMembers(cmd.Context(), &group.QueryGroupMembersRequest{
				GroupId:    groupID,
				Pagination: pageReq,
			})
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)
	flags.AddPaginationFlagsToCmd(cmd, "group-members")

	return cmd
}

// QueryGroupsByAdminCmd creates a CLI command for Query/GroupsByAdmin.
func QueryGroupsByAdminCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "groups-by-admin [admin]",
		Short: "Query for groups by admin account address with pagination flags",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			pageReq, err := client.ReadPageRequest(cmd.Flags())
			if err != nil {
				return err
			}

			queryClient := group.NewQueryClient(clientCtx)

			res, err := queryClient.GroupsByAdmin(cmd.Context(), &group.QueryGroupsByAdminRequest{
				Admin:      args[0],
				Pagination: pageReq,
			})
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)
	flags.AddPaginationFlagsToCmd(cmd, "groups-by-admin")

	return cmd
}

// QueryGroupPoliciesByGroupCmd creates a CLI command for Query/GroupPoliciesByGroup.
func QueryGroupPoliciesByGroupCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "group-policies-by-group [group-id]",
		Short: "Query for group policies by group id with pagination flags",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			groupID, err := strconv.ParseUint(args[0], 10, 64)
			if err != nil {
				return err
			}

			pageReq, err := client.ReadPageRequest(cmd.Flags())
			if err != nil {
				return err
			}

			queryClient := group.NewQueryClient(clientCtx)

			res, err := queryClient.GroupPoliciesByGroup(cmd.Context(), &group.QueryGroupPoliciesByGroupRequest{
				GroupId:    groupID,
				Pagination: pageReq,
			})
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)
	flags.AddPaginationFlagsToCmd(cmd, "groups-policies-by-group")

	return cmd
}

// QueryGroupPoliciesByAdminCmd creates a CLI command for Query/GroupPoliciesByAdmin.
func QueryGroupPoliciesByAdminCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "group-policies-by-admin [admin]",
		Short: "Query for group policies by admin account address with pagination flags",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			pageReq, err := client.ReadPageRequest(cmd.Flags())
			if err != nil {
				return err
			}

			queryClient := group.NewQueryClient(clientCtx)

			res, err := queryClient.GroupPoliciesByAdmin(cmd.Context(), &group.QueryGroupPoliciesByAdminRequest{
				Admin:      args[0],
				Pagination: pageReq,
			})
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)
	flags.AddPaginationFlagsToCmd(cmd, "group-policies-by-admin")

	return cmd
}

// QueryProposalCmd creates a CLI command for Query/Proposal.
func QueryProposalCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "proposal [id]",
		Short: "Query for proposal by id",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			proposalID, err := strconv.ParseUint(args[0], 10, 64)
			if err != nil {
				return err
			}

			queryClient := group.NewQueryClient(clientCtx)

			res, err := queryClient.Proposal(cmd.Context(), &group.QueryProposalRequest{
				ProposalId: proposalID,
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

// QueryProposalsByGroupPolicyCmd creates a CLI command for Query/ProposalsByGroupPolicy.
func QueryProposalsByGroupPolicyCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "proposals-by-group-policy [group-policy-account]",
		Short: "Query for proposals by account address of group policy with pagination flags",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			pageReq, err := client.ReadPageRequest(cmd.Flags())
			if err != nil {
				return err
			}

			queryClient := group.NewQueryClient(clientCtx)

			res, err := queryClient.ProposalsByGroupPolicy(cmd.Context(), &group.QueryProposalsByGroupPolicyRequest{
				Address:    args[0],
				Pagination: pageReq,
			})
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)
	flags.AddPaginationFlagsToCmd(cmd, "proposals-by-group-policy")

	return cmd
}

// QueryVoteByProposalVoterCmd creates a CLI command for Query/VoteByProposalVoter.
func QueryVoteByProposalVoterCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "vote [proposal-id] [voter]",
		Short: "Query for vote by proposal id and voter account address",
		Args:  cobra.ExactArgs(2),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			proposalID, err := strconv.ParseUint(args[0], 10, 64)
			if err != nil {
				return err
			}

			queryClient := group.NewQueryClient(clientCtx)

			res, err := queryClient.VoteByProposalVoter(cmd.Context(), &group.QueryVoteByProposalVoterRequest{
				ProposalId: proposalID,
				Voter:      args[1],
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

// QueryVotesByProposalCmd creates a CLI command for Query/VotesByProposal.
func QueryVotesByProposalCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "votes-by-proposal [proposal-id]",
		Short: "Query for votes by proposal id with pagination flags",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			proposalID, err := strconv.ParseUint(args[0], 10, 64)
			if err != nil {
				return err
			}

			pageReq, err := client.ReadPageRequest(cmd.Flags())
			if err != nil {
				return err
			}

			queryClient := group.NewQueryClient(clientCtx)

			res, err := queryClient.VotesByProposal(cmd.Context(), &group.QueryVotesByProposalRequest{
				ProposalId: proposalID,
				Pagination: pageReq,
			})
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)
	flags.AddPaginationFlagsToCmd(cmd, "votes-by-proposal")

	return cmd
}

// QueryTallyResultCmd creates a CLI command for Query/TallyResult.
func QueryTallyResultCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "tally-result [proposal-id]",
		Short: "Query tally result of proposal",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			proposalID, err := strconv.ParseUint(args[0], 10, 64)
			if err != nil {
				return err
			}

			queryClient := group.NewQueryClient(clientCtx)

			res, err := queryClient.TallyResult(cmd.Context(), &group.QueryTallyResultRequest{
				ProposalId: proposalID,
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

// QueryVotesByVoterCmd creates a CLI command for Query/VotesByVoter.
func QueryVotesByVoterCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "votes-by-voter [voter]",
		Short: "Query for votes by voter account address with pagination flags",
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			pageReq, err := client.ReadPageRequest(cmd.Flags())
			if err != nil {
				return err
			}

			queryClient := group.NewQueryClient(clientCtx)

			res, err := queryClient.VotesByVoter(cmd.Context(), &group.QueryVotesByVoterRequest{
				Voter:      args[0],
				Pagination: pageReq,
			})
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)
	flags.AddPaginationFlagsToCmd(cmd, "votes-by-voter")

	return cmd
}

func QueryGroupsCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "groups",
		Short: "Query for groups present in the state",
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			pageReq, err := client.ReadPageRequest(cmd.Flags())
			if err != nil {
				return err
			}

			queryClient := group.NewQueryClient(clientCtx)

			res, err := queryClient.Groups(cmd.Context(), &group.QueryGroupsRequest{
				Pagination: pageReq,
			})
			if err != nil {
				return err
			}

			return clientCtx.PrintProto(res)
		},
	}

	flags.AddQueryFlagsToCmd(cmd)
	flags.AddPaginationFlagsToCmd(cmd, "groups")

	return cmd
}

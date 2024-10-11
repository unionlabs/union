package module

import (
	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	groupv1 "cosmossdk.io/api/cosmos/group/v1"
)

// AutoCLIOptions implements the autocli.HasAutoCLIConfig interface.
func (am AppModule) AutoCLIOptions() *autocliv1.ModuleOptions {
	return &autocliv1.ModuleOptions{
		Query: &autocliv1.ServiceCommandDescriptor{
			Service: groupv1.Query_ServiceDesc.ServiceName,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod: "GroupInfo",
					Use:       "group-info <group-id>",
					Short:     "Query for group info by group id",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "group_id"},
					},
				},
				{
					RpcMethod: "GroupPolicyInfo",
					Use:       "group-policy-info <group-policy-account>",
					Short:     "Query for group policy info by account address of group policy",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "address"},
					},
				},
				{
					RpcMethod: "GroupMembers",
					Use:       "group-members <group-id>",
					Short:     "Query for group members by group id",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "group_id"},
					},
				},
				{
					RpcMethod: "GroupsByAdmin",
					Use:       "groups-by-admin <admin>",
					Short:     "Query for groups by admin account address",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "admin"},
					},
				},
				{
					RpcMethod: "GroupPoliciesByGroup",
					Use:       "group-policies-by-group <group-id>",
					Short:     "Query for group policies by group id",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "group_id"},
					},
				},
				{
					RpcMethod: "GroupPoliciesByAdmin",
					Use:       "group-policies-by-admin <admin>",
					Short:     "Query for group policies by admin account address",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "admin"},
					},
				},
				{
					RpcMethod: "Proposal",
					Use:       "proposal <proposal-id>",
					Short:     "Query for proposal by id",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "proposal_id"},
					},
				},
				{
					RpcMethod: "ProposalsByGroupPolicy",
					Use:       "proposals-by-group-policy <group-policy-account>",
					Short:     "Query for proposals by account address of group policy",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "address"},
					},
				},
				{
					RpcMethod: "VoteByProposalVoter",
					Use:       "vote <proposal-id> <voter>",
					Short:     "Query for vote by proposal id and voter account address",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "proposal_id"},
						{ProtoField: "voter"},
					},
				},
				{
					RpcMethod: "VotesByProposal",
					Use:       "votes-by-proposal <proposal-id>",
					Short:     "Query for votes by proposal id",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "proposal_id"},
					},
				},
				{
					RpcMethod: "VotesByVoter",
					Use:       "votes-by-voter <voter>",
					Short:     "Query for votes by voter account address",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "voter"},
					},
				},
				{
					RpcMethod: "GroupsByMember",
					Use:       "groups-by-member <address>",
					Short:     "Query for groups by member address",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "address"},
					},
				},
				{
					RpcMethod: "TallyResult",
					Use:       "tally-result <proposal-id>",
					Short:     "Query tally result of proposal",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "proposal_id"},
					},
				},
				{
					RpcMethod: "Groups",
					Use:       "groups",
					Short:     "Query for all groups on chain",
				},
			},
		},
		Tx: &autocliv1.ServiceCommandDescriptor{
			Service:              groupv1.Msg_ServiceDesc.ServiceName,
			EnhanceCustomCommand: true,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod: "UpdateGroupAdmin",
					Use:       "update-group-admin <admin> <group-id> <new-admin>",
					Short:     "Update a group's admin",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "admin"}, {ProtoField: "group_id"}, {ProtoField: "new_admin"},
					},
				},
				{
					RpcMethod: "UpdateGroupMetadata",
					Use:       "update-group-metadata <admin> <group-id> <metadata>",
					Short:     "Update a group's metadata",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "admin"}, {ProtoField: "group_id"}, {ProtoField: "metadata"},
					},
				},
				{
					RpcMethod: "UpdateGroupPolicyAdmin",
					Use:       "update-group-policy-admin <admin> <group-policy-account> <new-admin>",
					Short:     "Update a group policy admin",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "admin"}, {ProtoField: "group_policy_address"}, {ProtoField: "new_admin"},
					},
				},
				{
					RpcMethod: "UpdateGroupPolicyMetadata",
					Use:       "update-group-policy-metadata <admin> <group-policy-account> <new-metadata>",
					Short:     "Update a group policy metadata",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "admin"}, {ProtoField: "group_policy_address"}, {ProtoField: "metadata"},
					},
				},
				{
					RpcMethod: "WithdrawProposal",
					Use:       "withdraw-proposal <proposal-id> <group-policy-admin-or-proposer>",
					Short:     "Withdraw a submitted proposal",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "proposal_id"}, {ProtoField: "address"},
					},
				},
				{
					RpcMethod: "Vote",
					Use:       "vote <proposal-id> <voter> <vote-option> <metadata>",
					Long: `Vote on a proposal.
Parameters:
	proposal-id: unique ID of the proposal
	voter: voter account addresses.
	vote-option: choice of the voter(s)
		VOTE_OPTION_UNSPECIFIED: no-op
		VOTE_OPTION_NO: no
		VOTE_OPTION_YES: yes
		VOTE_OPTION_ABSTAIN: abstain
		VOTE_OPTION_NO_WITH_VETO: no-with-veto
	Metadata: metadata for the vote
`,
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "proposal_id"}, {ProtoField: "voter"}, {ProtoField: "option"}, {ProtoField: "metadata"},
					},
					FlagOptions: map[string]*autocliv1.FlagOptions{
						"exec": {Name: "exec", DefaultValue: "", Usage: "Set to 'try' for trying to execute proposal immediately after voting"},
					},
				},
				{
					RpcMethod: "Exec",
					Use:       "exec <proposal-id>",
					Short:     "Execute a proposal",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "proposal_id"},
					},
				},
				{
					RpcMethod: "LeaveGroup",
					Use:       "leave-group <member-address> <group-id>",
					Short:     "Remove member from the group",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "address"}, {ProtoField: "group_id"},
					},
				},
			},
		},
	}
}

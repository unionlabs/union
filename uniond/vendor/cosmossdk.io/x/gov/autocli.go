package gov

import (
	"fmt"

	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	govv1 "cosmossdk.io/api/cosmos/gov/v1"

	"github.com/cosmos/cosmos-sdk/version"
)

// AutoCLIOptions implements the autocli.HasAutoCLIConfig interface.
func (am AppModule) AutoCLIOptions() *autocliv1.ModuleOptions {
	return &autocliv1.ModuleOptions{
		Query: &autocliv1.ServiceCommandDescriptor{
			Service: govv1.Query_ServiceDesc.ServiceName,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod: "Params",
					Use:       "params",
					Short:     "Query the parameters of the governance module",
				},
				{
					RpcMethod: "MessageBasedParams",
					Use:       "params-by-msg-url <msg-url>",
					Short:     "Query the governance parameters of specific message",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "msg_url"},
					},
				},
				{
					RpcMethod: "Proposals",
					Use:       "proposals",
					Short:     "Query proposals with optional filters",
					Example:   fmt.Sprintf("%[1]s query gov proposals --depositor cosmos1...\n%[1]s query gov proposals --voter cosmos1...\n%[1]s query gov proposals --proposal-status (unspecified | deposit-period | voting-period | passed | rejected | failed)", version.AppName),
				},
				{
					RpcMethod: "Proposal",
					Use:       "proposal <proposal-id>",
					Alias:     []string{"proposer"},
					Short:     "Query details of a single proposal",
					Example:   fmt.Sprintf("%s query gov proposal 1", version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "proposal_id"},
					},
				},
				{
					RpcMethod: "Vote",
					Use:       "vote <proposal-id> <voter-addr>",
					Short:     "Query details of a single vote",
					Example:   fmt.Sprintf("%s query gov vote 1 cosmos1...", version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "proposal_id"},
						{ProtoField: "voter"},
					},
				},
				{
					RpcMethod: "Votes",
					Use:       "votes <proposal-id>",
					Short:     "Query votes of a single proposal",
					Example:   fmt.Sprintf("%s query gov votes 1", version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "proposal_id"},
					},
				},
				{
					RpcMethod: "Deposit",
					Use:       "deposit <proposal-id> <depositor-addr>",
					Short:     "Query details of a deposit",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "proposal_id"},
						{ProtoField: "depositor"},
					},
				},
				{
					RpcMethod: "Deposits",
					Use:       "deposits <proposal-id>",
					Short:     "Query deposits on a proposal",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "proposal_id"},
					},
				},
				{
					RpcMethod: "TallyResult",
					Use:       "tally <proposal-id>",
					Short:     "Query the tally of a proposal vote",
					Example:   fmt.Sprintf("%s query gov tally 1", version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "proposal_id"},
					},
				},
				{
					RpcMethod: "Constitution",
					Use:       "constitution",
					Short:     "Query the current chain constitution",
				},
			},
			EnhanceCustomCommand: true, // We still have manual commands in gov that we want to keep
		},
		Tx: &autocliv1.ServiceCommandDescriptor{
			Service: govv1.Msg_ServiceDesc.ServiceName,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod: "Deposit",
					Use:       "deposit <proposal-id> <deposit>",
					Short:     "Deposit tokens for an active proposal",
					Long:      fmt.Sprintf(`Submit a deposit for an active proposal. You can find the proposal-id by running "%s query gov proposals"`, version.AppName),
					Example:   fmt.Sprintf(`$ %s tx gov deposit 1 10stake --from mykey`, version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "proposal_id"},
						{ProtoField: "amount", Varargs: true},
					},
				},
				{
					RpcMethod: "CancelProposal",
					Use:       "cancel-proposal <proposal-id>",
					Short:     "Cancel governance proposal before the voting period ends. Must be signed by the proposal creator.",
					Example:   fmt.Sprintf(`$ %s tx gov cancel-proposal 1 --from mykey`, version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "proposal_id"},
					},
				},
				{
					RpcMethod: "Vote",
					Use:       "vote <proposal-id> <option>",
					Short:     "Vote for an active proposal, options: yes/no/no-with-veto/abstain",
					Long:      fmt.Sprintf(`Submit a vote for an active proposal. Use the --metadata to optionally give a reason. You can find the proposal-id by running "%s query gov proposals"`, version.AppName),
					Example:   fmt.Sprintf("$ %s tx gov vote 1 yes --from mykey", version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "proposal_id"},
						{ProtoField: "option"},
					},
					FlagOptions: map[string]*autocliv1.FlagOptions{
						"metadata": {Name: "metadata", Usage: "Add a description to the vote"},
					},
				},
				{
					RpcMethod:      "UpdateParams",
					Use:            "update-params-proposal <params>",
					Short:          "Submit a proposal to update gov module params. Note: the entire params must be provided.",
					Long:           fmt.Sprintf("Submit a proposal to update gov module params. Note: the entire params must be provided.\n See the fields to fill in by running `%s query gov params --output json`", version.AppName),
					Example:        fmt.Sprintf(`%s tx gov update-params-proposal '{ params }'`, version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "params"}},
					GovProposal:    true,
				},
				{
					RpcMethod: "UpdateMessageParams",
					Use:       "update-params-msg-url-params <msg-url> <msg-params>",
					Short:     "Submit a proposal to update gov module message params. Note: the entire params must be provided.",
					Example:   fmt.Sprintf(`%s tx gov update-msg-params-proposal [msg-url]'{ params }'`, version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "msg_url"},
						{ProtoField: "params"},
					},
					GovProposal: true,
				},
			},
			EnhanceCustomCommand: true, // We still have manual commands in gov that we want to keep
		},
	}
}

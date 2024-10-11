package protocolpool

import (
	"fmt"

	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	poolv1 "cosmossdk.io/api/cosmos/protocolpool/v1"

	"github.com/cosmos/cosmos-sdk/version"
)

// AutoCLIOptions implements the autocli.HasAutoCLIConfig interface.
func (am AppModule) AutoCLIOptions() *autocliv1.ModuleOptions {
	return &autocliv1.ModuleOptions{
		Query: &autocliv1.ServiceCommandDescriptor{
			Service: poolv1.Query_ServiceDesc.ServiceName,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod: "CommunityPool",
					Use:       "community-pool",
					Short:     "Query the amount of coins in the community pool",
					Example:   fmt.Sprintf(`$ %s query protocolpool community-pool`, version.AppName),
				},
				{
					RpcMethod:      "UnclaimedBudget",
					Use:            "unclaimed-budget [recipient-address]",
					Short:          "Query the remaining budget left to be claimed",
					Example:        fmt.Sprintf(`$ %s query protocolpool unclaimed-budget cosmos1...`, version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "address"}},
				},
			},
		},
		Tx: &autocliv1.ServiceCommandDescriptor{
			Service: poolv1.Msg_ServiceDesc.ServiceName,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod:      "FundCommunityPool",
					Use:            "fund-community-pool <amount>",
					Short:          "Funds the community pool with the specified amount",
					Example:        fmt.Sprintf(`$ %s tx protocolpool fund-community-pool 100uatom --from mykey`, version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "amount"}},
				},
				{
					RpcMethod: "SubmitBudgetProposal",
					Use:       "submit-budget-proposal <recipient> <budget-per-tranche> <start-time> <tranches> <period>",
					Short:     "Submit a budget proposal",
					Example:   fmt.Sprintf(`$ %s tx protocolpool submit-budget-proposal cosmos1... 1000000uatom 2023-10-31T12:34:56.789Z 10 1000 --from mykey`, version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "recipient_address"},
						{ProtoField: "budget_per_tranche"},
						{ProtoField: "start_time"},
						{ProtoField: "tranches"},
						{ProtoField: "period"},
					},
				},
				{
					RpcMethod:      "ClaimBudget",
					Use:            "claim-budget <recipient>",
					Short:          "Claim the distributed budget",
					Example:        fmt.Sprintf(`$ %s tx protocolpool claim-budget cosmos1... --from mykey`, version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "recipient_address"}},
				},
				{
					RpcMethod: "CreateContinuousFund",
					Use:       "create-continuous-fund <recipient> <percentage> <expiry>",
					Short:     "Create continuous fund for a recipient with optional expiry",
					Example:   fmt.Sprintf(`$ %s tx protocolpool create-continuous-fund cosmos1... 0.2 2023-11-31T12:34:56.789Z --from mykey`, version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "recipient"},
						{ProtoField: "percentage"},
						{ProtoField: "expiry", Optional: true},
					},
					GovProposal: true,
				},
				{
					RpcMethod:      "WithdrawContinuousFund",
					Use:            "withdraw-continuous-fund <recipient>",
					Short:          "Withdraw continuous fund allocated to the recipient",
					Example:        fmt.Sprintf(`$ %s tx protocolpool withdraw-continuous-fund cosmos1... --from mykey`, version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "recipient_address"}},
				},
				{
					RpcMethod: "CancelContinuousFund",
					Use:       "cancel-continuous-fund <recipient_address>",
					Short:     "Cancel continuous fund for a specific recipient",
					Example:   fmt.Sprintf(`$ %s tx protocolpool cancel-continuous-fund cosmos1... --from mykey`, version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "recipient_address"},
					},
					GovProposal: true,
				},
			},
		},
	}
}

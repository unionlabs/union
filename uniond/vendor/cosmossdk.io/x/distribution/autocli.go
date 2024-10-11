package distribution

import (
	"fmt"

	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	distributionv1beta1 "cosmossdk.io/api/cosmos/distribution/v1beta1"

	"github.com/cosmos/cosmos-sdk/version"
)

// AutoCLIOptions implements the autocli.HasAutoCLIConfig interface.
func (am AppModule) AutoCLIOptions() *autocliv1.ModuleOptions {
	return &autocliv1.ModuleOptions{
		Query: &autocliv1.ServiceCommandDescriptor{
			Service: distributionv1beta1.Query_ServiceDesc.ServiceName,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod: "Params",
					Use:       "params",
					Short:     "Query the current distribution parameters.",
				},
				{
					RpcMethod: "ValidatorDistributionInfo",
					Use:       "validator-distribution-info <validator>",
					Short:     "Query validator distribution info",
					Example:   fmt.Sprintf(`Example: $ %s query distribution validator-distribution-info [validator-address]`, version.AppName),

					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "validator_address"},
					},
				},
				{
					RpcMethod: "ValidatorOutstandingRewards",
					Use:       "validator-outstanding-rewards <validator>",
					Short:     "Query distribution outstanding (un-withdrawn) rewards for a validator and all their delegations",
					Example:   fmt.Sprintf(`$ %s query distribution validator-outstanding-rewards [validator-address]`, version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "validator_address"},
					},
				},
				{
					RpcMethod: "ValidatorCommission",
					Use:       "commission <validator>",
					Short:     "Query distribution validator commission",
					Example:   fmt.Sprintf(`$ %s query distribution commission [validator-address]`, version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "validator_address"},
					},
				},
				{
					RpcMethod: "ValidatorSlashes",
					Use:       "slashes <validator> <start-height> <end-height>",
					Short:     "Query distribution validator slashes",
					Example:   fmt.Sprintf(`$ %s query distribution slashes [validator-address] 0 100`, version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "validator_address"},
						{ProtoField: "starting_height"},
						{ProtoField: "ending_height"},
					},
				},
				{
					RpcMethod: "DelegationRewards",
					Use:       "rewards-by-validator <delegator-addr> <validator-addr>",
					Short:     "Query all distribution delegator from a particular validator",
					Example:   fmt.Sprintf("$ %s query distribution rewards [delegator-address] [validator-address]", version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "delegator_address"},
						{ProtoField: "validator_address"},
					},
				},
				{
					RpcMethod: "DelegationTotalRewards",
					Use:       "rewards <delegator-addr>",
					Short:     "Query all distribution delegator rewards",
					Long:      "Query all rewards earned by a delegator",
					Example:   fmt.Sprintf("$ %s query distribution rewards [delegator-address]", version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "delegator_address"},
					},
				},
				{
					RpcMethod: "CommunityPool",
					Use:       "community-pool",
					Short:     "Query the amount of coins in the community pool",
					Example:   fmt.Sprintf(`$ %s query distribution community-pool`, version.AppName),
				},
			},
		},
		Tx: &autocliv1.ServiceCommandDescriptor{
			Service: distributionv1beta1.Msg_ServiceDesc.ServiceName,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod: "SetWithdrawAddress",
					Use:       "set-withdraw-addr <withdraw-addr>",
					Short:     "Change the default withdraw address for rewards associated with an address",
					Example:   fmt.Sprintf("%s tx distribution set-withdraw-addr cosmos1gghjut3ccd8ay0zduzj64hwre2fxs9ld75ru9p --from mykey", version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "withdraw_address"},
					},
				},
				{
					RpcMethod: "WithdrawDelegatorReward",
					Use:       "withdraw-rewards <validator-addr>",
					Short:     "Withdraw rewards from a given delegation address",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "validator_address"},
					},
				},
				{
					RpcMethod: "WithdrawValidatorCommission",
					Use:       "withdraw-validator-commission <validator-addr>",
					Short:     "Withdraw commissions from a validator address (must be a validator operator)",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "validator_address"},
					},
				},
				{
					RpcMethod: "DepositValidatorRewardsPool",
					Use:       "fund-validator-rewards-pool <validator-addr> <amount>",
					Short:     "Fund the validator rewards pool with the specified amount",
					Example:   fmt.Sprintf("%s tx distribution fund-validator-rewards-pool cosmosvaloper1x20lytyf6zkcrv5edpkfkn8sz578qg5sqfyqnp 100uatom --from mykey", version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "validator_address"},
						{ProtoField: "amount", Varargs: true},
					},
				},
				{
					RpcMethod:  "FundCommunityPool",
					Deprecated: fmt.Sprintf("Use %s tx protocolpool fund-community-pool", version.AppName),
					Use:        "fund-community-pool <amount>",
					Short:      "Funds the community pool with the specified amount",
					Example:    fmt.Sprintf(`$ %s tx distribution fund-community-pool 100uatom --from mykey`, version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "amount", Varargs: true},
					},
				},
				{
					RpcMethod:      "UpdateParams",
					Use:            "update-params-proposal <params>",
					Short:          "Submit a proposal to update distribution module params. Note: the entire params must be provided.",
					Example:        fmt.Sprintf(`%s tx distribution update-params-proposal '{ "community_tax": "20000", "base_proposer_reward": "0", "bonus_proposer_reward": "0", "withdraw_addr_enabled": true }'`, version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "params"}},
					GovProposal:    true,
				},
				{
					RpcMethod: "CommunityPoolSpend",
					Skip:      true, // skipped because deprecated in favor of protocolpool
				},
			},
			EnhanceCustomCommand: true,
		},
	}
}

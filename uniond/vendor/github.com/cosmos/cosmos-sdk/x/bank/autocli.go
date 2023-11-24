package bank

import (
	"strings"

	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	bankv1beta1 "cosmossdk.io/api/cosmos/bank/v1beta1"
)

// AutoCLIOptions implements the autocli.HasAutoCLIConfig interface.
func (am AppModule) AutoCLIOptions() *autocliv1.ModuleOptions {
	return &autocliv1.ModuleOptions{
		Query: &autocliv1.ServiceCommandDescriptor{
			Service: bankv1beta1.Query_ServiceDesc.ServiceName,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod:      "Balance",
					Use:            "balance [address] [denom]",
					Short:          "Query an account balance by address and denom",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "address"}, {ProtoField: "denom"}},
				},
				{
					RpcMethod:      "AllBalances",
					Use:            "balances [address]",
					Short:          "Query for account balances by address",
					Long:           "Query the total balance of an account or of a specific denomination.",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "address"}},
				},
				{
					RpcMethod:      "SpendableBalances",
					Use:            "spendable-balances [address]",
					Short:          "Query for account spendable balances by address",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "address"}},
				},
				{
					RpcMethod:      "SpendableBalanceByDenom",
					Use:            "spendable-balance [address] [denom]",
					Short:          "Query the spendable balance of a single denom for a single account.",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "address"}, {ProtoField: "denom"}},
				},
				{
					RpcMethod: "TotalSupply",
					Use:       "total-supply",
					Alias:     []string{"total"},
					Short:     "Query the total supply of coins of the chain",
					Long:      "Query total supply of coins that are held by accounts in the chain. To query for the total supply of a specific coin denomination use --denom flag.",
				},
				{
					RpcMethod:      "SupplyOf",
					Use:            "total-supply-of [denom]",
					Short:          "Query the supply of a single coin denom",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "denom"}},
				},
				{
					RpcMethod: "Params",
					Use:       "params",
					Short:     "Query the current bank parameters",
				},
				{
					RpcMethod:      "DenomMetadata",
					Use:            "denom-metadata [denom]",
					Short:          "Query the client metadata of a given coin denomination",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "denom"}},
				},
				{
					RpcMethod: "DenomsMetadata",
					Use:       "denoms-metadata",
					Short:     "Query the client metadata for all registered coin denominations",
				},
				{
					RpcMethod:      "DenomOwners",
					Use:            "denom-owners [denom]",
					Short:          "Query for all account addresses that own a particular token denomination.",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "denom"}},
				},
				{
					RpcMethod: "SendEnabled",
					Use:       "send-enabled [denom1 ...]",
					Short:     "Query for send enabled entries",
					Long: strings.TrimSpace(`Query for send enabled entries that have been specifically set.
			
To look up one or more specific denoms, supply them as arguments to this command.
To look up all denoms, do not provide any arguments.`,
					),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "denoms"}},
				},
			},
		},
		Tx: &autocliv1.ServiceCommandDescriptor{
			Service:              bankv1beta1.Msg_ServiceDesc.ServiceName,
			EnhanceCustomCommand: false, // use custom commands only until v0.51
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod: "Send",
					Use:       "send [from_key_or_address] [to_address] [amount]",
					Short:     "Send funds from one account to another.",
					Long: `Send funds from one account to another.
Note, the '--from' flag is ignored as it is implied from [from_key_or_address].
When using '--dry-run' a key name cannot be used, only a bech32 address.
Note: multiple coins can be send by space separated.`,
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "from_address"}, {ProtoField: "to_address"}, {ProtoField: "amount", Varargs: true}},
				},
				{
					RpcMethod: "UpdateParams",
					Skip:      true, // skipped because authority gated
				},
				{
					RpcMethod: "SetSendEnabled",
					Skip:      true, // skipped because authority gated
				},
				{
					RpcMethod: "Burn",
					Skip:      true, // skipped because available from v0.51
				},
			},
		},
	}
}

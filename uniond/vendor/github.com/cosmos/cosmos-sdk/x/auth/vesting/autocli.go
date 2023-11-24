package vesting

import (
	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	vestingv1beta1 "cosmossdk.io/api/cosmos/vesting/v1beta1"
)

// AutoCLIOptions implements the autocli.HasAutoCLIConfig interface.
func (am AppModule) AutoCLIOptions() *autocliv1.ModuleOptions {
	return &autocliv1.ModuleOptions{
		Tx: &autocliv1.ServiceCommandDescriptor{
			Service: vestingv1beta1.Msg_ServiceDesc.ServiceName,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod: "CreateVestingAccount",
					Use:       "create-vesting-account [to_address] [end_time] [amount]",
					Short:     "Create a new vesting account funded with an allocation of tokens.",
					Long: `Create a new vesting account funded with an allocation of tokens. The
account can either be a delayed or continuous vesting account, which is determined
by the '--delayed' flag. All vesting accounts created will have their start time
set by the committed block's time. The end_time must be provided as a UNIX epoch
timestamp.`,
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "to_address"},
						{ProtoField: "end_time"},
						{ProtoField: "amount", Varargs: true},
					},
					FlagOptions: map[string]*autocliv1.FlagOptions{
						"delayed": {Name: "delayed", Usage: "Create a delayed vesting account if true"},
					},
				},
				{
					RpcMethod: "CreatePermanentLockedAccount",
					Use:       "create-permanent-locked-account [to_address] [amount]",
					Short:     "Create a new permanently locked account funded with an allocation of tokens.",
					Long: `Create a new account funded with an allocation of permanently locked tokens.
These tokens may be used for staking but are non-transferable. Staking rewards will acrue as liquid and transferable tokens.`,
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "to_address"},
						{ProtoField: "amount", Varargs: true},
					},
				},
			},
			EnhanceCustomCommand: false, // use custom commands only until v0.51
		},
	}
}

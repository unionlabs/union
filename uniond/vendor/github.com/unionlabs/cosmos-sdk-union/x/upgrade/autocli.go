package upgrade

import (
	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	upgradev1beta1 "cosmossdk.io/api/cosmos/upgrade/v1beta1"
)

func (am AppModule) AutoCLIOptions() *autocliv1.ModuleOptions {
	return &autocliv1.ModuleOptions{
		Query: &autocliv1.ServiceCommandDescriptor{
			Service: upgradev1beta1.Query_ServiceDesc.ServiceName,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod: "CurrentPlan",
					Use:       "plan",
					Short:     "Query the upgrade plan (if one exists)",
					Long:      "Gets the currently scheduled upgrade plan, if one exists",
				},
				{
					RpcMethod: "AppliedPlan",
					Use:       "applied <upgrade-name>",
					Short:     "Query the block header for height at which a completed upgrade was applied",
					Long:      "If upgrade-name was previously executed on the chain, this returns the header for the block at which it was applied. This helps a client determine which binary was valid over a given range of blocks, as well as more context to understand past migrations.",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "name"},
					},
				},
				{
					RpcMethod: "ModuleVersions",
					Use:       "module-versions [optional module_name]",
					Alias:     []string{"module_versions"},
					Short:     "Query the list of module versions",
					Long:      "Gets a list of module names and their respective consensus versions. Following the command with a specific module name will return only that module's information.",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "module_name", Optional: true},
					},
				},
				{
					RpcMethod: "Authority",
					Use:       "authority",
					Short:     "Get the upgrade authority address",
				},
				{
					RpcMethod: "UpgradedConsensusState",
					Skip:      true, // Skipping this command as the query is deprecated.
				},
			},
		},
		Tx: &autocliv1.ServiceCommandDescriptor{
			Service: upgradev1beta1.Msg_ServiceDesc.ServiceName,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod:   "CancelUpgrade",
					Use:         "cancel-upgrade-proposal",
					Short:       "Submit a proposal to cancel a planned chain upgrade.",
					GovProposal: true,
				},
				{
					RpcMethod: "SoftwareUpgrade",
					Skip:      true, // skipped because authority gated
				},
			},
			EnhanceCustomCommand: true,
		},
	}
}

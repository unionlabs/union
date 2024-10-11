package epochs

import (
	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	epochsv1beta1 "cosmossdk.io/api/cosmos/epochs/v1beta1"
)

// AutoCLIOptions implements the autocli.HasAutoCLIConfig interface.
func (am AppModule) AutoCLIOptions() *autocliv1.ModuleOptions {
	return &autocliv1.ModuleOptions{
		Query: &autocliv1.ServiceCommandDescriptor{
			Service: epochsv1beta1.Query_ServiceDesc.ServiceName,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod: "EpochInfos",
					Use:       "epoch-infos",
					Short:     "Query running epoch infos",
				},
				{
					RpcMethod: "CurrentEpoch",
					Use:       "current-epoch",
					Short:     "Query current epoch by specified identifier",
				},
			},
		},
	}
}

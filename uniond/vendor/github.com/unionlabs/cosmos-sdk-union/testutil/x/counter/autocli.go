package counter

import (
	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	counterv1 "cosmossdk.io/api/cosmos/counter/v1"
)

// AutoCLIOptions implements the autocli.HasAutoCLIConfig interface.
func (am AppModule) AutoCLIOptions() *autocliv1.ModuleOptions {
	return &autocliv1.ModuleOptions{
		Query: &autocliv1.ServiceCommandDescriptor{
			Service: counterv1.Query_ServiceDesc.ServiceName,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod: "GetCount",
					Use:       "count",
					Short:     "Query the current counter value",
				},
			},
		},
		Tx: &autocliv1.ServiceCommandDescriptor{
			Service: counterv1.Msg_ServiceDesc.ServiceName,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod:      "IncreaseCount",
					Use:            "increase-count <count>",
					Alias:          []string{"increase-counter", "increase", "inc", "bump"},
					Short:          "Increase the counter by the specified amount",
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "count"}},
				},
			},
		},
	}
}

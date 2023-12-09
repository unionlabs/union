package consensus

import (
	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	consensusv1 "cosmossdk.io/api/cosmos/consensus/v1"

	"github.com/cosmos/cosmos-sdk/client/grpc/cmtservice"
)

// AutoCLIOptions implements the autocli.HasAutoCLIConfig interface.
func (am AppModule) AutoCLIOptions() *autocliv1.ModuleOptions {
	return &autocliv1.ModuleOptions{
		Query: &autocliv1.ServiceCommandDescriptor{
			Service: consensusv1.Query_ServiceDesc.ServiceName,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod: "Params",
					Use:       "params",
					Short:     "Query the current consensus parameters",
				},
			},
			SubCommands: map[string]*autocliv1.ServiceCommandDescriptor{
				"comet": cmtservice.CometBFTAutoCLIDescriptor,
			},
		},
		Tx: &autocliv1.ServiceCommandDescriptor{
			Service: consensusv1.Msg_ServiceDesc.ServiceName,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod: "UpdateParams",
					Skip:      true, // skipped because authority gated
				},
			},
		},
	}
}

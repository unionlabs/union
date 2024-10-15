package consensus

import (
	"fmt"

	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	consensusv1 "cosmossdk.io/api/cosmos/consensus/v1"

	"github.com/cosmos/cosmos-sdk/client/grpc/cmtservice"
	"github.com/cosmos/cosmos-sdk/version"
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
					Use:       "update-params-proposal <params>",
					Short:     "Submit a proposal to update consensus module params. Note: the entire params must be provided.",
					Example:   fmt.Sprintf(`%s tx consensus update-params-proposal '{ params }'`, version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{
						{ProtoField: "block"},
						{ProtoField: "evidence"},
						{ProtoField: "validator"},
						{ProtoField: "abci"},
					},
					GovProposal: true,
				},
			},
		},
	}
}

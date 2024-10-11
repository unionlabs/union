package mint

import (
	"fmt"

	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	mintv1beta1 "cosmossdk.io/api/cosmos/mint/v1beta1"

	"github.com/cosmos/cosmos-sdk/version"
)

func (am AppModule) AutoCLIOptions() *autocliv1.ModuleOptions {
	return &autocliv1.ModuleOptions{
		Query: &autocliv1.ServiceCommandDescriptor{
			Service: mintv1beta1.Query_ServiceDesc.ServiceName,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod: "Params",
					Use:       "params",
					Short:     "Query the current minting parameters",
				},
				{
					RpcMethod: "Inflation",
					Use:       "inflation",
					Short:     "Query the current minting inflation value",
				},
				{
					RpcMethod: "AnnualProvisions",
					Use:       "annual-provisions",
					Short:     "Query the current minting annual provisions value",
				},
			},
		},
		Tx: &autocliv1.ServiceCommandDescriptor{
			Service: mintv1beta1.Msg_ServiceDesc.ServiceName,
			RpcCommandOptions: []*autocliv1.RpcCommandOptions{
				{
					RpcMethod:      "UpdateParams",
					Use:            "update-params-proposal <params>",
					Short:          "Submit a proposal to update mint module params. Note: the entire params must be provided.",
					Long:           fmt.Sprintf("Submit a proposal to update mint module params. Note: the entire params must be provided.\n See the fields to fill in by running `%s query mint params --output json`", version.AppName),
					Example:        fmt.Sprintf(`%s tx mint update-params-proposal '{ "mint_denom": "stake", ... }'`, version.AppName),
					PositionalArgs: []*autocliv1.PositionalArgDescriptor{{ProtoField: "params"}},
					GovProposal:    true,
				},
			},
		},
	}
}

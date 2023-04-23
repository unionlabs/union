package gov

import (
	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	govv1 "cosmossdk.io/api/cosmos/gov/v1"
	govv1beta1 "cosmossdk.io/api/cosmos/gov/v1beta1"
)

// AutoCLIOptions implements the autocli.HasAutoCLIConfig interface.
func (am AppModule) AutoCLIOptions() *autocliv1.ModuleOptions {
	return &autocliv1.ModuleOptions{
		Tx: &autocliv1.ServiceCommandDescriptor{
			Service: govv1.Msg_ServiceDesc.ServiceName,
			// map v1beta1 as a sub-command
			SubCommands: map[string]*autocliv1.ServiceCommandDescriptor{
				"v1beta1": {Service: govv1beta1.Msg_ServiceDesc.ServiceName},
			},
		},
		Query: &autocliv1.ServiceCommandDescriptor{
			Service: govv1.Query_ServiceDesc.ServiceName,
			// map v1beta1 as a sub-command
			SubCommands: map[string]*autocliv1.ServiceCommandDescriptor{
				"v1beta1": {Service: govv1beta1.Query_ServiceDesc.ServiceName},
			},
		},
	}
}

package node

import (
	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	nodev1beta1 "cosmossdk.io/api/cosmos/base/node/v1beta1"
)

var ServiceAutoCLIDescriptor = &autocliv1.ServiceCommandDescriptor{
	Service: nodev1beta1.Service_ServiceDesc.ServiceName,
	RpcCommandOptions: []*autocliv1.RpcCommandOptions{
		{
			RpcMethod: "Config",
			Use:       "config",
			Short:     "Query the current node config",
		},
		{
			RpcMethod: "Status",
			Use:       "status",
			Short:     "Query the current node status",
		},
	},
}

// NewNodeCommands is a fake `appmodule.Module` to be considered as a module
// and be added in AutoCLI.
func NewNodeCommands() *nodeModule {
	return &nodeModule{}
}

type nodeModule struct{}

func (m nodeModule) IsOnePerModuleType() {}
func (m nodeModule) IsAppModule()        {}

func (m nodeModule) Name() string {
	return "node"
}

func (m nodeModule) AutoCLIOptions() *autocliv1.ModuleOptions {
	return &autocliv1.ModuleOptions{
		Query: ServiceAutoCLIDescriptor,
	}
}

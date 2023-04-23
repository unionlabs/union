package services

import (
	"context"

	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	gogogrpc "github.com/cosmos/gogoproto/grpc"
	"google.golang.org/grpc"

	"github.com/cosmos/cosmos-sdk/types/module"
)

// AutoCLIQueryService implements the cosmos.autocli.v1.Query service.
type AutoCLIQueryService struct {
	autocliv1.UnimplementedQueryServer

	moduleOptions map[string]*autocliv1.ModuleOptions
}

func NewAutoCLIQueryService(appModules map[string]interface{}) *AutoCLIQueryService {
	moduleOptions := map[string]*autocliv1.ModuleOptions{}
	for modName, mod := range appModules {
		if autoCliMod, ok := mod.(interface {
			AutoCLIOptions() *autocliv1.ModuleOptions
		}); ok {
			moduleOptions[modName] = autoCliMod.AutoCLIOptions()
		} else if mod, ok := mod.(module.HasServices); ok {
			// try to auto-discover options based on the last msg and query
			// services registered for the module
			cfg := &autocliConfigurator{}
			mod.RegisterServices(cfg)
			modOptions := &autocliv1.ModuleOptions{}
			haveServices := false

			if cfg.msgServer.serviceName != "" {
				haveServices = true
				modOptions.Tx = &autocliv1.ServiceCommandDescriptor{
					Service: cfg.msgServer.serviceName,
				}
			}

			if cfg.queryServer.serviceName != "" {
				haveServices = true
				modOptions.Query = &autocliv1.ServiceCommandDescriptor{
					Service: cfg.queryServer.serviceName,
				}
			}

			if haveServices {
				moduleOptions[modName] = modOptions
			}
		}
	}
	return &AutoCLIQueryService{
		moduleOptions: moduleOptions,
	}
}

func (a AutoCLIQueryService) AppOptions(context.Context, *autocliv1.AppOptionsRequest) (*autocliv1.AppOptionsResponse, error) {
	return &autocliv1.AppOptionsResponse{
		ModuleOptions: a.moduleOptions,
	}, nil
}

// autocliConfigurator allows us to call RegisterServices and introspect the services
type autocliConfigurator struct {
	msgServer   autocliServiceRegistrar
	queryServer autocliServiceRegistrar
}

func (a *autocliConfigurator) MsgServer() gogogrpc.Server { return &a.msgServer }

func (a *autocliConfigurator) QueryServer() gogogrpc.Server { return &a.queryServer }

func (a *autocliConfigurator) RegisterMigration(string, uint64, module.MigrationHandler) error {
	return nil
}

// autocliServiceRegistrar is used to capture the service name for registered services
type autocliServiceRegistrar struct {
	serviceName string
}

func (a *autocliServiceRegistrar) RegisterService(sd *grpc.ServiceDesc, _ interface{}) {
	a.serviceName = sd.ServiceName
}

var _ autocliv1.QueryServer = &AutoCLIQueryService{}

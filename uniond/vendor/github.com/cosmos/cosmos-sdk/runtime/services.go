package runtime

import (
	"context"

	appv1alpha1 "cosmossdk.io/api/cosmos/app/v1alpha1"
	autocliv1 "cosmossdk.io/api/cosmos/autocli/v1"
	reflectionv1 "cosmossdk.io/api/cosmos/reflection/v1"
	"cosmossdk.io/core/comet"
	"cosmossdk.io/core/header"

	"github.com/cosmos/cosmos-sdk/runtime/services"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
)

func (a *App) registerRuntimeServices(cfg module.Configurator) error {
	appv1alpha1.RegisterQueryServer(cfg.QueryServer(), services.NewAppQueryService(a.appConfig))
	autocliv1.RegisterQueryServer(cfg.QueryServer(), services.NewAutoCLIQueryService(a.ModuleManager.Modules))

	reflectionSvc, err := services.NewReflectionService()
	if err != nil {
		return err
	}
	reflectionv1.RegisterReflectionServiceServer(cfg.QueryServer(), reflectionSvc)

	return nil
}

var _ comet.BlockInfoService = cometInfoService{}

type cometInfoService struct{}

func (cometInfoService) GetCometBlockInfo(ctx context.Context) comet.BlockInfo {
	return sdk.UnwrapSDKContext(ctx).CometInfo()
}

var _ header.Service = headerInfoService{}

type headerInfoService struct{}

func (headerInfoService) GetHeaderInfo(ctx context.Context) header.Info {
	return sdk.UnwrapSDKContext(ctx).HeaderInfo()
}

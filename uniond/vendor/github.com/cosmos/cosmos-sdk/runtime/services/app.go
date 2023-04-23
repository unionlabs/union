package services

import (
	"context"

	appv1alpha1 "cosmossdk.io/api/cosmos/app/v1alpha1"
)

// AppQueryService implements the cosmos.app.v1alpha1.Query service
type AppQueryService struct {
	appv1alpha1.UnimplementedQueryServer
	appConfig *appv1alpha1.Config
}

func NewAppQueryService(appConfig *appv1alpha1.Config) *AppQueryService {
	return &AppQueryService{appConfig: appConfig}
}

func (a *AppQueryService) Config(context.Context, *appv1alpha1.QueryConfigRequest) (*appv1alpha1.QueryConfigResponse, error) {
	return &appv1alpha1.QueryConfigResponse{Config: a.appConfig}, nil
}

var _ appv1alpha1.QueryServer = &AppQueryService{}

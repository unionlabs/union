package runtime

import (
	"context"

	corecomet "cosmossdk.io/core/comet"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

var _ corecomet.Service = &ContextAwareCometInfoService{}

// ContextAwareCometInfoService provides CometInfo which is embedded as a value in a Context.
// This the legacy (server v1, baseapp) way of accessing CometInfo at the module level.
type ContextAwareCometInfoService struct{}

func (c ContextAwareCometInfoService) CometInfo(ctx context.Context) corecomet.Info {
	return sdk.UnwrapSDKContext(ctx).CometInfo()
}

func NewContextAwareCometInfoService() *ContextAwareCometInfoService {
	return &ContextAwareCometInfoService{}
}

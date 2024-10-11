package runtime

import (
	"context"

	"cosmossdk.io/core/header"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

var _ header.Service = (*HeaderService)(nil)

type HeaderService struct{}

func (h HeaderService) HeaderInfo(ctx context.Context) header.Info {
	return sdk.UnwrapSDKContext(ctx).HeaderInfo()
}

package simsx

import (
	"context"
	"time"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// BlockTime read header block time from sdk context or sims context key if not present
func BlockTime(ctx context.Context) time.Time {
	sdkCtx, ok := sdk.TryUnwrapSDKContext(ctx)
	if ok {
		return sdkCtx.BlockTime()
	}
	return ctx.Value("sims.header.time").(time.Time)
}

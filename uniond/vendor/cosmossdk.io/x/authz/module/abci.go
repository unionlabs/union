package module

import (
	"context"

	"cosmossdk.io/x/authz/keeper"
)

// BeginBlocker is called at the beginning of every block
func BeginBlocker(ctx context.Context, keeper keeper.Keeper) error {
	// delete all the mature grants
	// 200 is an arbitrary value, we can change it later if needed
	return keeper.DequeueAndDeleteExpiredGrants(ctx, 200)
}

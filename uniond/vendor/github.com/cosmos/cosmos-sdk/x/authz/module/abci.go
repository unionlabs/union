package authz

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/authz/keeper"
)

// BeginBlocker is called at the beginning of every block
func BeginBlocker(ctx sdk.Context, keeper keeper.Keeper) error {
	// delete all the mature grants
	return keeper.DequeueAndDeleteExpiredGrants(ctx)
}

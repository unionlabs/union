package accountstd

import (
	"context"

	"cosmossdk.io/core/store"
	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/x/accounts/internal/implementation"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

func NewMockContext(
	accNumber uint64,
	accountAddr []byte,
	sender []byte,
	funds sdk.Coins,
	moduleExec implementation.ModuleExecFunc,
	moduleQuery implementation.ModuleQueryFunc,
) (context.Context, store.KVStoreService) {
	ctx := coretesting.Context()
	ss := coretesting.KVStoreService(ctx, "test")

	return implementation.MakeAccountContext(
		ctx, ss, accNumber, accountAddr, sender, funds, moduleExec, moduleQuery,
	), ss
}

func SetSender(ctx context.Context, sender []byte) context.Context {
	return implementation.SetSender(ctx, sender)
}

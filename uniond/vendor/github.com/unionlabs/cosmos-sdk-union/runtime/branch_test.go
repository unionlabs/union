package runtime

import (
	"context"
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

func TestBranchService(t *testing.T) {
	bs := BranchService{}
	sk := storetypes.NewKVStoreKey("test")
	tsk := storetypes.NewTransientStoreKey("transient-test")
	// helper to create a state change
	doStateChange := func(ctx context.Context) {
		t.Helper()
		sdkCtx := sdk.UnwrapSDKContext(ctx)
		store := sdkCtx.KVStore(sk)
		store.Set([]byte("key"), []byte("value"))
	}

	// asserts a state change
	assertRollback := func(ctx context.Context, shouldRollback bool) {
		sdkCtx := sdk.UnwrapSDKContext(ctx).WithGasMeter(storetypes.NewInfiniteGasMeter()) // we don't want to consume gas for assertions
		store := sdkCtx.KVStore(sk)
		if shouldRollback && store.Has([]byte("key")) {
			t.Error("expected key to not exist")
		}
		if !shouldRollback && !store.Has([]byte("key")) {
			t.Error("expected key to exist")
		}
	}

	t.Run("execute successful", func(t *testing.T) {
		ctx := testutil.DefaultContext(sk, tsk)
		err := bs.Execute(ctx, func(ctx context.Context) error {
			doStateChange(ctx)
			return nil
		})
		require.NoError(t, err)
		assertRollback(ctx, false)
	})

	t.Run("execute failed", func(t *testing.T) {
		ctx := testutil.DefaultContext(sk, tsk)
		err := bs.Execute(ctx, func(ctx context.Context) error {
			doStateChange(ctx)
			return fmt.Errorf("failure")
		})
		require.Error(t, err)
		assertRollback(ctx, true)
	})

	t.Run("execute with limit successful", func(t *testing.T) {
		ctx := testutil.DefaultContext(sk, tsk)
		gasUsed, err := bs.ExecuteWithGasLimit(ctx, 4_000, func(ctx context.Context) error {
			doStateChange(ctx)
			return nil
		})
		require.NoError(t, err)
		// assert gas used
		require.Equal(t, 2240, int(gasUsed))
		assertRollback(ctx, false)
		// assert that original context gas was applied
		require.Equal(t, 2240, int(ctx.GasMeter().GasConsumed()))
	})

	t.Run("execute with limit failed", func(t *testing.T) {
		ctx := testutil.DefaultContext(sk, tsk)
		gasUsed, err := bs.ExecuteWithGasLimit(ctx, 4_000, func(ctx context.Context) error {
			doStateChange(ctx)
			return fmt.Errorf("failure")
		})
		require.Error(t, err)
		// assert gas used
		require.Equal(t, 2240, int(gasUsed))
		assertRollback(ctx, true)
		// assert that original context gas was applied
		require.Equal(t, 2240, int(ctx.GasMeter().GasConsumed()))
	})

	t.Run("execute with limit out of gas", func(t *testing.T) {
		ctx := testutil.DefaultContext(sk, tsk)
		gasUsed, err := bs.ExecuteWithGasLimit(ctx, 2239, func(ctx context.Context) error {
			doStateChange(ctx)
			return nil
		})
		require.ErrorIs(t, err, sdkerrors.ErrOutOfGas)
		// assert gas used
		require.Equal(t, 2240, int(gasUsed))
		assertRollback(ctx, true)
		// assert that original context gas was applied
		require.Equal(t, 2240, int(ctx.GasMeter().GasConsumed()))
	})

	t.Run("execute with gas limit other panic error", func(t *testing.T) {
		// ensures other panic errors are not caught by the gas limit panic catcher
		ctx := testutil.DefaultContext(sk, tsk)
		require.Panics(t, func() {
			_, _ = bs.ExecuteWithGasLimit(ctx, 2239, func(ctx context.Context) error {
				panic("other panic error")
			})
		})
	})
}

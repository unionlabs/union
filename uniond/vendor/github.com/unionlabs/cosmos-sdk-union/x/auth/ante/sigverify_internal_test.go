package ante_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/transaction"

	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/auth/ante"
	authcodec "github.com/cosmos/cosmos-sdk/x/auth/codec"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

type mockAccount struct {
	ante.AccountKeeper
}

func (*mockAccount) GetEnvironment() appmodule.Environment {
	return appmodule.Environment{
		TransactionService: &mockTransactionService{},
	}
}

type mockTransactionService struct {
	transaction.Service
}

func (*mockTransactionService) ExecMode(ctx context.Context) transaction.ExecMode {
	return transaction.ExecMode(sdk.UnwrapSDKContext(ctx).ExecMode())
}

func TestSigVerify_setPubKey(t *testing.T) {
	svd := ante.NewSigVerificationDecorator(&mockAccount{}, nil, nil, nil)

	alicePk := secp256k1.GenPrivKey().PubKey()
	bobPk := secp256k1.GenPrivKey().PubKey()

	cdc := authcodec.NewBech32Codec("cosmos")

	aliceAddr, err := cdc.BytesToString(alicePk.Address())
	require.NoError(t, err)

	ctx := sdk.NewContext(nil, false, nil)

	t.Run("on not sig verify tx - skip", func(t *testing.T) {
		acc := &authtypes.BaseAccount{}
		ctx = ctx.WithExecMode(sdk.ExecModeSimulate).WithIsSigverifyTx(false)
		err := ante.SetSVDPubKey(svd, ctx, acc, nil)
		require.NoError(t, err)
	})

	t.Run("on sim, populate with sim key, if pubkey is nil", func(t *testing.T) {
		acc := &authtypes.BaseAccount{Address: aliceAddr}
		ctx = ctx.WithExecMode(sdk.ExecModeSimulate).WithIsSigverifyTx(true)
		err := ante.SetSVDPubKey(svd, ctx, acc, nil)
		require.NoError(t, err)
		require.Equal(t, acc.PubKey.GetCachedValue(), ante.SimSecp256k1PubkeyInternal)
	})

	t.Run("on sim, populate with real pub key, if pubkey is not nil", func(t *testing.T) {
		acc := &authtypes.BaseAccount{Address: aliceAddr}
		ctx = ctx.WithExecMode(sdk.ExecModeSimulate).WithIsSigverifyTx(true)
		err := ante.SetSVDPubKey(svd, ctx, acc, alicePk)
		require.NoError(t, err)
		require.Equal(t, acc.PubKey.GetCachedValue(), alicePk)
	})

	t.Run("not on sim, populate the address", func(t *testing.T) {
		acc := &authtypes.BaseAccount{Address: aliceAddr}
		ctx = ctx.WithExecMode(sdk.ExecModeFinalize).WithIsSigverifyTx(true)
		err := ante.SetSVDPubKey(svd, ctx, acc, alicePk)
		require.NoError(t, err)
		require.Equal(t, acc.PubKey.GetCachedValue(), alicePk)
	})

	t.Run("not on sim, fail on invalid pubkey.address", func(t *testing.T) {
		acc := &authtypes.BaseAccount{Address: aliceAddr}
		ctx = ctx.WithExecMode(sdk.ExecModeFinalize).WithIsSigverifyTx(true)
		err := ante.SetSVDPubKey(svd, ctx, acc, bobPk)
		require.ErrorContains(t, err, "cannot be claimed")
	})
}

package v2_test

import (
	"bytes"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/testutil"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	v1 "github.com/cosmos/cosmos-sdk/x/slashing/migrations/v1"
	v2 "github.com/cosmos/cosmos-sdk/x/slashing/migrations/v2"
	"github.com/cosmos/cosmos-sdk/x/slashing/types"
)

func TestStoreMigration(t *testing.T) {
	slashingKey := sdk.NewKVStoreKey("slashing")
	ctx := testutil.DefaultContext(slashingKey, sdk.NewTransientStoreKey("transient_test"))
	store := ctx.KVStore(slashingKey)

	_, _, addr1 := testdata.KeyTestPubAddr()
	consAddr := sdk.ConsAddress(addr1)
	// Use dummy value for all keys.
	value := []byte("foo")

	testCases := []struct {
		name   string
		oldKey []byte
		newKey []byte
	}{
		{
			"ValidatorSigningInfoKey",
			v1.ValidatorSigningInfoKey(consAddr),
			types.ValidatorSigningInfoKey(consAddr),
		},
		{
			"ValidatorMissedBlockBitArrayKey",
			v1.ValidatorMissedBlockBitArrayKey(consAddr, 2),
			types.ValidatorMissedBlockBitArrayKey(consAddr, 2),
		},
		{
			"AddrPubkeyRelationKey",
			v1.AddrPubkeyRelationKey(consAddr),
			types.AddrPubkeyRelationKey(consAddr),
		},
	}

	// Set all the old keys to the store
	for _, tc := range testCases {
		store.Set(tc.oldKey, value)
	}

	// Run migrations.
	err := v2.MigrateStore(ctx, slashingKey)
	require.NoError(t, err)

	// Make sure the new keys are set and old keys are deleted.
	for _, tc := range testCases {
		tc := tc
		t.Run(tc.name, func(t *testing.T) {
			if !bytes.Equal(tc.oldKey, tc.newKey) {
				require.Nil(t, store.Get(tc.oldKey))
			}
			require.Equal(t, value, store.Get(tc.newKey))
		})
	}
}

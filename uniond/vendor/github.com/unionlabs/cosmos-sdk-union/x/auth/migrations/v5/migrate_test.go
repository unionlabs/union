package v5

import (
	"testing"

	"github.com/cosmos/gogoproto/types"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/collections"
	"cosmossdk.io/core/testing"
)

func TestMigrate(t *testing.T) {
	ctx := coretesting.Context()
	kv := coretesting.KVStoreService(ctx, "test")
	sb := collections.NewSchemaBuilder(kv)
	seq := collections.NewSequence(sb, collections.NewPrefix(0), "seq")

	wantValue := uint64(100)

	// set old sequence to wanted value
	legacySeqBytes, err := (&types.UInt64Value{Value: wantValue}).Marshal()
	require.NoError(t, err)

	err = kv.OpenKVStore(ctx).Set(LegacyGlobalAccountNumberKey, legacySeqBytes)
	require.NoError(t, err)

	err = Migrate(ctx, kv, seq)
	require.NoError(t, err)

	// check that after migration the sequence is what we want it to be
	gotValue, err := seq.Peek(ctx)
	require.NoError(t, err)
	require.Equal(t, wantValue, gotValue)

	// case the global account number was not set
	ctx = coretesting.Context()
	kv = coretesting.KVStoreService(ctx, "test")
	wantValue = collections.DefaultSequenceStart

	err = Migrate(ctx, kv, seq)
	require.NoError(t, err)

	gotValue, err = seq.Next(ctx)
	require.NoError(t, err)
	require.Equal(t, wantValue, gotValue)
}

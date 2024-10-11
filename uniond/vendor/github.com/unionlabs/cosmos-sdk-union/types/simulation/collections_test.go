package simulation

import (
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/collections"
	"cosmossdk.io/core/testing"

	"github.com/cosmos/cosmos-sdk/types/kv"
)

func TestNewStoreDecoderFuncFromCollectionsSchema(t *testing.T) {
	ctx := coretesting.Context()
	store := coretesting.KVStoreService(ctx, "test")
	sb := collections.NewSchemaBuilder(store)

	prefixM1 := collections.NewPrefix("map_1")
	prefixM2 := collections.NewPrefix("map_2")

	m1 := collections.NewMap(sb, prefixM1, "map_1", collections.StringKey, collections.StringValue)
	m2 := collections.NewMap(sb, prefixM2, "map_2", collections.Int32Key, collections.Int32Value)

	schema, err := sb.Build()
	require.NoError(t, err)

	// create a new store decoder function from the schema
	dec := NewStoreDecoderFuncFromCollectionsSchema(schema)

	key1M1, err := collections.EncodeKeyWithPrefix(prefixM1, m1.KeyCodec(), "key_1")
	require.NoError(t, err)
	key2M1, err := collections.EncodeKeyWithPrefix(prefixM1, m1.KeyCodec(), "key_2")
	require.NoError(t, err)
	key1M2, err := collections.EncodeKeyWithPrefix(prefixM2, m2.KeyCodec(), int32(1))
	require.NoError(t, err)
	key2M2, err := collections.EncodeKeyWithPrefix(prefixM2, m2.KeyCodec(), int32(2))
	require.NoError(t, err)

	storeDec1 := dec(kv.Pair{
		Key:   key1M1,
		Value: []byte("value_1"),
	}, kv.Pair{
		Key:   key2M1,
		Value: []byte("value_2"),
	})
	require.Equal(t, "value_1\nvalue_2", storeDec1)

	storeDec2 := dec(kv.Pair{
		Key:   key1M2,
		Value: []byte{0, 0, 0, 1},
	}, kv.Pair{
		Key:   key2M2,
		Value: []byte{0, 0, 0, 2},
	})

	require.Equal(t, "-2147483647\n-2147483646", storeDec2)

	// test key conflict

	require.Panics(t, func() {
		dec(
			kv.Pair{Key: append(prefixM1.Bytes(), 0x1)},
			kv.Pair{Key: append(prefixM2.Bytes(), 0x1)},
		)
	}, "must panic when keys do not have the same prefix")

	require.Panics(t, func() {
		dec(
			kv.Pair{Key: []byte("unknown_1")},
			kv.Pair{Key: []byte("unknown_2")},
		)
	}, "must panic on unknown prefixes")
}

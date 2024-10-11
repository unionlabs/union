package codec_test

import (
	"testing"

	gogotypes "github.com/cosmos/gogoproto/types"
	"github.com/google/go-cmp/cmp"
	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/testing/protocmp"
	"google.golang.org/protobuf/types/known/wrapperspb"

	"cosmossdk.io/collections/colltest"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
)

func TestCollectionsCorrectness(t *testing.T) {
	t.Run("CollValue", func(t *testing.T) {
		cdc := codec.NewProtoCodec(codectypes.NewInterfaceRegistry())
		colltest.TestValueCodec(t, codec.CollValue[gogotypes.UInt64Value](cdc), gogotypes.UInt64Value{
			Value: 500,
		})
	})

	t.Run("CollValueV2", func(t *testing.T) {
		// NOTE: we cannot use colltest.TestValueCodec because protov2 has different
		// compare semantics than protov1. We need to use protocmp.Transform() alongside
		// cmp to ensure equality.
		encoder := codec.CollValueV2[wrapperspb.UInt64Value]()
		value := &wrapperspb.UInt64Value{Value: 500}
		encodedValue, err := encoder.Encode(value)
		require.NoError(t, err)
		decodedValue, err := encoder.Decode(encodedValue)
		require.NoError(t, err)
		require.True(t, cmp.Equal(value, decodedValue, protocmp.Transform()), "encoding and decoding produces different values")

		encodedJSONValue, err := encoder.EncodeJSON(value)
		require.NoError(t, err)
		decodedJSONValue, err := encoder.DecodeJSON(encodedJSONValue)
		require.NoError(t, err)
		require.True(t, cmp.Equal(value, decodedJSONValue, protocmp.Transform()), "encoding and decoding produces different values")
		require.NotEmpty(t, encoder.ValueType())

		_ = encoder.Stringify(value)
	})

	t.Run("BoolValue", func(t *testing.T) {
		colltest.TestValueCodec(t, codec.BoolValue, true)
		colltest.TestValueCodec(t, codec.BoolValue, false)

		// asserts produced bytes are equal
		valueAssert := func(b bool) {
			wantBytes, err := (&gogotypes.BoolValue{Value: b}).Marshal()
			require.NoError(t, err)
			gotBytes, err := codec.BoolValue.Encode(b)
			require.NoError(t, err)
			require.Equal(t, wantBytes, gotBytes)
		}

		valueAssert(true)
		valueAssert(false)
	})

	t.Run("CollInterfaceValue", func(t *testing.T) {
		cdc := codec.NewProtoCodec(codectypes.NewInterfaceRegistry())
		cdc.InterfaceRegistry().RegisterInterface("animal", (*testdata.Animal)(nil), &testdata.Dog{}, &testdata.Cat{})
		valueCodec := codec.CollInterfaceValue[testdata.Animal](cdc)

		colltest.TestValueCodec[testdata.Animal](t, valueCodec, &testdata.Dog{Name: "Doggo"})
		colltest.TestValueCodec[testdata.Animal](t, valueCodec, &testdata.Cat{Moniker: "Kitty"})

		// assert if used with a non interface type it yields a panic.
		require.Panics(t, func() {
			codec.CollInterfaceValue[*testdata.Dog](cdc)
		})
	})
}

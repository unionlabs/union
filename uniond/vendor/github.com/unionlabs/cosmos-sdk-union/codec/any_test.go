package codec_test

import (
	"testing"

	"github.com/cosmos/gogoproto/types/any/test"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/types/module/testutil"
)

func NewTestInterfaceRegistry() codectypes.InterfaceRegistry {
	registry := codectypes.NewInterfaceRegistry()
	registry.RegisterInterface("Animal", (*test.Animal)(nil))
	registry.RegisterImplementations(
		(*test.Animal)(nil),
		&test.Dog{},
		&test.Cat{},
	)
	return registry
}

func TestMarshalAny(t *testing.T) {
	catRegistry := codectypes.NewInterfaceRegistry()
	catRegistry.RegisterImplementations((*test.Animal)(nil), &test.Cat{})

	registry := codectypes.NewInterfaceRegistry()

	cdc := codec.NewProtoCodec(registry)

	kitty := &test.Cat{Moniker: "Kitty"}
	emptyBz, err := cdc.MarshalInterface(kitty)
	require.ErrorContains(t, err, "does not have a registered interface")

	catBz, err := codec.NewProtoCodec(catRegistry).MarshalInterface(kitty)
	require.NoError(t, err)
	require.NotEmpty(t, catBz)

	var animal test.Animal

	// deserializing cat bytes should error in an empty registry
	err = cdc.UnmarshalInterface(catBz, &animal)
	require.ErrorContains(t, err, "no registered implementations of type test.Animal")

	// deserializing an empty byte array will return nil, but no error
	err = cdc.UnmarshalInterface(emptyBz, &animal)
	require.Nil(t, animal)
	require.NoError(t, err)

	// wrong type registration should fail
	registry.RegisterImplementations((*test.Animal)(nil), &test.Dog{})
	err = cdc.UnmarshalInterface(catBz, &animal)
	require.Error(t, err)

	// should pass
	registry = NewTestInterfaceRegistry()
	cdc = codec.NewProtoCodec(registry)
	err = cdc.UnmarshalInterface(catBz, &animal)
	require.NoError(t, err)
	require.Equal(t, kitty, animal)

	// nil should fail
	_ = NewTestInterfaceRegistry()
	err = cdc.UnmarshalInterface(catBz, nil)
	require.Error(t, err)
}

func TestMarshalProtoPubKey(t *testing.T) {
	require := require.New(t)
	ccfg := testutil.MakeTestEncodingConfig(codectestutil.CodecOptions{})
	privKey := ed25519.GenPrivKey()
	pk := privKey.PubKey()

	// **** test JSON serialization ****

	pkAny, err := codectypes.NewAnyWithValue(pk)
	require.NoError(err)
	bz, err := ccfg.Codec.MarshalJSON(pkAny)
	require.NoError(err)

	var pkAny2 codectypes.Any
	err = ccfg.Codec.UnmarshalJSON(bz, &pkAny2)
	require.NoError(err)
	// Before getting a cached value we need to unpack it.
	// Normally this happens in types which implement UnpackInterfaces
	var pkI cryptotypes.PubKey
	err = ccfg.InterfaceRegistry.UnpackAny(&pkAny2, &pkI)
	require.NoError(err)
	pk2 := pkAny2.GetCachedValue().(cryptotypes.PubKey)
	require.True(pk2.Equals(pk))

	// **** test binary serialization ****

	bz, err = ccfg.Codec.Marshal(pkAny)
	require.NoError(err)

	var pkAny3 codectypes.Any
	err = ccfg.Codec.Unmarshal(bz, &pkAny3)
	require.NoError(err)
	err = ccfg.InterfaceRegistry.UnpackAny(&pkAny3, &pkI)
	require.NoError(err)
	pk3 := pkAny3.GetCachedValue().(cryptotypes.PubKey)
	require.True(pk3.Equals(pk))
}

// TestMarshalProtoInterfacePubKey tests PubKey marshaling using (Un)marshalInterface
// helper functions
func TestMarshalProtoInterfacePubKey(t *testing.T) {
	require := require.New(t)
	ccfg := testutil.MakeTestEncodingConfig(codectestutil.CodecOptions{})
	privKey := ed25519.GenPrivKey()
	pk := privKey.PubKey()

	// **** test JSON serialization ****

	bz, err := ccfg.Codec.MarshalInterfaceJSON(pk)
	require.NoError(err)

	var pk3 cryptotypes.PubKey
	err = ccfg.Codec.UnmarshalInterfaceJSON(bz, &pk3)
	require.NoError(err)
	require.True(pk3.Equals(pk))

	// ** Check unmarshal using JSONCodec **
	// Unpacking won't work straightforward s Any type
	// Any can't implement UnpackInterfacesMessage interface. So Any is not
	// automatically unpacked and we won't get a value.
	var pkAny codectypes.Any
	err = ccfg.Codec.UnmarshalJSON(bz, &pkAny)
	require.NoError(err)
	ifc := pkAny.GetCachedValue()
	require.Nil(ifc)

	// **** test binary serialization ****

	bz, err = ccfg.Codec.MarshalInterface(pk)
	require.NoError(err)

	var pk2 cryptotypes.PubKey
	err = ccfg.Codec.UnmarshalInterface(bz, &pk2)
	require.NoError(err)
	require.True(pk2.Equals(pk))
}

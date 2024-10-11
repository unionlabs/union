package codec_test

import (
	"testing"

	"github.com/cosmos/gogoproto/proto"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
)

type interfaceMarshaler struct {
	marshal   func(i proto.Message) ([]byte, error)
	unmarshal func(bz []byte, ptr interface{}) error
}

func testInterfaceMarshaling(require *require.Assertions, cdc interfaceMarshaler, isAminoBin bool) {
	_, err := cdc.marshal(nil)
	require.Error(err, "can't marshal a nil value")

	dog := &testdata.Dog{Name: "rufus"}
	var dogI testdata.Animal = dog
	bz, err := cdc.marshal(dogI)
	require.NoError(err)

	var animal testdata.Animal
	if isAminoBin {
		require.PanicsWithValue("Unmarshal expects a pointer", func() {
			_ = cdc.unmarshal(bz, animal)
		})
	} else {
		err = cdc.unmarshal(bz, animal)
		require.Error(err)
		require.Contains(err.Error(), "expects a pointer")
	}
	require.NoError(cdc.unmarshal(bz, &animal))
	require.Equal(dog, animal)

	// Amino doesn't wrap into Any, so it doesn't need to register self type
	if isAminoBin {
		var dog2 testdata.Dog
		require.NoError(cdc.unmarshal(bz, &dog2))
		require.Equal(*dog, dog2)
	}

	var cat testdata.Cat
	require.Error(cdc.unmarshal(bz, &cat))
}

type mustMarshaler struct {
	marshal       func(i proto.Message) ([]byte, error)
	mustMarshal   func(i proto.Message) []byte
	unmarshal     func(bz []byte, ptr proto.Message) error
	mustUnmarshal func(bz []byte, ptr proto.Message)
}

type testCase struct {
	name         string
	input        proto.Message
	recv         proto.Message
	marshalErr   bool
	unmarshalErr bool
}

func testMarshalingTestCase(require *require.Assertions, tc testCase, m mustMarshaler) {
	bz, err := m.marshal(tc.input)
	if tc.marshalErr {
		require.Error(err)
		require.Panics(func() { m.mustMarshal(tc.input) })
	} else {
		var bz2 []byte
		require.NoError(err)
		require.NotPanics(func() { bz2 = m.mustMarshal(tc.input) })
		require.Equal(bz, bz2)

		err := m.unmarshal(bz, tc.recv)
		if tc.unmarshalErr {
			require.Error(err)
			require.Panics(func() { m.mustUnmarshal(bz, tc.recv) })
		} else {
			require.NoError(err)
			require.NotPanics(func() { m.mustUnmarshal(bz, tc.recv) })
			require.Equal(tc.input, tc.recv)
		}
	}
}

func testMarshaling(t *testing.T, cdc interface {
	codec.BinaryCodec
	codec.JSONCodec
},
) {
	t.Helper()
	any, err := types.NewAnyWithValue(&testdata.Dog{Name: "rufus"})
	require.NoError(t, err)

	testCases := []testCase{
		{
			"valid encoding and decoding",
			&testdata.Dog{Name: "rufus"},
			&testdata.Dog{},
			false,
			false,
		}, {
			"invalid decode type",
			&testdata.Dog{Name: "rufus"},
			&testdata.Cat{},
			false,
			true,
		},
	}
	if _, ok := cdc.(*codec.AminoCodec); ok {
		testCases = append(testCases, testCase{
			"any marshaling",
			&testdata.HasAnimal{Animal: any},
			&testdata.HasAnimal{Animal: any},
			false,
			false,
		})
	}

	for _, tc := range testCases {

		m1 := mustMarshaler{cdc.Marshal, cdc.MustMarshal, cdc.Unmarshal, cdc.MustUnmarshal}
		m2 := mustMarshaler{cdc.MarshalLengthPrefixed, cdc.MustMarshalLengthPrefixed, cdc.UnmarshalLengthPrefixed, cdc.MustUnmarshalLengthPrefixed}
		m3 := mustMarshaler{
			func(i proto.Message) ([]byte, error) { return cdc.MarshalJSON(i) },
			func(i proto.Message) []byte { return cdc.MustMarshalJSON(i) },
			func(bz []byte, ptr proto.Message) error { return cdc.UnmarshalJSON(bz, ptr) },
			func(bz []byte, ptr proto.Message) { cdc.MustUnmarshalJSON(bz, ptr) },
		}

		t.Run(tc.name+"_BinaryBare",
			func(t *testing.T) { testMarshalingTestCase(require.New(t), tc, m1) })
		t.Run(tc.name+"_BinaryLengthPrefixed",
			func(t *testing.T) { testMarshalingTestCase(require.New(t), tc, m2) })
		t.Run(tc.name+"_JSON",
			func(t *testing.T) { testMarshalingTestCase(require.New(t), tc, m3) })
	}
}

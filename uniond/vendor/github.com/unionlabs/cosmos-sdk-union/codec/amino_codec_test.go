package codec_test

import (
	"bytes"
	"errors"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
)

func createTestCodec() *codec.LegacyAmino {
	cdc := codec.NewLegacyAmino()
	cdc.RegisterInterface((*testdata.Animal)(nil), nil)
	// NOTE: since we unmarshal interface using pointers, we need to register a pointer
	// types here.
	cdc.RegisterConcrete(&testdata.Dog{}, "testdata/Dog")
	cdc.RegisterConcrete(&testdata.Cat{}, "testdata/Cat")

	return cdc
}

func TestAminoMarsharlInterface(t *testing.T) {
	cdc := codec.NewAminoCodec(createTestCodec())
	m := interfaceMarshaler{cdc.MarshalInterface, cdc.UnmarshalInterface}
	testInterfaceMarshaling(require.New(t), m, true)

	m = interfaceMarshaler{cdc.MarshalInterfaceJSON, cdc.UnmarshalInterfaceJSON}
	testInterfaceMarshaling(require.New(t), m, false)
}

func TestAminoCodec(t *testing.T) {
	testMarshaling(t, codec.NewAminoCodec(createTestCodec()))
}

func TestAminoCodecMarshalJSONIndent(t *testing.T) {
	any, err := types.NewAnyWithValue(&testdata.Dog{Name: "rufus"})
	require.NoError(t, err)

	testCases := []struct {
		name       string
		input      interface{}
		marshalErr bool
		wantJSON   string
	}{
		{
			name:  "valid encoding and decoding",
			input: &testdata.Dog{Name: "rufus"},
			wantJSON: `{
  "type": "testdata/Dog",
  "value": {
    "name": "rufus"
  }
}`,
		},
		{
			name:  "any marshaling",
			input: &testdata.HasAnimal{Animal: any},
			wantJSON: `{
  "animal": {
    "type": "testdata/Dog",
    "value": {
      "name": "rufus"
    }
  }
}`,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			cdc := codec.NewAminoCodec(createTestCodec())
			bz, err := cdc.MarshalJSONIndent(tc.input, "", "  ")

			if tc.marshalErr {
				require.Error(t, err)
				require.Panics(t, func() { codec.MustMarshalJSONIndent(cdc.LegacyAmino, tc.input) })
				return
			}

			// Otherwise these are expected to pass.
			require.NoError(t, err)
			require.Equal(t, bz, []byte(tc.wantJSON))

			var bz2 []byte
			require.NotPanics(t, func() { bz2 = codec.MustMarshalJSONIndent(cdc.LegacyAmino, tc.input) })
			require.Equal(t, bz2, []byte(tc.wantJSON))
		})
	}
}

func TestAminoCodecPrintTypes(t *testing.T) {
	cdc := codec.NewAminoCodec(createTestCodec())
	buf := new(bytes.Buffer)
	require.NoError(t, cdc.PrintTypes(buf))
	lines := bytes.Split(buf.Bytes(), []byte("\n"))
	require.True(t, len(lines) > 1)
	wantHeader := "| Type | Name | Prefix | Length | Notes |"
	require.Equal(t, lines[0], []byte(wantHeader))

	// Expecting the types to be listed in the order that they were registered.
	require.True(t, bytes.HasPrefix(lines[2], []byte("| Dog | testdata/Dog |")))
	require.True(t, bytes.HasPrefix(lines[3], []byte("| Cat | testdata/Cat |")))
}

func TestAminoCodecUnpackAnyFails(t *testing.T) {
	cdc := codec.NewAminoCodec(createTestCodec())
	err := cdc.UnpackAny(new(types.Any), &testdata.Cat{})
	require.Error(t, err)
	require.Equal(t, err, errors.New("AminoCodec can't handle unpack protobuf Any's"))
}

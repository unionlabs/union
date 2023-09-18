package codec_test

import (
	"bytes"
	"errors"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	"github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/x/auth"
	"github.com/cosmos/cosmos-sdk/x/auth/client/cli"
	"github.com/cosmos/cosmos-sdk/x/auth/migrations/legacytx"
	"github.com/cosmos/cosmos-sdk/x/staking"
)

func createTestCodec() *codec.LegacyAmino {
	cdc := codec.NewLegacyAmino()
	cdc.RegisterInterface((*testdata.Animal)(nil), nil)
	// NOTE: since we unmarshal interface using pointers, we need to register a pointer
	// types here.
	cdc.RegisterConcrete(&testdata.Dog{}, "testdata/Dog", nil)
	cdc.RegisterConcrete(&testdata.Cat{}, "testdata/Cat", nil)

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
		tc := tc

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

func TestAminoCodecFullDecodeAndEncode(t *testing.T) {
	// This tx comes from https://github.com/cosmos/cosmos-sdk/issues/8117.
	txSigned := `{"type":"cosmos-sdk/StdTx","value":{"msg":[{"type":"cosmos-sdk/MsgCreateValidator","value":{"description":{"moniker":"fulltest","identity":"satoshi","website":"example.com","details":"example inc"},"commission":{"rate":"0.500000000000000000","max_rate":"1.000000000000000000","max_change_rate":"0.200000000000000000"},"min_self_delegation":"1000000","delegator_address":"cosmos14pt0q5cwf38zt08uu0n6yrstf3rndzr5057jys","validator_address":"cosmosvaloper14pt0q5cwf38zt08uu0n6yrstf3rndzr52q28gr","pubkey":{"type":"tendermint/PubKeyEd25519","value":"CYrOiM3HtS7uv1B1OAkknZnFYSRpQYSYII8AtMMtev0="},"value":{"denom":"umuon","amount":"700000000"}}}],"fee":{"amount":[{"denom":"umuon","amount":"6000"}],"gas":"160000"},"signatures":[{"pub_key":{"type":"tendermint/PubKeySecp256k1","value":"AwAOXeWgNf1FjMaayrSnrOOKz+Fivr6DiI/i0x0sZCHw"},"signature":"RcnfS/u2yl7uIShTrSUlDWvsXo2p2dYu6WJC8VDVHMBLEQZWc8bsINSCjOnlsIVkUNNe1q/WCA9n3Gy1+0zhYA=="}],"memo":"","timeout_height":"0"}}`
	legacyCdc := testutil.MakeTestEncodingConfig(staking.AppModuleBasic{}, auth.AppModuleBasic{}).Amino
	var tx legacytx.StdTx
	err := legacyCdc.UnmarshalJSON([]byte(txSigned), &tx)
	require.NoError(t, err)

	// Marshalling/unmarshalling the tx should work.
	marshaledTx, err := legacyCdc.MarshalJSON(tx)
	require.NoError(t, err)
	require.Equal(t, txSigned, string(marshaledTx))

	// Marshalling/unmarshalling the tx wrapped in a struct should work.
	txRequest := &cli.BroadcastReq{
		Mode: flags.BroadcastSync,
		Tx:   tx,
	}
	_, err = legacyCdc.MarshalJSON(txRequest)
	require.NoError(t, err)
}

package tx

import (
	"encoding/binary"
	"fmt"
	"math"
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/encoding/protowire"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/tx"
	signingtypes "github.com/cosmos/cosmos-sdk/types/tx/signing"
	"github.com/cosmos/cosmos-sdk/x/auth/signing"
)

func TestDefaultTxDecoderError(t *testing.T) {
	registry := codectypes.NewInterfaceRegistry()
	cdc := codec.NewProtoCodec(registry)
	encoder := DefaultTxEncoder()
	decoder := DefaultTxDecoder(cdc)

	builder := newBuilder(nil)
	err := builder.SetMsgs(testdata.NewTestMsg())
	require.NoError(t, err)

	txBz, err := encoder(builder.GetTx())
	require.NoError(t, err)

	_, err = decoder(txBz)
	require.EqualError(t, err, "unable to resolve type URL /testpb.TestMsg: tx parse error")

	testdata.RegisterInterfaces(registry)
	_, err = decoder(txBz)
	require.NoError(t, err)
}

func TestUnknownFields(t *testing.T) {
	registry := codectypes.NewInterfaceRegistry()
	cdc := codec.NewProtoCodec(registry)
	decoder := DefaultTxDecoder(cdc)

	tests := []struct {
		name           string
		body           *testdata.TestUpdatedTxBody
		authInfo       *testdata.TestUpdatedAuthInfo
		shouldErr      bool
		shouldAminoErr string
	}{
		{
			name: "no new fields should pass",
			body: &testdata.TestUpdatedTxBody{
				Memo: "foo",
			},
			authInfo:  &testdata.TestUpdatedAuthInfo{},
			shouldErr: false,
		},
		{
			name: "non-critical fields in TxBody should not error on decode, but should error with amino",
			body: &testdata.TestUpdatedTxBody{
				Memo:                         "foo",
				SomeNewFieldNonCriticalField: "blah",
			},
			authInfo:       &testdata.TestUpdatedAuthInfo{},
			shouldErr:      false,
			shouldAminoErr: fmt.Sprintf("%s: %s", aminoNonCriticalFieldsError, sdkerrors.ErrInvalidRequest.Error()),
		},
		{
			name: "critical fields in TxBody should error on decode",
			body: &testdata.TestUpdatedTxBody{
				Memo:         "foo",
				SomeNewField: 10,
			},
			authInfo:  &testdata.TestUpdatedAuthInfo{},
			shouldErr: true,
		},
		{
			name: "critical fields in AuthInfo should error on decode",
			body: &testdata.TestUpdatedTxBody{
				Memo: "foo",
			},
			authInfo: &testdata.TestUpdatedAuthInfo{
				NewField_3: []byte("xyz"),
			},
			shouldErr: true,
		},
		{
			name: "non-critical fields in AuthInfo should error on decode",
			body: &testdata.TestUpdatedTxBody{
				Memo: "foo",
			},
			authInfo: &testdata.TestUpdatedAuthInfo{
				NewField_1024: []byte("xyz"),
			},
			shouldErr: true,
		},
	}

	for _, tt := range tests {
		tt := tt
		t.Run(tt.name, func(t *testing.T) {
			bodyBz, err := tt.body.Marshal()
			require.NoError(t, err)

			authInfoBz, err := tt.authInfo.Marshal()
			require.NoError(t, err)

			txRaw := &tx.TxRaw{
				BodyBytes:     bodyBz,
				AuthInfoBytes: authInfoBz,
			}
			txBz, err := txRaw.Marshal()
			require.NoError(t, err)

			_, err = decoder(txBz)
			if tt.shouldErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}

			if tt.shouldAminoErr != "" {
				handler := signModeLegacyAminoJSONHandler{}
				decoder := DefaultTxDecoder(codec.NewProtoCodec(codectypes.NewInterfaceRegistry()))
				theTx, err := decoder(txBz)
				require.NoError(t, err)
				_, err = handler.GetSignBytes(signingtypes.SignMode_SIGN_MODE_LEGACY_AMINO_JSON, signing.SignerData{}, theTx)
				require.EqualError(t, err, tt.shouldAminoErr)
			}
		})
	}

	t.Log("test TxRaw no new fields, should succeed")
	txRaw := &testdata.TestUpdatedTxRaw{}
	txBz, err := txRaw.Marshal()
	require.NoError(t, err)
	_, err = decoder(txBz)
	require.NoError(t, err)

	t.Log("new field in TxRaw should fail")
	txRaw = &testdata.TestUpdatedTxRaw{
		NewField_5: []byte("abc"),
	}
	txBz, err = txRaw.Marshal()
	require.NoError(t, err)
	_, err = decoder(txBz)
	require.Error(t, err)

	//
	t.Log("new \"non-critical\" field in TxRaw should fail")
	txRaw = &testdata.TestUpdatedTxRaw{
		NewField_1024: []byte("abc"),
	}
	txBz, err = txRaw.Marshal()
	require.NoError(t, err)
	_, err = decoder(txBz)
	require.Error(t, err)
}

func TestRejectNonADR027(t *testing.T) {
	registry := codectypes.NewInterfaceRegistry()
	cdc := codec.NewProtoCodec(registry)
	decoder := DefaultTxDecoder(cdc)

	body := &testdata.TestUpdatedTxBody{Memo: "AAA"} // Look for "65 65 65" when debugging the bytes stream.
	bodyBz, err := body.Marshal()
	require.NoError(t, err)
	authInfo := &testdata.TestUpdatedAuthInfo{Fee: &tx.Fee{GasLimit: 127}} // Look for "127" when debugging the bytes stream.
	authInfoBz, err := authInfo.Marshal()
	txRaw := &tx.TxRaw{
		BodyBytes:     bodyBz,
		AuthInfoBytes: authInfoBz,
		Signatures:    [][]byte{{41}, {42}, {43}}, // Look for "42" when debugging the bytes stream.
	}

	// We know these bytes are ADR-027-compliant.
	txBz, err := txRaw.Marshal()

	// From the `txBz`, we extract the 3 components:
	// bodyBz, authInfoBz, sigsBz.
	// In our tests, we will try to decode txs with those 3 components in all
	// possible orders.
	//
	// Consume "BodyBytes" field.
	_, _, m := protowire.ConsumeField(txBz)
	bodyBz = append([]byte{}, txBz[:m]...)
	txBz = txBz[m:] // Skip over "BodyBytes" bytes.
	// Consume "AuthInfoBytes" field.
	_, _, m = protowire.ConsumeField(txBz)
	authInfoBz = append([]byte{}, txBz[:m]...)
	txBz = txBz[m:] // Skip over "AuthInfoBytes" bytes.
	// Consume "Signature" field, it's the remaining bytes.
	sigsBz := append([]byte{}, txBz...)

	// bodyBz's length prefix is 5, with `5` as varint encoding. We also try a
	// longer varint encoding for 5: `133 00`.
	longVarintBodyBz := append(append([]byte{bodyBz[0]}, byte(133), byte(0o0)), bodyBz[2:]...)

	tests := []struct {
		name      string
		txBz      []byte
		shouldErr bool
	}{
		{
			"authInfo, body, sigs",
			append(append(authInfoBz, bodyBz...), sigsBz...),
			true,
		},
		{
			"authInfo, sigs, body",
			append(append(authInfoBz, sigsBz...), bodyBz...),
			true,
		},
		{
			"sigs, body, authInfo",
			append(append(sigsBz, bodyBz...), authInfoBz...),
			true,
		},
		{
			"sigs, authInfo, body",
			append(append(sigsBz, authInfoBz...), bodyBz...),
			true,
		},
		{
			"body, sigs, authInfo",
			append(append(bodyBz, sigsBz...), authInfoBz...),
			true,
		},
		{
			"body, authInfo, sigs (valid txRaw)",
			append(append(bodyBz, authInfoBz...), sigsBz...),
			false,
		},
		{
			"longer varint than needed",
			append(append(longVarintBodyBz, authInfoBz...), sigsBz...),
			true,
		},
	}

	for _, tt := range tests {
		tt := tt
		t.Run(tt.name, func(t *testing.T) {
			_, err = decoder(tt.txBz)
			if tt.shouldErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestVarintMinLength(t *testing.T) {
	tests := []struct {
		n uint64
	}{
		{1<<7 - 1},
		{1 << 7},
		{1<<14 - 1},
		{1 << 14},
		{1<<21 - 1},
		{1 << 21},
		{1<<28 - 1},
		{1 << 28},
		{1<<35 - 1},
		{1 << 35},
		{1<<42 - 1},
		{1 << 42},
		{1<<49 - 1},
		{1 << 49},
		{1<<56 - 1},
		{1 << 56},
		{1<<63 - 1},
		{1 << 63},
		{math.MaxUint64},
	}

	for _, tt := range tests {
		tt := tt
		t.Run(fmt.Sprintf("test %d", tt.n), func(t *testing.T) {
			l1 := varintMinLength(tt.n)
			buf := make([]byte, binary.MaxVarintLen64)
			l2 := binary.PutUvarint(buf, tt.n)
			require.Equal(t, l2, l1)
		})
	}
}

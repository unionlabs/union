package tx_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client/tx"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/testutil"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	"github.com/cosmos/cosmos-sdk/testutil/x/counter"
	countertypes "github.com/cosmos/cosmos-sdk/testutil/x/counter/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	typestx "github.com/cosmos/cosmos-sdk/types/tx"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
)

const (
	memo          = "waboom"
	timeoutHeight = uint64(5)
)

var (
	_, pub1, addr1 = testdata.KeyTestPubAddr()
	addr1Str, _    = testutil.CodecOptions{}.GetAddressCodec().BytesToString(addr1)
	rawSig         = []byte("dummy")
	msg1           = &countertypes.MsgIncreaseCounter{Signer: addr1Str, Count: 1}

	chainID = "test-chain"
)

func TestAuxTxBuilder(t *testing.T) {
	counterModule := counter.AppModule{}
	cdc := moduletestutil.MakeTestEncodingConfig(testutil.CodecOptions{}, counterModule).Codec
	reg := codectypes.NewInterfaceRegistry()

	testdata.RegisterInterfaces(reg)
	// required for test case: "GetAuxSignerData works for DIRECT_AUX"
	counterModule.RegisterInterfaces(reg)

	var b tx.AuxTxBuilder

	testcases := []struct {
		name      string
		malleate  func() error
		expErr    bool
		expErrStr string
	}{
		{
			"cannot set SIGN_MODE_DIRECT",
			func() error {
				return b.SetSignMode(signing.SignMode_SIGN_MODE_DIRECT)
			},
			true, "AuxTxBuilder can only sign with SIGN_MODE_DIRECT_AUX or SIGN_MODE_LEGACY_AMINO_JSON",
		},
		{
			"cannot set invalid pubkey",
			func() error {
				return b.SetPubKey(cryptotypes.PubKey(nil))
			},
			true, "failed packing protobuf message to Any",
		},
		{
			"cannot set invalid Msg",
			func() error {
				return b.SetMsgs(sdk.Msg(nil))
			},
			true, "failed packing protobuf message to Any",
		},
		{
			"GetSignBytes body should not be nil",
			func() error {
				_, err := b.GetSignBytes()
				return err
			},
			true, "aux tx is nil, call setters on AuxTxBuilder first",
		},
		{
			"GetSignBytes pubkey should not be nil",
			func() error {
				require.NoError(t, b.SetMsgs(msg1))

				_, err := b.GetSignBytes()
				return err
			},
			true, "public key cannot be empty: invalid pubkey",
		},
		{
			"GetSignBytes invalid sign mode",
			func() error {
				require.NoError(t, b.SetMsgs(msg1))
				require.NoError(t, b.SetPubKey(pub1))

				_, err := b.GetSignBytes()
				return err
			},
			true, "got unknown sign mode SIGN_MODE_UNSPECIFIED",
		},
		{
			"GetSignBytes works for DIRECT_AUX",
			func() error {
				require.NoError(t, b.SetMsgs(msg1))
				require.NoError(t, b.SetPubKey(pub1))
				require.NoError(t, b.SetSignMode(signing.SignMode_SIGN_MODE_DIRECT_AUX))

				_, err := b.GetSignBytes()
				return err
			},
			false, "",
		},
		{
			"GetAuxSignerData address should not be empty",
			func() error {
				require.NoError(t, b.SetMsgs(msg1))
				require.NoError(t, b.SetPubKey(pub1))
				require.NoError(t, b.SetSignMode(signing.SignMode_SIGN_MODE_DIRECT_AUX))

				_, err := b.GetSignBytes()
				require.NoError(t, err)

				_, err = b.GetAuxSignerData()
				return err
			},
			true, "address cannot be empty: invalid request",
		},
		{
			"GetAuxSignerData signature should not be empty",
			func() error {
				require.NoError(t, b.SetMsgs(msg1))
				require.NoError(t, b.SetPubKey(pub1))
				b.SetAddress(addr1Str)
				require.NoError(t, b.SetSignMode(signing.SignMode_SIGN_MODE_DIRECT_AUX))

				_, err := b.GetSignBytes()
				require.NoError(t, err)

				_, err = b.GetAuxSignerData()
				return err
			},
			true, "signature cannot be empty: no signatures supplied",
		},
		{
			"GetAuxSignerData works for DIRECT_AUX",
			func() error {
				b.SetAccountNumber(1)
				b.SetSequence(2)
				b.SetTimeoutHeight(timeoutHeight)
				b.SetMemo(memo)
				b.SetChainID(chainID)
				require.NoError(t, b.SetMsgs(msg1))
				require.NoError(t, b.SetPubKey(pub1))
				b.SetAddress(addr1Str)
				err := b.SetSignMode(signing.SignMode_SIGN_MODE_DIRECT_AUX)
				require.NoError(t, err)

				_, err = b.GetSignBytes()
				require.NoError(t, err)
				b.SetSignature(rawSig)

				auxSignerData, err := b.GetAuxSignerData()

				// Make sure auxSignerData is correctly populated
				checkCorrectData(t, cdc, auxSignerData, signing.SignMode_SIGN_MODE_DIRECT_AUX)

				return err
			},
			false, "",
		},
		{
			"GetSignBytes works for LEGACY_AMINO_JSON",
			func() error {
				require.NoError(t, b.SetMsgs(msg1))
				require.NoError(t, b.SetPubKey(pub1))
				b.SetAddress(addr1Str)
				err := b.SetSignMode(signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON)
				require.NoError(t, err)

				_, err = b.GetSignBytes()
				return err
			},
			false, "",
		},
		{
			"GetAuxSignerData works for LEGACY_AMINO_JSON",
			func() error {
				b.SetAccountNumber(1)
				b.SetSequence(2)
				b.SetTimeoutHeight(timeoutHeight)
				b.SetMemo(memo)
				b.SetChainID(chainID)
				require.NoError(t, b.SetMsgs(msg1))
				require.NoError(t, b.SetPubKey(pub1))
				b.SetAddress(addr1Str)
				err := b.SetSignMode(signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON)
				require.NoError(t, err)

				_, err = b.GetSignBytes()
				require.NoError(t, err)
				b.SetSignature(rawSig)

				auxSignerData, err := b.GetAuxSignerData()

				// Make sure auxSignerData is correctly populated
				checkCorrectData(t, cdc, auxSignerData, signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON)

				return err
			},
			false, "",
		},
	}

	for _, tc := range testcases {
		t.Run(tc.name, func(t *testing.T) {
			b = tx.NewAuxTxBuilder()
			err := tc.malleate()

			if tc.expErr {
				require.Error(t, err)
				require.Contains(t, err.Error(), tc.expErrStr)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

// checkCorrectData that the auxSignerData's content matches the inputs we gave.
func checkCorrectData(t *testing.T, cdc codec.Codec, auxSignerData typestx.AuxSignerData, signMode signing.SignMode) {
	t.Helper()
	pkAny, err := codectypes.NewAnyWithValue(pub1)
	require.NoError(t, err)
	msgAny, err := codectypes.NewAnyWithValue(msg1)
	require.NoError(t, err)

	var body typestx.TxBody
	err = cdc.Unmarshal(auxSignerData.SignDoc.BodyBytes, &body)
	require.NoError(t, err)

	require.Equal(t, uint64(1), auxSignerData.SignDoc.AccountNumber)
	require.Equal(t, uint64(2), auxSignerData.SignDoc.Sequence)
	require.Equal(t, timeoutHeight, body.TimeoutHeight)
	require.Equal(t, memo, body.Memo)
	require.Equal(t, chainID, auxSignerData.SignDoc.ChainId)
	require.Equal(t, msgAny, body.GetMessages()[0])
	require.Equal(t, pkAny, auxSignerData.SignDoc.PublicKey)
	require.Equal(t, signMode, auxSignerData.Mode)
	require.Equal(t, rawSig, auxSignerData.Sig)
}

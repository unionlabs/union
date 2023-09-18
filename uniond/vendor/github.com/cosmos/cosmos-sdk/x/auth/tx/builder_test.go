package tx

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/legacy"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	txtypes "github.com/cosmos/cosmos-sdk/types/tx"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
)

func TestTxBuilder(t *testing.T) {
	_, pubkey, addr := testdata.KeyTestPubAddr()

	marshaler := codec.NewProtoCodec(codectypes.NewInterfaceRegistry())
	txBuilder := newBuilder(nil)

	memo := "sometestmemo"
	msgs := []sdk.Msg{testdata.NewTestMsg(addr)}
	accSeq := uint64(2) // Arbitrary account sequence
	any, err := codectypes.NewAnyWithValue(pubkey)
	require.NoError(t, err)

	var signerInfo []*txtypes.SignerInfo
	signerInfo = append(signerInfo, &txtypes.SignerInfo{
		PublicKey: any,
		ModeInfo: &txtypes.ModeInfo{
			Sum: &txtypes.ModeInfo_Single_{
				Single: &txtypes.ModeInfo_Single{
					Mode: signing.SignMode_SIGN_MODE_DIRECT,
				},
			},
		},
		Sequence: accSeq,
	})

	var sig signing.SignatureV2 = signing.SignatureV2{
		PubKey: pubkey,
		Data: &signing.SingleSignatureData{
			SignMode:  signing.SignMode_SIGN_MODE_DIRECT,
			Signature: legacy.Cdc.MustMarshal(pubkey),
		},
		Sequence: accSeq,
	}

	fee := txtypes.Fee{Amount: sdk.NewCoins(sdk.NewInt64Coin("atom", 150)), GasLimit: 20000}

	t.Log("verify that authInfo bytes encoded with DefaultTxEncoder and decoded with DefaultTxDecoder can be retrieved from getAuthInfoBytes")
	authInfo := &txtypes.AuthInfo{
		Fee:         &fee,
		SignerInfos: signerInfo,
	}

	authInfoBytes := marshaler.MustMarshal(authInfo)

	require.NotEmpty(t, authInfoBytes)

	t.Log("verify that body bytes encoded with DefaultTxEncoder and decoded with DefaultTxDecoder can be retrieved from getBodyBytes")
	anys := make([]*codectypes.Any, len(msgs))

	for i, msg := range msgs {
		var err error
		anys[i], err = codectypes.NewAnyWithValue(msg)
		if err != nil {
			panic(err)
		}
	}

	txBody := &txtypes.TxBody{
		Memo:     memo,
		Messages: anys,
	}
	bodyBytes := marshaler.MustMarshal(txBody)
	require.NotEmpty(t, bodyBytes)
	require.Empty(t, txBuilder.getBodyBytes())

	t.Log("verify that calling the SetMsgs, SetMemo results in the correct getBodyBytes")
	require.NotEqual(t, bodyBytes, txBuilder.getBodyBytes())
	err = txBuilder.SetMsgs(msgs...)
	require.NoError(t, err)
	require.NotEqual(t, bodyBytes, txBuilder.getBodyBytes())
	txBuilder.SetMemo(memo)
	require.Equal(t, bodyBytes, txBuilder.getBodyBytes())
	require.Equal(t, len(msgs), len(txBuilder.GetMsgs()))
	pks, err := txBuilder.GetPubKeys()
	require.NoError(t, err)
	require.Empty(t, pks)

	t.Log("verify that updated AuthInfo  results in the correct getAuthInfoBytes and GetPubKeys")
	require.NotEqual(t, authInfoBytes, txBuilder.getAuthInfoBytes())
	txBuilder.SetFeeAmount(fee.Amount)
	require.NotEqual(t, authInfoBytes, txBuilder.getAuthInfoBytes())
	txBuilder.SetGasLimit(fee.GasLimit)
	require.NotEqual(t, authInfoBytes, txBuilder.getAuthInfoBytes())
	err = txBuilder.SetSignatures(sig)
	require.NoError(t, err)

	// once fee, gas and signerInfos are all set, AuthInfo bytes should match
	require.Equal(t, authInfoBytes, txBuilder.getAuthInfoBytes())

	require.Equal(t, len(msgs), len(txBuilder.GetMsgs()))
	pks, err = txBuilder.GetPubKeys()
	require.NoError(t, err)
	require.Equal(t, 1, len(pks))
	require.True(t, pubkey.Equals(pks[0]))

	any, err = codectypes.NewAnyWithValue(testdata.NewTestMsg())
	require.NoError(t, err)
	txBuilder.SetExtensionOptions(any)
	require.Equal(t, []*codectypes.Any{any}, txBuilder.GetExtensionOptions())
	txBuilder.SetNonCriticalExtensionOptions(any)
	require.Equal(t, []*codectypes.Any{any}, txBuilder.GetNonCriticalExtensionOptions())

	txBuilder = &wrapper{}
	require.NotPanics(t, func() {
		_ = txBuilder.GetMsgs()
	})
}

func TestBuilderValidateBasic(t *testing.T) {
	// keys and addresses
	_, pubKey1, addr1 := testdata.KeyTestPubAddr()
	_, pubKey2, addr2 := testdata.KeyTestPubAddr()

	// msg and signatures
	msg1 := testdata.NewTestMsg(addr1, addr2)
	feeAmount := testdata.NewTestFeeAmount()
	msgs := []sdk.Msg{msg1}

	// require to fail validation upon invalid fee
	badFeeAmount := testdata.NewTestFeeAmount()
	badFeeAmount[0].Amount = sdk.NewInt(-5)
	txBuilder := newBuilder(nil)

	var sig1, sig2 signing.SignatureV2
	sig1 = signing.SignatureV2{
		PubKey: pubKey1,
		Data: &signing.SingleSignatureData{
			SignMode:  signing.SignMode_SIGN_MODE_DIRECT,
			Signature: legacy.Cdc.MustMarshal(pubKey1),
		},
		Sequence: 0, // Arbitrary account sequence
	}

	sig2 = signing.SignatureV2{
		PubKey: pubKey2,
		Data: &signing.SingleSignatureData{
			SignMode:  signing.SignMode_SIGN_MODE_DIRECT,
			Signature: legacy.Cdc.MustMarshal(pubKey2),
		},
		Sequence: 0, // Arbitrary account sequence
	}

	err := txBuilder.SetMsgs(msgs...)
	require.NoError(t, err)
	txBuilder.SetGasLimit(200000)
	err = txBuilder.SetSignatures(sig1, sig2)
	require.NoError(t, err)
	txBuilder.SetFeeAmount(badFeeAmount)
	err = txBuilder.ValidateBasic()
	require.Error(t, err)
	_, code, _ := sdkerrors.ABCIInfo(err, false)
	require.Equal(t, sdkerrors.ErrInsufficientFee.ABCICode(), code)

	// require to fail validation when no signatures exist
	err = txBuilder.SetSignatures()
	require.NoError(t, err)
	txBuilder.SetFeeAmount(feeAmount)
	err = txBuilder.ValidateBasic()
	require.Error(t, err)
	_, code, _ = sdkerrors.ABCIInfo(err, false)
	require.Equal(t, sdkerrors.ErrNoSignatures.ABCICode(), code)

	// require to fail with nil values for tx, authinfo
	err = txBuilder.SetMsgs(msgs...)
	require.NoError(t, err)
	err = txBuilder.ValidateBasic()
	require.Error(t, err)

	// require to fail validation when signatures do not match expected signers
	err = txBuilder.SetSignatures(sig1)
	require.NoError(t, err)

	err = txBuilder.ValidateBasic()
	require.Error(t, err)
	_, code, _ = sdkerrors.ABCIInfo(err, false)
	require.Equal(t, sdkerrors.ErrUnauthorized.ABCICode(), code)

	require.Error(t, err)
	txBuilder.SetFeeAmount(feeAmount)
	err = txBuilder.SetSignatures(sig1, sig2)
	require.NoError(t, err)
	err = txBuilder.ValidateBasic()
	require.NoError(t, err)

	// gas limit too high
	txBuilder.SetGasLimit(txtypes.MaxGasWanted + 1)
	err = txBuilder.ValidateBasic()
	require.Error(t, err)
	txBuilder.SetGasLimit(txtypes.MaxGasWanted - 1)
	err = txBuilder.ValidateBasic()
	require.NoError(t, err)

	// bad builder structs

	// missing body
	body := txBuilder.tx.Body
	txBuilder.tx.Body = nil
	err = txBuilder.ValidateBasic()
	require.Error(t, err)
	txBuilder.tx.Body = body
	err = txBuilder.ValidateBasic()
	require.NoError(t, err)

	// missing fee
	f := txBuilder.tx.AuthInfo.Fee
	txBuilder.tx.AuthInfo.Fee = nil
	err = txBuilder.ValidateBasic()
	require.Error(t, err)
	txBuilder.tx.AuthInfo.Fee = f
	err = txBuilder.ValidateBasic()
	require.NoError(t, err)

	// missing AuthInfo
	authInfo := txBuilder.tx.AuthInfo
	txBuilder.tx.AuthInfo = nil
	err = txBuilder.ValidateBasic()
	require.Error(t, err)
	txBuilder.tx.AuthInfo = authInfo
	err = txBuilder.ValidateBasic()
	require.NoError(t, err)

	// missing tx
	txBuilder.tx = nil
	err = txBuilder.ValidateBasic()
	require.Error(t, err)
}

func TestBuilderFeePayer(t *testing.T) {
	// keys and addresses
	_, _, addr1 := testdata.KeyTestPubAddr()
	_, _, addr2 := testdata.KeyTestPubAddr()
	_, _, addr3 := testdata.KeyTestPubAddr()

	// msg and signatures
	msg1 := testdata.NewTestMsg(addr1, addr2)
	feeAmount := testdata.NewTestFeeAmount()
	msgs := []sdk.Msg{msg1}

	cases := map[string]struct {
		txFeePayer      sdk.AccAddress
		expectedSigners []sdk.AccAddress
		expectedPayer   sdk.AccAddress
	}{
		"no fee payer specified": {
			expectedSigners: []sdk.AccAddress{addr1, addr2},
			expectedPayer:   addr1,
		},
		"secondary signer set as fee payer": {
			txFeePayer:      addr2,
			expectedSigners: []sdk.AccAddress{addr1, addr2},
			expectedPayer:   addr2,
		},
		"outside signer set as fee payer": {
			txFeePayer:      addr3,
			expectedSigners: []sdk.AccAddress{addr1, addr2, addr3},
			expectedPayer:   addr3,
		},
	}

	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			// setup basic tx
			txBuilder := newBuilder(nil)
			err := txBuilder.SetMsgs(msgs...)
			require.NoError(t, err)
			txBuilder.SetGasLimit(200000)
			txBuilder.SetFeeAmount(feeAmount)

			// set fee payer
			txBuilder.SetFeePayer(tc.txFeePayer)
			// and check it updates fields properly
			require.Equal(t, tc.expectedSigners, txBuilder.GetSigners())
			require.Equal(t, tc.expectedPayer, txBuilder.FeePayer())
		})
	}
}

func TestBuilderFeeGranter(t *testing.T) {
	// keys and addresses
	_, _, addr1 := testdata.KeyTestPubAddr()

	// msg and signatures
	msg1 := testdata.NewTestMsg(addr1, addr2)
	feeAmount := testdata.NewTestFeeAmount()
	msgs := []sdk.Msg{msg1}

	txBuilder := newBuilder(nil)
	err := txBuilder.SetMsgs(msgs...)
	require.NoError(t, err)
	txBuilder.SetGasLimit(200000)
	txBuilder.SetFeeAmount(feeAmount)

	require.Empty(t, txBuilder.GetTx().FeeGranter())

	// set fee granter
	txBuilder.SetFeeGranter(addr1)
	require.Equal(t, addr1, txBuilder.GetTx().FeeGranter())
}

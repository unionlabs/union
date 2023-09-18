package tx

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	txtypes "github.com/cosmos/cosmos-sdk/types/tx"
	signingtypes "github.com/cosmos/cosmos-sdk/types/tx/signing"
	"github.com/cosmos/cosmos-sdk/x/auth/signing"
)

func TestDirectAuxHandler(t *testing.T) {
	privKey, pubkey, addr := testdata.KeyTestPubAddr()
	_, feePayerPubKey, feePayerAddr := testdata.KeyTestPubAddr()
	interfaceRegistry := codectypes.NewInterfaceRegistry()
	interfaceRegistry.RegisterImplementations((*sdk.Msg)(nil), &testdata.TestMsg{})
	marshaler := codec.NewProtoCodec(interfaceRegistry)

	txConfig := NewTxConfig(marshaler, []signingtypes.SignMode{signingtypes.SignMode_SIGN_MODE_DIRECT_AUX})
	txBuilder := txConfig.NewTxBuilder()

	chainID := "test-chain"
	memo := "sometestmemo"
	msgs := []sdk.Msg{testdata.NewTestMsg(addr)}
	accNum, accSeq := uint64(1), uint64(2) // Arbitrary account number/sequence

	any, err := codectypes.NewAnyWithValue(pubkey)
	require.NoError(t, err)

	sigData := &signingtypes.SingleSignatureData{
		SignMode: signingtypes.SignMode_SIGN_MODE_DIRECT_AUX,
	}

	sig := signingtypes.SignatureV2{
		PubKey:   pubkey,
		Data:     sigData,
		Sequence: accSeq,
	}
	feePayerSig := signingtypes.SignatureV2{
		PubKey:   feePayerPubKey,
		Data:     sigData,
		Sequence: accSeq,
	}

	fee := txtypes.Fee{Amount: sdk.NewCoins(sdk.NewInt64Coin("atom", 150)), GasLimit: 20000}
	tip := &txtypes.Tip{Amount: sdk.NewCoins(sdk.NewInt64Coin("tip-token", 10))}

	err = txBuilder.SetMsgs(msgs...)
	require.NoError(t, err)
	txBuilder.SetMemo(memo)
	txBuilder.SetFeeAmount(fee.Amount)
	txBuilder.SetFeePayer(feePayerAddr)
	txBuilder.SetGasLimit(fee.GasLimit)
	txBuilder.SetTip(tip)

	err = txBuilder.SetSignatures(sig, feePayerSig)
	require.NoError(t, err)

	signingData := signing.SignerData{
		Address:       addr.String(),
		ChainID:       chainID,
		AccountNumber: accNum,
		Sequence:      accSeq,
		PubKey:        pubkey,
	}
	feePayerSigningData := signing.SignerData{
		Address:       feePayerAddr.String(),
		ChainID:       chainID,
		AccountNumber: accNum,
		Sequence:      accSeq,
		PubKey:        feePayerPubKey,
	}

	modeHandler := signModeDirectAuxHandler{}

	t.Log("verify fee payer cannot use SIGN_MODE_DIRECT_AUX")
	_, err = modeHandler.GetSignBytes(signingtypes.SignMode_SIGN_MODE_DIRECT_AUX, feePayerSigningData, txBuilder.GetTx())
	require.EqualError(t, err, fmt.Sprintf("fee payer %s cannot sign with %s: unauthorized", feePayerAddr.String(), signingtypes.SignMode_SIGN_MODE_DIRECT_AUX))

	t.Log("verify GetSignBytes with generating sign bytes by marshaling signDocDirectAux")
	signBytes, err := modeHandler.GetSignBytes(signingtypes.SignMode_SIGN_MODE_DIRECT_AUX, signingData, txBuilder.GetTx())
	require.NoError(t, err)
	require.NotNil(t, signBytes)

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

	signDocDirectAux := txtypes.SignDocDirectAux{
		AccountNumber: accNum,
		BodyBytes:     bodyBytes,
		ChainId:       "test-chain",
		PublicKey:     any,
		Sequence:      accSeq,
		Tip:           tip,
	}

	expectedSignBytes, err := signDocDirectAux.Marshal()
	require.NoError(t, err)
	require.Equal(t, expectedSignBytes, signBytes)

	t.Log("verify that setting signature doesn't change sign bytes")
	sigData.Signature, err = privKey.Sign(signBytes)
	require.NoError(t, err)
	err = txBuilder.SetSignatures(sig)
	require.NoError(t, err)
	signBytes, err = modeHandler.GetSignBytes(signingtypes.SignMode_SIGN_MODE_DIRECT_AUX, signingData, txBuilder.GetTx())
	require.NoError(t, err)
	require.Equal(t, expectedSignBytes, signBytes)

	t.Log("verify GetSignBytes with false txBody data")
	signDocDirectAux.BodyBytes = []byte("dfafdasfds")
	expectedSignBytes, err = signDocDirectAux.Marshal()
	require.NoError(t, err)
	require.NotEqual(t, expectedSignBytes, signBytes)
}

func TestDirectAuxHandler_DefaultMode(t *testing.T) {
	handler := signModeDirectAuxHandler{}
	require.Equal(t, signingtypes.SignMode_SIGN_MODE_DIRECT_AUX, handler.DefaultMode())
}

func TestDirectAuxModeHandler_nonDIRECT_MODE(t *testing.T) {
	invalidModes := []signingtypes.SignMode{
		signingtypes.SignMode_SIGN_MODE_DIRECT,
		signingtypes.SignMode_SIGN_MODE_TEXTUAL,
		signingtypes.SignMode_SIGN_MODE_LEGACY_AMINO_JSON,
		signingtypes.SignMode_SIGN_MODE_UNSPECIFIED,
	}
	for _, invalidMode := range invalidModes {
		t.Run(invalidMode.String(), func(t *testing.T) {
			var dh signModeDirectAuxHandler
			var signingData signing.SignerData
			_, err := dh.GetSignBytes(invalidMode, signingData, nil)
			require.Error(t, err)
			wantErr := fmt.Errorf("expected %s, got %s", signingtypes.SignMode_SIGN_MODE_DIRECT_AUX, invalidMode)
			require.Equal(t, err, wantErr)
		})
	}
}

func TestDirectAuxModeHandler_nonProtoTx(t *testing.T) {
	var dh signModeDirectAuxHandler
	var signingData signing.SignerData
	tx := new(nonProtoTx)
	_, err := dh.GetSignBytes(signingtypes.SignMode_SIGN_MODE_DIRECT_AUX, signingData, tx)
	require.Error(t, err)
	wantErr := fmt.Errorf("can only handle a protobuf Tx, got %T", tx)
	require.Equal(t, err, wantErr)
}

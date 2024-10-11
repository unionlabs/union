package tx_test

import (
	"context"
	"testing"

	"github.com/stretchr/testify/require"

	clienttx "github.com/cosmos/cosmos-sdk/client/tx"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	_ "github.com/cosmos/cosmos-sdk/testutil/testdata/testpb"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
	authsigning "github.com/cosmos/cosmos-sdk/x/auth/signing"
)

var (
	// The final TX has 3 signers, in this order.
	tipperPriv, tipperPk, tipperAddr       = testdata.KeyTestPubAddr()
	aux2Priv, aux2Pk, aux2Addr             = testdata.KeyTestPubAddr()
	feepayerPriv, feepayerPk, feepayerAddr = testdata.KeyTestPubAddr()

	msg  = testdata.NewTestMsg(tipperAddr, aux2Addr)
	memo = "test-memo"

	chainID = "test-chain"
	gas     = testdata.NewTestGasLimit()
	fee     = testdata.NewTestFeeAmount()
	extOpt  = &testdata.Cat{}
)

// TestBuilderWithAux creates a tx with 2 aux signers:
// - 1st one is tipper,
// - 2nd one is just an aux signer.
// Then it tests integrating the 2 AuxSignerData into a
// client.TxBuilder created by the fee payer.
func TestBuilderWithAux(t *testing.T) {
	t.Skip("restore when we re-enable aux on the TX builder")
	encodingConfig := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{})
	interfaceRegistry := encodingConfig.InterfaceRegistry
	txConfig := encodingConfig.TxConfig

	testdata.RegisterInterfaces(interfaceRegistry)

	// Create an AuxTxBuilder for tipper (1st signer)
	txBuilder, txSig := makeTxBuilder(t)
	txSignerData, err := txBuilder.GetAuxSignerData()
	require.NoError(t, err)

	// Create an AuxTxBuilder for aux2 (2nd signer)
	aux2Builder := clienttx.NewAuxTxBuilder()
	aux2Builder.SetAddress(aux2Addr.String())
	aux2Builder.SetAccountNumber(11)
	aux2Builder.SetSequence(12)
	aux2Builder.SetTimeoutHeight(3)
	aux2Builder.SetMemo(memo)
	aux2Builder.SetChainID(chainID)
	err = aux2Builder.SetMsgs(msg)
	require.NoError(t, err)
	err = aux2Builder.SetPubKey(aux2Pk)
	require.NoError(t, err)
	extOptAny, err := codectypes.NewAnyWithValue(extOpt)
	require.NoError(t, err)
	aux2Builder.SetExtensionOptions(extOptAny)
	aux2Builder.SetNonCriticalExtensionOptions(extOptAny)
	err = aux2Builder.SetSignMode(signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON)
	require.NoError(t, err)
	signBz, err := aux2Builder.GetSignBytes()
	require.NoError(t, err)
	aux2Sig, err := aux2Priv.Sign(signBz)
	require.NoError(t, err)
	aux2Builder.SetSignature(aux2Sig)
	aux2SignerData, err := aux2Builder.GetAuxSignerData()
	require.NoError(t, err)

	// Fee payer (3rd and last signer) creates a TxBuilder.
	w := txConfig.NewTxBuilder()
	// Note: we're testing calling AddAuxSignerData in the wrong order, i.e.
	// adding the aux2 signer data first before the tipper.
	err = w.AddAuxSignerData(aux2SignerData)
	require.NoError(t, err)

	// Test that when adding another AuxSignerData, the 2nd data should match
	// the 1st one.
	testcases := []struct {
		name     string
		malleate func()
		expErr   bool
	}{
		{"address and msg signer mistacher", func() { txBuilder.SetAddress("foobar") }, true},
		{"memo mismatch", func() { txBuilder.SetMemo("mismatch") }, true},
		{"timeout height mismatch", func() { txBuilder.SetTimeoutHeight(98) }, true},
		{"extension options length mismatch", func() { txBuilder.SetExtensionOptions() }, true},
		{"extension options member mismatch", func() { txBuilder.SetExtensionOptions(&codectypes.Any{}) }, true},
		{"non-critical extension options length mismatch", func() { txBuilder.SetNonCriticalExtensionOptions() }, true},
		{"non-critical extension options member mismatch", func() { txBuilder.SetNonCriticalExtensionOptions(&codectypes.Any{}) }, true},
		{"happy case", func() {}, false},
	}
	for _, tc := range testcases {
		t.Run(tc.name, func(t *testing.T) {
			txBuilder, txSig = makeTxBuilder(t)

			tc.malleate()

			_, err := txBuilder.GetSignBytes()
			require.NoError(t, err)
			txSignerData, err = txBuilder.GetAuxSignerData()
			require.NoError(t, err)

			err = w.AddAuxSignerData(txSignerData)
			if tc.expErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}

	w.SetFeePayer(feepayerAddr)
	w.SetFeeAmount(fee)
	w.SetGasLimit(gas)
	sigs, err := w.(authsigning.SigVerifiableTx).GetSignaturesV2()
	require.NoError(t, err)
	txSigV2 := sigs[0]
	aux2SigV2 := sigs[1]
	// Set all signer infos.
	err = w.SetSignatures(txSigV2, aux2SigV2, signing.SignatureV2{
		PubKey:   feepayerPk,
		Sequence: 15,
	})
	require.NoError(t, err)

	signerData := authsigning.SignerData{
		Address:       feepayerAddr.String(),
		ChainID:       chainID,
		AccountNumber: 11,
		Sequence:      15,
		PubKey:        feepayerPk,
	}

	signBz, err = authsigning.GetSignBytesAdapter(
		context.Background(), txConfig.SignModeHandler(), signing.SignMode_SIGN_MODE_DIRECT,
		signerData, w.GetTx())

	require.NoError(t, err)
	feepayerSig, err := feepayerPriv.Sign(signBz)
	require.NoError(t, err)
	// Set all signatures.
	err = w.SetSignatures(txSigV2, aux2SigV2, signing.SignatureV2{
		PubKey: feepayerPk,
		Data: &signing.SingleSignatureData{
			SignMode:  signing.SignMode_SIGN_MODE_DIRECT,
			Signature: feepayerSig,
		},
		Sequence: 22,
	})
	require.NoError(t, err)

	// Make sure tx is correct.
	txBz, err := txConfig.TxEncoder()(w.GetTx())
	require.NoError(t, err)
	tx, err := txConfig.TxDecoder()(txBz)
	require.NoError(t, err)
	require.Equal(t, tx.(sdk.FeeTx).FeePayer(), []byte(feepayerAddr))
	require.Equal(t, tx.(sdk.FeeTx).GetFee(), fee)
	require.Equal(t, tx.(sdk.FeeTx).GetGas(), gas)
	require.Equal(t, msg, tx.GetMsgs()[0])
	require.Equal(t, memo, tx.(sdk.TxWithMemo).GetMemo())
	require.Equal(t, uint64(3), tx.(sdk.TxWithTimeoutHeight).GetTimeoutHeight())
	sigs, err = tx.(authsigning.Tx).GetSignaturesV2()
	require.NoError(t, err)
	require.Len(t, sigs, 3)
	require.Equal(t, signing.SignatureV2{
		PubKey:   tipperPk,
		Data:     &signing.SingleSignatureData{SignMode: signing.SignMode_SIGN_MODE_DIRECT_AUX, Signature: txSig},
		Sequence: 2,
	}, sigs[0])
	require.Equal(t, signing.SignatureV2{
		PubKey:   aux2Pk,
		Data:     &signing.SingleSignatureData{SignMode: signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON, Signature: aux2Sig},
		Sequence: 12,
	}, sigs[1])
	require.Equal(t, signing.SignatureV2{
		PubKey:   feepayerPk,
		Data:     &signing.SingleSignatureData{SignMode: signing.SignMode_SIGN_MODE_DIRECT, Signature: feepayerSig},
		Sequence: 22,
	}, sigs[2])
}

func makeTxBuilder(t *testing.T) (clienttx.AuxTxBuilder, []byte) {
	t.Helper()
	txBuilder := clienttx.NewAuxTxBuilder()
	txBuilder.SetAddress(tipperAddr.String())
	txBuilder.SetAccountNumber(1)
	txBuilder.SetSequence(2)
	txBuilder.SetTimeoutHeight(3)
	txBuilder.SetMemo(memo)
	txBuilder.SetChainID(chainID)
	err := txBuilder.SetMsgs(msg)
	require.NoError(t, err)
	err = txBuilder.SetPubKey(tipperPk)
	require.NoError(t, err)
	extOptAny, err := codectypes.NewAnyWithValue(extOpt)
	require.NoError(t, err)
	txBuilder.SetExtensionOptions(extOptAny)
	txBuilder.SetNonCriticalExtensionOptions(extOptAny)
	err = txBuilder.SetSignMode(signing.SignMode_SIGN_MODE_DIRECT_AUX)
	require.NoError(t, err)
	signBz, err := txBuilder.GetSignBytes()
	require.NoError(t, err)
	tipperSig, err := tipperPriv.Sign(signBz)
	require.NoError(t, err)
	txBuilder.SetSignature(tipperSig)

	return txBuilder, tipperSig
}

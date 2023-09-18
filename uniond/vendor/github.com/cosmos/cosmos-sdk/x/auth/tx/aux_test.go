package tx_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/depinject"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/tx"
	clienttx "github.com/cosmos/cosmos-sdk/client/tx"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	txtypes "github.com/cosmos/cosmos-sdk/types/tx"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
	authsigning "github.com/cosmos/cosmos-sdk/x/auth/signing"
	"github.com/cosmos/cosmos-sdk/x/auth/testutil"
)

var (
	// The final TX has 3 signers, in this order.
	tipperPriv, tipperPk, tipperAddr       = testdata.KeyTestPubAddr()
	aux2Priv, aux2Pk, aux2Addr             = testdata.KeyTestPubAddr()
	feepayerPriv, feepayerPk, feepayerAddr = testdata.KeyTestPubAddr()

	msg     = testdata.NewTestMsg(tipperAddr, aux2Addr)
	memo    = "test-memo"
	tip     = &txtypes.Tip{Tipper: tipperAddr.String(), Amount: sdk.NewCoins(sdk.NewCoin("tip-denom", sdk.NewIntFromUint64(123)))}
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
	var (
		interfaceRegistry codectypes.InterfaceRegistry
		txConfig          client.TxConfig
	)

	err := depinject.Inject(testutil.AppConfig,
		&interfaceRegistry,
		&txConfig,
	)
	require.NoError(t, err)

	testdata.RegisterInterfaces(interfaceRegistry)

	// Create an AuxTxBuilder for tipper (1st signer)
	tipperBuilder, tipperSig := makeTipperTxBuilder(t)
	tipperSignerData, err := tipperBuilder.GetAuxSignerData()
	require.NoError(t, err)

	// Create an AuxTxBuilder for aux2 (2nd signer)
	aux2Builder := clienttx.NewAuxTxBuilder()
	aux2Builder.SetAddress(aux2Addr.String())
	aux2Builder.SetAccountNumber(11)
	aux2Builder.SetSequence(12)
	aux2Builder.SetTimeoutHeight(3)
	aux2Builder.SetMemo(memo)
	aux2Builder.SetChainID(chainID)
	aux2Builder.SetMsgs(msg)
	aux2Builder.SetPubKey(aux2Pk)
	aux2Builder.SetTip(tip)
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
		{"address and msg signer mistacher", func() { tipperBuilder.SetAddress("foobar") }, true},
		{"memo mismatch", func() { tipperBuilder.SetMemo("mismatch") }, true},
		{"timeout height mismatch", func() { tipperBuilder.SetTimeoutHeight(98) }, true},
		{"extension options length mismatch", func() { tipperBuilder.SetExtensionOptions() }, true},
		{"extension options member mismatch", func() { tipperBuilder.SetExtensionOptions(&codectypes.Any{}) }, true},
		{"non-critical extension options length mismatch", func() { tipperBuilder.SetNonCriticalExtensionOptions() }, true},
		{"non-critical extension options member mismatch", func() { tipperBuilder.SetNonCriticalExtensionOptions(&codectypes.Any{}) }, true},
		{"tip amount mismatch", func() { tipperBuilder.SetTip(&txtypes.Tip{Tipper: tip.Tipper, Amount: sdk.NewCoins()}) }, true},
		{"tipper mismatch", func() { tipperBuilder.SetTip(&txtypes.Tip{Tipper: "mismatch", Amount: tip.Amount}) }, true},
		{"happy case", func() {}, false},
	}
	for _, tc := range testcases {
		tc := tc
		t.Run(tc.name, func(t *testing.T) {
			tipperBuilder, tipperSig = makeTipperTxBuilder(t)

			tc.malleate()

			_, err := tipperBuilder.GetSignBytes()
			require.NoError(t, err)
			tipperSignerData, err = tipperBuilder.GetAuxSignerData()
			require.NoError(t, err)

			err = w.AddAuxSignerData(tipperSignerData)
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
	tipperSigV2 := sigs[0]
	aux2SigV2 := sigs[1]
	// Set all signer infos.
	w.SetSignatures(tipperSigV2, aux2SigV2, signing.SignatureV2{
		PubKey:   feepayerPk,
		Sequence: 15,
	})
	signBz, err = txConfig.SignModeHandler().GetSignBytes(
		signing.SignMode_SIGN_MODE_DIRECT,
		authsigning.SignerData{
			Address:       feepayerAddr.String(),
			ChainID:       chainID,
			AccountNumber: 11,
			Sequence:      15,
			PubKey:        feepayerPk,
		},
		w.GetTx(),
	)
	require.NoError(t, err)
	feepayerSig, err := feepayerPriv.Sign(signBz)
	require.NoError(t, err)
	// Set all signatures.
	w.SetSignatures(tipperSigV2, aux2SigV2, signing.SignatureV2{
		PubKey: feepayerPk,
		Data: &signing.SingleSignatureData{
			SignMode:  signing.SignMode_SIGN_MODE_DIRECT,
			Signature: feepayerSig,
		},
		Sequence: 22,
	})

	// Make sure tx is correct.
	txBz, err := txConfig.TxEncoder()(w.GetTx())
	require.NoError(t, err)
	tx, err := txConfig.TxDecoder()(txBz)
	require.NoError(t, err)
	require.Equal(t, tx.(sdk.FeeTx).FeePayer(), feepayerAddr)
	require.Equal(t, tx.(sdk.FeeTx).GetFee(), fee)
	require.Equal(t, tx.(sdk.FeeTx).GetGas(), gas)
	require.Equal(t, tip, tx.(txtypes.TipTx).GetTip())
	require.Equal(t, msg, tx.GetMsgs()[0])
	require.Equal(t, memo, tx.(sdk.TxWithMemo).GetMemo())
	require.Equal(t, uint64(3), tx.(sdk.TxWithTimeoutHeight).GetTimeoutHeight())
	sigs, err = tx.(authsigning.Tx).GetSignaturesV2()
	require.NoError(t, err)
	require.Len(t, sigs, 3)
	require.Equal(t, signing.SignatureV2{
		PubKey:   tipperPk,
		Data:     &signing.SingleSignatureData{SignMode: signing.SignMode_SIGN_MODE_DIRECT_AUX, Signature: tipperSig},
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

func makeTipperTxBuilder(t *testing.T) (tx.AuxTxBuilder, []byte) {
	tipperBuilder := clienttx.NewAuxTxBuilder()
	tipperBuilder.SetAddress(tipperAddr.String())
	tipperBuilder.SetAccountNumber(1)
	tipperBuilder.SetSequence(2)
	tipperBuilder.SetTimeoutHeight(3)
	tipperBuilder.SetMemo(memo)
	tipperBuilder.SetChainID(chainID)
	tipperBuilder.SetMsgs(msg)
	tipperBuilder.SetPubKey(tipperPk)
	tipperBuilder.SetTip(tip)
	extOptAny, err := codectypes.NewAnyWithValue(extOpt)
	require.NoError(t, err)
	tipperBuilder.SetExtensionOptions(extOptAny)
	tipperBuilder.SetNonCriticalExtensionOptions(extOptAny)
	err = tipperBuilder.SetSignMode(signing.SignMode_SIGN_MODE_DIRECT_AUX)
	require.NoError(t, err)
	signBz, err := tipperBuilder.GetSignBytes()
	require.NoError(t, err)
	tipperSig, err := tipperPriv.Sign(signBz)
	require.NoError(t, err)
	tipperBuilder.SetSignature(tipperSig)

	return tipperBuilder, tipperSig
}

package tx

import (
	"bytes"

	"github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/client"
	kmultisig "github.com/cosmos/cosmos-sdk/crypto/keys/multisig"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/crypto/types/multisig"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	signingtypes "github.com/cosmos/cosmos-sdk/types/tx/signing"
	"github.com/cosmos/cosmos-sdk/x/auth/signing"
)

// TxConfigTestSuite provides a test suite that can be used to test that a TxConfig implementation is correct.
type TxConfigTestSuite struct {
	suite.Suite
	TxConfig client.TxConfig
}

// NewTxConfigTestSuite returns a new TxConfigTestSuite with the provided TxConfig implementation
func NewTxConfigTestSuite(txConfig client.TxConfig) *TxConfigTestSuite {
	return &TxConfigTestSuite{TxConfig: txConfig}
}

func (s *TxConfigTestSuite) TestTxBuilderGetTx() {
	txBuilder := s.TxConfig.NewTxBuilder()
	tx := txBuilder.GetTx()
	s.Require().NotNil(tx)
	s.Require().Equal(len(tx.GetMsgs()), 0)
}

func (s *TxConfigTestSuite) TestTxBuilderSetFeeAmount() {
	txBuilder := s.TxConfig.NewTxBuilder()
	feeAmount := sdk.Coins{
		sdk.NewInt64Coin("atom", 20000000),
	}
	txBuilder.SetFeeAmount(feeAmount)
	feeTx := txBuilder.GetTx()
	s.Require().Equal(feeAmount, feeTx.GetFee())
}

func (s *TxConfigTestSuite) TestTxBuilderSetGasLimit() {
	const newGas uint64 = 300000
	txBuilder := s.TxConfig.NewTxBuilder()
	txBuilder.SetGasLimit(newGas)
	feeTx := txBuilder.GetTx()
	s.Require().Equal(newGas, feeTx.GetGas())
}

func (s *TxConfigTestSuite) TestTxBuilderSetMemo() {
	const newMemo string = "newfoomemo"
	txBuilder := s.TxConfig.NewTxBuilder()
	txBuilder.SetMemo(newMemo)
	txWithMemo := txBuilder.GetTx()
	s.Require().Equal(txWithMemo.GetMemo(), newMemo)
}

func (s *TxConfigTestSuite) TestTxBuilderSetMsgs() {
	_, _, addr1 := testdata.KeyTestPubAddr()
	_, _, addr2 := testdata.KeyTestPubAddr()
	msg1 := testdata.NewTestMsg(addr1)
	msg2 := testdata.NewTestMsg(addr2)
	msgs := []sdk.Msg{msg1, msg2}

	txBuilder := s.TxConfig.NewTxBuilder()

	err := txBuilder.SetMsgs(msgs...)
	s.Require().NoError(err)
	tx := txBuilder.GetTx()
	s.Require().Equal(msgs, tx.GetMsgs())
	s.Require().Equal([]sdk.AccAddress{addr1, addr2}, tx.GetSigners())
	s.Require().Equal(addr1, tx.FeePayer())
	s.Require().Error(tx.ValidateBasic()) // should fail because of no signatures
}

func (s *TxConfigTestSuite) TestTxBuilderSetSignatures() {
	privKey, pubkey, addr := testdata.KeyTestPubAddr()
	privKey2, pubkey2, _ := testdata.KeyTestPubAddr()
	multisigPk := kmultisig.NewLegacyAminoPubKey(2, []cryptotypes.PubKey{pubkey, pubkey2})

	txBuilder := s.TxConfig.NewTxBuilder()

	// set test msg
	msg := testdata.NewTestMsg(addr)
	msigAddr := sdk.AccAddress(multisigPk.Address())
	msg2 := testdata.NewTestMsg(msigAddr)
	err := txBuilder.SetMsgs(msg, msg2)
	txBuilder.SetFeeAmount(testdata.NewTestFeeAmount())
	s.Require().NoError(err)

	// check that validation fails
	s.Require().Error(txBuilder.GetTx().ValidateBasic())

	signModeHandler := s.TxConfig.SignModeHandler()
	s.Require().Contains(signModeHandler.Modes(), signModeHandler.DefaultMode())

	// set SignatureV2 without actual signature bytes
	seq1 := uint64(2) // Arbitrary account sequence
	sigData1 := &signingtypes.SingleSignatureData{SignMode: signModeHandler.DefaultMode()}
	sig1 := signingtypes.SignatureV2{PubKey: pubkey, Data: sigData1, Sequence: seq1}

	mseq := uint64(4) // Arbitrary account sequence
	msigData := multisig.NewMultisig(2)
	multisig.AddSignature(msigData, &signingtypes.SingleSignatureData{SignMode: signModeHandler.DefaultMode()}, 0)
	multisig.AddSignature(msigData, &signingtypes.SingleSignatureData{SignMode: signModeHandler.DefaultMode()}, 1)
	msig := signingtypes.SignatureV2{PubKey: multisigPk, Data: msigData, Sequence: mseq}

	// fail validation without required signers
	err = txBuilder.SetSignatures(sig1)
	s.Require().NoError(err)
	sigTx := txBuilder.GetTx()
	s.Require().Error(sigTx.ValidateBasic())

	err = txBuilder.SetSignatures(sig1, msig)
	s.Require().NoError(err)
	sigTx = txBuilder.GetTx()
	sigsV2, err := sigTx.GetSignaturesV2()
	s.Require().NoError(err)
	s.Require().Len(sigsV2, 2)
	s.Require().True(sigEquals(sig1, sigsV2[0]))
	s.Require().True(sigEquals(msig, sigsV2[1]))
	s.Require().Equal([]sdk.AccAddress{addr, msigAddr}, sigTx.GetSigners())
	s.Require().NoError(sigTx.ValidateBasic())

	// sign transaction
	signerData := signing.SignerData{
		Address:       addr.String(),
		ChainID:       "test",
		AccountNumber: 1,
		Sequence:      seq1,
		PubKey:        pubkey,
	}
	signBytes, err := signModeHandler.GetSignBytes(signModeHandler.DefaultMode(), signerData, sigTx)
	s.Require().NoError(err)
	sigBz, err := privKey.Sign(signBytes)
	s.Require().NoError(err)

	signerData = signing.SignerData{
		Address:       msigAddr.String(),
		ChainID:       "test",
		AccountNumber: 3,
		Sequence:      mseq,
		PubKey:        multisigPk,
	}
	mSignBytes, err := signModeHandler.GetSignBytes(signModeHandler.DefaultMode(), signerData, sigTx)
	s.Require().NoError(err)
	mSigBz1, err := privKey.Sign(mSignBytes)
	s.Require().NoError(err)
	mSigBz2, err := privKey2.Sign(mSignBytes)
	s.Require().NoError(err)
	msigData = multisig.NewMultisig(2)
	multisig.AddSignature(msigData, &signingtypes.SingleSignatureData{
		SignMode: signModeHandler.DefaultMode(), Signature: mSigBz1,
	}, 0)
	multisig.AddSignature(msigData, &signingtypes.SingleSignatureData{
		SignMode: signModeHandler.DefaultMode(), Signature: mSigBz2,
	}, 0)

	// set signature
	sigData1.Signature = sigBz
	sig1 = signingtypes.SignatureV2{PubKey: pubkey, Data: sigData1, Sequence: seq1}
	msig = signingtypes.SignatureV2{PubKey: multisigPk, Data: msigData, Sequence: mseq}
	err = txBuilder.SetSignatures(sig1, msig)
	s.Require().NoError(err)
	sigTx = txBuilder.GetTx()
	sigsV2, err = sigTx.GetSignaturesV2()
	s.Require().NoError(err)
	s.Require().Len(sigsV2, 2)
	s.Require().True(sigEquals(sig1, sigsV2[0]))
	s.Require().True(sigEquals(msig, sigsV2[1]))
	s.Require().Equal([]sdk.AccAddress{addr, msigAddr}, sigTx.GetSigners())
	s.Require().NoError(sigTx.ValidateBasic())
}

func sigEquals(sig1, sig2 signingtypes.SignatureV2) bool {
	if !sig1.PubKey.Equals(sig2.PubKey) {
		return false
	}

	if sig1.Data == nil && sig2.Data == nil {
		return true
	}

	return sigDataEquals(sig1.Data, sig2.Data)
}

func sigDataEquals(data1, data2 signingtypes.SignatureData) bool {
	switch data1 := data1.(type) {
	case *signingtypes.SingleSignatureData:
		data2, ok := data2.(*signingtypes.SingleSignatureData)
		if !ok {
			return false
		}

		if data1.SignMode != data2.SignMode {
			return false
		}

		return bytes.Equal(data1.Signature, data2.Signature)
	case *signingtypes.MultiSignatureData:
		data2, ok := data2.(*signingtypes.MultiSignatureData)
		if !ok {
			return false
		}
		if !data1.BitArray.Equal(data2.BitArray) || len(data1.Signatures) != len(data2.Signatures) {
			return false
		}

		for i, s := range data1.Signatures {
			if !sigDataEquals(s, data2.Signatures[i]) {
				return false
			}
		}

		return true
	default:
		return false
	}
}

func (s *TxConfigTestSuite) TestTxEncodeDecode() {
	log := s.T().Log
	_, pubkey, addr := testdata.KeyTestPubAddr()
	feeAmount := sdk.Coins{sdk.NewInt64Coin("atom", 150)}
	gasLimit := uint64(50000)
	memo := "foomemo"
	msg := testdata.NewTestMsg(addr)
	dummySig := []byte("dummySig")
	sig := signingtypes.SignatureV2{
		PubKey: pubkey,
		Data: &signingtypes.SingleSignatureData{
			SignMode:  signingtypes.SignMode_SIGN_MODE_LEGACY_AMINO_JSON,
			Signature: dummySig,
		},
	}

	txBuilder := s.TxConfig.NewTxBuilder()
	txBuilder.SetFeeAmount(feeAmount)
	txBuilder.SetGasLimit(gasLimit)
	txBuilder.SetMemo(memo)
	err := txBuilder.SetMsgs(msg)
	s.Require().NoError(err)
	err = txBuilder.SetSignatures(sig)
	s.Require().NoError(err)
	tx := txBuilder.GetTx()

	log("encode transaction")
	txBytes, err := s.TxConfig.TxEncoder()(tx)
	s.Require().NoError(err)
	s.Require().NotNil(txBytes)
	log("decode transaction", s.TxConfig)
	tx2, err := s.TxConfig.TxDecoder()(txBytes)

	s.Require().NoError(err)
	tx3, ok := tx2.(signing.Tx)
	s.Require().True(ok)
	s.Require().Equal([]sdk.Msg{msg}, tx3.GetMsgs())
	s.Require().Equal(feeAmount, tx3.GetFee())
	s.Require().Equal(gasLimit, tx3.GetGas())
	s.Require().Equal(memo, tx3.GetMemo())
	tx3Sigs, err := tx3.GetSignaturesV2()
	s.Require().NoError(err)
	s.Require().Equal([]signingtypes.SignatureV2{sig}, tx3Sigs)
	pks, err := tx3.GetPubKeys()
	s.Require().NoError(err)
	s.Require().Equal([]cryptotypes.PubKey{pubkey}, pks)

	log("JSON encode transaction")
	jsonTxBytes, err := s.TxConfig.TxJSONEncoder()(tx)
	s.Require().NoError(err)
	s.Require().NotNil(jsonTxBytes)

	log("JSON decode transaction")
	tx2, err = s.TxConfig.TxJSONDecoder()(jsonTxBytes)
	s.Require().NoError(err)
	tx3, ok = tx2.(signing.Tx)
	s.Require().True(ok)
	s.Require().Equal([]sdk.Msg{msg}, tx3.GetMsgs())
	s.Require().Equal(feeAmount, tx3.GetFee())
	s.Require().Equal(gasLimit, tx3.GetGas())
	s.Require().Equal(memo, tx3.GetMemo())
	tx3Sigs, err = tx3.GetSignaturesV2()
	s.Require().NoError(err)
	s.Require().Equal([]signingtypes.SignatureV2{sig}, tx3Sigs)
	pks, err = tx3.GetPubKeys()
	s.Require().NoError(err)
	s.Require().Equal([]cryptotypes.PubKey{pubkey}, pks)
}

func (s *TxConfigTestSuite) TestWrapTxBuilder() {
	_, _, addr := testdata.KeyTestPubAddr()
	feeAmount := sdk.Coins{sdk.NewInt64Coin("atom", 150)}
	gasLimit := uint64(50000)
	memo := "foomemo"
	msg := testdata.NewTestMsg(addr)

	txBuilder := s.TxConfig.NewTxBuilder()
	txBuilder.SetFeeAmount(feeAmount)
	txBuilder.SetGasLimit(gasLimit)
	txBuilder.SetMemo(memo)
	err := txBuilder.SetMsgs(msg)
	s.Require().NoError(err)

	newTxBldr, err := s.TxConfig.WrapTxBuilder(txBuilder.GetTx())
	s.Require().NoError(err)
	s.Require().Equal(txBuilder, newTxBldr)
}

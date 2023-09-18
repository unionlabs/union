package tx_test

import (
	"testing"

	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"

	"cosmossdk.io/depinject"
	"github.com/cosmos/cosmos-sdk/client"
	clienttestutil "github.com/cosmos/cosmos-sdk/client/testutil"
	tx2 "github.com/cosmos/cosmos-sdk/client/tx"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	"github.com/cosmos/cosmos-sdk/types"
	typestx "github.com/cosmos/cosmos-sdk/types/tx"
	signing2 "github.com/cosmos/cosmos-sdk/types/tx/signing"
	"github.com/cosmos/cosmos-sdk/x/auth/migrations/legacytx"
	"github.com/cosmos/cosmos-sdk/x/auth/tx"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
)

const (
	memo          = "waboom"
	gas           = uint64(10000)
	timeoutHeight = uint64(5)
)

var (
	fee            = types.NewCoins(types.NewInt64Coin("bam", 100))
	_, pub1, addr1 = testdata.KeyTestPubAddr()
	_, _, addr2    = testdata.KeyTestPubAddr()
	rawSig         = []byte("dummy")
	sig            = signing2.SignatureV2{
		PubKey: pub1,
		Data: &signing2.SingleSignatureData{
			SignMode:  signing2.SignMode_SIGN_MODE_LEGACY_AMINO_JSON,
			Signature: rawSig,
		},
	}
	msg0 = banktypes.NewMsgSend(addr1, addr2, types.NewCoins(types.NewInt64Coin("wack", 1)))
	msg1 = banktypes.NewMsgSend(addr1, addr2, types.NewCoins(types.NewInt64Coin("wack", 2)))

	chainID = "test-chain"
	tip     = &typestx.Tip{Tipper: addr1.String(), Amount: testdata.NewTestFeeAmount()}
)

func buildTestTx(t *testing.T, builder client.TxBuilder) {
	builder.SetMemo(memo)
	builder.SetGasLimit(gas)
	builder.SetFeeAmount(fee)
	err := builder.SetMsgs(msg0, msg1)
	require.NoError(t, err)
	err = builder.SetSignatures(sig)
	require.NoError(t, err)
	builder.SetTimeoutHeight(timeoutHeight)
}

type TestSuite struct {
	suite.Suite
	codec    codec.Codec
	amino    *codec.LegacyAmino
	protoCfg client.TxConfig
	aminoCfg client.TxConfig
}

func (s *TestSuite) SetupSuite() {
	var (
		reg   codectypes.InterfaceRegistry
		amino *codec.LegacyAmino
	)
	err := depinject.Inject(clienttestutil.TestConfig, &reg, &amino)
	require.NoError(s.T(), err)

	s.protoCfg = tx.NewTxConfig(codec.NewProtoCodec(reg), tx.DefaultSignModes)
	s.aminoCfg = legacytx.StdTxConfig{Cdc: amino}
}

func (s *TestSuite) TestCopyTx() {
	// proto -> amino -> proto
	protoBuilder := s.protoCfg.NewTxBuilder()
	buildTestTx(s.T(), protoBuilder)
	aminoBuilder := s.aminoCfg.NewTxBuilder()
	err := tx2.CopyTx(protoBuilder.GetTx(), aminoBuilder, false)
	s.Require().NoError(err)
	protoBuilder2 := s.protoCfg.NewTxBuilder()
	err = tx2.CopyTx(aminoBuilder.GetTx(), protoBuilder2, false)
	s.Require().NoError(err)
	// Check sigs, signers and msgs.
	sigsV2_1, err := protoBuilder.GetTx().GetSignaturesV2()
	s.Require().NoError(err)
	sigsV2_2, err := protoBuilder2.GetTx().GetSignaturesV2()
	s.Require().NoError(err)
	s.Require().Equal(sigsV2_1, sigsV2_2)
	s.Require().Equal(protoBuilder.GetTx().GetSigners(), protoBuilder2.GetTx().GetSigners())
	s.Require().Equal(protoBuilder.GetTx().GetMsgs()[0], protoBuilder2.GetTx().GetMsgs()[0])
	s.Require().Equal(protoBuilder.GetTx().GetMsgs()[1], protoBuilder2.GetTx().GetMsgs()[1])

	// amino -> proto -> amino
	aminoBuilder = s.aminoCfg.NewTxBuilder()
	buildTestTx(s.T(), aminoBuilder)
	protoBuilder = s.protoCfg.NewTxBuilder()
	err = tx2.CopyTx(aminoBuilder.GetTx(), protoBuilder, false)
	s.Require().NoError(err)
	aminoBuilder2 := s.aminoCfg.NewTxBuilder()
	err = tx2.CopyTx(protoBuilder.GetTx(), aminoBuilder2, false)
	s.Require().NoError(err)
	// Check sigs, signers, and msgs
	sigsV2_1, err = aminoBuilder.GetTx().GetSignaturesV2()
	s.Require().NoError(err)
	sigsV2_2, err = aminoBuilder2.GetTx().GetSignaturesV2()
	s.Require().NoError(err)
	s.Require().Equal(sigsV2_1, sigsV2_2)
	s.Require().Equal(aminoBuilder.GetTx().GetSigners(), aminoBuilder2.GetTx().GetSigners())
	s.Require().Equal(aminoBuilder.GetTx().GetMsgs()[0], aminoBuilder2.GetTx().GetMsgs()[0])
	s.Require().Equal(aminoBuilder.GetTx().GetMsgs()[1], aminoBuilder2.GetTx().GetMsgs()[1])
}

func (s *TestSuite) TestConvertTxToStdTx() {
	// proto tx
	protoBuilder := s.protoCfg.NewTxBuilder()
	buildTestTx(s.T(), protoBuilder)
	stdTx, err := tx2.ConvertTxToStdTx(s.amino, protoBuilder.GetTx())
	s.Require().NoError(err)
	s.Require().Equal(memo, stdTx.Memo)
	s.Require().Equal(gas, stdTx.Fee.Gas)
	s.Require().Equal(fee, stdTx.Fee.Amount)
	s.Require().Equal(msg0, stdTx.Msgs[0])
	s.Require().Equal(msg1, stdTx.Msgs[1])
	s.Require().Equal(timeoutHeight, stdTx.TimeoutHeight)
	s.Require().Equal(sig.PubKey, stdTx.Signatures[0].PubKey)
	s.Require().Equal(sig.Data.(*signing2.SingleSignatureData).Signature, stdTx.Signatures[0].Signature)

	// SIGN_MODE_DIRECT should fall back to an unsigned tx
	err = protoBuilder.SetSignatures(signing2.SignatureV2{
		PubKey: pub1,
		Data: &signing2.SingleSignatureData{
			SignMode:  signing2.SignMode_SIGN_MODE_DIRECT,
			Signature: []byte("dummy"),
		},
	})
	s.Require().NoError(err)
	stdTx, err = tx2.ConvertTxToStdTx(s.amino, protoBuilder.GetTx())
	s.Require().NoError(err)
	s.Require().Equal(memo, stdTx.Memo)
	s.Require().Equal(gas, stdTx.Fee.Gas)
	s.Require().Equal(fee, stdTx.Fee.Amount)
	s.Require().Equal(msg0, stdTx.Msgs[0])
	s.Require().Equal(msg1, stdTx.Msgs[1])
	s.Require().Equal(timeoutHeight, stdTx.TimeoutHeight)
	s.Require().Empty(stdTx.Signatures)

	// std tx
	aminoBuilder := s.aminoCfg.NewTxBuilder()
	buildTestTx(s.T(), aminoBuilder)
	stdTx = aminoBuilder.GetTx().(legacytx.StdTx)
	stdTx2, err := tx2.ConvertTxToStdTx(s.amino, stdTx)
	s.Require().NoError(err)
	s.Require().Equal(stdTx, stdTx2)
}

func TestTestSuite(t *testing.T) {
	suite.Run(t, new(TestSuite))
}

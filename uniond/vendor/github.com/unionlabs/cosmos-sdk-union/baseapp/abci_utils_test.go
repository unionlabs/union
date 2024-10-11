package baseapp_test

import (
	"bytes"
	"sort"
	"testing"

	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"
	cmtprotocrypto "github.com/cometbft/cometbft/api/cometbft/crypto/v1"
	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	cmtsecp256k1 "github.com/cometbft/cometbft/crypto/secp256k1"
	cmttypes "github.com/cometbft/cometbft/types"
	protoio "github.com/cosmos/gogoproto/io"
	"github.com/cosmos/gogoproto/proto"
	gogotypes "github.com/cosmos/gogoproto/types"
	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"

	"cosmossdk.io/core/address"
	"cosmossdk.io/core/comet"
	"cosmossdk.io/core/header"
	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/log"

	"github.com/cosmos/cosmos-sdk/baseapp"
	baseapptestutil "github.com/cosmos/cosmos-sdk/baseapp/testutil"
	"github.com/cosmos/cosmos-sdk/baseapp/testutil/mock"
	"github.com/cosmos/cosmos-sdk/client"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/mempool"
	signingtypes "github.com/cosmos/cosmos-sdk/types/tx/signing"
	authtx "github.com/cosmos/cosmos-sdk/x/auth/tx"
)

const (
	chainID = "chain-id"
)

type testValidator struct {
	consAddr sdk.ConsAddress
	tmPk     cmtprotocrypto.PublicKey
	privKey  cmtsecp256k1.PrivKey
}

func newTestValidator() testValidator {
	privkey := cmtsecp256k1.GenPrivKey()
	pubkey := privkey.PubKey()
	tmPk := cmtprotocrypto.PublicKey{
		Sum: &cmtprotocrypto.PublicKey_Secp256K1{
			Secp256K1: pubkey.Bytes(),
		},
	}

	return testValidator{
		consAddr: sdk.ConsAddress(pubkey.Address()),
		tmPk:     tmPk,
		privKey:  privkey,
	}
}

func (t testValidator) toValidator(power int64) abci.Validator {
	return abci.Validator{
		Address: t.consAddr.Bytes(),
		Power:   power,
	}
}

type ABCIUtilsTestSuite struct {
	suite.Suite

	valStore *mock.MockValidatorStore
	vals     [3]testValidator
	ctx      sdk.Context
}

func NewABCIUtilsTestSuite(t *testing.T) *ABCIUtilsTestSuite {
	t.Helper()
	// create 3 validators
	s := &ABCIUtilsTestSuite{
		vals: [3]testValidator{
			newTestValidator(),
			newTestValidator(),
			newTestValidator(),
		},
	}

	// create mock
	ctrl := gomock.NewController(t)
	valStore := mock.NewMockValidatorStore(ctrl)
	s.valStore = valStore

	// set up mock
	for _, val := range s.vals {
		pk, err := cryptocodec.FromCmtProtoPublicKey(val.tmPk)
		require.NoError(t, err)
		valStore.EXPECT().GetPubKeyByConsAddr(gomock.Any(), val.consAddr.Bytes()).Return(pk, nil).AnyTimes()
	}

	// create context
	s.ctx = sdk.Context{}.WithConsensusParams(cmtproto.ConsensusParams{
		Feature: &cmtproto.FeatureParams{
			VoteExtensionsEnableHeight: &gogotypes.Int64Value{Value: 2},
		},
	}).WithBlockHeader(cmtproto.Header{
		ChainID: chainID,
	}).WithLogger(log.NewTestLogger(t))
	return s
}

func TestABCIUtilsTestSuite(t *testing.T) {
	suite.Run(t, NewABCIUtilsTestSuite(t))
}

// check ValidateVoteExtensions works when all nodes have CommitBlockID votes
func (s *ABCIUtilsTestSuite) TestValidateVoteExtensionsHappyPath() {
	ext := []byte("vote-extension")
	cve := cmtproto.CanonicalVoteExtension{
		Extension: ext,
		Height:    2,
		Round:     int64(0),
		ChainId:   chainID,
	}

	bz, err := marshalDelimitedFn(&cve)
	s.Require().NoError(err)

	extSig0, err := s.vals[0].privKey.Sign(bz)
	s.Require().NoError(err)

	extSig1, err := s.vals[1].privKey.Sign(bz)
	s.Require().NoError(err)

	extSig2, err := s.vals[2].privKey.Sign(bz)
	s.Require().NoError(err)

	s.ctx = s.ctx.WithBlockHeight(3).WithHeaderInfo(header.Info{Height: 3, ChainID: chainID}) // enable vote-extensions

	llc := abci.ExtendedCommitInfo{
		Round: 0,
		Votes: []abci.ExtendedVoteInfo{
			{
				Validator:          s.vals[0].toValidator(333),
				VoteExtension:      ext,
				ExtensionSignature: extSig0,
				BlockIdFlag:        cmtproto.BlockIDFlagCommit,
			},
			{
				Validator:          s.vals[1].toValidator(333),
				VoteExtension:      ext,
				ExtensionSignature: extSig1,
				BlockIdFlag:        cmtproto.BlockIDFlagCommit,
			},
			{
				Validator:          s.vals[2].toValidator(334),
				VoteExtension:      ext,
				ExtensionSignature: extSig2,
				BlockIdFlag:        cmtproto.BlockIDFlagCommit,
			},
		},
	}

	// order + convert to last commit
	llc, info := extendedCommitToLastCommit(llc)
	s.ctx = s.ctx.WithCometInfo(info)

	// expect-pass (votes of height 2 are included in next block)
	s.Require().NoError(baseapp.ValidateVoteExtensions(s.ctx, s.valStore, llc))
}

// check ValidateVoteExtensions works when a single node has submitted a BlockID_Absent
func (s *ABCIUtilsTestSuite) TestValidateVoteExtensionsSingleVoteAbsent() {
	ext := []byte("vote-extension")
	cve := cmtproto.CanonicalVoteExtension{
		Extension: ext,
		Height:    2,
		Round:     int64(0),
		ChainId:   chainID,
	}

	bz, err := marshalDelimitedFn(&cve)
	s.Require().NoError(err)

	extSig0, err := s.vals[0].privKey.Sign(bz)
	s.Require().NoError(err)

	extSig2, err := s.vals[2].privKey.Sign(bz)
	s.Require().NoError(err)

	s.ctx = s.ctx.WithBlockHeight(3) // vote-extensions are enabled

	llc := abci.ExtendedCommitInfo{
		Round: 0,
		Votes: []abci.ExtendedVoteInfo{
			{
				Validator:          s.vals[0].toValidator(333),
				VoteExtension:      ext,
				ExtensionSignature: extSig0,
				BlockIdFlag:        cmtproto.BlockIDFlagCommit,
			},
			// validator of power <1/3 is missing, so commit-info shld still be valid
			{
				Validator:   s.vals[1].toValidator(333),
				BlockIdFlag: cmtproto.BlockIDFlagAbsent,
			},
			{
				Validator:          s.vals[2].toValidator(334),
				VoteExtension:      ext,
				ExtensionSignature: extSig2,
				BlockIdFlag:        cmtproto.BlockIDFlagCommit,
			},
		},
	}

	llc, info := extendedCommitToLastCommit(llc)
	s.ctx = s.ctx.WithCometInfo(info)

	// expect-pass (votes of height 2 are included in next block)
	s.Require().NoError(baseapp.ValidateVoteExtensions(s.ctx, s.valStore, llc))
}

// check ValidateVoteExtensions works with duplicate votes
func (s *ABCIUtilsTestSuite) TestValidateVoteExtensionsDuplicateVotes() {
	ext := []byte("vote-extension")
	cve := cmtproto.CanonicalVoteExtension{
		Extension: ext,
		Height:    2,
		Round:     int64(0),
		ChainId:   chainID,
	}

	bz, err := marshalDelimitedFn(&cve)
	s.Require().NoError(err)

	extSig0, err := s.vals[0].privKey.Sign(bz)
	s.Require().NoError(err)

	ve := abci.ExtendedVoteInfo{
		Validator:          s.vals[0].toValidator(333),
		VoteExtension:      ext,
		ExtensionSignature: extSig0,
		BlockIdFlag:        cmtproto.BlockIDFlagCommit,
	}

	ve2 := abci.ExtendedVoteInfo{
		Validator:          s.vals[0].toValidator(334), // use diff voting-power to dupe
		VoteExtension:      ext,
		ExtensionSignature: extSig0,
		BlockIdFlag:        cmtproto.BlockIDFlagCommit,
	}

	llc := abci.ExtendedCommitInfo{
		Round: 0,
		Votes: []abci.ExtendedVoteInfo{
			ve,
			ve2,
		},
	}

	s.ctx = s.ctx.WithBlockHeight(3) // vote-extensions are enabled
	llc, info := extendedCommitToLastCommit(llc)
	s.ctx = s.ctx.WithCometInfo(info)

	// expect fail (duplicate votes)
	s.Require().Error(baseapp.ValidateVoteExtensions(s.ctx, s.valStore, llc))
}

// check ValidateVoteExtensions works when a single node has submitted a BlockID_Nil
func (s *ABCIUtilsTestSuite) TestValidateVoteExtensionsSingleVoteNil() {
	ext := []byte("vote-extension")
	cve := cmtproto.CanonicalVoteExtension{
		Extension: ext,
		Height:    2,
		Round:     int64(0),
		ChainId:   chainID,
	}

	bz, err := marshalDelimitedFn(&cve)
	s.Require().NoError(err)

	extSig0, err := s.vals[0].privKey.Sign(bz)
	s.Require().NoError(err)

	extSig2, err := s.vals[2].privKey.Sign(bz)
	s.Require().NoError(err)

	llc := abci.ExtendedCommitInfo{
		Round: 0,
		Votes: []abci.ExtendedVoteInfo{
			{
				Validator:          s.vals[0].toValidator(333),
				VoteExtension:      ext,
				ExtensionSignature: extSig0,
				BlockIdFlag:        cmtproto.BlockIDFlagCommit,
			},
			// validator of power <1/3 is missing, so commit-info should still be valid
			{
				Validator:   s.vals[1].toValidator(333),
				BlockIdFlag: cmtproto.BlockIDFlagNil,
			},
			{
				Validator:          s.vals[2].toValidator(334),
				VoteExtension:      ext,
				ExtensionSignature: extSig2,
				BlockIdFlag:        cmtproto.BlockIDFlagCommit,
			},
		},
	}

	s.ctx = s.ctx.WithBlockHeight(3) // vote-extensions are enabled

	// create last commit
	llc, info := extendedCommitToLastCommit(llc)
	s.ctx = s.ctx.WithCometInfo(info)

	// expect-pass (votes of height 2 are included in next block)
	s.Require().NoError(baseapp.ValidateVoteExtensions(s.ctx, s.valStore, llc))
}

// check ValidateVoteExtensions works when two nodes have submitted a BlockID_Nil / BlockID_Absent
func (s *ABCIUtilsTestSuite) TestValidateVoteExtensionsTwoVotesNilAbsent() {
	ext := []byte("vote-extension")
	cve := cmtproto.CanonicalVoteExtension{
		Extension: ext,
		Height:    2,
		Round:     int64(0),
		ChainId:   chainID,
	}

	bz, err := marshalDelimitedFn(&cve)
	s.Require().NoError(err)

	extSig0, err := s.vals[0].privKey.Sign(bz)
	s.Require().NoError(err)

	llc := abci.ExtendedCommitInfo{
		Round: 0,
		Votes: []abci.ExtendedVoteInfo{
			// validator of power >2/3 is missing, so commit-info should not be valid
			{
				Validator:          s.vals[0].toValidator(333),
				BlockIdFlag:        cmtproto.BlockIDFlagCommit,
				VoteExtension:      ext,
				ExtensionSignature: extSig0,
			},
			{
				Validator:   s.vals[1].toValidator(333),
				BlockIdFlag: cmtproto.BlockIDFlagNil,
			},
			{
				Validator:     s.vals[2].toValidator(334),
				VoteExtension: ext,
				BlockIdFlag:   cmtproto.BlockIDFlagAbsent,
			},
		},
	}

	s.ctx = s.ctx.WithBlockHeight(3) // vote-extensions are enabled

	// create last commit
	llc, info := extendedCommitToLastCommit(llc)
	s.ctx = s.ctx.WithCometInfo(info)

	// expect-pass (votes of height 2 are included in next block)
	s.Require().Error(baseapp.ValidateVoteExtensions(s.ctx, s.valStore, llc))
}

func (s *ABCIUtilsTestSuite) TestValidateVoteExtensionsIncorrectVotingPower() {
	ext := []byte("vote-extension")
	cve := cmtproto.CanonicalVoteExtension{
		Extension: ext,
		Height:    2,
		Round:     int64(0),
		ChainId:   chainID,
	}

	bz, err := marshalDelimitedFn(&cve)
	s.Require().NoError(err)

	extSig0, err := s.vals[0].privKey.Sign(bz)
	s.Require().NoError(err)

	llc := abci.ExtendedCommitInfo{
		Round: 0,
		Votes: []abci.ExtendedVoteInfo{
			// validator of power >2/3 is missing, so commit-info should not be valid
			{
				Validator:          s.vals[0].toValidator(333),
				BlockIdFlag:        cmtproto.BlockIDFlagCommit,
				VoteExtension:      ext,
				ExtensionSignature: extSig0,
			},
			{
				Validator:   s.vals[1].toValidator(333),
				BlockIdFlag: cmtproto.BlockIDFlagNil,
			},
			{
				Validator:     s.vals[2].toValidator(334),
				VoteExtension: ext,
				BlockIdFlag:   cmtproto.BlockIDFlagAbsent,
			},
		},
	}

	s.ctx = s.ctx.WithBlockHeight(3) // vote-extensions are enabled

	// create last commit
	llc, info := extendedCommitToLastCommit(llc)
	s.ctx = s.ctx.WithCometInfo(info)

	// modify voting powers to differ from the last-commit
	llc.Votes[0].Validator.Power = 335
	llc.Votes[2].Validator.Power = 332

	// expect-pass (votes of height 2 are included in next block)
	s.Require().Error(baseapp.ValidateVoteExtensions(s.ctx, s.valStore, llc))
}

func (s *ABCIUtilsTestSuite) TestValidateVoteExtensionsIncorrectOrder() {
	ext := []byte("vote-extension")
	cve := cmtproto.CanonicalVoteExtension{
		Extension: ext,
		Height:    2,
		Round:     int64(0),
		ChainId:   chainID,
	}

	bz, err := marshalDelimitedFn(&cve)
	s.Require().NoError(err)

	extSig0, err := s.vals[0].privKey.Sign(bz)
	s.Require().NoError(err)

	llc := abci.ExtendedCommitInfo{
		Round: 0,
		Votes: []abci.ExtendedVoteInfo{
			// validator of power >2/3 is missing, so commit-info should not be valid
			{
				Validator:          s.vals[0].toValidator(333),
				BlockIdFlag:        cmtproto.BlockIDFlagCommit,
				VoteExtension:      ext,
				ExtensionSignature: extSig0,
			},
			{
				Validator:   s.vals[1].toValidator(333),
				BlockIdFlag: cmtproto.BlockIDFlagNil,
			},
			{
				Validator:     s.vals[2].toValidator(334),
				VoteExtension: ext,
				BlockIdFlag:   cmtproto.BlockIDFlagAbsent,
			},
		},
	}

	s.ctx = s.ctx.WithBlockHeight(3) // vote-extensions are enabled

	// create last commit
	llc, info := extendedCommitToLastCommit(llc)
	s.ctx = s.ctx.WithCometInfo(info)

	// modify voting powers to differ from the last-commit
	llc.Votes[0], llc.Votes[2] = llc.Votes[2], llc.Votes[0]

	// expect-pass (votes of height 2 are included in next block)
	s.Require().Error(baseapp.ValidateVoteExtensions(s.ctx, s.valStore, llc))
}

func (s *ABCIUtilsTestSuite) TestDefaultProposalHandler_NoOpMempoolTxSelection() {
	// create a codec for marshaling
	cdc := codectestutil.CodecOptions{}.NewCodec()
	baseapptestutil.RegisterInterfaces(cdc.InterfaceRegistry())
	signingCtx := cdc.InterfaceRegistry().SigningContext()

	// create a baseapp along with a tx config for tx generation
	txConfig := authtx.NewTxConfig(cdc, signingCtx.AddressCodec(), signingCtx.ValidatorAddressCodec(), authtx.DefaultSignModes)
	app := baseapp.NewBaseApp(s.T().Name(), log.NewNopLogger(), coretesting.NewMemDB(), txConfig.TxDecoder())

	// create a proposal handler
	ph := baseapp.NewDefaultProposalHandler(mempool.NoOpMempool{}, app)
	handler := ph.PrepareProposalHandler()

	// build a tx
	_, _, addr := testdata.KeyTestPubAddr()
	addrStr, err := signingCtx.AddressCodec().BytesToString(addr)
	require.NoError(s.T(), err)
	builder := txConfig.NewTxBuilder()
	s.Require().NoError(builder.SetMsgs(
		&baseapptestutil.MsgCounter{Counter: 0, FailOnHandler: false, Signer: addrStr},
	))
	builder.SetGasLimit(100)
	setTxSignature(s.T(), builder, 0)

	// encode the tx to be used in the proposal request
	tx := builder.GetTx()
	txBz, err := txConfig.TxEncoder()(tx)
	s.Require().NoError(err)
	s.Require().Len(txBz, 165)

	txDataSize := int(cmttypes.ComputeProtoSizeForTxs([]cmttypes.Tx{txBz}))
	s.Require().Equal(txDataSize, 168)

	testCases := map[string]struct {
		ctx         sdk.Context
		req         *abci.PrepareProposalRequest
		expectedTxs int
	}{
		"small max tx bytes": {
			ctx: s.ctx,
			req: &abci.PrepareProposalRequest{
				Txs:        [][]byte{txBz, txBz, txBz, txBz, txBz},
				MaxTxBytes: 10,
			},
			expectedTxs: 0,
		},
		"small max gas": {
			ctx: s.ctx.WithConsensusParams(cmtproto.ConsensusParams{
				Block: &cmtproto.BlockParams{
					MaxGas: 10,
				},
			}),
			req: &abci.PrepareProposalRequest{
				Txs:        [][]byte{txBz, txBz, txBz, txBz, txBz},
				MaxTxBytes: 465,
			},
			expectedTxs: 0,
		},
		"large max tx bytes": {
			ctx: s.ctx,
			req: &abci.PrepareProposalRequest{
				Txs:        [][]byte{txBz, txBz, txBz, txBz, txBz},
				MaxTxBytes: 465,
			},
			expectedTxs: 2,
		},
		"large max tx bytes len calculation": {
			ctx: s.ctx,
			req: &abci.PrepareProposalRequest{
				Txs:        [][]byte{txBz, txBz, txBz, txBz, txBz},
				MaxTxBytes: 504,
			},
			expectedTxs: 3,
		},
		"max gas and tx bytes": {
			ctx: s.ctx.WithConsensusParams(cmtproto.ConsensusParams{
				Block: &cmtproto.BlockParams{
					MaxGas: 200,
				},
			}),
			req: &abci.PrepareProposalRequest{
				Txs:        [][]byte{txBz, txBz, txBz, txBz, txBz},
				MaxTxBytes: 465,
			},
			expectedTxs: 2,
		},
	}

	for name, tc := range testCases {
		s.Run(name, func() {
			// iterate multiple times to ensure the tx selector is cleared each time
			for i := 0; i < 6; i++ {
				resp, err := handler(tc.ctx, tc.req)
				s.Require().NoError(err)
				s.Require().Len(resp.Txs, tc.expectedTxs)
			}
		})
	}
}

func (s *ABCIUtilsTestSuite) TestDefaultProposalHandler_PriorityNonceMempoolTxSelection() {
	cdc := codectestutil.CodecOptions{}.NewCodec()
	baseapptestutil.RegisterInterfaces(cdc.InterfaceRegistry())
	signingCtx := cdc.InterfaceRegistry().SigningContext()
	txConfig := authtx.NewTxConfig(cdc, signingCtx.AddressCodec(), signingCtx.ValidatorAddressCodec(), authtx.DefaultSignModes)

	var (
		secret1 = []byte("secret1")
		secret2 = []byte("secret2")
		secret3 = []byte("secret3")
		secret4 = []byte("secret4")
		secret5 = []byte("secret5")
		secret6 = []byte("secret6")
	)

	type testTx struct {
		tx       sdk.Tx
		priority int64
		bz       []byte
		size     int
	}

	testTxs := []testTx{
		// test 1
		{tx: buildMsg(s.T(), txConfig, signingCtx.AddressCodec(), []byte(`0`), [][]byte{secret1}, []uint64{1}), priority: 10},
		{tx: buildMsg(s.T(), txConfig, signingCtx.AddressCodec(), []byte(`12345678910`), [][]byte{secret1}, []uint64{2}), priority: 10},
		{tx: buildMsg(s.T(), txConfig, signingCtx.AddressCodec(), []byte(`22`), [][]byte{secret1}, []uint64{3}), priority: 10},
		{tx: buildMsg(s.T(), txConfig, signingCtx.AddressCodec(), []byte(`32`), [][]byte{secret2}, []uint64{1}), priority: 8},
		// test 2
		{tx: buildMsg(s.T(), txConfig, signingCtx.AddressCodec(), []byte(`4`), [][]byte{secret1, secret2}, []uint64{3, 3}), priority: 10},
		{tx: buildMsg(s.T(), txConfig, signingCtx.AddressCodec(), []byte(`52345678910`), [][]byte{secret1, secret3}, []uint64{4, 3}), priority: 10},
		{tx: buildMsg(s.T(), txConfig, signingCtx.AddressCodec(), []byte(`62`), [][]byte{secret1, secret4}, []uint64{5, 3}), priority: 8},
		{tx: buildMsg(s.T(), txConfig, signingCtx.AddressCodec(), []byte(`72`), [][]byte{secret3, secret5}, []uint64{4, 3}), priority: 8},
		{tx: buildMsg(s.T(), txConfig, signingCtx.AddressCodec(), []byte(`82`), [][]byte{secret2, secret6}, []uint64{4, 3}), priority: 8},
		// test 3
		{tx: buildMsg(s.T(), txConfig, signingCtx.AddressCodec(), []byte(`9`), [][]byte{secret3, secret4}, []uint64{3, 3}), priority: 10},
		{tx: buildMsg(s.T(), txConfig, signingCtx.AddressCodec(), []byte(`1052345678910`), [][]byte{secret1, secret2}, []uint64{4, 4}), priority: 8},
		{tx: buildMsg(s.T(), txConfig, signingCtx.AddressCodec(), []byte(`11`), [][]byte{secret1, secret2}, []uint64{5, 5}), priority: 8},
		// test 4
		{tx: buildMsg(s.T(), txConfig, signingCtx.AddressCodec(), []byte(`1252345678910`), [][]byte{secret1}, []uint64{3}), priority: 10},
		{tx: buildMsg(s.T(), txConfig, signingCtx.AddressCodec(), []byte(`13`), [][]byte{secret1}, []uint64{5}), priority: 10},
		{tx: buildMsg(s.T(), txConfig, signingCtx.AddressCodec(), []byte(`14`), [][]byte{secret1}, []uint64{6}), priority: 8},
	}

	for i := range testTxs {
		bz, err := txConfig.TxEncoder()(testTxs[i].tx)
		s.Require().NoError(err)
		testTxs[i].bz = bz
		testTxs[i].size = int(cmttypes.ComputeProtoSizeForTxs([]cmttypes.Tx{bz}))
	}

	s.Require().Equal(193, testTxs[0].size)
	s.Require().Equal(203, testTxs[1].size)
	s.Require().Equal(194, testTxs[2].size)
	s.Require().Equal(194, testTxs[3].size)
	s.Require().Equal(276, testTxs[4].size)
	s.Require().Equal(286, testTxs[5].size)
	s.Require().Equal(277, testTxs[6].size)
	s.Require().Equal(277, testTxs[7].size)
	s.Require().Equal(277, testTxs[8].size)

	testCases := map[string]struct {
		ctx         sdk.Context
		txInputs    []testTx
		req         *abci.PrepareProposalRequest
		handler     sdk.PrepareProposalHandler
		expectedTxs []int
	}{
		"skip same-sender non-sequential sequence and then add others txs": {
			ctx:      s.ctx,
			txInputs: []testTx{testTxs[0], testTxs[1], testTxs[2], testTxs[3]},
			req: &abci.PrepareProposalRequest{
				MaxTxBytes: 193 + 194,
			},
			expectedTxs: []int{0, 3},
		},
		"skip multi-signers msg non-sequential sequence": {
			ctx:      s.ctx,
			txInputs: []testTx{testTxs[4], testTxs[5], testTxs[6], testTxs[7], testTxs[8]},
			req: &abci.PrepareProposalRequest{
				MaxTxBytes: 276 + 277,
			},
			expectedTxs: []int{4, 8},
		},
		"only the first tx is added": {
			// Because tx 10 is valid, tx 11 can't be valid as they have higher sequence numbers.
			ctx:      s.ctx,
			txInputs: []testTx{testTxs[9], testTxs[10], testTxs[11]},
			req: &abci.PrepareProposalRequest{
				MaxTxBytes: 276 + 277,
			},
			expectedTxs: []int{9},
		},
		"no txs added": {
			// Because the first tx was deemed valid but too big, the next expected valid sequence is tx[0].seq (3), so
			// the rest of the txs fail because they have a seq of 4.
			ctx:      s.ctx,
			txInputs: []testTx{testTxs[12], testTxs[13], testTxs[14]},
			req: &abci.PrepareProposalRequest{
				MaxTxBytes: 112,
			},
			expectedTxs: []int{},
		},
	}

	for name, tc := range testCases {
		s.Run(name, func() {
			ctrl := gomock.NewController(s.T())
			app := mock.NewMockProposalTxVerifier(ctrl)
			mp := mempool.NewPriorityMempool(
				mempool.PriorityNonceMempoolConfig[int64]{
					TxPriority:      mempool.NewDefaultTxPriority(),
					MaxTx:           0,
					SignerExtractor: mempool.NewDefaultSignerExtractionAdapter(),
				},
			)

			ph := baseapp.NewDefaultProposalHandler(mp, app)

			for _, v := range tc.txInputs {
				app.EXPECT().TxDecode(v.bz).Return(v.tx, nil).AnyTimes()
				app.EXPECT().PrepareProposalVerifyTx(v.tx).Return(v.bz, nil).AnyTimes()
				s.NoError(mp.Insert(s.ctx.WithPriority(v.priority), v.tx))
				tc.req.Txs = append(tc.req.Txs, v.bz)
			}

			resp, err := ph.PrepareProposalHandler()(tc.ctx, tc.req)
			s.Require().NoError(err)
			respTxIndexes := []int{}
			for _, tx := range resp.Txs {
				for i, v := range testTxs {
					if bytes.Equal(tx, v.bz) {
						respTxIndexes = append(respTxIndexes, i)
					}
				}
			}

			s.Require().EqualValues(tc.expectedTxs, respTxIndexes)
		})
	}
}

func marshalDelimitedFn(msg proto.Message) ([]byte, error) {
	var buf bytes.Buffer
	if err := protoio.NewDelimitedWriter(&buf).WriteMsg(msg); err != nil {
		return nil, err
	}

	return buf.Bytes(), nil
}

func buildMsg(t *testing.T, txConfig client.TxConfig, ac address.Codec, value []byte, secrets [][]byte, nonces []uint64) sdk.Tx {
	t.Helper()
	builder := txConfig.NewTxBuilder()

	require.Equal(t, len(secrets), len(nonces))
	signatures := make([]signingtypes.SignatureV2, 0)
	for index, secret := range secrets {
		nonce := nonces[index]
		privKey := secp256k1.GenPrivKeyFromSecret(secret)
		pubKey := privKey.PubKey()
		signatures = append(signatures, signingtypes.SignatureV2{
			PubKey:   pubKey,
			Sequence: nonce,
			Data:     &signingtypes.SingleSignatureData{},
		})
	}

	addr, err := ac.BytesToString(signatures[0].PubKey.Bytes())
	require.NoError(t, err)

	_ = builder.SetMsgs(
		&baseapptestutil.MsgKeyValue{
			Signer: addr,
			Value:  value,
		},
	)

	setTxSignatureWithSecret(t, builder, signatures...)
	return builder.GetTx()
}

func setTxSignatureWithSecret(t *testing.T, builder client.TxBuilder, signatures ...signingtypes.SignatureV2) {
	t.Helper()
	err := builder.SetSignatures(
		signatures...,
	)
	require.NoError(t, err)
}

func extendedCommitToLastCommit(ec abci.ExtendedCommitInfo) (abci.ExtendedCommitInfo, comet.Info) {
	// sort the extended commit info
	sort.Sort(extendedVoteInfos(ec.Votes))

	// convert the extended commit info to last commit info
	lastCommit := comet.CommitInfo{
		Round: ec.Round,
		Votes: make([]comet.VoteInfo, len(ec.Votes)),
	}

	for i, vote := range ec.Votes {
		lastCommit.Votes[i] = comet.VoteInfo{
			Validator: comet.Validator{
				Address: vote.Validator.Address,
				Power:   vote.Validator.Power,
			},
		}
	}

	return ec, comet.Info{
		LastCommit: lastCommit,
	}
}

type extendedVoteInfos []abci.ExtendedVoteInfo

func (v extendedVoteInfos) Len() int {
	return len(v)
}

func (v extendedVoteInfos) Less(i, j int) bool {
	if v[i].Validator.Power == v[j].Validator.Power {
		return bytes.Compare(v[i].Validator.Address, v[j].Validator.Address) == -1
	}
	return v[i].Validator.Power > v[j].Validator.Power
}

func (v extendedVoteInfos) Swap(i, j int) {
	v[i], v[j] = v[j], v[i]
}

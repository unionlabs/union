package ante_test

import (
	"context"
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/require"

	// TODO We don't need to import these API types if we use gogo's registry
	// ref: https://github.com/cosmos/cosmos-sdk/issues/14647
	_ "cosmossdk.io/api/cosmos/bank/v1beta1"
	_ "cosmossdk.io/api/cosmos/crypto/secp256k1"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/header"
	coretesting "cosmossdk.io/core/testing"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/tx"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
	clitestutil "github.com/cosmos/cosmos-sdk/testutil/cli"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	_ "github.com/cosmos/cosmos-sdk/testutil/testdata/testpb"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
	"github.com/cosmos/cosmos-sdk/x/auth"
	"github.com/cosmos/cosmos-sdk/x/auth/ante"
	antetestutil "github.com/cosmos/cosmos-sdk/x/auth/ante/testutil"
	authcodec "github.com/cosmos/cosmos-sdk/x/auth/codec"
	"github.com/cosmos/cosmos-sdk/x/auth/keeper"
	xauthsigning "github.com/cosmos/cosmos-sdk/x/auth/signing"
	authtestutil "github.com/cosmos/cosmos-sdk/x/auth/testutil"
	txtestutil "github.com/cosmos/cosmos-sdk/x/auth/tx/testutil"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

// TestAccount represents an account used in the tests in x/auth/ante.
type TestAccount struct {
	acc  sdk.AccountI
	priv cryptotypes.PrivKey
}

// AnteTestSuite is a test suite to be used with ante handler tests.
type AnteTestSuite struct {
	env             appmodule.Environment
	anteHandler     sdk.AnteHandler
	ctx             sdk.Context
	clientCtx       client.Context
	txBuilder       client.TxBuilder
	accountKeeper   keeper.AccountKeeper
	bankKeeper      *authtestutil.MockBankKeeper
	consensusKeeper *antetestutil.MockConsensusKeeper
	acctsModKeeper  *authtestutil.MockAccountsModKeeper
	txBankKeeper    *txtestutil.MockBankKeeper
	feeGrantKeeper  *antetestutil.MockFeegrantKeeper
	encCfg          moduletestutil.TestEncodingConfig
}

// SetupTest setups a new test, with new app, context, and anteHandler.
func SetupTestSuite(t *testing.T, isCheckTx bool) *AnteTestSuite {
	t.Helper()
	suite := &AnteTestSuite{}
	// gomock initializations
	ctrl := gomock.NewController(t)
	suite.bankKeeper = authtestutil.NewMockBankKeeper(ctrl)
	suite.txBankKeeper = txtestutil.NewMockBankKeeper(ctrl)
	suite.feeGrantKeeper = antetestutil.NewMockFeegrantKeeper(ctrl)
	suite.acctsModKeeper = authtestutil.NewMockAccountsModKeeper(ctrl)

	key := storetypes.NewKVStoreKey(types.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	suite.ctx = testCtx.Ctx.WithIsCheckTx(isCheckTx).WithBlockHeight(1).WithHeaderInfo(header.Info{Height: 1, ChainID: testCtx.Ctx.ChainID()})
	suite.encCfg = moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, auth.AppModule{})

	accNum := uint64(0)
	suite.acctsModKeeper.EXPECT().NextAccountNumber(gomock.Any()).AnyTimes().DoAndReturn(func(ctx context.Context) (uint64, error) {
		currNum := accNum
		accNum++
		return currNum, nil
	})
	maccPerms := map[string][]string{
		"fee_collector":          nil,
		"mint":                   {"minter"},
		"bonded_tokens_pool":     {"burner", "staking"},
		"not_bonded_tokens_pool": {"burner", "staking"},
		"multiPerm":              {"burner", "minter", "staking"},
		"random":                 {"random"},
	}

	msgRouter := baseapp.NewMsgServiceRouter()
	grpcQueryRouter := baseapp.NewGRPCQueryRouter()
	grpcQueryRouter.SetInterfaceRegistry(suite.encCfg.InterfaceRegistry)

	suite.consensusKeeper = antetestutil.NewMockConsensusKeeper(ctrl)
	suite.consensusKeeper.EXPECT().BlockParams(gomock.Any()).Return(uint64(simtestutil.DefaultConsensusParams.Block.MaxGas), uint64(simtestutil.DefaultConsensusParams.Block.MaxBytes), nil).AnyTimes()

	suite.env = runtime.NewEnvironment(runtime.NewKVStoreService(key), coretesting.NewNopLogger(), runtime.EnvWithQueryRouterService(grpcQueryRouter), runtime.EnvWithMsgRouterService(msgRouter))
	suite.accountKeeper = keeper.NewAccountKeeper(
		runtime.NewEnvironment(runtime.NewKVStoreService(key), coretesting.NewNopLogger()), suite.encCfg.Codec, types.ProtoBaseAccount, suite.acctsModKeeper, maccPerms, authcodec.NewBech32Codec("cosmos"),
		sdk.Bech32MainPrefix, types.NewModuleAddress("gov").String(),
	)
	suite.accountKeeper.GetModuleAccount(suite.ctx, types.FeeCollectorName)
	err := suite.accountKeeper.Params.Set(suite.ctx, types.DefaultParams())
	require.NoError(t, err)

	// We're using TestMsg encoding in some tests, so register it here.
	suite.encCfg.Amino.RegisterConcrete(&testdata.TestMsg{}, "testdata.TestMsg")
	testdata.RegisterInterfaces(suite.encCfg.InterfaceRegistry)

	suite.clientCtx = client.Context{}.
		WithTxConfig(suite.encCfg.TxConfig).
		WithClient(clitestutil.NewMockCometRPCWithResponseQueryValue(nil))

	anteHandler, err := ante.NewAnteHandler(
		ante.HandlerOptions{
			AccountKeeper:   suite.accountKeeper,
			BankKeeper:      suite.bankKeeper,
			ConsensusKeeper: suite.consensusKeeper,
			FeegrantKeeper:  suite.feeGrantKeeper,
			SignModeHandler: suite.encCfg.TxConfig.SignModeHandler(),
			SigGasConsumer:  ante.DefaultSigVerificationGasConsumer,
			Environment:     suite.env,
		},
	)

	require.NoError(t, err)
	suite.anteHandler = anteHandler

	suite.txBuilder = suite.clientCtx.TxConfig.NewTxBuilder()

	return suite
}

func (suite *AnteTestSuite) CreateTestAccounts(numAccs int) []TestAccount {
	var accounts []TestAccount

	for i := 0; i < numAccs; i++ {
		priv, _, addr := testdata.KeyTestPubAddr()
		acc := suite.accountKeeper.NewAccountWithAddress(suite.ctx, addr)
		err := acc.SetAccountNumber(uint64(i + 1000))
		if err != nil {
			panic(err)
		}
		suite.accountKeeper.SetAccount(suite.ctx, acc)
		accounts = append(accounts, TestAccount{acc, priv})
	}

	return accounts
}

// TestCase represents a test case used in test tables.
type TestCase struct {
	desc     string
	malleate func(*AnteTestSuite) TestCaseArgs
	simulate bool
	expPass  bool
	expErr   error
}

type TestCaseArgs struct {
	chainID   string
	accNums   []uint64
	accSeqs   []uint64
	feeAmount sdk.Coins
	gasLimit  uint64
	msgs      []sdk.Msg
	privs     []cryptotypes.PrivKey
}

func (t TestCaseArgs) WithAccountsInfo(accs []TestAccount) TestCaseArgs {
	newT := t
	for _, acc := range accs {
		newT.accNums = append(newT.accNums, acc.acc.GetAccountNumber())
		newT.accSeqs = append(newT.accSeqs, acc.acc.GetSequence())
		newT.privs = append(newT.privs, acc.priv)
	}
	return newT
}

// DeliverMsgs constructs a tx and runs it through the ante handler. This is used to set the context for a test case, for
// example to test for replay protection.
func (suite *AnteTestSuite) DeliverMsgs(t *testing.T, privs []cryptotypes.PrivKey, msgs []sdk.Msg, feeAmount sdk.Coins, gasLimit uint64, accNums, accSeqs []uint64, chainID string, simulate bool) (sdk.Context, error) {
	t.Helper()
	require.NoError(t, suite.txBuilder.SetMsgs(msgs...))
	suite.txBuilder.SetFeeAmount(feeAmount)
	suite.txBuilder.SetGasLimit(gasLimit)

	tx, txErr := suite.CreateTestTx(suite.ctx, privs, accNums, accSeqs, chainID, signing.SignMode_SIGN_MODE_DIRECT)
	require.NoError(t, txErr)
	txBytes, err := suite.clientCtx.TxConfig.TxEncoder()(tx)
	bytesCtx := suite.ctx.WithTxBytes(txBytes)
	require.NoError(t, err)
	return suite.anteHandler(bytesCtx, tx, simulate)
}

func (suite *AnteTestSuite) RunTestCase(t *testing.T, tc TestCase, args TestCaseArgs) {
	t.Helper()
	require.NoError(t, suite.txBuilder.SetMsgs(args.msgs...))
	suite.txBuilder.SetFeeAmount(args.feeAmount)
	suite.txBuilder.SetGasLimit(args.gasLimit)

	// Theoretically speaking, ante handler unit tests should only test
	// ante handlers, but here we sometimes also test the tx creation
	// process.
	tx, txErr := suite.CreateTestTx(suite.ctx, args.privs, args.accNums, args.accSeqs, args.chainID, signing.SignMode_SIGN_MODE_DIRECT)
	txBytes, err := suite.clientCtx.TxConfig.TxEncoder()(tx)
	require.NoError(t, err)
	bytesCtx := suite.ctx.WithTxBytes(txBytes)
	newCtx, anteErr := suite.anteHandler(bytesCtx, tx, tc.simulate)

	if tc.expPass {
		require.NoError(t, txErr)
		require.NoError(t, anteErr)
		require.NotNil(t, newCtx)

		suite.ctx = newCtx
	} else {
		switch {
		case txErr != nil:
			require.Error(t, txErr)
			require.ErrorIs(t, txErr, tc.expErr)

		case anteErr != nil:
			require.Error(t, anteErr)
			require.ErrorIs(t, anteErr, tc.expErr)

		default:
			t.Fatal("expected one of txErr, anteErr to be an error")
		}
	}
}

// CreateTestTx is a helper function to create a tx given multiple inputs.
func (suite *AnteTestSuite) CreateTestTx(
	ctx sdk.Context, privs []cryptotypes.PrivKey,
	accNums, accSeqs []uint64,
	chainID string, signMode signing.SignMode,
) (xauthsigning.Tx, error) {
	// First round: we gather all the signer infos. We use the "set empty
	// signature" hack to do that.
	var sigsV2 []signing.SignatureV2
	for i, priv := range privs {
		sigV2 := signing.SignatureV2{
			PubKey: priv.PubKey(),
			Data: &signing.SingleSignatureData{
				SignMode:  signMode,
				Signature: nil,
			},
			Sequence: accSeqs[i],
		}

		sigsV2 = append(sigsV2, sigV2)
	}
	err := suite.txBuilder.SetSignatures(sigsV2...)
	if err != nil {
		return nil, err
	}

	// Second round: all signer infos are set, so each signer can sign.
	sigsV2 = []signing.SignatureV2{}
	for i, priv := range privs {
		signerData := xauthsigning.SignerData{
			Address:       sdk.AccAddress(priv.PubKey().Address()).String(),
			ChainID:       chainID,
			AccountNumber: accNums[i],
			Sequence:      accSeqs[i],
			PubKey:        priv.PubKey(),
		}
		sigV2, err := tx.SignWithPrivKey(
			ctx, signMode, signerData,
			suite.txBuilder, priv, suite.clientCtx.TxConfig, accSeqs[i])
		if err != nil {
			return nil, err
		}

		sigsV2 = append(sigsV2, sigV2)
	}
	err = suite.txBuilder.SetSignatures(sigsV2...)
	if err != nil {
		return nil, err
	}

	return suite.txBuilder.GetTx(), nil
}

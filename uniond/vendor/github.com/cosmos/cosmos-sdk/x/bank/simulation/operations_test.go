package simulation_test

import (
	"math/rand"
	"testing"

	abci "github.com/cometbft/cometbft/abci/types"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil/configurator"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	_ "github.com/cosmos/cosmos-sdk/x/auth"
	_ "github.com/cosmos/cosmos-sdk/x/auth/tx/config"
	_ "github.com/cosmos/cosmos-sdk/x/bank"
	"github.com/cosmos/cosmos-sdk/x/bank/keeper"
	"github.com/cosmos/cosmos-sdk/x/bank/simulation"
	"github.com/cosmos/cosmos-sdk/x/bank/testutil"
	"github.com/cosmos/cosmos-sdk/x/bank/types"
	_ "github.com/cosmos/cosmos-sdk/x/consensus"
	_ "github.com/cosmos/cosmos-sdk/x/params"
	_ "github.com/cosmos/cosmos-sdk/x/staking"
)

type SimTestSuite struct {
	suite.Suite

	ctx           sdk.Context
	accountKeeper types.AccountKeeper
	bankKeeper    keeper.Keeper
	cdc           codec.Codec
	app           *runtime.App
}

func (suite *SimTestSuite) SetupTest() {
	var (
		appBuilder *runtime.AppBuilder
		err        error
	)
	suite.app, err = simtestutil.Setup(configurator.NewAppConfig(
		configurator.AuthModule(),
		configurator.ParamsModule(),
		configurator.BankModule(),
		configurator.StakingModule(),
		configurator.ConsensusModule(),
		configurator.TxModule(),
	), &suite.accountKeeper, &suite.bankKeeper, &suite.cdc, &appBuilder)

	suite.NoError(err)

	suite.ctx = suite.app.BaseApp.NewContext(false, tmproto.Header{})
}

// TestWeightedOperations tests the weights of the operations.
func (suite *SimTestSuite) TestWeightedOperations() {
	cdc := suite.cdc
	appParams := make(simtypes.AppParams)

	weightesOps := simulation.WeightedOperations(appParams, cdc, suite.accountKeeper, suite.bankKeeper)

	// setup 3 accounts
	s := rand.NewSource(1)
	r := rand.New(s)
	accs := suite.getTestingAccounts(r, 3)

	expected := []struct {
		weight     int
		opMsgRoute string
		opMsgName  string
	}{
		{100, types.ModuleName, types.TypeMsgSend},
		{10, types.ModuleName, types.TypeMsgMultiSend},
	}

	for i, w := range weightesOps {
		operationMsg, _, err := w.Op()(r, suite.app.BaseApp, suite.ctx, accs, "")
		suite.Require().NoError(err)

		// the following checks are very much dependent from the ordering of the output given
		// by WeightedOperations. if the ordering in WeightedOperations changes some tests
		// will fail
		suite.Require().Equal(expected[i].weight, w.Weight(), "weight should be the same")
		suite.Require().Equal(expected[i].opMsgRoute, operationMsg.Route, "route should be the same")
		suite.Require().Equal(expected[i].opMsgName, operationMsg.Name, "operation Msg name should be the same")
	}
}

// TestSimulateMsgSend tests the normal scenario of a valid message of type TypeMsgSend.
// Abonormal scenarios, where the message is created by an errors, are not tested here.
func (suite *SimTestSuite) TestSimulateMsgSend() {
	// setup 3 accounts
	s := rand.NewSource(1)
	r := rand.New(s)
	accounts := suite.getTestingAccounts(r, 3)

	// begin a new block
	suite.app.BeginBlock(abci.RequestBeginBlock{Header: tmproto.Header{Height: suite.app.LastBlockHeight() + 1, AppHash: suite.app.LastCommitID().Hash}})

	// execute operation
	op := simulation.SimulateMsgSend(suite.accountKeeper, suite.bankKeeper)
	operationMsg, futureOperations, err := op(r, suite.app.BaseApp, suite.ctx, accounts, "")
	suite.Require().NoError(err)

	var msg types.MsgSend
	types.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)

	suite.Require().True(operationMsg.OK)
	suite.Require().Equal("65337742stake", msg.Amount.String())
	suite.Require().Equal("cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r", msg.FromAddress)
	suite.Require().Equal("cosmos1p8wcgrjr4pjju90xg6u9cgq55dxwq8j7u4x9a0", msg.ToAddress)
	suite.Require().Equal(types.TypeMsgSend, msg.Type())
	suite.Require().Equal(types.ModuleName, msg.Route())
	suite.Require().Len(futureOperations, 0)
}

// TestSimulateMsgSend tests the normal scenario of a valid message of type TypeMsgMultiSend.
// Abonormal scenarios, where the message is created by an errors, are not tested here.
func (suite *SimTestSuite) TestSimulateMsgMultiSend() {
	// setup 3 accounts
	s := rand.NewSource(1)
	r := rand.New(s)
	accounts := suite.getTestingAccounts(r, 3)

	// begin a new block
	suite.app.BeginBlock(abci.RequestBeginBlock{Header: tmproto.Header{Height: suite.app.LastBlockHeight() + 1, AppHash: suite.app.LastCommitID().Hash}})

	// execute operation
	op := simulation.SimulateMsgMultiSend(suite.accountKeeper, suite.bankKeeper)
	operationMsg, futureOperations, err := op(r, suite.app.BaseApp, suite.ctx, accounts, "")
	require := suite.Require()
	require.NoError(err)

	var msg types.MsgMultiSend
	types.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)

	require.True(operationMsg.OK)
	require.Len(msg.Inputs, 1)
	require.Equal("cosmos1tnh2q55v8wyygtt9srz5safamzdengsnqeycj3", msg.Inputs[0].Address)
	require.Equal("114949958stake", msg.Inputs[0].Coins.String())
	require.Len(msg.Outputs, 2)
	require.Equal("cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r", msg.Outputs[1].Address)
	require.Equal("107287087stake", msg.Outputs[1].Coins.String())
	require.Equal(types.TypeMsgMultiSend, msg.Type())
	require.Equal(types.ModuleName, msg.Route())
	require.Len(futureOperations, 0)
}

func (suite *SimTestSuite) TestSimulateModuleAccountMsgSend() {
	const (
		accCount       = 1
		moduleAccCount = 1
	)

	s := rand.NewSource(1)
	r := rand.New(s)
	accounts := suite.getTestingAccounts(r, accCount)

	// begin a new block
	suite.app.BeginBlock(abci.RequestBeginBlock{Header: tmproto.Header{Height: suite.app.LastBlockHeight() + 1, AppHash: suite.app.LastCommitID().Hash}})

	// execute operation
	op := simulation.SimulateMsgSendToModuleAccount(suite.accountKeeper, suite.bankKeeper, moduleAccCount)

	s = rand.NewSource(1)
	r = rand.New(s)

	operationMsg, futureOperations, err := op(r, suite.app.BaseApp, suite.ctx, accounts, "")
	suite.Require().Error(err)

	var msg types.MsgSend
	types.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)

	suite.Require().False(operationMsg.OK)
	suite.Require().Equal(operationMsg.Comment, "invalid transfers")
	suite.Require().Equal(types.TypeMsgSend, msg.Type())
	suite.Require().Equal(types.ModuleName, msg.Route())
	suite.Require().Len(futureOperations, 0)
}

func (suite *SimTestSuite) TestSimulateMsgMultiSendToModuleAccount() {
	const (
		accCount  = 2
		mAccCount = 2
	)

	s := rand.NewSource(1)
	r := rand.New(s)
	accounts := suite.getTestingAccounts(r, accCount)

	// begin a new block
	suite.app.BeginBlock(abci.RequestBeginBlock{Header: tmproto.Header{Height: suite.app.LastBlockHeight() + 1, AppHash: suite.app.LastCommitID().Hash}})

	// execute operation
	op := simulation.SimulateMsgMultiSendToModuleAccount(suite.accountKeeper, suite.bankKeeper, mAccCount)

	operationMsg, futureOperations, err := op(r, suite.app.BaseApp, suite.ctx, accounts, "")
	suite.Require().Error(err)

	var msg types.MsgMultiSend
	types.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)

	suite.Require().False(operationMsg.OK) // sending tokens to a module account should fail
	suite.Require().Equal(operationMsg.Comment, "invalid transfers")
	suite.Require().Equal(types.TypeMsgMultiSend, msg.Type())
	suite.Require().Equal(types.ModuleName, msg.Route())
	suite.Require().Len(futureOperations, 0)
}

func (suite *SimTestSuite) getTestingAccounts(r *rand.Rand, n int) []simtypes.Account {
	accounts := simtypes.RandomAccounts(r, n)

	initAmt := sdk.TokensFromConsensusPower(200, sdk.DefaultPowerReduction)
	initCoins := sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, initAmt))

	// add coins to the accounts
	for _, account := range accounts {
		acc := suite.accountKeeper.NewAccountWithAddress(suite.ctx, account.Address)
		suite.accountKeeper.SetAccount(suite.ctx, acc)
		suite.Require().NoError(testutil.FundAccount(suite.bankKeeper, suite.ctx, account.Address, initCoins))
	}

	return accounts
}

func TestSimTestSuite(t *testing.T) {
	suite.Run(t, new(SimTestSuite))
}

package simulation_test

import (
	"math/big"
	"math/rand"
	"testing"
	"time"

	"cosmossdk.io/math"
	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"

	abci "github.com/cometbft/cometbft/abci/types"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	tmtypes "github.com/cometbft/cometbft/types"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/runtime"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/staking/simulation"
	"github.com/cosmos/cosmos-sdk/x/staking/testutil"

	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	banktestutil "github.com/cosmos/cosmos-sdk/x/bank/testutil"

	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	distrtypes "github.com/cosmos/cosmos-sdk/x/distribution/types"
	minttypes "github.com/cosmos/cosmos-sdk/x/mint/types"
	"github.com/cosmos/cosmos-sdk/x/staking/types"

	authkeeper "github.com/cosmos/cosmos-sdk/x/auth/keeper"
	bankkeeper "github.com/cosmos/cosmos-sdk/x/bank/keeper"
	distrkeeper "github.com/cosmos/cosmos-sdk/x/distribution/keeper"
	mintkeeper "github.com/cosmos/cosmos-sdk/x/mint/keeper"
	stakingkeeper "github.com/cosmos/cosmos-sdk/x/staking/keeper"
)

type SimTestSuite struct {
	suite.Suite

	r             *rand.Rand
	accounts      []simtypes.Account
	ctx           sdk.Context
	app           *runtime.App
	bankKeeper    bankkeeper.Keeper
	accountKeeper authkeeper.AccountKeeper
	distrKeeper   distrkeeper.Keeper
	stakingKeeper *stakingkeeper.Keeper

	encCfg moduletestutil.TestEncodingConfig
}

func (s *SimTestSuite) SetupTest() {
	sdk.DefaultPowerReduction = sdk.NewIntFromBigInt(new(big.Int).Exp(big.NewInt(10), big.NewInt(18), nil))

	s.r = rand.New(rand.NewSource(1))
	accounts := simtypes.RandomAccounts(s.r, 4)

	// create genesis accounts
	senderPrivKey := secp256k1.GenPrivKey()
	acc := authtypes.NewBaseAccount(senderPrivKey.PubKey().Address().Bytes(), senderPrivKey.PubKey(), 0, 0)
	accs := []simtestutil.GenesisAccount{
		{GenesisAccount: acc, Coins: sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdk.NewInt(100000000000000)))},
	}

	// create validator set with single validator
	account := accounts[0]
	tmPk, err := cryptocodec.ToTmPubKeyInterface(account.PubKey)
	require.NoError(s.T(), err)
	validator := tmtypes.NewValidator(tmPk, 1)

	startupCfg := simtestutil.DefaultStartUpConfig()
	startupCfg.GenesisAccounts = accs
	startupCfg.ValidatorSet = func() (*tmtypes.ValidatorSet, error) {
		return tmtypes.NewValidatorSet([]*tmtypes.Validator{validator}), nil
	}

	var (
		accountKeeper authkeeper.AccountKeeper
		mintKeeper    mintkeeper.Keeper
		bankKeeper    bankkeeper.Keeper
		distrKeeper   distrkeeper.Keeper
		stakingKeeper *stakingkeeper.Keeper
	)

	app, err := simtestutil.SetupWithConfiguration(testutil.AppConfig, startupCfg, &bankKeeper, &accountKeeper, &mintKeeper, &distrKeeper, &stakingKeeper)
	require.NoError(s.T(), err)

	ctx := app.BaseApp.NewContext(false, tmproto.Header{})
	mintKeeper.SetParams(ctx, minttypes.DefaultParams())
	mintKeeper.SetMinter(ctx, minttypes.DefaultInitialMinter())

	initAmt := stakingKeeper.TokensFromConsensusPower(ctx, 200)
	initCoins := sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, initAmt))

	s.accounts = accounts
	// remove genesis validator account
	// add coins to the accounts
	for _, account := range accounts[1:] {
		acc := accountKeeper.NewAccountWithAddress(ctx, account.Address)
		accountKeeper.SetAccount(ctx, acc)
		s.Require().NoError(banktestutil.FundAccount(bankKeeper, ctx, account.Address, initCoins))
	}

	s.accountKeeper = accountKeeper
	s.bankKeeper = bankKeeper
	s.distrKeeper = distrKeeper
	s.stakingKeeper = stakingKeeper
	s.ctx = ctx
	s.app = app
}

// TestWeightedOperations tests the weights of the operations.
func (s *SimTestSuite) TestWeightedOperations() {
	require := s.Require()

	s.ctx.WithChainID("test-chain")

	cdc := s.encCfg.Codec
	appParams := make(simtypes.AppParams)

	weightesOps := simulation.WeightedOperations(appParams, cdc, s.accountKeeper,
		s.bankKeeper, s.stakingKeeper,
	)

	expected := []struct {
		weight     int
		opMsgRoute string
		opMsgName  string
	}{
		{simulation.DefaultWeightMsgCreateValidator, types.ModuleName, types.TypeMsgCreateValidator},
		{simulation.DefaultWeightMsgEditValidator, types.ModuleName, types.TypeMsgEditValidator},
		{simulation.DefaultWeightMsgDelegate, types.ModuleName, types.TypeMsgDelegate},
		{simulation.DefaultWeightMsgUndelegate, types.ModuleName, types.TypeMsgUndelegate},
		{simulation.DefaultWeightMsgBeginRedelegate, types.ModuleName, types.TypeMsgBeginRedelegate},
		{simulation.DefaultWeightMsgCancelUnbondingDelegation, types.ModuleName, types.TypeMsgCancelUnbondingDelegation},
	}

	for i, w := range weightesOps {
		operationMsg, _, _ := w.Op()(s.r, s.app.BaseApp, s.ctx, s.accounts, s.ctx.ChainID())
		// require.NoError(t, err) // TODO check if it should be NoError

		// the following checks are very much dependent from the ordering of the output given
		// by WeightedOperations. if the ordering in WeightedOperations changes some tests
		// will fail
		require.Equal(expected[i].weight, w.Weight(), "weight should be the same")
		require.Equal(expected[i].opMsgRoute, operationMsg.Route, "route should be the same")
		require.Equal(expected[i].opMsgName, operationMsg.Name, "operation Msg name should be the same")
	}
}

// TestSimulateMsgCreateValidator tests the normal scenario of a valid message of type TypeMsgCreateValidator.
// Abonormal scenarios, where the message are created by an errors are not tested here.
func (s *SimTestSuite) TestSimulateMsgCreateValidator() {
	require := s.Require()
	// begin a new block
	s.app.BeginBlock(abci.RequestBeginBlock{Header: tmproto.Header{Height: s.app.LastBlockHeight() + 1, AppHash: s.app.LastCommitID().Hash}})

	// execute operation
	op := simulation.SimulateMsgCreateValidator(s.accountKeeper, s.bankKeeper, s.stakingKeeper)
	operationMsg, futureOperations, err := op(s.r, s.app.BaseApp, s.ctx, s.accounts[1:], "")
	require.NoError(err)

	var msg types.MsgCreateValidator
	types.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)

	require.True(operationMsg.OK)
	require.Equal(types.TypeMsgCreateValidator, msg.Type())
	require.Equal("cosmos1p8wcgrjr4pjju90xg6u9cgq55dxwq8j7u4x9a0", msg.DelegatorAddress)
	require.Equal("cosmosvaloper1p8wcgrjr4pjju90xg6u9cgq55dxwq8j7epjs3u", msg.ValidatorAddress)
	require.Len(futureOperations, 0)
}

// TestSimulateMsgCancelUnbondingDelegation tests the normal scenario of a valid message of type TypeMsgCancelUnbondingDelegation.
// Abonormal scenarios, where the message is
func (s *SimTestSuite) TestSimulateMsgCancelUnbondingDelegation() {
	require := s.Require()
	blockTime := time.Now().UTC()
	ctx := s.ctx.WithBlockTime(blockTime)

	// setup accounts[1] as validator
	validator0 := s.getTestingValidator0(ctx)

	// setup delegation
	delTokens := s.stakingKeeper.TokensFromConsensusPower(ctx, 2)
	validator0, issuedShares := validator0.AddTokensFromDel(delTokens)
	delegator := s.accounts[2]
	delegation := types.NewDelegation(delegator.Address, validator0.GetOperator(), issuedShares)
	s.stakingKeeper.SetDelegation(ctx, delegation)
	s.distrKeeper.SetDelegatorStartingInfo(ctx, validator0.GetOperator(), delegator.Address, distrtypes.NewDelegatorStartingInfo(2, math.LegacyOneDec(), 200))

	s.setupValidatorRewards(ctx, validator0.GetOperator())

	// unbonding delegation
	udb := types.NewUnbondingDelegation(delegator.Address, validator0.GetOperator(), s.app.LastBlockHeight(), blockTime.Add(2*time.Minute), delTokens, 0)
	s.stakingKeeper.SetUnbondingDelegation(ctx, udb)
	s.setupValidatorRewards(ctx, validator0.GetOperator())

	// begin a new block
	s.app.BeginBlock(abci.RequestBeginBlock{Header: tmproto.Header{Height: s.app.LastBlockHeight() + 1, AppHash: s.app.LastCommitID().Hash, Time: blockTime}})

	// execute operation
	op := simulation.SimulateMsgCancelUnbondingDelegate(s.accountKeeper, s.bankKeeper, s.stakingKeeper)
	accounts := []simtypes.Account{delegator}
	operationMsg, futureOperations, err := op(s.r, s.app.BaseApp, ctx, accounts, "")
	require.NoError(err)

	var msg types.MsgCancelUnbondingDelegation
	types.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)

	require.True(operationMsg.OK)
	require.Equal(types.TypeMsgCancelUnbondingDelegation, msg.Type())
	require.Equal(delegator.Address.String(), msg.DelegatorAddress)
	require.Equal(validator0.GetOperator().String(), msg.ValidatorAddress)
	require.Len(futureOperations, 0)
}

// TestSimulateMsgEditValidator tests the normal scenario of a valid message of type TypeMsgEditValidator.
// Abonormal scenarios, where the message is created by an errors are not tested here.
func (s *SimTestSuite) TestSimulateMsgEditValidator() {
	require := s.Require()
	blockTime := time.Now().UTC()
	ctx := s.ctx.WithBlockTime(blockTime)

	// setup accounts[0] as validator
	_ = s.getTestingValidator0(ctx)

	// begin a new block
	s.app.BeginBlock(abci.RequestBeginBlock{Header: tmproto.Header{Height: s.app.LastBlockHeight() + 1, AppHash: s.app.LastCommitID().Hash, Time: blockTime}})

	// execute operation
	op := simulation.SimulateMsgEditValidator(s.accountKeeper, s.bankKeeper, s.stakingKeeper)
	operationMsg, futureOperations, err := op(s.r, s.app.BaseApp, ctx, s.accounts, "")
	require.NoError(err)

	var msg types.MsgEditValidator
	types.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)

	require.True(operationMsg.OK)
	require.Equal(types.TypeMsgEditValidator, msg.Type())
	require.Equal("cosmosvaloper1p8wcgrjr4pjju90xg6u9cgq55dxwq8j7epjs3u", msg.ValidatorAddress)
	require.Len(futureOperations, 0)
}

// TestSimulateMsgDelegate tests the normal scenario of a valid message of type TypeMsgDelegate.
// Abonormal scenarios, where the message is created by an errors are not tested here.
func (s *SimTestSuite) TestSimulateMsgDelegate() {
	require := s.Require()
	blockTime := time.Now().UTC()
	ctx := s.ctx.WithBlockTime(blockTime)

	// execute operation
	op := simulation.SimulateMsgDelegate(s.accountKeeper, s.bankKeeper, s.stakingKeeper)
	operationMsg, futureOperations, err := op(s.r, s.app.BaseApp, ctx, s.accounts[1:], "")
	require.NoError(err)

	var msg types.MsgDelegate
	types.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)

	require.True(operationMsg.OK)
	require.Equal("cosmos1p8wcgrjr4pjju90xg6u9cgq55dxwq8j7u4x9a0", msg.DelegatorAddress)
	require.Equal("stake", msg.Amount.Denom)
	require.Equal(types.TypeMsgDelegate, msg.Type())
	require.Equal("cosmosvaloper1tnh2q55v8wyygtt9srz5safamzdengsn9dsd7z", msg.ValidatorAddress)
	require.Len(futureOperations, 0)
}

// TestSimulateMsgUndelegate tests the normal scenario of a valid message of type TypeMsgUndelegate.
// Abonormal scenarios, where the message is created by an errors are not tested here.
func (s *SimTestSuite) TestSimulateMsgUndelegate() {
	require := s.Require()
	blockTime := time.Now().UTC()
	ctx := s.ctx.WithBlockTime(blockTime)

	// setup accounts[1] as validator
	validator0 := s.getTestingValidator0(ctx)

	// setup delegation
	delTokens := s.stakingKeeper.TokensFromConsensusPower(ctx, 2)
	validator0, issuedShares := validator0.AddTokensFromDel(delTokens)
	delegator := s.accounts[2]
	delegation := types.NewDelegation(delegator.Address, validator0.GetOperator(), issuedShares)
	s.stakingKeeper.SetDelegation(ctx, delegation)
	s.distrKeeper.SetDelegatorStartingInfo(ctx, validator0.GetOperator(), delegator.Address, distrtypes.NewDelegatorStartingInfo(2, math.LegacyOneDec(), 200))

	s.setupValidatorRewards(ctx, validator0.GetOperator())

	// begin a new block
	s.app.BeginBlock(abci.RequestBeginBlock{Header: tmproto.Header{Height: s.app.LastBlockHeight() + 1, AppHash: s.app.LastCommitID().Hash, Time: blockTime}})

	// execute operation
	op := simulation.SimulateMsgUndelegate(s.accountKeeper, s.bankKeeper, s.stakingKeeper)
	operationMsg, futureOperations, err := op(s.r, s.app.BaseApp, ctx, s.accounts, "")
	require.NoError(err)

	var msg types.MsgUndelegate
	types.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)

	require.True(operationMsg.OK)
	require.Equal("cosmos1ghekyjucln7y67ntx7cf27m9dpuxxemn4c8g4r", msg.DelegatorAddress)
	require.Equal("1646627814093010272", msg.Amount.Amount.String())
	require.Equal("stake", msg.Amount.Denom)
	require.Equal(types.TypeMsgUndelegate, msg.Type())
	require.Equal("cosmosvaloper1p8wcgrjr4pjju90xg6u9cgq55dxwq8j7epjs3u", msg.ValidatorAddress)
	require.Len(futureOperations, 0)
}

// TestSimulateMsgBeginRedelegate tests the normal scenario of a valid message of type TypeMsgBeginRedelegate.
// Abonormal scenarios, where the message is created by an errors, are not tested here.
func (s *SimTestSuite) TestSimulateMsgBeginRedelegate() {
	require := s.Require()
	blockTime := time.Now().UTC()
	ctx := s.ctx.WithBlockTime(blockTime)

	// setup accounts[1] as validator0 and accounts[2] as validator1
	validator0 := s.getTestingValidator0(ctx)
	validator1 := s.getTestingValidator1(ctx)

	delTokens := s.stakingKeeper.TokensFromConsensusPower(ctx, 2)
	validator1, issuedShares := validator1.AddTokensFromDel(delTokens)

	// setup accounts[3] as delegator
	delegator := s.accounts[3]
	delegation := types.NewDelegation(delegator.Address, validator0.GetOperator(), issuedShares)
	s.stakingKeeper.SetDelegation(ctx, delegation)
	s.distrKeeper.SetDelegatorStartingInfo(ctx, validator0.GetOperator(), delegator.Address, distrtypes.NewDelegatorStartingInfo(2, math.LegacyOneDec(), 200))

	s.setupValidatorRewards(ctx, validator0.GetOperator())
	s.setupValidatorRewards(ctx, validator1.GetOperator())

	// begin a new block
	s.app.BeginBlock(abci.RequestBeginBlock{Header: tmproto.Header{Height: s.app.LastBlockHeight() + 1, AppHash: s.app.LastCommitID().Hash, Time: blockTime}})

	// execute operation
	op := simulation.SimulateMsgBeginRedelegate(s.accountKeeper, s.bankKeeper, s.stakingKeeper)
	operationMsg, futureOperations, err := op(s.r, s.app.BaseApp, ctx, s.accounts, "")
	s.T().Logf("operation message: %v", operationMsg)
	require.NoError(err)

	var msg types.MsgBeginRedelegate
	types.ModuleCdc.UnmarshalJSON(operationMsg.Msg, &msg)

	require.True(operationMsg.OK)
	require.Equal("cosmos1ua0fwyws7vzjrry3pqkklvf8mny93l9s9zg0h4", msg.DelegatorAddress)
	require.Equal("stake", msg.Amount.Denom)
	require.Equal(types.TypeMsgBeginRedelegate, msg.Type())
	require.Equal("cosmosvaloper1ghekyjucln7y67ntx7cf27m9dpuxxemnsvnaes", msg.ValidatorDstAddress)
	require.Equal("cosmosvaloper1p8wcgrjr4pjju90xg6u9cgq55dxwq8j7epjs3u", msg.ValidatorSrcAddress)
	require.Len(futureOperations, 0)
}

func (s *SimTestSuite) getTestingValidator0(ctx sdk.Context) types.Validator {
	commission0 := types.NewCommission(math.LegacyZeroDec(), math.LegacyOneDec(), math.LegacyOneDec())
	return s.getTestingValidator(ctx, commission0, 1)
}

func (s *SimTestSuite) getTestingValidator1(ctx sdk.Context) types.Validator {
	commission1 := types.NewCommission(math.LegacyZeroDec(), math.LegacyZeroDec(), math.LegacyZeroDec())
	return s.getTestingValidator(ctx, commission1, 2)
}

func (s *SimTestSuite) getTestingValidator(ctx sdk.Context, commission types.Commission, n int) types.Validator {
	account := s.accounts[n]
	valPubKey := account.PubKey
	valAddr := sdk.ValAddress(account.PubKey.Address().Bytes())
	validator := testutil.NewValidator(s.T(), valAddr, valPubKey)
	validator, err := validator.SetInitialCommission(commission)
	s.Require().NoError(err)

	validator.DelegatorShares = math.LegacyNewDec(100)
	validator.Tokens = s.stakingKeeper.TokensFromConsensusPower(ctx, 100)

	s.stakingKeeper.SetValidator(ctx, validator)

	return validator
}

func (s *SimTestSuite) setupValidatorRewards(ctx sdk.Context, valAddress sdk.ValAddress) {
	decCoins := sdk.DecCoins{sdk.NewDecCoinFromDec(sdk.DefaultBondDenom, math.LegacyOneDec())}
	historicalRewards := distrtypes.NewValidatorHistoricalRewards(decCoins, 2)
	s.distrKeeper.SetValidatorHistoricalRewards(ctx, valAddress, 2, historicalRewards)
	// setup current revards
	currentRewards := distrtypes.NewValidatorCurrentRewards(decCoins, 3)
	s.distrKeeper.SetValidatorCurrentRewards(ctx, valAddress, currentRewards)
}

func TestSimTestSuite(t *testing.T) {
	suite.Run(t, new(SimTestSuite))
}

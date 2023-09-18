package keeper_test

import (
	"math/big"
	"math/rand"
	"testing"
	"time"

	"cosmossdk.io/math"
	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"

	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	tmtypes "github.com/cometbft/cometbft/types"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/runtime"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/staking/testutil"

	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	banktestutil "github.com/cosmos/cosmos-sdk/x/bank/testutil"

	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	minttypes "github.com/cosmos/cosmos-sdk/x/mint/types"
	"github.com/cosmos/cosmos-sdk/x/staking/types"

	authkeeper "github.com/cosmos/cosmos-sdk/x/auth/keeper"
	bankkeeper "github.com/cosmos/cosmos-sdk/x/bank/keeper"
	distrkeeper "github.com/cosmos/cosmos-sdk/x/distribution/keeper"
	mintkeeper "github.com/cosmos/cosmos-sdk/x/mint/keeper"
	stakingkeeper "github.com/cosmos/cosmos-sdk/x/staking/keeper"
)

// A test suite not unlike `SimTestSuite` built for the needs of testing epoch staking
type EpochTestSuite struct {
	suite.Suite

	r             *rand.Rand
	accounts      []simtypes.Account
	ctx           sdk.Context
	app           *runtime.App
	bankKeeper    bankkeeper.Keeper
	accountKeeper authkeeper.AccountKeeper
	distrKeeper   distrkeeper.Keeper
	stakingKeeper *stakingkeeper.Keeper
	encCfg        moduletestutil.TestEncodingConfig
}

func (s *EpochTestSuite) SetupTest() {
	sdk.DefaultPowerReduction = sdk.NewIntFromBigInt(new(big.Int).Exp(big.NewInt(10), big.NewInt(18), nil))

	s.r = rand.New(rand.NewSource(1))
	accounts := simtypes.RandomAccounts(s.r, 16)

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

	stakingParams := types.DefaultParams()
	stakingParams.EpochLength = 6
	stakingParams.MaxValidators = 8
	stakingParams.JailedValidatorThreshold = 20
	stakingKeeper.SetParams(ctx, stakingParams)

	initAmt := stakingKeeper.TokensFromConsensusPower(ctx, 10)
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
	s.ctx = ctx.WithBlockHeight(1)
	s.app = app
}

func (s *EpochTestSuite) TestValidatorSetRotationOnEpoch() {
	require := s.Require()
	blockTime := time.Now().UTC()

	s.ctx = s.ctx.WithBlockTime(blockTime)

	var validators []types.Validator
	for n := 1; n < 8; n++ {
		validators = append(validators, s.getTestingValidator(s.ctx, n, 1))
	}

	active_set := s.stakingKeeper.GetBondedValidatorsByPower(s.ctx)
	require.Equal(1, len(active_set))

	s.ctx = s.processStakingBlocks(3)

	active_set = s.stakingKeeper.GetBondedValidatorsByPower(s.ctx)
	require.Equal(1, len(active_set))

	s.ctx = s.processStakingBlocks(3)

	active_set = s.stakingKeeper.GetBondedValidatorsByPower(s.ctx)

	require.Equal(8, len(active_set))
}

func (s *EpochTestSuite) TestForceValidatorSetRotationOnAfterJailThreshold() {
	require := s.Require()
	blockTime := time.Now().UTC()

	s.ctx = s.ctx.WithBlockTime(blockTime)

	var validators []types.Validator
	for n := 1; n < 8; n++ {
		validators = append(validators, s.getTestingValidator(s.ctx, n, 1))
	}

	active_set := s.stakingKeeper.GetBondedValidatorsByPower(s.ctx)
	require.Equal(1, len(active_set))

	s.ctx = s.processStakingBlocks(6)

	active_set = s.stakingKeeper.GetBondedValidatorsByPower(s.ctx)
	require.Equal(8, len(active_set))

	for n := 8; n < 16; n++ {
		validators = append(validators, s.getTestingValidator(s.ctx, n, 1))
	}
	s.ctx = s.processStakingBlocks(1)

	// Jail 4 validators
	for n := 0; n < 4; n++ {
		consAddr, _ := validators[n].GetConsAddr()
		s.stakingKeeper.Jail(s.ctx, consAddr)
	}
	// Ensure validators are jailed before end of block
	active_set = s.stakingKeeper.GetBondedValidatorsByPower(s.ctx)
	require.Equal(4, len(active_set))

	// Ensure forced rotation of validators after end of block
	s.ctx = s.processStakingBlocks(1)
	active_set = s.stakingKeeper.GetBondedValidatorsByPower(s.ctx)
	require.Equal(8, len(active_set))
}

func (s *EpochTestSuite) TestNoSetRotationWhileUnderThreshold() {
	require := s.Require()
	blockTime := time.Now().UTC()

	s.ctx = s.ctx.WithBlockTime(blockTime)

	var validators []types.Validator
	for n := 1; n < 8; n++ {
		validators = append(validators, s.getTestingValidator(s.ctx, n, 1))
	}

	s.ctx = s.processStakingBlocks(6)

	active_set := s.stakingKeeper.GetBondedValidatorsByPower(s.ctx)
	require.Equal(8, len(active_set))

	for n := 8; n < 16; n++ {
		validators = append(validators, s.getTestingValidator(s.ctx, n, 1))
	}
	s.ctx = s.processStakingBlocks(1)

	consAddr, _ := validators[0].GetConsAddr()
	s.stakingKeeper.Jail(s.ctx, consAddr)

	// Ensure validators are jailed before end of block
	active_set = s.stakingKeeper.GetBondedValidatorsByPower(s.ctx)
	require.Equal(7, len(active_set))

	// Ensure no forced rotation of validators after end of block
	s.ctx = s.processStakingBlocks(1)
	active_set = s.stakingKeeper.GetBondedValidatorsByPower(s.ctx)
	require.Equal(7, len(active_set))
}

func (s *EpochTestSuite) TestChangeJailedThreshold() {
	require := s.Require()
	blockTime := time.Now().UTC()

	s.ctx = s.ctx.WithBlockTime(blockTime)

	// Update `JailedValidatorThreshold` to 75%
	stakingParams := s.stakingKeeper.GetParams(s.ctx)
	stakingParams.JailedValidatorThreshold = 75
	s.stakingKeeper.SetParams(s.ctx, stakingParams)

	// On-board full validator set
	var validators []types.Validator
	for n := 1; n < 8; n++ {
		validators = append(validators, s.getTestingValidator(s.ctx, n, 1))
	}
	s.ctx = s.processStakingBlocks(6)

	active_set := s.stakingKeeper.GetBondedValidatorsByPower(s.ctx)
	require.Equal(8, len(active_set))

	// Create additonal validators, excluded from the active set
	for n := 8; n < 16; n++ {
		validators = append(validators, s.getTestingValidator(s.ctx, n, 1))
	}
	s.ctx = s.processStakingBlocks(1)

	// Jail 5 validators (62.5% of the active set)
	for n := 0; n < 5; n++ {
		consAddr, _ := validators[n].GetConsAddr()
		s.stakingKeeper.Jail(s.ctx, consAddr)
	}
	// Ensure validators are jailed before end of block
	active_set = s.stakingKeeper.GetBondedValidatorsByPower(s.ctx)
	require.Equal(3, len(active_set))

	// Ensure validators are still jailed after end of block
	s.ctx = s.processStakingBlocks(1)
	active_set = s.stakingKeeper.GetBondedValidatorsByPower(s.ctx)
	require.Equal(3, len(active_set))

	// Jail 6th validator
	consAddr, _ := validators[5].GetConsAddr()
	s.stakingKeeper.Jail(s.ctx, consAddr)

	// Ensure forced rotation of validators after end of block
	s.ctx = s.processStakingBlocks(1)
	active_set = s.stakingKeeper.GetBondedValidatorsByPower(s.ctx)
	require.Equal(8, len(active_set))
}

func (s *EpochTestSuite) TestChangeMaxValidators() {
	require := s.Require()
	blockTime := time.Now().UTC()

	s.ctx = s.ctx.WithBlockTime(blockTime)

	// Update `JailedValidatorThreshold` to 75%
	stakingParams := s.stakingKeeper.GetParams(s.ctx)
	stakingParams.MaxValidators = 12
	s.stakingKeeper.SetParams(s.ctx, stakingParams)

	// On-board full validator set
	var validators []types.Validator
	for n := 1; n < 8; n++ {
		validators = append(validators, s.getTestingValidator(s.ctx, n, 1))
	}
	s.ctx = s.processStakingBlocks(6)

	active_set := s.stakingKeeper.GetBondedValidatorsByPower(s.ctx)
	require.Equal(8, len(active_set))

	// Create additonal validators, excluded from the active set
	for n := 8; n < 16; n++ {
		validators = append(validators, s.getTestingValidator(s.ctx, n, 1))
	}
	s.ctx = s.processStakingBlocks(1)

	// Jail 3 validators (25% of the active set)
	for n := 0; n < 3; n++ {
		consAddr, _ := validators[n].GetConsAddr()
		s.stakingKeeper.Jail(s.ctx, consAddr)
	}
	// Ensure validators are jailed before end of block
	active_set = s.stakingKeeper.GetBondedValidatorsByPower(s.ctx)
	require.Equal(5, len(active_set))

	// Ensure validator set rotation after end of block
	s.ctx = s.processStakingBlocks(1)
	active_set = s.stakingKeeper.GetBondedValidatorsByPower(s.ctx)
	require.Equal(12, len(active_set))
}

// Process the given number of blocks, calling `BlockValidatorUpdates` and
// incrementing the height. Returns the updated context.
func (s *EpochTestSuite) processStakingBlocks(blocks uint64) sdk.Context {
	ctx := s.ctx

	for n := uint64(0); n < blocks; n++ {
		height := ctx.BlockHeight() + 1
		ctx = ctx.WithBlockHeight(height)
		s.stakingKeeper.BlockValidatorUpdates(ctx)
	}

	return ctx
}

// Gets validator `n` from `s.accounts` and gives them a commission with a
// starting rate of zero and a max rate and max rate change of one.
func (s *EpochTestSuite) getTestingValidator(ctx sdk.Context, n int, delegation int64) types.Validator {
	account := s.accounts[n]
	valPubKey := account.PubKey
	valAddr := sdk.ValAddress(account.PubKey.Address().Bytes())
	validator := testutil.NewValidator(s.T(), valAddr, valPubKey)

	convertedDelegation := s.stakingKeeper.TokensFromConsensusPower(ctx, delegation)
	validator.DelegatorShares = math.LegacyOneDec()
	validator.Tokens = math.OneInt() // validators must not have zero tokens before delegation

	s.stakingKeeper.SetValidator(ctx, validator)
	s.stakingKeeper.SetValidatorByConsAddr(ctx, validator)
	s.stakingKeeper.SetNewValidatorByPowerIndex(ctx, validator)

	err := s.stakingKeeper.Hooks().AfterValidatorCreated(ctx, validator.GetOperator())
	s.Require().NoError(err)

	_, err = s.stakingKeeper.Delegate(ctx, account.Address, convertedDelegation, types.Unbonded, validator, true)
	validator, _ = s.stakingKeeper.GetValidator(ctx, valAddr)
	validator.Tokens = validator.Tokens.Sub(math.OneInt())
	s.stakingKeeper.SetValidator(ctx, validator)
	s.Require().NoError(err)

	return validator
}

func TestEpochSimTestSuite(t *testing.T) {
	suite.Run(t, new(EpochTestSuite))
}

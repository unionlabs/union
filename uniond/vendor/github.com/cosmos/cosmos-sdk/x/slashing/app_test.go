package slashing_test

import (
	"errors"
	"testing"

	abci "github.com/cometbft/cometbft/abci/types"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/math"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/testutil/configurator"
	"github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	bankkeeper "github.com/cosmos/cosmos-sdk/x/bank/keeper"
	"github.com/cosmos/cosmos-sdk/x/slashing/keeper"
	"github.com/cosmos/cosmos-sdk/x/slashing/types"
	stakingkeeper "github.com/cosmos/cosmos-sdk/x/staking/keeper"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
)

var (
	priv1 = secp256k1.GenPrivKey()
	addr1 = sdk.AccAddress(priv1.PubKey().Address())

	valKey  = ed25519.GenPrivKey()
	valAddr = sdk.AccAddress(valKey.PubKey().Address())
)

func TestSlashingMsgs(t *testing.T) {
	genTokens := sdk.TokensFromConsensusPower(42, sdk.DefaultPowerReduction)
	bondTokens := sdk.TokensFromConsensusPower(10, sdk.DefaultPowerReduction)
	genCoin := sdk.NewCoin(sdk.DefaultBondDenom, genTokens)
	bondCoin := sdk.NewCoin(sdk.DefaultBondDenom, bondTokens)

	acc1 := &authtypes.BaseAccount{
		Address: addr1.String(),
	}
	accs := []sims.GenesisAccount{{GenesisAccount: acc1, Coins: sdk.Coins{genCoin}}}

	startupCfg := sims.DefaultStartUpConfig()
	startupCfg.GenesisAccounts = accs

	var (
		stakingKeeper  *stakingkeeper.Keeper
		bankKeeper     bankkeeper.Keeper
		slashingKeeper keeper.Keeper
	)

	app, err := sims.SetupWithConfiguration(configurator.NewAppConfig(
		configurator.ParamsModule(),
		configurator.AuthModule(),
		configurator.StakingModule(),
		configurator.SlashingModule(),
		configurator.TxModule(),
		configurator.ConsensusModule(),
		configurator.BankModule()),
		startupCfg, &stakingKeeper, &bankKeeper, &slashingKeeper)

	baseApp := app.BaseApp

	ctxCheck := baseApp.NewContext(true, tmproto.Header{})
	require.True(t, sdk.Coins{genCoin}.IsEqual(bankKeeper.GetAllBalances(ctxCheck, addr1)))

	require.NoError(t, err)

	description := stakingtypes.NewDescription("foo_moniker", "", "", "", "")
	commission := stakingtypes.NewCommissionRates(math.LegacyZeroDec(), math.LegacyZeroDec(), math.LegacyZeroDec())

	createValidatorMsg, err := stakingtypes.NewMsgCreateValidator(
		sdk.ValAddress(addr1), valKey.PubKey(), bondCoin, description, commission, math.OneInt(),
	)
	require.NoError(t, err)

	header := tmproto.Header{Height: app.LastBlockHeight() + 1}
	txConfig := moduletestutil.MakeTestEncodingConfig().TxConfig
	_, _, err = sims.SignCheckDeliver(t, txConfig, app.BaseApp, header, []sdk.Msg{createValidatorMsg}, "", []uint64{0}, []uint64{0}, true, true, priv1)
	require.NoError(t, err)
	require.True(t, sdk.Coins{genCoin.Sub(bondCoin)}.IsEqual(bankKeeper.GetAllBalances(ctxCheck, addr1)))

	header = tmproto.Header{Height: app.LastBlockHeight() + 1}
	app.BeginBlock(abci.RequestBeginBlock{Header: header})

	ctxCheck = baseApp.NewContext(true, tmproto.Header{})
	validator, found := stakingKeeper.GetValidator(ctxCheck, sdk.ValAddress(addr1))
	require.True(t, found)
	require.Equal(t, sdk.ValAddress(addr1).String(), validator.OperatorAddress)
	require.Equal(t, stakingtypes.Bonded, validator.Status)
	require.True(math.IntEq(t, bondTokens, validator.BondedTokens()))
	unjailMsg := &types.MsgUnjail{ValidatorAddr: sdk.ValAddress(addr1).String()}

	ctxCheck = app.BaseApp.NewContext(true, tmproto.Header{})
	_, found = slashingKeeper.GetValidatorSigningInfo(ctxCheck, sdk.ConsAddress(valAddr))
	require.True(t, found)

	// unjail should fail with unknown validator
	header = tmproto.Header{Height: app.LastBlockHeight() + 1}
	_, res, err := sims.SignCheckDeliver(t, txConfig, app.BaseApp, header, []sdk.Msg{unjailMsg}, "", []uint64{0}, []uint64{1}, false, false, priv1)
	require.Error(t, err)
	require.Nil(t, res)
	require.True(t, errors.Is(types.ErrValidatorNotJailed, err))
}

package genutil_test

import (
	"encoding/json"
	"errors"
	"fmt"
	"math/rand"
	"testing"
	"time"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/suite"

	_ "cosmossdk.io/api/cosmos/crypto/secp256k1"
	"cosmossdk.io/math"
	storetypes "cosmossdk.io/store/types"
	banktypes "cosmossdk.io/x/bank/types"
	stakingtypes "cosmossdk.io/x/staking/types"

	addresscodec "github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/testutil"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/x/genutil"
	genutiltestutil "github.com/cosmos/cosmos-sdk/x/genutil/testutil"
	"github.com/cosmos/cosmos-sdk/x/genutil/types"
)

var (
	priv1 = secp256k1.GenPrivKey()
	priv2 = secp256k1.GenPrivKey()
	pk1   = priv1.PubKey()
	pk2   = priv2.PubKey()
	addr1 = sdk.AccAddress(pk1.Address())
	addr2 = sdk.AccAddress(pk2.Address())
	desc  = stakingtypes.NewDescription("testname", "", "", "", "")
	comm  = stakingtypes.CommissionRates{}
)

// GenTxTestSuite is a test suite to be used with gentx tests.
type GenTxTestSuite struct {
	suite.Suite

	ctx sdk.Context

	stakingKeeper  *genutiltestutil.MockStakingKeeper
	encodingConfig moduletestutil.TestEncodingConfig
	msg1, msg2     *stakingtypes.MsgCreateValidator
}

func (suite *GenTxTestSuite) SetupTest() {
	valAc := codectestutil.CodecOptions{}.GetValidatorCodec()
	suite.encodingConfig = moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, genutil.AppModule{})
	key := storetypes.NewKVStoreKey("a_Store_Key")
	tkey := storetypes.NewTransientStoreKey("a_transient_store")
	suite.ctx = testutil.DefaultContext(key, tkey)

	ctrl := gomock.NewController(suite.T())
	suite.stakingKeeper = genutiltestutil.NewMockStakingKeeper(ctrl)

	stakingtypes.RegisterInterfaces(suite.encodingConfig.InterfaceRegistry)
	banktypes.RegisterInterfaces(suite.encodingConfig.InterfaceRegistry)

	var err error
	amount := sdk.NewInt64Coin(sdk.DefaultBondDenom, 50)
	one := math.OneInt()
	pk1Addr, err := valAc.BytesToString(pk1.Address())
	suite.NoError(err)
	suite.msg1, err = stakingtypes.NewMsgCreateValidator(pk1Addr, pk1, amount, desc, comm, one)
	suite.NoError(err)
	pk2Addr, err := valAc.BytesToString(pk2.Address())
	suite.NoError(err)
	suite.msg2, err = stakingtypes.NewMsgCreateValidator(pk2Addr, pk1, amount, desc, comm, one)
	suite.NoError(err)
}

func (suite *GenTxTestSuite) setAccountBalance(balances []banktypes.Balance) json.RawMessage {
	bankGenesisState := banktypes.GenesisState{
		Params: banktypes.Params{DefaultSendEnabled: true},
		Balances: []banktypes.Balance{
			{
				Address: "cosmos1fl48vsnmsdzcv85q5d2q4z5ajdha8yu34mf0eh",
				Coins:   sdk.Coins{sdk.NewInt64Coin(sdk.DefaultBondDenom, 1000000)},
			},
			{
				Address: "cosmos1jv65s3grqf6v6jl3dp4t6c9t9rk99cd88lyufl",
				Coins:   sdk.Coins{sdk.NewInt64Coin(sdk.DefaultBondDenom, 2059726)},
			},
			{
				Address: "cosmos1k5lndq46x9xpejdxq52q3ql3ycrphg4qxlfqn7",
				Coins:   sdk.Coins{sdk.NewInt64Coin(sdk.DefaultBondDenom, 100000000000000)},
			},
		},
		Supply: sdk.Coins{sdk.NewInt64Coin(sdk.DefaultBondDenom, 0)},
	}
	bankGenesisState.Balances = append(bankGenesisState.Balances, balances...)
	for _, balance := range bankGenesisState.Balances {
		bankGenesisState.Supply.Add(balance.Coins...)
	}
	bankGenesis, err := suite.encodingConfig.Amino.MarshalJSON(bankGenesisState) // TODO switch this to use Marshaler
	suite.Require().NoError(err)

	return bankGenesis
}

func (suite *GenTxTestSuite) TestSetGenTxsInAppGenesisState() {
	var (
		txBuilder = suite.encodingConfig.TxConfig.NewTxBuilder()
		genTxs    []sdk.Tx
	)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"one genesis transaction",
			func() {
				err := txBuilder.SetMsgs(suite.msg1)
				suite.Require().NoError(err)
				tx := txBuilder.GetTx()
				genTxs = []sdk.Tx{tx}
			},
			true,
		},
		{
			"two genesis transactions",
			func() {
				err := txBuilder.SetMsgs(suite.msg1, suite.msg2)
				suite.Require().NoError(err)
				tx := txBuilder.GetTx()
				genTxs = []sdk.Tx{tx}
			},
			true,
		},
	}

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			suite.SetupTest()
			cdc := suite.encodingConfig.Codec
			txJSONEncoder := suite.encodingConfig.TxConfig.TxJSONEncoder()

			tc.malleate()
			appGenesisState, err := genutil.SetGenTxsInAppGenesisState(cdc, txJSONEncoder, make(map[string]json.RawMessage), genTxs)

			suite.Require().NoError(err)
			suite.Require().NotNil(appGenesisState[types.ModuleName])

			var genesisState types.GenesisState
			err = cdc.UnmarshalJSON(appGenesisState[types.ModuleName], &genesisState)
			suite.Require().NoError(err)
			suite.Require().NotNil(genesisState.GenTxs)
		})
	}
}

func (suite *GenTxTestSuite) TestValidateAccountInGenesis() {
	var (
		appGenesisState = make(map[string]json.RawMessage)
		coins           sdk.Coins
		ac              = codectestutil.CodecOptions{}.GetAddressCodec()
	)

	addr1Str, err := ac.BytesToString(addr1)
	suite.Require().NoError(err)
	addr2Str, err := ac.BytesToString(addr2)
	suite.Require().NoError(err)

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"no accounts",
			func() {
				coins = sdk.Coins{sdk.NewInt64Coin(sdk.DefaultBondDenom, 0)}
			},
			false,
		},
		{
			"account without balance in the genesis state",
			func() {
				coins = sdk.Coins{sdk.NewInt64Coin(sdk.DefaultBondDenom, 0)}
				balances := banktypes.Balance{
					Address: addr2Str,
					Coins:   sdk.Coins{sdk.NewInt64Coin(sdk.DefaultBondDenom, 50)},
				}
				appGenesisState[banktypes.ModuleName] = suite.setAccountBalance([]banktypes.Balance{balances})
			},
			false,
		},
		{
			"account without enough funds of default bond denom",
			func() {
				coins = sdk.Coins{sdk.NewInt64Coin(sdk.DefaultBondDenom, 50)}
				balances := banktypes.Balance{
					Address: addr1Str,
					Coins:   sdk.Coins{sdk.NewInt64Coin(sdk.DefaultBondDenom, 25)},
				}
				appGenesisState[banktypes.ModuleName] = suite.setAccountBalance([]banktypes.Balance{balances})
			},
			false,
		},
		{
			"account with enough funds of default bond denom",
			func() {
				coins = sdk.Coins{sdk.NewInt64Coin(sdk.DefaultBondDenom, 10)}
				balances := banktypes.Balance{
					Address: addr1Str,
					Coins:   sdk.Coins{sdk.NewInt64Coin(sdk.DefaultBondDenom, 25)},
				}
				appGenesisState[banktypes.ModuleName] = suite.setAccountBalance([]banktypes.Balance{balances})
			},
			true,
		},
	}
	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			suite.SetupTest()
			cdc := suite.encodingConfig.Codec

			stakingGenesis, err := cdc.MarshalJSON(&stakingtypes.GenesisState{Params: stakingtypes.DefaultParams()}) // TODO switch this to use Marshaler
			suite.Require().NoError(err)
			appGenesisState[stakingtypes.ModuleName] = stakingGenesis

			addr, err := addresscodec.NewBech32Codec("cosmos").BytesToString(addr1)
			suite.Require().NoError(err)

			tc.malleate()
			err = genutil.ValidateAccountInGenesis(
				appGenesisState, banktypes.GenesisBalancesIterator{},
				addr, coins, cdc,
			)

			if tc.expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *GenTxTestSuite) TestDeliverGenTxs() {
	var (
		genTxs    []json.RawMessage
		txBuilder = suite.encodingConfig.TxConfig.NewTxBuilder()
	)

	ac := addresscodec.NewBech32Codec("cosmos")

	addr1Str, err := ac.BytesToString(addr1)
	suite.Require().NoError(err)
	addr2Str, err := ac.BytesToString(addr2)
	suite.Require().NoError(err)

	testCases := []struct {
		msg         string
		malleate    func()
		deliverTxFn genutil.TxHandler
		expPass     bool
	}{
		{
			"no signature supplied",
			func() {
				err := txBuilder.SetMsgs(suite.msg1)
				suite.Require().NoError(err)

				genTxs = make([]json.RawMessage, 1)
				tx, err := suite.encodingConfig.TxConfig.TxJSONEncoder()(txBuilder.GetTx())
				suite.Require().NoError(err)
				genTxs[0] = tx
			},
			GenesisState1{},
			false,
		},
		{
			"success",
			func() {
				r := rand.New(rand.NewSource(time.Now().UnixNano()))
				msg := banktypes.NewMsgSend(addr1Str, addr2Str, sdk.Coins{sdk.NewInt64Coin(sdk.DefaultBondDenom, 1)})
				tx, err := simtestutil.GenSignedMockTx(
					r,
					suite.encodingConfig.TxConfig,
					[]sdk.Msg{msg},
					sdk.Coins{sdk.NewInt64Coin(sdk.DefaultBondDenom, 10)},
					simtestutil.DefaultGenTxGas,
					suite.ctx.ChainID(),
					[]uint64{7},
					[]uint64{0},
					priv1,
				)
				suite.Require().NoError(err)

				genTxs = make([]json.RawMessage, 1)
				genTx, err := suite.encodingConfig.TxConfig.TxJSONEncoder()(tx)
				suite.Require().NoError(err)
				genTxs[0] = genTx
			},
			GenesisState2{},
			true,
		},
	}

	for _, tc := range testCases {
		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			suite.SetupTest()

			tc.malleate()

			if tc.expPass {
				suite.stakingKeeper.EXPECT().ApplyAndReturnValidatorSetUpdates(gomock.Any()).Return(nil, nil).AnyTimes()
				suite.Require().NotPanics(func() {
					_, err := genutil.DeliverGenTxs(
						suite.ctx, genTxs, suite.stakingKeeper, tc.deliverTxFn,
						suite.encodingConfig.TxConfig,
					)
					suite.Require().NoError(err)
				})
			} else {
				_, err := genutil.DeliverGenTxs(
					suite.ctx, genTxs, suite.stakingKeeper, tc.deliverTxFn,
					suite.encodingConfig.TxConfig,
				)

				suite.Require().Error(err)
			}
		})
	}
}

func TestGenTxTestSuite(t *testing.T) {
	suite.Run(t, new(GenTxTestSuite))
}

type GenesisState1 struct{}

func (GenesisState1) ExecuteGenesisTx(_ []byte) error {
	return errors.New("no signatures supplied")
}

type GenesisState2 struct{}

func (GenesisState2) ExecuteGenesisTx(tx []byte) error {
	return nil
}

package app

import (
	"encoding/json"
	"fmt"
	"math/rand"
	"os"
	"path/filepath"
	"testing"
	"time"

	abci "github.com/cometbft/cometbft/abci/types"
	cmtjson "github.com/cometbft/cometbft/libs/json"
	cmttypes "github.com/cometbft/cometbft/types"
	dbm "github.com/cosmos/cosmos-db"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/log"
	sdkmath "cosmossdk.io/math"
	pruningtypes "cosmossdk.io/store/pruning/types"
	"cosmossdk.io/store/snapshots"
	snapshottypes "cosmossdk.io/store/snapshots/types"

	bam "github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/server"
	servertypes "github.com/cosmos/cosmos-sdk/server/types"
	"github.com/cosmos/cosmos-sdk/testutil/mock"
	"github.com/cosmos/cosmos-sdk/testutil/network"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	banktypes "cosmossdk.io/x/bank/types"
	minttypes "cosmossdk.io/x/mint/types"
	slashingtypes "cosmossdk.io/x/slashing/types"
	stakingtypes "cosmossdk.io/x/staking/types"

	wasmkeeper "github.com/CosmWasm/wasmd/x/wasm/keeper"
)

// SetupOptions defines arguments that are passed into `WasmApp` constructor.
type SetupOptions struct {
	Logger   log.Logger
	DB       *dbm.MemDB
	AppOpts  servertypes.AppOptions
	WasmOpts []wasmkeeper.Option
}

func setup(t testing.TB, chainID string, withGenesis bool, invCheckPeriod uint, opts ...wasmkeeper.Option) (*WasmApp, GenesisState) {
	db := dbm.NewMemDB()
	nodeHome := t.TempDir()
	snapshotDir := filepath.Join(nodeHome, "data", "snapshots")

	snapshotDB, err := dbm.NewDB("metadata", dbm.GoLevelDBBackend, snapshotDir)
	require.NoError(t, err)
	t.Cleanup(func() { snapshotDB.Close() })
	snapshotStore, err := snapshots.NewStore(snapshotDB, snapshotDir)
	require.NoError(t, err)

	appOptions := make(simtestutil.AppOptionsMap, 0)
	appOptions[flags.FlagHome] = nodeHome // ensure unique folder
	appOptions[server.FlagInvCheckPeriod] = invCheckPeriod
	app := NewWasmApp(log.NewNopLogger(), db, nil, true, appOptions, opts, bam.SetChainID(chainID), bam.SetSnapshot(snapshotStore, snapshottypes.SnapshotOptions{KeepRecent: 2}))
	if withGenesis {
		return app, app.DefaultGenesis()
	}
	return app, GenesisState{}
}

// NewWasmAppWithCustomOptions initializes a new WasmApp with custom options.
func NewWasmAppWithCustomOptions(t *testing.T, isCheckTx bool, options SetupOptions) *WasmApp {
	t.Helper()

	privVal := mock.NewPV()
	pubKey, err := privVal.GetPubKey()
	require.NoError(t, err)
	// create validator set with single validator
	validator := cmttypes.NewValidator(pubKey, 1)
	valSet := cmttypes.NewValidatorSet([]*cmttypes.Validator{validator})

	// generate genesis account
	senderPrivKey := secp256k1.GenPrivKey()
	acc := authtypes.NewBaseAccount(senderPrivKey.PubKey().Address().Bytes(), senderPrivKey.PubKey(), 0, 0)
	balance := banktypes.Balance{
		Address: acc.GetAddress().String(),
		Coins:   sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100000000000000))),
	}

	app := NewWasmApp(options.Logger, options.DB, nil, true, options.AppOpts, options.WasmOpts)
	genesisState := app.DefaultGenesis()
	genesisState, err = GenesisStateWithValSet(app.AppCodec(), genesisState, valSet, []authtypes.GenesisAccount{acc}, balance)
	require.NoError(t, err)

	if !isCheckTx {
		// init chain must be called to stop deliverState from being nil
		stateBytes, err := cmtjson.MarshalIndent(genesisState, "", " ")
		require.NoError(t, err)

		// Initialize the chain
		_, err = app.InitChain(
			&abci.RequestInitChain{
				Validators:      []abci.ValidatorUpdate{},
				ConsensusParams: simtestutil.DefaultConsensusParams,
				AppStateBytes:   stateBytes,
			})
		require.NoError(t, err)
	}

	return app
}

// Setup initializes a new WasmApp. A Nop logger is set in WasmApp.
func Setup(t *testing.T, opts ...wasmkeeper.Option) *WasmApp {
	t.Helper()

	privVal := mock.NewPV()
	pubKey, err := privVal.GetPubKey()
	require.NoError(t, err)

	// create validator set with single validator
	validator := cmttypes.NewValidator(pubKey, 1)
	valSet := cmttypes.NewValidatorSet([]*cmttypes.Validator{validator})

	// generate genesis account
	senderPrivKey := secp256k1.GenPrivKey()
	acc := authtypes.NewBaseAccount(senderPrivKey.PubKey().Address().Bytes(), senderPrivKey.PubKey(), 0, 0)
	balance := banktypes.Balance{
		Address: acc.GetAddress().String(),
		Coins:   sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100000000000000))),
	}
	chainID := "testing"
	app := SetupWithGenesisValSet(t, valSet, []authtypes.GenesisAccount{acc}, chainID, opts, balance)

	return app
}

// SetupWithGenesisValSet initializes a new WasmApp with a validator set and genesis accounts
// that also act as delegators. For simplicity, each validator is bonded with a delegation
// of one consensus engine unit in the default token of the WasmApp from first genesis
// account. A Nop logger is set in WasmApp.
func SetupWithGenesisValSet(
	t *testing.T,
	valSet *cmttypes.ValidatorSet,
	genAccs []authtypes.GenesisAccount,
	chainID string,
	opts []wasmkeeper.Option,
	balances ...banktypes.Balance,
) *WasmApp {
	t.Helper()

	app, genesisState := setup(t, chainID, true, 5, opts...)
	genesisState, err := GenesisStateWithValSet(app.AppCodec(), genesisState, valSet, genAccs, balances...)
	require.NoError(t, err)

	stateBytes, err := json.MarshalIndent(genesisState, "", " ")
	require.NoError(t, err)

	// init chain will set the validator set and initialize the genesis accounts
	consensusParams := simtestutil.DefaultConsensusParams
	consensusParams.Block.MaxGas = 100 * simtestutil.DefaultGenTxGas
	_, err = app.InitChain(&abci.RequestInitChain{
		ChainId:         chainID,
		Time:            time.Now().UTC(),
		Validators:      []abci.ValidatorUpdate{},
		ConsensusParams: consensusParams,
		InitialHeight:   app.LastBlockHeight() + 1,
		AppStateBytes:   stateBytes,
	})
	require.NoError(t, err)

	_, err = app.FinalizeBlock(&abci.RequestFinalizeBlock{
		Height:             app.LastBlockHeight() + 1,
		Hash:               app.LastCommitID().Hash,
		NextValidatorsHash: valSet.Hash(),
	})
	require.NoError(t, err)

	return app
}

// SetupWithEmptyStore set up a wasmd app instance with empty DB
func SetupWithEmptyStore(t testing.TB) *WasmApp {
	app, _ := setup(t, "testing", false, 0)
	return app
}

// GenesisStateWithSingleValidator initializes GenesisState with a single validator and genesis accounts
// that also act as delegators.
func GenesisStateWithSingleValidator(t *testing.T, app *WasmApp) GenesisState {
	t.Helper()

	privVal := mock.NewPV()
	pubKey, err := privVal.GetPubKey()
	require.NoError(t, err)

	// create validator set with single validator
	validator := cmttypes.NewValidator(pubKey, 1)
	valSet := cmttypes.NewValidatorSet([]*cmttypes.Validator{validator})

	// generate genesis account
	senderPrivKey := secp256k1.GenPrivKey()
	acc := authtypes.NewBaseAccount(senderPrivKey.PubKey().Address().Bytes(), senderPrivKey.PubKey(), 0, 0)
	balances := []banktypes.Balance{
		{
			Address: acc.GetAddress().String(),
			Coins:   sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100000000000000))),
		},
	}

	genesisState := app.DefaultGenesis()
	genesisState, err = GenesisStateWithValSet(app.AppCodec(), genesisState, valSet, []authtypes.GenesisAccount{acc}, balances...)
	require.NoError(t, err)

	return genesisState
}

// AddTestAddrsIncremental constructs and returns accNum amount of accounts with an
// initial balance of accAmt in random order
func AddTestAddrsIncremental(app *WasmApp, ctx sdk.Context, accNum int, accAmt sdkmath.Int) []sdk.AccAddress {
	return addTestAddrs(app, ctx, accNum, accAmt, simtestutil.CreateIncrementalAccounts)
}

func addTestAddrs(app *WasmApp, ctx sdk.Context, accNum int, accAmt sdkmath.Int, strategy simtestutil.GenerateAccountStrategy) []sdk.AccAddress {
	testAddrs := strategy(accNum)
	bondDenom, err := app.StakingKeeper.BondDenom(ctx)
	if err != nil {
		panic(err)
	}

	initCoins := sdk.NewCoins(sdk.NewCoin(bondDenom, accAmt))

	for _, addr := range testAddrs {
		initAccountWithCoins(app, ctx, addr, initCoins)
	}

	return testAddrs
}

func initAccountWithCoins(app *WasmApp, ctx sdk.Context, addr sdk.AccAddress, coins sdk.Coins) {
	err := app.BankKeeper.MintCoins(ctx, minttypes.ModuleName, coins)
	if err != nil {
		panic(err)
	}

	err = app.BankKeeper.SendCoinsFromModuleToAccount(ctx, minttypes.ModuleName, addr, coins)
	if err != nil {
		panic(err)
	}
}

var emptyWasmOptions []wasmkeeper.Option

// NewTestNetworkFixture returns a new WasmApp AppConstructor for network simulation tests
func NewTestNetworkFixture() network.TestFixture {
	dir, err := os.MkdirTemp("", "simapp")
	if err != nil {
		panic(fmt.Sprintf("failed creating temporary directory: %v", err))
	}
	defer os.RemoveAll(dir)

	app := NewWasmApp(log.NewNopLogger(), dbm.NewMemDB(), nil, true, simtestutil.NewAppOptionsWithFlagHome(dir), emptyWasmOptions)
	appCtr := func(val network.ValidatorI) servertypes.Application {
		return NewWasmApp(
			val.GetCtx().Logger, dbm.NewMemDB(), nil, true,
			simtestutil.NewAppOptionsWithFlagHome(val.GetCtx().Config.RootDir),
			emptyWasmOptions,
			bam.SetPruning(pruningtypes.NewPruningOptionsFromString(val.GetAppConfig().Pruning)),
			bam.SetMinGasPrices(val.GetAppConfig().MinGasPrices),
			bam.SetChainID(val.GetCtx().Viper.GetString(flags.FlagChainID)),
		)
	}

	return network.TestFixture{
		AppConstructor: appCtr,
		GenesisState:   app.DefaultGenesis(),
		EncodingConfig: testutil.TestEncodingConfig{
			InterfaceRegistry: app.InterfaceRegistry(),
			Codec:             app.AppCodec(),
			TxConfig:          app.TxConfig(),
			Amino:             app.LegacyAmino(),
		},
	}
}

// SignAndDeliverWithoutCommit signs and delivers a transaction. No commit
func SignAndDeliverWithoutCommit(t *testing.T, txCfg client.TxConfig, app *bam.BaseApp, msgs []sdk.Msg, fees sdk.Coins, chainID string, accNums, accSeqs []uint64, blockTime time.Time, priv ...cryptotypes.PrivKey) (*abci.ResponseFinalizeBlock, error) {
	tx, err := simtestutil.GenSignedMockTx(
		rand.New(rand.NewSource(time.Now().UnixNano())),
		txCfg,
		msgs,
		fees,
		simtestutil.DefaultGenTxGas,
		chainID,
		accNums,
		accSeqs,
		priv...,
	)
	require.NoError(t, err)

	bz, err := txCfg.TxEncoder()(tx)
	require.NoError(t, err)

	return app.FinalizeBlock(&abci.RequestFinalizeBlock{
		Height: app.LastBlockHeight() + 1,
		Time:   blockTime,
		Txs:    [][]byte{bz},
	})
}

// GenesisStateWithValSet returns a new genesis state with the validator set
// copied from simtestutil with delegation not added to supply
func GenesisStateWithValSet(
	codec codec.Codec,
	genesisState map[string]json.RawMessage,
	valSet *cmttypes.ValidatorSet,
	genAccs []authtypes.GenesisAccount,
	balances ...banktypes.Balance,
) (map[string]json.RawMessage, error) {
	// set genesis accounts
	authGenesis := authtypes.NewGenesisState(authtypes.DefaultParams(), genAccs)
	genesisState[authtypes.ModuleName] = codec.MustMarshalJSON(authGenesis)

	validators := make([]stakingtypes.Validator, 0, len(valSet.Validators))
	delegations := make([]stakingtypes.Delegation, 0, len(valSet.Validators))

	bondAmt := sdk.DefaultPowerReduction

	for _, val := range valSet.Validators {
		pk, err := cryptocodec.FromCmtPubKeyInterface(val.PubKey)
		if err != nil {
			return nil, fmt.Errorf("failed to convert pubkey: %w", err)
		}

		pkAny, err := codectypes.NewAnyWithValue(pk)
		if err != nil {
			return nil, fmt.Errorf("failed to create new any: %w", err)
		}

		validator := stakingtypes.Validator{
			OperatorAddress:   sdk.ValAddress(val.Address).String(),
			ConsensusPubkey:   pkAny,
			Jailed:            false,
			Status:            stakingtypes.Bonded,
			Tokens:            bondAmt,
			DelegatorShares:   sdkmath.LegacyOneDec(),
			Description:       stakingtypes.Description{},
			UnbondingHeight:   int64(0),
			UnbondingTime:     time.Unix(0, 0).UTC(),
			Commission:        stakingtypes.NewCommission(sdkmath.LegacyZeroDec(), sdkmath.LegacyZeroDec(), sdkmath.LegacyZeroDec()),
			MinSelfDelegation: sdkmath.ZeroInt(),
		}
		validators = append(validators, validator)
		delegations = append(delegations, stakingtypes.NewDelegation(genAccs[0].GetAddress().String(), sdk.ValAddress(val.Address).String(), sdkmath.LegacyOneDec()))

	}

	// set validators and delegations
	stakingGenesis := stakingtypes.NewGenesisState(stakingtypes.DefaultParams(), validators, delegations)
	genesisState[stakingtypes.ModuleName] = codec.MustMarshalJSON(stakingGenesis)

	signingInfos := make([]slashingtypes.SigningInfo, len(valSet.Validators))
	for i, val := range valSet.Validators {
		signingInfos[i] = slashingtypes.SigningInfo{
			Address:              sdk.ConsAddress(val.Address).String(),
			ValidatorSigningInfo: slashingtypes.ValidatorSigningInfo{},
		}
	}
	slashingGenesis := slashingtypes.NewGenesisState(slashingtypes.DefaultParams(), signingInfos, nil)
	genesisState[slashingtypes.ModuleName] = codec.MustMarshalJSON(slashingGenesis)

	// add bonded amount to bonded pool module account
	balances = append(balances, banktypes.Balance{
		Address: authtypes.NewModuleAddress(stakingtypes.BondedPoolName).String(),
		Coins:   sdk.Coins{sdk.NewCoin(sdk.DefaultBondDenom, bondAmt.MulRaw(int64(len(valSet.Validators))))},
	})

	totalSupply := sdk.NewCoins()
	for _, b := range balances {
		// add genesis acc tokens to total supply
		totalSupply = totalSupply.Add(b.Coins...)
	}

	// update total supply
	bankGenesis := banktypes.NewGenesisState(banktypes.DefaultGenesisState().Params, balances, totalSupply, []banktypes.Metadata{}, []banktypes.SendEnabled{})
	genesisState[banktypes.ModuleName] = codec.MustMarshalJSON(bankGenesis)
	println(string(genesisState[banktypes.ModuleName]))
	return genesisState, nil
}

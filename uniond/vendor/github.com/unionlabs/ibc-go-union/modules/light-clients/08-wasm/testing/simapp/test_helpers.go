package simapp

import (
	"encoding/json"
	"path/filepath"
	"testing"

	dbm "github.com/cosmos/cosmos-db"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/log"
	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/store/snapshots"
	snapshottypes "cosmossdk.io/store/snapshots/types"

	bam "github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/server"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	banktypes "cosmossdk.io/x/bank/types"

	abci "github.com/cometbft/cometbft/abci/types"
	cmttypes "github.com/cometbft/cometbft/types"

	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/internal/ibcwasm"
	"github.com/cosmos/ibc-go/v8/testing/mock"
)

func setup(tb testing.TB, chainID string, withGenesis bool, invCheckPeriod uint, mockVM ibcwasm.WasmEngine) (*SimApp, GenesisState) {
	tb.Helper()

	db := dbm.NewMemDB()
	nodeHome := tb.TempDir()
	snapshotDir := filepath.Join(nodeHome, "data", "snapshots")

	snapshotDB, err := dbm.NewDB("metadata", dbm.GoLevelDBBackend, snapshotDir)
	require.NoError(tb, err)
	tb.Cleanup(func() { snapshotDB.Close() })
	snapshotStore, err := snapshots.NewStore(snapshotDB, snapshotDir)
	require.NoError(tb, err)

	appOptions := make(simtestutil.AppOptionsMap, 0)
	appOptions[flags.FlagHome] = nodeHome // ensure unique folder
	appOptions[server.FlagInvCheckPeriod] = invCheckPeriod
	app := NewSimApp(log.NewNopLogger(), db, nil, true, appOptions, mockVM, bam.SetChainID(chainID), bam.SetSnapshot(snapshotStore, snapshottypes.SnapshotOptions{KeepRecent: 2}))

	if withGenesis {
		return app, app.DefaultGenesis()
	}

	return app, GenesisState{}
}

// SetupWithEmptyStore set up a simapp instance with empty DB
func SetupWithEmptyStore(tb testing.TB, mockVM ibcwasm.WasmEngine) *SimApp {
	tb.Helper()

	app, _ := setup(tb, "", false, 0, mockVM)
	return app
}

// SetupWithGenesisValSet initializes a new SimApp with a validator set and genesis accounts
// that also act as delegators. For simplicity, each validator is bonded with a delegation
// of one consensus engine unit in the default token of the simapp from first genesis
// account. A Nop logger is set in SimApp.
func SetupWithGenesisValSetSnapshotter(t *testing.T, mockVM ibcwasm.WasmEngine, valSet *cmttypes.ValidatorSet, genAccs []authtypes.GenesisAccount, balances ...banktypes.Balance) *SimApp {
	t.Helper()

	app, genesisState := setup(t, "", true, 5, mockVM)
	genesisState, err := simtestutil.GenesisStateWithValSet(app.AppCodec(), genesisState, valSet, genAccs, balances...)
	require.NoError(t, err)

	stateBytes, err := json.MarshalIndent(genesisState, "", " ")
	require.NoError(t, err)

	// init chain will set the validator set and initialize the genesis accounts
	_, err = app.InitChain(&abci.RequestInitChain{
		Validators:      []abci.ValidatorUpdate{},
		ConsensusParams: simtestutil.DefaultConsensusParams,
		AppStateBytes:   stateBytes,
	})
	require.NoError(t, err)

	return app
}

// SetupWithSnapshotter initializes a new SimApp with a configured snapshot db. A Nop logger is set in SimApp.
func SetupWithSnapshotter(t *testing.T, mockVM ibcwasm.WasmEngine) *SimApp {
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

	app := SetupWithGenesisValSetSnapshotter(t, mockVM, valSet, []authtypes.GenesisAccount{acc}, balance)

	return app
}

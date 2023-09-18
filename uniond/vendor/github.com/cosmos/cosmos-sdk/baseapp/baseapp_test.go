package baseapp_test

import (
	"fmt"
	"math/rand"
	"testing"
	"time"

	dbm "github.com/cometbft/cometbft-db"
	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/libs/log"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/baseapp"
	baseapptestutil "github.com/cosmos/cosmos-sdk/baseapp/testutil"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/snapshots"
	snapshottypes "github.com/cosmos/cosmos-sdk/snapshots/types"
	pruningtypes "github.com/cosmos/cosmos-sdk/store/pruning/types"
	"github.com/cosmos/cosmos-sdk/store/rootmulti"
	storetypes "github.com/cosmos/cosmos-sdk/store/types"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	authtx "github.com/cosmos/cosmos-sdk/x/auth/tx"
)

var (
	capKey1 = sdk.NewKVStoreKey("key1")
	capKey2 = sdk.NewKVStoreKey("key2")

	// testTxPriority is the CheckTx priority that we set in the test
	// AnteHandler.
	testTxPriority = int64(42)
)

type (
	BaseAppSuite struct {
		baseApp  *baseapp.BaseApp
		cdc      *codec.ProtoCodec
		txConfig client.TxConfig
	}

	SnapshotsConfig struct {
		blocks             uint64
		blockTxs           int
		snapshotInterval   uint64
		snapshotKeepRecent uint32
		pruningOpts        pruningtypes.PruningOptions
	}
)

func NewBaseAppSuite(t *testing.T, opts ...func(*baseapp.BaseApp)) *BaseAppSuite {
	cdc := codec.NewProtoCodec(codectypes.NewInterfaceRegistry())
	baseapptestutil.RegisterInterfaces(cdc.InterfaceRegistry())

	txConfig := authtx.NewTxConfig(cdc, authtx.DefaultSignModes)
	logger := defaultLogger()
	db := dbm.NewMemDB()

	app := baseapp.NewBaseApp(t.Name(), logger, db, txConfig.TxDecoder(), opts...)
	require.Equal(t, t.Name(), app.Name())

	app.SetInterfaceRegistry(cdc.InterfaceRegistry())
	app.MsgServiceRouter().SetInterfaceRegistry(cdc.InterfaceRegistry())
	app.MountStores(capKey1, capKey2)
	app.SetParamStore(&paramStore{db: dbm.NewMemDB()})
	app.SetTxDecoder(txConfig.TxDecoder())
	app.SetTxEncoder(txConfig.TxEncoder())

	// mount stores and seal
	require.Nil(t, app.LoadLatestVersion())

	return &BaseAppSuite{
		baseApp:  app,
		cdc:      cdc,
		txConfig: txConfig,
	}
}

func NewBaseAppSuiteWithSnapshots(t *testing.T, cfg SnapshotsConfig, opts ...func(*baseapp.BaseApp)) *BaseAppSuite {
	snapshotTimeout := 1 * time.Minute
	snapshotStore, err := snapshots.NewStore(dbm.NewMemDB(), testutil.GetTempDir(t))
	require.NoError(t, err)

	suite := NewBaseAppSuite(
		t,
		append(
			opts,
			baseapp.SetSnapshot(snapshotStore, snapshottypes.NewSnapshotOptions(cfg.snapshotInterval, cfg.snapshotKeepRecent)),
			baseapp.SetPruning(cfg.pruningOpts),
		)...,
	)

	baseapptestutil.RegisterKeyValueServer(suite.baseApp.MsgServiceRouter(), MsgKeyValueImpl{})

	suite.baseApp.InitChain(abci.RequestInitChain{
		ConsensusParams: &tmproto.ConsensusParams{},
	})

	r := rand.New(rand.NewSource(3920758213583))
	keyCounter := 0

	for height := int64(1); height <= int64(cfg.blocks); height++ {
		suite.baseApp.BeginBlock(abci.RequestBeginBlock{Header: tmproto.Header{Height: height}})

		for txNum := 0; txNum < cfg.blockTxs; txNum++ {
			msgs := []sdk.Msg{}
			for msgNum := 0; msgNum < 100; msgNum++ {
				key := []byte(fmt.Sprintf("%v", keyCounter))
				value := make([]byte, 10000)

				_, err := r.Read(value)
				require.NoError(t, err)

				msgs = append(msgs, &baseapptestutil.MsgKeyValue{Key: key, Value: value})
				keyCounter++
			}

			builder := suite.txConfig.NewTxBuilder()
			builder.SetMsgs(msgs...)
			setTxSignature(t, builder, 0)

			txBytes, err := suite.txConfig.TxEncoder()(builder.GetTx())
			require.NoError(t, err)

			resp := suite.baseApp.DeliverTx(abci.RequestDeliverTx{Tx: txBytes})
			require.True(t, resp.IsOK(), "%v", resp.String())
		}

		suite.baseApp.EndBlock(abci.RequestEndBlock{Height: height})
		suite.baseApp.Commit()

		// wait for snapshot to be taken, since it happens asynchronously
		if cfg.snapshotInterval > 0 && uint64(height)%cfg.snapshotInterval == 0 {
			start := time.Now()
			for {
				if time.Since(start) > snapshotTimeout {
					t.Errorf("timed out waiting for snapshot after %v", snapshotTimeout)
				}

				snapshot, err := snapshotStore.Get(uint64(height), snapshottypes.CurrentFormat)
				require.NoError(t, err)

				if snapshot != nil {
					break
				}

				time.Sleep(100 * time.Millisecond)
			}
		}
	}

	return suite
}

func TestLoadVersion(t *testing.T) {
	logger := defaultLogger()
	pruningOpt := baseapp.SetPruning(pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	db := dbm.NewMemDB()
	name := t.Name()
	app := baseapp.NewBaseApp(name, logger, db, nil, pruningOpt)

	// make a cap key and mount the store
	err := app.LoadLatestVersion() // needed to make stores non-nil
	require.Nil(t, err)

	emptyCommitID := storetypes.CommitID{}

	// fresh store has zero/empty last commit
	lastHeight := app.LastBlockHeight()
	lastID := app.LastCommitID()
	require.Equal(t, int64(0), lastHeight)
	require.Equal(t, emptyCommitID, lastID)

	// execute a block, collect commit ID
	header := tmproto.Header{Height: 1}
	app.BeginBlock(abci.RequestBeginBlock{Header: header})
	res := app.Commit()
	commitID1 := storetypes.CommitID{Version: 1, Hash: res.Data}

	// execute a block, collect commit ID
	header = tmproto.Header{Height: 2}
	app.BeginBlock(abci.RequestBeginBlock{Header: header})
	res = app.Commit()
	commitID2 := storetypes.CommitID{Version: 2, Hash: res.Data}

	// reload with LoadLatestVersion
	app = baseapp.NewBaseApp(name, logger, db, nil, pruningOpt)
	app.MountStores()

	err = app.LoadLatestVersion()
	require.Nil(t, err)

	testLoadVersionHelper(t, app, int64(2), commitID2)

	// Reload with LoadVersion, see if you can commit the same block and get
	// the same result.
	app = baseapp.NewBaseApp(name, logger, db, nil, pruningOpt)
	err = app.LoadVersion(1)
	require.Nil(t, err)

	testLoadVersionHelper(t, app, int64(1), commitID1)

	app.BeginBlock(abci.RequestBeginBlock{Header: header})
	app.Commit()

	testLoadVersionHelper(t, app, int64(2), commitID2)
}

func TestSetLoader(t *testing.T) {
	useDefaultLoader := func(app *baseapp.BaseApp) {
		app.SetStoreLoader(baseapp.DefaultStoreLoader)
	}

	initStore := func(t *testing.T, db dbm.DB, storeKey string, k, v []byte) {
		rs := rootmulti.NewStore(db, log.NewNopLogger())
		rs.SetPruning(pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))

		key := sdk.NewKVStoreKey(storeKey)
		rs.MountStoreWithDB(key, storetypes.StoreTypeIAVL, nil)

		err := rs.LoadLatestVersion()
		require.Nil(t, err)
		require.Equal(t, int64(0), rs.LastCommitID().Version)

		// write some data in substore
		kv, _ := rs.GetStore(key).(storetypes.KVStore)
		require.NotNil(t, kv)
		kv.Set(k, v)

		commitID := rs.Commit()
		require.Equal(t, int64(1), commitID.Version)
	}

	checkStore := func(t *testing.T, db dbm.DB, ver int64, storeKey string, k, v []byte) {
		rs := rootmulti.NewStore(db, log.NewNopLogger())
		rs.SetPruning(pruningtypes.NewPruningOptions(pruningtypes.PruningDefault))

		key := sdk.NewKVStoreKey(storeKey)
		rs.MountStoreWithDB(key, storetypes.StoreTypeIAVL, nil)

		err := rs.LoadLatestVersion()
		require.Nil(t, err)
		require.Equal(t, ver, rs.LastCommitID().Version)

		// query data in substore
		kv, _ := rs.GetStore(key).(storetypes.KVStore)
		require.NotNil(t, kv)
		require.Equal(t, v, kv.Get(k))
	}

	testCases := map[string]struct {
		setLoader    func(*baseapp.BaseApp)
		origStoreKey string
		loadStoreKey string
	}{
		"don't set loader": {
			origStoreKey: "foo",
			loadStoreKey: "foo",
		},
		"default loader": {
			setLoader:    useDefaultLoader,
			origStoreKey: "foo",
			loadStoreKey: "foo",
		},
	}

	k := []byte("key")
	v := []byte("value")

	for name, tc := range testCases {
		t.Run(name, func(t *testing.T) {
			// prepare a db with some data
			db := dbm.NewMemDB()
			initStore(t, db, tc.origStoreKey, k, v)

			// load the app with the existing db
			opts := []func(*baseapp.BaseApp){baseapp.SetPruning(pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))}
			if tc.setLoader != nil {
				opts = append(opts, tc.setLoader)
			}
			app := baseapp.NewBaseApp(t.Name(), defaultLogger(), db, nil, opts...)
			app.MountStores(sdk.NewKVStoreKey(tc.loadStoreKey))
			err := app.LoadLatestVersion()
			require.Nil(t, err)

			// "execute" one block
			app.BeginBlock(abci.RequestBeginBlock{Header: tmproto.Header{Height: 2}})
			res := app.Commit()
			require.NotNil(t, res.Data)

			// check db is properly updated
			checkStore(t, db, 2, tc.loadStoreKey, k, v)
			checkStore(t, db, 2, tc.loadStoreKey, []byte("foo"), nil)
		})
	}
}

func TestVersionSetterGetter(t *testing.T) {
	logger := defaultLogger()
	pruningOpt := baseapp.SetPruning(pruningtypes.NewPruningOptions(pruningtypes.PruningDefault))
	db := dbm.NewMemDB()
	name := t.Name()
	app := baseapp.NewBaseApp(name, logger, db, nil, pruningOpt)

	require.Equal(t, "", app.Version())
	res := app.Query(abci.RequestQuery{Path: "app/version"})
	require.True(t, res.IsOK())
	require.Equal(t, "", string(res.Value))

	versionString := "1.0.0"
	app.SetVersion(versionString)
	require.Equal(t, versionString, app.Version())

	res = app.Query(abci.RequestQuery{Path: "app/version"})
	require.True(t, res.IsOK())
	require.Equal(t, versionString, string(res.Value))
}

func TestLoadVersionInvalid(t *testing.T) {
	logger := log.NewNopLogger()
	pruningOpt := baseapp.SetPruning(pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	db := dbm.NewMemDB()
	name := t.Name()
	app := baseapp.NewBaseApp(name, logger, db, nil, pruningOpt)

	err := app.LoadLatestVersion()
	require.Nil(t, err)

	// require error when loading an invalid version
	err = app.LoadVersion(-1)
	require.Error(t, err)

	header := tmproto.Header{Height: 1}
	app.BeginBlock(abci.RequestBeginBlock{Header: header})
	res := app.Commit()
	commitID1 := storetypes.CommitID{Version: 1, Hash: res.Data}

	// create a new app with the stores mounted under the same cap key
	app = baseapp.NewBaseApp(name, logger, db, nil, pruningOpt)

	// require we can load the latest version
	err = app.LoadVersion(1)
	require.Nil(t, err)
	testLoadVersionHelper(t, app, int64(1), commitID1)

	// require error when loading an invalid version
	err = app.LoadVersion(2)
	require.Error(t, err)
}

func TestOptionFunction(t *testing.T) {
	testChangeNameHelper := func(name string) func(*baseapp.BaseApp) {
		return func(bap *baseapp.BaseApp) {
			bap.SetName(name)
		}
	}

	logger := defaultLogger()
	db := dbm.NewMemDB()
	bap := baseapp.NewBaseApp("starting name", logger, db, nil, testChangeNameHelper("new name"))
	require.Equal(t, bap.Name(), "new name", "BaseApp should have had name changed via option function")
}

func TestBaseAppOptionSeal(t *testing.T) {
	suite := NewBaseAppSuite(t)

	require.Panics(t, func() {
		suite.baseApp.SetName("")
	})
	require.Panics(t, func() {
		suite.baseApp.SetVersion("")
	})
	require.Panics(t, func() {
		suite.baseApp.SetDB(nil)
	})
	require.Panics(t, func() {
		suite.baseApp.SetCMS(nil)
	})
	require.Panics(t, func() {
		suite.baseApp.SetInitChainer(nil)
	})
	require.Panics(t, func() {
		suite.baseApp.SetBeginBlocker(nil)
	})
	require.Panics(t, func() {
		suite.baseApp.SetEndBlocker(nil)
	})
	require.Panics(t, func() {
		suite.baseApp.SetAnteHandler(nil)
	})
	require.Panics(t, func() {
		suite.baseApp.SetAddrPeerFilter(nil)
	})
	require.Panics(t, func() {
		suite.baseApp.SetIDPeerFilter(nil)
	})
	require.Panics(t, func() {
		suite.baseApp.SetFauxMerkleMode()
	})
}

func TestTxDecoder(t *testing.T) {
	cdc := codec.NewProtoCodec(codectypes.NewInterfaceRegistry())
	baseapptestutil.RegisterInterfaces(cdc.InterfaceRegistry())

	// patch in TxConfig instead of using an output from x/auth/tx
	txConfig := authtx.NewTxConfig(cdc, authtx.DefaultSignModes)

	tx := newTxCounter(t, txConfig, 1, 0)
	txBytes, err := txConfig.TxEncoder()(tx)
	require.NoError(t, err)

	dTx, err := txConfig.TxDecoder()(txBytes)
	require.NoError(t, err)

	counter, _ := parseTxMemo(t, tx)
	dTxCounter, _ := parseTxMemo(t, dTx)
	require.Equal(t, counter, dTxCounter)
}

func TestCustomRunTxPanicHandler(t *testing.T) {
	customPanicMsg := "test panic"
	anteErr := sdkerrors.Register("fakeModule", 100500, "fakeError")
	anteOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetAnteHandler(func(ctx sdk.Context, tx sdk.Tx, simulate bool) (newCtx sdk.Context, err error) {
			panic(sdkerrors.Wrap(anteErr, "anteHandler"))
		})
	}
	suite := NewBaseAppSuite(t, anteOpt)

	suite.baseApp.InitChain(abci.RequestInitChain{
		ConsensusParams: &tmproto.ConsensusParams{},
	})

	header := tmproto.Header{Height: 1}
	suite.baseApp.BeginBlock(abci.RequestBeginBlock{Header: header})

	suite.baseApp.AddRunTxRecoveryHandler(func(recoveryObj interface{}) error {
		err, ok := recoveryObj.(error)
		if !ok {
			return nil
		}

		if anteErr.Is(err) {
			panic(customPanicMsg)
		} else {
			return nil
		}
	})

	// transaction should panic with custom handler above
	{
		tx := newTxCounter(t, suite.txConfig, 0, 0)

		require.PanicsWithValue(t, customPanicMsg, func() {
			suite.baseApp.SimDeliver(suite.txConfig.TxEncoder(), tx)
		})
	}
}

func TestBaseAppAnteHandler(t *testing.T) {
	anteKey := []byte("ante-key")
	anteOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetAnteHandler(anteHandlerTxTest(t, capKey1, anteKey))
	}
	suite := NewBaseAppSuite(t, anteOpt)

	deliverKey := []byte("deliver-key")
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), CounterServerImpl{t, capKey1, deliverKey})

	suite.baseApp.InitChain(abci.RequestInitChain{
		ConsensusParams: &tmproto.ConsensusParams{},
	})

	header := tmproto.Header{Height: suite.baseApp.LastBlockHeight() + 1}
	suite.baseApp.BeginBlock(abci.RequestBeginBlock{Header: header})

	// execute a tx that will fail ante handler execution
	//
	// NOTE: State should not be mutated here. This will be implicitly checked by
	// the next txs ante handler execution (anteHandlerTxTest).
	tx := newTxCounter(t, suite.txConfig, 0, 0)
	tx = setFailOnAnte(t, suite.txConfig, tx, true)

	txBytes, err := suite.txConfig.TxEncoder()(tx)
	require.NoError(t, err)

	res := suite.baseApp.DeliverTx(abci.RequestDeliverTx{Tx: txBytes})
	require.Empty(t, res.Events)
	require.False(t, res.IsOK(), fmt.Sprintf("%v", res))

	ctx := getDeliverStateCtx(suite.baseApp)
	store := ctx.KVStore(capKey1)
	require.Equal(t, int64(0), getIntFromStore(t, store, anteKey))

	// execute at tx that will pass the ante handler (the checkTx state should
	// mutate) but will fail the message handler
	tx = newTxCounter(t, suite.txConfig, 0, 0)
	tx = setFailOnHandler(suite.txConfig, tx, true)

	txBytes, err = suite.txConfig.TxEncoder()(tx)
	require.NoError(t, err)

	res = suite.baseApp.DeliverTx(abci.RequestDeliverTx{Tx: txBytes})
	require.NotEmpty(t, res.Events)
	require.False(t, res.IsOK(), fmt.Sprintf("%v", res))

	ctx = getDeliverStateCtx(suite.baseApp)
	store = ctx.KVStore(capKey1)
	require.Equal(t, int64(1), getIntFromStore(t, store, anteKey))
	require.Equal(t, int64(0), getIntFromStore(t, store, deliverKey))

	// Execute a successful ante handler and message execution where state is
	// implicitly checked by previous tx executions.
	tx = newTxCounter(t, suite.txConfig, 1, 0)

	txBytes, err = suite.txConfig.TxEncoder()(tx)
	require.NoError(t, err)

	res = suite.baseApp.DeliverTx(abci.RequestDeliverTx{Tx: txBytes})
	require.NotEmpty(t, res.Events)
	require.True(t, res.IsOK(), fmt.Sprintf("%v", res))

	ctx = getDeliverStateCtx(suite.baseApp)
	store = ctx.KVStore(capKey1)
	require.Equal(t, int64(2), getIntFromStore(t, store, anteKey))
	require.Equal(t, int64(1), getIntFromStore(t, store, deliverKey))

	suite.baseApp.EndBlock(abci.RequestEndBlock{})
	suite.baseApp.Commit()
}

// Test and ensure that invalid block heights always cause errors.
// See issues:
// - https://github.com/cosmos/cosmos-sdk/issues/11220
// - https://github.com/cosmos/cosmos-sdk/issues/7662
func TestABCI_CreateQueryContext(t *testing.T) {
	t.Parallel()

	logger := defaultLogger()
	db := dbm.NewMemDB()
	name := t.Name()
	app := baseapp.NewBaseApp(name, logger, db, nil)

	app.BeginBlock(abci.RequestBeginBlock{Header: tmproto.Header{Height: 1}})
	app.Commit()

	app.BeginBlock(abci.RequestBeginBlock{Header: tmproto.Header{Height: 2}})
	app.Commit()

	testCases := []struct {
		name   string
		height int64
		prove  bool
		expErr bool
	}{
		{"valid height", 2, true, false},
		{"future height", 10, true, true},
		{"negative height, prove=true", -1, true, true},
		{"negative height, prove=false", -1, false, true},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			_, err := app.CreateQueryContext(tc.height, tc.prove)
			if tc.expErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestSetMinGasPrices(t *testing.T) {
	minGasPrices := sdk.DecCoins{sdk.NewInt64DecCoin("stake", 5000)}
	suite := NewBaseAppSuite(t, baseapp.SetMinGasPrices(minGasPrices.String()))

	ctx := getCheckStateCtx(suite.baseApp)
	require.Equal(t, minGasPrices, ctx.MinGasPrices())
}

func TestGetMaximumBlockGas(t *testing.T) {
	suite := NewBaseAppSuite(t)
	suite.baseApp.InitChain(abci.RequestInitChain{})
	ctx := suite.baseApp.NewContext(true, tmproto.Header{})

	suite.baseApp.StoreConsensusParams(ctx, &tmproto.ConsensusParams{Block: &tmproto.BlockParams{MaxGas: 0}})
	require.Equal(t, uint64(0), suite.baseApp.GetMaximumBlockGas(ctx))

	suite.baseApp.StoreConsensusParams(ctx, &tmproto.ConsensusParams{Block: &tmproto.BlockParams{MaxGas: -1}})
	require.Equal(t, uint64(0), suite.baseApp.GetMaximumBlockGas(ctx))

	suite.baseApp.StoreConsensusParams(ctx, &tmproto.ConsensusParams{Block: &tmproto.BlockParams{MaxGas: 5000000}})
	require.Equal(t, uint64(5000000), suite.baseApp.GetMaximumBlockGas(ctx))

	suite.baseApp.StoreConsensusParams(ctx, &tmproto.ConsensusParams{Block: &tmproto.BlockParams{MaxGas: -5000000}})
	require.Panics(t, func() { suite.baseApp.GetMaximumBlockGas(ctx) })
}

func TestLoadVersionPruning(t *testing.T) {
	logger := log.NewNopLogger()
	pruningOptions := pruningtypes.NewCustomPruningOptions(10, 15)
	pruningOpt := baseapp.SetPruning(pruningOptions)
	db := dbm.NewMemDB()
	name := t.Name()
	app := baseapp.NewBaseApp(name, logger, db, nil, pruningOpt)

	// make a cap key and mount the store
	capKey := sdk.NewKVStoreKey("key1")
	app.MountStores(capKey)

	err := app.LoadLatestVersion() // needed to make stores non-nil
	require.Nil(t, err)

	emptyCommitID := storetypes.CommitID{}

	// fresh store has zero/empty last commit
	lastHeight := app.LastBlockHeight()
	lastID := app.LastCommitID()
	require.Equal(t, int64(0), lastHeight)
	require.Equal(t, emptyCommitID, lastID)

	var lastCommitID storetypes.CommitID

	// Commit seven blocks, of which 7 (latest) is kept in addition to 6, 5
	// (keep recent) and 3 (keep every).
	for i := int64(1); i <= 7; i++ {
		app.BeginBlock(abci.RequestBeginBlock{Header: tmproto.Header{Height: i}})
		res := app.Commit()
		lastCommitID = storetypes.CommitID{Version: i, Hash: res.Data}
	}

	for _, v := range []int64{1, 2, 4} {
		_, err = app.CommitMultiStore().CacheMultiStoreWithVersion(v)
		require.NoError(t, err)
	}

	for _, v := range []int64{3, 5, 6, 7} {
		_, err = app.CommitMultiStore().CacheMultiStoreWithVersion(v)
		require.NoError(t, err)
	}

	// reload with LoadLatestVersion, check it loads last version
	app = baseapp.NewBaseApp(name, logger, db, nil, pruningOpt)
	app.MountStores(capKey)

	err = app.LoadLatestVersion()
	require.Nil(t, err)
	testLoadVersionHelper(t, app, int64(7), lastCommitID)
}

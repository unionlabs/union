package baseapp_test

import (
	"bytes"
	"errors"
	"fmt"
	"strconv"
	"strings"
	"testing"

	dbm "github.com/cometbft/cometbft-db"
	abci "github.com/cometbft/cometbft/abci/types"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/cosmos/gogoproto/jsonpb"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/baseapp"
	baseapptestutil "github.com/cosmos/cosmos-sdk/baseapp/testutil"
	"github.com/cosmos/cosmos-sdk/snapshots"
	snapshottypes "github.com/cosmos/cosmos-sdk/snapshots/types"
	pruningtypes "github.com/cosmos/cosmos-sdk/store/pruning/types"
	"github.com/cosmos/cosmos-sdk/testutil"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/mempool"
	"github.com/cosmos/cosmos-sdk/x/auth/signing"
)

func TestABCI_Info(t *testing.T) {
	suite := NewBaseAppSuite(t)

	reqInfo := abci.RequestInfo{}
	res := suite.baseApp.Info(reqInfo)

	require.Equal(t, "", res.Version)
	require.Equal(t, t.Name(), res.GetData())
	require.Equal(t, int64(0), res.LastBlockHeight)
	require.Equal(t, []uint8(nil), res.LastBlockAppHash)
	require.Equal(t, suite.baseApp.AppVersion(), res.AppVersion)
}

func TestABCI_InitChain(t *testing.T) {
	name := t.Name()
	db := dbm.NewMemDB()
	logger := defaultLogger()
	app := baseapp.NewBaseApp(name, logger, db, nil, baseapp.SetChainID("test-chain-id"))

	capKey := sdk.NewKVStoreKey("main")
	capKey2 := sdk.NewKVStoreKey("key2")
	app.MountStores(capKey, capKey2)

	// set a value in the store on init chain
	key, value := []byte("hello"), []byte("goodbye")
	var initChainer sdk.InitChainer = func(ctx sdk.Context, req abci.RequestInitChain) abci.ResponseInitChain {
		store := ctx.KVStore(capKey)
		store.Set(key, value)
		return abci.ResponseInitChain{}
	}

	query := abci.RequestQuery{
		Path: "/store/main/key",
		Data: key,
	}

	// initChain is nil and chain ID is wrong - panics
	require.Panics(t, func() {
		app.InitChain(abci.RequestInitChain{ChainId: "wrong-chain-id"})
	})

	// initChain is nil - nothing happens
	app.InitChain(abci.RequestInitChain{ChainId: "test-chain-id"})
	res := app.Query(query)
	require.Equal(t, 0, len(res.Value))

	// set initChainer and try again - should see the value
	app.SetInitChainer(initChainer)

	// stores are mounted and private members are set - sealing baseapp
	err := app.LoadLatestVersion() // needed to make stores non-nil
	require.Nil(t, err)
	require.Equal(t, int64(0), app.LastBlockHeight())

	initChainRes := app.InitChain(abci.RequestInitChain{AppStateBytes: []byte("{}"), ChainId: "test-chain-id"}) // must have valid JSON genesis file, even if empty

	// The AppHash returned by a new chain is the sha256 hash of "".
	// $ echo -n '' | sha256sum
	// e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
	require.Equal(
		t,
		[]byte{0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f, 0xb9, 0x24, 0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b, 0x78, 0x52, 0xb8, 0x55},
		initChainRes.AppHash,
	)

	// assert that chainID is set correctly in InitChain
	chainID := getDeliverStateCtx(app).ChainID()
	require.Equal(t, "test-chain-id", chainID, "ChainID in deliverState not set correctly in InitChain")

	chainID = getCheckStateCtx(app).ChainID()
	require.Equal(t, "test-chain-id", chainID, "ChainID in checkState not set correctly in InitChain")

	app.Commit()
	res = app.Query(query)
	require.Equal(t, int64(1), app.LastBlockHeight())
	require.Equal(t, value, res.Value)

	// reload app
	app = baseapp.NewBaseApp(name, logger, db, nil)
	app.SetInitChainer(initChainer)
	app.MountStores(capKey, capKey2)
	err = app.LoadLatestVersion() // needed to make stores non-nil
	require.Nil(t, err)
	require.Equal(t, int64(1), app.LastBlockHeight())

	// ensure we can still query after reloading
	res = app.Query(query)
	require.Equal(t, value, res.Value)

	// commit and ensure we can still query
	header := tmproto.Header{Height: app.LastBlockHeight() + 1}
	app.BeginBlock(abci.RequestBeginBlock{Header: header})
	app.Commit()

	res = app.Query(query)
	require.Equal(t, value, res.Value)
}

func TestABCI_InitChain_WithInitialHeight(t *testing.T) {
	name := t.Name()
	db := dbm.NewMemDB()
	logger := defaultLogger()
	app := baseapp.NewBaseApp(name, logger, db, nil)

	app.InitChain(
		abci.RequestInitChain{
			InitialHeight: 3,
		},
	)
	app.Commit()

	require.Equal(t, int64(3), app.LastBlockHeight())
}

func TestABCI_BeginBlock_WithInitialHeight(t *testing.T) {
	name := t.Name()
	db := dbm.NewMemDB()
	logger := defaultLogger()
	app := baseapp.NewBaseApp(name, logger, db, nil)

	app.InitChain(
		abci.RequestInitChain{
			InitialHeight: 3,
		},
	)

	require.PanicsWithError(t, "invalid height: 4; expected: 3", func() {
		app.BeginBlock(abci.RequestBeginBlock{
			Header: tmproto.Header{
				Height: 4,
			},
		})
	})

	app.BeginBlock(abci.RequestBeginBlock{
		Header: tmproto.Header{
			Height: 3,
		},
	})
	app.Commit()

	require.Equal(t, int64(3), app.LastBlockHeight())
}

func TestABCI_GRPCQuery(t *testing.T) {
	grpcQueryOpt := func(bapp *baseapp.BaseApp) {
		testdata.RegisterQueryServer(
			bapp.GRPCQueryRouter(),
			testdata.QueryImpl{},
		)
	}

	suite := NewBaseAppSuite(t, grpcQueryOpt)

	suite.baseApp.InitChain(abci.RequestInitChain{
		ConsensusParams: &tmproto.ConsensusParams{},
	})

	req := testdata.SayHelloRequest{Name: "foo"}
	reqBz, err := req.Marshal()
	require.NoError(t, err)

	resQuery := suite.baseApp.Query(abci.RequestQuery{
		Data: reqBz,
		Path: "/testpb.Query/SayHello",
	})
	require.Equal(t, sdkerrors.ErrInvalidHeight.ABCICode(), resQuery.Code, resQuery)
	require.Contains(t, resQuery.Log, "TestABCI_GRPCQuery is not ready; please wait for first block")

	header := tmproto.Header{Height: suite.baseApp.LastBlockHeight() + 1}
	suite.baseApp.BeginBlock(abci.RequestBeginBlock{Header: header})
	suite.baseApp.Commit()

	reqQuery := abci.RequestQuery{
		Data: reqBz,
		Path: "/testpb.Query/SayHello",
	}

	resQuery = suite.baseApp.Query(reqQuery)
	require.Equal(t, abci.CodeTypeOK, resQuery.Code, resQuery)

	var res testdata.SayHelloResponse
	require.NoError(t, res.Unmarshal(resQuery.Value))
	require.Equal(t, "Hello foo!", res.Greeting)
}

func TestABCI_P2PQuery(t *testing.T) {
	addrPeerFilterOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetAddrPeerFilter(func(addrport string) abci.ResponseQuery {
			require.Equal(t, "1.1.1.1:8000", addrport)
			return abci.ResponseQuery{Code: uint32(3)}
		})
	}

	idPeerFilterOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetIDPeerFilter(func(id string) abci.ResponseQuery {
			require.Equal(t, "testid", id)
			return abci.ResponseQuery{Code: uint32(4)}
		})
	}

	suite := NewBaseAppSuite(t, addrPeerFilterOpt, idPeerFilterOpt)

	addrQuery := abci.RequestQuery{
		Path: "/p2p/filter/addr/1.1.1.1:8000",
	}
	res := suite.baseApp.Query(addrQuery)
	require.Equal(t, uint32(3), res.Code)

	idQuery := abci.RequestQuery{
		Path: "/p2p/filter/id/testid",
	}
	res = suite.baseApp.Query(idQuery)
	require.Equal(t, uint32(4), res.Code)
}

func TestABCI_ListSnapshots(t *testing.T) {
	ssCfg := SnapshotsConfig{
		blocks:             5,
		blockTxs:           4,
		snapshotInterval:   2,
		snapshotKeepRecent: 2,
		pruningOpts:        pruningtypes.NewPruningOptions(pruningtypes.PruningNothing),
	}

	suite := NewBaseAppSuiteWithSnapshots(t, ssCfg)

	resp := suite.baseApp.ListSnapshots(abci.RequestListSnapshots{})
	for _, s := range resp.Snapshots {
		require.NotEmpty(t, s.Hash)
		require.NotEmpty(t, s.Metadata)

		s.Hash = nil
		s.Metadata = nil
	}

	require.Equal(t, abci.ResponseListSnapshots{Snapshots: []*abci.Snapshot{
		{Height: 4, Format: snapshottypes.CurrentFormat, Chunks: 2},
		{Height: 2, Format: snapshottypes.CurrentFormat, Chunks: 1},
	}}, resp)
}

func TestABCI_SnapshotWithPruning(t *testing.T) {
	testCases := map[string]struct {
		ssCfg             SnapshotsConfig
		expectedSnapshots []*abci.Snapshot
	}{
		"prune nothing with snapshot": {
			ssCfg: SnapshotsConfig{
				blocks:             20,
				blockTxs:           2,
				snapshotInterval:   5,
				snapshotKeepRecent: 1,
				pruningOpts:        pruningtypes.NewPruningOptions(pruningtypes.PruningNothing),
			},
			expectedSnapshots: []*abci.Snapshot{
				{Height: 20, Format: snapshottypes.CurrentFormat, Chunks: 5},
			},
		},
		"prune everything with snapshot": {
			ssCfg: SnapshotsConfig{
				blocks:             20,
				blockTxs:           2,
				snapshotInterval:   5,
				snapshotKeepRecent: 1,
				pruningOpts:        pruningtypes.NewPruningOptions(pruningtypes.PruningEverything),
			},
			expectedSnapshots: []*abci.Snapshot{
				{Height: 20, Format: snapshottypes.CurrentFormat, Chunks: 5},
			},
		},
		"default pruning with snapshot": {
			ssCfg: SnapshotsConfig{
				blocks:             20,
				blockTxs:           2,
				snapshotInterval:   5,
				snapshotKeepRecent: 1,
				pruningOpts:        pruningtypes.NewPruningOptions(pruningtypes.PruningDefault),
			},
			expectedSnapshots: []*abci.Snapshot{
				{Height: 20, Format: snapshottypes.CurrentFormat, Chunks: 5},
			},
		},
		"custom": {
			ssCfg: SnapshotsConfig{
				blocks:             25,
				blockTxs:           2,
				snapshotInterval:   5,
				snapshotKeepRecent: 2,
				pruningOpts:        pruningtypes.NewCustomPruningOptions(12, 12),
			},
			expectedSnapshots: []*abci.Snapshot{
				{Height: 25, Format: snapshottypes.CurrentFormat, Chunks: 6},
				{Height: 20, Format: snapshottypes.CurrentFormat, Chunks: 5},
			},
		},
		"no snapshots": {
			ssCfg: SnapshotsConfig{
				blocks:           10,
				blockTxs:         2,
				snapshotInterval: 0, // 0 implies disable snapshots
				pruningOpts:      pruningtypes.NewPruningOptions(pruningtypes.PruningNothing),
			},
			expectedSnapshots: []*abci.Snapshot{},
		},
		"keep all snapshots": {
			ssCfg: SnapshotsConfig{
				blocks:             10,
				blockTxs:           2,
				snapshotInterval:   3,
				snapshotKeepRecent: 0, // 0 implies keep all snapshots
				pruningOpts:        pruningtypes.NewPruningOptions(pruningtypes.PruningNothing),
			},
			expectedSnapshots: []*abci.Snapshot{
				{Height: 9, Format: snapshottypes.CurrentFormat, Chunks: 2},
				{Height: 6, Format: snapshottypes.CurrentFormat, Chunks: 2},
				{Height: 3, Format: snapshottypes.CurrentFormat, Chunks: 1},
			},
		},
	}

	for name, tc := range testCases {
		t.Run(name, func(t *testing.T) {
			suite := NewBaseAppSuiteWithSnapshots(t, tc.ssCfg)

			resp := suite.baseApp.ListSnapshots(abci.RequestListSnapshots{})
			for _, s := range resp.Snapshots {
				require.NotEmpty(t, s.Hash)
				require.NotEmpty(t, s.Metadata)

				s.Hash = nil
				s.Metadata = nil
			}

			require.Equal(t, abci.ResponseListSnapshots{Snapshots: tc.expectedSnapshots}, resp)

			// Validate that heights were pruned correctly by querying the state at the last height that should be present relative to latest
			// and the first height that should be pruned.
			//
			// Exceptions:
			//   * Prune nothing: should be able to query all heights (we only test first and latest)
			//   * Prune default: should be able to query all heights (we only test first and latest)
			//      * The reason for default behaving this way is that we only commit 20 heights but default has 100_000 keep-recent
			var lastExistingHeight int64
			if tc.ssCfg.pruningOpts.GetPruningStrategy() == pruningtypes.PruningNothing || tc.ssCfg.pruningOpts.GetPruningStrategy() == pruningtypes.PruningDefault {
				lastExistingHeight = 1
			} else {
				// Integer division rounds down so by multiplying back we get the last height at which we pruned
				lastExistingHeight = int64((tc.ssCfg.blocks/tc.ssCfg.pruningOpts.Interval)*tc.ssCfg.pruningOpts.Interval - tc.ssCfg.pruningOpts.KeepRecent)
			}

			// Query 1
			res := suite.baseApp.Query(abci.RequestQuery{Path: fmt.Sprintf("/store/%s/key", capKey2.Name()), Data: []byte("0"), Height: lastExistingHeight})
			require.NotNil(t, res, "height: %d", lastExistingHeight)
			require.NotNil(t, res.Value, "height: %d", lastExistingHeight)

			// Query 2
			res = suite.baseApp.Query(abci.RequestQuery{Path: fmt.Sprintf("/store/%s/key", capKey2.Name()), Data: []byte("0"), Height: lastExistingHeight - 1})
			require.NotNil(t, res, "height: %d", lastExistingHeight-1)

			if tc.ssCfg.pruningOpts.GetPruningStrategy() == pruningtypes.PruningNothing || tc.ssCfg.pruningOpts.GetPruningStrategy() == pruningtypes.PruningDefault {
				// With prune nothing or default, we query height 0 which translates to the latest height.
				require.NotNil(t, res.Value, "height: %d", lastExistingHeight-1)
			}
		})
	}
}

func TestABCI_LoadSnapshotChunk(t *testing.T) {
	ssCfg := SnapshotsConfig{
		blocks:             2,
		blockTxs:           5,
		snapshotInterval:   2,
		snapshotKeepRecent: snapshottypes.CurrentFormat,
		pruningOpts:        pruningtypes.NewPruningOptions(pruningtypes.PruningNothing),
	}
	suite := NewBaseAppSuiteWithSnapshots(t, ssCfg)

	testCases := map[string]struct {
		height      uint64
		format      uint32
		chunk       uint32
		expectEmpty bool
	}{
		"Existing snapshot": {2, snapshottypes.CurrentFormat, 1, false},
		"Missing height":    {100, snapshottypes.CurrentFormat, 1, true},
		"Missing format":    {2, snapshottypes.CurrentFormat + 1, 1, true},
		"Missing chunk":     {2, snapshottypes.CurrentFormat, 9, true},
		"Zero height":       {0, snapshottypes.CurrentFormat, 1, true},
		"Zero format":       {2, 0, 1, true},
		"Zero chunk":        {2, snapshottypes.CurrentFormat, 0, false},
	}

	for name, tc := range testCases {
		t.Run(name, func(t *testing.T) {
			resp := suite.baseApp.LoadSnapshotChunk(abci.RequestLoadSnapshotChunk{
				Height: tc.height,
				Format: tc.format,
				Chunk:  tc.chunk,
			})
			if tc.expectEmpty {
				require.Equal(t, abci.ResponseLoadSnapshotChunk{}, resp)
				return
			}

			require.NotEmpty(t, resp.Chunk)
		})
	}
}

func TestABCI_OfferSnapshot_Errors(t *testing.T) {
	ssCfg := SnapshotsConfig{
		blocks:             0,
		blockTxs:           0,
		snapshotInterval:   2,
		snapshotKeepRecent: 2,
		pruningOpts:        pruningtypes.NewPruningOptions(pruningtypes.PruningNothing),
	}
	suite := NewBaseAppSuiteWithSnapshots(t, ssCfg)

	m := snapshottypes.Metadata{ChunkHashes: [][]byte{{1}, {2}, {3}}}
	metadata, err := m.Marshal()
	require.NoError(t, err)

	hash := []byte{1, 2, 3}

	testCases := map[string]struct {
		snapshot *abci.Snapshot
		result   abci.ResponseOfferSnapshot_Result
	}{
		"nil snapshot": {nil, abci.ResponseOfferSnapshot_REJECT},
		"invalid format": {&abci.Snapshot{
			Height: 1, Format: 9, Chunks: 3, Hash: hash, Metadata: metadata,
		}, abci.ResponseOfferSnapshot_REJECT_FORMAT},
		"incorrect chunk count": {&abci.Snapshot{
			Height: 1, Format: snapshottypes.CurrentFormat, Chunks: 2, Hash: hash, Metadata: metadata,
		}, abci.ResponseOfferSnapshot_REJECT},
		"no chunks": {&abci.Snapshot{
			Height: 1, Format: snapshottypes.CurrentFormat, Chunks: 0, Hash: hash, Metadata: metadata,
		}, abci.ResponseOfferSnapshot_REJECT},
		"invalid metadata serialization": {&abci.Snapshot{
			Height: 1, Format: snapshottypes.CurrentFormat, Chunks: 0, Hash: hash, Metadata: []byte{3, 1, 4},
		}, abci.ResponseOfferSnapshot_REJECT},
	}
	for name, tc := range testCases {
		tc := tc
		t.Run(name, func(t *testing.T) {
			resp := suite.baseApp.OfferSnapshot(abci.RequestOfferSnapshot{Snapshot: tc.snapshot})
			require.Equal(t, tc.result, resp.Result)
		})
	}

	// Offering a snapshot after one has been accepted should error
	resp := suite.baseApp.OfferSnapshot(abci.RequestOfferSnapshot{Snapshot: &abci.Snapshot{
		Height:   1,
		Format:   snapshottypes.CurrentFormat,
		Chunks:   3,
		Hash:     []byte{1, 2, 3},
		Metadata: metadata,
	}})
	require.Equal(t, abci.ResponseOfferSnapshot{Result: abci.ResponseOfferSnapshot_ACCEPT}, resp)

	resp = suite.baseApp.OfferSnapshot(abci.RequestOfferSnapshot{Snapshot: &abci.Snapshot{
		Height:   2,
		Format:   snapshottypes.CurrentFormat,
		Chunks:   3,
		Hash:     []byte{1, 2, 3},
		Metadata: metadata,
	}})
	require.Equal(t, abci.ResponseOfferSnapshot{Result: abci.ResponseOfferSnapshot_ABORT}, resp)
}

func TestABCI_ApplySnapshotChunk(t *testing.T) {
	srcCfg := SnapshotsConfig{
		blocks:             4,
		blockTxs:           10,
		snapshotInterval:   2,
		snapshotKeepRecent: 2,
		pruningOpts:        pruningtypes.NewPruningOptions(pruningtypes.PruningNothing),
	}
	srcSuite := NewBaseAppSuiteWithSnapshots(t, srcCfg)

	targetCfg := SnapshotsConfig{
		blocks:             0,
		blockTxs:           0,
		snapshotInterval:   2,
		snapshotKeepRecent: 2,
		pruningOpts:        pruningtypes.NewPruningOptions(pruningtypes.PruningNothing),
	}
	targetSuite := NewBaseAppSuiteWithSnapshots(t, targetCfg)

	// fetch latest snapshot to restore
	respList := srcSuite.baseApp.ListSnapshots(abci.RequestListSnapshots{})
	require.NotEmpty(t, respList.Snapshots)
	snapshot := respList.Snapshots[0]

	// make sure the snapshot has at least 3 chunks
	require.GreaterOrEqual(t, snapshot.Chunks, uint32(3), "Not enough snapshot chunks")

	// begin a snapshot restoration in the target
	respOffer := targetSuite.baseApp.OfferSnapshot(abci.RequestOfferSnapshot{Snapshot: snapshot})
	require.Equal(t, abci.ResponseOfferSnapshot{Result: abci.ResponseOfferSnapshot_ACCEPT}, respOffer)

	// We should be able to pass an invalid chunk and get a verify failure, before
	// reapplying it.
	respApply := targetSuite.baseApp.ApplySnapshotChunk(abci.RequestApplySnapshotChunk{
		Index:  0,
		Chunk:  []byte{9},
		Sender: "sender",
	})
	require.Equal(t, abci.ResponseApplySnapshotChunk{
		Result:        abci.ResponseApplySnapshotChunk_RETRY,
		RefetchChunks: []uint32{0},
		RejectSenders: []string{"sender"},
	}, respApply)

	// fetch each chunk from the source and apply it to the target
	for index := uint32(0); index < snapshot.Chunks; index++ {
		respChunk := srcSuite.baseApp.LoadSnapshotChunk(abci.RequestLoadSnapshotChunk{
			Height: snapshot.Height,
			Format: snapshot.Format,
			Chunk:  index,
		})
		require.NotNil(t, respChunk.Chunk)

		respApply := targetSuite.baseApp.ApplySnapshotChunk(abci.RequestApplySnapshotChunk{
			Index: index,
			Chunk: respChunk.Chunk,
		})
		require.Equal(t, abci.ResponseApplySnapshotChunk{
			Result: abci.ResponseApplySnapshotChunk_ACCEPT,
		}, respApply)
	}

	// the target should now have the same hash as the source
	require.Equal(t, srcSuite.baseApp.LastCommitID(), targetSuite.baseApp.LastCommitID())
}

func TestABCI_EndBlock(t *testing.T) {
	db := dbm.NewMemDB()
	name := t.Name()
	logger := defaultLogger()

	cp := &tmproto.ConsensusParams{
		Block: &tmproto.BlockParams{
			MaxGas: 5000000,
		},
	}

	app := baseapp.NewBaseApp(name, logger, db, nil)
	app.SetParamStore(&paramStore{db: dbm.NewMemDB()})
	app.InitChain(abci.RequestInitChain{
		ConsensusParams: cp,
	})

	app.SetEndBlocker(func(ctx sdk.Context, req abci.RequestEndBlock) abci.ResponseEndBlock {
		return abci.ResponseEndBlock{
			ValidatorUpdates: []abci.ValidatorUpdate{
				{Power: 100},
			},
		}
	})
	app.Seal()

	res := app.EndBlock(abci.RequestEndBlock{})
	require.Len(t, res.GetValidatorUpdates(), 1)
	require.Equal(t, int64(100), res.GetValidatorUpdates()[0].Power)
	require.Equal(t, cp.Block.MaxGas, res.ConsensusParamUpdates.Block.MaxGas)
}

func TestABCI_CheckTx(t *testing.T) {
	// This ante handler reads the key and checks that the value matches the
	// current counter. This ensures changes to the KVStore persist across
	// successive CheckTx runs.
	counterKey := []byte("counter-key")
	anteOpt := func(bapp *baseapp.BaseApp) { bapp.SetAnteHandler(anteHandlerTxTest(t, capKey1, counterKey)) }
	suite := NewBaseAppSuite(t, anteOpt)

	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), CounterServerImpl{t, capKey1, counterKey})

	nTxs := int64(5)
	suite.baseApp.InitChain(abci.RequestInitChain{
		ConsensusParams: &tmproto.ConsensusParams{},
	})

	for i := int64(0); i < nTxs; i++ {
		tx := newTxCounter(t, suite.txConfig, i, 0) // no messages
		txBytes, err := suite.txConfig.TxEncoder()(tx)
		require.NoError(t, err)

		r := suite.baseApp.CheckTx(abci.RequestCheckTx{Tx: txBytes})
		require.True(t, r.IsOK(), fmt.Sprintf("%v", r))
		require.Equal(t, testTxPriority, r.Priority)
		require.Empty(t, r.GetEvents())
	}

	checkStateStore := getCheckStateCtx(suite.baseApp).KVStore(capKey1)
	storedCounter := getIntFromStore(t, checkStateStore, counterKey)

	// ensure AnteHandler ran
	require.Equal(t, nTxs, storedCounter)

	// if a block is committed, CheckTx state should be reset
	header := tmproto.Header{Height: 1}
	suite.baseApp.BeginBlock(abci.RequestBeginBlock{Header: header, Hash: []byte("hash")})

	require.NotNil(t, getCheckStateCtx(suite.baseApp).BlockGasMeter(), "block gas meter should have been set to checkState")
	require.NotEmpty(t, getCheckStateCtx(suite.baseApp).HeaderHash())

	suite.baseApp.EndBlock(abci.RequestEndBlock{})
	suite.baseApp.Commit()

	checkStateStore = getCheckStateCtx(suite.baseApp).KVStore(capKey1)
	storedBytes := checkStateStore.Get(counterKey)
	require.Nil(t, storedBytes)
}

func TestABCI_DeliverTx(t *testing.T) {
	anteKey := []byte("ante-key")
	anteOpt := func(bapp *baseapp.BaseApp) { bapp.SetAnteHandler(anteHandlerTxTest(t, capKey1, anteKey)) }
	suite := NewBaseAppSuite(t, anteOpt)

	suite.baseApp.InitChain(abci.RequestInitChain{
		ConsensusParams: &tmproto.ConsensusParams{},
	})

	deliverKey := []byte("deliver-key")
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), CounterServerImpl{t, capKey1, deliverKey})

	nBlocks := 3
	txPerHeight := 5

	for blockN := 0; blockN < nBlocks; blockN++ {
		header := tmproto.Header{Height: int64(blockN) + 1}
		suite.baseApp.BeginBlock(abci.RequestBeginBlock{Header: header})

		for i := 0; i < txPerHeight; i++ {
			counter := int64(blockN*txPerHeight + i)
			tx := newTxCounter(t, suite.txConfig, counter, counter)

			txBytes, err := suite.txConfig.TxEncoder()(tx)
			require.NoError(t, err)

			res := suite.baseApp.DeliverTx(abci.RequestDeliverTx{Tx: txBytes})
			require.True(t, res.IsOK(), fmt.Sprintf("%v", res))

			events := res.GetEvents()
			require.Len(t, events, 3, "should contain ante handler, message type and counter events respectively")
			require.Equal(t, sdk.MarkEventsToIndex(counterEvent("ante_handler", counter).ToABCIEvents(), map[string]struct{}{})[0], events[0], "ante handler event")
			require.Equal(t, sdk.MarkEventsToIndex(counterEvent(sdk.EventTypeMessage, counter).ToABCIEvents(), map[string]struct{}{})[0], events[2], "msg handler update counter event")
		}

		suite.baseApp.EndBlock(abci.RequestEndBlock{})
		suite.baseApp.Commit()
	}
}

func TestABCI_DeliverTx_MultiMsg(t *testing.T) {
	anteKey := []byte("ante-key")
	anteOpt := func(bapp *baseapp.BaseApp) { bapp.SetAnteHandler(anteHandlerTxTest(t, capKey1, anteKey)) }
	suite := NewBaseAppSuite(t, anteOpt)

	suite.baseApp.InitChain(abci.RequestInitChain{
		ConsensusParams: &tmproto.ConsensusParams{},
	})

	deliverKey := []byte("deliver-key")
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), CounterServerImpl{t, capKey1, deliverKey})

	deliverKey2 := []byte("deliver-key2")
	baseapptestutil.RegisterCounter2Server(suite.baseApp.MsgServiceRouter(), Counter2ServerImpl{t, capKey1, deliverKey2})

	// run a multi-msg tx
	// with all msgs the same route
	header := tmproto.Header{Height: 1}
	suite.baseApp.BeginBlock(abci.RequestBeginBlock{Header: header})

	tx := newTxCounter(t, suite.txConfig, 0, 0, 1, 2)
	txBytes, err := suite.txConfig.TxEncoder()(tx)
	require.NoError(t, err)

	res := suite.baseApp.DeliverTx(abci.RequestDeliverTx{Tx: txBytes})
	require.True(t, res.IsOK(), fmt.Sprintf("%v", res))

	store := getDeliverStateCtx(suite.baseApp).KVStore(capKey1)

	// tx counter only incremented once
	txCounter := getIntFromStore(t, store, anteKey)
	require.Equal(t, int64(1), txCounter)

	// msg counter incremented three times
	msgCounter := getIntFromStore(t, store, deliverKey)
	require.Equal(t, int64(3), msgCounter)

	// replace the second message with a Counter2
	tx = newTxCounter(t, suite.txConfig, 1, 3)

	builder := suite.txConfig.NewTxBuilder()
	msgs := tx.GetMsgs()
	msgs = append(msgs, &baseapptestutil.MsgCounter2{Counter: 0})
	msgs = append(msgs, &baseapptestutil.MsgCounter2{Counter: 1})

	builder.SetMsgs(msgs...)
	builder.SetMemo(tx.GetMemo())
	setTxSignature(t, builder, 0)

	txBytes, err = suite.txConfig.TxEncoder()(builder.GetTx())
	require.NoError(t, err)

	res = suite.baseApp.DeliverTx(abci.RequestDeliverTx{Tx: txBytes})
	require.True(t, res.IsOK(), fmt.Sprintf("%v", res))

	store = getDeliverStateCtx(suite.baseApp).KVStore(capKey1)

	// tx counter only incremented once
	txCounter = getIntFromStore(t, store, anteKey)
	require.Equal(t, int64(2), txCounter)

	// original counter increments by one
	// new counter increments by two
	msgCounter = getIntFromStore(t, store, deliverKey)
	require.Equal(t, int64(4), msgCounter)

	msgCounter2 := getIntFromStore(t, store, deliverKey2)
	require.Equal(t, int64(2), msgCounter2)
}

func TestABCI_Query_SimulateTx(t *testing.T) {
	gasConsumed := uint64(5)
	anteOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetAnteHandler(func(ctx sdk.Context, tx sdk.Tx, simulate bool) (newCtx sdk.Context, err error) {
			newCtx = ctx.WithGasMeter(sdk.NewGasMeter(gasConsumed))
			return
		})
	}
	suite := NewBaseAppSuite(t, anteOpt)

	suite.baseApp.InitChain(abci.RequestInitChain{
		ConsensusParams: &tmproto.ConsensusParams{},
	})

	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), CounterServerImplGasMeterOnly{gasConsumed})

	nBlocks := 3
	for blockN := 0; blockN < nBlocks; blockN++ {
		count := int64(blockN + 1)
		header := tmproto.Header{Height: count}
		suite.baseApp.BeginBlock(abci.RequestBeginBlock{Header: header})

		tx := newTxCounter(t, suite.txConfig, count, count)

		txBytes, err := suite.txConfig.TxEncoder()(tx)
		require.Nil(t, err)

		// simulate a message, check gas reported
		gInfo, result, err := suite.baseApp.Simulate(txBytes)
		require.NoError(t, err)
		require.NotNil(t, result)
		require.Equal(t, gasConsumed, gInfo.GasUsed)

		// simulate again, same result
		gInfo, result, err = suite.baseApp.Simulate(txBytes)
		require.NoError(t, err)
		require.NotNil(t, result)
		require.Equal(t, gasConsumed, gInfo.GasUsed)

		// simulate by calling Query with encoded tx
		query := abci.RequestQuery{
			Path: "/app/simulate",
			Data: txBytes,
		}
		queryResult := suite.baseApp.Query(query)
		require.True(t, queryResult.IsOK(), queryResult.Log)

		var simRes sdk.SimulationResponse
		require.NoError(t, jsonpb.Unmarshal(strings.NewReader(string(queryResult.Value)), &simRes))

		require.Equal(t, gInfo, simRes.GasInfo)
		require.Equal(t, result.Log, simRes.Result.Log)
		require.Equal(t, result.Events, simRes.Result.Events)
		require.True(t, bytes.Equal(result.Data, simRes.Result.Data))

		suite.baseApp.EndBlock(abci.RequestEndBlock{})
		suite.baseApp.Commit()
	}
}

func TestABCI_InvalidTransaction(t *testing.T) {
	anteOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetAnteHandler(func(ctx sdk.Context, tx sdk.Tx, simulate bool) (newCtx sdk.Context, err error) {
			return
		})
	}

	suite := NewBaseAppSuite(t, anteOpt)
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), CounterServerImplGasMeterOnly{})

	suite.baseApp.InitChain(abci.RequestInitChain{
		ConsensusParams: &tmproto.ConsensusParams{},
	})

	header := tmproto.Header{Height: 1}
	suite.baseApp.BeginBlock(abci.RequestBeginBlock{Header: header})

	// transaction with no messages
	{
		emptyTx := suite.txConfig.NewTxBuilder().GetTx()
		_, result, err := suite.baseApp.SimDeliver(suite.txConfig.TxEncoder(), emptyTx)
		require.Error(t, err)
		require.Nil(t, result)

		space, code, _ := sdkerrors.ABCIInfo(err, false)
		require.EqualValues(t, sdkerrors.ErrInvalidRequest.Codespace(), space, err)
		require.EqualValues(t, sdkerrors.ErrInvalidRequest.ABCICode(), code, err)
	}

	// transaction where ValidateBasic fails
	{
		testCases := []struct {
			tx   signing.Tx
			fail bool
		}{
			{newTxCounter(t, suite.txConfig, 0, 0), false},
			{newTxCounter(t, suite.txConfig, -1, 0), false},
			{newTxCounter(t, suite.txConfig, 100, 100), false},
			{newTxCounter(t, suite.txConfig, 100, 5, 4, 3, 2, 1), false},

			{newTxCounter(t, suite.txConfig, 0, -1), true},
			{newTxCounter(t, suite.txConfig, 0, 1, -2), true},
			{newTxCounter(t, suite.txConfig, 0, 1, 2, -10, 5), true},
		}

		for _, testCase := range testCases {
			tx := testCase.tx
			_, result, err := suite.baseApp.SimDeliver(suite.txConfig.TxEncoder(), tx)

			if testCase.fail {
				require.Error(t, err)

				space, code, _ := sdkerrors.ABCIInfo(err, false)
				require.EqualValues(t, sdkerrors.ErrInvalidSequence.Codespace(), space, err)
				require.EqualValues(t, sdkerrors.ErrInvalidSequence.ABCICode(), code, err)
			} else {
				require.NotNil(t, result)
			}
		}
	}

	// transaction with no known route
	{
		txBuilder := suite.txConfig.NewTxBuilder()
		txBuilder.SetMsgs(&baseapptestutil.MsgCounter2{})
		setTxSignature(t, txBuilder, 0)
		unknownRouteTx := txBuilder.GetTx()

		_, result, err := suite.baseApp.SimDeliver(suite.txConfig.TxEncoder(), unknownRouteTx)
		require.Error(t, err)
		require.Nil(t, result)

		space, code, _ := sdkerrors.ABCIInfo(err, false)
		require.EqualValues(t, sdkerrors.ErrUnknownRequest.Codespace(), space, err)
		require.EqualValues(t, sdkerrors.ErrUnknownRequest.ABCICode(), code, err)

		txBuilder = suite.txConfig.NewTxBuilder()
		txBuilder.SetMsgs(&baseapptestutil.MsgCounter{}, &baseapptestutil.MsgCounter2{})
		setTxSignature(t, txBuilder, 0)
		unknownRouteTx = txBuilder.GetTx()

		_, result, err = suite.baseApp.SimDeliver(suite.txConfig.TxEncoder(), unknownRouteTx)
		require.Error(t, err)
		require.Nil(t, result)

		space, code, _ = sdkerrors.ABCIInfo(err, false)
		require.EqualValues(t, sdkerrors.ErrUnknownRequest.Codespace(), space, err)
		require.EqualValues(t, sdkerrors.ErrUnknownRequest.ABCICode(), code, err)
	}

	// Transaction with an unregistered message
	{
		txBuilder := suite.txConfig.NewTxBuilder()
		txBuilder.SetMsgs(&testdata.MsgCreateDog{})
		tx := txBuilder.GetTx()

		txBytes, err := suite.txConfig.TxEncoder()(tx)
		require.NoError(t, err)

		res := suite.baseApp.DeliverTx(abci.RequestDeliverTx{Tx: txBytes})
		require.EqualValues(t, sdkerrors.ErrTxDecode.ABCICode(), res.Code)
		require.EqualValues(t, sdkerrors.ErrTxDecode.Codespace(), res.Codespace)
	}
}

func TestABCI_TxGasLimits(t *testing.T) {
	gasGranted := uint64(10)
	anteOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetAnteHandler(func(ctx sdk.Context, tx sdk.Tx, simulate bool) (newCtx sdk.Context, err error) {
			newCtx = ctx.WithGasMeter(sdk.NewGasMeter(gasGranted))

			// AnteHandlers must have their own defer/recover in order for the BaseApp
			// to know how much gas was used! This is because the GasMeter is created in
			// the AnteHandler, but if it panics the context won't be set properly in
			// runTx's recover call.
			defer func() {
				if r := recover(); r != nil {
					switch rType := r.(type) {
					case sdk.ErrorOutOfGas:
						err = sdkerrors.Wrapf(sdkerrors.ErrOutOfGas, "out of gas in location: %v", rType.Descriptor)
					default:
						panic(r)
					}
				}
			}()

			count, _ := parseTxMemo(t, tx)
			newCtx.GasMeter().ConsumeGas(uint64(count), "counter-ante")

			return newCtx, nil
		})
	}

	suite := NewBaseAppSuite(t, anteOpt)
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), CounterServerImplGasMeterOnly{})

	suite.baseApp.InitChain(abci.RequestInitChain{
		ConsensusParams: &tmproto.ConsensusParams{},
	})

	header := tmproto.Header{Height: 1}
	suite.baseApp.BeginBlock(abci.RequestBeginBlock{Header: header})

	testCases := []struct {
		tx      signing.Tx
		gasUsed uint64
		fail    bool
	}{
		{newTxCounter(t, suite.txConfig, 0, 0), 0, false},
		{newTxCounter(t, suite.txConfig, 1, 1), 2, false},
		{newTxCounter(t, suite.txConfig, 9, 1), 10, false},
		{newTxCounter(t, suite.txConfig, 1, 9), 10, false},
		{newTxCounter(t, suite.txConfig, 10, 0), 10, false},
		{newTxCounter(t, suite.txConfig, 0, 10), 10, false},
		{newTxCounter(t, suite.txConfig, 0, 8, 2), 10, false},
		{newTxCounter(t, suite.txConfig, 0, 5, 1, 1, 1, 1, 1), 10, false},
		{newTxCounter(t, suite.txConfig, 0, 5, 1, 1, 1, 1), 9, false},

		{newTxCounter(t, suite.txConfig, 9, 2), 11, true},
		{newTxCounter(t, suite.txConfig, 2, 9), 11, true},
		{newTxCounter(t, suite.txConfig, 9, 1, 1), 11, true},
		{newTxCounter(t, suite.txConfig, 1, 8, 1, 1), 11, true},
		{newTxCounter(t, suite.txConfig, 11, 0), 11, true},
		{newTxCounter(t, suite.txConfig, 0, 11), 11, true},
		{newTxCounter(t, suite.txConfig, 0, 5, 11), 16, true},
	}

	for i, tc := range testCases {
		tx := tc.tx
		gInfo, result, err := suite.baseApp.SimDeliver(suite.txConfig.TxEncoder(), tx)

		// check gas used and wanted
		require.Equal(t, tc.gasUsed, gInfo.GasUsed, fmt.Sprintf("tc #%d; gas: %v, result: %v, err: %s", i, gInfo, result, err))

		// check for out of gas
		if !tc.fail {
			require.NotNil(t, result, fmt.Sprintf("%d: %v, %v", i, tc, err))
		} else {
			require.Error(t, err)
			require.Nil(t, result)

			space, code, _ := sdkerrors.ABCIInfo(err, false)
			require.EqualValues(t, sdkerrors.ErrOutOfGas.Codespace(), space, err)
			require.EqualValues(t, sdkerrors.ErrOutOfGas.ABCICode(), code, err)
		}
	}
}

func TestABCI_MaxBlockGasLimits(t *testing.T) {
	gasGranted := uint64(10)
	anteOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetAnteHandler(func(ctx sdk.Context, tx sdk.Tx, simulate bool) (newCtx sdk.Context, err error) {
			newCtx = ctx.WithGasMeter(sdk.NewGasMeter(gasGranted))

			defer func() {
				if r := recover(); r != nil {
					switch rType := r.(type) {
					case sdk.ErrorOutOfGas:
						err = sdkerrors.Wrapf(sdkerrors.ErrOutOfGas, "out of gas in location: %v", rType.Descriptor)
					default:
						panic(r)
					}
				}
			}()

			count, _ := parseTxMemo(t, tx)
			newCtx.GasMeter().ConsumeGas(uint64(count), "counter-ante")

			return
		})
	}

	suite := NewBaseAppSuite(t, anteOpt)
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), CounterServerImplGasMeterOnly{})

	suite.baseApp.InitChain(abci.RequestInitChain{
		ConsensusParams: &tmproto.ConsensusParams{
			Block: &tmproto.BlockParams{
				MaxGas: 100,
			},
		},
	})

	header := tmproto.Header{Height: 1}
	suite.baseApp.BeginBlock(abci.RequestBeginBlock{Header: header})

	testCases := []struct {
		tx                signing.Tx
		numDelivers       int
		gasUsedPerDeliver uint64
		fail              bool
		failAfterDeliver  int
	}{
		{newTxCounter(t, suite.txConfig, 0, 0), 0, 0, false, 0},
		{newTxCounter(t, suite.txConfig, 9, 1), 2, 10, false, 0},
		{newTxCounter(t, suite.txConfig, 10, 0), 3, 10, false, 0},
		{newTxCounter(t, suite.txConfig, 10, 0), 10, 10, false, 0},
		{newTxCounter(t, suite.txConfig, 2, 7), 11, 9, false, 0},
		{newTxCounter(t, suite.txConfig, 10, 0), 10, 10, false, 0}, // hit the limit but pass

		{newTxCounter(t, suite.txConfig, 10, 0), 11, 10, true, 10},
		{newTxCounter(t, suite.txConfig, 10, 0), 15, 10, true, 10},
		{newTxCounter(t, suite.txConfig, 9, 0), 12, 9, true, 11}, // fly past the limit
	}

	for i, tc := range testCases {
		tx := tc.tx

		// reset the block gas
		header := tmproto.Header{Height: suite.baseApp.LastBlockHeight() + 1}
		suite.baseApp.BeginBlock(abci.RequestBeginBlock{Header: header})

		// execute the transaction multiple times
		for j := 0; j < tc.numDelivers; j++ {
			_, result, err := suite.baseApp.SimDeliver(suite.txConfig.TxEncoder(), tx)

			ctx := getDeliverStateCtx(suite.baseApp)

			// check for failed transactions
			if tc.fail && (j+1) > tc.failAfterDeliver {
				require.Error(t, err, fmt.Sprintf("tc #%d; result: %v, err: %s", i, result, err))
				require.Nil(t, result, fmt.Sprintf("tc #%d; result: %v, err: %s", i, result, err))

				space, code, _ := sdkerrors.ABCIInfo(err, false)
				require.EqualValues(t, sdkerrors.ErrOutOfGas.Codespace(), space, err)
				require.EqualValues(t, sdkerrors.ErrOutOfGas.ABCICode(), code, err)
				require.True(t, ctx.BlockGasMeter().IsOutOfGas())
			} else {
				// check gas used and wanted
				blockGasUsed := ctx.BlockGasMeter().GasConsumed()
				expBlockGasUsed := tc.gasUsedPerDeliver * uint64(j+1)
				require.Equal(
					t, expBlockGasUsed, blockGasUsed,
					fmt.Sprintf("%d,%d: %v, %v, %v, %v", i, j, tc, expBlockGasUsed, blockGasUsed, result),
				)

				require.NotNil(t, result, fmt.Sprintf("tc #%d; currDeliver: %d, result: %v, err: %s", i, j, result, err))
				require.False(t, ctx.BlockGasMeter().IsPastLimit())
			}
		}
	}
}

func TestABCI_GasConsumptionBadTx(t *testing.T) {
	gasWanted := uint64(5)
	anteOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetAnteHandler(func(ctx sdk.Context, tx sdk.Tx, simulate bool) (newCtx sdk.Context, err error) {
			newCtx = ctx.WithGasMeter(sdk.NewGasMeter(gasWanted))

			defer func() {
				if r := recover(); r != nil {
					switch rType := r.(type) {
					case sdk.ErrorOutOfGas:
						log := fmt.Sprintf("out of gas in location: %v", rType.Descriptor)
						err = sdkerrors.Wrap(sdkerrors.ErrOutOfGas, log)
					default:
						panic(r)
					}
				}
			}()

			counter, failOnAnte := parseTxMemo(t, tx)
			newCtx.GasMeter().ConsumeGas(uint64(counter), "counter-ante")
			if failOnAnte {
				return newCtx, sdkerrors.Wrap(sdkerrors.ErrUnauthorized, "ante handler failure")
			}

			return
		})
	}

	suite := NewBaseAppSuite(t, anteOpt)
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), CounterServerImplGasMeterOnly{})

	suite.baseApp.InitChain(abci.RequestInitChain{
		ConsensusParams: &tmproto.ConsensusParams{
			Block: &tmproto.BlockParams{
				MaxGas: 9,
			},
		},
	})

	header := tmproto.Header{Height: suite.baseApp.LastBlockHeight() + 1}
	suite.baseApp.BeginBlock(abci.RequestBeginBlock{Header: header})

	tx := newTxCounter(t, suite.txConfig, 5, 0)
	tx = setFailOnAnte(t, suite.txConfig, tx, true)
	txBytes, err := suite.txConfig.TxEncoder()(tx)
	require.NoError(t, err)

	res := suite.baseApp.DeliverTx(abci.RequestDeliverTx{Tx: txBytes})
	require.False(t, res.IsOK(), fmt.Sprintf("%v", res))

	// require next tx to fail due to black gas limit
	tx = newTxCounter(t, suite.txConfig, 5, 0)
	txBytes, err = suite.txConfig.TxEncoder()(tx)
	require.NoError(t, err)

	res = suite.baseApp.DeliverTx(abci.RequestDeliverTx{Tx: txBytes})
	require.False(t, res.IsOK(), fmt.Sprintf("%v", res))
}

func TestABCI_Query(t *testing.T) {
	key, value := []byte("hello"), []byte("goodbye")
	anteOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetAnteHandler(func(ctx sdk.Context, tx sdk.Tx, simulate bool) (newCtx sdk.Context, err error) {
			store := ctx.KVStore(capKey1)
			store.Set(key, value)
			return
		})
	}

	suite := NewBaseAppSuite(t, anteOpt)
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), CounterServerImplGasMeterOnly{})

	suite.baseApp.InitChain(abci.RequestInitChain{
		ConsensusParams: &tmproto.ConsensusParams{},
	})

	// NOTE: "/store/key1" tells us KVStore
	// and the final "/key" says to use the data as the
	// key in the given KVStore ...
	query := abci.RequestQuery{
		Path: "/store/key1/key",
		Data: key,
	}
	tx := newTxCounter(t, suite.txConfig, 0, 0)

	// query is empty before we do anything
	res := suite.baseApp.Query(query)
	require.Equal(t, 0, len(res.Value))

	// query is still empty after a CheckTx
	_, resTx, err := suite.baseApp.SimCheck(suite.txConfig.TxEncoder(), tx)
	require.NoError(t, err)
	require.NotNil(t, resTx)

	res = suite.baseApp.Query(query)
	require.Equal(t, 0, len(res.Value))

	// query is still empty after a DeliverTx before we commit
	header := tmproto.Header{Height: suite.baseApp.LastBlockHeight() + 1}
	suite.baseApp.BeginBlock(abci.RequestBeginBlock{Header: header})

	_, resTx, err = suite.baseApp.SimDeliver(suite.txConfig.TxEncoder(), tx)
	require.NoError(t, err)
	require.NotNil(t, resTx)

	res = suite.baseApp.Query(query)
	require.Equal(t, 0, len(res.Value))

	// query returns correct value after Commit
	suite.baseApp.Commit()

	res = suite.baseApp.Query(query)
	require.Equal(t, value, res.Value)
}

func TestABCI_GetBlockRetentionHeight(t *testing.T) {
	logger := defaultLogger()
	db := dbm.NewMemDB()
	name := t.Name()

	snapshotStore, err := snapshots.NewStore(dbm.NewMemDB(), testutil.GetTempDir(t))
	require.NoError(t, err)

	testCases := map[string]struct {
		bapp         *baseapp.BaseApp
		maxAgeBlocks int64
		commitHeight int64
		expected     int64
	}{
		"defaults": {
			bapp:         baseapp.NewBaseApp(name, logger, db, nil),
			maxAgeBlocks: 0,
			commitHeight: 499000,
			expected:     0,
		},
		"pruning unbonding time only": {
			bapp:         baseapp.NewBaseApp(name, logger, db, nil, baseapp.SetMinRetainBlocks(1)),
			maxAgeBlocks: 362880,
			commitHeight: 499000,
			expected:     136120,
		},
		"pruning iavl snapshot only": {
			bapp: baseapp.NewBaseApp(
				name, logger, db, nil,
				baseapp.SetPruning(pruningtypes.NewPruningOptions(pruningtypes.PruningNothing)),
				baseapp.SetMinRetainBlocks(1),
				baseapp.SetSnapshot(snapshotStore, snapshottypes.NewSnapshotOptions(10000, 1)),
			),
			maxAgeBlocks: 0,
			commitHeight: 499000,
			expected:     489000,
		},
		"pruning state sync snapshot only": {
			bapp: baseapp.NewBaseApp(
				name, logger, db, nil,
				baseapp.SetSnapshot(snapshotStore, snapshottypes.NewSnapshotOptions(50000, 3)),
				baseapp.SetMinRetainBlocks(1),
			),
			maxAgeBlocks: 0,
			commitHeight: 499000,
			expected:     349000,
		},
		"pruning min retention only": {
			bapp: baseapp.NewBaseApp(
				name, logger, db, nil,
				baseapp.SetMinRetainBlocks(400000),
			),
			maxAgeBlocks: 0,
			commitHeight: 499000,
			expected:     99000,
		},
		"pruning all conditions": {
			bapp: baseapp.NewBaseApp(
				name, logger, db, nil,
				baseapp.SetPruning(pruningtypes.NewCustomPruningOptions(0, 0)),
				baseapp.SetMinRetainBlocks(400000),
				baseapp.SetSnapshot(snapshotStore, snapshottypes.NewSnapshotOptions(50000, 3)),
			),
			maxAgeBlocks: 362880,
			commitHeight: 499000,
			expected:     99000,
		},
		"no pruning due to no persisted state": {
			bapp: baseapp.NewBaseApp(
				name, logger, db, nil,
				baseapp.SetPruning(pruningtypes.NewCustomPruningOptions(0, 0)),
				baseapp.SetMinRetainBlocks(400000),
				baseapp.SetSnapshot(snapshotStore, snapshottypes.NewSnapshotOptions(50000, 3)),
			),
			maxAgeBlocks: 362880,
			commitHeight: 10000,
			expected:     0,
		},
		"disable pruning": {
			bapp: baseapp.NewBaseApp(
				name, logger, db, nil,
				baseapp.SetPruning(pruningtypes.NewCustomPruningOptions(0, 0)),
				baseapp.SetMinRetainBlocks(0),
				baseapp.SetSnapshot(snapshotStore, snapshottypes.NewSnapshotOptions(50000, 3)),
			),
			maxAgeBlocks: 362880,
			commitHeight: 499000,
			expected:     0,
		},
	}

	for name, tc := range testCases {
		tc := tc

		tc.bapp.SetParamStore(&paramStore{db: dbm.NewMemDB()})
		tc.bapp.InitChain(abci.RequestInitChain{
			ConsensusParams: &tmproto.ConsensusParams{
				Evidence: &tmproto.EvidenceParams{
					MaxAgeNumBlocks: tc.maxAgeBlocks,
				},
			},
		})

		t.Run(name, func(t *testing.T) {
			require.Equal(t, tc.expected, tc.bapp.GetBlockRetentionHeight(tc.commitHeight))
		})
	}
}

func TestABCI_Proposal_HappyPath(t *testing.T) {
	anteKey := []byte("ante-key")
	pool := mempool.NewSenderNonceMempool()
	anteOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetAnteHandler(anteHandlerTxTest(t, capKey1, anteKey))
	}

	suite := NewBaseAppSuite(t, anteOpt, baseapp.SetMempool(pool))
	baseapptestutil.RegisterKeyValueServer(suite.baseApp.MsgServiceRouter(), MsgKeyValueImpl{})
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), NoopCounterServerImpl{})

	suite.baseApp.InitChain(abci.RequestInitChain{
		ConsensusParams: &tmproto.ConsensusParams{},
	})

	tx := newTxCounter(t, suite.txConfig, 0, 1)
	txBytes, err := suite.txConfig.TxEncoder()(tx)
	require.NoError(t, err)

	reqCheckTx := abci.RequestCheckTx{
		Tx:   txBytes,
		Type: abci.CheckTxType_New,
	}
	suite.baseApp.CheckTx(reqCheckTx)

	tx2 := newTxCounter(t, suite.txConfig, 1, 1)

	tx2Bytes, err := suite.txConfig.TxEncoder()(tx2)
	require.NoError(t, err)

	err = pool.Insert(sdk.Context{}, tx2)
	require.NoError(t, err)

	reqPrepareProposal := abci.RequestPrepareProposal{
		MaxTxBytes: 1000,
		Height:     1,
	}
	resPrepareProposal := suite.baseApp.PrepareProposal(reqPrepareProposal)
	require.Equal(t, 2, len(resPrepareProposal.Txs))

	reqProposalTxBytes := [2][]byte{
		txBytes,
		tx2Bytes,
	}
	reqProcessProposal := abci.RequestProcessProposal{
		Txs:    reqProposalTxBytes[:],
		Height: reqPrepareProposal.Height,
	}

	resProcessProposal := suite.baseApp.ProcessProposal(reqProcessProposal)
	require.Equal(t, abci.ResponseProcessProposal_ACCEPT, resProcessProposal.Status)

	suite.baseApp.BeginBlock(abci.RequestBeginBlock{
		Header: tmproto.Header{Height: suite.baseApp.LastBlockHeight() + 1},
	})

	res := suite.baseApp.DeliverTx(abci.RequestDeliverTx{Tx: txBytes})
	require.Equal(t, 1, pool.CountTx())

	require.NotEmpty(t, res.Events)
	require.True(t, res.IsOK(), fmt.Sprintf("%v", res))
}

func TestABCI_Proposal_Read_State_PrepareProposal(t *testing.T) {
	someKey := []byte("some-key")

	setInitChainerOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetInitChainer(func(ctx sdk.Context, req abci.RequestInitChain) abci.ResponseInitChain {
			ctx.KVStore(capKey1).Set(someKey, []byte("foo"))
			return abci.ResponseInitChain{}
		})
	}

	prepareOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetPrepareProposal(func(ctx sdk.Context, req abci.RequestPrepareProposal) abci.ResponsePrepareProposal {
			value := ctx.KVStore(capKey1).Get(someKey)
			// We should be able to access any state written in InitChain
			require.Equal(t, "foo", string(value))
			return abci.ResponsePrepareProposal{Txs: req.Txs}
		})
	}

	suite := NewBaseAppSuite(t, setInitChainerOpt, prepareOpt)

	suite.baseApp.InitChain(abci.RequestInitChain{
		InitialHeight:   1,
		ConsensusParams: &tmproto.ConsensusParams{},
	})

	reqPrepareProposal := abci.RequestPrepareProposal{
		MaxTxBytes: 1000,
		Height:     1, // this value can't be 0
	}
	resPrepareProposal := suite.baseApp.PrepareProposal(reqPrepareProposal)
	require.Equal(t, 0, len(resPrepareProposal.Txs))

	reqProposalTxBytes := [][]byte{}
	reqProcessProposal := abci.RequestProcessProposal{
		Txs:    reqProposalTxBytes,
		Height: reqPrepareProposal.Height,
	}

	resProcessProposal := suite.baseApp.ProcessProposal(reqProcessProposal)
	require.Equal(t, abci.ResponseProcessProposal_ACCEPT, resProcessProposal.Status)

	suite.baseApp.BeginBlock(abci.RequestBeginBlock{
		Header: tmproto.Header{Height: suite.baseApp.LastBlockHeight() + 1},
	})
}

func TestABCI_PrepareProposal_ReachedMaxBytes(t *testing.T) {
	anteKey := []byte("ante-key")
	pool := mempool.NewSenderNonceMempool()
	anteOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetAnteHandler(anteHandlerTxTest(t, capKey1, anteKey))
	}
	suite := NewBaseAppSuite(t, anteOpt, baseapp.SetMempool(pool))

	suite.baseApp.InitChain(abci.RequestInitChain{
		ConsensusParams: &tmproto.ConsensusParams{},
	})

	for i := 0; i < 100; i++ {
		tx2 := newTxCounter(t, suite.txConfig, int64(i), int64(i))
		err := pool.Insert(sdk.Context{}, tx2)
		require.NoError(t, err)
	}

	reqPrepareProposal := abci.RequestPrepareProposal{
		MaxTxBytes: 1500,
		Height:     1,
	}
	resPrepareProposal := suite.baseApp.PrepareProposal(reqPrepareProposal)
	require.Equal(t, 11, len(resPrepareProposal.Txs))
}

func TestABCI_PrepareProposal_BadEncoding(t *testing.T) {
	anteKey := []byte("ante-key")
	pool := mempool.NewSenderNonceMempool()
	anteOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetAnteHandler(anteHandlerTxTest(t, capKey1, anteKey))
	}
	suite := NewBaseAppSuite(t, anteOpt, baseapp.SetMempool(pool))

	suite.baseApp.InitChain(abci.RequestInitChain{
		ConsensusParams: &tmproto.ConsensusParams{},
	})

	tx := newTxCounter(t, suite.txConfig, 0, 0)
	err := pool.Insert(sdk.Context{}, tx)
	require.NoError(t, err)

	reqPrepareProposal := abci.RequestPrepareProposal{
		MaxTxBytes: 1000,
		Height:     1,
	}
	resPrepareProposal := suite.baseApp.PrepareProposal(reqPrepareProposal)
	require.Equal(t, 1, len(resPrepareProposal.Txs))
}

func TestABCI_PrepareProposal_OverGasUnderBytes(t *testing.T) {
	pool := mempool.NewSenderNonceMempool()
	suite := NewBaseAppSuite(t, baseapp.SetMempool(pool))
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), NoopCounterServerImpl{})

	// set max block gas limit to 99, this will allow 9 txs of 10 gas each.
	suite.baseApp.InitChain(abci.RequestInitChain{
		ConsensusParams: &tmproto.ConsensusParams{
			Block: &tmproto.BlockParams{MaxGas: 99},
		},
	})

	// insert 100 txs, each with a gas limit of 10
	for i := int64(0); i < 100; i++ {
		msg := &baseapptestutil.MsgCounter{Counter: i, FailOnHandler: false}
		msgs := []sdk.Msg{msg}

		builder := suite.txConfig.NewTxBuilder()
		err := builder.SetMsgs(msgs...)
		require.NoError(t, err)
		builder.SetMemo("counter=" + strconv.FormatInt(i, 10) + "&failOnAnte=false")
		builder.SetGasLimit(10)
		setTxSignature(t, builder, uint64(i))

		err = pool.Insert(sdk.Context{}, builder.GetTx())
		require.NoError(t, err)
	}

	// ensure we only select transactions that fit within the block gas limit
	res := suite.baseApp.PrepareProposal(abci.RequestPrepareProposal{
		MaxTxBytes: 1_000_000, // large enough to ignore restriction
		Height:     1,
	})

	// Should include 9 transactions
	require.Len(t, res.Txs, 9, "invalid number of transactions returned")
}

func TestABCI_PrepareProposal_MaxGas(t *testing.T) {
	pool := mempool.NewSenderNonceMempool()
	suite := NewBaseAppSuite(t, baseapp.SetMempool(pool))
	baseapptestutil.RegisterCounterServer(suite.baseApp.MsgServiceRouter(), NoopCounterServerImpl{})

	// set max block gas limit to 100
	suite.baseApp.InitChain(abci.RequestInitChain{
		ConsensusParams: &tmproto.ConsensusParams{
			Block: &tmproto.BlockParams{MaxGas: 100},
		},
	})

	// insert 100 txs, each with a gas limit of 10
	for i := int64(0); i < 100; i++ {
		msg := &baseapptestutil.MsgCounter{Counter: i, FailOnHandler: false}
		msgs := []sdk.Msg{msg}

		builder := suite.txConfig.NewTxBuilder()
		builder.SetMsgs(msgs...)
		builder.SetMemo("counter=" + strconv.FormatInt(i, 10) + "&failOnAnte=false")
		builder.SetGasLimit(10)
		setTxSignature(t, builder, uint64(i))

		err := pool.Insert(sdk.Context{}, builder.GetTx())
		require.NoError(t, err)
	}

	// ensure we only select transactions that fit within the block gas limit
	res := suite.baseApp.PrepareProposal(abci.RequestPrepareProposal{
		MaxTxBytes: 1_000_000, // large enough to ignore restriction
		Height:     1,
	})
	require.Len(t, res.Txs, 10, "invalid number of transactions returned")
}

func TestABCI_PrepareProposal_Failures(t *testing.T) {
	anteKey := []byte("ante-key")
	pool := mempool.NewSenderNonceMempool()
	anteOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetAnteHandler(anteHandlerTxTest(t, capKey1, anteKey))
	}
	suite := NewBaseAppSuite(t, anteOpt, baseapp.SetMempool(pool))

	suite.baseApp.InitChain(abci.RequestInitChain{
		ConsensusParams: &tmproto.ConsensusParams{},
	})

	tx := newTxCounter(t, suite.txConfig, 0, 0)
	txBytes, err := suite.txConfig.TxEncoder()(tx)
	require.NoError(t, err)

	reqCheckTx := abci.RequestCheckTx{
		Tx:   txBytes,
		Type: abci.CheckTxType_New,
	}
	checkTxRes := suite.baseApp.CheckTx(reqCheckTx)
	require.True(t, checkTxRes.IsOK())

	failTx := newTxCounter(t, suite.txConfig, 1, 1)
	failTx = setFailOnAnte(t, suite.txConfig, failTx, true)

	err = pool.Insert(sdk.Context{}, failTx)
	require.NoError(t, err)
	require.Equal(t, 2, pool.CountTx())

	req := abci.RequestPrepareProposal{
		MaxTxBytes: 1000,
		Height:     1,
	}
	res := suite.baseApp.PrepareProposal(req)
	require.Equal(t, 1, len(res.Txs))
}

func TestABCI_PrepareProposal_PanicRecovery(t *testing.T) {
	prepareOpt := func(app *baseapp.BaseApp) {
		app.SetPrepareProposal(func(ctx sdk.Context, rpp abci.RequestPrepareProposal) abci.ResponsePrepareProposal {
			panic(errors.New("test"))
		})
	}
	suite := NewBaseAppSuite(t, prepareOpt)

	suite.baseApp.InitChain(abci.RequestInitChain{
		ConsensusParams: &tmproto.ConsensusParams{},
	})

	req := abci.RequestPrepareProposal{
		MaxTxBytes: 1000,
		Height:     1,
	}

	require.NotPanics(t, func() {
		res := suite.baseApp.PrepareProposal(req)
		require.Equal(t, req.Txs, res.Txs)
	})
}

func TestABCI_ProcessProposal_PanicRecovery(t *testing.T) {
	processOpt := func(app *baseapp.BaseApp) {
		app.SetProcessProposal(func(ctx sdk.Context, rpp abci.RequestProcessProposal) abci.ResponseProcessProposal {
			panic(errors.New("test"))
		})
	}
	suite := NewBaseAppSuite(t, processOpt)

	suite.baseApp.InitChain(abci.RequestInitChain{
		ConsensusParams: &tmproto.ConsensusParams{},
	})

	require.NotPanics(t, func() {
		res := suite.baseApp.ProcessProposal(abci.RequestProcessProposal{Height: 1})
		require.Equal(t, res.Status, abci.ResponseProcessProposal_REJECT)
	})
}

// TestABCI_Proposal_Reset_State ensures that state is reset between runs of
// PrepareProposal and ProcessProposal in case they are called multiple times.
// This is only valid for heights > 1, given that on height 1 we always set the
// state to be deliverState.
func TestABCI_Proposal_Reset_State_Between_Calls(t *testing.T) {
	someKey := []byte("some-key")

	prepareOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetPrepareProposal(func(ctx sdk.Context, req abci.RequestPrepareProposal) abci.ResponsePrepareProposal {
			// This key should not exist given that we reset the state on every call.
			require.False(t, ctx.KVStore(capKey1).Has(someKey))
			ctx.KVStore(capKey1).Set(someKey, someKey)
			return abci.ResponsePrepareProposal{Txs: req.Txs}
		})
	}

	processOpt := func(bapp *baseapp.BaseApp) {
		bapp.SetProcessProposal(func(ctx sdk.Context, req abci.RequestProcessProposal) abci.ResponseProcessProposal {
			// This key should not exist given that we reset the state on every call.
			require.False(t, ctx.KVStore(capKey1).Has(someKey))
			ctx.KVStore(capKey1).Set(someKey, someKey)
			return abci.ResponseProcessProposal{Status: abci.ResponseProcessProposal_ACCEPT}
		})
	}

	suite := NewBaseAppSuite(t, prepareOpt, processOpt)

	suite.baseApp.InitChain(abci.RequestInitChain{
		ConsensusParams: &tmproto.ConsensusParams{},
	})

	reqPrepareProposal := abci.RequestPrepareProposal{
		MaxTxBytes: 1000,
		Height:     2, // this value can't be 0
	}

	// Let's pretend something happened and PrepareProposal gets called many
	// times, this must be safe to do.
	for i := 0; i < 5; i++ {
		resPrepareProposal := suite.baseApp.PrepareProposal(reqPrepareProposal)
		require.Equal(t, 0, len(resPrepareProposal.Txs))
	}

	reqProposalTxBytes := [][]byte{}
	reqProcessProposal := abci.RequestProcessProposal{
		Txs:    reqProposalTxBytes,
		Height: 2,
	}

	// Let's pretend something happened and ProcessProposal gets called many
	// times, this must be safe to do.
	for i := 0; i < 5; i++ {
		resProcessProposal := suite.baseApp.ProcessProposal(reqProcessProposal)
		require.Equal(t, abci.ResponseProcessProposal_ACCEPT, resProcessProposal.Status)
	}

	suite.baseApp.BeginBlock(abci.RequestBeginBlock{
		Header: tmproto.Header{Height: suite.baseApp.LastBlockHeight() + 1},
	})
}

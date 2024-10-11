package store

import (
	"encoding/json"
	"fmt"
	"os"
	"runtime/debug"
	"strings"
	"testing"
	"time"

	"github.com/cosmos/gogoproto/proto"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	dbm "github.com/cometbft/cometbft-db"
	cmtstore "github.com/cometbft/cometbft/api/cometbft/store/v1"
	cmtversion "github.com/cometbft/cometbft/api/cometbft/version/v1"
	cfg "github.com/cometbft/cometbft/config"
	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/ed25519"
	cmtrand "github.com/cometbft/cometbft/internal/rand"
	"github.com/cometbft/cometbft/internal/test"
	"github.com/cometbft/cometbft/libs/log"
	sm "github.com/cometbft/cometbft/state"
	"github.com/cometbft/cometbft/state/indexer"
	"github.com/cometbft/cometbft/state/indexer/block"
	"github.com/cometbft/cometbft/state/txindex"
	"github.com/cometbft/cometbft/types"
	cmttime "github.com/cometbft/cometbft/types/time"
	"github.com/cometbft/cometbft/version"
)

// make an extended commit with a single vote containing just the height and a
// timestamp.
func makeTestExtCommit(height int64, timestamp time.Time) *types.ExtendedCommit {
	return makeTestExtCommitWithNumSigs(height, timestamp, 1)
}

func makeTestExtCommitWithNumSigs(height int64, timestamp time.Time, numSigs int) *types.ExtendedCommit {
	extCommitSigs := []types.ExtendedCommitSig{}
	for i := 0; i < numSigs; i++ {
		extCommitSigs = append(extCommitSigs, types.ExtendedCommitSig{
			CommitSig: types.CommitSig{
				BlockIDFlag:      types.BlockIDFlagCommit,
				ValidatorAddress: cmtrand.Bytes(crypto.AddressSize),
				Timestamp:        timestamp,
				Signature:        cmtrand.Bytes(64),
			},
			ExtensionSignature: []byte("ExtensionSignature"),
		})
	}
	return &types.ExtendedCommit{
		Height: height,
		BlockID: types.BlockID{
			Hash:          crypto.CRandBytes(32),
			PartSetHeader: types.PartSetHeader{Hash: crypto.CRandBytes(32), Total: 2},
		},
		ExtendedSignatures: extCommitSigs,
	}
}

func makeStateAndBlockStoreAndIndexers() (sm.State, *BlockStore, txindex.TxIndexer, indexer.BlockIndexer, func(), sm.Store) {
	config := test.ResetTestRoot("blockchain_reactor_test")
	blockDB := dbm.NewMemDB()
	stateDB := dbm.NewMemDB()
	stateStore := sm.NewStore(stateDB, sm.StoreOptions{
		DiscardABCIResponses: false,
	})
	state, err := sm.MakeGenesisStateFromFile(config.GenesisFile())
	if err != nil {
		panic(fmt.Errorf("error constructing state from genesis file: %w", err))
	}

	txIndexer, blockIndexer, _, err := block.IndexerFromConfig(config, cfg.DefaultDBProvider, "test")
	if err != nil {
		panic(err)
	}

	return state, NewBlockStore(blockDB), txIndexer, blockIndexer, func() { os.RemoveAll(config.RootDir) }, stateStore
}

func TestLoadBlockStoreState(t *testing.T) {
	type blockStoreTest struct {
		testName string
		bss      *cmtstore.BlockStoreState
		want     cmtstore.BlockStoreState
	}

	testCases := []blockStoreTest{
		{
			"success", &cmtstore.BlockStoreState{Base: 100, Height: 1000},
			cmtstore.BlockStoreState{Base: 100, Height: 1000},
		},
		{"empty", &cmtstore.BlockStoreState{}, cmtstore.BlockStoreState{}},
		{"no base", &cmtstore.BlockStoreState{Height: 1000}, cmtstore.BlockStoreState{Base: 1, Height: 1000}},
	}

	for _, tc := range testCases {
		db := dbm.NewMemDB()
		batch := db.NewBatch()
		SaveBlockStoreState(tc.bss, batch)
		err := batch.WriteSync()
		require.NoError(t, err)
		retrBSJ := LoadBlockStoreState(db)
		assert.Equal(t, tc.want, retrBSJ, "expected the retrieved DBs to match: %s", tc.testName)
		err = batch.Close()
		require.NoError(t, err)
	}
}

func TestNewBlockStore(t *testing.T) {
	db := dbm.NewMemDB()
	bss := cmtstore.BlockStoreState{Base: 100, Height: 10000}
	bz, _ := proto.Marshal(&bss)
	err := db.Set(blockStoreKey, bz)
	require.NoError(t, err)
	bs := NewBlockStore(db)
	require.Equal(t, int64(100), bs.Base(), "failed to properly parse blockstore")
	require.Equal(t, int64(10000), bs.Height(), "failed to properly parse blockstore")

	panicCausers := []struct {
		data    []byte
		wantErr string
	}{
		{[]byte("artful-doger"), "not unmarshal bytes"},
		{[]byte(" "), "unmarshal bytes"},
	}

	for i, tt := range panicCausers {
		// Expecting a panic here on trying to parse an invalid blockStore
		_, _, panicErr := doFn(func() (any, error) {
			err := db.Set(blockStoreKey, tt.data)
			require.NoError(t, err)
			_ = NewBlockStore(db)
			return nil, nil
		})
		require.Error(t, panicErr, "#%d panicCauser: %q expected a panic", i, tt.data)
		assert.Contains(t, fmt.Sprintf("%#v", panicErr), tt.wantErr, "#%d data: %q", i, tt.data)
	}

	err = db.Set(blockStoreKey, []byte{})
	require.NoError(t, err)
	bs = NewBlockStore(db)
	assert.Equal(t, int64(0), bs.Height(), "expecting empty bytes to be unmarshaled alright")
}

func newInMemoryBlockStore() (*BlockStore, dbm.DB) {
	db := dbm.NewMemDB()
	return NewBlockStore(db), db
}

// TODO: This test should be simplified ...

func TestBlockStoreSaveLoadBlock(t *testing.T) {
	state, bs, _, _, cleanup, _ := makeStateAndBlockStoreAndIndexers()
	defer cleanup()
	require.Equal(t, int64(0), bs.Base(), "initially the base should be zero")
	require.Equal(t, int64(0), bs.Height(), "initially the height should be zero")

	// check there are no blocks at various heights
	noBlockHeights := []int64{0, -1, 100, 1000, 2}
	for i, height := range noBlockHeights {
		if g, _ := bs.LoadBlock(height); g != nil {
			t.Errorf("#%d: height(%d) got a block; want nil", i, height)
		}
	}

	// save a block big enough to have two block parts
	txs := []types.Tx{make([]byte, types.BlockPartSizeBytes)} // TX taking one block part alone
	block := state.MakeBlock(bs.Height()+1, txs, new(types.Commit), nil, state.Validators.GetProposer().Address)
	validPartSet, err := block.MakePartSet(types.BlockPartSizeBytes)
	require.NoError(t, err)
	require.GreaterOrEqual(t, validPartSet.Total(), uint32(2))
	part2 := validPartSet.GetPart(1)

	seenCommit := makeTestExtCommit(block.Header.Height, cmttime.Now())
	bs.SaveBlockWithExtendedCommit(block, validPartSet, seenCommit)
	require.EqualValues(t, 1, bs.Base(), "expecting the new height to be changed")
	require.EqualValues(t, block.Header.Height, bs.Height(), "expecting the new height to be changed")

	incompletePartSet := types.NewPartSetFromHeader(types.PartSetHeader{Total: 2})
	uncontiguousPartSet := types.NewPartSetFromHeader(types.PartSetHeader{Total: 0})
	_, err = uncontiguousPartSet.AddPart(part2)
	require.Error(t, err)

	header1 := types.Header{
		Version:         cmtversion.Consensus{Block: version.BlockProtocol},
		Height:          1,
		ChainID:         "block_test",
		Time:            cmttime.Now(),
		ProposerAddress: cmtrand.Bytes(crypto.AddressSize),
	}

	// End of setup, test data

	commitAtH10 := makeTestExtCommit(10, cmttime.Now()).ToCommit()
	tuples := []struct {
		block      *types.Block
		parts      *types.PartSet
		seenCommit *types.ExtendedCommit
		wantPanic  string
		wantErr    bool

		corruptBlockInDB      bool
		corruptCommitInDB     bool
		corruptSeenCommitInDB bool
		eraseCommitInDB       bool
		eraseSeenCommitInDB   bool
	}{
		{
			block:      newBlock(header1, commitAtH10),
			parts:      validPartSet,
			seenCommit: seenCommit,
		},

		{
			block:     nil,
			wantPanic: "only save a non-nil block",
		},

		{
			block: newBlock( // New block at height 5 in empty block store is fine
				types.Header{
					Version:         cmtversion.Consensus{Block: version.BlockProtocol},
					Height:          5,
					ChainID:         "block_test",
					Time:            cmttime.Now(),
					ProposerAddress: cmtrand.Bytes(crypto.AddressSize),
				},
				makeTestExtCommit(5, cmttime.Now()).ToCommit(),
			),
			parts:      validPartSet,
			seenCommit: makeTestExtCommit(5, cmttime.Now()),
		},

		{
			block:      newBlock(header1, commitAtH10),
			parts:      incompletePartSet,
			wantPanic:  "only save complete block", // incomplete parts
			seenCommit: makeTestExtCommit(10, cmttime.Now()),
		},

		{
			block:             newBlock(header1, commitAtH10),
			parts:             validPartSet,
			seenCommit:        seenCommit,
			corruptCommitInDB: true, // Corrupt the DB's commit entry
			wantPanic:         "error reading block commit",
		},

		{
			block:            newBlock(header1, commitAtH10),
			parts:            validPartSet,
			seenCommit:       seenCommit,
			wantPanic:        "unmarshal to cmtproto.BlockMeta",
			corruptBlockInDB: true, // Corrupt the DB's block entry
		},

		{
			block:      newBlock(header1, commitAtH10),
			parts:      validPartSet,
			seenCommit: seenCommit,

			// Expecting no error and we want a nil back
			eraseSeenCommitInDB: true,
		},

		{
			block:      block,
			parts:      validPartSet,
			seenCommit: seenCommit,

			corruptSeenCommitInDB: true,
			wantPanic:             "error reading block seen commit",
		},

		{
			block:      block,
			parts:      validPartSet,
			seenCommit: seenCommit,

			// Expecting no error and we want a nil back
			eraseCommitInDB: true,
		},
	}

	type quad struct {
		block  *types.Block
		commit *types.Commit
		meta   *types.BlockMeta

		seenCommit *types.Commit
	}

	for i, tuple := range tuples {
		bs, db := newInMemoryBlockStore()
		// SaveBlock
		res, err, panicErr := doFn(func() (any, error) {
			bs.SaveBlockWithExtendedCommit(tuple.block, tuple.parts, tuple.seenCommit)
			if tuple.block == nil {
				return nil, nil
			}

			if tuple.corruptBlockInDB {
				err := db.Set(bs.dbKeyLayout.CalcBlockMetaKey(tuple.block.Height), []byte("block-bogus"))
				require.NoError(t, err)
			}
			bBlock, bBlockMeta := bs.LoadBlock(tuple.block.Height)

			if tuple.eraseSeenCommitInDB {
				err := db.Delete(bs.dbKeyLayout.CalcSeenCommitKey(tuple.block.Height))
				require.NoError(t, err)
			}
			if tuple.corruptSeenCommitInDB {
				err := db.Set(bs.dbKeyLayout.CalcSeenCommitKey(tuple.block.Height), []byte("bogus-seen-commit"))
				require.NoError(t, err)
			}
			bSeenCommit := bs.LoadSeenCommit(tuple.block.Height)

			commitHeight := tuple.block.Height - 1
			if tuple.eraseCommitInDB {
				err := db.Delete(bs.dbKeyLayout.CalcBlockCommitKey(commitHeight))
				require.NoError(t, err)
			}
			if tuple.corruptCommitInDB {
				err := db.Set(bs.dbKeyLayout.CalcBlockCommitKey(commitHeight), []byte("foo-bogus"))
				require.NoError(t, err)
			}
			bCommit := bs.LoadBlockCommit(commitHeight)
			return &quad{
				block: bBlock, seenCommit: bSeenCommit, commit: bCommit,
				meta: bBlockMeta,
			}, nil
		})

		if subStr := tuple.wantPanic; subStr != "" {
			if panicErr == nil {
				t.Errorf("#%d: want a non-nil panic", i)
			} else if got := fmt.Sprintf("%#v", panicErr); !strings.Contains(got, subStr) {
				t.Errorf("#%d:\n\tgotErr: %q\nwant substring: %q", i, got, subStr)
			}
			continue
		}

		if tuple.wantErr {
			if err == nil {
				t.Errorf("#%d: got nil error", i)
			}
			continue
		}

		require.NoError(t, panicErr, "#%d: unexpected panic", i)
		require.NoError(t, err, "#%d: expecting a non-nil error", i)
		qua, ok := res.(*quad)
		if !ok || qua == nil {
			t.Errorf("#%d: got nil quad back; gotType=%T", i, res)
			continue
		}
		if tuple.eraseSeenCommitInDB {
			assert.Nil(t, qua.seenCommit,
				"erased the seenCommit in the DB hence we should get back a nil seenCommit")
		}
		if tuple.eraseCommitInDB {
			assert.Nil(t, qua.commit,
				"erased the commit in the DB hence we should get back a nil commit")
		}
	}
}

// stripExtensions removes all VoteExtension data from an ExtendedCommit. This
// is useful when dealing with an ExendedCommit but vote extension data is
// expected to be absent.
func stripExtensions(ec *types.ExtendedCommit) bool {
	stripped := false
	for idx := range ec.ExtendedSignatures {
		if len(ec.ExtendedSignatures[idx].Extension) > 0 || len(ec.ExtendedSignatures[idx].ExtensionSignature) > 0 {
			stripped = true
		}
		ec.ExtendedSignatures[idx].Extension = nil
		ec.ExtendedSignatures[idx].ExtensionSignature = nil
	}
	return stripped
}

// TestSaveBlockWithExtendedCommitPanicOnAbsentExtension tests that saving a
// block with an extended commit panics when the extension data is absent.
func TestSaveBlockWithExtendedCommitPanicOnAbsentExtension(t *testing.T) {
	for _, testCase := range []struct {
		name           string
		malleateCommit func(*types.ExtendedCommit)
		shouldPanic    bool
	}{
		{
			name:           "basic save",
			malleateCommit: func(_ *types.ExtendedCommit) {},
			shouldPanic:    false,
		},
		{
			name: "save commit with no extensions",
			malleateCommit: func(c *types.ExtendedCommit) {
				stripExtensions(c)
			},
			shouldPanic: true,
		},
	} {
		t.Run(testCase.name, func(t *testing.T) {
			state, bs, _, _, cleanup, _ := makeStateAndBlockStoreAndIndexers()
			defer cleanup()
			h := bs.Height() + 1
			block := state.MakeBlock(h, test.MakeNTxs(h, 10), new(types.Commit), nil, state.Validators.GetProposer().Address)

			seenCommit := makeTestExtCommit(block.Header.Height, cmttime.Now())
			ps, err := block.MakePartSet(types.BlockPartSizeBytes)
			require.NoError(t, err)
			testCase.malleateCommit(seenCommit)
			if testCase.shouldPanic {
				require.Panics(t, func() {
					bs.SaveBlockWithExtendedCommit(block, ps, seenCommit)
				})
			} else {
				bs.SaveBlockWithExtendedCommit(block, ps, seenCommit)
			}
		})
	}
}

// TestLoadBlockExtendedCommit tests loading the extended commit for a previously
// saved block. The load method should return nil when only a commit was saved and
// return the extended commit otherwise.
func TestLoadBlockExtendedCommit(t *testing.T) {
	for _, testCase := range []struct {
		name         string
		saveExtended bool
		expectResult bool
	}{
		{
			name:         "save commit",
			saveExtended: false,
			expectResult: false,
		},
		{
			name:         "save extended commit",
			saveExtended: true,
			expectResult: true,
		},
	} {
		t.Run(testCase.name, func(t *testing.T) {
			state, bs, _, _, cleanup, _ := makeStateAndBlockStoreAndIndexers()
			defer cleanup()
			h := bs.Height() + 1
			block := state.MakeBlock(h, test.MakeNTxs(h, 10), new(types.Commit), nil, state.Validators.GetProposer().Address)
			seenCommit := makeTestExtCommit(block.Header.Height, cmttime.Now())
			ps, err := block.MakePartSet(types.BlockPartSizeBytes)
			require.NoError(t, err)
			if testCase.saveExtended {
				bs.SaveBlockWithExtendedCommit(block, ps, seenCommit)
			} else {
				bs.SaveBlock(block, ps, seenCommit.ToCommit())
			}
			res := bs.LoadBlockExtendedCommit(block.Height)
			if testCase.expectResult {
				require.Equal(t, seenCommit, res)
			} else {
				require.Nil(t, res)
			}
		})
	}
}

func TestLoadBaseMeta(t *testing.T) {
	config := test.ResetTestRoot("blockchain_reactor_test")
	defer os.RemoveAll(config.RootDir)
	stateStore := sm.NewStore(dbm.NewMemDB(), sm.StoreOptions{
		DiscardABCIResponses: false,
	})
	state, err := stateStore.LoadFromDBOrGenesisFile(config.GenesisFile())
	require.NoError(t, err)
	bs := NewBlockStore(dbm.NewMemDB())

	for h := int64(1); h <= 10; h++ {
		block := state.MakeBlock(h, test.MakeNTxs(h, 10), new(types.Commit), nil, state.Validators.GetProposer().Address)
		partSet, err := block.MakePartSet(types.BlockPartSizeBytes)
		require.NoError(t, err)
		seenCommit := makeTestExtCommit(h, cmttime.Now())
		bs.SaveBlockWithExtendedCommit(block, partSet, seenCommit)
	}

	_, _, err = bs.PruneBlocks(4, state)
	require.NoError(t, err)

	baseBlock := bs.LoadBaseMeta()
	assert.EqualValues(t, 4, baseBlock.Header.Height)
	assert.EqualValues(t, 4, bs.Base())

	require.NoError(t, bs.DeleteLatestBlock())
	require.EqualValues(t, 9, bs.Height())
}

func TestLoadBlockPart(t *testing.T) {
	config := test.ResetTestRoot("blockchain_reactor_test")

	bs, db := newInMemoryBlockStore()
	const height, index = 10, 1
	loadPart := func() (any, error) {
		part := bs.LoadBlockPart(height, index)
		return part, nil
	}

	state, err := sm.MakeGenesisStateFromFile(config.GenesisFile())
	require.NoError(t, err)

	// Initially no contents.
	// 1. Requesting for a non-existent block shouldn't fail
	res, _, panicErr := doFn(loadPart)
	require.NoError(t, panicErr, "a non-existent block part shouldn't cause a panic")
	require.Nil(t, res, "a non-existent block part should return nil")

	// 2. Next save a corrupted block then try to load it
	err = db.Set(bs.dbKeyLayout.CalcBlockPartKey(height, index), []byte("CometBFT"))
	require.NoError(t, err)
	res, _, panicErr = doFn(loadPart)
	require.Error(t, panicErr, "expecting a non-nil panic")
	require.Contains(t, panicErr.Error(), "unmarshal to cmtproto.Part failed")

	// 3. A good block serialized and saved to the DB should be retrievable
	block := state.MakeBlock(height, nil, new(types.Commit), nil, state.Validators.GetProposer().Address)
	partSet, err := block.MakePartSet(types.BlockPartSizeBytes)
	require.NoError(t, err)
	part1 := partSet.GetPart(0)

	pb1, err := part1.ToProto()
	require.NoError(t, err)
	err = db.Set(bs.dbKeyLayout.CalcBlockPartKey(height, index), mustEncode(pb1))
	require.NoError(t, err)
	gotPart, _, panicErr := doFn(loadPart)
	require.NoError(t, panicErr, "an existent and proper block should not panic")
	require.Nil(t, res, "a properly saved block should return a proper block")

	// Having to do this because of https://github.com/stretchr/testify/issues/1141
	gotPartJSON, err := json.Marshal(gotPart.(*types.Part))
	require.NoError(t, err)
	part1JSON, err := json.Marshal(part1)
	require.NoError(t, err)
	require.JSONEq(t, string(gotPartJSON), string(part1JSON),
		"expecting successful retrieval of previously saved block")
}

type prunerObserver struct {
	sm.NoopPrunerObserver
	prunedABCIResInfoCh   chan *sm.ABCIResponsesPrunedInfo
	prunedBlocksResInfoCh chan *sm.BlocksPrunedInfo
}

func newPrunerObserver(infoChCap int) *prunerObserver {
	return &prunerObserver{
		prunedABCIResInfoCh:   make(chan *sm.ABCIResponsesPrunedInfo, infoChCap),
		prunedBlocksResInfoCh: make(chan *sm.BlocksPrunedInfo, infoChCap),
	}
}

func (o *prunerObserver) PrunerPrunedABCIRes(info *sm.ABCIResponsesPrunedInfo) {
	o.prunedABCIResInfoCh <- info
}

func (o *prunerObserver) PrunerPrunedBlocks(info *sm.BlocksPrunedInfo) {
	o.prunedBlocksResInfoCh <- info
}

// This test tests the pruning service and its pruning of the blockstore
// The state store cannot be pruned here because we do not have proper
// state stored. The test is expected to pass even though the log should
// inform about the inability to prune the state store.
func TestPruningService(t *testing.T) {
	config := test.ResetTestRoot("blockchain_reactor_pruning_test")
	defer os.RemoveAll(config.RootDir)
	state, bs, txIndexer, blockIndexer, cleanup, stateStore := makeStateAndBlockStoreAndIndexers()
	defer cleanup()
	assert.EqualValues(t, 0, bs.Base())
	assert.EqualValues(t, 0, bs.Height())
	assert.EqualValues(t, 0, bs.Size())

	err := initStateStoreRetainHeights(stateStore, 0, 0, 0)
	require.NoError(t, err)

	obs := newPrunerObserver(1)

	pruner := sm.NewPruner(
		stateStore,
		bs,
		blockIndexer,
		txIndexer,
		log.TestingLogger(),
		sm.WithPrunerInterval(time.Second*1),
		sm.WithPrunerObserver(obs),
		sm.WithPrunerCompanionEnabled(),
	)

	err = pruner.SetApplicationBlockRetainHeight(1)
	require.Error(t, err)
	err = pruner.SetApplicationBlockRetainHeight(0)
	require.NoError(t, err)

	// make more than 1000 blocks, to test batch deletions
	for h := int64(1); h <= 1500; h++ {
		block := state.MakeBlock(h, test.MakeNTxs(h, 10), new(types.Commit), nil, state.Validators.GetProposer().Address)
		partSet, err := block.MakePartSet(types.BlockPartSizeBytes)
		require.NoError(t, err)
		seenCommit := makeTestExtCommit(h, cmttime.Now())
		bs.SaveBlockWithExtendedCommit(block, partSet, seenCommit)
	}

	assert.EqualValues(t, 1, bs.Base())
	assert.EqualValues(t, 1500, bs.Height())
	assert.EqualValues(t, 1500, bs.Size())

	state.LastBlockTime = cmttime.Now().Add(24 * time.Hour)
	state.LastBlockHeight = 1500

	state.ConsensusParams.Evidence.MaxAgeNumBlocks = 400
	state.ConsensusParams.Evidence.MaxAgeDuration = 1 * time.Minute

	pk := ed25519.GenPrivKey().PubKey()

	// Generate a bunch of state data.
	// This is needed because the pruning is expecting to load the state from the database thus
	// We have to have acceptable values for all fields of the state
	validator := &types.Validator{Address: pk.Address(), VotingPower: 100, PubKey: pk}
	validatorSet := &types.ValidatorSet{
		Validators: []*types.Validator{validator},
		Proposer:   validator,
	}
	state.Validators = validatorSet
	state.NextValidators = validatorSet
	if state.LastBlockHeight >= 1 {
		state.LastValidators = state.Validators
	}

	err = stateStore.Save(state)
	require.NoError(t, err)
	// Check that basic pruning works
	err = pruner.SetApplicationBlockRetainHeight(1200)
	require.NoError(t, err)
	err = pruner.SetCompanionBlockRetainHeight(1200)
	require.NoError(t, err)
	err = pruner.Start()
	require.NoError(t, err)

	select {
	case info := <-obs.prunedBlocksResInfoCh:
		assert.EqualValues(t, 0, info.FromHeight)
		assert.EqualValues(t, 1199, info.ToHeight)
		assert.EqualValues(t, 1200, bs.Base())
		assert.EqualValues(t, 1500, bs.Height())
		assert.EqualValues(t, 301, bs.Size())
		block, meta := bs.LoadBlock(1200)
		require.NotNil(t, block)
		require.NotNil(t, meta)
		block, meta = bs.LoadBlock(1199)
		require.Nil(t, block)
		require.Nil(t, meta)
		// The header and commit for heights 1100 onwards
		// need to remain to verify evidence
		require.NotNil(t, bs.LoadBlockMeta(1100))
		require.Nil(t, bs.LoadBlockMeta(1099))
		require.NotNil(t, bs.LoadBlockCommit(1100))
		require.Nil(t, bs.LoadBlockCommit(1099))
		for i := int64(1); i < 1200; i++ {
			block, meta = bs.LoadBlock(i)
			require.Nil(t, block)
			require.Nil(t, meta)
		}
		for i := int64(1200); i <= 1500; i++ {
			block, meta = bs.LoadBlock(i)
			require.NotNil(t, block)
			require.NotNil(t, meta)
		}
		t.Log("Done pruning blocks until height 1200")

	case <-time.After(5 * time.Second):
		require.Fail(t, "timed out waiting for pruning run to complete")
	}

	// Pruning below the current base should error
	err = pruner.SetApplicationBlockRetainHeight(1199)
	require.Error(t, err)

	// Pruning to the current base should work
	err = pruner.SetApplicationBlockRetainHeight(1200)
	require.NoError(t, err)

	// Pruning again should work
	err = pruner.SetApplicationBlockRetainHeight(1300)
	require.NoError(t, err)

	err = pruner.SetCompanionBlockRetainHeight(1350)
	require.NoError(t, err)

	select {
	case <-obs.prunedBlocksResInfoCh:
		assert.EqualValues(t, 1300, bs.Base())

		// we should still have the header and the commit
		// as they're needed for evidence
		require.NotNil(t, bs.LoadBlockMeta(1100))
		require.Nil(t, bs.LoadBlockMeta(1099))
		require.NotNil(t, bs.LoadBlockCommit(1100))
		require.Nil(t, bs.LoadBlockCommit(1099))
		t.Log("Done pruning up until 1300")
	case <-time.After(5 * time.Second):
		require.Fail(t, "timed out waiting for pruning run to complete")
	}
	// Setting the pruning height beyond the current height should error
	err = pruner.SetApplicationBlockRetainHeight(1501)
	require.Error(t, err)

	// Pruning to the current height should work
	err = pruner.SetApplicationBlockRetainHeight(1500)
	require.NoError(t, err)

	select {
	case <-obs.prunedBlocksResInfoCh:
		// But we will prune only until 1350 because that was the Companions height
		// and it is lower
		block, meta := bs.LoadBlock(1349)
		assert.Nil(t, block)
		assert.Nil(t, meta)
		block, meta = bs.LoadBlock(1350)
		assert.NotNil(t, block, fmt.Sprintf("expected block at height 1350 to be there, but it was not; block store base height = %d", bs.Base()))
		assert.NotNil(t, meta)
		block, meta = bs.LoadBlock(1500)
		assert.NotNil(t, block)
		assert.NotNil(t, meta)
		block, meta = bs.LoadBlock(1501)
		assert.Nil(t, block)
		assert.Nil(t, meta)
		t.Log("Done pruning blocks until 1500")

	case <-time.After(5 * time.Second):
		require.Fail(t, "timed out waiting for pruning run to complete")
	}
}

func TestPruneBlocks(t *testing.T) {
	config := test.ResetTestRoot("blockchain_reactor_test")
	defer os.RemoveAll(config.RootDir)
	stateStore := sm.NewStore(dbm.NewMemDB(), sm.StoreOptions{
		DiscardABCIResponses: false,
	})
	state, err := stateStore.LoadFromDBOrGenesisFile(config.GenesisFile())
	require.NoError(t, err)
	db := dbm.NewMemDB()
	bs := NewBlockStore(db)
	assert.EqualValues(t, 0, bs.Base())
	assert.EqualValues(t, 0, bs.Height())
	assert.EqualValues(t, 0, bs.Size())

	// pruning an empty store should error, even when pruning to 0
	_, _, err = bs.PruneBlocks(1, state)
	require.Error(t, err)

	_, _, err = bs.PruneBlocks(0, state)
	require.Error(t, err)

	// make more than 1000 blocks, to test batch deletions
	for h := int64(1); h <= 1500; h++ {
		block := state.MakeBlock(h, test.MakeNTxs(h, 10), new(types.Commit), nil, state.Validators.GetProposer().Address)
		partSet, err := block.MakePartSet(types.BlockPartSizeBytes)
		require.NoError(t, err)
		seenCommit := makeTestExtCommit(h, cmttime.Now())
		bs.SaveBlockWithExtendedCommit(block, partSet, seenCommit)
	}

	assert.EqualValues(t, 1, bs.Base())
	assert.EqualValues(t, 1500, bs.Height())
	assert.EqualValues(t, 1500, bs.Size())

	state.LastBlockTime = cmttime.Now().Add(24 * time.Hour)
	state.LastBlockHeight = 1500

	state.ConsensusParams.Evidence.MaxAgeNumBlocks = 400
	state.ConsensusParams.Evidence.MaxAgeDuration = 1 * time.Minute

	// Check that basic pruning works
	pruned, evidenceRetainHeight, err := bs.PruneBlocks(1200, state)
	require.NoError(t, err)
	assert.EqualValues(t, 1199, pruned)
	assert.EqualValues(t, 1200, bs.Base())
	assert.EqualValues(t, 1500, bs.Height())
	assert.EqualValues(t, 301, bs.Size())
	assert.EqualValues(t, 1100, evidenceRetainHeight)

	block, meta := bs.LoadBlock(1200)
	require.NotNil(t, block)
	require.NotNil(t, meta)
	block, meta = bs.LoadBlock(1199)
	require.Nil(t, block)
	require.Nil(t, meta)

	// The header and commit for heights 1100 onwards
	// need to remain to verify evidence
	require.NotNil(t, bs.LoadBlockMeta(1100))
	require.Nil(t, bs.LoadBlockMeta(1099))
	require.NotNil(t, bs.LoadBlockCommit(1100))
	require.Nil(t, bs.LoadBlockCommit(1099))

	for i := int64(1); i < 1200; i++ {
		block, meta = bs.LoadBlock(i)
		require.Nil(t, block)
		require.Nil(t, meta)
	}
	for i := int64(1200); i <= 1500; i++ {
		block, meta = bs.LoadBlock(i)
		require.NotNil(t, block)
		require.NotNil(t, meta)
	}

	// Pruning below the current base should error
	_, _, err = bs.PruneBlocks(1199, state)
	require.Error(t, err)

	// Pruning to the current base should work
	pruned, _, err = bs.PruneBlocks(1200, state)
	require.NoError(t, err)
	assert.EqualValues(t, 0, pruned)

	// Pruning again should work
	pruned, _, err = bs.PruneBlocks(1300, state)
	require.NoError(t, err)
	assert.EqualValues(t, 100, pruned)
	assert.EqualValues(t, 1300, bs.Base())

	// we should still have the header and the commit
	// as they're needed for evidence
	require.NotNil(t, bs.LoadBlockMeta(1100))
	require.Nil(t, bs.LoadBlockMeta(1099))
	require.NotNil(t, bs.LoadBlockCommit(1100))
	require.Nil(t, bs.LoadBlockCommit(1099))

	// Pruning beyond the current height should error
	_, _, err = bs.PruneBlocks(1501, state)
	require.Error(t, err)

	// Pruning to the current height should work
	pruned, _, err = bs.PruneBlocks(1500, state)
	require.NoError(t, err)
	assert.EqualValues(t, 200, pruned)
	block, meta = bs.LoadBlock(1499)
	assert.Nil(t, block)
	assert.Nil(t, meta)
	block, meta = bs.LoadBlock(1500)
	assert.NotNil(t, block)
	assert.NotNil(t, meta)
	block, meta = bs.LoadBlock(1501)
	assert.Nil(t, block)
	assert.Nil(t, meta)
}

func TestLoadBlockMeta(t *testing.T) {
	bs, db := newInMemoryBlockStore()
	height := int64(10)
	loadMeta := func() (any, error) {
		meta := bs.LoadBlockMeta(height)
		return meta, nil
	}

	// Initially no contents.
	// 1. Requesting for a non-existent blockMeta shouldn't fail
	res, _, panicErr := doFn(loadMeta)
	require.NoError(t, panicErr)
	require.Nil(t, res, "a non-existent blockMeta should return nil")

	// 2. Next save a corrupted blockMeta then try to load it
	err := db.Set(bs.dbKeyLayout.CalcBlockMetaKey(height), []byte("CometBFT-Meta"))
	require.NoError(t, err)
	res, _, panicErr = doFn(loadMeta)
	require.Error(t, panicErr)
	require.Contains(t, panicErr.Error(), "unmarshal to cmtproto.BlockMeta")

	// 3. A good blockMeta serialized and saved to the DB should be retrievable
	meta := &types.BlockMeta{Header: types.Header{
		Version: cmtversion.Consensus{
			Block: version.BlockProtocol, App: 0,
		}, Height: 1, ProposerAddress: cmtrand.Bytes(crypto.AddressSize),
	}}
	pbm := meta.ToProto()
	err = db.Set(bs.dbKeyLayout.CalcBlockMetaKey(height), mustEncode(pbm))
	require.NoError(t, err)
	gotMeta, _, panicErr := doFn(loadMeta)
	require.NoError(t, panicErr, "an existent and proper block should not panic")
	require.Nil(t, res, "a properly saved blockMeta should return a proper blocMeta ")
	pbmeta := meta.ToProto()
	if gmeta, ok := gotMeta.(*types.BlockMeta); ok {
		pbgotMeta := gmeta.ToProto()
		require.Equal(t, mustEncode(pbmeta), mustEncode(pbgotMeta),
			"expecting successful retrieval of previously saved blockMeta")
	}
}

func TestLoadBlockMetaByHash(t *testing.T) {
	config := test.ResetTestRoot("blockchain_reactor_test")
	defer os.RemoveAll(config.RootDir)
	stateStore := sm.NewStore(dbm.NewMemDB(), sm.StoreOptions{
		DiscardABCIResponses: false,
	})
	state, err := stateStore.LoadFromDBOrGenesisFile(config.GenesisFile())
	require.NoError(t, err)
	bs := NewBlockStore(dbm.NewMemDB())

	b1 := state.MakeBlock(state.LastBlockHeight+1, test.MakeNTxs(state.LastBlockHeight+1, 10), new(types.Commit), nil, state.Validators.GetProposer().Address)
	partSet, err := b1.MakePartSet(types.BlockPartSizeBytes)
	require.NoError(t, err)
	seenCommit := makeTestExtCommit(1, cmttime.Now())
	bs.SaveBlock(b1, partSet, seenCommit.ToCommit())

	baseBlock := bs.LoadBlockMetaByHash(b1.Hash())
	assert.EqualValues(t, b1.Header.Height, baseBlock.Header.Height)
	assert.EqualValues(t, b1.Header.LastBlockID, baseBlock.Header.LastBlockID)
	assert.EqualValues(t, b1.Header.ChainID, baseBlock.Header.ChainID)
}

func TestBlockFetchAtHeight(t *testing.T) {
	state, bs, _, _, cleanup, _ := makeStateAndBlockStoreAndIndexers()
	defer cleanup()
	require.Equal(t, int64(0), bs.Height(), "initially the height should be zero")
	block := state.MakeBlock(bs.Height()+1, nil, new(types.Commit), nil, state.Validators.GetProposer().Address)

	partSet, err := block.MakePartSet(types.BlockPartSizeBytes)
	require.NoError(t, err)
	seenCommit := makeTestExtCommit(block.Header.Height, cmttime.Now())
	bs.SaveBlockWithExtendedCommit(block, partSet, seenCommit)
	require.Equal(t, bs.Height(), block.Header.Height, "expecting the new height to be changed")

	blockAtHeight, _ := bs.LoadBlock(bs.Height())
	b1, err := block.ToProto()
	require.NoError(t, err)
	b2, err := blockAtHeight.ToProto()
	require.NoError(t, err)
	bz1 := mustEncode(b1)
	bz2 := mustEncode(b2)
	require.Equal(t, bz1, bz2)
	require.Equal(t, block.Hash(), blockAtHeight.Hash(),
		"expecting a successful load of the last saved block")

	blockAtHeightPlus1, _ := bs.LoadBlock(bs.Height() + 1)
	require.Nil(t, blockAtHeightPlus1, "expecting an unsuccessful load of Height()+1")
	blockAtHeightPlus2, _ := bs.LoadBlock(bs.Height() + 2)
	require.Nil(t, blockAtHeightPlus2, "expecting an unsuccessful load of Height()+2")
}

func doFn(fn func() (any, error)) (res any, err error, panicErr error) {
	defer func() {
		if r := recover(); r != nil {
			switch e := r.(type) {
			case error:
				panicErr = e
			case string:
				panicErr = fmt.Errorf("%s", e)
			default:
				if st, ok := r.(fmt.Stringer); ok {
					panicErr = fmt.Errorf("%s", st)
				} else {
					panicErr = fmt.Errorf("%s", debug.Stack())
				}
			}
		}
	}()

	res, err = fn()
	return res, err, panicErr
}

func newBlock(hdr types.Header, lastCommit *types.Commit) *types.Block {
	return &types.Block{
		Header:     hdr,
		LastCommit: lastCommit,
	}
}

func initStateStoreRetainHeights(stateStore sm.Store, appBlockRH, dcBlockRH, dcBlockResultsRH int64) error {
	if err := stateStore.SaveApplicationRetainHeight(appBlockRH); err != nil {
		return fmt.Errorf("failed to set initial application block retain height: %w", err)
	}
	if err := stateStore.SaveCompanionBlockRetainHeight(dcBlockRH); err != nil {
		return fmt.Errorf("failed to set initial companion block retain height: %w", err)
	}
	if err := stateStore.SaveABCIResRetainHeight(dcBlockResultsRH); err != nil {
		return fmt.Errorf("failed to set initial ABCI results retain height: %w", err)
	}
	return nil
}

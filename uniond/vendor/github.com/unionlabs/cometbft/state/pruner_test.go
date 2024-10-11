package state_test

import (
	"context"
	"fmt"
	"os"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"golang.org/x/exp/slices"

	db "github.com/cometbft/cometbft-db"
	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/internal/test"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/libs/pubsub/query"
	sm "github.com/cometbft/cometbft/state"
	blockidxkv "github.com/cometbft/cometbft/state/indexer/block/kv"
	"github.com/cometbft/cometbft/state/txindex"
	"github.com/cometbft/cometbft/state/txindex/kv"
	"github.com/cometbft/cometbft/store"
	"github.com/cometbft/cometbft/types"
)

func TestPruneBlockIndexerToRetainHeight(t *testing.T) {
	pruner, _, blockIndexer := createTestSetup(t)

	for height := int64(1); height <= 4; height++ {
		events, _, _ := getEventsAndResults(height)
		err := blockIndexer.Index(events)
		require.NoError(t, err)
	}
	err := pruner.SetBlockIndexerRetainHeight(2)
	require.NoError(t, err)
	actual, err := pruner.GetBlockIndexerRetainHeight()
	require.NoError(t, err)
	require.Equal(t, int64(2), actual)

	heights, err := blockIndexer.Search(context.Background(), query.MustCompile("block.height <= 2"))
	require.NoError(t, err)
	require.Equal(t, []int64{1, 2}, heights)

	newRetainHeight := pruner.PruneBlockIndexerToRetainHeight(0)
	require.Equal(t, int64(2), newRetainHeight)

	heights, err = blockIndexer.Search(context.Background(), query.MustCompile("block.height <= 2"))
	require.NoError(t, err)
	require.Equal(t, []int64{2}, heights)

	err = pruner.SetBlockIndexerRetainHeight(int64(4))
	require.NoError(t, err)
	actual, err = pruner.GetBlockIndexerRetainHeight()
	require.NoError(t, err)
	require.Equal(t, int64(4), actual)

	heights, err = blockIndexer.Search(context.Background(), query.MustCompile("block.height <= 4"))
	require.NoError(t, err)
	require.Equal(t, []int64{2, 3, 4}, heights)

	pruner.PruneBlockIndexerToRetainHeight(2)

	heights, err = blockIndexer.Search(context.Background(), query.MustCompile("block.height <= 4"))
	require.NoError(t, err)
	require.Equal(t, []int64{4}, heights)

	events, _, _ := getEventsAndResults(1)

	err = blockIndexer.Index(events)
	require.NoError(t, err)

	heights, err = blockIndexer.Search(context.Background(), query.MustCompile("block.height <= 4"))
	require.NoError(t, err)
	require.Equal(t, []int64{1, 4}, heights)

	pruner.PruneBlockIndexerToRetainHeight(4)

	heights, err = blockIndexer.Search(context.Background(), query.MustCompile("block.height <= 4"))
	require.NoError(t, err)
	require.Equal(t, []int64{1, 4}, heights)
}

func TestPruneTxIndexerToRetainHeight(t *testing.T) {
	pruner, txIndexer, _ := createTestSetup(t)

	for height := int64(1); height <= 4; height++ {
		_, txResult1, txResult2 := getEventsAndResults(height)
		err := txIndexer.Index(txResult1)
		require.NoError(t, err)
		err = txIndexer.Index(txResult2)
		require.NoError(t, err)
	}

	err := pruner.SetTxIndexerRetainHeight(2)
	require.NoError(t, err)
	actual, err := pruner.GetTxIndexerRetainHeight()
	require.NoError(t, err)
	require.Equal(t, int64(2), actual)

	results, _, err := txIndexer.Search(context.Background(), query.MustCompile("tx.height < 2"), txindex.Pagination{})
	require.NoError(t, err)
	require.True(t, containsAllTxs(results, []string{"foo1", "bar1"}))

	newRetainHeight := pruner.PruneTxIndexerToRetainHeight(0)
	require.Equal(t, int64(2), newRetainHeight)

	results, _, err = txIndexer.Search(context.Background(), query.MustCompile("tx.height < 2"), txindex.Pagination{})
	require.NoError(t, err)
	require.Empty(t, results)

	err = pruner.SetTxIndexerRetainHeight(int64(4))
	require.NoError(t, err)
	actual, err = pruner.GetTxIndexerRetainHeight()
	require.NoError(t, err)
	require.Equal(t, int64(4), actual)

	results, _, err = txIndexer.Search(context.Background(), query.MustCompile("tx.height < 4"), txindex.Pagination{})
	require.NoError(t, err)
	require.True(t, containsAllTxs(results, []string{"foo2", "bar2", "foo3", "bar3"}))

	pruner.PruneTxIndexerToRetainHeight(2)

	results, _, err = txIndexer.Search(context.Background(), query.MustCompile("tx.height < 4"), txindex.Pagination{})
	require.NoError(t, err)
	require.Empty(t, results)

	_, txResult1, txResult2 := getEventsAndResults(1)
	err = txIndexer.Index(txResult1)
	require.NoError(t, err)
	err = txIndexer.Index(txResult2)
	require.NoError(t, err)

	results, _, err = txIndexer.Search(context.Background(), query.MustCompile("tx.height <= 4"), txindex.Pagination{})
	require.NoError(t, err)
	require.True(t, containsAllTxs(results, []string{"foo1", "bar1", "foo4", "bar4"}))

	pruner.PruneTxIndexerToRetainHeight(4)

	results, _, err = txIndexer.Search(context.Background(), query.MustCompile("tx.height <= 4"), txindex.Pagination{})
	require.NoError(t, err)
	require.True(t, containsAllTxs(results, []string{"foo1", "bar1", "foo4", "bar4"}))
}

func containsAllTxs(results []*abci.TxResult, txs []string) bool {
	for _, tx := range txs {
		if !slices.ContainsFunc(results, func(result *abci.TxResult) bool {
			return string(result.Tx) == tx
		}) {
			return false
		}
	}
	return true
}

func createTestSetup(t *testing.T) (*sm.Pruner, *kv.TxIndex, blockidxkv.BlockerIndexer) {
	t.Helper()
	config := test.ResetTestRoot("pruner_test")
	t.Cleanup(func() {
		err := os.RemoveAll(config.RootDir)
		if err != nil {
			t.Error(err)
		}
	})
	// event bus
	eventBus := types.NewEventBus()
	eventBus.SetLogger(log.TestingLogger())
	err := eventBus.Start()
	require.NoError(t, err)

	t.Cleanup(func() {
		if err := eventBus.Stop(); err != nil {
			t.Error(err)
		}
		err := os.RemoveAll(config.RootDir)
		if err != nil {
			t.Error(err)
		}
	})

	// tx indexer
	memDB := db.NewMemDB()
	txIndexer := kv.NewTxIndex(memDB)
	blockIndexer := blockidxkv.New(db.NewPrefixDB(memDB, []byte("block_events")))

	blockDB := db.NewMemDB()
	stateDB := db.NewMemDB()
	stateStore := sm.NewStore(stateDB, sm.StoreOptions{
		DiscardABCIResponses: false,
	})
	bs := store.NewBlockStore(blockDB)
	pruner := sm.NewPruner(stateStore, bs, blockIndexer, txIndexer, log.TestingLogger())

	return pruner, txIndexer, *blockIndexer
}

func getEventsAndResults(height int64) (types.EventDataNewBlockEvents, *abci.TxResult, *abci.TxResult) {
	events := types.EventDataNewBlockEvents{
		Height: height,
		Events: []abci.Event{
			{
				Type: "begin_event",
				Attributes: []abci.EventAttribute{
					{
						Key:   "proposer",
						Value: "FCAA001",
						Index: true,
					},
				},
			},
		},
		NumTxs: int64(2),
	}
	txResult1 := &abci.TxResult{
		Height: height,
		Index:  uint32(0),
		Tx:     types.Tx(fmt.Sprintf("foo%d", height)),
		Result: abci.ExecTxResult{Code: 0},
	}
	txResult2 := &abci.TxResult{
		Height: height,
		Index:  uint32(1),
		Tx:     types.Tx(fmt.Sprintf("bar%d", height)),
		Result: abci.ExecTxResult{Code: 0},
	}
	return events, txResult1, txResult2
}

// When trying to prune the only block in the store it should not succeed
// State should also not be pruned.
func TestPruningWithHeight1(t *testing.T) {
	config := test.ResetTestRoot("blockchain_reactor_pruning_test")
	defer os.RemoveAll(config.RootDir)
	state, bs, txIndexer, blockIndexer, cleanup, stateStore := makeStateAndBlockStoreAndIndexers()
	defer cleanup()
	require.EqualValues(t, 0, bs.Base())
	require.EqualValues(t, 0, bs.Height())
	require.EqualValues(t, 0, bs.Size())

	err := initStateStoreRetainHeights(stateStore)
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

	block := state.MakeBlock(1, test.MakeNTxs(1, 10), new(types.Commit), nil, state.Validators.GetProposer().Address)
	partSet, err := block.MakePartSet(2)
	require.NoError(t, err)

	bs.SaveBlock(block, partSet, &types.Commit{Height: 1})
	require.EqualValues(t, 1, bs.Base())
	require.EqualValues(t, 1, bs.Height())

	err = stateStore.Save(state)
	require.NoError(t, err)

	err = pruner.SetApplicationBlockRetainHeight(1)
	require.NoError(t, err)
	err = pruner.SetCompanionBlockRetainHeight(1)
	require.NoError(t, err)

	pruned, _, err := pruner.PruneBlocksToHeight(1)
	require.Equal(t, uint64(0), pruned)
	require.NoError(t, err)
}

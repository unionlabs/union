package txindex_test

import (
	"fmt"
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	db "github.com/cometbft/cometbft-db"
	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/state/indexer"
	blockidxkv "github.com/cometbft/cometbft/state/indexer/block/kv"
	"github.com/cometbft/cometbft/state/txindex"
	"github.com/cometbft/cometbft/state/txindex/kv"
	"github.com/cometbft/cometbft/types"
)

func TestIndexerServiceIndexesBlocks(t *testing.T) {
	_, txIndexer, blockIndexer, eventBus := createTestSetup(t)

	height := int64(1)

	events, txResult1, txResult2 := getEventsAndResults(height)
	// publish block with events
	err := eventBus.PublishEventNewBlockEvents(events)
	require.NoError(t, err)

	err = eventBus.PublishEventTx(types.EventDataTx{TxResult: *txResult1})
	require.NoError(t, err)

	err = eventBus.PublishEventTx(types.EventDataTx{TxResult: *txResult2})
	require.NoError(t, err)

	time.Sleep(100 * time.Millisecond)

	res, err := txIndexer.Get(types.Tx(fmt.Sprintf("foo%d", height)).Hash())
	require.NoError(t, err)
	require.Equal(t, txResult1, res)

	ok, err := blockIndexer.Has(height)
	require.NoError(t, err)
	require.True(t, ok)

	res, err = txIndexer.Get(types.Tx(fmt.Sprintf("bar%d", height)).Hash())
	require.NoError(t, err)
	require.Equal(t, txResult2, res)
}

func createTestSetup(t *testing.T) (*txindex.IndexerService, *kv.TxIndex, indexer.BlockIndexer, *types.EventBus) {
	t.Helper()
	// event bus
	eventBus := types.NewEventBus()
	eventBus.SetLogger(log.TestingLogger())
	err := eventBus.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := eventBus.Stop(); err != nil {
			t.Error(err)
		}
	})

	// tx indexer
	store := db.NewMemDB()
	txIndexer := kv.NewTxIndex(store)
	blockIndexer := blockidxkv.New(db.NewPrefixDB(store, []byte("block_events")))

	service := txindex.NewIndexerService(txIndexer, blockIndexer, eventBus, false)
	service.SetLogger(log.TestingLogger())
	err = service.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := service.Stop(); err != nil {
			t.Error(err)
		}
	})
	return service, txIndexer, blockIndexer, eventBus
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

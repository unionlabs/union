package kv

import (
	"context"
	"crypto/rand"
	"fmt"
	"os"
	"testing"

	dbm "github.com/cometbft/cometbft-db"
	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/libs/pubsub/query"
	"github.com/cometbft/cometbft/state/txindex"
	"github.com/cometbft/cometbft/types"
)

func generateDummyTxs(b *testing.B, indexer *TxIndex, numHeights int, numTxs int) {
	b.Helper()
	for h := 0; h < numHeights; h++ {
		batch := txindex.NewBatch(int64(numTxs))

		for i := 0; i < numTxs; i++ {
			events := []abci.Event{
				{
					Type: "transfer",
					Attributes: []abci.EventAttribute{
						{Key: "address", Value: fmt.Sprintf("address_%d", (h*numTxs+i)%100), Index: true},
						{Key: "amount", Value: "50", Index: true},
					},
				},
			}

			txBz := make([]byte, 8)
			if _, err := rand.Read(txBz); err != nil {
				b.Errorf("failed produce random bytes: %s", err)
			}

			if err := batch.Add(&abci.TxResult{
				Height: int64(h),
				Index:  uint32(i),
				Tx:     types.Tx(string(txBz)),
				Result: abci.ExecTxResult{
					Data:   []byte{0},
					Code:   abci.CodeTypeOK,
					Log:    "",
					Events: events,
				},
			}); err != nil {
				b.Errorf("failed to index tx: %s", err)
			}
		}

		if err := indexer.AddBatch(batch); err != nil {
			b.Errorf("failed to add batch: %s", err)
		}
	}
}

func BenchmarkTxSearchDisk(b *testing.B) {
	dbDir, err := os.MkdirTemp("", "benchmark_tx_search_test")
	if err != nil {
		b.Errorf("failed to create temporary directory: %s", err)
	}

	db, err := dbm.NewGoLevelDB("benchmark_tx_search_test", dbDir)
	if err != nil {
		b.Errorf("failed to create database: %s", err)
	}

	indexer := NewTxIndex(db)
	generateDummyTxs(b, indexer, 1000, 20)

	txQuery := query.MustCompile(`transfer.address = 'address_43' AND transfer.amount = 50`)

	b.ResetTimer()

	ctx := context.Background()

	for i := 0; i < b.N; i++ {
		if _, _, err := indexer.Search(ctx, txQuery, DefaultPagination); err != nil {
			b.Errorf("failed to query for txs: %s", err)
		}
	}
}

func BenchmarkTxSearchBigResult(b *testing.B) {
	db := dbm.NewMemDB()
	indexer := NewTxIndex(db)
	generateDummyTxs(b, indexer, 20000, 50)

	txQuery := query.MustCompile(`transfer.amount = 50`)

	b.ResetTimer()

	ctx := context.Background()

	for i := 0; i < b.N; i++ {
		if _, _, err := indexer.Search(ctx, txQuery, DefaultPagination); err != nil {
			b.Errorf("failed to query for txs: %s", err)
		}
	}
}

package kv

import (
	"bytes"
	"context"
	"fmt"
	"os"
	"testing"

	"github.com/cosmos/gogoproto/proto"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"golang.org/x/exp/slices"

	db "github.com/cometbft/cometbft-db"
	abci "github.com/cometbft/cometbft/abci/types"
	cmtrand "github.com/cometbft/cometbft/internal/rand"
	"github.com/cometbft/cometbft/libs/pubsub/query"
	blockidxkv "github.com/cometbft/cometbft/state/indexer/block/kv"
	"github.com/cometbft/cometbft/state/txindex"
	"github.com/cometbft/cometbft/types"
)

var DefaultPagination = txindex.Pagination{
	IsPaginated: true,
	Page:        1,
	PerPage:     100,
	OrderDesc:   false,
}

func TestTxIndex(t *testing.T) {
	indexer := NewTxIndex(db.NewMemDB())

	tx := types.Tx("HELLO WORLD")
	txResult := &abci.TxResult{
		Height: 1,
		Index:  0,
		Tx:     tx,
		Result: abci.ExecTxResult{
			Data: []byte{0},
			Code: abci.CodeTypeOK, Log: "", Events: nil,
		},
	}
	hash := tx.Hash()

	batch := txindex.NewBatch(1)
	if err := batch.Add(txResult); err != nil {
		t.Error(err)
	}
	err := indexer.AddBatch(batch)
	require.NoError(t, err)

	loadedTxResult, err := indexer.Get(hash)
	require.NoError(t, err)
	assert.True(t, proto.Equal(txResult, loadedTxResult))

	tx2 := types.Tx("BYE BYE WORLD")
	txResult2 := &abci.TxResult{
		Height: 1,
		Index:  0,
		Tx:     tx2,
		Result: abci.ExecTxResult{
			Data: []byte{0},
			Code: abci.CodeTypeOK, Log: "", Events: nil,
		},
	}
	hash2 := tx2.Hash()

	err = indexer.Index(txResult2)
	require.NoError(t, err)

	loadedTxResult2, err := indexer.Get(hash2)
	require.NoError(t, err)
	assert.True(t, proto.Equal(txResult2, loadedTxResult2))
}

func TestTxIndex_Prune(t *testing.T) {
	indexer := NewTxIndex(db.NewMemDB())

	metaKeys := [][]byte{
		LastTxIndexerRetainHeightKey,
		blockidxkv.LastBlockIndexerRetainHeightKey,
		TxIndexerRetainHeightKey,
		blockidxkv.BlockIndexerRetainHeightKey,
	}

	tx := types.Tx("HELLO WORLD")
	events := []abci.Event{
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "number", Value: "1", Index: true}}},
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "owner", Value: "/Ivan/", Index: true}}},
		{Type: "", Attributes: []abci.EventAttribute{{Key: "not_allowed", Value: "Vlad", Index: true}}},
	}
	txResult := &abci.TxResult{
		Height: 1,
		Index:  0,
		Tx:     tx,
		Result: abci.ExecTxResult{
			Data:   []byte{0},
			Code:   abci.CodeTypeOK,
			Log:    "",
			Events: events,
		},
	}

	batch := txindex.NewBatch(1)
	if err := batch.Add(txResult); err != nil {
		t.Error(err)
	}
	err := indexer.AddBatch(batch)
	require.NoError(t, err)

	keys1 := GetKeys(indexer)

	tx2 := types.Tx("BYE BYE WORLD")
	txResult2 := &abci.TxResult{
		Height: 2,
		Index:  0,
		Tx:     tx2,
		Result: abci.ExecTxResult{
			Data:   []byte{0},
			Code:   abci.CodeTypeOK,
			Log:    "",
			Events: events,
		},
	}
	hash2 := tx2.Hash()

	batch = txindex.NewBatch(1)
	if err := batch.Add(txResult2); err != nil {
		t.Error(err)
	}
	err = indexer.AddBatch(batch)
	require.NoError(t, err)

	keys2 := GetKeys(indexer)
	assert.True(t, isSubset(keys1, keys2))

	numPruned, retainedHeight, err := indexer.Prune(2)
	require.NoError(t, err)
	assert.Equal(t, int64(1), numPruned)
	assert.Equal(t, int64(2), retainedHeight)

	keys3 := GetKeys(indexer)
	assert.True(t, isEqualSets(setDiff(keys2, keys1), setDiff(keys3, metaKeys)))
	assert.True(t, emptyIntersection(keys1, keys3))

	loadedTxResult2, err := indexer.Get(hash2)
	require.NoError(t, err)
	assert.True(t, proto.Equal(txResult2, loadedTxResult2))
}

func TestTxSearch(t *testing.T) {
	indexer := NewTxIndex(db.NewMemDB())

	txResult := txResultWithEvents([]abci.Event{
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "number", Value: "1", Index: true}}},
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "owner", Value: "/Ivan/", Index: true}}},
		{Type: "", Attributes: []abci.EventAttribute{{Key: "not_allowed", Value: "Vlad", Index: true}}},
	})
	hash := types.Tx(txResult.Tx).Hash()

	err := indexer.Index(txResult)
	require.NoError(t, err)

	testCases := []struct {
		q             string
		resultsLength int
	}{
		//	search by hash
		{fmt.Sprintf("tx.hash = '%X'", hash), 1},
		// search by hash (lower)
		{fmt.Sprintf("tx.hash = '%x'", hash), 1},
		// search by exact match (one key)
		{"account.number = 1", 1},
		// search by exact match (two keys)
		{"account.number = 1 AND account.owner = 'Ivan'", 0},
		{"account.owner = 'Ivan' AND account.number = 1", 0},
		{"account.owner = '/Ivan/'", 1},
		// search by exact match (two keys)
		{"account.number = 1 AND account.owner = 'Vlad'", 0},
		{"account.owner = 'Vlad' AND account.number = 1", 0},
		{"account.number >= 1 AND account.owner = 'Vlad'", 0},
		{"account.owner = 'Vlad' AND account.number >= 1", 0},
		{"account.number <= 0", 0},
		{"account.number <= 0 AND account.owner = 'Ivan'", 0},
		{"account.number < 10000 AND account.owner = 'Ivan'", 0},
		// search using a prefix of the stored value
		{"account.owner = 'Iv'", 0},
		// search by range
		{"account.number >= 1 AND account.number <= 5", 1},
		// search by range and another key
		{"account.number >= 1 AND account.owner = 'Ivan' AND account.number <= 5", 0},
		// search by range (lower bound)
		{"account.number >= 1", 1},
		// search by range (upper bound)
		{"account.number <= 5", 1},
		{"account.number <= 1", 1},
		// search using not allowed key
		{"not_allowed = 'boom'", 0},
		{"not_allowed = 'Vlad'", 0},
		// search for not existing tx result
		{"account.number >= 2 AND account.number <= 5 AND tx.height > 0", 0},
		// search using not existing key
		{"account.date >= TIME 2013-05-03T14:45:00Z", 0},
		// search using CONTAINS
		{"account.owner CONTAINS 'an'", 1},
		//	search for non existing value using CONTAINS
		{"account.owner CONTAINS 'Vlad'", 0},
		{"account.owner CONTAINS 'Ivann'", 0},
		{"account.owner CONTAINS 'IIvan'", 0},
		{"account.owner CONTAINS 'Iva n'", 0},
		{"account.owner CONTAINS ' Ivan'", 0},
		{"account.owner CONTAINS 'Ivan '", 0},
		// search using the wrong key (of numeric type) using CONTAINS
		{"account.number CONTAINS 'Iv'", 0},
		// search using EXISTS
		{"account.number EXISTS", 1},
		// search using EXISTS for non existing key
		{"account.date EXISTS", 0},
		{"not_allowed EXISTS", 0},
	}

	ctx := context.Background()

	for _, tc := range testCases {
		t.Run(tc.q, func(t *testing.T) {
			results, _, err := indexer.Search(ctx, query.MustCompile(tc.q), DefaultPagination)
			require.NoError(t, err)

			assert.Len(t, results, tc.resultsLength)
			if tc.resultsLength > 0 {
				for _, txr := range results {
					assert.True(t, proto.Equal(txResult, txr))
				}
			}
		})
	}
}

func TestTxSearchEventMatch(t *testing.T) {
	indexer := NewTxIndex(db.NewMemDB())

	txResult := txResultWithEvents([]abci.Event{
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "number", Value: "1", Index: true}, {Key: "owner", Value: "Ana", Index: true}}},
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "number", Value: "2", Index: true}, {Key: "owner", Value: "/Ivan/.test", Index: true}}},
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "number", Value: "3", Index: false}, {Key: "owner", Value: "Mickey", Index: false}}},
		{Type: "", Attributes: []abci.EventAttribute{{Key: "not_allowed", Value: "Vlad", Index: true}}},
	})

	err := indexer.Index(txResult)
	require.NoError(t, err)

	testCases := map[string]struct {
		q             string
		resultsLength int
	}{
		"Return all events from a height": {
			q:             "tx.height = 1",
			resultsLength: 1,
		},
		"Don't match non-indexed events": {
			q:             "account.number = 3 AND account.owner = 'Mickey'",
			resultsLength: 0,
		},
		"Return all events from a height with range": {
			q:             "tx.height > 0",
			resultsLength: 1,
		},
		"Return all events from a height with range 2": {
			q:             "tx.height <= 1",
			resultsLength: 1,
		},
		"Return all events from a height (deduplicate height)": {
			q:             "tx.height = 1 AND tx.height = 1",
			resultsLength: 1,
		},
		"Match attributes with height range and event": {
			q:             "tx.height < 2 AND tx.height > 0 AND account.number > 0 AND account.number <= 1 AND account.owner CONTAINS 'Ana'",
			resultsLength: 1,
		},
		"Match attributes with multiple CONTAIN and height range": {
			q:             "tx.height < 2 AND tx.height > 0 AND account.number = 1 AND account.owner CONTAINS 'Ana' AND account.owner CONTAINS 'An'",
			resultsLength: 1,
		},
		"Match attributes with height range and event - no match": {
			q:             "tx.height < 2 AND tx.height > 0 AND account.number = 2 AND account.owner = 'Ana'",
			resultsLength: 0,
		},
		"Match attributes with event": {
			q:             "account.number = 2 AND account.owner = 'Ana' AND tx.height = 1",
			resultsLength: 0,
		},
		"Deduplication test - should return nothing if attribute repeats multiple times": {
			q:             "tx.height < 2 AND account.number = 3 AND account.number = 2 AND account.number = 5",
			resultsLength: 0,
		},
		" Match range with special character": {
			q:             "account.number < 2 AND account.owner = '/Ivan/.test'",
			resultsLength: 0,
		},
		" Match range with special character 2": {
			q:             "account.number <= 2 AND account.owner = '/Ivan/.test' AND tx.height > 0",
			resultsLength: 1,
		},
		" Match range with contains with multiple items": {
			q:             "account.number <= 2 AND account.owner CONTAINS '/Iv' AND account.owner CONTAINS 'an' AND tx.height = 1",
			resultsLength: 1,
		},
		" Match range with contains": {
			q:             "account.number <= 2 AND account.owner CONTAINS 'an' AND tx.height > 0",
			resultsLength: 1,
		},
	}

	ctx := context.Background()

	for _, tc := range testCases {
		t.Run(tc.q, func(t *testing.T) {
			results, _, err := indexer.Search(ctx, query.MustCompile(tc.q), DefaultPagination)
			require.NoError(t, err)

			assert.Len(t, results, tc.resultsLength)
			if tc.resultsLength > 0 {
				for _, txr := range results {
					assert.True(t, proto.Equal(txResult, txr))
				}
			}
		})
	}
}

func TestTxSearchEventMatchByHeight(t *testing.T) {
	indexer := NewTxIndex(db.NewMemDB())

	txResult := txResultWithEvents([]abci.Event{
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "number", Value: "1", Index: true}, {Key: "owner", Value: "Ana", Index: true}}},
	})

	err := indexer.Index(txResult)
	require.NoError(t, err)

	txResult10 := txResultWithEvents([]abci.Event{
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "number", Value: "1", Index: true}, {Key: "owner", Value: "/Ivan/.test", Index: true}}},
	})
	txResult10.Tx = types.Tx("HELLO WORLD 10")
	txResult10.Height = 10

	err = indexer.Index(txResult10)
	require.NoError(t, err)

	testCases := map[string]struct {
		q             string
		resultsLength int
	}{
		"Return all events from a height 1": {
			q:             "tx.height = 1",
			resultsLength: 1,
		},
		"Return all events from a height 10": {
			q:             "tx.height = 10",
			resultsLength: 1,
		},
		"Return all events from a height 5": {
			q:             "tx.height = 5",
			resultsLength: 0,
		},
		"Return all events from a height in [2; 5]": {
			q:             "tx.height >= 2 AND tx.height <= 5",
			resultsLength: 0,
		},
		"Return all events from a height in [1; 5]": {
			q:             "tx.height >= 1 AND tx.height <= 5",
			resultsLength: 1,
		},
		"Return all events from a height in [1; 10]": {
			q:             "tx.height >= 1 AND tx.height <= 10",
			resultsLength: 2,
		},
		"Return all events from a height in [1; 5] by account.number": {
			q:             "tx.height >= 1 AND tx.height <= 5 AND account.number=1",
			resultsLength: 1,
		},
		"Return all events from a height in [1; 10] by account.number 2": {
			q:             "tx.height >= 1 AND tx.height <= 10 AND account.number=1",
			resultsLength: 2,
		},
	}

	ctx := context.Background()

	for _, tc := range testCases {
		t.Run(tc.q, func(t *testing.T) {
			results, _, err := indexer.Search(ctx, query.MustCompile(tc.q), DefaultPagination)
			require.NoError(t, err)

			assert.Len(t, results, tc.resultsLength)
			if tc.resultsLength > 0 {
				for _, txr := range results {
					if txr.Height == 1 {
						assert.True(t, proto.Equal(txResult, txr))
					} else if txr.Height == 10 {
						assert.True(t, proto.Equal(txResult10, txr))
					} else {
						assert.True(t, false)
					}
				}
			}
		})
	}
}

func TestTxSearchWithCancelation(t *testing.T) {
	indexer := NewTxIndex(db.NewMemDB())

	txResult := txResultWithEvents([]abci.Event{
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "number", Value: "1", Index: true}}},
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "owner", Value: "Ivan", Index: true}}},
		{Type: "", Attributes: []abci.EventAttribute{{Key: "not_allowed", Value: "Vlad", Index: true}}},
	})
	err := indexer.Index(txResult)
	require.NoError(t, err)

	ctx, cancel := context.WithCancel(context.Background())
	cancel()
	results, _, err := indexer.Search(ctx, query.MustCompile(`account.number = 1`), DefaultPagination)
	require.NoError(t, err)
	assert.Empty(t, results)
}

func TestTxSearchDeprecatedIndexing(t *testing.T) {
	indexer := NewTxIndex(db.NewMemDB())

	// index tx using events indexing (composite key)
	txResult1 := txResultWithEvents([]abci.Event{
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "number", Value: "1", Index: true}}},
	})
	hash1 := types.Tx(txResult1.Tx).Hash()

	err := indexer.Index(txResult1)
	require.NoError(t, err)

	// index tx also using deprecated indexing (event as key)
	txResult2 := txResultWithEvents(nil)
	txResult2.Tx = types.Tx("HELLO WORLD 2")

	hash2 := types.Tx(txResult2.Tx).Hash()
	b := indexer.store.NewBatch()

	rawBytes, err := proto.Marshal(txResult2)
	require.NoError(t, err)

	depKey := []byte(fmt.Sprintf("%s/%s/%d/%d",
		"sender",
		"addr1",
		txResult2.Height,
		txResult2.Index,
	))

	err = b.Set(depKey, hash2)
	require.NoError(t, err)
	err = b.Set(keyForHeight(txResult2), hash2)
	require.NoError(t, err)
	err = b.Set(hash2, rawBytes)
	require.NoError(t, err)
	err = b.Write()
	require.NoError(t, err)

	testCases := []struct {
		q       string
		results []*abci.TxResult
	}{
		// search by hash
		{fmt.Sprintf("tx.hash = '%X'", hash1), []*abci.TxResult{txResult1}},
		// search by hash
		{fmt.Sprintf("tx.hash = '%X'", hash2), []*abci.TxResult{txResult2}},
		// search by exact match (one key)
		{"account.number = 1", []*abci.TxResult{txResult1}},
		{"account.number >= 1 AND account.number <= 5", []*abci.TxResult{txResult1}},
		// search by range (lower bound)
		{"account.number >= 1", []*abci.TxResult{txResult1}},
		// search by range (upper bound)
		{"account.number <= 5", []*abci.TxResult{txResult1}},
		// search using not allowed key
		{"not_allowed = 'boom'", []*abci.TxResult{}},
		// search for not existing tx result
		{"account.number >= 2 AND account.number <= 5", []*abci.TxResult{}},
		// search using not existing key
		{"account.date >= TIME 2013-05-03T14:45:00Z", []*abci.TxResult{}},
		// search by deprecated key
		{"sender = 'addr1'", []*abci.TxResult{txResult2}},
	}

	ctx := context.Background()

	for _, tc := range testCases {
		t.Run(tc.q, func(t *testing.T) {
			results, _, err := indexer.Search(ctx, query.MustCompile(tc.q), DefaultPagination)
			require.NoError(t, err)
			for _, txr := range results {
				for _, tr := range tc.results {
					assert.True(t, proto.Equal(tr, txr))
				}
			}
		})
	}
}

func TestTxSearchOneTxWithMultipleSameTagsButDifferentValues(t *testing.T) {
	indexer := NewTxIndex(db.NewMemDB())

	txResult := txResultWithEvents([]abci.Event{
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "number", Value: "1", Index: true}}},
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "number", Value: "2", Index: true}}},
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "number", Value: "3", Index: false}}},
	})

	err := indexer.Index(txResult)
	require.NoError(t, err)

	testCases := []struct {
		name  string
		q     string
		found bool
	}{
		{
			q:     "account.number >= 1",
			found: true,
		},
		{
			q:     "account.number > 2",
			found: false,
		},
		{
			q:     "account.number >= 1 AND tx.height = 3 AND tx.height > 0",
			found: true,
		},
		{
			q:     "account.number >= 1 AND tx.height > 0 AND tx.height = 3",
			found: true,
		},

		{
			q:     "account.number >= 1 AND tx.height = 1  AND tx.height = 2 AND tx.height = 3",
			found: true,
		},

		{
			q:     "account.number >= 1 AND tx.height = 3  AND tx.height = 2 AND tx.height = 1",
			found: false,
		},
		{
			q:     "account.number >= 1 AND tx.height = 3",
			found: false,
		},
		{
			q:     "account.number > 1 AND tx.height < 2",
			found: true,
		},
		{
			q:     "account.number >= 2",
			found: true,
		},
		{
			q:     "account.number <= 1",
			found: true,
		},
		{
			q:     "account.number = 'something'",
			found: false,
		},
		{
			q:     "account.number CONTAINS 'bla'",
			found: false,
		},
	}

	ctx := context.Background()

	for _, tc := range testCases {
		results, _, err := indexer.Search(ctx, query.MustCompile(tc.q), DefaultPagination)
		require.NoError(t, err)
		n := 0
		if tc.found {
			n = 1
		}
		assert.Len(t, results, n)
		assert.True(t, !tc.found || proto.Equal(txResult, results[0]))
	}
}

func TestTxIndexDuplicatePreviouslySuccessful(t *testing.T) {
	mockTx := types.Tx("MOCK_TX_HASH")

	testCases := []struct {
		name         string
		tx1          *abci.TxResult
		tx2          *abci.TxResult
		expOverwrite bool // do we expect the second tx to overwrite the first tx
	}{
		{
			"don't overwrite as a non-zero code was returned and the previous tx was successful",
			&abci.TxResult{
				Height: 1,
				Index:  0,
				Tx:     mockTx,
				Result: abci.ExecTxResult{
					Code: abci.CodeTypeOK,
				},
			},
			&abci.TxResult{
				Height: 2,
				Index:  0,
				Tx:     mockTx,
				Result: abci.ExecTxResult{
					Code: abci.CodeTypeOK + 1,
				},
			},
			false,
		},
		{
			"overwrite as the previous tx was also unsuccessful",
			&abci.TxResult{
				Height: 1,
				Index:  0,
				Tx:     mockTx,
				Result: abci.ExecTxResult{
					Code: abci.CodeTypeOK + 1,
				},
			},
			&abci.TxResult{
				Height: 2,
				Index:  0,
				Tx:     mockTx,
				Result: abci.ExecTxResult{
					Code: abci.CodeTypeOK + 1,
				},
			},
			true,
		},
		{
			"overwrite as the most recent tx was successful",
			&abci.TxResult{
				Height: 1,
				Index:  0,
				Tx:     mockTx,
				Result: abci.ExecTxResult{
					Code: abci.CodeTypeOK,
				},
			},
			&abci.TxResult{
				Height: 2,
				Index:  0,
				Tx:     mockTx,
				Result: abci.ExecTxResult{
					Code: abci.CodeTypeOK,
				},
			},
			true,
		},
	}

	hash := mockTx.Hash()

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			indexer := NewTxIndex(db.NewMemDB())

			// index the first tx
			err := indexer.Index(tc.tx1)
			require.NoError(t, err)

			// index the same tx with different results
			err = indexer.Index(tc.tx2)
			require.NoError(t, err)

			res, err := indexer.Get(hash)
			require.NoError(t, err)

			if tc.expOverwrite {
				require.Equal(t, tc.tx2, res)
			} else {
				require.Equal(t, tc.tx1, res)
			}
		})
	}
}

func TestTxSearchMultipleTxs(t *testing.T) {
	indexer := NewTxIndex(db.NewMemDB())

	// indexed first, but bigger height (to test the order of transactions)
	txResult := txResultWithEvents([]abci.Event{
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "number", Value: "1", Index: true}}},
	})

	txResult.Tx = types.Tx("Bob's account")
	txResult.Height = 2
	txResult.Index = 1
	err := indexer.Index(txResult)
	require.NoError(t, err)

	// indexed second, but smaller height (to test the order of transactions)
	txResult2 := txResultWithEvents([]abci.Event{
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "number", Value: "2", Index: true}}},
	})
	txResult2.Tx = types.Tx("Alice's account")
	txResult2.Height = 1
	txResult2.Index = 2

	err = indexer.Index(txResult2)
	require.NoError(t, err)

	// indexed third (to test the order of transactions)
	txResult3 := txResultWithEvents([]abci.Event{
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "number", Value: "3", Index: true}}},
	})
	txResult3.Tx = types.Tx("Jack's account")
	txResult3.Height = 1
	txResult3.Index = 1
	err = indexer.Index(txResult3)
	require.NoError(t, err)

	// indexed fourth (to test we don't include txs with similar events)
	// https://github.com/tendermint/tendermint/issues/2908
	txResult4 := txResultWithEvents([]abci.Event{
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "number.id", Value: "1", Index: true}}},
	})
	txResult4.Tx = types.Tx("Mike's account")
	txResult4.Height = 2
	txResult4.Index = 2
	err = indexer.Index(txResult4)
	require.NoError(t, err)

	ctx := context.Background()

	results, _, err := indexer.Search(ctx, query.MustCompile(`account.number >= 1`), DefaultPagination)
	require.NoError(t, err)

	require.Len(t, results, 3)
}

func txResultWithEvents(events []abci.Event) *abci.TxResult {
	tx := types.Tx("HELLO WORLD")
	return &abci.TxResult{
		Height: 1,
		Index:  0,
		Tx:     tx,
		Result: abci.ExecTxResult{
			Data:   []byte{0},
			Code:   abci.CodeTypeOK,
			Log:    "",
			Events: events,
		},
	}
}

func benchmarkTxIndex(b *testing.B, txsCount int64) {
	b.Helper()
	dir, err := os.MkdirTemp("", "tx_index_db")
	require.NoError(b, err)
	defer os.RemoveAll(dir)

	store, err := db.NewDB("tx_index", "goleveldb", dir)
	require.NoError(b, err)
	indexer := NewTxIndex(store)

	batch := txindex.NewBatch(txsCount)
	txIndex := uint32(0)
	for i := int64(0); i < txsCount; i++ {
		tx := cmtrand.Bytes(250)
		txResult := &abci.TxResult{
			Height: 1,
			Index:  txIndex,
			Tx:     tx,
			Result: abci.ExecTxResult{
				Data:   []byte{0},
				Code:   abci.CodeTypeOK,
				Log:    "",
				Events: []abci.Event{},
			},
		}
		if err := batch.Add(txResult); err != nil {
			b.Fatal(err)
		}
		txIndex++
	}

	b.ResetTimer()

	for n := 0; n < b.N; n++ {
		err = indexer.AddBatch(batch)
	}
	if err != nil {
		b.Fatal(err)
	}
}

func TestBigInt(t *testing.T) {
	indexer := NewTxIndex(db.NewMemDB())

	bigInt := "10000000000000000000"
	bigIntPlus1 := "10000000000000000001"
	bigFloat := bigInt + ".76"
	bigFloatLower := bigInt + ".1"
	bigFloatSmaller := "9999999999999999999" + ".1"
	bigIntSmaller := "9999999999999999999"

	txResult := txResultWithEvents([]abci.Event{
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "number", Value: bigInt, Index: true}}},
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "number", Value: bigFloatSmaller, Index: true}}},
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "number", Value: bigIntPlus1, Index: true}}},
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "number", Value: bigFloatLower, Index: true}}},
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "owner", Value: "/Ivan/", Index: true}}},
		{Type: "", Attributes: []abci.EventAttribute{{Key: "not_allowed", Value: "Vlad", Index: true}}},
	})
	hash := types.Tx(txResult.Tx).Hash()

	err := indexer.Index(txResult)

	require.NoError(t, err)

	txResult2 := txResultWithEvents([]abci.Event{
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "number", Value: bigFloat, Index: true}}},
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "number", Value: bigFloat, Index: true}, {Key: "amount", Value: "5", Index: true}}},
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "number", Value: bigIntSmaller, Index: true}}},
		{Type: "account", Attributes: []abci.EventAttribute{{Key: "number", Value: bigInt, Index: true}, {Key: "amount", Value: "3", Index: true}}},
	})

	txResult2.Tx = types.Tx("NEW TX")
	txResult2.Height = 2
	txResult2.Index = 2

	hash2 := types.Tx(txResult2.Tx).Hash()

	err = indexer.Index(txResult2)
	require.NoError(t, err)
	testCases := []struct {
		q             string
		txRes         *abci.TxResult
		resultsLength int
	}{
		//	search by hash
		{fmt.Sprintf("tx.hash = '%X'", hash), txResult, 1},
		// search by hash (lower)
		{fmt.Sprintf("tx.hash = '%x'", hash), txResult, 1},
		{fmt.Sprintf("tx.hash = '%x'", hash2), txResult2, 1},
		// search by exact match (one key) - bigint
		{"account.number >= " + bigInt, nil, 2},
		// search by exact match (one key) - bigint range
		{"account.number >= " + bigInt + " AND tx.height > 0", nil, 2},
		{"account.number >= " + bigInt + " AND tx.height > 0 AND account.owner = '/Ivan/'", nil, 0},
		// Floats are not parsed
		{"account.number >= " + bigInt + " AND tx.height > 0 AND account.amount > 4", txResult2, 1},
		{"account.number >= " + bigInt + " AND tx.height > 0 AND account.amount = 5", txResult2, 1},
		{"account.number >= " + bigInt + " AND account.amount <= 5", txResult2, 1},
		{"account.number > " + bigFloatSmaller + " AND account.amount = 3", txResult2, 1},
		{"account.number < " + bigInt + " AND tx.height >= 1", nil, 2},
		{"account.number < " + bigInt + " AND tx.height = 1", nil, 1},
		{"account.number < " + bigInt + " AND tx.height = 2", nil, 1},
	}

	ctx := context.Background()

	for _, tc := range testCases {
		t.Run(tc.q, func(t *testing.T) {
			results, _, err := indexer.Search(ctx, query.MustCompile(tc.q), DefaultPagination)
			require.NoError(t, err)
			assert.Len(t, results, tc.resultsLength)
			if tc.resultsLength > 0 && tc.txRes != nil {
				assert.True(t, proto.Equal(results[0], tc.txRes))
			}
		})
	}
}

func BenchmarkTxIndex(b *testing.B) {
	testCases := []struct {
		name     string
		txsCount int64
	}{
		{"1", 1},
		{"500", 500},
		{"1000", 1000},
		{"2000", 2000},
		{"10000", 10000},
	}

	for _, tc := range testCases {
		b.Run(tc.name, func(b *testing.B) {
			benchmarkTxIndex(b, tc.txsCount)
		})
	}
}

func isSubset(smaller [][]byte, bigger [][]byte) bool {
	for _, elem := range smaller {
		if !slices.ContainsFunc(bigger, func(i []byte) bool {
			return bytes.Equal(i, elem)
		}) {
			return false
		}
	}
	return true
}

func isEqualSets(x [][]byte, y [][]byte) bool {
	return isSubset(x, y) && isSubset(y, x)
}

func emptyIntersection(x [][]byte, y [][]byte) bool {
	for _, elem := range x {
		if slices.ContainsFunc(y, func(i []byte) bool {
			return bytes.Equal(i, elem)
		}) {
			return false
		}
	}
	return true
}

func setDiff(bigger [][]byte, smaller [][]byte) [][]byte {
	var diff [][]byte
	for _, elem := range bigger {
		if !slices.ContainsFunc(smaller, func(i []byte) bool {
			return bytes.Equal(i, elem)
		}) {
			diff = append(diff, elem)
		}
	}
	return diff
}

func TestExtractEventSeqFromKey(t *testing.T) {
	testCases := []struct {
		str      string
		expected string
	}{
		{
			"0/0/0/1234$es$0",
			"0",
		},
		{
			"0/0/0/1234$es$1234",
			"1234",
		},
		{
			"0/0/0/1234",
			"0",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.expected, func(t *testing.T) {
			assert.Equal(t, extractEventSeqFromKey([]byte(tc.str)), tc.expected)
		})
	}
}

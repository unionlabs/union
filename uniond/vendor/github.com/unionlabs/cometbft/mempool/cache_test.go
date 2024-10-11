package mempool

import (
	"crypto/rand"
	"crypto/sha256"
	"strconv"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/abci/example/kvstore"
	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/proxy"
	"github.com/cometbft/cometbft/types"
)

func TestCacheRemove(t *testing.T) {
	cache := NewLRUTxCache(100)
	numTxs := 10

	txs := make([][]byte, numTxs)
	for i := 0; i < numTxs; i++ {
		// probability of collision is 2**-256
		txBytes := make([]byte, 32)
		_, err := rand.Read(txBytes)
		require.NoError(t, err)

		txs[i] = txBytes
		cache.Push(txBytes)

		// make sure its added to both the linked list and the map
		require.Len(t, cache.cacheMap, i+1)
		require.Equal(t, i+1, cache.list.Len())
	}

	for i := 0; i < numTxs; i++ {
		cache.Remove(txs[i])
		// make sure its removed from both the map and the linked list
		require.Len(t, cache.cacheMap, numTxs-(i+1))
		require.Equal(t, numTxs-(i+1), cache.list.Len())
	}
}

func TestCacheAfterUpdate(t *testing.T) {
	app := kvstore.NewInMemoryApplication()
	cc := proxy.NewLocalClientCreator(app)
	mp, cleanup := newMempoolWithApp(cc)
	defer cleanup()

	// reAddIndices & txsInCache can have elements > numTxsToCreate
	// also assumes max index is 255 for convenience
	// txs in cache also checks order of elements
	tests := []struct {
		numTxsToCreate int
		updateIndices  []int
		reAddIndices   []int
		txsInCache     []int
	}{
		{1, []int{}, []int{1}, []int{1, 0}},    // adding new txs works
		{2, []int{1}, []int{}, []int{1, 0}},    // update doesn't remove tx from cache
		{2, []int{2}, []int{}, []int{2, 1, 0}}, // update adds new tx to cache
		{2, []int{1}, []int{1}, []int{1, 0}},   // re-adding after update doesn't make dupe
	}
	for tcIndex, tc := range tests {
		for i := 0; i < tc.numTxsToCreate; i++ {
			tx := kvstore.NewTx(strconv.Itoa(i), "value")
			reqRes, err := mp.CheckTx(tx, "")
			require.NoError(t, err)
			require.False(t, reqRes.Response.GetCheckTx().IsErr())
		}

		updateTxs := []types.Tx{}
		for _, v := range tc.updateIndices {
			tx := kvstore.NewTx(strconv.Itoa(v), "value")
			updateTxs = append(updateTxs, tx)
		}
		err := mp.Update(int64(tcIndex), updateTxs, abciResponses(len(updateTxs), abci.CodeTypeOK), nil, nil)
		require.NoError(t, err)

		for _, v := range tc.reAddIndices {
			tx := kvstore.NewTx(strconv.Itoa(v), "value")
			reqRes, err := mp.CheckTx(tx, "")
			if err == nil {
				require.False(t, reqRes.Response.GetCheckTx().IsErr())
			}
		}

		cache := mp.cache.(*LRUTxCache)
		node := cache.GetList().Front()
		counter := 0
		for node != nil {
			require.NotEqual(t, len(tc.txsInCache), counter,
				"cache larger than expected on testcase %d", tcIndex)

			nodeVal := node.Value.(types.TxKey)
			expTx := kvstore.NewTx(strconv.Itoa(tc.txsInCache[len(tc.txsInCache)-counter-1]), "value")
			expectedBz := sha256.Sum256(expTx)
			// Reference for reading the errors:
			// >>> sha256('\x00').hexdigest()
			// '6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d'
			// >>> sha256('\x01').hexdigest()
			// '4bf5122f344554c53bde2ebb8cd2b7e3d1600ad631c385a5d7cce23c7785459a'
			// >>> sha256('\x02').hexdigest()
			// 'dbc1b4c900ffe48d575b5da5c638040125f65db0fe3e24494b76ea986457d986'

			require.EqualValues(t, expectedBz, nodeVal, "Equality failed on index %d, tc %d", counter, tcIndex)
			counter++
			node = node.Next()
		}
		require.Len(t, tc.txsInCache, counter,
			"cache smaller than expected on testcase %d", tcIndex)
		mp.Flush()
	}
}

package kv

import (
	"bytes"
	"context"
	"encoding/hex"
	"errors"
	"fmt"
	"math"
	"math/big"
	"sort"
	"strconv"
	"strings"

	"github.com/cosmos/gogoproto/proto"

	dbm "github.com/cometbft/cometbft-db"
	abci "github.com/cometbft/cometbft/abci/types"
	idxutil "github.com/cometbft/cometbft/internal/indexer"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/libs/pubsub/query"
	"github.com/cometbft/cometbft/libs/pubsub/query/syntax"
	"github.com/cometbft/cometbft/state"
	"github.com/cometbft/cometbft/state/indexer"
	"github.com/cometbft/cometbft/state/txindex"
	"github.com/cometbft/cometbft/types"
)

const (
	tagKeySeparator          = "/"
	tagKeySeparatorRune      = '/'
	eventSeqSeparator        = "$es$"
	eventSeqSeperatorRuneAt0 = '$'
)

var (
	LastTxIndexerRetainHeightKey = []byte("LastTxIndexerRetainHeightKey")
	TxIndexerRetainHeightKey     = []byte("TxIndexerRetainHeightKey")
)

// TxIndex is the simplest possible indexer, backed by key-value storage (levelDB).
type TxIndex struct {
	store dbm.DB
	// Number the events in the event list
	eventSeq int64

	log log.Logger

	compact            bool
	compactionInterval int64
	lastPruned         int64
}

type IndexerOption func(*TxIndex)

// WithCompaction sets the compaciton parameters.
func WithCompaction(compact bool, compactionInterval int64) IndexerOption {
	return func(txi *TxIndex) {
		txi.compact = compact
		txi.compactionInterval = compactionInterval
	}
}

func (txi *TxIndex) Prune(retainHeight int64) (numPruned int64, newRetainHeight int64, err error) {
	// Returns numPruned, newRetainHeight, err
	// numPruned: the number of heights pruned. E.x. if heights {1, 3, 7} were pruned, numPruned == 3
	// newRetainHeight: new retain height after pruning
	// err: error

	lastRetainHeight, err := txi.getIndexerRetainHeight()
	if err != nil {
		return 0, 0, fmt.Errorf("failed to look up last block indexer retain height: %w", err)
	}
	if lastRetainHeight == 0 {
		lastRetainHeight = 1
	}

	ctx := context.Background()
	results, _, err := txi.Search(ctx, query.MustCompile(
		fmt.Sprintf("tx.height < %d AND tx.height >= %d", retainHeight, lastRetainHeight)), txindex.Pagination{})
	if err != nil {
		return 0, lastRetainHeight, err
	}
	if len(results) == 0 {
		return 0, lastRetainHeight, nil
	}

	batch := txi.store.NewBatch()
	closeBatch := func(batch dbm.Batch) {
		err := batch.Close()
		if err != nil {
			txi.log.Error(fmt.Sprintf("Error when closing tx indexer pruning batch: %v", err))
		}
	}
	defer closeBatch(batch)
	pruned := uint64(0)
	flush := func(batch dbm.Batch) error {
		err := batch.WriteSync()
		if err != nil {
			return fmt.Errorf("failed to flush tx indexer pruning batch %w", err)
		}
		err = batch.Close()
		if err != nil {
			txi.log.Error(fmt.Sprintf("Error when closing tx indexer pruning batch: %v", err))
		}
		return nil
	}

	sort.Sort(TxResultByHeight(results))
	numHeightsBatchPruned := int64(0)                     // number of heights pruned if counting batched
	currentBatchRetainedHeight := results[0].Height       // height retained if counting batched
	numHeightsPersistentlyPruned := int64(0)              // number of heights pruned persistently
	currentPersistentlyRetainedHeight := lastRetainHeight // height retained persistently
	for i, result := range results {
		errDeleteResult := txi.deleteResult(result, batch)
		if errDeleteResult != nil {
			// If we crashed in the middle of pruning the height,
			// we assume this height is retained
			errSetLastRetainHeight := txi.setIndexerRetainHeight(result.Height, batch)
			if errSetLastRetainHeight != nil {
				return 0, lastRetainHeight, fmt.Errorf("error setting last retain height '%v' while handling result deletion error '%v' for tx indexer", errSetLastRetainHeight, errDeleteResult)
			}
			errWriteBatch := batch.WriteSync()
			if errWriteBatch != nil {
				return 0, lastRetainHeight, fmt.Errorf("error writing tx indexer batch '%v' while handling result deletion error '%v'", errWriteBatch, errDeleteResult)
			}
			return result.Height - lastRetainHeight, result.Height, errDeleteResult
		}
		if i == len(results)-1 || results[i+1].Height > result.Height {
			numHeightsBatchPruned++
			currentBatchRetainedHeight = result.Height + 1
		}
		// flush every 1000 blocks to avoid batches becoming too large
		if pruned%1000 == 0 && pruned > 0 {
			err := flush(batch)
			if err != nil {
				return numHeightsPersistentlyPruned, currentPersistentlyRetainedHeight, err
			}
			numHeightsPersistentlyPruned = numHeightsBatchPruned
			currentPersistentlyRetainedHeight = currentBatchRetainedHeight
			batch = txi.store.NewBatch()
			defer closeBatch(batch)
		}
	}

	err = flush(batch)
	if err != nil {
		return numHeightsPersistentlyPruned, currentPersistentlyRetainedHeight, err
	}
	numHeightsPersistentlyPruned = numHeightsBatchPruned
	currentPersistentlyRetainedHeight = currentBatchRetainedHeight

	txi.lastPruned += numHeightsBatchPruned
	if txi.compact && txi.lastPruned >= txi.compactionInterval {
		err = txi.store.Compact(nil, nil)
		txi.lastPruned = 0
	}

	return numHeightsPersistentlyPruned, currentPersistentlyRetainedHeight, err
}

func (txi *TxIndex) SetRetainHeight(retainHeight int64) error {
	return txi.store.SetSync(TxIndexerRetainHeightKey, int64ToBytes(retainHeight))
}

func (txi *TxIndex) GetRetainHeight() (int64, error) {
	buf, err := txi.store.Get(TxIndexerRetainHeightKey)
	if err != nil {
		return 0, err
	}
	if buf == nil {
		return 0, state.ErrKeyNotFound
	}
	height := int64FromBytes(buf)

	if height < 0 {
		return 0, state.ErrInvalidHeightValue
	}

	return height, nil
}

func (*TxIndex) setIndexerRetainHeight(height int64, batch dbm.Batch) error {
	return batch.Set(LastTxIndexerRetainHeightKey, int64ToBytes(height))
}

func (txi *TxIndex) getIndexerRetainHeight() (int64, error) {
	bz, err := txi.store.Get(LastTxIndexerRetainHeightKey)
	if errors.Is(err, state.ErrKeyNotFound) {
		return 0, nil
	}
	height := int64FromBytes(bz)
	if height < 0 {
		return 0, state.ErrInvalidHeightValue
	}
	return height, nil
}

// NewTxIndex creates new KV indexer.
func NewTxIndex(store dbm.DB, options ...IndexerOption) *TxIndex {
	txIndex := &TxIndex{
		store: store,
	}

	for _, option := range options {
		option(txIndex)
	}

	return txIndex
}

func (txi *TxIndex) SetLogger(l log.Logger) {
	txi.log = l
}

// Get gets transaction from the TxIndex storage and returns it or nil if the
// transaction is not found.
func (txi *TxIndex) Get(hash []byte) (*abci.TxResult, error) {
	if len(hash) == 0 {
		return nil, txindex.ErrorEmptyHash
	}

	rawBytes, err := txi.store.Get(hash)
	if err != nil {
		panic(err)
	}
	if rawBytes == nil {
		return nil, nil
	}

	txResult := new(abci.TxResult)
	err = proto.Unmarshal(rawBytes, txResult)
	if err != nil {
		return nil, fmt.Errorf("error reading TxResult: %v", err)
	}

	return txResult, nil
}

// AddBatch indexes a batch of transactions using the given list of events. Each
// key that indexed from the tx's events is a composite of the event type and
// the respective attribute's key delimited by a "." (eg. "account.number").
// Any event with an empty type is not indexed.
func (txi *TxIndex) AddBatch(b *txindex.Batch) error {
	storeBatch := txi.store.NewBatch()
	defer storeBatch.Close()

	for _, result := range b.Ops {
		hash := types.Tx(result.Tx).Hash()

		// index tx by events
		err := txi.indexEvents(result, hash, storeBatch)
		if err != nil {
			return err
		}

		// index by height (always)
		err = storeBatch.Set(keyForHeight(result), hash)
		if err != nil {
			return err
		}

		rawBytes, err := proto.Marshal(result)
		if err != nil {
			return err
		}
		// index by hash (always)
		err = storeBatch.Set(hash, rawBytes)
		if err != nil {
			return err
		}
	}

	return storeBatch.WriteSync()
}

func (txi *TxIndex) deleteResult(result *abci.TxResult, batch dbm.Batch) error {
	hash := types.Tx(result.Tx).Hash()
	err := txi.deleteEvents(result, batch)
	if err != nil {
		return err
	}
	err = batch.Delete(keyForHeight(result))
	if err != nil {
		return err
	}
	err = batch.Delete(hash)
	if err != nil {
		return err
	}
	return nil
}

// Index indexes a single transaction using the given list of events. Each key
// that indexed from the tx's events is a composite of the event type and the
// respective attribute's key delimited by a "." (eg. "account.number").
// Any event with an empty type is not indexed.
//
// If a transaction is indexed with the same hash as a previous transaction, it will
// be overwritten unless the tx result was NOT OK and the prior result was OK i.e.
// more transactions that successfully executed overwrite transactions that failed
// or successful yet older transactions.
func (txi *TxIndex) Index(result *abci.TxResult) error {
	b := txi.store.NewBatch()
	defer b.Close()

	hash := types.Tx(result.Tx).Hash()

	if !result.Result.IsOK() {
		oldResult, err := txi.Get(hash)
		if err != nil {
			return err
		}

		// if the new transaction failed and it's already indexed in an older block and was successful
		// we skip it as we want users to get the older successful transaction when they query.
		if oldResult != nil && oldResult.Result.Code == abci.CodeTypeOK {
			return nil
		}
	}

	// index tx by events
	err := txi.indexEvents(result, hash, b)
	if err != nil {
		return err
	}

	// index by height (always)
	err = b.Set(keyForHeight(result), hash)
	if err != nil {
		return err
	}

	rawBytes, err := proto.Marshal(result)
	if err != nil {
		return err
	}
	// index by hash (always)
	err = b.Set(hash, rawBytes)
	if err != nil {
		return err
	}

	return b.WriteSync()
}

func (txi *TxIndex) deleteEvents(result *abci.TxResult, batch dbm.Batch) error {
	for _, event := range result.Result.Events {
		// only delete events with a non-empty type
		if len(event.Type) == 0 {
			continue
		}

		for _, attr := range event.Attributes {
			if len(attr.Key) == 0 {
				continue
			}

			compositeTag := fmt.Sprintf("%s.%s", event.Type, attr.Key)
			if attr.GetIndex() {
				zeroKey := keyForEvent(compositeTag, attr.Value, result, 0)
				endKey := keyForEvent(compositeTag, attr.Value, result, math.MaxInt64)
				itr, err := txi.store.Iterator(zeroKey, endKey)
				if err != nil {
					return err
				}
				for ; itr.Valid(); itr.Next() {
					err := batch.Delete(itr.Key())
					if err != nil {
						return err
					}
				}
			}
		}
	}
	return nil
}

func (txi *TxIndex) indexEvents(result *abci.TxResult, hash []byte, store dbm.Batch) error {
	for _, event := range result.Result.Events {
		txi.eventSeq++
		// only index events with a non-empty type
		if len(event.Type) == 0 {
			continue
		}

		for _, attr := range event.Attributes {
			if len(attr.Key) == 0 {
				continue
			}

			// index if `index: true` is set
			compositeTag := fmt.Sprintf("%s.%s", event.Type, attr.Key)
			// ensure event does not conflict with a reserved prefix key
			if compositeTag == types.TxHashKey || compositeTag == types.TxHeightKey {
				return fmt.Errorf("event type and attribute key \"%s\" is reserved; please use a different key", compositeTag)
			}
			if attr.GetIndex() {
				err := store.Set(keyForEvent(compositeTag, attr.Value, result, txi.eventSeq), hash)
				if err != nil {
					return err
				}
			}
		}
	}

	return nil
}

type hashKey struct {
	hash   string
	height int64
}

type hashKeySorter struct {
	keys []hashKey
	by   func(t1, t2 *hashKey) bool
}

func (t *hashKeySorter) Less(i, j int) bool { return t.by(&t.keys[i], &t.keys[j]) }
func (t *hashKeySorter) Len() int           { return len(t.keys) }
func (t *hashKeySorter) Swap(i, j int)      { t.keys[i], t.keys[j] = t.keys[j], t.keys[i] }

func byHeightDesc(i, j *hashKey) bool {
	hi := i.height
	hj := j.height
	if hi == hj {
		return i.hash > j.hash
	}
	return hi > hj
}

func byHeightAsc(i, j *hashKey) bool {
	hi := i.height
	hj := j.height
	if hi == hj {
		return i.hash < j.hash
	}
	return hi < hj
}

// Search performs a search using the given query.
//
// It breaks the query into conditions (like "tx.height > 5"). For each
// condition, it queries the DB index. One special use cases here: (1) if
// "tx.hash" is found, it returns tx result for it (2) for range queries it is
// better for the client to provide both lower and upper bounds, so we are not
// performing a full scan. Results from querying indexes are then intersected
// and returned to the caller, in no particular order.
//
// Search will exit early and return any result fetched so far,
// when a message is received on the context chan.
func (txi *TxIndex) Search(ctx context.Context, q *query.Query, pagSettings txindex.Pagination) ([]*abci.TxResult, int, error) {
	select {
	case <-ctx.Done():
		return make([]*abci.TxResult, 0), 0, nil

	default:
	}

	var hashesInitialized bool
	filteredHashes := make(map[string]TxInfo)

	// get a list of conditions (like "tx.height > 5")
	conditions := q.Syntax()

	// if there is a hash condition, return the result immediately
	hash, ok, err := lookForHash(conditions)
	if err != nil {
		return nil, 0, fmt.Errorf("error during searching for a hash in the query: %w", err)
	} else if ok {
		res, err := txi.Get(hash)
		switch {
		case err != nil:
			return []*abci.TxResult{}, 0, fmt.Errorf("error while retrieving the result: %w", err)
		case res == nil:
			return []*abci.TxResult{}, 0, nil
		default:
			return []*abci.TxResult{res}, 0, nil
		}
	}

	// conditions to skip because they're handled before "everything else"
	skipIndexes := make([]int, 0)
	var heightInfo HeightInfo

	// If we are not matching events and tx.height = 3 occurs more than once, the later value will
	// overwrite the first one.
	conditions, heightInfo = dedupHeight(conditions)

	if !heightInfo.onlyHeightEq {
		skipIndexes = append(skipIndexes, heightInfo.heightEqIdx)
	}

	// extract ranges
	// if both upper and lower bounds exist, it's better to get them in order not
	// no iterate over kvs that are not within range.
	ranges, rangeIndexes, heightRange := indexer.LookForRangesWithHeight(conditions)
	heightInfo.heightRange = heightRange
	if len(ranges) > 0 {
		skipIndexes = append(skipIndexes, rangeIndexes...)

		for _, qr := range ranges {
			// If we have a query range over height and want to still look for
			// specific event values we do not want to simply return all
			// transactios in this height range. We remember the height range info
			// and pass it on to match() to take into account when processing events.
			if qr.Key == types.TxHeightKey && !heightInfo.onlyHeightRange {
				continue
			}
			if !hashesInitialized {
				filteredHashes = txi.matchRange(ctx, qr, startKey(qr.Key), filteredHashes, true, heightInfo)
				hashesInitialized = true

				// Ignore any remaining conditions if the first condition resulted
				// in no matches (assuming implicit AND operand).
				if len(filteredHashes) == 0 {
					break
				}
			} else {
				filteredHashes = txi.matchRange(ctx, qr, startKey(qr.Key), filteredHashes, false, heightInfo)
			}
		}
	}

	// if there is a height condition ("tx.height=3"), extract it

	// for all other conditions
	for i, c := range conditions {
		if intInSlice(i, skipIndexes) {
			continue
		}

		if !hashesInitialized {
			filteredHashes = txi.match(ctx, c, startKeyForCondition(c, heightInfo.height), filteredHashes, true, heightInfo)
			hashesInitialized = true

			// Ignore any remaining conditions if the first condition resulted
			// in no matches (assuming implicit AND operand).
			if len(filteredHashes) == 0 {
				break
			}
		} else {
			filteredHashes = txi.match(ctx, c, startKeyForCondition(c, heightInfo.height), filteredHashes, false, heightInfo)
		}
	}

	numResults := len(filteredHashes)

	// Convert map keys to slice for deterministic ordering
	hashKeys := make([]hashKey, 0, numResults)
	for k, v := range filteredHashes {
		hashKeys = append(hashKeys, hashKey{hash: k, height: v.Height})
	}

	var by func(i, j *hashKey) bool

	if pagSettings.OrderDesc {
		by = byHeightDesc
	} else {
		by = byHeightAsc
	}

	// Sort by height
	sort.Sort(&hashKeySorter{
		keys: hashKeys,
		by:   by,
	})

	// If paginated, determine which hash keys to return
	if pagSettings.IsPaginated {
		// Now that we know the total number of results, validate that the page
		// requested is within bounds
		pagSettings.Page, err = validatePage(&pagSettings.Page, pagSettings.PerPage, numResults)
		if err != nil {
			return nil, 0, err
		}

		// Calculate pagination start and end indices
		startIndex := (pagSettings.Page - 1) * pagSettings.PerPage
		endIndex := startIndex + pagSettings.PerPage

		// Apply pagination limits
		if endIndex > len(hashKeys) {
			endIndex = len(hashKeys)
		}
		if startIndex >= len(hashKeys) {
			return []*abci.TxResult{}, 0, nil
		}

		hashKeys = hashKeys[startIndex:endIndex]
	}

	results := make([]*abci.TxResult, 0, len(hashKeys))
	resultMap := make(map[string]struct{})
RESULTS_LOOP:
	for _, hKey := range hashKeys {
		h := filteredHashes[hKey.hash].TxBytes
		res, err := txi.Get(h)
		if err != nil {
			return nil, 0, fmt.Errorf("failed to get Tx{%X}: %w", h, err)
		}
		hashString := string(h)
		if _, ok := resultMap[hashString]; !ok {
			resultMap[hashString] = struct{}{}
			results = append(results, res)
		}
		// Potentially exit early.
		select {
		case <-ctx.Done():
			break RESULTS_LOOP
		default:
		}
	}

	return results, numResults, nil
}

func lookForHash(conditions []syntax.Condition) (hash []byte, ok bool, err error) {
	for _, c := range conditions {
		if c.Tag == types.TxHashKey {
			decoded, err := hex.DecodeString(c.Arg.Value())
			return decoded, true, err
		}
	}
	return nil, false, nil
}

type TxInfo struct {
	TxBytes []byte
	Height  int64
}

func (*TxIndex) setTmpHashes(tmpHeights map[string]TxInfo, key, value []byte, height int64) {
	// value comes from cometbft-db Iterator interface Value() API.
	// Therefore, we must make a copy before storing references to it.
	valueCp := make([]byte, len(value))
	copy(valueCp, value)

	eventSeq := extractEventSeqFromKey(key)
	txInfo := TxInfo{
		TxBytes: valueCp,
		Height:  height,
	}
	tmpHeights[string(valueCp)+eventSeq] = txInfo
}

// match returns all matching txs by hash that meet a given condition and start
// key. An already filtered result (filteredHashes) is provided such that any
// non-intersecting matches are removed.
//
// NOTE: filteredHashes may be empty if no previous condition has matched.
//
// Additionally, this method retrieves the height of the hash via the key,
// and adds it to the TxInfo struct, which is then added to the filteredHashes.
// This is done to paginate the results prior to retrieving all the TxResults,
// which is needed for performance reasons.
func (txi *TxIndex) match(
	ctx context.Context,
	c syntax.Condition,
	startKeyBz []byte,
	filteredHashes map[string]TxInfo,
	firstRun bool,
	heightInfo HeightInfo,
) map[string]TxInfo {
	// A previous match was attempted but resulted in no matches, so we return
	// no matches (assuming AND operand).
	if !firstRun && len(filteredHashes) == 0 {
		return filteredHashes
	}

	tmpHashes := make(map[string]TxInfo)

	switch {
	case c.Op == syntax.TEq:
		it, err := dbm.IteratePrefix(txi.store, startKeyBz)
		if err != nil {
			panic(err)
		}
		defer it.Close()

	EQ_LOOP:
		for ; it.Valid(); it.Next() {
			// If we have a height range in a query, we need only transactions
			// for this height
			key := it.Key()
			keyHeight, err := extractHeightFromKey(key)
			if err != nil {
				txi.log.Error("failure to parse height from key:", err)
				continue
			}
			withinBounds, err := checkHeightConditions(heightInfo, keyHeight)
			if err != nil {
				txi.log.Error("failure checking for height bounds:", err)
				continue
			}
			if !withinBounds {
				continue
			}
			txi.setTmpHashes(tmpHashes, key, it.Value(), keyHeight)
			// Potentially exit early.
			select {
			case <-ctx.Done():
				break EQ_LOOP
			default:
			}
		}
		if err := it.Error(); err != nil {
			panic(err)
		}

	case c.Op == syntax.TExists:
		// XXX: can't use startKeyBz here because c.Operand is nil
		// (e.g. "account.owner/<nil>/" won't match w/ a single row)
		it, err := dbm.IteratePrefix(txi.store, startKey(c.Tag))
		if err != nil {
			panic(err)
		}
		defer it.Close()

	EXISTS_LOOP:
		for ; it.Valid(); it.Next() {
			key := it.Key()
			keyHeight, err := extractHeightFromKey(key)
			if err != nil {
				txi.log.Error("failure to parse height from key:", err)
				continue
			}
			withinBounds, err := checkHeightConditions(heightInfo, keyHeight)
			if err != nil {
				txi.log.Error("failure checking for height bounds:", err)
				continue
			}
			if !withinBounds {
				continue
			}
			txi.setTmpHashes(tmpHashes, key, it.Value(), keyHeight)

			// Potentially exit early.
			select {
			case <-ctx.Done():
				break EXISTS_LOOP
			default:
			}
		}
		if err := it.Error(); err != nil {
			panic(err)
		}

	case c.Op == syntax.TContains:
		// XXX: startKey does not apply here.
		// For example, if startKey = "account.owner/an/" and search query = "account.owner CONTAINS an"
		// we can't iterate with prefix "account.owner/an/" because we might miss keys like "account.owner/Ulan/"
		it, err := dbm.IteratePrefix(txi.store, startKey(c.Tag))
		if err != nil {
			panic(err)
		}
		defer it.Close()

	CONTAINS_LOOP:
		for ; it.Valid(); it.Next() {
			if !isTagKey(it.Key()) {
				continue
			}

			if strings.Contains(extractValueFromKey(it.Key()), c.Arg.Value()) {
				key := it.Key()
				keyHeight, err := extractHeightFromKey(key)
				if err != nil {
					txi.log.Error("failure to parse height from key:", err)
					continue
				}
				withinBounds, err := checkHeightConditions(heightInfo, keyHeight)
				if err != nil {
					txi.log.Error("failure checking for height bounds:", err)
					continue
				}
				if !withinBounds {
					continue
				}
				txi.setTmpHashes(tmpHashes, key, it.Value(), keyHeight)
			}

			// Potentially exit early.
			select {
			case <-ctx.Done():
				break CONTAINS_LOOP
			default:
			}
		}
		if err := it.Error(); err != nil {
			panic(err)
		}
	default:
		panic("other operators should be handled already")
	}

	if len(tmpHashes) == 0 || firstRun {
		// Either:
		//
		// 1. Regardless if a previous match was attempted, which may have had
		// results, but no match was found for the current condition, then we
		// return no matches (assuming AND operand).
		//
		// 2. A previous match was not attempted, so we return all results.
		return tmpHashes
	}

	// Remove/reduce matches in filteredHashes that were not found in this
	// match (tmpHashes).
REMOVE_LOOP:
	for k, v := range filteredHashes {
		tmpHash := tmpHashes[k]
		if tmpHash.TxBytes == nil || !bytes.Equal(tmpHash.TxBytes, v.TxBytes) {
			delete(filteredHashes, k)
		} else {
			// If there is a match, update the height in filteredHashes
			v.Height = tmpHash.Height
			filteredHashes[k] = v
		}
		// Potentially exit early.
		select {
		case <-ctx.Done():
			break REMOVE_LOOP
		default:
		}
	}

	return filteredHashes
}

// matchRange returns all matching txs by hash that meet a given queryRange and
// start key. An already filtered result (filteredHashes) is provided such that
// any non-intersecting matches are removed.
//
// NOTE: filteredHashes may be empty if no previous condition has matched.
//
// Additionally, this method retrieves the height of the hash via the key,
// and adds it to the TxInfo struct, which is then added to the filteredHashes.
// This is done to paginate the results prior to retrieving all the TxResults,
// which is needed for performance reasons.
func (txi *TxIndex) matchRange(
	ctx context.Context,
	qr indexer.QueryRange,
	startKey []byte,
	filteredHashes map[string]TxInfo,
	firstRun bool,
	heightInfo HeightInfo,
) map[string]TxInfo {
	// A previous match was attempted but resulted in no matches, so we return
	// no matches (assuming AND operand).
	if !firstRun && len(filteredHashes) == 0 {
		return filteredHashes
	}

	tmpHashes := make(map[string]TxInfo)

	it, err := dbm.IteratePrefix(txi.store, startKey)
	if err != nil {
		panic(err)
	}
	defer it.Close()
	bigIntValue := new(big.Int)

LOOP:
	for ; it.Valid(); it.Next() {
		key := it.Key()
		if !isTagKey(key) {
			continue
		}

		if _, ok := qr.AnyBound().(*big.Float); ok {
			value := extractValueFromKey(key)
			v, ok := bigIntValue.SetString(value, 10)
			var vF *big.Float
			if !ok {
				vF, _, err = big.ParseFloat(value, 10, 125, big.ToNearestEven)
				if err != nil {
					continue LOOP
				}
			}
			// Regardless of the query condition, we retrieve the height in order to sort later
			keyHeight, err := extractHeightFromKey(it.Key())
			if err != nil {
				txi.log.Error("failure to parse height from key:", err)
				continue
			}
			if qr.Key != types.TxHeightKey {
				withinBounds, err := checkHeightConditions(heightInfo, keyHeight)
				if err != nil {
					txi.log.Error("failure checking for height bounds:", err)
					continue
				}
				if !withinBounds {
					continue
				}
			}
			var withinBounds bool
			if !ok {
				withinBounds, err = idxutil.CheckBounds(qr, vF)
			} else {
				withinBounds, err = idxutil.CheckBounds(qr, v)
			}
			if err != nil {
				txi.log.Error("failed to parse bounds:", err)
			} else if withinBounds {
				txi.setTmpHashes(tmpHashes, key, it.Value(), keyHeight)
			}

			// XXX: passing time in a ABCI Events is not yet implemented
			// case time.Time:
			// 	v := strconv.ParseInt(extractValueFromKey(it.Key()), 10, 64)
			// 	if v == r.upperBound {
			// 		break
			// 	}
		}

		// Potentially exit early.
		select {
		case <-ctx.Done():
			break LOOP
		default:
		}
	}
	if err := it.Error(); err != nil {
		panic(err)
	}

	if len(tmpHashes) == 0 || firstRun {
		// Either:
		//
		// 1. Regardless if a previous match was attempted, which may have had
		// results, but no match was found for the current condition, then we
		// return no matches (assuming AND operand).
		//
		// 2. A previous match was not attempted, so we return all results.
		return tmpHashes
	}

	// Remove/reduce matches in filteredHashes that were not found in this
	// match (tmpHashes).
REMOVE_LOOP:
	for k, v := range filteredHashes {
		tmpHash := tmpHashes[k]
		if tmpHash.TxBytes == nil || !bytes.Equal(tmpHash.TxBytes, v.TxBytes) {
			delete(filteredHashes, k)
		} else {
			// If there is a match, update the height in filteredHashes
			v.Height = tmpHash.Height
			filteredHashes[k] = v
		}

		// Potentially exit early.
		select {
		case <-ctx.Done():
			break REMOVE_LOOP
		default:
		}
	}

	return filteredHashes
}

// Keys

func isTagKey(key []byte) bool {
	// Normally, if the event was indexed with an event sequence, the number of
	// tags should 4. Alternatively it should be 3 if the event was not indexed
	// with the corresponding event sequence. However, some attribute values in
	// production can contain the tag separator. Therefore, the condition is >= 3.
	numTags := 0
	for i := 0; i < len(key); i++ {
		if key[i] == tagKeySeparatorRune {
			numTags++
			if numTags >= 3 {
				return true
			}
		}
	}
	return false
}

func extractHeightFromKey(key []byte) (int64, error) {
	// the height is the second last element in the key.
	// Find the position of the last occurrence of tagKeySeparator
	endPos := bytes.LastIndexByte(key, tagKeySeparatorRune)
	if endPos == -1 {
		return 0, errors.New("separator not found")
	}

	// Find the position of the second last occurrence of tagKeySeparator
	startPos := bytes.LastIndexByte(key[:endPos-1], tagKeySeparatorRune)
	if startPos == -1 {
		return 0, errors.New("second last separator not found")
	}

	// Extract the height part of the key
	height, err := strconv.ParseInt(string(key[startPos+1:endPos]), 10, 64)
	if err != nil {
		return 0, err
	}
	return height, nil
}

func extractValueFromKey(key []byte) string {
	// Find the positions of tagKeySeparator in the byte slice
	var indices []int
	for i, b := range key {
		if b == tagKeySeparatorRune {
			indices = append(indices, i)
		}
	}

	// If there are less than 2 occurrences of tagKeySeparator, return an empty string
	if len(indices) < 2 {
		return ""
	}

	// Extract the value between the first and second last occurrence of tagKeySeparator
	value := key[indices[0]+1 : indices[len(indices)-2]]

	// Trim any leading or trailing whitespace
	value = bytes.TrimSpace(value)

	// TODO: Do an unsafe cast to avoid an extra allocation here
	return string(value)
}

func extractEventSeqFromKey(key []byte) string {
	endPos := bytes.LastIndexByte(key, tagKeySeparatorRune)

	if endPos == -1 {
		return "0"
	}

	for ; endPos < len(key); endPos++ {
		if key[endPos] == eventSeqSeperatorRuneAt0 {
			eventSeq := string(key[endPos:])
			if eventSeq, ok := strings.CutPrefix(eventSeq, eventSeqSeparator); ok {
				return eventSeq
			}
		}
	}

	return "0"
}

func keyForEvent(key string, value string, result *abci.TxResult, eventSeq int64) []byte {
	return []byte(fmt.Sprintf("%s/%s/%d/%d%s",
		key,
		value,
		result.Height,
		result.Index,
		eventSeqSeparator+strconv.FormatInt(eventSeq, 10),
	))
}

func keyForHeight(result *abci.TxResult) []byte {
	return []byte(fmt.Sprintf("%s/%d/%d/%d%s",
		types.TxHeightKey,
		result.Height,
		result.Height,
		result.Index,
		// Added to facilitate having the eventSeq in event keys
		// Otherwise queries break expecting 5 entries
		eventSeqSeparator+"0",
	))
}

func startKeyForCondition(c syntax.Condition, height int64) []byte {
	if height > 0 {
		return startKey(c.Tag, c.Arg.Value(), height)
	}
	return startKey(c.Tag, c.Arg.Value())
}

func startKey(fields ...any) []byte {
	var b bytes.Buffer
	for _, f := range fields {
		b.WriteString(fmt.Sprintf("%v", f) + tagKeySeparator)
	}
	return b.Bytes()
}

func validatePage(pagePtr *int, perPage, totalCount int) (int, error) {
	if perPage < 1 {
		return 1, fmt.Errorf("zero or negative perPage: %d", perPage)
	}

	if pagePtr == nil { // no page parameter
		return 1, nil
	}

	pages := ((totalCount - 1) / perPage) + 1
	if pages == 0 {
		pages = 1 // one page (even if it's empty)
	}
	page := *pagePtr
	if page <= 0 || page > pages {
		return 1, fmt.Errorf("page should be within [1, %d] range, given %d", pages, page)
	}

	return page, nil
}

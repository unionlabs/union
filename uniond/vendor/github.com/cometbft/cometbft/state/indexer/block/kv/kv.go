package kv

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"math/big"
	"sort"
	"strconv"
	"strings"

	"github.com/google/orderedcode"

	dbm "github.com/cometbft/cometbft-db"

	abci "github.com/cometbft/cometbft/abci/types"
	idxutil "github.com/cometbft/cometbft/internal/indexer"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/libs/pubsub/query"
	"github.com/cometbft/cometbft/libs/pubsub/query/syntax"
	"github.com/cometbft/cometbft/state/indexer"
	"github.com/cometbft/cometbft/types"
)

var _ indexer.BlockIndexer = (*BlockerIndexer)(nil)

// BlockerIndexer implements a block indexer, indexing FinalizeBlock
// events with an underlying KV store. Block events are indexed by their height,
// such that matching search criteria returns the respective block height(s).
type BlockerIndexer struct {
	store dbm.DB

	// Add unique event identifier to use when querying
	// Matching will be done both on height AND eventSeq
	eventSeq int64
	log      log.Logger
}

func New(store dbm.DB) *BlockerIndexer {
	return &BlockerIndexer{
		store: store,
	}
}

func (idx *BlockerIndexer) SetLogger(l log.Logger) {
	idx.log = l
}

// Has returns true if the given height has been indexed. An error is returned
// upon database query failure.
func (idx *BlockerIndexer) Has(height int64) (bool, error) {
	key, err := heightKey(height)
	if err != nil {
		return false, fmt.Errorf("failed to create block height index key: %w", err)
	}

	return idx.store.Has(key)
}

// Index indexes FinalizeBlock events for a given block by its height.
// The following is indexed:
//
// primary key: encode(block.height | height) => encode(height)
// FinalizeBlock events: encode(eventType.eventAttr|eventValue|height|finalize_block|eventSeq) => encode(height)
func (idx *BlockerIndexer) Index(bh types.EventDataNewBlockEvents) error {
	batch := idx.store.NewBatch()
	defer batch.Close()

	height := bh.Height

	// 1. index by height
	key, err := heightKey(height)
	if err != nil {
		return fmt.Errorf("failed to create block height index key: %w", err)
	}
	if err := batch.Set(key, int64ToBytes(height)); err != nil {
		return err
	}

	// 2. index block events
	if err := idx.indexEvents(batch, bh.Events, height); err != nil {
		return fmt.Errorf("failed to index FinalizeBlock events: %w", err)
	}

	return batch.WriteSync()
}

// Search performs a query for block heights that match a given FinalizeBlock
// event search criteria. The given query can match against zero,
// one or more block heights. In the case of height queries, i.e. block.height=H,
// if the height is indexed, that height alone will be returned. An error and
// nil slice is returned. Otherwise, a non-nil slice and nil error is returned.
func (idx *BlockerIndexer) Search(ctx context.Context, q *query.Query) ([]int64, error) {
	results := make([]int64, 0)
	select {
	case <-ctx.Done():
		return results, nil

	default:
	}

	conditions := q.Syntax()

	// conditions to skip because they're handled before "everything else"
	skipIndexes := make([]int, 0)

	var ok bool

	var heightInfo HeightInfo
	// If we are not matching events and block.height occurs more than once, the later value will
	// overwrite the first one.
	conditions, heightInfo, ok = dedupHeight(conditions)

	// Extract ranges. If both upper and lower bounds exist, it's better to get
	// them in order as to not iterate over kvs that are not within range.
	ranges, rangeIndexes, heightRange := indexer.LookForRangesWithHeight(conditions)
	heightInfo.heightRange = heightRange

	// If we have additional constraints and want to query per event
	// attributes, we cannot simply return all blocks for a height.
	// But we remember the height we want to find and forward it to
	// match(). If we only have the height constraint
	// in the query (the second part of the ||), we don't need to query
	// per event conditions and return all events within the height range.
	if ok && heightInfo.onlyHeightEq {
		ok, err := idx.Has(heightInfo.height)
		if err != nil {
			return nil, err
		}

		if ok {
			return []int64{heightInfo.height}, nil
		}

		return results, nil
	}

	var heightsInitialized bool
	filteredHeights := make(map[string][]byte)
	if heightInfo.heightEqIdx != -1 {
		skipIndexes = append(skipIndexes, heightInfo.heightEqIdx)
	}

	if len(ranges) > 0 {
		skipIndexes = append(skipIndexes, rangeIndexes...)

		for _, qr := range ranges {
			// If we have a query range over height and want to still look for
			// specific event values we do not want to simply return all
			// blocks in this height range. We remember the height range info
			// and pass it on to match() to take into account when processing events.
			if qr.Key == types.BlockHeightKey && !heightInfo.onlyHeightRange {
				// If the query contains ranges other than the height then we need to treat the height
				// range when querying the conditions of the other range.
				// Otherwise we can just return all the blocks within the height range (as there is no
				// additional constraint on events)

				continue

			}
			prefix, err := orderedcode.Append(nil, qr.Key)
			if err != nil {
				return nil, fmt.Errorf("failed to create prefix key: %w", err)
			}

			if !heightsInitialized {
				filteredHeights, err = idx.matchRange(ctx, qr, prefix, filteredHeights, true, heightInfo)
				if err != nil {
					return nil, err
				}

				heightsInitialized = true

				// Ignore any remaining conditions if the first condition resulted in no
				// matches (assuming implicit AND operand).
				if len(filteredHeights) == 0 {
					break
				}
			} else {
				filteredHeights, err = idx.matchRange(ctx, qr, prefix, filteredHeights, false, heightInfo)
				if err != nil {
					return nil, err
				}
			}
		}
	}

	// for all other conditions
	for i, c := range conditions {
		if intInSlice(i, skipIndexes) {
			continue
		}

		startKey, err := orderedcode.Append(nil, c.Tag, c.Arg.Value())
		if err != nil {
			return nil, err
		}

		if !heightsInitialized {
			filteredHeights, err = idx.match(ctx, c, startKey, filteredHeights, true, heightInfo)
			if err != nil {
				return nil, err
			}

			heightsInitialized = true

			// Ignore any remaining conditions if the first condition resulted in no
			// matches (assuming implicit AND operand).
			if len(filteredHeights) == 0 {
				break
			}
		} else {
			filteredHeights, err = idx.match(ctx, c, startKey, filteredHeights, false, heightInfo)
			if err != nil {
				return nil, err
			}
		}
	}

	// fetch matching heights
	results = make([]int64, 0, len(filteredHeights))
	resultMap := make(map[int64]struct{})
	for _, hBz := range filteredHeights {
		h := int64FromBytes(hBz)

		ok, err := idx.Has(h)
		if err != nil {
			return nil, err
		}
		if ok {
			if _, ok := resultMap[h]; !ok {
				resultMap[h] = struct{}{}
				results = append(results, h)
			}
		}

		select {
		case <-ctx.Done():
			break

		default:
		}
	}

	sort.Slice(results, func(i, j int) bool { return results[i] < results[j] })

	return results, nil
}

// matchRange returns all matching block heights that match a given QueryRange
// and start key. An already filtered result (filteredHeights) is provided such
// that any non-intersecting matches are removed.
//
// NOTE: The provided filteredHeights may be empty if no previous condition has
// matched.
func (idx *BlockerIndexer) matchRange(
	ctx context.Context,
	qr indexer.QueryRange,
	startKey []byte,
	filteredHeights map[string][]byte,
	firstRun bool,
	heightInfo HeightInfo,
) (map[string][]byte, error) {
	// A previous match was attempted but resulted in no matches, so we return
	// no matches (assuming AND operand).
	if !firstRun && len(filteredHeights) == 0 {
		return filteredHeights, nil
	}

	tmpHeights := make(map[string][]byte)

	it, err := dbm.IteratePrefix(idx.store, startKey)
	if err != nil {
		return nil, fmt.Errorf("failed to create prefix iterator: %w", err)
	}
	defer it.Close()

LOOP:
	for ; it.Valid(); it.Next() {
		var (
			eventValue string
			err        error
		)

		if qr.Key == types.BlockHeightKey {
			eventValue, err = parseValueFromPrimaryKey(it.Key())
		} else {
			eventValue, err = parseValueFromEventKey(it.Key())
		}

		if err != nil {
			continue
		}

		if _, ok := qr.AnyBound().(*big.Float); ok {
			v := new(big.Int)
			v, ok := v.SetString(eventValue, 10)
			var vF *big.Float
			if !ok {
				// The precision here is 125. For numbers bigger than this, the value
				// will not be parsed properly
				vF, _, err = big.ParseFloat(eventValue, 10, 125, big.ToNearestEven)
				if err != nil {
					continue LOOP
				}
			}

			if qr.Key != types.BlockHeightKey {
				keyHeight, err := parseHeightFromEventKey(it.Key())
				if err != nil {
					idx.log.Error("failure to parse height from key:", err)
					continue LOOP
				}
				withinHeight, err := checkHeightConditions(heightInfo, keyHeight)
				if err != nil {
					idx.log.Error("failure checking for height bounds:", err)
					continue LOOP
				}
				if !withinHeight {
					continue LOOP
				}
			}

			var withinBounds bool
			var err error
			if !ok {
				withinBounds, err = idxutil.CheckBounds(qr, vF)
			} else {
				withinBounds, err = idxutil.CheckBounds(qr, v)
			}
			if err != nil {
				idx.log.Error("failed to parse bounds:", err)
			} else {
				if withinBounds {
					idx.setTmpHeights(tmpHeights, it)
				}
			}
		}

		select {
		case <-ctx.Done():
			break

		default:
		}
	}

	if err := it.Error(); err != nil {
		return nil, err
	}

	if len(tmpHeights) == 0 || firstRun {
		// Either:
		//
		// 1. Regardless if a previous match was attempted, which may have had
		// results, but no match was found for the current condition, then we
		// return no matches (assuming AND operand).
		//
		// 2. A previous match was not attempted, so we return all results.
		return tmpHeights, nil
	}

	// Remove/reduce matches in filteredHashes that were not found in this
	// match (tmpHashes).
	for k, v := range filteredHeights {
		tmpHeight := tmpHeights[k]

		// Check whether in this iteration we have not found an overlapping height (tmpHeight == nil)
		// or whether the events in which the attributed occurred do not match (first part of the condition)
		if tmpHeight == nil || !bytes.Equal(tmpHeight, v) {
			delete(filteredHeights, k)

			select {
			case <-ctx.Done():
				break

			default:
			}
		}
	}

	return filteredHeights, nil
}

func (idx *BlockerIndexer) setTmpHeights(tmpHeights map[string][]byte, it dbm.Iterator) {
	// If we return attributes that occur within the same events, then store the event sequence in the
	// result map as well
	eventSeq, _ := parseEventSeqFromEventKey(it.Key())
	retVal := it.Value()
	tmpHeights[string(retVal)+strconv.FormatInt(eventSeq, 10)] = it.Value()

}

// match returns all matching heights that meet a given query condition and start
// key. An already filtered result (filteredHeights) is provided such that any
// non-intersecting matches are removed.
//
// NOTE: The provided filteredHeights may be empty if no previous condition has
// matched.
func (idx *BlockerIndexer) match(
	ctx context.Context,
	c syntax.Condition,
	startKeyBz []byte,
	filteredHeights map[string][]byte,
	firstRun bool,
	heightInfo HeightInfo,
) (map[string][]byte, error) {
	// A previous match was attempted but resulted in no matches, so we return
	// no matches (assuming AND operand).
	if !firstRun && len(filteredHeights) == 0 {
		return filteredHeights, nil
	}

	tmpHeights := make(map[string][]byte)

	switch {
	case c.Op == syntax.TEq:
		it, err := dbm.IteratePrefix(idx.store, startKeyBz)
		if err != nil {
			return nil, fmt.Errorf("failed to create prefix iterator: %w", err)
		}
		defer it.Close()

		for ; it.Valid(); it.Next() {

			keyHeight, err := parseHeightFromEventKey(it.Key())
			if err != nil {
				idx.log.Error("failure to parse height from key:", err)
				continue
			}
			withinHeight, err := checkHeightConditions(heightInfo, keyHeight)
			if err != nil {
				idx.log.Error("failure checking for height bounds:", err)
				continue
			}
			if !withinHeight {
				continue
			}

			idx.setTmpHeights(tmpHeights, it)

			if err := ctx.Err(); err != nil {
				break
			}
		}

		if err := it.Error(); err != nil {
			return nil, err
		}

	case c.Op == syntax.TExists:
		prefix, err := orderedcode.Append(nil, c.Tag)
		if err != nil {
			return nil, err
		}

		it, err := dbm.IteratePrefix(idx.store, prefix)
		if err != nil {
			return nil, fmt.Errorf("failed to create prefix iterator: %w", err)
		}
		defer it.Close()

		for ; it.Valid(); it.Next() {

			keyHeight, err := parseHeightFromEventKey(it.Key())
			if err != nil {
				idx.log.Error("failure to parse height from key:", err)
				continue
			}
			withinHeight, err := checkHeightConditions(heightInfo, keyHeight)
			if err != nil {
				idx.log.Error("failure checking for height bounds:", err)
				continue
			}
			if !withinHeight {
				continue
			}

			idx.setTmpHeights(tmpHeights, it)

			select {
			case <-ctx.Done():
				break

			default:
			}
		}

		if err := it.Error(); err != nil {
			return nil, err
		}

	case c.Op == syntax.TContains:
		prefix, err := orderedcode.Append(nil, c.Tag)
		if err != nil {
			return nil, err
		}

		it, err := dbm.IteratePrefix(idx.store, prefix)
		if err != nil {
			return nil, fmt.Errorf("failed to create prefix iterator: %w", err)
		}
		defer it.Close()

		for ; it.Valid(); it.Next() {
			eventValue, err := parseValueFromEventKey(it.Key())
			if err != nil {
				continue
			}

			if strings.Contains(eventValue, c.Arg.Value()) {
				keyHeight, err := parseHeightFromEventKey(it.Key())
				if err != nil {
					idx.log.Error("failure to parse height from key:", err)
					continue
				}
				withinHeight, err := checkHeightConditions(heightInfo, keyHeight)
				if err != nil {
					idx.log.Error("failure checking for height bounds:", err)
					continue
				}
				if !withinHeight {
					continue
				}
				idx.setTmpHeights(tmpHeights, it)
			}

			select {
			case <-ctx.Done():
				break

			default:
			}
		}
		if err := it.Error(); err != nil {
			return nil, err
		}

	default:
		return nil, errors.New("other operators should be handled already")
	}

	if len(tmpHeights) == 0 || firstRun {
		// Either:
		//
		// 1. Regardless if a previous match was attempted, which may have had
		// results, but no match was found for the current condition, then we
		// return no matches (assuming AND operand).
		//
		// 2. A previous match was not attempted, so we return all results.
		return tmpHeights, nil
	}

	// Remove/reduce matches in filteredHeights that were not found in this
	// match (tmpHeights).
	for k, v := range filteredHeights {
		tmpHeight := tmpHeights[k]
		if tmpHeight == nil || !bytes.Equal(tmpHeight, v) {
			delete(filteredHeights, k)

			select {
			case <-ctx.Done():
				break

			default:
			}
		}
	}

	return filteredHeights, nil
}

func (idx *BlockerIndexer) indexEvents(batch dbm.Batch, events []abci.Event, height int64) error {
	heightBz := int64ToBytes(height)

	for _, event := range events {
		idx.eventSeq = idx.eventSeq + 1
		// only index events with a non-empty type
		if len(event.Type) == 0 {
			continue
		}

		for _, attr := range event.Attributes {
			if len(attr.Key) == 0 {
				continue
			}

			// index iff the event specified index:true and it's not a reserved event
			compositeKey := fmt.Sprintf("%s.%s", event.Type, attr.Key)
			if compositeKey == types.BlockHeightKey {
				return fmt.Errorf("event type and attribute key \"%s\" is reserved; please use a different key", compositeKey)
			}

			if attr.GetIndex() {
				key, err := eventKey(compositeKey, attr.Value, height, idx.eventSeq)
				if err != nil {
					return fmt.Errorf("failed to create block index key: %w", err)
				}

				if err := batch.Set(key, heightBz); err != nil {
					return err
				}
			}
		}
	}

	return nil
}

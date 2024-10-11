package kv

import (
	"encoding/binary"
	"fmt"
	"math/big"

	"github.com/google/orderedcode"

	abci "github.com/cometbft/cometbft/abci/types"
	idxutil "github.com/cometbft/cometbft/internal/indexer"
	cmtsyntax "github.com/cometbft/cometbft/libs/pubsub/query/syntax"
	"github.com/cometbft/cometbft/state/indexer"
	"github.com/cometbft/cometbft/types"
)

type HeightInfo struct {
	heightRange     indexer.QueryRange
	height          int64
	heightEqIdx     int
	onlyHeightRange bool
	onlyHeightEq    bool
}

// IntInSlice returns true if a is found in the list.
func intInSlice(a int, list []int) bool {
	for _, b := range list {
		if b == a {
			return true
		}
	}
	return false
}

func ParseEventSeqFromEventKey(key []byte) (int64, error) {
	var (
		compositeKey, typ, eventValue string
		height                        int64
		eventSeq                      int64
	)

	remaining, err := orderedcode.Parse(string(key), &compositeKey, &eventValue, &height, &typ, &eventSeq)
	if err != nil {
		return 0, fmt.Errorf("failed to parse event key: %w", err)
	}

	if len(remaining) != 0 {
		return 0, fmt.Errorf("unexpected remainder in key: %s", remaining)
	}

	return eventSeq, nil
}

func dedupHeight(conditions []cmtsyntax.Condition) (dedupConditions []cmtsyntax.Condition, heightInfo HeightInfo) {
	heightInfo.heightEqIdx = -1
	heightRangeExists := false
	found := false
	var heightCondition []cmtsyntax.Condition
	heightInfo.onlyHeightEq = true
	heightInfo.onlyHeightRange = true
	for _, c := range conditions {
		if c.Tag == types.TxHeightKey {
			if c.Op == cmtsyntax.TEq {
				if heightRangeExists || found {
					continue
				}
				hFloat := c.Arg.Number()
				if hFloat != nil {
					h, _ := hFloat.Int64()
					heightInfo.height = h
					found = true
					heightCondition = append(heightCondition, c)
				}
			} else {
				heightInfo.onlyHeightEq = false
				heightRangeExists = true
				dedupConditions = append(dedupConditions, c)
			}
		} else {
			heightInfo.onlyHeightRange = false
			heightInfo.onlyHeightEq = false
			dedupConditions = append(dedupConditions, c)
		}
	}
	if !heightRangeExists && len(heightCondition) != 0 {
		heightInfo.heightEqIdx = len(dedupConditions)
		heightInfo.onlyHeightRange = false
		dedupConditions = append(dedupConditions, heightCondition...)
	} else {
		// If we found a range make sure we set the height idx to -1 as the height equality
		// will be removed
		heightInfo.heightEqIdx = -1
		heightInfo.height = 0
		heightInfo.onlyHeightEq = false
	}
	return dedupConditions, heightInfo
}

func checkHeightConditions(heightInfo HeightInfo, keyHeight int64) (bool, error) {
	if heightInfo.heightRange.Key != "" {
		withinBounds, err := idxutil.CheckBounds(heightInfo.heightRange, big.NewInt(keyHeight))
		if err != nil || !withinBounds {
			return false, err
		}
	} else if heightInfo.height != 0 && keyHeight != heightInfo.height {
		return false, nil
	}

	return true, nil
}

func int64FromBytes(bz []byte) int64 {
	v, _ := binary.Varint(bz)
	return v
}

func int64ToBytes(i int64) []byte {
	buf := make([]byte, binary.MaxVarintLen64)
	n := binary.PutVarint(buf, i)
	return buf[:n]
}

func getKeys(indexer *TxIndex) [][]byte {
	var keys [][]byte

	itr, err := indexer.store.Iterator(nil, nil)
	if err != nil {
		panic(err)
	}
	for ; itr.Valid(); itr.Next() {
		key := make([]byte, len(itr.Key()))
		copy(key, itr.Key())

		keys = append(keys, key)
	}
	return keys
}

type TxResultByHeight []*abci.TxResult

func (a TxResultByHeight) Len() int           { return len(a) }
func (a TxResultByHeight) Swap(i, j int)      { a[i], a[j] = a[j], a[i] }
func (a TxResultByHeight) Less(i, j int) bool { return a[i].Height < a[j].Height }

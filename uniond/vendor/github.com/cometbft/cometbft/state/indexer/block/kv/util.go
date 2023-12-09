package kv

import (
	"encoding/binary"
	"fmt"
	"math/big"
	"strconv"

	"github.com/google/orderedcode"

	idxutil "github.com/cometbft/cometbft/internal/indexer"
	"github.com/cometbft/cometbft/libs/pubsub/query/syntax"
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

func intInSlice(a int, list []int) bool {
	for _, b := range list {
		if b == a {
			return true
		}
	}

	return false
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

func heightKey(height int64) ([]byte, error) {
	return orderedcode.Append(
		nil,
		types.BlockHeightKey,
		height,
	)
}

func eventKey(compositeKey, eventValue string, height int64, eventSeq int64) ([]byte, error) {
	return orderedcode.Append(
		nil,
		compositeKey,
		eventValue,
		height,
		eventSeq,
	)
}

func parseValueFromPrimaryKey(key []byte) (string, error) {
	var (
		compositeKey string
		height       int64
	)

	remaining, err := orderedcode.Parse(string(key), &compositeKey, &height)
	if err != nil {
		return "", fmt.Errorf("failed to parse event key: %w", err)
	}

	if len(remaining) != 0 {
		return "", fmt.Errorf("unexpected remainder in key: %s", remaining)
	}

	return strconv.FormatInt(height, 10), nil
}

func parseValueFromEventKey(key []byte) (string, error) {
	var (
		compositeKey, eventValue string
		height                   int64
	)

	_, err := orderedcode.Parse(string(key), &compositeKey, &eventValue, &height)
	if err != nil {
		return "", fmt.Errorf("failed to parse event key: %w", err)
	}

	return eventValue, nil
}

func parseHeightFromEventKey(key []byte) (int64, error) {
	var (
		compositeKey, eventValue string
		height                   int64
	)

	_, err := orderedcode.Parse(string(key), &compositeKey, &eventValue, &height)
	if err != nil {
		return -1, fmt.Errorf("failed to parse event key: %w", err)
	}

	return height, nil
}

func parseEventSeqFromEventKey(key []byte) (int64, error) {
	var (
		compositeKey, eventValue string
		height                   int64
		eventSeq                 int64
	)

	remaining, err := orderedcode.Parse(string(key), &compositeKey, &eventValue, &height)
	if err != nil {
		return 0, fmt.Errorf("failed to parse event sequence: %w", err)
	}

	// We either have an event sequence or a function type (potentially) followed by an event sequence.
	// Potential scenarios:
	// 1. Events indexed with v0.38.x and later, will only have an event sequence
	// 2. Events indexed between v0.34.27 and v0.37.x will have a function type and an event sequence
	// 3. Events indexed before v0.34.27 will only have a function type
	// function_type = 'being_block_event' | 'end_block_event'

	if len(remaining) == 0 { // The event was not properly indexed
		return 0, fmt.Errorf("failed to parse event sequence, invalid event format")
	}
	var typ string
	remaining2, err := orderedcode.Parse(remaining, &typ) // Check if we have scenarios 2. or 3. (described above).
	if err != nil {                                       // If it cannot parse the event function type, it could be 1.
		remaining, err2 := orderedcode.Parse(string(key), &compositeKey, &eventValue, &height, &eventSeq)
		if err2 != nil || len(remaining) != 0 { // We should not have anything else after the eventSeq.
			return 0, fmt.Errorf("failed to parse event sequence: %w; and %w", err, err2)
		}
	} else if len(remaining2) != 0 { // Are we in case 2 or 3
		remaining, err2 := orderedcode.Parse(remaining2, &eventSeq) // the event follows the scenario in 2.,
		// retrieve the eventSeq
		// there should be no error
		if err2 != nil || len(remaining) != 0 { // We should not have anything else after the eventSeq if in 2.
			return 0, fmt.Errorf("failed to parse event sequence: %w", err2)
		}
	}
	return eventSeq, nil
}

// Remove all occurrences of height equality queries except one. While we are traversing the conditions, check whether the only condition in
// addition to match events is the height equality or height range query. At the same time, if we do have a height range condition
// ignore the height equality condition. If a height equality exists, place the condition index in the query and the desired height
// into the heightInfo struct
func dedupHeight(conditions []syntax.Condition) (dedupConditions []syntax.Condition, heightInfo HeightInfo, found bool) {
	heightInfo.heightEqIdx = -1
	heightRangeExists := false
	var heightCondition []syntax.Condition
	heightInfo.onlyHeightEq = true
	heightInfo.onlyHeightRange = true
	for _, c := range conditions {
		if c.Tag == types.BlockHeightKey {
			if c.Op == syntax.TEq {
				if found || heightRangeExists {
					continue
				}
				hFloat := c.Arg.Number()
				if hFloat != nil {
					h, _ := hFloat.Int64()
					heightInfo.height = h
					heightCondition = append(heightCondition, c)
					found = true
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
		// If we found a range make sure we set the hegiht idx to -1 as the height equality
		// will be removed
		heightInfo.heightEqIdx = -1
		heightInfo.height = 0
		heightInfo.onlyHeightEq = false
		found = false
	}
	return dedupConditions, heightInfo, found
}

func checkHeightConditions(heightInfo HeightInfo, keyHeight int64) (bool, error) {
	if heightInfo.heightRange.Key != "" {
		withinBounds, err := idxutil.CheckBounds(heightInfo.heightRange, big.NewInt(keyHeight))
		if err != nil || !withinBounds {
			return false, err
		}
	} else {
		if heightInfo.height != 0 && keyHeight != heightInfo.height {
			return false, nil
		}
	}
	return true, nil
}

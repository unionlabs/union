package kv

import (
	"bytes"
	"sort"
)

func (kvs Pairs) Len() int { return len(kvs.Pairs) }
func (kvs Pairs) Less(i, j int) bool {
	switch bytes.Compare(kvs.Pairs[i].Key, kvs.Pairs[j].Key) {
	case -1:
		return true

	case 0:
		return bytes.Compare(kvs.Pairs[i].Value, kvs.Pairs[j].Value) < 0

	case 1:
		return false

	default:
		panic("invalid comparison result")
	}
}

func (kvs Pairs) Swap(i, j int) { kvs.Pairs[i], kvs.Pairs[j] = kvs.Pairs[j], kvs.Pairs[i] }

// Sort invokes sort.Sort on kvs.
func (kvs Pairs) Sort() { sort.Sort(kvs) }

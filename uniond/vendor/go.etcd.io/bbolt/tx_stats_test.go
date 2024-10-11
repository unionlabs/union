package bbolt

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestTxStats_add(t *testing.T) {
	statsA := TxStats{
		PageCount:     1,
		PageAlloc:     2,
		CursorCount:   3,
		NodeCount:     100,
		NodeDeref:     101,
		Rebalance:     1000,
		RebalanceTime: 1001 * time.Second,
		Split:         10000,
		Spill:         10001,
		SpillTime:     10001 * time.Second,
		Write:         100000,
		WriteTime:     100001 * time.Second,
	}

	statsB := TxStats{
		PageCount:     2,
		PageAlloc:     3,
		CursorCount:   4,
		NodeCount:     101,
		NodeDeref:     102,
		Rebalance:     1001,
		RebalanceTime: 1002 * time.Second,
		Split:         11001,
		Spill:         11002,
		SpillTime:     11002 * time.Second,
		Write:         110001,
		WriteTime:     110010 * time.Second,
	}

	statsB.add(&statsA)
	assert.Equal(t, int64(3), statsB.GetPageCount())
	assert.Equal(t, int64(5), statsB.GetPageAlloc())
	assert.Equal(t, int64(7), statsB.GetCursorCount())
	assert.Equal(t, int64(201), statsB.GetNodeCount())
	assert.Equal(t, int64(203), statsB.GetNodeDeref())
	assert.Equal(t, int64(2001), statsB.GetRebalance())
	assert.Equal(t, 2003*time.Second, statsB.GetRebalanceTime())
	assert.Equal(t, int64(21001), statsB.GetSplit())
	assert.Equal(t, int64(21003), statsB.GetSpill())
	assert.Equal(t, 21003*time.Second, statsB.GetSpillTime())
	assert.Equal(t, int64(210001), statsB.GetWrite())
	assert.Equal(t, 210011*time.Second, statsB.GetWriteTime())
}

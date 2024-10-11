package grocksdb

import (
	"testing"
)

func TestBBT(t *testing.T) {
	t.Parallel()

	b := NewDefaultBlockBasedTableOptions()
	defer b.Destroy()

	b.SetBlockSize(123)
	b.SetOptimizeFiltersForMemory(true)
	b.SetTopLevelIndexPinningTier(KFallbackPinningTier)
	b.SetPartitionPinningTier(KNonePinningTier)
	b.SetUnpartitionedPinningTier(KAllPinningTier)
}

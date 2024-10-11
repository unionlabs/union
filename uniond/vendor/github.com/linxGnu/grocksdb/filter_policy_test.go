package grocksdb

import (
	"testing"
)

func TestFilterPolicy(t *testing.T) {
	t.Parallel()

	t.Run("Bloom", func(t *testing.T) {
		t.Parallel()

		flt := NewBloomFilter(1.2)
		defer flt.Destroy()
	})

	t.Run("BloomFull", func(t *testing.T) {
		t.Parallel()

		flt := NewBloomFilterFull(1.2)
		defer flt.Destroy()
	})

	t.Run("Ribbon", func(t *testing.T) {
		t.Parallel()

		flt := NewRibbonFilterPolicy(1.2)
		defer flt.Destroy()
	})

	t.Run("RibbonHybrid", func(t *testing.T) {
		t.Parallel()

		flt := NewRibbonHybridFilterPolicy(1.2, 1)
		defer flt.Destroy()
	})
}

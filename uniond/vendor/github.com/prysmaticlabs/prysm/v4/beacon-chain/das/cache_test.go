package das

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestCacheEnsureDelete(t *testing.T) {
	c := newCache()
	require.Equal(t, 0, len(c.entries))
	root := bytesutil.ToBytes32([]byte("root"))
	slot := primitives.Slot(1234)
	k := cacheKey{root: root, slot: slot}
	entry := c.ensure(k)
	require.Equal(t, 1, len(c.entries))
	require.Equal(t, c.entries[k], entry)

	c.delete(k)
	require.Equal(t, 0, len(c.entries))
	var nilEntry *cacheEntry
	require.Equal(t, nilEntry, c.entries[k])
}

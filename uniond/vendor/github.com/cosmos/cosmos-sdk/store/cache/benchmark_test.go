package cache

import (
	"testing"

	"github.com/cosmos/cosmos-sdk/store/types"
)

func freshMgr() *CommitKVStoreCacheManager {
	return &CommitKVStoreCacheManager{
		caches: map[string]types.CommitKVStore{
			"a1":           nil,
			"alalalalalal": nil,
		},
	}
}

func populate(mgr *CommitKVStoreCacheManager) {
	mgr.caches["this one"] = (types.CommitKVStore)(nil)
	mgr.caches["those ones are the ones"] = (types.CommitKVStore)(nil)
	mgr.caches["very huge key right here and there are we going to ones are the ones"] = (types.CommitKVStore)(nil)
}

func BenchmarkReset(b *testing.B) {
	b.ReportAllocs()
	mgr := freshMgr()

	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		mgr.Reset()
		if len(mgr.caches) != 0 {
			b.Fatal("Reset failed")
		}
		populate(mgr)
		if len(mgr.caches) == 0 {
			b.Fatal("populate failed")
		}
		mgr.Reset()
		if len(mgr.caches) != 0 {
			b.Fatal("Reset failed")
		}
	}

	if mgr == nil {
		b.Fatal("Impossible condition")
	}
}

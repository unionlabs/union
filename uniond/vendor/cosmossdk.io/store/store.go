package store

import (
	corestore "cosmossdk.io/core/store"
	"cosmossdk.io/store/cache"
	"cosmossdk.io/store/metrics"
	"cosmossdk.io/store/rootmulti"
	"cosmossdk.io/store/types"
)

func NewCommitMultiStore(db corestore.KVStoreWithBatch, logger types.Logger, metricGatherer metrics.StoreMetrics) types.CommitMultiStore {
	return rootmulti.NewStore(db, logger, metricGatherer)
}

func NewCommitKVStoreCacheManager() types.MultiStorePersistentCache {
	return cache.NewCommitKVStoreCacheManager(cache.DefaultCommitKVStoreCacheSize)
}

package iavl

import "sync/atomic"

// Statisc about db runtime state
type Statistics struct {
	// Each time GetNode operation hit cache
	cacheHitCnt uint64

	// Each time GetNode and GetFastNode operation miss cache
	cacheMissCnt uint64

	// Each time GetFastNode operation hit cache
	fastCacheHitCnt uint64

	// Each time GetFastNode operation miss cache
	fastCacheMissCnt uint64
}

func (stat *Statistics) IncCacheHitCnt() {
	if stat == nil {
		return
	}
	atomic.AddUint64(&stat.cacheHitCnt, 1)
}

func (stat *Statistics) IncCacheMissCnt() {
	if stat == nil {
		return
	}
	atomic.AddUint64(&stat.cacheMissCnt, 1)
}

func (stat *Statistics) IncFastCacheHitCnt() {
	if stat == nil {
		return
	}
	atomic.AddUint64(&stat.fastCacheHitCnt, 1)
}

func (stat *Statistics) IncFastCacheMissCnt() {
	if stat == nil {
		return
	}
	atomic.AddUint64(&stat.fastCacheMissCnt, 1)
}

func (stat *Statistics) GetCacheHitCnt() uint64 {
	return atomic.LoadUint64(&stat.cacheHitCnt)
}

func (stat *Statistics) GetCacheMissCnt() uint64 {
	return atomic.LoadUint64(&stat.cacheMissCnt)
}

func (stat *Statistics) GetFastCacheHitCnt() uint64 {
	return atomic.LoadUint64(&stat.fastCacheHitCnt)
}

func (stat *Statistics) GetFastCacheMissCnt() uint64 {
	return atomic.LoadUint64(&stat.fastCacheMissCnt)
}

func (stat *Statistics) Reset() {
	atomic.StoreUint64(&stat.cacheHitCnt, 0)
	atomic.StoreUint64(&stat.cacheMissCnt, 0)
	atomic.StoreUint64(&stat.fastCacheHitCnt, 0)
	atomic.StoreUint64(&stat.fastCacheMissCnt, 0)
}

// Options define tree options.
type Options struct {
	// Sync synchronously flushes all writes to storage, using e.g. the fsync syscall.
	// Disabling this significantly improves performance, but can lose data on e.g. power loss.
	Sync bool

	// InitialVersion specifies the initial version number. If any versions already exist below
	// this, an error is returned when loading the tree. Only used for the initial SaveVersion()
	// call.
	InitialVersion uint64

	// When Stat is not nil, statistical logic needs to be executed
	Stat *Statistics
}

// DefaultOptions returns the default options for IAVL.
func DefaultOptions() Options {
	return Options{}
}

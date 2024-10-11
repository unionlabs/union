package execution

import (
	"errors"
	"math/big"
	"sync"

	"github.com/ethereum/go-ethereum/common"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/execution/types"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"k8s.io/client-go/tools/cache"
)

var (
	// ErrNotAHeaderInfo will be returned when a cache object is not a pointer to
	// a headerInfo struct.
	ErrNotAHeaderInfo = errors.New("object is not a header info")

	// maxCacheSize is 2x of the follow distance for additional cache padding.
	// Requests should be only accessing blocks within recent blocks within the
	// Eth1FollowDistance.
	maxCacheSize = 2 * params.BeaconConfig().Eth1FollowDistance

	// Metrics
	headerCacheMiss = promauto.NewCounter(prometheus.CounterOpts{
		Name: "powchain_header_cache_miss",
		Help: "The number of header requests that aren't present in the cache.",
	})
	headerCacheHit = promauto.NewCounter(prometheus.CounterOpts{
		Name: "powchain_header_cache_hit",
		Help: "The number of header requests that are present in the cache.",
	})
	headerCacheSize = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "powchain_header_cache_size",
		Help: "The number of headers in the header cache",
	})
)

// hashKeyFn takes the hex string representation as the key for a headerInfo.
func hashKeyFn(obj interface{}) (string, error) {
	hInfo, ok := obj.(*types.HeaderInfo)
	if !ok {
		return "", ErrNotAHeaderInfo
	}

	return hInfo.Hash.Hex(), nil
}

// heightKeyFn takes the string representation of the block header number as the key
// for a headerInfo.
func heightKeyFn(obj interface{}) (string, error) {
	hInfo, ok := obj.(*types.HeaderInfo)
	if !ok {
		return "", ErrNotAHeaderInfo
	}

	return hInfo.Number.String(), nil
}

// headerCache struct with two queues for looking up by hash or by block height.
type headerCache struct {
	hashCache   *cache.FIFO
	heightCache *cache.FIFO
	lock        sync.RWMutex
}

// newHeaderCache creates a new block cache for storing/accessing headerInfo from
// memory.
func newHeaderCache() *headerCache {
	return &headerCache{
		hashCache:   cache.NewFIFO(hashKeyFn),
		heightCache: cache.NewFIFO(heightKeyFn),
	}
}

// HeaderInfoByHash fetches headerInfo by its header hash. Returns true with a
// reference to the header info, if exists. Otherwise returns false, nil.
func (c *headerCache) HeaderInfoByHash(hash common.Hash) (bool, *types.HeaderInfo, error) {
	c.lock.RLock()
	defer c.lock.RUnlock()

	obj, exists, err := c.hashCache.GetByKey(hash.Hex())
	if err != nil {
		return false, nil, err
	}

	if exists {
		headerCacheHit.Inc()
	} else {
		headerCacheMiss.Inc()
		return false, nil, nil
	}

	hInfo, ok := obj.(*types.HeaderInfo)
	if !ok {
		return false, nil, ErrNotAHeaderInfo
	}

	return true, hInfo.Copy(), nil
}

// HeaderInfoByHeight fetches headerInfo by its header number. Returns true with a
// reference to the header info, if exists. Otherwise returns false, nil.
func (c *headerCache) HeaderInfoByHeight(height *big.Int) (bool, *types.HeaderInfo, error) {
	c.lock.RLock()
	defer c.lock.RUnlock()

	obj, exists, err := c.heightCache.GetByKey(height.String())
	if err != nil {
		return false, nil, err
	}

	if exists {
		headerCacheHit.Inc()
	} else {
		headerCacheMiss.Inc()
		return false, nil, nil
	}

	hInfo, ok := obj.(*types.HeaderInfo)
	if !ok {
		return false, nil, ErrNotAHeaderInfo
	}

	return exists, hInfo.Copy(), nil
}

// AddHeader adds a headerInfo object to the cache. This method also trims the
// least recently added header info if the cache size has reached the max cache
// size limit. This method should be called in sequential header number order if
// the desired behavior is that the blocks with the highest header number should
// be present in the cache.
func (c *headerCache) AddHeader(hdr *types.HeaderInfo) error {
	c.lock.Lock()
	defer c.lock.Unlock()

	hInfo := hdr.Copy()

	if err := c.hashCache.AddIfNotPresent(hInfo); err != nil {
		return err
	}
	if err := c.heightCache.AddIfNotPresent(hInfo); err != nil {
		return err
	}

	trim(c.hashCache, maxCacheSize)
	trim(c.heightCache, maxCacheSize)

	headerCacheSize.Set(float64(len(c.hashCache.ListKeys())))

	return nil
}

// trim the FIFO queue to the maxSize.
func trim(queue *cache.FIFO, maxSize uint64) {
	for s := uint64(len(queue.ListKeys())); s > maxSize; s-- {
		// #nosec G104 popProcessNoopFunc never returns an error
		if _, err := queue.Pop(popProcessNoopFunc); err != nil { // This never returns an error, but we'll handle anyway for sanity.
			panic(err)
		}
	}
}

// popProcessNoopFunc is a no-op function that never returns an error.
func popProcessNoopFunc(_ interface{}) error {
	return nil
}

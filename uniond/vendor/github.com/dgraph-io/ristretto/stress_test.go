package ristretto

import (
	"container/heap"
	"fmt"
	"math/rand"
	"runtime"
	"sync"
	"testing"
	"time"

	"github.com/dgraph-io/ristretto/sim"
	"github.com/stretchr/testify/require"
)

func TestStressSetGet(t *testing.T) {
	c, err := NewCache(&Config[int, int]{
		NumCounters:        1000,
		MaxCost:            100,
		IgnoreInternalCost: true,
		BufferItems:        64,
		Metrics:            true,
	})
	require.NoError(t, err)

	for i := 0; i < 100; i++ {
		c.Set(i, i, 1)
	}
	time.Sleep(wait)
	wg := &sync.WaitGroup{}
	for i := 0; i < runtime.GOMAXPROCS(0); i++ {
		wg.Add(1)
		go func() {
			r := rand.New(rand.NewSource(time.Now().UnixNano()))
			for a := 0; a < 1000; a++ {
				k := r.Int() % 10
				if val, ok := c.Get(k); !ok {
					err = fmt.Errorf("expected %d but got nil", k)
					break
				} else if val != 0 && val != k {
					err = fmt.Errorf("expected %d but got %d", k, val)
					break
				}
			}
			wg.Done()
		}()
	}
	wg.Wait()
	require.NoError(t, err)
	require.Equal(t, 1.0, c.Metrics.Ratio())
}

func TestStressHitRatio(t *testing.T) {
	key := sim.NewZipfian(1.0001, 1, 1000)
	c, err := NewCache(&Config[uint64, uint64]{
		NumCounters: 1000,
		MaxCost:     100,
		BufferItems: 64,
		Metrics:     true,
	})
	require.NoError(t, err)

	o := NewClairvoyant(100)
	for i := 0; i < 10000; i++ {
		k, err := key()
		require.NoError(t, err)

		if _, ok := o.Get(k); !ok {
			o.Set(k, k, 1)
		}
		if _, ok := c.Get(k); !ok {
			c.Set(k, k, 1)
		}
	}
	t.Logf("actual: %.2f, optimal: %.2f", c.Metrics.Ratio(), o.Metrics().Ratio())
}

// Clairvoyant is a mock cache providing us with optimal hit ratios to compare
// with Ristretto's. It looks ahead and evicts the absolute least valuable item,
// which we try to approximate in a real cache.
type Clairvoyant struct {
	capacity uint64
	hits     map[uint64]uint64
	access   []uint64
}

func NewClairvoyant(capacity uint64) *Clairvoyant {
	return &Clairvoyant{
		capacity: capacity,
		hits:     make(map[uint64]uint64),
		access:   make([]uint64, 0),
	}
}

// Get just records the cache access so that we can later take this event into
// consideration when calculating the absolute least valuable item to evict.
func (c *Clairvoyant) Get(key interface{}) (interface{}, bool) {
	c.hits[key.(uint64)]++
	c.access = append(c.access, key.(uint64))
	return nil, false
}

// Set isn't important because it is only called after a Get (in the case of our
// hit ratio benchmarks, at least).
func (c *Clairvoyant) Set(key, value interface{}, cost int64) bool {
	return false
}

func (c *Clairvoyant) Metrics() *Metrics {
	stat := newMetrics()
	look := make(map[uint64]struct{}, c.capacity)
	data := &clairvoyantHeap{}
	heap.Init(data)
	for _, key := range c.access {
		if _, has := look[key]; has {
			stat.add(hit, 0, 1)
			continue
		}
		if uint64(data.Len()) >= c.capacity {
			victim := heap.Pop(data)
			delete(look, victim.(*clairvoyantItem).key)
		}
		stat.add(miss, 0, 1)
		look[key] = struct{}{}
		heap.Push(data, &clairvoyantItem{key, c.hits[key]})
	}
	return stat
}

type clairvoyantItem struct {
	key  uint64
	hits uint64
}

type clairvoyantHeap []*clairvoyantItem

func (h clairvoyantHeap) Len() int           { return len(h) }
func (h clairvoyantHeap) Less(i, j int) bool { return h[i].hits < h[j].hits }
func (h clairvoyantHeap) Swap(i, j int)      { h[i], h[j] = h[j], h[i] }

func (h *clairvoyantHeap) Push(x interface{}) {
	*h = append(*h, x.(*clairvoyantItem))
}

func (h *clairvoyantHeap) Pop() interface{} {
	old := *h
	n := len(old)
	x := old[n-1]
	*h = old[0 : n-1]
	return x
}

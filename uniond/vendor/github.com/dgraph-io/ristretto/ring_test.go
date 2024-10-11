package ristretto

import (
	"sync"
	"testing"

	"github.com/stretchr/testify/require"
)

type testConsumer struct {
	push func([]uint64)
	save bool
}

func (c *testConsumer) Push(items []uint64) bool {
	if c.save {
		c.push(items)
		return true
	}
	return false
}

func TestRingDrain(t *testing.T) {
	drains := 0
	r := newRingBuffer(&testConsumer{
		push: func(items []uint64) {
			drains++
		},
		save: true,
	}, 1)
	for i := 0; i < 100; i++ {
		r.Push(uint64(i))
	}
	require.Equal(t, 100, drains, "buffers shouldn't be dropped with BufferItems == 1")
}

func TestRingReset(t *testing.T) {
	drains := 0
	r := newRingBuffer(&testConsumer{
		push: func(items []uint64) {
			drains++
		},
		save: false,
	}, 4)
	for i := 0; i < 100; i++ {
		r.Push(uint64(i))
	}
	require.Equal(t, 0, drains, "testConsumer shouldn't be draining")
}

func TestRingConsumer(t *testing.T) {
	mu := &sync.Mutex{}
	drainItems := make(map[uint64]struct{})
	r := newRingBuffer(&testConsumer{
		push: func(items []uint64) {
			mu.Lock()
			defer mu.Unlock()
			for i := range items {
				drainItems[items[i]] = struct{}{}
			}
		},
		save: true,
	}, 4)
	for i := 0; i < 100; i++ {
		r.Push(uint64(i))
	}
	l := len(drainItems)
	require.NotEqual(t, 0, l)
	require.True(t, l <= 100)
}

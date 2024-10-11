package transition

import (
	"bytes"
	"context"
	"sync"

	"github.com/pkg/errors"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	types "github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
)

type nextSlotCache struct {
	sync.Mutex
	prevRoot  []byte
	lastRoot  []byte
	prevState state.BeaconState
	lastState state.BeaconState
}

var (
	nsc nextSlotCache
	// Metrics for the validator cache.
	nextSlotCacheHit = promauto.NewCounter(prometheus.CounterOpts{
		Name: "next_slot_cache_hit",
		Help: "The total number of cache hits on the next slot state cache.",
	})
	nextSlotCacheMiss = promauto.NewCounter(prometheus.CounterOpts{
		Name: "next_slot_cache_miss",
		Help: "The total number of cache misses on the next slot state cache.",
	})
)

// NextSlotState returns the saved state for the given blockroot.
// It returns the last updated state if it matches. Otherwise it returns the previously
// updated state if it matches its root. If no root matches it returns nil
func NextSlotState(root []byte, wantedSlot types.Slot) state.BeaconState {
	nsc.Lock()
	defer nsc.Unlock()
	if bytes.Equal(root, nsc.lastRoot) && nsc.lastState.Slot() <= wantedSlot {
		nextSlotCacheHit.Inc()
		return nsc.lastState.Copy()
	}
	if bytes.Equal(root, nsc.prevRoot) && nsc.prevState.Slot() <= wantedSlot {
		nextSlotCacheHit.Inc()
		return nsc.prevState.Copy()
	}
	nextSlotCacheMiss.Inc()
	return nil
}

// UpdateNextSlotCache updates the `nextSlotCache`. It saves the input state after advancing the state slot by 1
// by calling `ProcessSlots`, it also saves the input root for later look up.
// This is useful to call after successfully processing a block.
func UpdateNextSlotCache(ctx context.Context, root []byte, state state.BeaconState) error {
	// Advancing one slot by using a copied state.
	copied := state.Copy()
	copied, err := ProcessSlots(ctx, copied, copied.Slot()+1)
	if err != nil {
		return errors.Wrap(err, "could not process slots")
	}

	nsc.Lock()
	defer nsc.Unlock()

	nsc.prevRoot = nsc.lastRoot
	nsc.prevState = nsc.lastState
	nsc.lastRoot = bytesutil.SafeCopyBytes(root)
	nsc.lastState = copied
	return nil
}

// LastCachedState returns the last cached state and root in the cache
func LastCachedState() ([]byte, state.BeaconState) {
	nsc.Lock()
	defer nsc.Unlock()
	if nsc.lastState == nil {
		return nil, nil
	}
	return bytesutil.SafeCopyBytes(nsc.lastRoot), nsc.lastState.Copy()
}

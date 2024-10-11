/*
 *
 * Copyright 2021 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package rls

import (
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"google.golang.org/grpc/internal/backoff"
	"google.golang.org/grpc/internal/testutils/stats"
)

var (
	cacheKeys = []cacheKey{
		{path: "0", keys: "a"},
		{path: "1", keys: "b"},
		{path: "2", keys: "c"},
		{path: "3", keys: "d"},
		{path: "4", keys: "e"},
	}

	longDuration  = 10 * time.Minute
	shortDuration = 1 * time.Millisecond
	cacheEntries  []*cacheEntry
)

func initCacheEntries() {
	// All entries have a dummy size of 1 to simplify resize operations.
	cacheEntries = []*cacheEntry{
		{
			// Entry is valid and minimum expiry time has not expired.
			expiryTime:        time.Now().Add(longDuration),
			earliestEvictTime: time.Now().Add(longDuration),
			size:              1,
		},
		{
			// Entry is valid and is in backoff.
			expiryTime:   time.Now().Add(longDuration),
			backoffTime:  time.Now().Add(longDuration),
			backoffState: &backoffState{timer: time.NewTimer(longDuration)},
			size:         1,
		},
		{
			// Entry is valid, and not in backoff.
			expiryTime: time.Now().Add(longDuration),
			size:       1,
		},
		{
			// Entry is invalid.
			expiryTime: time.Time{}.Add(shortDuration),
			size:       1,
		},
		{
			// Entry is invalid valid and backoff has expired.
			expiryTime:        time.Time{}.Add(shortDuration),
			backoffExpiryTime: time.Time{}.Add(shortDuration),
			size:              1,
		},
	}
}

func (s) TestLRU_BasicOperations(t *testing.T) {
	initCacheEntries()
	// Create an LRU and add some entries to it.
	lru := newLRU()
	for _, k := range cacheKeys {
		lru.addEntry(k)
	}

	// Get the least recent entry. This should be the first entry we added.
	if got, want := lru.getLeastRecentlyUsed(), cacheKeys[0]; got != want {
		t.Fatalf("lru.getLeastRecentlyUsed() = %v, want %v", got, want)
	}

	// Iterate through the slice of keys we added earlier, making them the most
	// recent entry, one at a time. The least recent entry at that point should
	// be the next entry from our slice of keys.
	for i, k := range cacheKeys {
		lru.makeRecent(k)

		lruIndex := (i + 1) % len(cacheKeys)
		if got, want := lru.getLeastRecentlyUsed(), cacheKeys[lruIndex]; got != want {
			t.Fatalf("lru.getLeastRecentlyUsed() = %v, want %v", got, want)
		}
	}

	// Iterate through the slice of keys we added earlier, removing them one at
	// a time The least recent entry at that point should be the next entry from
	// our slice of keys, except for the last one because the lru will be empty.
	for i, k := range cacheKeys {
		lru.removeEntry(k)

		var want cacheKey
		if i < len(cacheKeys)-1 {
			want = cacheKeys[i+1]
		}
		if got := lru.getLeastRecentlyUsed(); got != want {
			t.Fatalf("lru.getLeastRecentlyUsed() = %v, want %v", got, want)
		}
	}
}

func (s) TestDataCache_BasicOperations(t *testing.T) {
	initCacheEntries()
	dc := newDataCache(5, nil, &stats.NoopMetricsRecorder{}, "")
	for i, k := range cacheKeys {
		dc.addEntry(k, cacheEntries[i])
	}
	for i, k := range cacheKeys {
		entry := dc.getEntry(k)
		if !cmp.Equal(entry, cacheEntries[i], cmp.AllowUnexported(cacheEntry{}, backoffState{}), cmpopts.IgnoreUnexported(time.Timer{})) {
			t.Fatalf("Data cache lookup for key %v returned entry %v, want %v", k, entry, cacheEntries[i])
		}
	}
}

func (s) TestDataCache_AddForcesResize(t *testing.T) {
	initCacheEntries()
	dc := newDataCache(1, nil, &stats.NoopMetricsRecorder{}, "")

	// The first entry in cacheEntries has a minimum expiry time in the future.
	// This entry would stop the resize operation since we do not evict entries
	// whose minimum expiration time is in the future. So, we do not use that
	// entry in this test. The entry being added has a running backoff timer.
	evicted, ok := dc.addEntry(cacheKeys[1], cacheEntries[1])
	if evicted || !ok {
		t.Fatalf("dataCache.addEntry() returned (%v, %v) want (false, true)", evicted, ok)
	}

	// Add another entry leading to the eviction of the above entry which has a
	// running backoff timer. The first return value is expected to be true.
	backoffCancelled, ok := dc.addEntry(cacheKeys[2], cacheEntries[2])
	if !backoffCancelled || !ok {
		t.Fatalf("dataCache.addEntry() returned (%v, %v) want (true, true)", backoffCancelled, ok)
	}

	// Add another entry leading to the eviction of the above entry which does not
	// have a running backoff timer. This should evict the above entry, but the
	// first return value is expected to be false.
	backoffCancelled, ok = dc.addEntry(cacheKeys[3], cacheEntries[3])
	if backoffCancelled || !ok {
		t.Fatalf("dataCache.addEntry() returned (%v, %v) want (false, true)", backoffCancelled, ok)
	}
}

func (s) TestDataCache_Resize(t *testing.T) {
	initCacheEntries()
	dc := newDataCache(5, nil, &stats.NoopMetricsRecorder{}, "")
	for i, k := range cacheKeys {
		dc.addEntry(k, cacheEntries[i])
	}

	// The first cache entry (with a key of cacheKeys[0]) that we added has an
	// earliestEvictTime in the future. As part of the resize operation, we
	// traverse the cache in least recently used order, and this will be first
	// entry that we will encounter. And since the earliestEvictTime is in the
	// future, the resize operation will stop, leaving the cache bigger than
	// what was asked for.
	if dc.resize(1) {
		t.Fatalf("dataCache.resize() returned true, want false")
	}
	if dc.currentSize != 5 {
		t.Fatalf("dataCache.size is %d, want 5", dc.currentSize)
	}

	// Remove the entry with earliestEvictTime in the future and retry the
	// resize operation.
	dc.removeEntryForTesting(cacheKeys[0])
	if !dc.resize(1) {
		t.Fatalf("dataCache.resize() returned false, want true")
	}
	if dc.currentSize != 1 {
		t.Fatalf("dataCache.size is %d, want 1", dc.currentSize)
	}
}

func (s) TestDataCache_EvictExpiredEntries(t *testing.T) {
	initCacheEntries()
	dc := newDataCache(5, nil, &stats.NoopMetricsRecorder{}, "")
	for i, k := range cacheKeys {
		dc.addEntry(k, cacheEntries[i])
	}

	// The last two entries in the cacheEntries list have expired, and will be
	// evicted. The first three should still remain in the cache.
	if !dc.evictExpiredEntries() {
		t.Fatal("dataCache.evictExpiredEntries() returned false, want true")
	}
	if dc.currentSize != 3 {
		t.Fatalf("dataCache.size is %d, want 3", dc.currentSize)
	}
	for i := 0; i < 3; i++ {
		entry := dc.getEntry(cacheKeys[i])
		if !cmp.Equal(entry, cacheEntries[i], cmp.AllowUnexported(cacheEntry{}, backoffState{}), cmpopts.IgnoreUnexported(time.Timer{})) {
			t.Fatalf("Data cache lookup for key %v returned entry %v, want %v", cacheKeys[i], entry, cacheEntries[i])
		}
	}
}

func (s) TestDataCache_ResetBackoffState(t *testing.T) {
	type fakeBackoff struct {
		backoff.Strategy
	}

	initCacheEntries()
	dc := newDataCache(5, nil, &stats.NoopMetricsRecorder{}, "")
	for i, k := range cacheKeys {
		dc.addEntry(k, cacheEntries[i])
	}

	newBackoffState := &backoffState{bs: &fakeBackoff{}}
	if updatePicker := dc.resetBackoffState(newBackoffState); !updatePicker {
		t.Fatal("dataCache.resetBackoffState() returned updatePicker is false, want true")
	}

	// Make sure that the entry with no backoff state was not touched.
	if entry := dc.getEntry(cacheKeys[0]); cmp.Equal(entry.backoffState, newBackoffState, cmp.AllowUnexported(backoffState{})) {
		t.Fatal("dataCache.resetBackoffState() touched entries without a valid backoffState")
	}

	// Make sure that the entry with a valid backoff state was reset.
	entry := dc.getEntry(cacheKeys[1])
	if diff := cmp.Diff(entry.backoffState, newBackoffState, cmp.AllowUnexported(backoffState{})); diff != "" {
		t.Fatalf("unexpected diff in backoffState for cache entry after dataCache.resetBackoffState(): %s", diff)
	}
}

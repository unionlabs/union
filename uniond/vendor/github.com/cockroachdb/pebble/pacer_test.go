// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"fmt"
	"math"
	"math/rand"
	"sort"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

func TestDeletionPacer(t *testing.T) {
	const MB = 1 << 20
	const GB = 1 << 30
	testCases := []struct {
		freeBytes     uint64
		obsoleteBytes uint64
		liveBytes     uint64
		// history of deletion reporting; first value in the pair is the time,
		// second value is the deleted bytes. The time of pacing is the same as the
		// last time in the history.
		history [][2]int64
		// expected pacing rate in MB/s.
		expected float64
	}{
		{
			freeBytes:     160 * GB,
			obsoleteBytes: 1 * MB,
			liveBytes:     160 * MB,
			expected:      100.0,
		},
		// As freeBytes is 2GB below the free space threshold, rate should be
		// increased by 204.8MB/s.
		{
			freeBytes:     14 * GB,
			obsoleteBytes: 1 * MB,
			liveBytes:     100 * MB,
			expected:      304.8,
		},
		// As freeBytes is 10GB below the free space threshold, rate should be
		// increased to by 1GB/s.
		{
			freeBytes:     6 * GB,
			obsoleteBytes: 1 * MB,
			liveBytes:     100 * MB,
			expected:      1124.0,
		},
		// obsoleteBytesRatio is 50%. We need to delete 30GB within 5 minutes.
		{
			freeBytes:     500 * GB,
			obsoleteBytes: 50 * GB,
			liveBytes:     100 * GB,
			expected:      202.4,
		},
		// When obsolete ratio unknown, there should be no throttling.
		{
			freeBytes:     500 * GB,
			obsoleteBytes: 0,
			liveBytes:     0,
			expected:      math.Inf(1),
		},
		// History shows 200MB/sec deletions on average over last 5 minutes.
		{
			freeBytes:     160 * GB,
			obsoleteBytes: 1 * MB,
			liveBytes:     160 * MB,
			history:       [][2]int64{{0, 5 * 60 * 200 * MB}},
			expected:      200.0,
		},
		// History shows 200MB/sec deletions on average over last 5 minutes and
		// freeBytes is 10GB below the threshold.
		{
			freeBytes:     6 * GB,
			obsoleteBytes: 1 * MB,
			liveBytes:     160 * MB,
			history:       [][2]int64{{0, 5 * 60 * 200 * MB}},
			expected:      1224.0,
		},
		// History shows 200MB/sec deletions on average over last 5 minutes and
		// obsoleteBytesRatio is 50%.
		{
			freeBytes:     500 * GB,
			obsoleteBytes: 50 * GB,
			liveBytes:     100 * GB,
			history:       [][2]int64{{0, 5 * 60 * 200 * MB}},
			expected:      302.4,
		},
		// History shows 1000MB/sec deletions on average over last 5 minutes.
		{
			freeBytes:     160 * GB,
			obsoleteBytes: 1 * MB,
			liveBytes:     160 * MB,
			history:       [][2]int64{{0, 60 * 1000 * MB}, {3 * 60, 60 * 4 * 1000 * MB}, {4 * 60, 0}},
			expected:      1000.0,
		},
		// First entry in history is too old, it should be discarded.
		{
			freeBytes:     160 * GB,
			obsoleteBytes: 1 * MB,
			liveBytes:     160 * MB,
			history:       [][2]int64{{0, 10 * 60 * 10000 * MB}, {3 * 60, 4 * 60 * 200 * MB}, {7 * 60, 1 * 60 * 200 * MB}},
			expected:      200.0,
		},
	}
	for tcIdx, tc := range testCases {
		t.Run(fmt.Sprintf("%d", tcIdx), func(t *testing.T) {
			getInfo := func() deletionPacerInfo {
				return deletionPacerInfo{
					freeBytes:     tc.freeBytes,
					liveBytes:     tc.liveBytes,
					obsoleteBytes: tc.obsoleteBytes,
				}
			}
			start := time.Now()
			last := start
			pacer := newDeletionPacer(start, 100*MB, getInfo)
			for _, h := range tc.history {
				last = start.Add(time.Second * time.Duration(h[0]))
				pacer.ReportDeletion(last, uint64(h[1]))
			}
			result := 1.0 / pacer.PacingDelay(last, 1*MB)
			require.InDelta(t, tc.expected, result, 1e-7)
		})
	}
}

// TestDeletionPacerHistory tests the history helper by crosschecking Sum()
// against a naive implementation.
func TestDeletionPacerHistory(t *testing.T) {
	type event struct {
		time time.Time
		// If report is 0, this event is a Sum(). Otherwise it is an Add().
		report int64
	}
	numEvents := 1 + rand.Intn(200)
	timeframe := time.Duration(1+rand.Intn(60*100)) * time.Second
	events := make([]event, numEvents)
	startTime := time.Now()
	for i := range events {
		events[i].time = startTime.Add(time.Duration(rand.Int63n(int64(timeframe))))
		if rand.Intn(3) == 0 {
			events[i].report = 0
		} else {
			events[i].report = int64(rand.Intn(100000))
		}
	}
	sort.Slice(events, func(i, j int) bool {
		return events[i].time.Before(events[j].time)
	})

	var h history
	h.Init(startTime, timeframe)

	// partialSums[i] := SUM_j<i events[j].report
	partialSums := make([]int64, len(events)+1)
	for i := range events {
		partialSums[i+1] = partialSums[i] + events[i].report
	}

	for i, e := range events {
		if e.report != 0 {
			h.Add(e.time, e.report)
			continue
		}

		result := h.Sum(e.time)

		// getIdx returns the largest event index <= i that is before the cutoff
		// time.
		getIdx := func(cutoff time.Time) int {
			for j := i; j >= 0; j-- {
				if events[j].time.Before(cutoff) {
					return j
				}
			}
			return -1
		}

		// Sum all report values in the last timeframe, and see if recent events
		// (allowing 1% error in the cutoff time) match the result.
		a := getIdx(e.time.Add(-timeframe * (historyEpochs + 1) / historyEpochs))
		b := getIdx(e.time.Add(-timeframe * (historyEpochs - 1) / historyEpochs))
		found := false
		for j := a; j <= b; j++ {
			if partialSums[i+1]-partialSums[j+1] == result {
				found = true
				break
			}
		}
		if !found {
			t.Fatalf("incorrect Sum() result %d; %v", result, events[a+1:i+1])
		}
	}
}

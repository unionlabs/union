// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"sync"
	"time"
)

// deletionPacerInfo contains any info from the db necessary to make deletion
// pacing decisions (to limit background IO usage so that it does not contend
// with foreground traffic).
type deletionPacerInfo struct {
	freeBytes     uint64
	obsoleteBytes uint64
	liveBytes     uint64
}

// deletionPacer rate limits deletions of obsolete files. This is necessary to
// prevent overloading the disk with too many deletions too quickly after a
// large compaction, or an iterator close. On some SSDs, disk performance can be
// negatively impacted if too many blocks are deleted very quickly, so this
// mechanism helps mitigate that.
type deletionPacer struct {
	// If there are less than freeSpaceThreshold bytes of free space on
	// disk, increase the pace of deletions such that we delete enough bytes to
	// get back to the threshold within the freeSpaceTimeframe.
	freeSpaceThreshold uint64
	freeSpaceTimeframe time.Duration

	// If the ratio of obsolete bytes to live bytes is greater than
	// obsoleteBytesMaxRatio, increase the pace of deletions such that we delete
	// enough bytes to get back to the threshold within the obsoleteBytesTimeframe.
	obsoleteBytesMaxRatio  float64
	obsoleteBytesTimeframe time.Duration

	mu struct {
		sync.Mutex

		// history keeps rack of recent deletion history; it used to increase the
		// deletion rate to match the pace of deletions.
		history history
	}

	targetByteDeletionRate int64

	getInfo func() deletionPacerInfo
}

const deletePacerHistory = 5 * time.Minute

// newDeletionPacer instantiates a new deletionPacer for use when deleting
// obsolete files.
//
// targetByteDeletionRate is the rate (in bytes/sec) at which we want to
// normally limit deletes (when we are not falling behind or running out of
// space). A value of 0.0 disables pacing.
func newDeletionPacer(
	now time.Time, targetByteDeletionRate int64, getInfo func() deletionPacerInfo,
) *deletionPacer {
	d := &deletionPacer{
		freeSpaceThreshold: 16 << 30, // 16 GB
		freeSpaceTimeframe: 10 * time.Second,

		obsoleteBytesMaxRatio:  0.20,
		obsoleteBytesTimeframe: 5 * time.Minute,

		targetByteDeletionRate: targetByteDeletionRate,
		getInfo:                getInfo,
	}
	d.mu.history.Init(now, deletePacerHistory)
	return d
}

// ReportDeletion is used to report a deletion to the pacer. The pacer uses it
// to keep track of the recent rate of deletions and potentially increase the
// deletion rate accordingly.
//
// ReportDeletion is thread-safe.
func (p *deletionPacer) ReportDeletion(now time.Time, bytesToDelete uint64) {
	p.mu.Lock()
	defer p.mu.Unlock()
	p.mu.history.Add(now, int64(bytesToDelete))
}

// PacingDelay returns the recommended pacing wait time (in seconds) for
// deleting the given number of bytes.
//
// PacingDelay is thread-safe.
func (p *deletionPacer) PacingDelay(now time.Time, bytesToDelete uint64) (waitSeconds float64) {
	if p.targetByteDeletionRate == 0 {
		// Pacing disabled.
		return 0.0
	}

	baseRate := float64(p.targetByteDeletionRate)
	// If recent deletion rate is more than our target, use that so that we don't
	// fall behind.
	historicRate := func() float64 {
		p.mu.Lock()
		defer p.mu.Unlock()
		return float64(p.mu.history.Sum(now)) / deletePacerHistory.Seconds()
	}()
	if historicRate > baseRate {
		baseRate = historicRate
	}

	// Apply heuristics to increase the deletion rate.
	var extraRate float64
	info := p.getInfo()
	if info.freeBytes <= p.freeSpaceThreshold {
		// Increase the rate so that we can free up enough bytes within the timeframe.
		extraRate = float64(p.freeSpaceThreshold-info.freeBytes) / p.freeSpaceTimeframe.Seconds()
	}
	if info.liveBytes == 0 {
		// We don't know the obsolete bytes ratio. Disable pacing altogether.
		return 0.0
	}
	obsoleteBytesRatio := float64(info.obsoleteBytes) / float64(info.liveBytes)
	if obsoleteBytesRatio >= p.obsoleteBytesMaxRatio {
		// Increase the rate so that we can free up enough bytes within the timeframe.
		r := (obsoleteBytesRatio - p.obsoleteBytesMaxRatio) * float64(info.liveBytes) / p.obsoleteBytesTimeframe.Seconds()
		if extraRate < r {
			extraRate = r
		}
	}

	return float64(bytesToDelete) / (baseRate + extraRate)
}

// history is a helper used to keep track of the recent history of a set of
// data points (in our case deleted bytes), at limited granularity.
// Specifically, we split the desired timeframe into 100 "epochs" and all times
// are effectively rounded down to the nearest epoch boundary.
type history struct {
	epochDuration time.Duration
	startTime     time.Time
	// currEpoch is the epoch of the most recent operation.
	currEpoch int64
	// val contains the recent epoch values.
	// val[currEpoch % historyEpochs] is the current epoch.
	// val[(currEpoch + 1) % historyEpochs] is the oldest epoch.
	val [historyEpochs]int64
	// sum is always equal to the sum of values in val.
	sum int64
}

const historyEpochs = 100

// Init the history helper to keep track of data over the given number of
// seconds.
func (h *history) Init(now time.Time, timeframe time.Duration) {
	*h = history{
		epochDuration: timeframe / time.Duration(historyEpochs),
		startTime:     now,
		currEpoch:     0,
		sum:           0,
	}
}

// Add adds a value for the current time.
func (h *history) Add(now time.Time, val int64) {
	h.advance(now)
	h.val[h.currEpoch%historyEpochs] += val
	h.sum += val
}

// Sum returns the sum of recent values. The result is approximate in that the
// cut-off time is within 1% of the exact one.
func (h *history) Sum(now time.Time) int64 {
	h.advance(now)
	return h.sum
}

func (h *history) epoch(t time.Time) int64 {
	return int64(t.Sub(h.startTime) / h.epochDuration)
}

// advance advances the time to the given time.
func (h *history) advance(now time.Time) {
	epoch := h.epoch(now)
	for h.currEpoch < epoch {
		h.currEpoch++
		// Forget the data for the oldest epoch.
		h.sum -= h.val[h.currEpoch%historyEpochs]
		h.val[h.currEpoch%historyEpochs] = 0
	}
}

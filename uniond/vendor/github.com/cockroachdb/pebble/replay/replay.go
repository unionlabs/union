// Copyright 2022 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

// Package replay implements collection and replaying of compaction benchmarking
// workloads. A workload is a collection of flushed and ingested sstables, along
// with the corresponding manifests describing the order and grouping with which
// they were applied. Replaying a workload flushes and ingests the same keys and
// sstables to reproduce the write workload for the purpose of evaluating
// compaction heuristics.
package replay

import (
	"context"
	"encoding/binary"
	"fmt"
	"io"
	"os"
	"sort"
	"strings"
	"sync"
	"sync/atomic"
	"time"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/bytealloc"
	"github.com/cockroachdb/pebble/internal/manifest"
	"github.com/cockroachdb/pebble/internal/rangedel"
	"github.com/cockroachdb/pebble/internal/rangekey"
	"github.com/cockroachdb/pebble/record"
	"github.com/cockroachdb/pebble/sstable"
	"github.com/cockroachdb/pebble/vfs"
	"golang.org/x/perf/benchfmt"
	"golang.org/x/sync/errgroup"
)

// A Pacer paces replay of a workload, determining when to apply the next
// incoming write.
type Pacer interface {
	pace(r *Runner, step workloadStep) time.Duration
}

// computeReadAmp calculates the read amplification from a manifest.Version
func computeReadAmp(v *manifest.Version) int {
	refRAmp := v.L0Sublevels.ReadAmplification()
	for _, lvl := range v.Levels[1:] {
		if !lvl.Empty() {
			refRAmp++
		}
	}
	return refRAmp
}

// waitForReadAmpLE is a common function used by PaceByReferenceReadAmp and
// PaceByFixedReadAmp to wait on the dbMetricsNotifier condition variable if the
// read amplification observed is greater than the specified target (refRAmp).
func waitForReadAmpLE(r *Runner, rAmp int) {
	r.dbMetricsCond.L.Lock()
	m := r.dbMetrics
	ra := m.ReadAmp()
	for ra > rAmp {
		r.dbMetricsCond.Wait()
		ra = r.dbMetrics.ReadAmp()
	}
	r.dbMetricsCond.L.Unlock()
}

// Unpaced implements Pacer by applying each new write as soon as possible. It
// may be useful for examining performance under high read amplification.
type Unpaced struct{}

func (Unpaced) pace(*Runner, workloadStep) (d time.Duration) { return }

// PaceByReferenceReadAmp implements Pacer by applying each new write following
// the collected workloads read amplification.
type PaceByReferenceReadAmp struct{}

func (PaceByReferenceReadAmp) pace(r *Runner, w workloadStep) time.Duration {
	startTime := time.Now()
	refRAmp := computeReadAmp(w.pv)
	waitForReadAmpLE(r, refRAmp)
	return time.Since(startTime)
}

// PaceByFixedReadAmp implements Pacer by applying each new write following a
// fixed read amplification.
type PaceByFixedReadAmp int

func (pra PaceByFixedReadAmp) pace(r *Runner, _ workloadStep) time.Duration {
	startTime := time.Now()
	waitForReadAmpLE(r, int(pra))
	return time.Since(startTime)
}

// Metrics holds the various statistics on a replay run and its performance.
type Metrics struct {
	CompactionCounts struct {
		Total       int64
		Default     int64
		DeleteOnly  int64
		ElisionOnly int64
		Move        int64
		Read        int64
		Rewrite     int64
		MultiLevel  int64
	}
	EstimatedDebt SampledMetric
	Final         *pebble.Metrics
	Ingest        struct {
		BytesIntoL0 uint64
		// BytesWeightedByLevel is calculated as the number of bytes ingested
		// into a level multiplied by the level's distance from the bottommost
		// level (L6), summed across all levels. It can be used to guage how
		// effective heuristics are at ingesting files into lower levels, saving
		// write amplification.
		BytesWeightedByLevel uint64
	}
	// PaceDuration is the time waiting for the pacer to allow the workload to
	// continue.
	PaceDuration time.Duration
	ReadAmp      SampledMetric
	// QuiesceDuration is the time between completing application of the workload and
	// compactions quiescing.
	QuiesceDuration time.Duration
	TombstoneCount  SampledMetric
	// TotalSize holds the total size of the database, sampled after each
	// workload step.
	TotalSize           SampledMetric
	TotalWriteAmp       float64
	WorkloadDuration    time.Duration
	WriteBytes          uint64
	WriteStalls         map[string]int
	WriteStallsDuration map[string]time.Duration
	WriteThroughput     SampledMetric
}

// Plot holds an ascii plot and its name.
type Plot struct {
	Name string
	Plot string
}

// Plots returns a slice of ascii plots describing metrics change over time.
func (m *Metrics) Plots(width, height int) []Plot {
	const scaleMB = 1.0 / float64(1<<20)
	return []Plot{
		{Name: "Write throughput (MB/s)", Plot: m.WriteThroughput.PlotIncreasingPerSec(width, height, scaleMB)},
		{Name: "Estimated compaction debt (MB)", Plot: m.EstimatedDebt.Plot(width, height, scaleMB)},
		{Name: "Total database size (MB)", Plot: m.TotalSize.Plot(width, height, scaleMB)},
		{Name: "ReadAmp", Plot: m.ReadAmp.Plot(width, height, 1.0)},
	}
}

// WriteBenchmarkString writes the metrics in the form of a series of
// 'Benchmark' lines understandable by benchstat.
func (m *Metrics) WriteBenchmarkString(name string, w io.Writer) error {
	type benchmarkSection struct {
		label  string
		values []benchfmt.Value
	}
	groups := []benchmarkSection{
		{label: "CompactionCounts", values: []benchfmt.Value{
			{Value: float64(m.CompactionCounts.Total), Unit: "compactions"},
			{Value: float64(m.CompactionCounts.Default), Unit: "default"},
			{Value: float64(m.CompactionCounts.DeleteOnly), Unit: "delete"},
			{Value: float64(m.CompactionCounts.ElisionOnly), Unit: "elision"},
			{Value: float64(m.CompactionCounts.Move), Unit: "move"},
			{Value: float64(m.CompactionCounts.Read), Unit: "read"},
			{Value: float64(m.CompactionCounts.Rewrite), Unit: "rewrite"},
			{Value: float64(m.CompactionCounts.MultiLevel), Unit: "multilevel"},
		}},
		// Total database sizes sampled after every workload step and
		// compaction. This can be used to evaluate the relative LSM space
		// amplification between runs of the same workload. Calculating the true
		// space amplification continuously is prohibitvely expensive (it
		// requires totally compacting a copy of the LSM).
		{label: "DatabaseSize/mean", values: []benchfmt.Value{
			{Value: m.TotalSize.Mean(), Unit: "bytes"},
		}},
		{label: "DatabaseSize/max", values: []benchfmt.Value{
			{Value: float64(m.TotalSize.Max()), Unit: "bytes"},
		}},
		// Time applying the workload and time waiting for compactions to
		// quiesce after the workload has completed.
		{label: "DurationWorkload", values: []benchfmt.Value{
			{Value: m.WorkloadDuration.Seconds(), Unit: "sec/op"},
		}},
		{label: "DurationQuiescing", values: []benchfmt.Value{
			{Value: m.QuiesceDuration.Seconds(), Unit: "sec/op"},
		}},
		{label: "DurationPaceDelay", values: []benchfmt.Value{
			{Value: m.PaceDuration.Seconds(), Unit: "sec/op"},
		}},
		// Estimated compaction debt, sampled after every workload step and
		// compaction.
		{label: "EstimatedDebt/mean", values: []benchfmt.Value{
			{Value: m.EstimatedDebt.Mean(), Unit: "bytes"},
		}},
		{label: "EstimatedDebt/max", values: []benchfmt.Value{
			{Value: float64(m.EstimatedDebt.Max()), Unit: "bytes"},
		}},
		{label: "FlushUtilization", values: []benchfmt.Value{
			{Value: m.Final.Flush.WriteThroughput.Utilization(), Unit: "util"},
		}},
		{label: "IngestedIntoL0", values: []benchfmt.Value{
			{Value: float64(m.Ingest.BytesIntoL0), Unit: "bytes"},
		}},
		{label: "IngestWeightedByLevel", values: []benchfmt.Value{
			{Value: float64(m.Ingest.BytesWeightedByLevel), Unit: "bytes"},
		}},
		{label: "ReadAmp/mean", values: []benchfmt.Value{
			{Value: m.ReadAmp.Mean(), Unit: "files"},
		}},
		{label: "ReadAmp/max", values: []benchfmt.Value{
			{Value: float64(m.ReadAmp.Max()), Unit: "files"},
		}},
		{label: "TombstoneCount/mean", values: []benchfmt.Value{
			{Value: m.TombstoneCount.Mean(), Unit: "tombstones"},
		}},
		{label: "TombstoneCount/max", values: []benchfmt.Value{
			{Value: float64(m.TombstoneCount.Max()), Unit: "tombstones"},
		}},
		{label: "Throughput", values: []benchfmt.Value{
			{Value: float64(m.WriteBytes) / (m.WorkloadDuration + m.QuiesceDuration).Seconds(), Unit: "B/s"},
		}},
		{label: "WriteAmp", values: []benchfmt.Value{
			{Value: float64(m.TotalWriteAmp), Unit: "wamp"},
		}},
	}

	for _, reason := range []string{"L0", "memtable"} {
		groups = append(groups, benchmarkSection{
			label: fmt.Sprintf("WriteStall/%s", reason),
			values: []benchfmt.Value{
				{Value: float64(m.WriteStalls[reason]), Unit: "stalls"},
				{Value: float64(m.WriteStallsDuration[reason].Seconds()), Unit: "stallsec/op"},
			},
		})
	}

	bw := benchfmt.NewWriter(w)
	for _, grp := range groups {
		err := bw.Write(&benchfmt.Result{
			Name:   benchfmt.Name(fmt.Sprintf("BenchmarkReplay/%s/%s", name, grp.label)),
			Iters:  1,
			Values: grp.values,
		})
		if err != nil {
			return err
		}
	}
	return nil
}

// Runner runs a captured workload against a test database, collecting
// metrics on performance.
type Runner struct {
	RunDir        string
	WorkloadFS    vfs.FS
	WorkloadPath  string
	Pacer         Pacer
	Opts          *pebble.Options
	MaxWriteBytes uint64

	// Internal state.

	d *pebble.DB
	// dbMetrics and dbMetricsCond work in unison to update the metrics and
	// notify (broadcast) to any waiting clients that metrics have been updated.
	dbMetrics     *pebble.Metrics
	dbMetricsCond sync.Cond
	cancel        func()
	err           atomic.Value
	errgroup      *errgroup.Group
	readerOpts    sstable.ReaderOptions
	stagingDir    string
	steps         chan workloadStep
	stepsApplied  chan workloadStep

	metrics struct {
		estimatedDebt    SampledMetric
		quiesceDuration  time.Duration
		readAmp          SampledMetric
		tombstoneCount   SampledMetric
		totalSize        SampledMetric
		paceDurationNano atomic.Uint64
		workloadDuration time.Duration
		writeBytes       atomic.Uint64
		writeThroughput  SampledMetric
	}
	writeStallMetrics struct {
		sync.Mutex
		countByReason    map[string]int
		durationByReason map[string]time.Duration
	}
	// compactionMu holds state for tracking the number of compactions
	// started and completed and waking waiting goroutines when a new compaction
	// completes. See nextCompactionCompletes.
	compactionMu struct {
		sync.Mutex
		ch        chan struct{}
		started   int64
		completed int64
	}
	workload struct {
		manifests []string
		// manifest{Idx,Off} record the starting position of the workload
		// relative to the initial database state.
		manifestIdx int
		manifestOff int64
		// sstables records the set of captured workload sstables by file num.
		sstables map[base.FileNum]struct{}
	}
}

// Run begins executing the workload and returns.
//
// The workload application will respect the provided context's cancellation.
func (r *Runner) Run(ctx context.Context) error {
	// Find the workload start relative to the RunDir's existing database state.
	// A prefix of the workload's manifest edits are expected to have already
	// been applied to the checkpointed existing database state.
	var err error
	r.workload.manifests, r.workload.sstables, err = findWorkloadFiles(r.WorkloadPath, r.WorkloadFS)
	if err != nil {
		return err
	}
	r.workload.manifestIdx, r.workload.manifestOff, err = findManifestStart(r.RunDir, r.Opts.FS, r.workload.manifests)
	if err != nil {
		return err
	}

	// Set up a staging dir for files that will be ingested.
	r.stagingDir = r.Opts.FS.PathJoin(r.RunDir, "staging")
	if err := r.Opts.FS.MkdirAll(r.stagingDir, os.ModePerm); err != nil {
		return err
	}

	r.dbMetricsCond = sync.Cond{
		L: &sync.Mutex{},
	}

	// Extend the user-provided Options with extensions necessary for replay
	// mechanics.
	r.compactionMu.ch = make(chan struct{})
	r.Opts.AddEventListener(r.eventListener())
	r.writeStallMetrics.countByReason = make(map[string]int)
	r.writeStallMetrics.durationByReason = make(map[string]time.Duration)
	r.Opts.EnsureDefaults()
	r.readerOpts = r.Opts.MakeReaderOptions()
	r.Opts.DisableWAL = true
	r.d, err = pebble.Open(r.RunDir, r.Opts)
	if err != nil {
		return err
	}

	r.dbMetrics = r.d.Metrics()

	// Use a buffered channel to allow the prepareWorkloadSteps to read ahead,
	// buffering up to cap(r.steps) steps ahead of the current applied state.
	// Flushes need to be buffered and ingested sstables need to be copied, so
	// pipelining this preparation makes it more likely the step will be ready
	// to apply when the pacer decides to apply it.
	r.steps = make(chan workloadStep, 5)
	r.stepsApplied = make(chan workloadStep, 5)

	ctx, r.cancel = context.WithCancel(ctx)
	r.errgroup, ctx = errgroup.WithContext(ctx)
	r.errgroup.Go(func() error { return r.prepareWorkloadSteps(ctx) })
	r.errgroup.Go(func() error { return r.applyWorkloadSteps(ctx) })
	r.errgroup.Go(func() error { return r.refreshMetrics(ctx) })
	return nil
}

// refreshMetrics runs in its own goroutine, collecting metrics from the Pebble
// instance whenever a) a workload step completes, or b) a compaction completes.
// The Pacer implementations that pace based on read-amplification rely on these
// refreshed metrics to decide when to allow the workload to proceed.
func (r *Runner) refreshMetrics(ctx context.Context) error {
	startAt := time.Now()
	var workloadExhausted bool
	var workloadExhaustedAt time.Time
	stepsApplied := r.stepsApplied
	compactionCount, alreadyCompleted, compactionCh := r.nextCompactionCompletes(0)
	for {
		if !alreadyCompleted {
			select {
			case <-ctx.Done():
				return ctx.Err()
			case <-compactionCh:
				// Fall through to refreshing dbMetrics.
			case _, ok := <-stepsApplied:
				if !ok {
					workloadExhausted = true
					workloadExhaustedAt = time.Now()
					// Set the [stepsApplied] channel to nil so that we'll never
					// hit this case again, and we don't busy loop.
					stepsApplied = nil
					// Record the replay time.
					r.metrics.workloadDuration = workloadExhaustedAt.Sub(startAt)
				}
				// Fall through to refreshing dbMetrics.
			}
		}

		m := r.d.Metrics()
		r.dbMetricsCond.L.Lock()
		r.dbMetrics = m
		r.dbMetricsCond.Broadcast()
		r.dbMetricsCond.L.Unlock()

		// Collect sample metrics. These metrics are calculated by sampling
		// every time we collect metrics.
		r.metrics.readAmp.record(int64(m.ReadAmp()))
		r.metrics.estimatedDebt.record(int64(m.Compact.EstimatedDebt))
		r.metrics.tombstoneCount.record(int64(m.Keys.TombstoneCount))
		r.metrics.totalSize.record(int64(m.DiskSpaceUsage()))
		r.metrics.writeThroughput.record(int64(r.metrics.writeBytes.Load()))

		compactionCount, alreadyCompleted, compactionCh = r.nextCompactionCompletes(compactionCount)
		// Consider whether replaying is complete. There are two necessary
		// conditions:
		//
		//   1. The workload must be exhausted.
		//   2. Compactions must have quiesced.
		//
		// The first condition is simple. The replay tool is responsible for
		// applying the workload. The goroutine responsible for applying the
		// workload closes the `stepsApplied` channel after the last step has
		// been applied, and we'll flip `workloadExhausted` to true.
		//
		// The second condition is tricky. The replay tool doesn't control
		// compactions and doesn't have visibility into whether the compaction
		// picker is about to schedule a new compaction. We can tell when
		// compactions are in progress or may be immeninent (eg, flushes in
		// progress). If it appears that compactions have quiesced, pause for a
		// fixed duration to see if a new one is scheduled. If not, consider
		// compactions quiesced.
		if workloadExhausted && !alreadyCompleted && r.compactionsAppearQuiesced(m) {
			select {
			case <-compactionCh:
				// A new compaction just finished; compactions have not
				// quiesced.
				continue
			case <-time.After(time.Second):
				// No compactions completed. If it still looks like they've
				// quiesced according to the metrics, consider them quiesced.
				if r.compactionsAppearQuiesced(r.d.Metrics()) {
					r.metrics.quiesceDuration = time.Since(workloadExhaustedAt)
					return nil
				}
			}
		}
	}
}

// compactionsAppearQuiesced returns true if the database may have quiesced, and
// there likely won't be additional compactions scheduled. Detecting quiescence
// is a bit fraught: The various signals that Pebble makes available are
// adjusted at different points in the compaction lifecycle, and database
// mutexes are dropped and acquired between them. This makes it difficult to
// reliably identify when compactions quiesce.
//
// For example, our call to DB.Metrics() may acquire the DB.mu mutex when a
// compaction has just successfully completed, but before it's managed to
// schedule the next compaction (DB.mu is dropped while it attempts to acquire
// the manifest lock).
func (r *Runner) compactionsAppearQuiesced(m *pebble.Metrics) bool {
	r.compactionMu.Lock()
	defer r.compactionMu.Unlock()
	if m.Flush.NumInProgress > 0 {
		return false
	} else if m.Compact.NumInProgress > 0 && r.compactionMu.started != r.compactionMu.completed {
		return false
	}
	return true
}

// nextCompactionCompletes may be used to be notified when new compactions
// complete. The caller is responsible for holding on to a monotonically
// increasing count representing the number of compactions that have been
// observed, beginning at zero.
//
// The caller passes their current count as an argument. If a new compaction has
// already completed since their provided count, nextCompactionCompletes returns
// the new count and a true boolean return value. If a new compaction has not
// yet completed, it returns a channel that will be closed when the next
// compaction completes. This scheme allows the caller to select{...},
// performing some action on every compaction completion.
func (r *Runner) nextCompactionCompletes(
	lastObserved int64,
) (count int64, alreadyOccurred bool, ch chan struct{}) {
	r.compactionMu.Lock()
	defer r.compactionMu.Unlock()

	if lastObserved < r.compactionMu.completed {
		// There has already been another compaction since the last one observed
		// by this caller. Return immediately.
		return r.compactionMu.completed, true, nil
	}

	// The last observed compaction is still the most recent compaction.
	// Return a channel that the caller can wait on to be notified when the
	// next compaction occurs.
	if r.compactionMu.ch == nil {
		r.compactionMu.ch = make(chan struct{})
	}
	return lastObserved, false, r.compactionMu.ch
}

// Wait waits for the workload replay to complete. Wait returns once the entire
// workload has been replayed, and compactions have quiesced.
func (r *Runner) Wait() (Metrics, error) {
	err := r.errgroup.Wait()
	if storedErr := r.err.Load(); storedErr != nil {
		err = storedErr.(error)
	}
	pm := r.d.Metrics()
	total := pm.Total()
	var ingestBytesWeighted uint64
	for l := 0; l < len(pm.Levels); l++ {
		ingestBytesWeighted += pm.Levels[l].BytesIngested * uint64(len(pm.Levels)-l-1)
	}

	m := Metrics{
		Final:               pm,
		EstimatedDebt:       r.metrics.estimatedDebt,
		PaceDuration:        time.Duration(r.metrics.paceDurationNano.Load()),
		ReadAmp:             r.metrics.readAmp,
		QuiesceDuration:     r.metrics.quiesceDuration,
		TombstoneCount:      r.metrics.tombstoneCount,
		TotalSize:           r.metrics.totalSize,
		TotalWriteAmp:       total.WriteAmp(),
		WorkloadDuration:    r.metrics.workloadDuration,
		WriteBytes:          r.metrics.writeBytes.Load(),
		WriteStalls:         make(map[string]int),
		WriteStallsDuration: make(map[string]time.Duration),
		WriteThroughput:     r.metrics.writeThroughput,
	}

	r.writeStallMetrics.Lock()
	for reason, count := range r.writeStallMetrics.countByReason {
		m.WriteStalls[reason] = count
	}
	for reason, duration := range r.writeStallMetrics.durationByReason {
		m.WriteStallsDuration[reason] = duration
	}
	r.writeStallMetrics.Unlock()
	m.CompactionCounts.Total = pm.Compact.Count
	m.CompactionCounts.Default = pm.Compact.DefaultCount
	m.CompactionCounts.DeleteOnly = pm.Compact.DeleteOnlyCount
	m.CompactionCounts.ElisionOnly = pm.Compact.ElisionOnlyCount
	m.CompactionCounts.Move = pm.Compact.MoveCount
	m.CompactionCounts.Read = pm.Compact.ReadCount
	m.CompactionCounts.Rewrite = pm.Compact.RewriteCount
	m.CompactionCounts.MultiLevel = pm.Compact.MultiLevelCount
	m.Ingest.BytesIntoL0 = pm.Levels[0].BytesIngested
	m.Ingest.BytesWeightedByLevel = ingestBytesWeighted
	return m, err
}

// Close closes remaining open resources, including the database. It must be
// called after Wait.
func (r *Runner) Close() error {
	return r.d.Close()
}

// A workloadStep describes a single manifest edit in the workload. It may be a
// flush or ingest that should be applied to the test database, or it may be a
// compaction that is surfaced to allow the replay logic to compare against the
// state of the database at workload collection time.
type workloadStep struct {
	kind stepKind
	ve   manifest.VersionEdit
	// a Version describing the state of the LSM *before* the workload was
	// collected.
	pv *manifest.Version
	// a Version describing the state of the LSM when the workload was
	// collected.
	v *manifest.Version
	// non-nil for flushStepKind
	flushBatch           *pebble.Batch
	tablesToIngest       []string
	cumulativeWriteBytes uint64
}

type stepKind uint8

const (
	flushStepKind stepKind = iota
	ingestStepKind
	compactionStepKind
)

// eventListener returns a Pebble EventListener that is installed on the replay
// database so that the replay runner has access to internal Pebble events.
func (r *Runner) eventListener() pebble.EventListener {
	var writeStallBegin time.Time
	var writeStallReason string
	l := pebble.EventListener{
		BackgroundError: func(err error) {
			r.err.Store(err)
			r.cancel()
		},
		WriteStallBegin: func(info pebble.WriteStallBeginInfo) {
			r.writeStallMetrics.Lock()
			defer r.writeStallMetrics.Unlock()
			writeStallReason = info.Reason
			// Take just the first word of the reason.
			if j := strings.IndexByte(writeStallReason, ' '); j != -1 {
				writeStallReason = writeStallReason[:j]
			}
			switch writeStallReason {
			case "L0", "memtable":
				r.writeStallMetrics.countByReason[writeStallReason]++
			default:
				panic(fmt.Sprintf("unrecognized write stall reason %q", info.Reason))
			}
			writeStallBegin = time.Now()
		},
		WriteStallEnd: func() {
			r.writeStallMetrics.Lock()
			defer r.writeStallMetrics.Unlock()
			r.writeStallMetrics.durationByReason[writeStallReason] += time.Since(writeStallBegin)
		},
		CompactionBegin: func(_ pebble.CompactionInfo) {
			r.compactionMu.Lock()
			defer r.compactionMu.Unlock()
			r.compactionMu.started++
		},
		CompactionEnd: func(_ pebble.CompactionInfo) {
			// Keep track of the number of compactions that complete and notify
			// anyone waiting for a compaction to complete. See the function
			// nextCompactionCompletes for the corresponding receiver side.
			r.compactionMu.Lock()
			defer r.compactionMu.Unlock()
			r.compactionMu.completed++
			if r.compactionMu.ch != nil {
				// Signal that a compaction has completed.
				close(r.compactionMu.ch)
				r.compactionMu.ch = nil
			}
		},
	}
	l.EnsureDefaults(nil)
	return l
}

// applyWorkloadSteps runs in its own goroutine, reading workload steps off the
// r.steps channel and applying them to the test database.
func (r *Runner) applyWorkloadSteps(ctx context.Context) error {
	for {
		var ok bool
		var step workloadStep
		select {
		case <-ctx.Done():
			return ctx.Err()
		case step, ok = <-r.steps:
			if !ok {
				// Exhausted the workload. Exit.
				close(r.stepsApplied)
				return nil
			}
		}

		paceDur := r.Pacer.pace(r, step)
		r.metrics.paceDurationNano.Add(uint64(paceDur))

		switch step.kind {
		case flushStepKind:
			if err := step.flushBatch.Commit(&pebble.WriteOptions{Sync: false}); err != nil {
				return err
			}
			_, err := r.d.AsyncFlush()
			if err != nil {
				return err
			}
			r.metrics.writeBytes.Store(step.cumulativeWriteBytes)
			r.stepsApplied <- step
		case ingestStepKind:
			if err := r.d.Ingest(step.tablesToIngest); err != nil {
				return err
			}
			r.metrics.writeBytes.Store(step.cumulativeWriteBytes)
			r.stepsApplied <- step
		case compactionStepKind:
			// No-op.
			// TODO(jackson): Should we elide this earlier?
		default:
			panic("unreachable")
		}
	}
}

// prepareWorkloadSteps runs in its own goroutine, reading the workload
// manifests in order to reconstruct the workload and prepare each step to be
// applied. It sends each workload step to the r.steps channel.
func (r *Runner) prepareWorkloadSteps(ctx context.Context) error {
	defer func() { close(r.steps) }()

	idx := r.workload.manifestIdx

	var cumulativeWriteBytes uint64
	var flushBufs flushBuffers
	var v *manifest.Version
	var previousVersion *manifest.Version
	var bve manifest.BulkVersionEdit
	bve.AddedByFileNum = make(map[base.FileNum]*manifest.FileMetadata)
	applyVE := func(ve *manifest.VersionEdit) error {
		return bve.Accumulate(ve)
	}
	currentVersion := func() (*manifest.Version, error) {
		var err error
		v, err = bve.Apply(v,
			r.Opts.Comparer.Compare,
			r.Opts.Comparer.FormatKey,
			r.Opts.FlushSplitBytes,
			r.Opts.Experimental.ReadCompactionRate,
			nil, /* zombies */
			manifest.ProhibitSplitUserKeys)
		bve = manifest.BulkVersionEdit{AddedByFileNum: bve.AddedByFileNum}
		return v, err
	}

	for ; idx < len(r.workload.manifests); idx++ {
		if r.MaxWriteBytes != 0 && cumulativeWriteBytes > r.MaxWriteBytes {
			break
		}

		err := func() error {
			manifestName := r.workload.manifests[idx]
			f, err := r.WorkloadFS.Open(r.WorkloadFS.PathJoin(r.WorkloadPath, manifestName))
			if err != nil {
				return err
			}
			defer f.Close()

			rr := record.NewReader(f, 0 /* logNum */)
			// A manifest's first record always holds the initial version state.
			// If this is the first manifest we're examining, we load it in
			// order to seed `metas` with the file metadata of the existing
			// files. Otherwise, we can skip it because we already know all the
			// file metadatas up to this point.
			rec, err := rr.Next()
			if err != nil {
				return err
			}
			if idx == r.workload.manifestIdx {
				var ve manifest.VersionEdit
				if err := ve.Decode(rec); err != nil {
					return err
				}
				if err := applyVE(&ve); err != nil {
					return err
				}
			}

			// Read the remaining of the manifests version edits, one-by-one.
			for {
				rec, err := rr.Next()
				if err == io.EOF || record.IsInvalidRecord(err) {
					break
				} else if err != nil {
					return err
				}
				var ve manifest.VersionEdit
				if err = ve.Decode(rec); err == io.EOF || record.IsInvalidRecord(err) {
					break
				} else if err != nil {
					return err
				}
				if err := applyVE(&ve); err != nil {
					return err
				}
				if idx == r.workload.manifestIdx && rr.Offset() <= r.workload.manifestOff {
					// The record rec began at an offset strictly less than
					// rr.Offset(), which means it's strictly less than
					// r.workload.manifestOff, and we should skip it.
					continue
				}
				if len(ve.NewFiles) == 0 && len(ve.DeletedFiles) == 0 {
					// Skip WAL rotations and other events that don't affect the
					// files of the LSM.
					continue
				}

				s := workloadStep{ve: ve}
				if len(ve.DeletedFiles) > 0 {
					// If a version edit deletes files, we assume it's a compaction.
					s.kind = compactionStepKind
				} else {
					// Default to ingest. If any files have unequal
					// smallest,largest sequence numbers, we'll update this to a
					// flush.
					s.kind = ingestStepKind
				}
				var newFiles []base.DiskFileNum
				for _, nf := range ve.NewFiles {
					newFiles = append(newFiles, nf.Meta.FileBacking.DiskFileNum)
					if s.kind == ingestStepKind && (nf.Meta.SmallestSeqNum != nf.Meta.LargestSeqNum || nf.Level != 0) {
						s.kind = flushStepKind
					}
				}
				// Add the current reference *Version to the step. This provides
				// access to, for example, the read-amplification of the
				// database at this point when the workload was collected. This
				// can be useful for pacing.
				if s.v, err = currentVersion(); err != nil {
					return err
				}
				// On the first time through, we set the previous version to the current
				// version otherwise we set it to the actual previous version.
				if previousVersion == nil {
					previousVersion = s.v
				}
				s.pv = previousVersion
				previousVersion = s.v

				// It's possible that the workload collector captured this
				// version edit, but wasn't able to collect all of the
				// corresponding sstables before being terminated.
				if s.kind == flushStepKind || s.kind == ingestStepKind {
					for _, fileNum := range newFiles {
						if _, ok := r.workload.sstables[fileNum.FileNum()]; !ok {
							// TODO(jackson,leon): This isn't exactly an error
							// condition. Give this more thought; do we want to
							// require graceful exiting of workload collection,
							// such that the last version edit must have had its
							// corresponding sstables collected?
							return errors.Newf("sstable %s not found", fileNum)
						}
					}
				}

				switch s.kind {
				case flushStepKind:
					// Load all of the flushed sstables' keys into a batch.
					s.flushBatch = r.d.NewBatch()
					if err := loadFlushedSSTableKeys(s.flushBatch, r.WorkloadFS, r.WorkloadPath, newFiles, r.readerOpts, &flushBufs); err != nil {
						return errors.Wrapf(err, "flush in %q at offset %d", manifestName, rr.Offset())
					}
					cumulativeWriteBytes += uint64(s.flushBatch.Len())
				case ingestStepKind:
					// Copy the ingested sstables into a staging area within the
					// run dir. This is necessary for two reasons:
					//  a) Ingest will remove the source file, and we don't want
					//     to mutate the workload.
					//  b) If the workload stored on another volume, Ingest
					//     would need to fall back to copying the file since
					//     it's not possible to link across volumes. The true
					//     workload likely linked the file. Staging the file
					//     ahead of time ensures that we're able to Link the
					//     file like the original workload did.
					for _, fileNum := range newFiles {
						src := base.MakeFilepath(r.WorkloadFS, r.WorkloadPath, base.FileTypeTable, fileNum)
						dst := base.MakeFilepath(r.Opts.FS, r.stagingDir, base.FileTypeTable, fileNum)
						if err := vfs.CopyAcrossFS(r.WorkloadFS, src, r.Opts.FS, dst); err != nil {
							return errors.Wrapf(err, "ingest in %q at offset %d", manifestName, rr.Offset())
						}
						finfo, err := r.Opts.FS.Stat(dst)
						if err != nil {
							return errors.Wrapf(err, "stating %q", dst)
						}
						cumulativeWriteBytes += uint64(finfo.Size())
						s.tablesToIngest = append(s.tablesToIngest, dst)
					}
				case compactionStepKind:
					// Nothing to do.
				}
				s.cumulativeWriteBytes = cumulativeWriteBytes

				select {
				case <-ctx.Done():
					return ctx.Err()
				case r.steps <- s:
				}

				if r.MaxWriteBytes != 0 && cumulativeWriteBytes > r.MaxWriteBytes {
					break
				}
			}
			return nil
		}()
		if err != nil {
			return err
		}
	}
	return nil
}

// findWorkloadFiles finds all manifests and tables in the provided path on fs.
func findWorkloadFiles(
	path string, fs vfs.FS,
) (manifests []string, sstables map[base.FileNum]struct{}, err error) {
	dirents, err := fs.List(path)
	if err != nil {
		return nil, nil, err
	}
	sstables = make(map[base.FileNum]struct{})
	for _, dirent := range dirents {
		typ, fileNum, ok := base.ParseFilename(fs, dirent)
		if !ok {
			continue
		}
		switch typ {
		case base.FileTypeManifest:
			manifests = append(manifests, dirent)
		case base.FileTypeTable:
			sstables[fileNum.FileNum()] = struct{}{}
		}
	}
	if len(manifests) == 0 {
		return nil, nil, errors.Newf("no manifests found")
	}
	sort.Strings(manifests)
	return manifests, sstables, err
}

// findManifestStart takes a database directory and FS containing the initial
// database state that a workload will be run against, and a list of a workloads
// manifests. It examines the database's current manifest to determine where
// workload replay should begin, so as to not duplicate already-applied version
// edits.
//
// It returns the index of the starting manifest, and the database's current
// offset within the manifest.
func findManifestStart(
	dbDir string, dbFS vfs.FS, manifests []string,
) (index int, offset int64, err error) {
	// Identify the database's current manifest.
	dbDesc, err := pebble.Peek(dbDir, dbFS)
	if err != nil {
		return 0, 0, err
	}
	dbManifest := dbFS.PathBase(dbDesc.ManifestFilename)
	// If there is no initial database state, begin workload replay from the
	// beginning of the first manifest.
	if !dbDesc.Exists {
		return 0, 0, nil
	}
	for index = 0; index < len(manifests); index++ {
		if manifests[index] == dbManifest {
			break
		}
	}
	if index == len(manifests) {
		// The initial database state has a manifest that does not appear within
		// the workload's set of manifests. This is possible if we began
		// recording the workload at the same time as a manifest rotation, but
		// more likely we're applying a workload to a different initial database
		// state than the one from which the workload was collected. Either way,
		// start from the beginning of the first manifest.
		return 0, 0, nil
	}
	// Find the initial database's offset within the manifest.
	info, err := dbFS.Stat(dbFS.PathJoin(dbDir, dbManifest))
	if err != nil {
		return 0, 0, err
	}
	return index, info.Size(), nil
}

// loadFlushedSSTableKeys copies keys from the sstables specified by `fileNums`
// in the directory specified by `path` into the provided the batch. Keys are
// applied to the batch in the order dictated by their sequence numbers within
// the sstables, ensuring the relative relationship between sequence numbers is
// maintained.
//
// Preserving the relative relationship between sequence numbers is not strictly
// necessary, but it ensures we accurately exercise some microoptimizations (eg,
// detecting user key changes by descending trailer). There may be additional
// dependencies on sequence numbers in the future.
func loadFlushedSSTableKeys(
	b *pebble.Batch,
	fs vfs.FS,
	path string,
	fileNums []base.DiskFileNum,
	readOpts sstable.ReaderOptions,
	bufs *flushBuffers,
) error {
	// Load all the keys across all the sstables.
	for _, fileNum := range fileNums {
		if err := func() error {
			filePath := base.MakeFilepath(fs, path, base.FileTypeTable, fileNum)
			f, err := fs.Open(filePath)
			if err != nil {
				return err
			}
			readable, err := sstable.NewSimpleReadable(f)
			if err != nil {
				f.Close()
				return err
			}
			r, err := sstable.NewReader(readable, readOpts)
			if err != nil {
				return err
			}
			defer r.Close()

			// Load all the point keys.
			iter, err := r.NewIter(nil, nil)
			if err != nil {
				return err
			}
			defer iter.Close()
			for k, lv := iter.First(); k != nil; k, lv = iter.Next() {
				var key flushedKey
				key.Trailer = k.Trailer
				bufs.alloc, key.UserKey = bufs.alloc.Copy(k.UserKey)
				if v, callerOwned, err := lv.Value(nil); err != nil {
					return err
				} else if callerOwned {
					key.value = v
				} else {
					bufs.alloc, key.value = bufs.alloc.Copy(v)
				}
				bufs.keys = append(bufs.keys, key)
			}

			// Load all the range tombstones.
			if iter, err := r.NewRawRangeDelIter(); err != nil {
				return err
			} else if iter != nil {
				defer iter.Close()
				for s := iter.First(); s != nil; s = iter.Next() {
					if err := rangedel.Encode(s, func(k base.InternalKey, v []byte) error {
						var key flushedKey
						key.Trailer = k.Trailer
						bufs.alloc, key.UserKey = bufs.alloc.Copy(k.UserKey)
						bufs.alloc, key.value = bufs.alloc.Copy(v)
						bufs.keys = append(bufs.keys, key)
						return nil
					}); err != nil {
						return err
					}
				}
			}

			// Load all the range keys.
			if iter, err := r.NewRawRangeKeyIter(); err != nil {
				return err
			} else if iter != nil {
				defer iter.Close()
				for s := iter.First(); s != nil; s = iter.Next() {
					if err := rangekey.Encode(s, func(k base.InternalKey, v []byte) error {
						var key flushedKey
						key.Trailer = k.Trailer
						bufs.alloc, key.UserKey = bufs.alloc.Copy(k.UserKey)
						bufs.alloc, key.value = bufs.alloc.Copy(v)
						bufs.keys = append(bufs.keys, key)
						return nil
					}); err != nil {
						return err
					}
				}
			}
			return nil
		}(); err != nil {
			return err
		}
	}

	// Sort the flushed keys by their sequence numbers so that we can apply them
	// to the batch in the same order, maintaining the relative relationship
	// between keys.
	// NB: We use a stable sort so that keys corresponding to span fragments
	// (eg, range tombstones and range keys) have a deterministic ordering for
	// testing.
	sort.Stable(bufs.keys)

	// Add the keys to the batch in the order they were committed when the
	// workload was captured.
	for i := 0; i < len(bufs.keys); i++ {
		var err error
		switch bufs.keys[i].Kind() {
		case base.InternalKeyKindDelete:
			err = b.Delete(bufs.keys[i].UserKey, nil)
		case base.InternalKeyKindDeleteSized:
			v, _ := binary.Uvarint(bufs.keys[i].value)
			// Batch.DeleteSized takes just the length of the value being
			// deleted and adds the key's length to derive the overall entry
			// size of the value being deleted. This has already been done to
			// the key we're reading from the sstable, so we must subtract the
			// key length from the encoded value before calling b.DeleteSized,
			// which will again add the key length before encoding.
			err = b.DeleteSized(bufs.keys[i].UserKey, uint32(v-uint64(len(bufs.keys[i].UserKey))), nil)
		case base.InternalKeyKindSet, base.InternalKeyKindSetWithDelete:
			err = b.Set(bufs.keys[i].UserKey, bufs.keys[i].value, nil)
		case base.InternalKeyKindMerge:
			err = b.Merge(bufs.keys[i].UserKey, bufs.keys[i].value, nil)
		case base.InternalKeyKindSingleDelete:
			err = b.SingleDelete(bufs.keys[i].UserKey, nil)
		case base.InternalKeyKindRangeDelete:
			err = b.DeleteRange(bufs.keys[i].UserKey, bufs.keys[i].value, nil)
		case base.InternalKeyKindRangeKeySet, base.InternalKeyKindRangeKeyUnset, base.InternalKeyKindRangeKeyDelete:
			s, err := rangekey.Decode(bufs.keys[i].InternalKey, bufs.keys[i].value, nil)
			if err != nil {
				return err
			}
			if len(s.Keys) != 1 {
				return errors.Newf("range key span unexpectedly contains %d keys", len(s.Keys))
			}
			switch bufs.keys[i].Kind() {
			case base.InternalKeyKindRangeKeySet:
				err = b.RangeKeySet(s.Start, s.End, s.Keys[0].Suffix, s.Keys[0].Value, nil)
			case base.InternalKeyKindRangeKeyUnset:
				err = b.RangeKeyUnset(s.Start, s.End, s.Keys[0].Suffix, nil)
			case base.InternalKeyKindRangeKeyDelete:
				err = b.RangeKeyDelete(s.Start, s.End, nil)
			default:
				err = errors.Newf("unexpected key kind %q", bufs.keys[i].Kind())
			}
			if err != nil {
				return err
			}
		default:
			err = errors.Newf("unexpected key kind %q", bufs.keys[i].Kind())
		}
		if err != nil {
			return err
		}
	}

	// Done with the flushBuffers. Reset.
	bufs.keys = bufs.keys[:0]
	return nil
}

type flushBuffers struct {
	keys  flushedKeysByTrailer
	alloc bytealloc.A
}

type flushedKeysByTrailer []flushedKey

func (s flushedKeysByTrailer) Len() int           { return len(s) }
func (s flushedKeysByTrailer) Less(i, j int) bool { return s[i].Trailer < s[j].Trailer }
func (s flushedKeysByTrailer) Swap(i, j int)      { s[i], s[j] = s[j], s[i] }

type flushedKey struct {
	base.InternalKey
	value []byte
}

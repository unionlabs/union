// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.22

package trace

import (
	"bytes"
	"encoding/binary"
	"errors"
	"fmt"
	"io"
	"math/bits"
	"runtime/trace"
	"slices"
	"sync"
	"time"
	_ "unsafe" // for go:linkname

	"golang.org/x/exp/trace/internal/event/go122"
)

// FlightRecorder represents a flight recording configuration.
//
// Flight recording holds execution trace data in a circular buffer representing
// the most recent execution data.
//
// Only one flight recording may be active at any given time.
type FlightRecorder struct {
	err error

	// State specific to the recorder.
	header [16]byte
	active rawGeneration
	ringMu sync.Mutex
	ring   []rawGeneration

	// Externally-set options.
	targetSize   int
	targetPeriod time.Duration

	enabled bool       // whether the flight recorder is enabled.
	writing sync.Mutex // protects concurrent calls to WriteTo

	// The values of targetSize and targetPeriod we've committed to since the last Start.
	wantSize int
	wantDur  time.Duration
}

// NewFlightRecorder creates a new flight recording configuration.
func NewFlightRecorder() *FlightRecorder {
	return &FlightRecorder{
		// These are just some optimistic, reasonable defaults.
		//
		// In reality we're also bound by whatever the runtime defaults are, because
		// we currently have no way to change them.
		//
		// TODO(mknyszek): Consider adding a function that allows mutating one or
		// both of these values' equivalents in the runtime.
		targetSize:   10 << 20, // 10 MiB.
		targetPeriod: 10 * time.Second,
	}
}

// SetPeriod sets the approximate time duration that the flight recorder's circular buffer
// represents.
//
// Note that SetPeriod does not make any guarantees on the amount of time the trace
// produced by WriteTo will represent.
// This is just a hint to the runtime to enable some control the resulting trace.
//
// The initial period is implementation defined, but can be assumed to be on the order
// of seconds.
//
// Adjustments to this value will not apply to an active flight recorder, and will not apply
// if tracing is already enabled via trace.Start. All tracing must be stopped and started
// again to change this value.
func (r *FlightRecorder) SetPeriod(d time.Duration) {
	r.targetPeriod = d
}

// SetSize sets the approximate size of the flight recorder's circular buffer.
//
// This generally takes precedence over the duration passed to SetPeriod.
// However, it does not make any guarantees on the size of the data WriteTo will write.
// This is just a hint to the runtime to enable some control over the memory overheads
// of tracing.
//
// The initial size is implementation defined.
//
// Adjustments to this value will not apply to an active flight recorder, and will not apply
// if tracing is already enabled via trace.Start. All tracing must be stopped and started
// again to change this value.
func (r *FlightRecorder) SetSize(bytes int) {
	r.targetSize = bytes
}

// A recorder receives bytes from the runtime tracer, processes it.
type recorder struct {
	r *FlightRecorder

	headerReceived bool
}

func (w *recorder) Write(p []byte) (n int, err error) {
	r := w.r

	defer func() {
		if err != nil {
			// Propagate errors to the flightrecorder.
			if r.err == nil {
				r.err = err
			}
			trace.Stop() // Stop the tracer, preventing further writes.
		}
	}()

	rd := bytes.NewReader(p)

	if !w.headerReceived {
		if len(p) < len(r.header) {
			return 0, fmt.Errorf("expected at least %d bytes in the first write", len(r.header))
		}
		rd.Read(r.header[:])
		w.headerReceived = true
	}

	b, gen, err := readBatch(rd) // Every write from the runtime is guaranteed to be a complete batch.
	if err == io.EOF {
		if rd.Len() > 0 {
			return len(p) - rd.Len(), errors.New("short read")
		}
		return len(p), nil
	}
	if err != nil {
		return len(p) - rd.Len(), err
	}

	// Check if we're entering a new generation.
	if r.active.gen != 0 && r.active.gen+1 == gen {
		r.ringMu.Lock()

		// Validate r.active.freq before we use it. It's required for a generation
		// to not be considered broken, and without it, we can't correctly handle
		// SetPeriod.
		if r.active.freq == 0 {
			return len(p) - rd.Len(), fmt.Errorf("broken trace: failed to find frequency event in generation %d", r.active.gen)
		}

		// Get the current trace clock time.
		now := traceTimeNow(r.active.freq)

		// Add the current generation to the ring. Make sure we always have at least one
		// complete generation by putting the active generation onto the new list, regardless
		// of whatever our settings are.
		//
		// N.B. Let's completely replace the ring here, so that WriteTo can just make a copy
		// and not worry about aliasing. This creates allocations, but at a very low rate.
		newRing := []rawGeneration{r.active}
		size := r.active.size
		for i := len(r.ring) - 1; i >= 0; i-- {
			// Stop adding older generations if the new ring already exceeds the thresholds.
			// This ensures we keep generations that cross a threshold, but not any that lie
			// entirely outside it.
			if size > r.wantSize || now.Sub(newRing[len(newRing)-1].minTraceTime()) > r.wantDur {
				break
			}
			size += r.ring[i].size
			newRing = append(newRing, r.ring[i])
		}
		slices.Reverse(newRing)
		r.ring = newRing
		r.ringMu.Unlock()

		// Start a new active generation.
		r.active = rawGeneration{}
	}

	// Obtain the frequency if this is a frequency batch.
	if b.isFreqBatch() {
		freq, err := parseFreq(b)
		if err != nil {
			return len(p) - rd.Len(), err
		}
		r.active.freq = freq
	}

	// Append the batch to the current generation.
	if r.active.gen == 0 {
		r.active.gen = gen
	}
	if r.active.minTime == 0 || r.active.minTime > b.time {
		r.active.minTime = b.time
	}
	r.active.size += 1
	r.active.size += uvarintSize(gen)
	r.active.size += uvarintSize(uint64(b.m))
	r.active.size += uvarintSize(uint64(b.time))
	r.active.size += uvarintSize(uint64(len(b.data)))
	r.active.size += len(b.data)
	r.active.batches = append(r.active.batches, b)

	return len(p) - rd.Len(), nil
}

// Start begins flight recording. Only one flight recorder or one call to [runtime/trace.Start]
// may be active at any given time. Returns an error if starting the flight recorder would
// violate this rule.
func (r *FlightRecorder) Start() error {
	if r.enabled {
		return fmt.Errorf("cannot enable a enabled flight recorder")
	}

	r.wantSize = r.targetSize
	r.wantDur = r.targetPeriod
	r.err = nil

	// Start tracing, data is sent to a recorder which forwards it to our own
	// storage.
	if err := trace.Start(&recorder{r: r}); err != nil {
		return err
	}

	r.enabled = true
	return nil
}

// Stop ends flight recording. It waits until any concurrent [FlightRecorder.WriteTo] calls exit.
// Returns an error if the flight recorder is inactive.
func (r *FlightRecorder) Stop() error {
	if !r.enabled {
		return fmt.Errorf("cannot disable a disabled flight recorder")
	}
	r.enabled = false
	trace.Stop()

	// Reset all state. No need to lock because the reader has already exited.
	r.active = rawGeneration{}
	r.ring = nil
	return r.err
}

// Enabled returns true if the flight recorder is active. Specifically, it will return true if
// Start did not return an error, and Stop has not yet been called.
// It is safe to call from multiple goroutines simultaneously.
func (r *FlightRecorder) Enabled() bool {
	return r.enabled
}

// ErrSnapshotActive indicates that a call to WriteTo was made while one was already in progress.
// If the caller of WriteTo sees this error, they should use the result from the other call to WriteTo.
var ErrSnapshotActive = fmt.Errorf("call to WriteTo for trace.FlightRecorder already in progress")

// WriteTo takes a snapshots of the circular buffer's contents and writes the execution data to w.
// Returns the number of bytes written and an error.
// An error is returned upon failure to write to w or if the flight recorder is inactive.
// Only one goroutine may execute WriteTo at a time, but it is safe to call from multiple goroutines.
// If a goroutine calls WriteTo while another goroutine is currently executing it, WriteTo will return
// ErrSnapshotActive to that goroutine.
func (r *FlightRecorder) WriteTo(w io.Writer) (total int, err error) {
	if !r.enabled {
		return 0, fmt.Errorf("cannot snapshot a disabled flight recorder")
	}
	if !r.writing.TryLock() {
		return 0, ErrSnapshotActive
	}
	defer r.writing.Unlock()

	// Force a global buffer flush twice.
	//
	// This is pretty unfortunate, but because the signal that a generation is done is that a new
	// generation appears in the trace *or* the trace stream ends, the recorder goroutine will
	// have no idea when to add a generation to the ring if we just flush once. If we flush twice,
	// at least the first one will end up on the ring, which is the one we wanted anyway.
	//
	// In a runtime-internal implementation this is a non-issue. The runtime is fully aware
	// of what generations are complete, so only one flush is necessary.
	runtime_traceAdvance(false)
	runtime_traceAdvance(false)

	// Now that everything has been flushed and written, grab whatever we have.
	//
	// N.B. traceAdvance blocks until the tracer goroutine has actually written everything
	// out, which means the generation we just flushed must have been already been observed
	// by the recorder goroutine. Because we flushed twice, the first flush is guaranteed to
	// have been both completed *and* processed by the recorder goroutine.
	r.ringMu.Lock()
	gens := r.ring
	r.ringMu.Unlock()

	// Write the header.
	total, err = w.Write(r.header[:])
	if err != nil {
		return total, err
	}

	// Helper for writing varints.
	var varintBuf [binary.MaxVarintLen64]byte
	writeUvarint := func(u uint64) error {
		v := binary.PutUvarint(varintBuf[:], u)
		n, err := w.Write(varintBuf[:v])
		total += n
		return err
	}

	// Write all the data.
	for _, gen := range gens {
		for _, batch := range gen.batches {
			// Rewrite the batch header event with four arguments: gen, M ID, timestamp, and data length.
			n, err := w.Write([]byte{byte(go122.EvEventBatch)})
			total += n
			if err != nil {
				return total, err
			}
			if err := writeUvarint(gen.gen); err != nil {
				return total, err
			}
			if err := writeUvarint(uint64(batch.m)); err != nil {
				return total, err
			}
			if err := writeUvarint(uint64(batch.time)); err != nil {
				return total, err
			}
			if err := writeUvarint(uint64(len(batch.data))); err != nil {
				return total, err
			}

			// Write batch data.
			n, err = w.Write(batch.data)
			total += n
			if err != nil {
				return total, err
			}
		}
	}
	return total, nil
}

type rawGeneration struct {
	gen     uint64
	size    int
	minTime timestamp
	freq    frequency
	batches []batch
}

func (r *rawGeneration) minTraceTime() Time {
	return r.freq.mul(r.minTime)
}

func traceTimeNow(freq frequency) Time {
	// TODO(mknyszek): It's unfortunate that we have to rely on runtime-internal details
	// like this. This would be better off in the runtime.
	return freq.mul(timestamp(runtime_traceClockNow()))
}

func uvarintSize(x uint64) int {
	return 1 + bits.Len64(x)/7
}

//go:linkname runtime_traceAdvance runtime.traceAdvance
func runtime_traceAdvance(stopTrace bool)

//go:linkname runtime_traceClockNow runtime.traceClockNow
func runtime_traceClockNow() int64

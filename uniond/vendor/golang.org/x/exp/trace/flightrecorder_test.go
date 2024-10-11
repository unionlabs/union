// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.22

package trace_test

import (
	"bytes"
	"context"
	"io"
	"runtime/trace"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"golang.org/x/exp/trace/internal/testtrace"

	. "golang.org/x/exp/trace"
)

func TestFlightRecorderDoubleStart(t *testing.T) {
	fr := NewFlightRecorder()

	if err := fr.Start(); err != nil {
		t.Fatalf("unexpected error on Start: %v", err)
	}
	if err := fr.Start(); err == nil {
		t.Fatalf("expected error from double Start: %v", err)
	}
	if err := fr.Stop(); err != nil {
		t.Fatalf("unexpected error on Stop: %v", err)
	}
}

func TestFlightRecorderDoubleStop(t *testing.T) {
	fr := NewFlightRecorder()

	if err := fr.Start(); err != nil {
		t.Fatalf("unexpected error on Start: %v", err)
	}
	if err := fr.Stop(); err != nil {
		t.Fatalf("unexpected error on Stop: %v", err)
	}
	if err := fr.Stop(); err == nil {
		t.Fatalf("expected error from double Stop: %v", err)
	}
}

func TestFlightRecorderEnabled(t *testing.T) {
	fr := NewFlightRecorder()

	if fr.Enabled() {
		t.Fatal("flight recorder is enabled, but never started")
	}
	if err := fr.Start(); err != nil {
		t.Fatalf("unexpected error on Start: %v", err)
	}
	if !fr.Enabled() {
		t.Fatal("flight recorder is not enabled, but started")
	}
	if err := fr.Stop(); err != nil {
		t.Fatalf("unexpected error on Stop: %v", err)
	}
	if fr.Enabled() {
		t.Fatal("flight recorder is enabled, but stopped")
	}
}

func TestFlightRecorderWriteToDisabled(t *testing.T) {
	var buf bytes.Buffer

	fr := NewFlightRecorder()
	if n, err := fr.WriteTo(&buf); err == nil {
		t.Fatalf("successfully wrote %d bytes from disabled flight recorder", n)
	}
	if err := fr.Start(); err != nil {
		t.Fatalf("unexpected error on Start: %v", err)
	}
	if err := fr.Stop(); err != nil {
		t.Fatalf("unexpected error on Stop: %v", err)
	}
	if n, err := fr.WriteTo(&buf); err == nil {
		t.Fatalf("successfully wrote %d bytes from disabled flight recorder", n)
	}
}

func TestFlightRecorderConcurrentWriteTo(t *testing.T) {
	fr := NewFlightRecorder()
	if err := fr.Start(); err != nil {
		t.Fatalf("unexpected error on Start: %v", err)
	}

	// Start two goroutines to write snapshots.
	//
	// Most of the time one will fail and one will succeed, but we don't require this.
	// Due to a variety of factors, it's definitely possible for them both to succeed.
	// However, at least one should succeed.
	var bufs [2]bytes.Buffer
	var wg sync.WaitGroup
	var successes atomic.Uint32
	for i := range bufs {
		wg.Add(1)
		go func() {
			defer wg.Done()

			n, err := fr.WriteTo(&bufs[i])
			if err == ErrSnapshotActive {
				if n != 0 {
					t.Errorf("(goroutine %d) WriteTo bytes written is non-zero for early bail out: %d", i, n)
				}
				return
			}
			if err != nil {
				t.Errorf("(goroutine %d) failed to write snapshot for unexpected reason: %v", i, err)
			}
			successes.Add(1)

			if n == 0 {
				t.Errorf("(goroutine %d) wrote invalid trace of zero bytes in size", i)
			}
			if n != bufs[i].Len() {
				t.Errorf("(goroutine %d) trace length doesn't match WriteTo result: got %d, want %d", i, n, bufs[i].Len())
			}
		}()
	}
	wg.Wait()

	// Stop tracing.
	if err := fr.Stop(); err != nil {
		t.Fatalf("unexpected error on Stop: %v", err)
	}

	// Make sure at least one succeeded to write.
	if successes.Load() == 0 {
		t.Fatal("expected at least one success to write a snapshot, got zero")
	}

	// Validate the traces that came out.
	for i := range bufs {
		buf := &bufs[i]
		if buf.Len() == 0 {
			continue
		}
		testReader(t, buf, testtrace.ExpectSuccess())
	}
}

func TestFlightRecorder(t *testing.T) {
	testFlightRecorder(t, NewFlightRecorder(), func(snapshot func()) {
		snapshot()
	})
}

func TestFlightRecorderStartStop(t *testing.T) {
	fr := NewFlightRecorder()
	for i := 0; i < 5; i++ {
		testFlightRecorder(t, fr, func(snapshot func()) {
			snapshot()
		})
	}
}

func TestFlightRecorderLog(t *testing.T) {
	tr := testFlightRecorder(t, NewFlightRecorder(), func(snapshot func()) {
		trace.Log(context.Background(), "message", "hello")
		snapshot()
	})

	// Prepare to read the trace snapshot.
	r, err := NewReader(bytes.NewReader(tr))
	if err != nil {
		t.Fatalf("unexpected error creating trace reader: %v", err)
	}

	// Find the log message in the trace.
	found := false
	for {
		ev, err := r.ReadEvent()
		if err == io.EOF {
			break
		}
		if err != nil {
			t.Fatalf("unexpected error reading trace: %v", err)
		}
		if !found && ev.Kind() == EventLog {
			log := ev.Log()
			found = log.Category == "message" && log.Message == "hello"
		}
	}
	if !found {
		t.Errorf("failed to find expected log message (%q, %q) in snapshot", "message", "hello")
	}
}

func TestFlightRecorderOneGeneration(t *testing.T) {
	test := func(t *testing.T, fr *FlightRecorder) {
		tr := testFlightRecorder(t, fr, func(snapshot func()) {
			// Sleep to let a few generations pass.
			time.Sleep(3 * time.Second)
			snapshot()
		})

		// Prepare to read the trace snapshot.
		r, err := NewReader(bytes.NewReader(tr))
		if err != nil {
			t.Fatalf("unexpected error creating trace reader: %v", err)
		}

		// Make sure there's only exactly one Sync event.
		sync := 0
		for {
			ev, err := r.ReadEvent()
			if err == io.EOF {
				break
			}
			if err != nil {
				t.Fatalf("unexpected error reading trace: %v", err)
			}
			if ev.Kind() == EventSync {
				sync++
			}
		}
		if sync != 1 {
			t.Errorf("expected one sync event, found %d", sync)
		}
	}
	t.Run("SetPeriod", func(t *testing.T) {
		// Set the period to 0 so that we're always throwing away old generations.
		// This should always result in exactly one generation.
		// Note: this is always going to result in taking the second generation
		// flushed, which is the much less useful one. That's OK, because in practice
		// SetPeriod shouldn't ever be called with a value this low.
		fr := NewFlightRecorder()
		fr.SetPeriod(0)
		test(t, fr)
	})
	t.Run("SetSize", func(t *testing.T) {
		// Set the size to 0 so that we're always throwing away old generations.
		// This should always result in exactly one generation.
		// Note: this is always going to result in taking the second generation
		// flushed, which is the much less useful one. That's OK, because in practice
		// SetPeriod shouldn't ever be called with a value this low.
		fr := NewFlightRecorder()
		fr.SetSize(0)
		test(t, fr)
	})
}

type flightRecorderTestFunc func(snapshot func())

func testFlightRecorder(t *testing.T, fr *FlightRecorder, f flightRecorderTestFunc) []byte {
	if trace.IsEnabled() {
		t.Skip("cannot run flight recorder tests when tracing is enabled")
	}

	// Start the flight recorder.
	if err := fr.Start(); err != nil {
		t.Fatalf("unexpected error on Start: %v", err)
	}

	// Set up snapshot callback.
	var buf bytes.Buffer
	callback := func() {
		n, err := fr.WriteTo(&buf)
		if err != nil {
			t.Errorf("unexpected failure during flight recording: %v", err)
			return
		}
		if n < 16 {
			t.Errorf("expected a trace size of at least 16 bytes, got %d", n)
		}
		if n != buf.Len() {
			t.Errorf("WriteTo result doesn't match trace size: got %d, want %d", n, buf.Len())
		}
	}

	// Call the test function.
	f(callback)

	// Stop the flight recorder.
	if err := fr.Stop(); err != nil {
		t.Fatalf("unexpected error on Stop: %v", err)
	}

	// Get the trace bytes; we don't want to use the Buffer as a Reader directly
	// since we may want to consume this data more than once.
	traceBytes := buf.Bytes()

	// Parse the trace to make sure it's not broken.
	testReader(t, bytes.NewReader(traceBytes), testtrace.ExpectSuccess())
	return traceBytes
}

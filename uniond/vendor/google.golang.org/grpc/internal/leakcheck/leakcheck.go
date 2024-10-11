/*
 *
 * Copyright 2017 gRPC authors.
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

// Package leakcheck contains functions to check leaked goroutines and buffers.
//
// Call the following at the beginning of test:
//
//	defer leakcheck.NewLeakChecker(t).Check()
package leakcheck

import (
	"runtime"
	"runtime/debug"
	"slices"
	"sort"
	"strconv"
	"strings"
	"sync"
	"sync/atomic"
	"time"

	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/mem"
)

// failTestsOnLeakedBuffers is a special flag that will cause tests to fail if
// leaked buffers are detected, instead of simply logging them as an
// informational failure. This can be enabled with the "checkbuffers" compile
// flag, e.g.:
//
//	go test -tags=checkbuffers
var failTestsOnLeakedBuffers = false

func init() {
	defaultPool := mem.DefaultBufferPool()
	globalPool.Store(&defaultPool)
	(internal.SetDefaultBufferPoolForTesting.(func(mem.BufferPool)))(&globalPool)
}

var globalPool swappableBufferPool

type swappableBufferPool struct {
	atomic.Pointer[mem.BufferPool]
}

func (b *swappableBufferPool) Get(length int) *[]byte {
	return (*b.Load()).Get(length)
}

func (b *swappableBufferPool) Put(buf *[]byte) {
	(*b.Load()).Put(buf)
}

// SetTrackingBufferPool replaces the default buffer pool in the mem package to
// one that tracks where buffers are allocated. CheckTrackingBufferPool should
// then be invoked at the end of the test to validate that all buffers pulled
// from the pool were returned.
func SetTrackingBufferPool(logger Logger) {
	newPool := mem.BufferPool(&trackingBufferPool{
		pool:             *globalPool.Load(),
		logger:           logger,
		allocatedBuffers: make(map[*[]byte][]uintptr),
	})
	globalPool.Store(&newPool)
}

// CheckTrackingBufferPool undoes the effects of SetTrackingBufferPool, and fails
// unit tests if not all buffers were returned. It is invalid to invoke this
// method without previously having invoked SetTrackingBufferPool.
func CheckTrackingBufferPool() {
	p := (*globalPool.Load()).(*trackingBufferPool)
	p.lock.Lock()
	defer p.lock.Unlock()

	globalPool.Store(&p.pool)

	type uniqueTrace struct {
		stack []uintptr
		count int
	}

	var totalLeakedBuffers int
	var uniqueTraces []uniqueTrace
	for _, stack := range p.allocatedBuffers {
		idx, ok := slices.BinarySearchFunc(uniqueTraces, stack, func(trace uniqueTrace, stack []uintptr) int {
			return slices.Compare(trace.stack, stack)
		})
		if !ok {
			uniqueTraces = slices.Insert(uniqueTraces, idx, uniqueTrace{stack: stack})
		}
		uniqueTraces[idx].count++
		totalLeakedBuffers++
	}

	for _, ut := range uniqueTraces {
		frames := runtime.CallersFrames(ut.stack)
		var trace strings.Builder
		for {
			f, ok := frames.Next()
			if !ok {
				break
			}
			trace.WriteString(f.Function)
			trace.WriteString("\n\t")
			trace.WriteString(f.File)
			trace.WriteString(":")
			trace.WriteString(strconv.Itoa(f.Line))
			trace.WriteString("\n")
		}
		format := "%d allocated buffers never freed:\n%s"
		args := []any{ut.count, trace.String()}
		if failTestsOnLeakedBuffers {
			p.logger.Errorf(format, args...)
		} else {
			p.logger.Logf("WARNING "+format, args...)
		}
	}

	if totalLeakedBuffers > 0 {
		p.logger.Logf("%g%% of buffers never freed", float64(totalLeakedBuffers)/float64(p.bufferCount))
	}
}

type trackingBufferPool struct {
	pool   mem.BufferPool
	logger Logger

	lock             sync.Mutex
	bufferCount      int
	allocatedBuffers map[*[]byte][]uintptr
}

func (p *trackingBufferPool) Get(length int) *[]byte {
	p.lock.Lock()
	defer p.lock.Unlock()

	p.bufferCount++

	buf := p.pool.Get(length)

	var stackBuf [16]uintptr
	var stack []uintptr
	skip := 2
	for {
		n := runtime.Callers(skip, stackBuf[:])
		stack = append(stack, stackBuf[:n]...)
		if n < len(stackBuf) {
			break
		}
		skip += len(stackBuf)
	}
	p.allocatedBuffers[buf] = stack

	return buf
}

func (p *trackingBufferPool) Put(buf *[]byte) {
	p.lock.Lock()
	defer p.lock.Unlock()

	if _, ok := p.allocatedBuffers[buf]; !ok {
		p.logger.Errorf("Unknown buffer freed:\n%s", string(debug.Stack()))
	} else {
		delete(p.allocatedBuffers, buf)
	}
	p.pool.Put(buf)
}

var goroutinesToIgnore = []string{
	"testing.Main(",
	"testing.tRunner(",
	"testing.(*M).",
	"runtime.goexit",
	"created by runtime.gc",
	"created by runtime/trace.Start",
	"interestingGoroutines",
	"runtime.MHeap_Scavenger",
	"signal.signal_recv",
	"sigterm.handler",
	"runtime_mcall",
	"(*loggingT).flushDaemon",
	"goroutine in C code",
	// Ignore the http read/write goroutines. gce metadata.OnGCE() was leaking
	// these, root cause unknown.
	//
	// https://github.com/grpc/grpc-go/issues/5171
	// https://github.com/grpc/grpc-go/issues/5173
	"created by net/http.(*Transport).dialConn",
}

// RegisterIgnoreGoroutine appends s into the ignore goroutine list. The
// goroutines whose stack trace contains s will not be identified as leaked
// goroutines. Not thread-safe, only call this function in init().
func RegisterIgnoreGoroutine(s string) {
	goroutinesToIgnore = append(goroutinesToIgnore, s)
}

func ignore(g string) bool {
	sl := strings.SplitN(g, "\n", 2)
	if len(sl) != 2 {
		return true
	}
	stack := strings.TrimSpace(sl[1])
	if strings.HasPrefix(stack, "testing.RunTests") {
		return true
	}

	if stack == "" {
		return true
	}

	for _, s := range goroutinesToIgnore {
		if strings.Contains(stack, s) {
			return true
		}
	}

	return false
}

// interestingGoroutines returns all goroutines we care about for the purpose of
// leak checking. It excludes testing or runtime ones.
func interestingGoroutines() (gs []string) {
	buf := make([]byte, 2<<20)
	buf = buf[:runtime.Stack(buf, true)]
	for _, g := range strings.Split(string(buf), "\n\n") {
		if !ignore(g) {
			gs = append(gs, g)
		}
	}
	sort.Strings(gs)
	return
}

// Logger is the interface that wraps the Logf and Errorf method. It's a subset
// of testing.TB to make it easy to use this package.
type Logger interface {
	Logf(format string, args ...any)
	Errorf(format string, args ...any)
}

// CheckGoroutines looks at the currently-running goroutines and checks if there
// are any interesting (created by gRPC) goroutines leaked. It waits up to 10
// seconds in the error cases.
func CheckGoroutines(logger Logger, timeout time.Duration) {
	// Loop, waiting for goroutines to shut down.
	// Wait up to timeout, but finish as quickly as possible.
	deadline := time.Now().Add(timeout)
	var leaked []string
	for time.Now().Before(deadline) {
		if leaked = interestingGoroutines(); len(leaked) == 0 {
			return
		}
		time.Sleep(50 * time.Millisecond)
	}
	for _, g := range leaked {
		logger.Errorf("Leaked goroutine: %v", g)
	}
}

// LeakChecker captures an Logger and is returned by NewLeakChecker as a
// convenient method to set up leak check tests in a unit test.
type LeakChecker struct {
	logger Logger
}

// Check executes the leak check tests, failing the unit test if any buffer or
// goroutine leaks are detected.
func (lc *LeakChecker) Check() {
	CheckTrackingBufferPool()
	CheckGoroutines(lc.logger, 10*time.Second)
}

// NewLeakChecker offers a convenient way to set up the leak checks for a
// specific unit test. It can be used as follows, at the beginning of tests:
//
//	defer leakcheck.NewLeakChecker(t).Check()
//
// It initially invokes SetTrackingBufferPool to set up buffer tracking, then the
// deferred LeakChecker.Check call will invoke CheckTrackingBufferPool and
// CheckGoroutines with a default timeout of 10 seconds.
func NewLeakChecker(logger Logger) *LeakChecker {
	SetTrackingBufferPool(logger)
	return &LeakChecker{logger: logger}
}

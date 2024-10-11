package sentry

import (
	"encoding/json"
	"fmt"
	"io"
	"math/rand"
	"os"
	"runtime"
	"strings"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/getsentry/sentry-go/internal/testutils"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// Test ticker that ticks on demand instead of relying on go runtime timing.
type profilerTestTicker struct {
	log             func(format string, args ...any)
	sleepBeforeTick time.Duration
	tick            chan time.Time
	ticked          chan struct{}
}

func (t *profilerTestTicker) TickSource() <-chan time.Time {
	t.log("Ticker: tick source requested.\n")
	return t.tick
}

func (t *profilerTestTicker) Ticked() {
	t.log("Ticker: tick acknowledged (on the profiler goroutine).\n")
	t.ticked <- struct{}{}
}

func (t *profilerTestTicker) Stop() {
	t.log("Ticker: stopping.\n")
	close(t.ticked)
}

// Sleeps before a tick to emulate a reasonable frequency of ticks, or they may all come at the same relative time.
// Then, sends a tick and waits for the profiler to process it.
func (t *profilerTestTicker) Tick() bool {
	time.Sleep(t.sleepBeforeTick)
	t.log("Ticker: ticking\n")
	t.tick <- time.Now()
	select {
	case _, ok := <-t.ticked:
		if ok {
			t.log("Ticker: tick acknowledged (received on the test goroutine).\n") // logged on the test goroutine
			return true
		}
		t.log("Ticker: tick not acknowledged (ticker stopped).\n")
		return false
	case <-time.After(1 * time.Second):
		t.log("Ticker: timed out waiting for Tick ACK.")
		return false
	}
}

func setupProfilerTestTicker(logWriter io.Writer) *profilerTestTicker {
	ticker := &profilerTestTicker{
		log: func(format string, args ...any) {
			fmt.Fprintf(logWriter, format, args...)
		},
		sleepBeforeTick: time.Millisecond,
		tick:            make(chan time.Time, 1),
		ticked:          make(chan struct{}),
	}
	profilerTickerFactory = func(d time.Duration) profilerTicker { return ticker }
	return ticker
}

func restoreProfilerTicker() {
	profilerTickerFactory = profilerTickerFactoryDefault
}

func TestProfilerCollection(t *testing.T) {
	t.Run("RealTicker", func(t *testing.T) {
		var require = require.New(t)
		var goID = getCurrentGoID()

		start := time.Now()
		profiler := startProfiling(start)
		defer profiler.Stop(true)
		if testutils.IsCI() {
			doWorkFor(5 * time.Second)
		} else {
			doWorkFor(35 * time.Millisecond)
		}
		end := time.Now()
		result := profiler.GetSlice(start, end)
		require.NotNil(result)
		require.Greater(result.callerGoID, uint64(0))
		require.Equal(goID, result.callerGoID)
		validateProfile(t, result.trace, end.Sub(start))
	})

	t.Run("CustomTicker", func(t *testing.T) {
		var require = require.New(t)
		var goID = getCurrentGoID()

		ticker := setupProfilerTestTicker(io.Discard)
		defer restoreProfilerTicker()

		start := time.Now()
		profiler := startProfiling(start)
		defer profiler.Stop(true)
		require.True(ticker.Tick())
		end := time.Now()
		result := profiler.GetSlice(start, end)
		require.NotNil(result)
		require.Greater(result.callerGoID, uint64(0))
		require.Equal(goID, result.callerGoID)
		validateProfile(t, result.trace, end.Sub(start))

		// Another slice that has start time different than the profiler start time.
		start = end
		require.True(ticker.Tick())
		require.True(ticker.Tick())
		end = time.Now()
		result = profiler.GetSlice(start, end)
		validateProfile(t, result.trace, end.Sub(start))
	})
}

func TestProfilerRingBufferOverflow(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping in short mode.")
	}

	var require = require.New(t)
	ticker := setupProfilerTestTicker(io.Discard)
	defer restoreProfilerTicker()

	profiler := startProfiling(time.Now())
	defer profiler.Stop(true)

	// Skip a few ticks to emulate the profiler already running before a transaction starts
	for i := 0; i < 100; i++ {
		require.True(ticker.Tick())
	}
	start := time.Now()

	// Emulate a transaction running for longer than the limit (30 seconds).
	// The buffer should be 3030 items long, i.e. 30 seconds * 101 samples per second.
	recorder := profiler.(*profileRecorder)
	require.Equal(3030, recorder.samplesBucketsHead.Len())
	for i := 0; i < recorder.samplesBucketsHead.Len(); i++ {
		require.True(ticker.Tick())
	}

	// Add a few more ticks after the transaction ends but prior collecting it.
	// This emulates how the SDK normally behaves.
	const ticksAfterEnd = 5
	end := time.Now()
	for i := 0; i < ticksAfterEnd; i++ {
		require.True(ticker.Tick())
	}

	result := profiler.GetSlice(start, end)
	require.NotNil(result)
	validateProfile(t, result.trace, end.Sub(start))

	// Calculate the number of buckets (profiler internal representation).
	// It should be the same as the length of the ring buffer minus ticksAfterEnd.
	buckets := make(map[uint64]bool)
	for _, sample := range result.trace.Samples {
		buckets[sample.ElapsedSinceStartNS] = true
	}
	require.Equal(recorder.samplesBucketsHead.Len()-ticksAfterEnd, len(buckets))
}

// Check the order of frames for a known stack trace (i.e. this test case).
func TestProfilerStackTrace(t *testing.T) {
	var require = require.New(t)

	ticker := setupProfilerTestTicker(io.Discard)
	defer restoreProfilerTicker()

	start := time.Now()
	profiler := startProfiling(start)
	defer profiler.Stop(true)
	require.True(ticker.Tick())
	result := profiler.GetSlice(start, time.Now())
	require.NotNil(result)

	var actual = ""
	for _, sample := range result.trace.Samples {
		if sample.ThreadID == result.callerGoID {
			t.Logf("Found a sample for the calling goroutine ID: %d", result.callerGoID)
			var stack = result.trace.Stacks[sample.StackID]
			for _, frameIndex := range stack {
				var frame = result.trace.Frames[frameIndex]
				actual += fmt.Sprintf("%s %s\n", frame.Module, frame.Function)
			}
			break
		}
	}
	require.NotZero(len(actual))
	actual = actual[:len(actual)-1] // remove trailing newline
	t.Log(actual)

	// Note: we can't check the exact stack trace because the profiler runs its own goroutine
	// And this test goroutine may be interrupted at multiple points.
	require.True(strings.HasSuffix(actual, `
github.com/getsentry/sentry-go TestProfilerStackTrace
testing tRunner
testing (*T).Run`))
}

func TestProfilerCollectsOnStart(t *testing.T) {
	Logger.SetOutput(os.Stdout)
	defer Logger.SetOutput(io.Discard)
	var require = require.New(t)

	setupProfilerTestTicker(io.Discard)
	defer restoreProfilerTicker()

	start := time.Now()
	profiler := startProfiling(start)
	profiler.Stop(true)
	require.NotNil(profiler.(*profileRecorder).samplesBucketsHead.Value)
}

func TestProfilerPanicDuringStartup(t *testing.T) {
	Logger.SetOutput(os.Stdout)
	defer Logger.SetOutput(io.Discard)
	var require = require.New(t)

	_ = setupProfilerTestTicker(os.Stdout)
	defer restoreProfilerTicker()

	atomic.StoreInt64(&testProfilerPanic, -1)

	start := time.Now()
	profiler := startProfiling(start)
	require.Nil(profiler)
}

func TestProfilerPanicOnTick(t *testing.T) {
	var assert = assert.New(t)

	ticker := setupProfilerTestTicker(os.Stdout)
	defer restoreProfilerTicker()

	// Panic after the first sample is collected.
	atomic.StoreInt64(&testProfilerPanic, 3)

	start := time.Now()
	profiler := startProfiling(start)
	defer profiler.Stop(true)
	assert.True(ticker.Tick())
	assert.False(ticker.Tick())

	end := time.Now()
	result := profiler.GetSlice(start, end)

	assert.Zero(atomic.LoadInt64(&testProfilerPanic))
	assert.NotNil(result)
	validateProfile(t, result.trace, end.Sub(start))
}

func TestProfilerPanicOnTickDirect(t *testing.T) {
	var require = require.New(t)

	profiler := newProfiler(time.Now())
	profiler.testProfilerPanic = 2

	// first tick won't panic
	profiler.onTick()
	samplesBucket := profiler.samplesBucketsHead.Value
	require.NotNil(samplesBucket)

	// This is normally handled by the profiler goroutine and stops the profiler.
	require.Panics(profiler.onTick)
	require.Equal(samplesBucket, profiler.samplesBucketsHead.Value)

	profiler.testProfilerPanic = 0

	profiler.onTick()
	require.NotEqual(samplesBucket, profiler.samplesBucketsHead.Value)
	require.NotNil(profiler.samplesBucketsHead.Value)
}

func doWorkFor(duration time.Duration) {
	start := time.Now()
	for time.Since(start) < duration {
		_ = findPrimeNumber(1000)
		runtime.Gosched()
	}
}

//nolint:unparam
func findPrimeNumber(n int) int {
	count := 0
	a := 2
	for count < n {
		b := 2
		prime := true // to check if found a prime
		for b*b <= a {
			if a%b == 0 {
				prime = false
				break
			}
			b++
		}
		if prime {
			count++
		}
		a++
	}
	return a - 1
}

func validateProfile(t *testing.T, trace *profileTrace, duration time.Duration) {
	var require = require.New(t)
	var assert = assert.New(t)
	require.NotNil(trace)
	assert.NotEmpty(trace.Samples)
	assert.NotEmpty(trace.Stacks)
	assert.NotEmpty(trace.Frames)
	assert.NotEmpty(trace.ThreadMetadata)

	for _, sample := range trace.Samples {
		assert.GreaterOrEqual(sample.ElapsedSinceStartNS, uint64(0))
		assert.GreaterOrEqual(uint64(duration.Nanoseconds()), sample.ElapsedSinceStartNS)
		assert.GreaterOrEqual(sample.StackID, 0)
		assert.Less(sample.StackID, len(trace.Stacks))
		assert.Contains(trace.ThreadMetadata, sample.ThreadID)
	}

	for _, thread := range trace.ThreadMetadata {
		assert.NotEmpty(thread.Name)
	}

	for i, frame := range trace.Frames {
		if jsonData, err := json.Marshal(frame); err == nil {
			t.Logf("Frame %d: %v", i, string(jsonData))
		}

		assert.NotEmpty(frame.Function)
		assert.NotContains(frame.Function, " ") // Space in the function name is likely a parsing error
		assert.Greater(len(frame.AbsPath)+len(frame.Filename), 0)
		assert.Greater(frame.Lineno, 0)
	}
}

func TestProfilerSamplingRate(t *testing.T) {
	if testutils.IsCI() {
		t.Skip("Skipping on CI because the machines are too overloaded to provide consistent ticker resolution.")
	}
	if testing.Short() {
		t.Skip("Skipping in short mode.")
	}

	var require = require.New(t)

	start := time.Now()
	profiler := startProfiling(start)
	defer profiler.Stop(true)
	doWorkFor(500 * time.Millisecond)
	end := time.Now()
	result := profiler.GetSlice(start, end)

	require.NotEmpty(result.trace.Samples)
	var samplesByThread = map[uint64]uint64{}
	var outliersByThread = map[uint64]uint64{}
	var outliers = 0
	var lastLogTime = uint64(0)
	for _, sample := range result.trace.Samples {
		count := samplesByThread[sample.ThreadID]

		var lowerBound = count * uint64(profilerSamplingRate.Nanoseconds())
		var upperBound = (count + 1 + outliersByThread[sample.ThreadID]) * uint64(profilerSamplingRate.Nanoseconds())

		if lastLogTime != sample.ElapsedSinceStartNS {
			t.Logf("Sample %d (%d) should be between %d and %d", count, sample.ElapsedSinceStartNS, lowerBound, upperBound)
			lastLogTime = sample.ElapsedSinceStartNS
		}

		// We can check the lower bound explicitly, but the upper bound is problematic as some samples may get delayed.
		// Therefore, we collect the number of outliers and check if it's reasonably low.
		require.GreaterOrEqual(sample.ElapsedSinceStartNS, lowerBound)
		if sample.ElapsedSinceStartNS > upperBound {
			// We also increase the count by one to shift the followup samples too.
			outliersByThread[sample.ThreadID]++
			if int(outliersByThread[sample.ThreadID]) > outliers {
				outliers = int(outliersByThread[sample.ThreadID])
			}
		}

		samplesByThread[sample.ThreadID] = count + 1
	}

	require.Less(outliers, len(result.trace.Samples)/10)
}

func TestProfilerStackBufferGrowth(t *testing.T) {
	var require = require.New(t)
	profiler := newProfiler(time.Now())

	_ = profiler.collectRecords()

	profiler.stacksBuffer = make([]byte, 1)
	require.Equal(1, len(profiler.stacksBuffer))
	var bytesWithAutoAlloc = profiler.collectRecords()
	var lenAfterAutoAlloc = len(profiler.stacksBuffer)
	require.Greater(lenAfterAutoAlloc, 1)
	require.Greater(lenAfterAutoAlloc, len(bytesWithAutoAlloc))

	_ = profiler.collectRecords()
	require.Equal(lenAfterAutoAlloc, len(profiler.stacksBuffer))
}

func countSamples(profiler *profileRecorder) (value int) {
	profiler.samplesBucketsHead.Do(func(bucket interface{}) {
		if bucket != nil {
			value += len(bucket.(*profileSamplesBucket).goIDs)
		}
	})
	return value
}

// This tests profiler internals and replaces in-code asserts. While this shouldn't generally be done and instead
// we should test the profiler API only, this is trying to reduce a chance of a broken code that may externally work
// but has unbounded memory usage or similar performance issue.
func TestProfilerInternalMaps(t *testing.T) {
	if testutils.IsCI() && testutils.IsRaceTest() {
		t.Skip("This is too flaky on slow CI when run with other goroutines in parallel " +
			" (there are multiple instances of HTTPTransport.worker() when the whole test suite is run).")
	}
	var assert = assert.New(t)

	profiler := newProfiler(time.Now())

	// The size of the ring buffer is fixed throughout
	ringBufferSize := 3030

	// First, there is no data.
	assert.Zero(len(profiler.frames))
	assert.Zero(len(profiler.frameIndexes))
	assert.Zero(len(profiler.newFrames))
	assert.Zero(len(profiler.stacks))
	assert.Zero(len(profiler.stackIndexes))
	assert.Zero(len(profiler.newStacks))
	assert.Zero(countSamples(profiler))
	assert.Equal(ringBufferSize, profiler.samplesBucketsHead.Len())

	// After a tick, there is some data.
	profiler.onTick()
	assert.NotZero(len(profiler.frames))
	assert.NotZero(len(profiler.frameIndexes))
	assert.NotZero(len(profiler.newFrames))
	assert.NotZero(len(profiler.stacks))
	assert.NotZero(len(profiler.stackIndexes))
	assert.NotZero(len(profiler.newStacks))
	assert.NotZero(countSamples(profiler))
	assert.Equal(ringBufferSize, profiler.samplesBucketsHead.Len())

	framesLen := len(profiler.frames)
	frameIndexesLen := len(profiler.frameIndexes)
	stacksLen := len(profiler.stacks)
	stackIndexesLen := len(profiler.stackIndexes)
	samplesLen := countSamples(profiler)

	// On another tick, we will have the same data plus one frame and stack representing the profiler.onTick() call on the next line.
	profiler.onTick()
	assert.Equal(framesLen+1, len(profiler.frames))
	assert.Equal(frameIndexesLen+1, len(profiler.frameIndexes))
	assert.Equal(1, len(profiler.newFrames))
	assert.Equal(stacksLen+1, len(profiler.stacks))
	assert.Equal(stackIndexesLen+1, len(profiler.stackIndexes))
	assert.Equal(1, len(profiler.newStacks))
	assert.Equal(samplesLen*2, countSamples(profiler))
	assert.Equal(ringBufferSize, profiler.samplesBucketsHead.Len())

	// On another tick, we will have the same data plus one frame and stack representing the profiler.onTick() call on the next line.
	profiler.onTick()
	assert.Equal(framesLen+2, len(profiler.frames))
	assert.Equal(frameIndexesLen+2, len(profiler.frameIndexes))
	assert.Equal(1, len(profiler.newFrames))
	assert.Equal(stacksLen+2, len(profiler.stacks))
	assert.Equal(stackIndexesLen+2, len(profiler.stackIndexes))
	assert.Equal(1, len(profiler.newStacks))
	assert.Equal(samplesLen*3, countSamples(profiler))
	assert.Equal(ringBufferSize, profiler.samplesBucketsHead.Len())
}

func testTick(t *testing.T, count, i int, prevTick time.Time) time.Time {
	var sinceLastTick = time.Since(prevTick).Microseconds()
	t.Logf("tick %2d/%d after %d μs", i+1, count, sinceLastTick)
	return time.Now()
}

// This test measures the accuracy of time.NewTicker() on the current system.
func TestProfilerTimeTicker(t *testing.T) {
	if testutils.IsCI() {
		t.Skip("Skipping on CI because the machines are too overloaded to provide consistent ticker resolution.")
	}

	onProfilerStart() // This fixes Windows ticker resolution.

	t.Logf("We're expecting a tick once every %d μs", profilerSamplingRate.Microseconds())

	var startTime = time.Now()
	var ticker = time.NewTicker(profilerSamplingRate)
	defer ticker.Stop()

	// wait until 10 ticks have passed
	var count = 10
	var prevTick = time.Now()
	for i := 0; i < count; i++ {
		<-ticker.C
		prevTick = testTick(t, count, i, prevTick)
	}

	var elapsed = time.Since(startTime)
	require.LessOrEqual(t, elapsed.Microseconds(), profilerSamplingRate.Microseconds()*int64(count+3))
}

// This test measures the accuracy of time.Sleep() on the current system.
func TestProfilerTimeSleep(t *testing.T) {
	t.Skip("This test isn't necessary at the moment because we don't use time.Sleep() in the profiler.")

	onProfilerStart() // This fixes Windows ticker resolution.

	t.Logf("We're expecting a tick once every %d μs", profilerSamplingRate.Microseconds())

	var startTime = time.Now()

	// wait until 10 ticks have passed
	var count = 10
	var prevTick = time.Now()
	var next = time.Now()
	for i := 0; i < count; i++ {
		next = next.Add(profilerSamplingRate)
		time.Sleep(time.Until(next))
		prevTick = testTick(t, count, i, prevTick)
	}

	var elapsed = time.Since(startTime)
	require.LessOrEqual(t, elapsed.Microseconds(), profilerSamplingRate.Microseconds()*int64(count+3))
}

// Benchmark results (run without executing test which mess up results)
// $ go test -run=^$ -bench "BenchmarkProfiler*"
//
// goos: windows
// goarch: amd64
// pkg: github.com/getsentry/sentry-go
// cpu: 12th Gen Intel(R) Core(TM) i7-12700K
// BenchmarkProfilerStartStop/Wait-20                 12507             94991 ns/op          130506 B/op       3166 allocs/op
// BenchmarkProfilerStartStop/NoWait-20                9600            112354 ns/op          131125 B/op       3166 allocs/op
// BenchmarkProfilerOnTick-20                         65040             17771 ns/op            1008 B/op          8 allocs/op
// BenchmarkProfilerCollect-20                        64430             18223 ns/op               0 B/op          0 allocs/op
// BenchmarkProfilerProcess-20                       972006              1118 ns/op             960 B/op          8 allocs/op
// BenchmarkProfilerGetSlice-20                       37144             31289 ns/op           75813 B/op         19 allocs/op

func BenchmarkProfilerStartStop(b *testing.B) {
	var bench = func(name string, wait bool) {
		b.Run(name, func(b *testing.B) {
			b.ReportAllocs()
			for i := 0; i < b.N; i++ {
				startProfiling(time.Now()).Stop(wait)
			}
		})
	}

	bench("Wait", true)
	bench("NoWait", false)
}

func BenchmarkProfilerOnTick(b *testing.B) {
	profiler := newProfiler(time.Now())
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		profiler.onTick()
	}
}

func BenchmarkProfilerCollect(b *testing.B) {
	profiler := newProfiler(time.Now())
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = profiler.collectRecords()
	}
}

func BenchmarkProfilerProcess(b *testing.B) {
	profiler := newProfiler(time.Now())
	records := profiler.collectRecords()
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		profiler.processRecords(uint64(i), records)
	}
}

var profilerSliceBenchmarkData = struct {
	profiler *profileRecorder
	spans    []struct {
		start time.Time
		end   time.Time
	}
}{
	spans: make([]struct {
		start time.Time
		end   time.Time
	}, 100),
}

func setupProfilerSliceBenchmark(b *testing.B) {
	ticker := setupProfilerTestTicker(io.Discard)
	ticker.sleepBeforeTick = time.Microsecond

	start := time.Now()
	profiler := startProfiling(start).(*profileRecorder)

	// Fill in the profiler circular buffer first
	for i := 0; i < profiler.samplesBucketsHead.Len(); i++ {
		if !ticker.Tick() {
			b.Fatal("Tick() failed")
		}
	}

	if profiler.samplesBucketsHead.Next().Value == nil {
		b.Fatal("Profiler circular buffer is not filled completely")
	}

	end := time.Now()
	if end.Sub(start) <= 0 {
		b.Fatal("Unexpected end time")
	}

	// Prepare a set of spans we will be collecting.
	//nolint:gosec // We don't need a secure random number generator here.
	random := rand.New(rand.NewSource(42))
	collected := 0
	for i := 0; i < len(profilerSliceBenchmarkData.spans); i++ {
		spanStart := start.Add(time.Duration(random.Int63n(end.Sub(start).Nanoseconds())))
		spanEnd := spanStart.Add(time.Duration(random.Int63n(end.Sub(spanStart).Nanoseconds())))

		profilerSliceBenchmarkData.spans[i].start = spanStart
		profilerSliceBenchmarkData.spans[i].end = spanEnd

		slice := profiler.GetSlice(spanStart, spanEnd)
		if slice != nil {
			collected += len(slice.trace.Samples)
			b.Logf("Picked span: %d ms - %d ms with %d samples.\n", spanStart.Sub(start).Milliseconds(), spanEnd.Sub(start).Milliseconds(), len(slice.trace.Samples))
		}
	}

	if collected <= 0 {
		b.Fatal("Profiler failed to collect data")
	}

	b.Logf("Preparation took %d ms. Prepared %d samples in %d spans.\n", end.Sub(start).Milliseconds(), collected, len(profilerSliceBenchmarkData.spans))

	defer restoreProfilerTicker()
	defer profiler.Stop(true)

	profilerSliceBenchmarkData.profiler = profiler
}

func BenchmarkProfilerGetSlice(b *testing.B) {
	if profilerSliceBenchmarkData.profiler == nil {
		setupProfilerSliceBenchmark(b)
	}

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		span := profilerSliceBenchmarkData.spans[i%len(profilerSliceBenchmarkData.spans)]
		_ = profilerSliceBenchmarkData.profiler.GetSlice(span.start, span.end)
	}
}

func profilerBenchmark(t *testing.T, b *testing.B, withProfiling bool, arg int) {
	var p profiler
	if withProfiling {
		p = startProfiling(time.Now())
	}
	b.ResetTimer()

	var wg sync.WaitGroup
	wg.Add(b.N)
	for i := 0; i < b.N; i++ {
		go func() {
			start := time.Now()
			_ = findPrimeNumber(arg)
			end := time.Now()
			if p != nil {
				_ = p.GetSlice(start, end)
			}
			wg.Done()
		}()
	}
	wg.Wait()

	b.StopTimer()
	if p != nil {
		p.Stop(true)
		// Let's captured data so we can see what has been profiled if there's an error.
		// Previously, there have been tests that have started (and left running) global Sentry instance and goroutines.
		t.Log("Captured frames related to the profiler benchmark:")
		isRelatedToProfilerBenchmark := func(f *Frame) bool {
			return strings.Contains(f.AbsPath, "profiler") || strings.Contains(f.AbsPath, "benchmark.go") || strings.Contains(f.AbsPath, "testing.go")
		}
		for _, frame := range p.(*profileRecorder).frames {
			if isRelatedToProfilerBenchmark(frame) {
				t.Logf("%s %s\tat %s:%d", frame.Module, frame.Function, frame.AbsPath, frame.Lineno)
			}
		}
		t.Log(strings.Repeat("-", 80))
		t.Log("Unknown frames (these may be a cause of high overhead):")
		for _, frame := range p.(*profileRecorder).frames {
			if !isRelatedToProfilerBenchmark(frame) {
				t.Logf("%s %s\tat %s:%d", frame.Module, frame.Function, frame.AbsPath, frame.Lineno)
			}
		}
		t.Log(strings.Repeat("=", 80))
	}
}

func TestProfilerOverhead(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping overhead benchmark in short mode.")
	}
	if testutils.IsCI() {
		t.Skip("Skipping on CI because the machines are too overloaded to run the test properly - they show between 3 and 30 %% overhead....")
	}

	// First, find a large-enough argument so that findPrimeNumber(arg) takes more than 100ms.
	var arg = 10000
	for {
		start := time.Now()
		_ = findPrimeNumber(arg)
		end := time.Now()
		if end.Sub(start) > 100*time.Millisecond {
			t.Logf("Found arg = %d that takes %d ms to process.", arg, end.Sub(start).Milliseconds())
			break
		}
		arg += 10000
	}

	var assert = assert.New(t)
	var baseline = testing.Benchmark(func(b *testing.B) { profilerBenchmark(t, b, false, arg) })
	var profiling = testing.Benchmark(func(b *testing.B) { profilerBenchmark(t, b, true, arg) })

	t.Logf("Without profiling: %v\n", baseline.String())
	t.Logf("With profiling:    %v\n", profiling.String())

	var overhead = float64(profiling.NsPerOp())/float64(baseline.NsPerOp())*100 - 100
	var maxOverhead = 5.0
	t.Logf("Profiling overhead: %f percent\n", overhead)
	assert.Less(overhead, maxOverhead)
}

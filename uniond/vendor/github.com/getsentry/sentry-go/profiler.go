package sentry

import (
	"runtime"
	"strconv"
	"sync/atomic"
	"time"

	"github.com/getsentry/sentry-go/internal/traceparser"
)

// Start collecting profile data and returns a function that stops profiling, producing a Trace.
// The returned stop function May return nil or an incomplete trace in case of a panic.
func startProfiling(startTime time.Time) (stopFunc func() *profilerResult) {
	onProfilerStart()

	// buffered channels to handle the recover() case without blocking
	resultChannel := make(chan *profilerResult, 2)
	stopSignal := make(chan struct{}, 2)

	go profilerGoroutine(startTime, resultChannel, stopSignal)

	var goID = getCurrentGoID()

	return func() *profilerResult {
		stopSignal <- struct{}{}
		var result = <-resultChannel
		if result != nil {
			result.callerGoID = goID
		}
		return result
	}
}

// This allows us to test whether panic during profiling are handled correctly and don't block execution.
// If the number is lower than 0, profilerGoroutine() will panic immedately.
// If the number is higher than 0, profiler.onTick() will panic after the given number of samples collected.
var testProfilerPanic int64

func profilerGoroutine(startTime time.Time, result chan<- *profilerResult, stopSignal chan struct{}) {
	// We shouldn't panic but let's be super safe.
	defer func() {
		_ = recover()

		// Make sure we don't block the caller of stopFn() even if we panic.
		result <- nil

		atomic.StoreInt64(&testProfilerPanic, 0)
	}()

	// Stop after 30 seconds unless stopped manually.
	timeout := time.AfterFunc(30*time.Second, func() { stopSignal <- struct{}{} })
	defer timeout.Stop()

	var localTestProfilerPanic = atomic.LoadInt64(&testProfilerPanic)
	if localTestProfilerPanic < 0 {
		panic("This is an expected panic in profilerGoroutine() during tests")
	}

	profiler := newProfiler(startTime)
	profiler.testProfilerPanic = localTestProfilerPanic

	// Collect the first sample immediately.
	profiler.onTick()

	// Periodically collect stacks, starting after profilerSamplingRate has passed.
	collectTicker := profilerTickerFactory(profilerSamplingRate)
	defer collectTicker.Stop()
	var tickerChannel = collectTicker.Channel()

	defer func() {
		result <- &profilerResult{0, profiler.trace}
	}()

	for {
		select {
		case <-tickerChannel:
			profiler.onTick()
		case <-stopSignal:
			return
		}
	}
}

type profilerResult struct {
	callerGoID uint64
	trace      *profileTrace
}

func getCurrentGoID() uint64 {
	// We shouldn't panic but let's be super safe.
	defer func() {
		_ = recover()
	}()

	// Buffer to read the stack trace into. We should be good with a small buffer because we only need the first line.
	var stacksBuffer = make([]byte, 100)
	var n = runtime.Stack(stacksBuffer, false)
	if n > 0 {
		var traces = traceparser.Parse(stacksBuffer[0:n])
		if traces.Length() > 0 {
			var trace = traces.Item(0)
			return trace.GoID()
		}
	}
	return 0
}

func newProfiler(startTime time.Time) *profileRecorder {
	// Pre-allocate the profile trace for the currently active number of routines & 100 ms worth of samples.
	// Other coefficients are just guesses of what might be a good starting point to avoid allocs on short runs.
	numRoutines := runtime.NumGoroutine()
	trace := &profileTrace{
		Frames:         make([]*Frame, 0, 32),
		Samples:        make([]*profileSample, 0, numRoutines*10), // 100 ms @ 101 Hz
		Stacks:         make([]profileStack, 0, 8),
		ThreadMetadata: make(map[string]profileThreadMetadata, numRoutines),
	}

	return &profileRecorder{
		startTime:    startTime,
		trace:        trace,
		stackIndexes: make(map[string]int, cap(trace.Stacks)),
		frameIndexes: make(map[string]int, cap(trace.Frames)),
		// A buffer of 2 KiB per stack looks like a good starting point (empirically determined).
		stacksBuffer: make([]byte, numRoutines*2048),
	}
}

const profilerSamplingRate = time.Second / 101 // 101 Hz; not 100 Hz because of the lockstep sampling (https://stackoverflow.com/a/45471031/1181370)
const stackBufferMaxGrowth = 512 * 1024
const stackBufferLimit = 10 * 1024 * 1024

type profileRecorder struct {
	startTime         time.Time
	trace             *profileTrace
	testProfilerPanic int64

	// Buffer to read current stacks - will grow automatically up to stackBufferLimit.
	stacksBuffer []byte

	// Map from runtime.StackRecord.Stack0 to an index trace.Stacks.
	stackIndexes map[string]int

	// Map from runtime.Frame.PC to an index trace.Frames.
	frameIndexes map[string]int
}

func (p *profileRecorder) onTick() {
	elapsedNs := time.Since(p.startTime).Nanoseconds()

	if p.testProfilerPanic > 0 && int64(len(p.trace.Samples)) > p.testProfilerPanic {
		panic("This is an expected panic in Profiler.OnTick() during tests")
	}

	records := p.collectRecords()
	p.processRecords(uint64(elapsedNs), records)

	// Free up some memory if we don't need such a large buffer anymore.
	if len(p.stacksBuffer) > len(records)*3 {
		p.stacksBuffer = make([]byte, len(records)*3)
	}
}

func (p *profileRecorder) collectRecords() []byte {
	for {
		// Capture stacks for all existing goroutines.
		// Note: runtime.GoroutineProfile() would be better but we can't use it at the moment because
		//       it doesn't give us `gid` for each routine, see https://github.com/golang/go/issues/59663
		n := runtime.Stack(p.stacksBuffer, true)

		// If we couldn't read everything, increase the buffer and try again.
		if n >= len(p.stacksBuffer) && n < stackBufferLimit {
			var newSize = n * 2
			if newSize > n+stackBufferMaxGrowth {
				newSize = n + stackBufferMaxGrowth
			}
			if newSize > stackBufferLimit {
				newSize = stackBufferLimit
			}
			p.stacksBuffer = make([]byte, newSize)
		} else {
			return p.stacksBuffer[0:n]
		}
	}
}

func (p *profileRecorder) processRecords(elapsedNs uint64, stacksBuffer []byte) {
	var traces = traceparser.Parse(stacksBuffer)
	for i := traces.Length() - 1; i >= 0; i-- {
		var stack = traces.Item(i)
		threadIndex := p.addThread(stack.GoID())
		stackIndex := p.addStackTrace(stack)
		if stackIndex < 0 {
			return
		}

		p.trace.Samples = append(p.trace.Samples, &profileSample{
			ElapsedSinceStartNS: elapsedNs,
			StackID:             stackIndex,
			ThreadID:            threadIndex,
		})
	}
}

func (p *profileRecorder) addThread(id uint64) uint64 {
	index := strconv.FormatUint(id, 10)
	if _, exists := p.trace.ThreadMetadata[index]; !exists {
		p.trace.ThreadMetadata[index] = profileThreadMetadata{
			Name: "Goroutine " + index,
		}
	}
	return id
}

func (p *profileRecorder) addStackTrace(capturedStack traceparser.Trace) int {
	// NOTE: Don't convert to string yet, it's expensive and compiler can avoid it when
	//       indexing into a map (only needs a copy when adding a new key to the map).
	var key = capturedStack.UniqueIdentifier()

	stackIndex, exists := p.stackIndexes[string(key)]
	if !exists {
		iter := capturedStack.Frames()
		stack := make(profileStack, 0, iter.LengthUpperBound())
		for iter.HasNext() {
			var frame = iter.Next()

			if frameIndex := p.addFrame(frame); frameIndex >= 0 {
				stack = append(stack, frameIndex)
			}
		}
		stackIndex = len(p.trace.Stacks)
		p.trace.Stacks = append(p.trace.Stacks, stack)
		p.stackIndexes[string(key)] = stackIndex
	}

	return stackIndex
}

func (p *profileRecorder) addFrame(capturedFrame traceparser.Frame) int {
	// NOTE: Don't convert to string yet, it's expensive and compiler can avoid it when
	//       indexing into a map (only needs a copy when adding a new key to the map).
	var key = capturedFrame.UniqueIdentifier()

	frameIndex, exists := p.frameIndexes[string(key)]
	if !exists {
		module, function := splitQualifiedFunctionName(string(capturedFrame.Func()))
		file, line := capturedFrame.File()
		frame := newFrame(module, function, string(file), line)
		frameIndex = len(p.trace.Frames)
		p.trace.Frames = append(p.trace.Frames, &frame)
		p.frameIndexes[string(key)] = frameIndex
	}
	return frameIndex
}

// A Ticker holds a channel that delivers “ticks” of a clock at intervals.
type profilerTicker interface {
	Stop()
	Channel() <-chan time.Time
}

type timeTicker struct {
	*time.Ticker
}

func (t *timeTicker) Channel() <-chan time.Time {
	return t.C
}

func profilerTickerFactoryDefault(d time.Duration) profilerTicker {
	return &timeTicker{time.NewTicker(d)}
}

// We allow overriding the ticker for tests. CI is terribly flaky
// because the time.Ticker doesn't guarantee regular ticks - they may come (a lot) later than the given interval.
var profilerTickerFactory = profilerTickerFactoryDefault

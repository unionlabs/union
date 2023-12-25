package sentry

import (
	"container/ring"
	"strconv"

	"runtime"
	"sync"
	"sync/atomic"
	"time"

	"github.com/getsentry/sentry-go/internal/traceparser"
)

// Start a profiler that collects samples continuously, with a buffer of up to 30 seconds.
// Later, you can collect a slice from this buffer, producing a Trace.
func startProfiling(startTime time.Time) profiler {
	onProfilerStart()

	p := newProfiler(startTime)

	// Wait for the profiler to finish setting up before returning to the caller.
	started := make(chan struct{})
	go p.run(started)

	if _, ok := <-started; ok {
		return p
	}
	return nil
}

type profiler interface {
	// GetSlice returns a slice of the profiled data between the given times.
	GetSlice(startTime, endTime time.Time) *profilerResult
	Stop(wait bool)
}

type profilerResult struct {
	callerGoID uint64
	trace      *profileTrace
}

func getCurrentGoID() uint64 {
	// We shouldn't panic but let's be super safe.
	defer func() {
		if err := recover(); err != nil {
			Logger.Printf("Profiler panic in getCurrentGoID(): %v\n", err)
		}
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

const profilerSamplingRateHz = 101 // 101 Hz; not 100 Hz because of the lockstep sampling (https://stackoverflow.com/a/45471031/1181370)
const profilerSamplingRate = time.Second / profilerSamplingRateHz
const stackBufferMaxGrowth = 512 * 1024
const stackBufferLimit = 10 * 1024 * 1024
const profilerRuntimeLimit = 30 // seconds

type profileRecorder struct {
	startTime         time.Time
	stopSignal        chan struct{}
	stopped           int64
	mutex             sync.RWMutex
	testProfilerPanic int64

	// Map from runtime.StackRecord.Stack0 to an index in stacks.
	stackIndexes   map[string]int
	stacks         []profileStack
	newStacks      []profileStack // New stacks created in the current interation.
	stackKeyBuffer []byte

	// Map from runtime.Frame.PC to an index in frames.
	frameIndexes map[string]int
	frames       []*Frame
	newFrames    []*Frame // New frames created in the current interation.

	// We keep a ring buffer of 30 seconds worth of samples, so that we can later slice it.
	// Each bucket is a slice of samples all taken at the same time.
	samplesBucketsHead *ring.Ring

	// Buffer to read current stacks - will grow automatically up to stackBufferLimit.
	stacksBuffer []byte
}

func newProfiler(startTime time.Time) *profileRecorder {
	// Pre-allocate the profile trace for the currently active number of routines & 100 ms worth of samples.
	// Other coefficients are just guesses of what might be a good starting point to avoid allocs on short runs.
	return &profileRecorder{
		startTime:  startTime,
		stopSignal: make(chan struct{}, 1),

		stackIndexes: make(map[string]int, 32),
		stacks:       make([]profileStack, 0, 32),
		newStacks:    make([]profileStack, 0, 32),

		frameIndexes: make(map[string]int, 128),
		frames:       make([]*Frame, 0, 128),
		newFrames:    make([]*Frame, 0, 128),

		samplesBucketsHead: ring.New(profilerRuntimeLimit * profilerSamplingRateHz),

		// A buffer of 2 KiB per goroutine stack looks like a good starting point (empirically determined).
		stacksBuffer: make([]byte, runtime.NumGoroutine()*2048),
	}
}

// This allows us to test whether panic during profiling are handled correctly and don't block execution.
// If the number is lower than 0, profilerGoroutine() will panic immedately.
// If the number is higher than 0, profiler.onTick() will panic when the given samples-set index is being collected.
var testProfilerPanic int64
var profilerRunning int64

func (p *profileRecorder) run(started chan struct{}) {
	// Code backup for manual test debugging:
	// if !atomic.CompareAndSwapInt64(&profilerRunning, 0, 1) {
	// 	panic("Only one profiler can be running at a time")
	// }

	// We shouldn't panic but let's be super safe.
	defer func() {
		if err := recover(); err != nil {
			Logger.Printf("Profiler panic in run(): %v\n", err)
		}
		atomic.StoreInt64(&testProfilerPanic, 0)
		close(started)
		p.stopSignal <- struct{}{}
		atomic.StoreInt64(&p.stopped, 1)
		atomic.StoreInt64(&profilerRunning, 0)
	}()

	p.testProfilerPanic = atomic.LoadInt64(&testProfilerPanic)
	if p.testProfilerPanic < 0 {
		Logger.Printf("Profiler panicking during startup because testProfilerPanic == %v\n", p.testProfilerPanic)
		panic("This is an expected panic in profilerGoroutine() during tests")
	}

	// Collect the first sample immediately.
	p.onTick()

	// Periodically collect stacks, starting after profilerSamplingRate has passed.
	collectTicker := profilerTickerFactory(profilerSamplingRate)
	defer collectTicker.Stop()
	var tickerChannel = collectTicker.TickSource()

	started <- struct{}{}

	for {
		select {
		case <-tickerChannel:
			p.onTick()
			collectTicker.Ticked()
		case <-p.stopSignal:
			return
		}
	}
}

func (p *profileRecorder) Stop(wait bool) {
	if atomic.LoadInt64(&p.stopped) == 1 {
		return
	}
	p.stopSignal <- struct{}{}
	if wait {
		<-p.stopSignal
	}
}

func (p *profileRecorder) GetSlice(startTime, endTime time.Time) *profilerResult {
	// Unlikely edge cases - profiler wasn't running at all or the given times are invalid in relation to each other.
	if p.startTime.After(endTime) || startTime.After(endTime) {
		return nil
	}

	var relativeStartNS = uint64(0)
	if p.startTime.Before(startTime) {
		relativeStartNS = uint64(startTime.Sub(p.startTime).Nanoseconds())
	}
	var relativeEndNS = uint64(endTime.Sub(p.startTime).Nanoseconds())

	samplesCount, bucketsReversed, trace := p.getBuckets(relativeStartNS, relativeEndNS)
	if samplesCount == 0 {
		return nil
	}

	var result = &profilerResult{
		callerGoID: getCurrentGoID(),
		trace:      trace,
	}

	trace.Samples = make([]profileSample, samplesCount)
	trace.ThreadMetadata = make(map[uint64]*profileThreadMetadata, len(bucketsReversed[0].goIDs))
	var s = samplesCount - 1
	for _, bucket := range bucketsReversed {
		var elapsedSinceStartNS = bucket.relativeTimeNS - relativeStartNS
		for i, goID := range bucket.goIDs {
			trace.Samples[s].ElapsedSinceStartNS = elapsedSinceStartNS
			trace.Samples[s].ThreadID = goID
			trace.Samples[s].StackID = bucket.stackIDs[i]
			s--

			if _, goroutineExists := trace.ThreadMetadata[goID]; !goroutineExists {
				trace.ThreadMetadata[goID] = &profileThreadMetadata{
					Name: "Goroutine " + strconv.FormatUint(goID, 10),
				}
			}
		}
	}

	return result
}

// Collect all buckets of samples in the given time range while holding a read lock.
func (p *profileRecorder) getBuckets(relativeStartNS, relativeEndNS uint64) (samplesCount int, buckets []*profileSamplesBucket, trace *profileTrace) {
	p.mutex.RLock()
	defer p.mutex.RUnlock()

	// sampleBucketsHead points at the last stored bucket so it's a good starting point to search backwards for the end.
	var end = p.samplesBucketsHead
	for end.Value != nil && end.Value.(*profileSamplesBucket).relativeTimeNS > relativeEndNS {
		end = end.Prev()
	}

	// Edge case - no items stored before the given endTime.
	if end.Value == nil {
		return 0, nil, nil
	}

	{ // Find the first item after the given startTime.
		var start = end
		var prevBucket *profileSamplesBucket
		samplesCount = 0
		buckets = make([]*profileSamplesBucket, 0, int64((relativeEndNS-relativeStartNS)/uint64(profilerSamplingRate.Nanoseconds()))+1)
		for start.Value != nil {
			var bucket = start.Value.(*profileSamplesBucket)

			// If this bucket's time is before the requests start time, don't collect it (and stop iterating further).
			if bucket.relativeTimeNS < relativeStartNS {
				break
			}

			// If this bucket time is greater than previous the bucket's time, we have exhausted the whole ring buffer
			// before we were able to find the start time. That means the start time is not present and we must break.
			// This happens if the slice duration exceeds the ring buffer capacity.
			if prevBucket != nil && bucket.relativeTimeNS > prevBucket.relativeTimeNS {
				break
			}

			samplesCount += len(bucket.goIDs)
			buckets = append(buckets, bucket)

			start = start.Prev()
			prevBucket = bucket
		}
	}

	// Edge case - if the period requested was too short and we haven't collected enough samples.
	if len(buckets) < 2 {
		return 0, nil, nil
	}

	trace = &profileTrace{
		Frames: p.frames,
		Stacks: p.stacks,
	}
	return samplesCount, buckets, trace
}

func (p *profileRecorder) onTick() {
	elapsedNs := time.Since(p.startTime).Nanoseconds()

	if p.testProfilerPanic > 0 {
		Logger.Printf("Profiler testProfilerPanic == %v\n", p.testProfilerPanic)
		if p.testProfilerPanic == 1 {
			Logger.Println("Profiler panicking onTick()")
			panic("This is an expected panic in Profiler.OnTick() during tests")
		}
		p.testProfilerPanic--
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
	var length = traces.Length()

	// Shouldn't happen but let's be safe and don't store empty buckets.
	if length == 0 {
		return
	}

	var bucket = &profileSamplesBucket{
		relativeTimeNS: elapsedNs,
		stackIDs:       make([]int, length),
		goIDs:          make([]uint64, length),
	}

	// reset buffers
	p.newFrames = p.newFrames[:0]
	p.newStacks = p.newStacks[:0]

	for i := 0; i < length; i++ {
		var stack = traces.Item(i)
		bucket.stackIDs[i] = p.addStackTrace(stack)
		bucket.goIDs[i] = stack.GoID()
	}

	p.mutex.Lock()
	defer p.mutex.Unlock()

	p.stacks = append(p.stacks, p.newStacks...)
	p.frames = append(p.frames, p.newFrames...)

	p.samplesBucketsHead = p.samplesBucketsHead.Next()
	p.samplesBucketsHead.Value = bucket
}

func (p *profileRecorder) addStackTrace(capturedStack traceparser.Trace) int {
	iter := capturedStack.Frames()
	stack := make(profileStack, 0, iter.LengthUpperBound())

	// Originally, we've used `capturedStack.UniqueIdentifier()` as a key but that was incorrect because it also
	// contains function arguments and we want to group stacks by function name and file/line only.
	// Instead, we need to parse frames and we use a list of their indexes as a key.
	// We reuse the same buffer for each stack to avoid allocations; this is a hot spot.
	var expectedBufferLen = cap(stack) * 5 // 4 bytes per frame + 1 byte for space
	if cap(p.stackKeyBuffer) < expectedBufferLen {
		p.stackKeyBuffer = make([]byte, 0, expectedBufferLen)
	} else {
		p.stackKeyBuffer = p.stackKeyBuffer[:0]
	}

	for iter.HasNext() {
		var frame = iter.Next()
		if frameIndex := p.addFrame(frame); frameIndex >= 0 {
			stack = append(stack, frameIndex)

			p.stackKeyBuffer = append(p.stackKeyBuffer, 0) // space

			// The following code is just like binary.AppendUvarint() which isn't yet available in Go 1.18.
			x := uint64(frameIndex) + 1
			for x >= 0x80 {
				p.stackKeyBuffer = append(p.stackKeyBuffer, byte(x)|0x80)
				x >>= 7
			}
			p.stackKeyBuffer = append(p.stackKeyBuffer, byte(x))
		}
	}

	stackIndex, exists := p.stackIndexes[string(p.stackKeyBuffer)]
	if !exists {
		stackIndex = len(p.stacks) + len(p.newStacks)
		p.newStacks = append(p.newStacks, stack)
		p.stackIndexes[string(p.stackKeyBuffer)] = stackIndex
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
		frameIndex = len(p.frames) + len(p.newFrames)
		p.newFrames = append(p.newFrames, &frame)
		p.frameIndexes[string(key)] = frameIndex
	}
	return frameIndex
}

type profileSamplesBucket struct {
	relativeTimeNS uint64
	stackIDs       []int
	goIDs          []uint64
}

// A Ticker holds a channel that delivers “ticks” of a clock at intervals.
type profilerTicker interface {
	// Stop turns off a ticker. After Stop, no more ticks will be sent.
	Stop()

	// TickSource returns a read-only channel of ticks.
	TickSource() <-chan time.Time

	// Ticked is called by the Profiler after a tick is processed to notify the ticker. Used for testing.
	Ticked()
}

type timeTicker struct {
	*time.Ticker
}

func (t *timeTicker) TickSource() <-chan time.Time {
	return t.C
}

func (t *timeTicker) Ticked() {}

func profilerTickerFactoryDefault(d time.Duration) profilerTicker {
	return &timeTicker{time.NewTicker(d)}
}

// We allow overriding the ticker for tests. CI is terribly flaky
// because the time.Ticker doesn't guarantee regular ticks - they may come (a lot) later than the given interval.
var profilerTickerFactory = profilerTickerFactoryDefault

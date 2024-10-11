// Copyright 2015 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package rate

import (
	"context"
	"math"
	"sync"
	"sync/atomic"
	"testing"
	"time"
)

func TestLimit(t *testing.T) {
	if Limit(10) == Inf {
		t.Errorf("Limit(10) == Inf should be false")
	}
}

func closeEnough(a, b Limit) bool {
	return (math.Abs(float64(a)/float64(b)) - 1.0) < 1e-9
}

func TestEvery(t *testing.T) {
	cases := []struct {
		interval time.Duration
		lim      Limit
	}{
		{0, Inf},
		{-1, Inf},
		{1 * time.Nanosecond, Limit(1e9)},
		{1 * time.Microsecond, Limit(1e6)},
		{1 * time.Millisecond, Limit(1e3)},
		{10 * time.Millisecond, Limit(100)},
		{100 * time.Millisecond, Limit(10)},
		{1 * time.Second, Limit(1)},
		{2 * time.Second, Limit(0.5)},
		{time.Duration(2.5 * float64(time.Second)), Limit(0.4)},
		{4 * time.Second, Limit(0.25)},
		{10 * time.Second, Limit(0.1)},
		{time.Duration(math.MaxInt64), Limit(1e9 / float64(math.MaxInt64))},
	}
	for _, tc := range cases {
		lim := Every(tc.interval)
		if !closeEnough(lim, tc.lim) {
			t.Errorf("Every(%v) = %v want %v", tc.interval, lim, tc.lim)
		}
	}
}

const (
	d = 100 * time.Millisecond
)

var (
	t0 = time.Now()
	t1 = t0.Add(time.Duration(1) * d)
	t2 = t0.Add(time.Duration(2) * d)
	t3 = t0.Add(time.Duration(3) * d)
	t4 = t0.Add(time.Duration(4) * d)
	t5 = t0.Add(time.Duration(5) * d)
	t9 = t0.Add(time.Duration(9) * d)
)

type allow struct {
	t    time.Time
	toks float64
	n    int
	ok   bool
}

func run(t *testing.T, lim *Limiter, allows []allow) {
	t.Helper()
	for i, allow := range allows {
		if toks := lim.TokensAt(allow.t); toks != allow.toks {
			t.Errorf("step %d: lim.TokensAt(%v) = %v want %v",
				i, allow.t, toks, allow.toks)
		}
		ok := lim.AllowN(allow.t, allow.n)
		if ok != allow.ok {
			t.Errorf("step %d: lim.AllowN(%v, %v) = %v want %v",
				i, allow.t, allow.n, ok, allow.ok)
		}
	}
}

func TestLimiterBurst1(t *testing.T) {
	run(t, NewLimiter(10, 1), []allow{
		{t0, 1, 1, true},
		{t0, 0, 1, false},
		{t0, 0, 1, false},
		{t1, 1, 1, true},
		{t1, 0, 1, false},
		{t1, 0, 1, false},
		{t2, 1, 2, false}, // burst size is 1, so n=2 always fails
		{t2, 1, 1, true},
		{t2, 0, 1, false},
	})
}

func TestLimiterBurst3(t *testing.T) {
	run(t, NewLimiter(10, 3), []allow{
		{t0, 3, 2, true},
		{t0, 1, 2, false},
		{t0, 1, 1, true},
		{t0, 0, 1, false},
		{t1, 1, 4, false},
		{t2, 2, 1, true},
		{t3, 2, 1, true},
		{t4, 2, 1, true},
		{t4, 1, 1, true},
		{t4, 0, 1, false},
		{t4, 0, 1, false},
		{t9, 3, 3, true},
		{t9, 0, 0, true},
	})
}

func TestLimiterJumpBackwards(t *testing.T) {
	run(t, NewLimiter(10, 3), []allow{
		{t1, 3, 1, true}, // start at t1
		{t0, 2, 1, true}, // jump back to t0, two tokens remain
		{t0, 1, 1, true},
		{t0, 0, 1, false},
		{t0, 0, 1, false},
		{t1, 1, 1, true}, // got a token
		{t1, 0, 1, false},
		{t1, 0, 1, false},
		{t2, 1, 1, true}, // got another token
		{t2, 0, 1, false},
		{t2, 0, 1, false},
	})
}

// Ensure that tokensFromDuration doesn't produce
// rounding errors by truncating nanoseconds.
// See golang.org/issues/34861.
func TestLimiter_noTruncationErrors(t *testing.T) {
	if !NewLimiter(0.7692307692307693, 1).Allow() {
		t.Fatal("expected true")
	}
}

// testTime is a fake time used for testing.
type testTime struct {
	mu     sync.Mutex
	cur    time.Time   // current fake time
	timers []testTimer // fake timers
}

// testTimer is a fake timer.
type testTimer struct {
	when time.Time
	ch   chan<- time.Time
}

// now returns the current fake time.
func (tt *testTime) now() time.Time {
	tt.mu.Lock()
	defer tt.mu.Unlock()
	return tt.cur
}

// newTimer creates a fake timer. It returns the channel,
// a function to stop the timer (which we don't care about),
// and a function to advance to the next timer.
func (tt *testTime) newTimer(dur time.Duration) (<-chan time.Time, func() bool, func()) {
	tt.mu.Lock()
	defer tt.mu.Unlock()
	ch := make(chan time.Time, 1)
	timer := testTimer{
		when: tt.cur.Add(dur),
		ch:   ch,
	}
	tt.timers = append(tt.timers, timer)
	return ch, func() bool { return true }, tt.advanceToTimer
}

// since returns the fake time since the given time.
func (tt *testTime) since(t time.Time) time.Duration {
	tt.mu.Lock()
	defer tt.mu.Unlock()
	return tt.cur.Sub(t)
}

// advance advances the fake time.
func (tt *testTime) advance(dur time.Duration) {
	tt.mu.Lock()
	defer tt.mu.Unlock()
	tt.advanceUnlocked(dur)
}

// advanceUnlock advances the fake time, assuming it is already locked.
func (tt *testTime) advanceUnlocked(dur time.Duration) {
	tt.cur = tt.cur.Add(dur)
	i := 0
	for i < len(tt.timers) {
		if tt.timers[i].when.After(tt.cur) {
			i++
		} else {
			tt.timers[i].ch <- tt.cur
			copy(tt.timers[i:], tt.timers[i+1:])
			tt.timers = tt.timers[:len(tt.timers)-1]
		}
	}
}

// advanceToTimer advances the time to the next timer.
func (tt *testTime) advanceToTimer() {
	tt.mu.Lock()
	defer tt.mu.Unlock()
	if len(tt.timers) == 0 {
		panic("no timer")
	}
	when := tt.timers[0].when
	for _, timer := range tt.timers[1:] {
		if timer.when.Before(when) {
			when = timer.when
		}
	}
	tt.advanceUnlocked(when.Sub(tt.cur))
}

// makeTestTime hooks the testTimer into the package.
func makeTestTime(t *testing.T) *testTime {
	return &testTime{
		cur: time.Now(),
	}
}

func TestSimultaneousRequests(t *testing.T) {
	const (
		limit       = 1
		burst       = 5
		numRequests = 15
	)
	var (
		wg    sync.WaitGroup
		numOK = uint32(0)
	)

	// Very slow replenishing bucket.
	lim := NewLimiter(limit, burst)

	// Tries to take a token, atomically updates the counter and decreases the wait
	// group counter.
	f := func() {
		defer wg.Done()
		if ok := lim.Allow(); ok {
			atomic.AddUint32(&numOK, 1)
		}
	}

	wg.Add(numRequests)
	for i := 0; i < numRequests; i++ {
		go f()
	}
	wg.Wait()
	if numOK != burst {
		t.Errorf("numOK = %d, want %d", numOK, burst)
	}
}

func TestLongRunningQPS(t *testing.T) {
	// The test runs for a few (fake) seconds executing many requests
	// and then checks that overall number of requests is reasonable.
	const (
		limit = 100
		burst = 100
	)
	var (
		numOK = int32(0)
		tt    = makeTestTime(t)
	)

	lim := NewLimiter(limit, burst)

	start := tt.now()
	end := start.Add(5 * time.Second)
	for tt.now().Before(end) {
		if ok := lim.AllowN(tt.now(), 1); ok {
			numOK++
		}

		// This will still offer ~500 requests per second, but won't consume
		// outrageous amount of CPU.
		tt.advance(2 * time.Millisecond)
	}
	elapsed := tt.since(start)
	ideal := burst + (limit * float64(elapsed) / float64(time.Second))

	// We should never get more requests than allowed.
	if want := int32(ideal + 1); numOK > want {
		t.Errorf("numOK = %d, want %d (ideal %f)", numOK, want, ideal)
	}
	// We should get very close to the number of requests allowed.
	if want := int32(0.999 * ideal); numOK < want {
		t.Errorf("numOK = %d, want %d (ideal %f)", numOK, want, ideal)
	}
}

// A request provides the arguments to lim.reserveN(t, n) and the expected results (act, ok).
type request struct {
	t   time.Time
	n   int
	act time.Time
	ok  bool
}

// dFromDuration converts a duration to the nearest multiple of the global constant d.
func dFromDuration(dur time.Duration) int {
	// Add d/2 to dur so that integer division will round to
	// the nearest multiple instead of truncating.
	// (We don't care about small inaccuracies.)
	return int((dur + (d / 2)) / d)
}

// dSince returns multiples of d since t0
func dSince(t time.Time) int {
	return dFromDuration(t.Sub(t0))
}

func runReserve(t *testing.T, lim *Limiter, req request) *Reservation {
	t.Helper()
	return runReserveMax(t, lim, req, InfDuration)
}

// runReserveMax attempts to reserve req.n tokens at time req.t, limiting the delay until action to
// maxReserve. It checks whether the response matches req.act and req.ok. If not, it reports a test
// error including the difference from expected durations in multiples of d (global constant).
func runReserveMax(t *testing.T, lim *Limiter, req request, maxReserve time.Duration) *Reservation {
	t.Helper()
	r := lim.reserveN(req.t, req.n, maxReserve)
	if r.ok && (dSince(r.timeToAct) != dSince(req.act)) || r.ok != req.ok {
		t.Errorf("lim.reserveN(t%d, %v, %v) = (t%d, %v) want (t%d, %v)",
			dSince(req.t), req.n, maxReserve, dSince(r.timeToAct), r.ok, dSince(req.act), req.ok)
	}
	return &r
}

func TestSimpleReserve(t *testing.T) {
	lim := NewLimiter(10, 2)

	runReserve(t, lim, request{t0, 2, t0, true})
	runReserve(t, lim, request{t0, 2, t2, true})
	runReserve(t, lim, request{t3, 2, t4, true})
}

func TestMix(t *testing.T) {
	lim := NewLimiter(10, 2)

	runReserve(t, lim, request{t0, 3, t1, false}) // should return false because n > Burst
	runReserve(t, lim, request{t0, 2, t0, true})
	run(t, lim, []allow{{t1, 1, 2, false}}) // not enough tokens - don't allow
	runReserve(t, lim, request{t1, 2, t2, true})
	run(t, lim, []allow{{t1, -1, 1, false}}) // negative tokens - don't allow
	run(t, lim, []allow{{t3, 1, 1, true}})
}

func TestCancelInvalid(t *testing.T) {
	lim := NewLimiter(10, 2)

	runReserve(t, lim, request{t0, 2, t0, true})
	r := runReserve(t, lim, request{t0, 3, t3, false})
	r.CancelAt(t0)                               // should have no effect
	runReserve(t, lim, request{t0, 2, t2, true}) // did not get extra tokens
}

func TestCancelLast(t *testing.T) {
	lim := NewLimiter(10, 2)

	runReserve(t, lim, request{t0, 2, t0, true})
	r := runReserve(t, lim, request{t0, 2, t2, true})
	r.CancelAt(t1) // got 2 tokens back
	runReserve(t, lim, request{t1, 2, t2, true})
}

func TestCancelTooLate(t *testing.T) {
	lim := NewLimiter(10, 2)

	runReserve(t, lim, request{t0, 2, t0, true})
	r := runReserve(t, lim, request{t0, 2, t2, true})
	r.CancelAt(t3) // too late to cancel - should have no effect
	runReserve(t, lim, request{t3, 2, t4, true})
}

func TestCancel0Tokens(t *testing.T) {
	lim := NewLimiter(10, 2)

	runReserve(t, lim, request{t0, 2, t0, true})
	r := runReserve(t, lim, request{t0, 1, t1, true})
	runReserve(t, lim, request{t0, 1, t2, true})
	r.CancelAt(t0) // got 0 tokens back
	runReserve(t, lim, request{t0, 1, t3, true})
}

func TestCancel1Token(t *testing.T) {
	lim := NewLimiter(10, 2)

	runReserve(t, lim, request{t0, 2, t0, true})
	r := runReserve(t, lim, request{t0, 2, t2, true})
	runReserve(t, lim, request{t0, 1, t3, true})
	r.CancelAt(t2) // got 1 token back
	runReserve(t, lim, request{t2, 2, t4, true})
}

func TestCancelMulti(t *testing.T) {
	lim := NewLimiter(10, 4)

	runReserve(t, lim, request{t0, 4, t0, true})
	rA := runReserve(t, lim, request{t0, 3, t3, true})
	runReserve(t, lim, request{t0, 1, t4, true})
	rC := runReserve(t, lim, request{t0, 1, t5, true})
	rC.CancelAt(t1) // get 1 token back
	rA.CancelAt(t1) // get 2 tokens back, as if C was never reserved
	runReserve(t, lim, request{t1, 3, t5, true})
}

func TestReserveJumpBack(t *testing.T) {
	lim := NewLimiter(10, 2)

	runReserve(t, lim, request{t1, 2, t1, true}) // start at t1
	runReserve(t, lim, request{t0, 1, t1, true}) // should violate Limit,Burst
	runReserve(t, lim, request{t2, 2, t3, true})
	// burst size is 2, so n=3 always fails, and the state of lim should not be changed
	runReserve(t, lim, request{t0, 3, time.Time{}, false})
	runReserve(t, lim, request{t2, 1, t4, true})
	// the maxReserve is not enough so it fails, and the state of lim should not be changed
	runReserveMax(t, lim, request{t0, 2, time.Time{}, false}, d)
	runReserve(t, lim, request{t2, 1, t5, true})
}

func TestReserveJumpBackCancel(t *testing.T) {
	lim := NewLimiter(10, 2)

	runReserve(t, lim, request{t1, 2, t1, true}) // start at t1
	r := runReserve(t, lim, request{t1, 2, t3, true})
	runReserve(t, lim, request{t1, 1, t4, true})
	r.CancelAt(t0)                               // cancel at t0, get 1 token back
	runReserve(t, lim, request{t1, 2, t4, true}) // should violate Limit,Burst
}

func TestReserveSetLimit(t *testing.T) {
	lim := NewLimiter(5, 2)

	runReserve(t, lim, request{t0, 2, t0, true})
	runReserve(t, lim, request{t0, 2, t4, true})
	lim.SetLimitAt(t2, 10)
	runReserve(t, lim, request{t2, 1, t4, true}) // violates Limit and Burst
}

func TestReserveSetBurst(t *testing.T) {
	lim := NewLimiter(5, 2)

	runReserve(t, lim, request{t0, 2, t0, true})
	runReserve(t, lim, request{t0, 2, t4, true})
	lim.SetBurstAt(t3, 4)
	runReserve(t, lim, request{t0, 4, t9, true}) // violates Limit and Burst
}

func TestReserveSetLimitCancel(t *testing.T) {
	lim := NewLimiter(5, 2)

	runReserve(t, lim, request{t0, 2, t0, true})
	r := runReserve(t, lim, request{t0, 2, t4, true})
	lim.SetLimitAt(t2, 10)
	r.CancelAt(t2) // 2 tokens back
	runReserve(t, lim, request{t2, 2, t3, true})
}

func TestReserveMax(t *testing.T) {
	lim := NewLimiter(10, 2)
	maxT := d

	runReserveMax(t, lim, request{t0, 2, t0, true}, maxT)
	runReserveMax(t, lim, request{t0, 1, t1, true}, maxT)  // reserve for close future
	runReserveMax(t, lim, request{t0, 1, t2, false}, maxT) // time to act too far in the future
}

type wait struct {
	name   string
	ctx    context.Context
	n      int
	delay  int // in multiples of d
	nilErr bool
}

func runWait(t *testing.T, tt *testTime, lim *Limiter, w wait) {
	t.Helper()
	start := tt.now()
	err := lim.wait(w.ctx, w.n, start, tt.newTimer)
	delay := tt.since(start)

	if (w.nilErr && err != nil) || (!w.nilErr && err == nil) || !waitDelayOk(w.delay, delay) {
		errString := "<nil>"
		if !w.nilErr {
			errString = "<non-nil error>"
		}
		t.Errorf("lim.WaitN(%v, lim, %v) = %v with delay %v; want %v with delay %v (±%v)",
			w.name, w.n, err, delay, errString, d*time.Duration(w.delay), d/2)
	}
}

// waitDelayOk reports whether a duration spent in WaitN is “close enough” to
// wantD multiples of d, given scheduling slop.
func waitDelayOk(wantD int, got time.Duration) bool {
	gotD := dFromDuration(got)

	// The actual time spent waiting will be REDUCED by the amount of time spent
	// since the last call to the limiter. We expect the time in between calls to
	// be executing simple, straight-line, non-blocking code, so it should reduce
	// the wait time by no more than half a d, which would round to exactly wantD.
	if gotD < wantD {
		return false
	}

	// The actual time spend waiting will be INCREASED by the amount of scheduling
	// slop in the platform's sleep syscall, plus the amount of time spent executing
	// straight-line code before measuring the elapsed duration.
	//
	// The latter is surely less than half a d, but the former is empirically
	// sometimes larger on a number of platforms for a number of reasons.
	// NetBSD and OpenBSD tend to overshoot sleeps by a wide margin due to a
	// suspected platform bug; see https://go.dev/issue/44067 and
	// https://go.dev/issue/50189.
	// Longer delays were also also observed on slower builders with Linux kernels
	// (linux-ppc64le-buildlet, android-amd64-emu), and on Solaris and Plan 9.
	//
	// Since d is already fairly generous, we take 150% of wantD rounded up —
	// that's at least enough to account for the overruns we've seen so far in
	// practice.
	maxD := (wantD*3 + 1) / 2
	return gotD <= maxD
}

func TestWaitSimple(t *testing.T) {
	tt := makeTestTime(t)

	lim := NewLimiter(10, 3)

	ctx, cancel := context.WithCancel(context.Background())
	cancel()
	runWait(t, tt, lim, wait{"already-cancelled", ctx, 1, 0, false})

	runWait(t, tt, lim, wait{"exceed-burst-error", context.Background(), 4, 0, false})

	runWait(t, tt, lim, wait{"act-now", context.Background(), 2, 0, true})
	runWait(t, tt, lim, wait{"act-later", context.Background(), 3, 2, true})
}

func TestWaitCancel(t *testing.T) {
	tt := makeTestTime(t)

	lim := NewLimiter(10, 3)

	ctx, cancel := context.WithCancel(context.Background())
	runWait(t, tt, lim, wait{"act-now", ctx, 2, 0, true}) // after this lim.tokens = 1
	ch, _, _ := tt.newTimer(d)
	go func() {
		<-ch
		cancel()
	}()
	runWait(t, tt, lim, wait{"will-cancel", ctx, 3, 1, false})
	// should get 3 tokens back, and have lim.tokens = 2
	t.Logf("tokens:%v last:%v lastEvent:%v", lim.tokens, lim.last, lim.lastEvent)
	runWait(t, tt, lim, wait{"act-now-after-cancel", context.Background(), 2, 0, true})
}

func TestWaitTimeout(t *testing.T) {
	tt := makeTestTime(t)

	lim := NewLimiter(10, 3)

	ctx, cancel := context.WithTimeout(context.Background(), d)
	defer cancel()
	runWait(t, tt, lim, wait{"act-now", ctx, 2, 0, true})
	runWait(t, tt, lim, wait{"w-timeout-err", ctx, 3, 0, false})
}

func TestWaitInf(t *testing.T) {
	tt := makeTestTime(t)

	lim := NewLimiter(Inf, 0)

	runWait(t, tt, lim, wait{"exceed-burst-no-error", context.Background(), 3, 0, true})
}

func BenchmarkAllowN(b *testing.B) {
	lim := NewLimiter(Every(1*time.Second), 1)
	now := time.Now()
	b.ReportAllocs()
	b.ResetTimer()
	b.RunParallel(func(pb *testing.PB) {
		for pb.Next() {
			lim.AllowN(now, 1)
		}
	})
}

func BenchmarkWaitNNoDelay(b *testing.B) {
	lim := NewLimiter(Limit(b.N), b.N)
	ctx := context.Background()
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		lim.WaitN(ctx, 1)
	}
}

func TestZeroLimit(t *testing.T) {
	r := NewLimiter(0, 1)
	if !r.Allow() {
		t.Errorf("Limit(0, 1) want true when first used")
	}
	if r.Allow() {
		t.Errorf("Limit(0, 1) want false when already used")
	}
}

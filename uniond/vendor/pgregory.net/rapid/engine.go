// Copyright 2019 Gregory Petrosyan <gregory.petrosyan@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package rapid

import (
	"bytes"
	"encoding/binary"
	"flag"
	"fmt"
	"log"
	"os"
	"regexp"
	"runtime"
	"strings"
	"sync"
	"testing"
	"time"
)

const (
	small             = 5
	invalidChecksMult = 10
	exampleMaxTries   = 1000

	tracebackLen  = 32
	tracebackStop = "pgregory.net/rapid.checkOnce"
	runtimePrefix = "runtime."
)

var (
	flags cmdline

	tracebackBlacklist = map[string]bool{
		"pgregory.net/rapid.(*customGen[...]).maybeValue.func1": true,
		"pgregory.net/rapid.runAction.func1":                    true,
	}
)

type cmdline struct {
	checks     int
	steps      int
	failfile   string
	nofailfile bool
	seed       uint64
	log        bool
	verbose    bool
	debug      bool
	debugvis   bool
	shrinkTime time.Duration
}

func init() {
	flag.IntVar(&flags.checks, "rapid.checks", 100, "rapid: number of checks to perform")
	flag.IntVar(&flags.steps, "rapid.steps", 100, "rapid: number of state machine steps to perform")
	flag.StringVar(&flags.failfile, "rapid.failfile", "", "rapid: fail file to use to reproduce test failure")
	flag.BoolVar(&flags.nofailfile, "rapid.nofailfile", false, "rapid: do not write fail files on test failures")
	flag.Uint64Var(&flags.seed, "rapid.seed", 0, "rapid: PRNG seed to start with (0 to use a random one)")
	flag.BoolVar(&flags.log, "rapid.log", false, "rapid: eager verbose output to stdout (to aid with unrecoverable test failures)")
	flag.BoolVar(&flags.verbose, "rapid.v", false, "rapid: verbose output")
	flag.BoolVar(&flags.debug, "rapid.debug", false, "rapid: debugging output")
	flag.BoolVar(&flags.debugvis, "rapid.debugvis", false, "rapid: debugging visualization")
	flag.DurationVar(&flags.shrinkTime, "rapid.shrinktime", 30*time.Second, "rapid: maximum time to spend on test case minimization")
}

func assert(ok bool) {
	if !ok {
		panic("assertion failed")
	}
}

func assertf(ok bool, format string, args ...any) {
	if !ok {
		panic(fmt.Sprintf(format, args...))
	}
}

func assertValidRange(min int, max int) {
	if max >= 0 && min > max {
		panic(fmt.Sprintf("invalid range [%d, %d]", min, max))
	}
}

// Check fails the current test if rapid can find a test case which falsifies prop.
//
// Property is falsified in case of a panic or a call to
// [*T.Fatalf], [*T.Fatal], [*T.Errorf], [*T.Error], [*T.FailNow] or [*T.Fail].
func Check(t *testing.T, prop func(*T)) {
	t.Helper()
	checkTB(t, prop)
}

// MakeCheck is a convenience function for defining subtests suitable for
// [*testing.T.Run]. It allows you to write this:
//
//	t.Run("subtest name", rapid.MakeCheck(func(t *rapid.T) {
//	    // test code
//	}))
//
// instead of this:
//
//	t.Run("subtest name", func(t *testing.T) {
//	    rapid.Check(t, func(t *rapid.T) {
//	        // test code
//	    })
//	})
func MakeCheck(prop func(*T)) func(*testing.T) {
	return func(t *testing.T) {
		t.Helper()
		checkTB(t, prop)
	}
}

// MakeFuzz creates a fuzz target for [*testing.F.Fuzz]:
//
//	func FuzzFoo(f *testing.F) {
//	    f.Fuzz(rapid.MakeFuzz(func(t *rapid.T) {
//	        // test code
//	    }))
//	}
func MakeFuzz(prop func(*T)) func(*testing.T, []byte) {
	return func(t *testing.T, input []byte) {
		t.Helper()
		checkFuzz(t, prop, input)
	}
}

func checkFuzz(tb tb, prop func(*T), input []byte) {
	tb.Helper()

	var buf []uint64
	for len(input) > 0 {
		var tmp [8]byte
		n := copy(tmp[:], input)
		buf = append(buf, binary.LittleEndian.Uint64(tmp[:]))
		input = input[n:]
	}

	t := newT(tb, newBufBitStream(buf, false), flags.verbose, nil)
	err := checkOnce(t, prop)

	switch {
	case err == nil:
		// do nothing
	case err.isInvalidData():
		tb.SkipNow()
	case err.isStopTest():
		tb.Fatalf("[rapid] failed: %v", err)
	default:
		tb.Fatalf("[rapid] panic: %v\nTraceback:\n%v", err, traceback(err))
	}
}

func checkTB(tb tb, prop func(*T)) {
	tb.Helper()

	checks := flags.checks
	if testing.Short() {
		checks /= 5
	}

	start := time.Now()
	valid, invalid, seed, buf, err1, err2 := doCheck(tb, flags.failfile, checks, baseSeed(), prop)
	dt := time.Since(start)

	if err1 == nil && err2 == nil {
		if valid == checks {
			tb.Logf("[rapid] OK, passed %v tests (%v)", valid, dt)
		} else {
			tb.Errorf("[rapid] only generated %v valid tests from %v total (%v)", valid, valid+invalid, dt)
		}
	} else {
		repr := fmt.Sprintf("-rapid.seed=%d", seed)
		if flags.failfile != "" && seed == 0 {
			repr = fmt.Sprintf("-rapid.failfile=%q", flags.failfile)
		} else if !flags.nofailfile {
			_, failfile := failFileName(tb.Name())
			out := captureTestOutput(tb, prop, buf)
			err := saveFailFile(failfile, rapidVersion, out, seed, buf)
			if err == nil {
				repr = fmt.Sprintf("-rapid.failfile=%q (or -rapid.seed=%d)", failfile, seed)
			} else {
				tb.Logf("[rapid] %v", err)
			}
		}

		name := regexp.QuoteMeta(tb.Name())
		if traceback(err1) == traceback(err2) {
			if err2.isStopTest() {
				tb.Errorf("[rapid] failed after %v tests: %v\nTo reproduce, specify -run=%q %v\nFailed test output:", valid, err2, name, repr)
			} else {
				tb.Errorf("[rapid] panic after %v tests: %v\nTo reproduce, specify -run=%q %v\nTraceback:\n%vFailed test output:", valid, err2, name, repr, traceback(err2))
			}
		} else {
			tb.Errorf("[rapid] flaky test, can not reproduce a failure\nTo try to reproduce, specify -run=%q %v\nTraceback (%v):\n%vOriginal traceback (%v):\n%vFailed test output:", name, repr, err2, traceback(err2), err1, traceback(err1))
		}

		_ = checkOnce(newT(tb, newBufBitStream(buf, false), true, nil), prop) // output using (*testing.T).Log for proper line numbers
	}

	if tb.Failed() {
		tb.FailNow() // do not try to run any checks after the first failed one
	}
}

func doCheck(tb tb, failfile string, checks int, seed uint64, prop func(*T)) (int, int, uint64, []uint64, *testError, *testError) {
	tb.Helper()

	assertf(!tb.Failed(), "check function called with *testing.T which has already failed")

	if failfile != "" {
		buf, err1, err2 := checkFailFile(tb, failfile, prop)
		if err1 != nil || err2 != nil {
			return 0, 0, 0, buf, err1, err2
		}
	}

	seed, valid, invalid, err1 := findBug(tb, checks, seed, prop)
	if err1 == nil {
		return valid, invalid, 0, nil, nil, nil
	}

	s := newRandomBitStream(seed, true)
	t := newT(tb, s, flags.verbose, nil)
	t.Logf("[rapid] trying to reproduce the failure")
	err2 := checkOnce(t, prop)
	if !sameError(err1, err2) {
		return valid, invalid, seed, s.data, err1, err2
	}

	t.Logf("[rapid] trying to minimize the failing test case")
	buf, err3 := shrink(tb, s.recordedBits, err2, prop)

	return valid, invalid, seed, buf, err2, err3
}

func checkFailFile(tb tb, failfile string, prop func(*T)) ([]uint64, *testError, *testError) {
	tb.Helper()

	version, _, buf, err := loadFailFile(failfile)
	if err != nil {
		tb.Logf("[rapid] ignoring fail file: %v", err)
		return nil, nil, nil
	}
	if version != rapidVersion {
		tb.Logf("[rapid] ignoring fail file: version %q differs from rapid version %q", version, rapidVersion)
		return nil, nil, nil
	}

	s1 := newBufBitStream(buf, false)
	t1 := newT(tb, s1, flags.verbose, nil)
	err1 := checkOnce(t1, prop)
	if err1 == nil {
		return nil, nil, nil
	}
	if err1.isInvalidData() {
		tb.Logf("[rapid] fail file %q is no longer valid", failfile)
		return nil, nil, nil
	}

	s2 := newBufBitStream(buf, false)
	t2 := newT(tb, s2, flags.verbose, nil)
	t2.Logf("[rapid] trying to reproduce the failure")
	err2 := checkOnce(t2, prop)

	return buf, err1, err2
}

func findBug(tb tb, checks int, seed uint64, prop func(*T)) (uint64, int, int, *testError) {
	tb.Helper()

	var (
		r       = newRandomBitStream(0, false)
		t       = newT(tb, r, flags.verbose, nil)
		valid   = 0
		invalid = 0
	)

	for valid < checks && invalid < checks*invalidChecksMult {
		seed += uint64(valid) + uint64(invalid)
		r.init(seed)
		var start time.Time
		if t.shouldLog() {
			t.Logf("[rapid] test #%v start (seed %v)", valid+invalid+1, seed)
			start = time.Now()
		}

		err := checkOnce(t, prop)
		if err == nil {
			if t.shouldLog() {
				t.Logf("[rapid] test #%v OK (%v)", valid+invalid+1, time.Since(start))
			}
			valid++
		} else if err.isInvalidData() {
			if t.shouldLog() {
				t.Logf("[rapid] test #%v invalid (%v)", valid+invalid+1, time.Since(start))
			}
			invalid++
		} else {
			if t.shouldLog() {
				t.Logf("[rapid] test #%v failed: %v", valid+invalid+1, err)
			}
			return seed, valid, invalid, err
		}
	}

	return 0, valid, invalid, nil
}

func checkOnce(t *T, prop func(*T)) (err *testError) {
	if t.tbLog && t.tb != nil {
		t.tb.Helper()
	}
	defer func() { err = panicToError(recover(), 3) }()

	prop(t)
	t.failOnError()

	return nil
}

func captureTestOutput(tb tb, prop func(*T), buf []uint64) []byte {
	var b bytes.Buffer
	l := log.New(&b, fmt.Sprintf("%s ", tb.Name()), log.Lmsgprefix|log.Ldate|log.Ltime)
	_ = checkOnce(newT(tb, newBufBitStream(buf, false), false, l), prop)
	return b.Bytes()
}

type invalidData string
type stopTest string

type testError struct {
	data      any
	traceback string
}

func panicToError(p any, skip int) *testError {
	if p == nil {
		return nil
	}

	callers := make([]uintptr, tracebackLen)
	callers = callers[:runtime.Callers(skip, callers)]
	frames := runtime.CallersFrames(callers)

	b := &strings.Builder{}
	f, more, skipSpecial := runtime.Frame{}, true, true
	for more && !strings.HasSuffix(f.Function, tracebackStop) {
		f, more = frames.Next()

		if skipSpecial && (tracebackBlacklist[f.Function] || strings.HasPrefix(f.Function, runtimePrefix)) {
			continue
		}
		skipSpecial = false

		_, err := fmt.Fprintf(b, "    %s:%d in %s\n", f.File, f.Line, f.Function)
		assert(err == nil)
	}

	return &testError{
		data:      p,
		traceback: b.String(),
	}
}

func (err *testError) Error() string {
	if msg, ok := err.data.(stopTest); ok {
		return string(msg)
	}

	if msg, ok := err.data.(invalidData); ok {
		return fmt.Sprintf("invalid data: %s", string(msg))
	}

	return fmt.Sprintf("%v", err.data)
}

func (err *testError) isInvalidData() bool {
	_, ok := err.data.(invalidData)
	return ok
}

func (err *testError) isStopTest() bool {
	_, ok := err.data.(stopTest)
	return ok
}

func sameError(err1 *testError, err2 *testError) bool {
	return errorString(err1) == errorString(err2) && traceback(err1) == traceback(err2)
}

func errorString(err *testError) string {
	if err == nil {
		return ""
	}

	return err.Error()
}

func traceback(err *testError) string {
	if err == nil {
		return "    <no error>\n"
	}

	return err.traceback
}

// TB is a common interface between [*testing.T], [*testing.B] and [*T].
type TB interface {
	Helper()
	Name() string
	Logf(format string, args ...any)
	Log(args ...any)
	Skipf(format string, args ...any)
	Skip(args ...any)
	SkipNow()
	Errorf(format string, args ...any)
	Error(args ...any)
	Fatalf(format string, args ...any)
	Fatal(args ...any)
	FailNow()
	Fail()
	Failed() bool
}

// tb is a private copy of TB, made to avoid T having public fields
type tb interface {
	Helper()
	Name() string
	Logf(format string, args ...any)
	Log(args ...any)
	Skipf(format string, args ...any)
	Skip(args ...any)
	SkipNow()
	Errorf(format string, args ...any)
	Error(args ...any)
	Fatalf(format string, args ...any)
	Fatal(args ...any)
	FailNow()
	Fail()
	Failed() bool
}

// T is similar to [testing.T], but with extra bookkeeping for property-based tests.
//
// For tests to be reproducible, they should generally run in a single goroutine.
// If concurrency is unavoidable, methods on *T, such as [*testing.T.Helper] and [*T.Errorf],
// are safe for concurrent calls, but *Generator.Draw from a given *T is not.
type T struct {
	tb       // unnamed to force re-export of (*T).Helper()
	tbLog    bool
	rawLog   *log.Logger
	s        bitStream
	draws    int
	refDraws []any
	mu       sync.RWMutex
	failed   stopTest
}

func newT(tb tb, s bitStream, tbLog bool, rawLog *log.Logger, refDraws ...any) *T {
	t := &T{
		tb:       tb,
		tbLog:    tbLog,
		rawLog:   rawLog,
		s:        s,
		refDraws: refDraws,
	}

	if rawLog == nil && flags.log {
		testName := "rapid test"
		if tb != nil {
			testName = tb.Name()
		}

		t.rawLog = log.New(os.Stdout, fmt.Sprintf("[%v] ", testName), 0)
	}

	return t
}

func (t *T) shouldLog() bool {
	return t.rawLog != nil || (t.tbLog && t.tb != nil)
}

func (t *T) Logf(format string, args ...any) {
	if t.rawLog != nil {
		t.rawLog.Printf(format, args...)
	} else if t.tbLog && t.tb != nil {
		t.tb.Helper()
		t.tb.Logf(format, args...)
	}
}

func (t *T) Log(args ...any) {
	if t.rawLog != nil {
		t.rawLog.Print(args...)
	} else if t.tbLog && t.tb != nil {
		t.tb.Helper()
		t.tb.Log(args...)
	}
}

// Skipf is equivalent to [T.Logf] followed by [T.SkipNow].
func (t *T) Skipf(format string, args ...any) {
	if t.tbLog && t.tb != nil {
		t.tb.Helper()
	}
	t.Logf(format, args...)
	t.skip(fmt.Sprintf(format, args...))
}

// Skip is equivalent to [T.Log] followed by [T.SkipNow].
func (t *T) Skip(args ...any) {
	if t.tbLog && t.tb != nil {
		t.tb.Helper()
	}
	t.Log(args...)
	t.skip(fmt.Sprint(args...))
}

// SkipNow marks the current test case as invalid (except state machine
// tests, where it marks current action as non-applicable instead).
// If too many test cases are skipped, rapid will mark the test as failing
// due to inability to generate enough valid test cases.
//
// Prefer *Generator.Filter to SkipNow, and prefer generators that always produce
// valid test cases to Filter.
func (t *T) SkipNow() {
	t.skip("(*T).SkipNow() called")
}

// Errorf is equivalent to [T.Logf] followed by [T.Fail].
func (t *T) Errorf(format string, args ...any) {
	if t.tbLog && t.tb != nil {
		t.tb.Helper()
	}
	t.Logf(format, args...)
	t.fail(false, fmt.Sprintf(format, args...))
}

// Error is equivalent to [T.Log] followed by [T.Fail].
func (t *T) Error(args ...any) {
	if t.tbLog && t.tb != nil {
		t.tb.Helper()
	}
	t.Log(args...)
	t.fail(false, fmt.Sprint(args...))
}

// Fatalf is equivalent to [T.Logf] followed by [T.FailNow].
func (t *T) Fatalf(format string, args ...any) {
	if t.tbLog && t.tb != nil {
		t.tb.Helper()
	}
	t.Logf(format, args...)
	t.fail(true, fmt.Sprintf(format, args...))
}

// Fatal is equivalent to [T.Log] followed by [T.FailNow].
func (t *T) Fatal(args ...any) {
	if t.tbLog && t.tb != nil {
		t.tb.Helper()
	}
	t.Log(args...)
	t.fail(true, fmt.Sprint(args...))
}

func (t *T) FailNow() {
	t.fail(true, "(*T).FailNow() called")
}

func (t *T) Fail() {
	t.fail(false, "(*T).Fail() called")
}

func (t *T) Failed() bool {
	t.mu.RLock()
	defer t.mu.RUnlock()

	return t.failed != ""
}

func (t *T) skip(msg string) {
	panic(invalidData(msg))
}

func (t *T) fail(now bool, msg string) {
	t.mu.Lock()
	defer t.mu.Unlock()

	t.failed = stopTest(msg)
	if now {
		panic(t.failed)
	}
}

func (t *T) failOnError() {
	t.mu.RLock()
	defer t.mu.RUnlock()

	if t.failed != "" {
		panic(t.failed)
	}
}

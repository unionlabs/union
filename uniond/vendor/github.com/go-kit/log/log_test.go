package log_test

import (
	"bytes"
	"runtime"
	"sync"
	"testing"

	"github.com/go-kit/log"
)

func TestContext(t *testing.T) {
	t.Parallel()
	buf := &bytes.Buffer{}
	logger := log.NewLogfmtLogger(buf)

	kvs := []interface{}{"a", 123}
	lc := log.With(logger, kvs...)
	kvs[1] = 0 // With should copy its key values

	lc = log.With(lc, "b", "c") // With should stack
	if err := lc.Log("msg", "message"); err != nil {
		t.Fatal(err)
	}
	if want, have := "a=123 b=c msg=message\n", buf.String(); want != have {
		t.Errorf("\nwant: %shave: %s", want, have)
	}

	buf.Reset()
	lc = log.WithPrefix(lc, "p", "first")
	if err := lc.Log("msg", "message"); err != nil {
		t.Fatal(err)
	}
	if want, have := "p=first a=123 b=c msg=message\n", buf.String(); want != have {
		t.Errorf("\nwant: %shave: %s", want, have)
	}
}

func TestContextMissingValue(t *testing.T) {
	t.Parallel()
	var output []interface{}
	logger := log.Logger(log.LoggerFunc(func(keyvals ...interface{}) error {
		output = keyvals
		return nil
	}))

	log.WithPrefix(log.With(logger, "k1"), "k0").Log("k2")
	if want, have := 6, len(output); want != have {
		t.Errorf("want len(output) == %v, have %v", want, have)
	}
	for i := 1; i < 6; i += 2 {
		if want, have := log.ErrMissingValue, output[i]; want != have {
			t.Errorf("want output[%d] == %#v, have %#v", i, want, have)
		}
	}
}

func TestWithPrefixAndSuffix(t *testing.T) {
	t.Parallel()
	var output []interface{}
	logger := log.Logger(log.LoggerFunc(func(keyvals ...interface{}) error {
		output = keyvals
		return nil
	}))

	lc := log.WithPrefix(logger, "a", "first")
	lc = log.WithSuffix(lc, "z", "last")
	if err := lc.Log("msg", "message"); err != nil {
		t.Fatal(err)
	}
	if want, have := 6, len(output); want != have {
		t.Errorf("want len(output) == %v, have %v", want, have)
	}
	want := []string{"a", "first", "msg", "message", "z", "last"}
	for i := 0; i < 6; i++ {
		if want, have := want[i], output[i]; want != have {
			t.Errorf("want output[%d] == %#v, have %#v", i, want, have)
		}
	}

	lc = log.With(logger, "b", "second")
	lc = log.WithPrefix(lc, "a", "first")
	lc = log.With(lc, "c", "third")
	lc = log.WithSuffix(lc, "z", "last")
	lc = log.WithSuffix(lc, "aa", "sequel")
	if err := lc.Log("msg", "message"); err != nil {
		t.Fatal(err)
	}
	if want, have := 12, len(output); want != have {
		t.Errorf("want len(output) == %v, have %v", want, have)
	}
	want = []string{
		"a", "first",
		"b", "second",
		"c", "third",
		"msg", "message",
		"z", "last",
		"aa", "sequel",
	}
	for i := 0; i < 12; i++ {
		if want, have := want[i], output[i]; want != have {
			t.Errorf("want output[%d] == %#v, have %#v", i, want, have)
		}
	}
}

// Test that context.Log has a consistent function stack depth when binding
// Valuers, regardless of how many times With has been called.
func TestContextStackDepth(t *testing.T) {
	t.Parallel()
	fn := callingFunctions()[0]

	var output []interface{}

	logger := log.Logger(log.LoggerFunc(func(keyvals ...interface{}) error {
		output = keyvals
		return nil
	}))

	stackValuer := log.Valuer(func() interface{} {
		for i, f := range callingFunctions() {
			if f == fn {
				return i
			}
		}
		t.Fatal("Test function not found in stack trace.")
		return nil
	})

	logger = log.With(logger, "stack", stackValuer)

	// Call through interface to get baseline.
	logger.Log("k", "v")
	want := output[1].(int)

	for len(output) < 10 {
		logger.Log("k", "v")
		if have := output[1]; have != want {
			t.Errorf("%d Withs: have %v, want %v", len(output)/2-1, have, want)
		}

		wrapped := log.With(logger)
		wrapped.Log("k", "v")
		if have := output[1]; have != want {
			t.Errorf("%d Withs: have %v, want %v", len(output)/2-1, have, want)
		}

		logger = log.With(logger, "k", "v")
	}
}

// callingFunctions returns the names of the functions on the call stack for the
// current goroutine with element 0 identifying the calling function.
func callingFunctions() []string {
	pcs := make([]uintptr, 10)
	n := runtime.Callers(2, pcs)
	if n == 0 {
		return nil
	}

	frames := runtime.CallersFrames(pcs[:n])
	funcs := make([]string, 0, n)

	for {
		frame, more := frames.Next()
		funcs = append(funcs, frame.Function)
		if !more {
			break
		}
	}

	return funcs
}

// Test that With returns a Logger safe for concurrent use. This test
// validates that the stored logging context does not get corrupted when
// multiple clients concurrently log additional keyvals.
//
// This test must be run with go test -cpu 2 (or more) to achieve its goal.
func TestWithConcurrent(t *testing.T) {
	// Create some buckets to count how many events each goroutine logs.
	const goroutines = 8
	counts := [goroutines]int{}

	// This logger extracts a goroutine id from the last value field and
	// increments the referenced bucket.
	logger := log.LoggerFunc(func(kv ...interface{}) error {
		goroutine := kv[len(kv)-1].(int)
		counts[goroutine]++
		return nil
	})

	// With must be careful about handling slices that can grow without
	// copying the underlying array, so give it a challenge.
	l := log.With(logger, make([]interface{}, 0, 2)...)

	// Start logging concurrently. Each goroutine logs its id so the logger
	// can bucket the event counts.
	var wg sync.WaitGroup
	wg.Add(goroutines)
	const n = 10000
	for i := 0; i < goroutines; i++ {
		go func(idx int) {
			defer wg.Done()
			for j := 0; j < n; j++ {
				l.Log("goroutineIdx", idx)
			}
		}(i)
	}
	wg.Wait()

	for bucket, have := range counts {
		if want := n; want != have {
			t.Errorf("bucket %d: want %d, have %d", bucket, want, have) // note Errorf
		}
	}
}

func TestLogCopiesValuers(t *testing.T) {
	t.Parallel()
	var output []interface{}
	logger := log.Logger(log.LoggerFunc(func(keyvals ...interface{}) error {
		output = keyvals
		return nil
	}))

	valuerCallCount := 0
	counterValuer := log.Valuer(func() interface{} {
		valuerCallCount++
		return valuerCallCount
	})
	lc := log.WithPrefix(logger, "a", counterValuer)
	lc = log.WithSuffix(lc, "z", counterValuer)

	if err := lc.Log(); err != nil {
		t.Fatal(err)
	}
	want := []interface{}{"a", 1, "z", 2}
	for i := 0; i < 4; i++ {
		if want, have := want[i], output[i]; want != have {
			t.Errorf("want output[%d] == %#v, have %#v", i, want, have)
		}
	}

	if err := lc.Log(); err != nil {
		t.Fatal(err)
	}
	want = []interface{}{"a", 3, "z", 4}
	for i := 0; i < 4; i++ {
		if want, have := want[i], output[i]; want != have {
			t.Errorf("want output[%d] == %#v, have %#v", i, want, have)
		}
	}
}

func BenchmarkDiscard(b *testing.B) {
	logger := log.NewNopLogger()
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		logger.Log("k", "v")
	}
}

func BenchmarkOneWith(b *testing.B) {
	logger := log.NewNopLogger()
	lc := log.With(logger, "k", "v")
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		lc.Log("k", "v")
	}
}

func BenchmarkTwoWith(b *testing.B) {
	logger := log.NewNopLogger()
	lc := log.With(logger, "k", "v")
	for i := 1; i < 2; i++ {
		lc = log.With(lc, "k", "v")
	}
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		lc.Log("k", "v")
	}
}

func BenchmarkTenWith(b *testing.B) {
	logger := log.NewNopLogger()
	lc := log.With(logger, "k", "v")
	for i := 1; i < 10; i++ {
		lc = log.With(lc, "k", "v")
	}
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		lc.Log("k", "v")
	}
}

func BenchmarkOneWithPrefix(b *testing.B) {
	logger := log.NewNopLogger()
	lc := log.WithPrefix(logger, "a", "first")
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		lc.Log("k", "v")
	}
}

func BenchmarkTenWithPrefix(b *testing.B) {
	logger := log.NewNopLogger()
	lc := log.WithPrefix(logger, "a", "first")
	for i := 1; i < 10; i++ {
		lc = log.WithPrefix(lc, "a", "first")
	}
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		lc.Log("k", "v")
	}
}

func BenchmarkOneWithSuffix(b *testing.B) {
	logger := log.NewNopLogger()
	lc := log.WithSuffix(logger, "z", "last")
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		lc.Log("k", "v")
	}
}

func BenchmarkTenWithSuffix(b *testing.B) {
	logger := log.NewNopLogger()
	lc := log.WithSuffix(logger, "z", "last")
	for i := 1; i < 10; i++ {
		lc = log.WithSuffix(lc, "z", "last")
	}
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		lc.Log("k", "v")
	}
}

func BenchmarkOneWithPrefixSuffix(b *testing.B) {
	logger := log.NewNopLogger()
	lc := log.WithSuffix(logger, "a", "first")
	lc = log.WithSuffix(lc, "z", "last")
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		lc.Log("k", "v")
	}
}

func BenchmarkTenWithPrefixSuffix(b *testing.B) {
	logger := log.NewNopLogger()
	lc := log.WithPrefix(logger, "a", "first")
	lc = log.WithSuffix(lc, "z", "last")
	for i := 1; i < 10; i++ {
		lc = log.WithPrefix(lc, "a", "first")
		lc = log.WithSuffix(lc, "z", "last")
	}
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		lc.Log("k", "v")
	}
}

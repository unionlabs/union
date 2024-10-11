package logsink_test

import (
	"bytes"
	"errors"
	"math"
	"reflect"
	"runtime"
	"slices"
	"testing"
	"time"

	"github.com/golang/glog/internal/logsink"
	"github.com/golang/glog/internal/stackdump"
	"github.com/google/go-cmp/cmp"
)

// A savingTextSink saves the data argument of the last Emit call made to it.
type savingTextSink struct{ data []byte }

func (savingTextSink) Enabled(*logsink.Meta) bool { return true }
func (s *savingTextSink) Emit(meta *logsink.Meta, data []byte) (n int, err error) {
	s.data = slices.Clone(data)
	return len(data), nil
}

func TestThreadPadding(t *testing.T) {
	originalSinks := logsink.StructuredSinks
	defer func() { logsink.StructuredSinks = originalSinks }()
	var sink savingTextSink
	logsink.TextSinks = []logsink.Text{&sink}

	_, file, line, _ := runtime.Caller(0)
	meta := &logsink.Meta{
		Time:     time.Now(),
		File:     file,
		Line:     line,
		Severity: logsink.Info,
	}

	const msg = "DOOMBAH!"

	for _, tc := range [...]struct {
		n    uint64
		want []byte
	}{
		// Integers that encode as fewer than 7 ASCII characters are padded, the
		// rest is not; see nDigits().
		{want: []byte("         "), n: 0}, // nDigits does not support 0 (I presume for speed reasons).
		{want: []byte("       1 "), n: 1},
		{want: []byte("  912389 "), n: 912389},
		{want: []byte(" 2147483648 "), n: math.MaxInt32 + 1},
		{want: []byte(" 9223372036854775806 "), n: math.MaxInt64 - 1},
		{want: []byte(" 9223372036854775808 "), n: math.MaxInt64 + 1},   // Test int64 overflow.
		{want: []byte(" 9223372036854775817 "), n: math.MaxInt64 + 10},  // Test int64 overflow.
		{want: []byte(" 18446744073709551614 "), n: math.MaxUint64 - 1}, // Test int64 overflow.
	} {
		meta.Thread = int64(tc.n)
		logsink.Printf(meta, "%v", msg)
		t.Logf(`logsink.Printf(%+v, "%%v", %q)`, meta, msg)

		// Check if the needle is present exactly.
		if !bytes.Contains(sink.data, tc.want) {
			t.Errorf("needle = '%s' not found in %s", tc.want, sink.data)
		}
	}
}

func TestFatalMessage(t *testing.T) {
	const msg = "DOOOOOOM!"

	_, file, line, _ := runtime.Caller(0)
	meta := &logsink.Meta{
		Time:     time.Now(),
		File:     file,
		Line:     line,
		Severity: logsink.Fatal,
	}

	logsink.Printf(meta, "%v", msg)
	t.Logf(`logsink.Printf(%+v, "%%v", %q)`, meta, msg)

	gotMeta, gotMsg, ok := logsink.FatalMessage()
	if !ok || !reflect.DeepEqual(gotMeta, meta) || !bytes.Contains(gotMsg, []byte(msg)) {
		t.Errorf("logsink.FatalMessage() = %+v, %q, %v", gotMeta, gotMsg, ok)
	}
}

func TestStructuredSink(t *testing.T) {
	// Reset logsink.StructuredSinks at the end of the test.
	// Each test case will clear it and insert its own test sink.
	originalSinks := logsink.StructuredSinks
	defer func() {
		logsink.StructuredSinks = originalSinks
	}()

	testStacktrace := stackdump.Caller(0)

	for _, test := range []struct {
		name    string
		format  string
		args    []any
		meta    logsink.Meta
		wantErr bool
		sinks   []testStructuredSinkAndWants
	}{
		{
			name:   "sink is called with expected format and args",
			format: "test %d",
			args:   []any{1},
			sinks: []testStructuredSinkAndWants{
				{
					sink: &fakeStructuredSink{},
				},
			},
		},
		{
			name: "sink is called with expected meta",
			meta: logsink.Meta{
				Severity: logsink.Info,
				File:     "base/go/logsink_test.go",
				Line:     1,
				Time:     time.Unix(1545321163, 0),
				Thread:   1,
			},
			sinks: []testStructuredSinkAndWants{
				{
					sink: &fakeStructuredSink{},
				},
			},
		},
		{
			name: "sink is called with expected meta (2)",
			meta: logsink.Meta{
				Severity: logsink.Error,
				File:     "foo.go",
				Line:     1337,
				Time:     time.Unix(0, 0),
				Thread:   123,
			},
			sinks: []testStructuredSinkAndWants{
				{
					sink: &fakeStructuredSink{},
				},
			},
		},
		{
			name:   "sink returns error",
			format: "test",
			meta: logsink.Meta{
				Severity: logsink.Info,
				File:     "base/go/logsink_test.go",
				Line:     1,
				Time:     time.Unix(1545321163, 0),
				Thread:   1,
			},
			wantErr: true,
			sinks: []testStructuredSinkAndWants{
				{
					sink: &fakeStructuredSink{
						err: errors.New("err"),
					},
				},
			},
		},
		{
			name: "sink is StackWanter and WantStack() returns true",
			sinks: []testStructuredSinkAndWants{
				{
					sink: &fakeStructuredSinkThatWantsStack{
						wantStack: true,
					},
					wantStack: true,
				},
			},
		},
		{
			name: "sink is StackWanter and WantStack() returns false",
			sinks: []testStructuredSinkAndWants{
				{
					sink: &fakeStructuredSinkThatWantsStack{
						wantStack: false,
					},
					wantStack: false,
				},
			},
		},
		{
			name:   "use stacktrace from args if available",
			format: "test\n%s",
			args:   []any{testStacktrace},
			sinks: []testStructuredSinkAndWants{
				{
					sink: &fakeStructuredSinkThatWantsStack{
						wantStack: true,
					},
					wantStack:      true,
					wantStackEqual: &testStacktrace,
				},
			},
		},
		{
			name:   "respect StackWanter contract",
			format: "test\n%s",
			args:   []any{testStacktrace},
			sinks: []testStructuredSinkAndWants{
				{
					sink: &fakeStructuredSinkThatWantsStack{
						wantStack: true,
					},
					wantStack:      true,
					wantStackEqual: &testStacktrace,
				},
				{
					sink: &fakeStructuredSink{},
				},
			},
		},
		{
			name:   "respect StackWanter contract for multiple sinks",
			format: "test\n%s",
			args:   []any{testStacktrace},
			sinks: []testStructuredSinkAndWants{
				{
					sink:           &fakeStructuredSinkThatWantsStack{wantStack: true},
					wantStack:      true,
					wantStackEqual: &testStacktrace,
				},
				{
					sink:      &fakeStructuredSinkThatWantsStack{wantStack: false},
					wantStack: false,
				},
				{
					sink:           &fakeStructuredSinkThatWantsStack{wantStack: true},
					wantStack:      true,
					wantStackEqual: &testStacktrace,
				},
				{
					sink:      &fakeStructuredSink{},
					wantStack: false,
				},
				{
					sink:           &fakeStructuredSinkThatWantsStack{wantStack: true},
					wantStack:      true,
					wantStackEqual: &testStacktrace,
				},
			},
		},
	} {
		t.Run(test.name, func(t *testing.T) {
			testStructuredSinks := make([]logsink.Structured, len(test.sinks))
			for i, sink := range test.sinks {
				testStructuredSinks[i] = sink.sink
			}
			// Register test logsinks
			logsink.StructuredSinks = testStructuredSinks

			// logsink.Printf() should call Printf() on all registered logsinks.
			// Copy test.meta to prevent changes by the code under test.
			meta := test.meta
			_, err := logsink.Printf(&meta, test.format, test.args...)
			if gotErr := err != nil; gotErr != test.wantErr {
				t.Fatalf("logsink.Printf() = (_, %v), want err? %t", err, test.wantErr)
			}

			// Test the behavior for each registered StructuredSink.
			for _, testStructuredSinkAndWants := range test.sinks {
				// Check that the test logsink was called with expected arguments.
				if got, want := testStructuredSinkAndWants.sink.Calls(), 1; got != want {
					t.Fatalf("sink.calls = %d, want %d", got, want)
				}

				// Check that Meta was passed through to the logsink.
				gotMeta := testStructuredSinkAndWants.sink.GotMeta()
				// Ignore the Stack and Depth fields; these will be checked further down.
				cmpIgnoreSomeFields := cmp.FilterPath(func(p cmp.Path) bool { return p.String() == "Stack" || p.String() == "Depth" }, cmp.Ignore())
				if diff := cmp.Diff(&test.meta, gotMeta, cmpIgnoreSomeFields); diff != "" {
					t.Errorf("sink.meta diff -want +got:\n%s", diff)
				}

				// The contract is:
				//  - If WantStack is true, a Stack is present.
				//  - If WantStack is false, a Stack may be present.
				if testStructuredSinkAndWants.wantStack && gotMeta.Stack == nil {
					t.Errorf("sink.meta.Stack = %v, but WantStack = %t", gotMeta.Stack, testStructuredSinkAndWants.wantStack)
				} else if testStructuredSinkAndWants.wantStackEqual != nil {
					// We have a stack, but is it the right one?
					if diff := cmp.Diff(testStructuredSinkAndWants.wantStackEqual, gotMeta.Stack); diff != "" {
						t.Errorf("sink.meta.Stack diff -want +got:\n%s", diff)
					}
				}

				// Depth should be 1, since test.meta.Depth is always 0 and there's a single
				// function call, logsink.Printf(), between here and the logsink.
				if got, want := gotMeta.Depth, 1; got != want {
					t.Errorf("sink.meta.Depth = %d, want %d", got, want)
				}

				if got, want := testStructuredSinkAndWants.sink.GotFormat(), test.format; got != want {
					t.Errorf("sink.format = %q, want %q", got, want)
				}

				if diff := cmp.Diff(test.args, testStructuredSinkAndWants.sink.GotArgs()); diff != "" {
					t.Errorf("sink.args diff -want +got:\n%s", diff)
				}
			}
		})
	}
}

func BenchmarkStructuredSink(b *testing.B) {
	// Reset logsink.StructuredSinks at the end of the benchmark.
	// Each benchmark case will clear it and insert its own test sink.
	originalSinks := logsink.StructuredSinks
	defer func() {
		logsink.StructuredSinks = originalSinks
	}()

	noop := noopStructuredSink{}
	noopWS := noopStructuredSinkWantStack{}
	stringWS := stringStructuredSinkWantStack{}

	_, file, line, _ := runtime.Caller(0)
	stack := stackdump.Caller(0)
	genMeta := func(dump *stackdump.Stack) *logsink.Meta {
		return &logsink.Meta{
			Time:     time.Now(),
			File:     file,
			Line:     line,
			Severity: logsink.Warning,
			Thread:   1240,
			Stack:    dump,
		}
	}

	for _, test := range []struct {
		name  string
		sinks []logsink.Structured
		meta  *logsink.Meta
	}{
		{name: "meta_nostack_01_sinks_00_want_stack_pconly", meta: genMeta(nil), sinks: []logsink.Structured{noop}},
		{name: "meta___stack_01_sinks_01_want_stack_pconly", meta: genMeta(&stack), sinks: []logsink.Structured{noopWS}},
		{name: "meta_nostack_01_sinks_01_want_stack_pconly", meta: genMeta(nil), sinks: []logsink.Structured{noopWS}},
		{name: "meta_nostack_01_sinks_01_want_stack_string", meta: genMeta(nil), sinks: []logsink.Structured{stringWS}},
		{name: "meta_nostack_02_sinks_01_want_stack_pconly", meta: genMeta(nil), sinks: []logsink.Structured{noopWS, noop}},
		{name: "meta_nostack_02_sinks_02_want_stack_string", meta: genMeta(nil), sinks: []logsink.Structured{stringWS, stringWS}},
		{name: "meta_nostack_10_sinks_00_want_stack_pconly", meta: genMeta(nil), sinks: []logsink.Structured{noop, noop, noop, noop, noop, noop, noop, noop, noop, noop}},
		{name: "meta_nostack_10_sinks_05_want_stack_pconly", meta: genMeta(nil), sinks: []logsink.Structured{noop, noopWS, noop, noop, noopWS, noop, noopWS, noopWS, noopWS, noop}},
		{name: "meta_nostack_10_sinks_05_want_stack_string", meta: genMeta(nil), sinks: []logsink.Structured{noop, stringWS, noop, noop, stringWS, noop, stringWS, stringWS, stringWS, noop}},
		{name: "meta___stack_10_sinks_05_want_stack_pconly", meta: genMeta(&stack), sinks: []logsink.Structured{noop, noopWS, noop, noop, noopWS, noop, noopWS, noopWS, noopWS, noop}},
		{name: "meta___stack_10_sinks_05_want_stack_string", meta: genMeta(&stack), sinks: []logsink.Structured{noop, stringWS, noop, noop, stringWS, noop, stringWS, stringWS, stringWS, noop}},
	} {
		b.Run(test.name, func(b *testing.B) {
			logsink.StructuredSinks = test.sinks
			savedStack := test.meta.Stack

			args := []any{1} // Pre-allocate args slice to avoid allocation in benchmark loop.

			b.ResetTimer()
			for i := 0; i < b.N; i++ {
				_, err := logsink.Printf(test.meta, "test %d", args...)
				if err != nil {
					b.Fatalf("logsink.Printf(): didn't expect any error while benchmarking, got %v", err)
				}
				// logsink.Printf modifies Meta.Depth, which is used during stack
				// collection. If we don't reset it, stacks quickly become empty, making
				// the benchmark useless.
				test.meta.Depth = 0
				// There is a possible optimization where logsink.Printf will avoid
				// allocating a new meta and modify it in-place if it needs a stack.
				// This would throw off benchmarks as subsequent invocations would
				// re-use this stack. Since we know this memoization/modification only
				// happens with stacks, reset it manually to avoid skewing allocation
				// numbers.
				test.meta.Stack = savedStack
			}
		})
	}
}

// testStructuredSinkAndWants contains a StructuredSink under test
// and its wanted values. The struct is created to help with testing
// multiple StructuredSinks for Printf().
type testStructuredSinkAndWants struct {
	// The sink under test.
	sink testStructuredSink
	// Whether this sink should want stack in its meta.
	// Only set when the sink is fakeStructuredSinkThatWantsStack.
	wantStack bool
	// If this sink wants stack, the expected stack.
	// Only set when the sink is fakeStructuredSinkThatWantsStack and returns true for WantStack().
	wantStackEqual *stackdump.Stack
}

type testStructuredSink interface {
	logsink.Structured

	GotMeta() *logsink.Meta
	GotFormat() string
	GotArgs() []any
	Calls() int
}

type fakeStructuredSink struct {
	// err is returned by Printf().
	err error
	// gotMeta is the Meta passed to the last Printf() call.
	gotMeta *logsink.Meta
	// gotFormat is the format string passed to the last Printf() call.
	gotFormat string
	// gotArgs are the arguments passed to the last Printf() call.
	gotArgs []any
	// calls is a counter of the number of times Printf() has been called.
	calls int
}

func (s *fakeStructuredSink) GotMeta() *logsink.Meta {
	return s.gotMeta
}

func (s *fakeStructuredSink) GotFormat() string {
	return s.gotFormat
}

func (s *fakeStructuredSink) GotArgs() []any {
	return s.gotArgs
}

func (s *fakeStructuredSink) Calls() int {
	return s.calls
}

func (s *fakeStructuredSink) Printf(meta *logsink.Meta, format string, a ...any) (n int, err error) {
	s.gotMeta = meta
	s.gotFormat = format
	s.gotArgs = a
	s.calls++
	return 0, s.err
}

type fakeStructuredSinkThatWantsStack struct {
	fakeStructuredSink
	// wantStack controls what the WantStack() method returns.
	wantStack bool
}

func (s *fakeStructuredSinkThatWantsStack) WantStack(meta *logsink.Meta) bool {
	return s.wantStack
}

type noopStructuredSink struct{}

func (s noopStructuredSink) Printf(meta *logsink.Meta, format string, a ...any) (n int, err error) {
	return 0, nil
}

type noopStructuredSinkWantStack struct{}

func (s noopStructuredSinkWantStack) WantStack(_ *logsink.Meta) bool { return true }
func (s noopStructuredSinkWantStack) Printf(meta *logsink.Meta, format string, a ...any) (n int, err error) {
	return 0, nil
}

type stringStructuredSinkWantStack struct{}

func (s stringStructuredSinkWantStack) WantStack(_ *logsink.Meta) bool { return true }
func (s stringStructuredSinkWantStack) Printf(meta *logsink.Meta, format string, a ...any) (n int, err error) {
	return len(meta.Stack.String()), nil
}

// TestStructuredTextWrapper tests StructuredTextWrapper.Printf().
// It validates the input received by each Text sink in StructuredTextWrapper.TextSinks
// by comparing it to the input received by a Text sink in logsink.TextSinks. We assume
// that logsink.TextSinks receives a correct input (that fact is already tested in log.test.go)
func TestStructuredTextWrapper(t *testing.T) {
	// Reset logsink.TextSinks at the end of the test.
	originalTextSinks := logsink.TextSinks
	defer func() {
		logsink.TextSinks = originalTextSinks
	}()

	// The input received by the `reference` sink will be used to validate the input received by
	// each sink in StructuredTextWrapper.TextSinks.
	reference := fakeTextSink{enabled: true}
	logsink.TextSinks = []logsink.Text{&reference}

	meta := logsink.Meta{
		Severity: logsink.Info,
		File:     "base/go/logsink_test.go",
		Line:     1,
		Time:     time.Unix(1545321163, 0),
		Thread:   1,
	}
	format := "test %d"
	args := []any{1}

	for _, test := range []struct {
		name          string
		sinks         []fakeTextSink
		wantByteCount int
		wantErr       bool
	}{
		{
			name:  "no sinks",
			sinks: []fakeTextSink{},
		},
		{
			name: "single sink",
			sinks: []fakeTextSink{
				fakeTextSink{enabled: true, byteCount: 300},
			},
			wantByteCount: 300,
		},
		{
			name: "multiple sinks",
			sinks: []fakeTextSink{
				fakeTextSink{enabled: true, byteCount: 100},
				fakeTextSink{enabled: true, byteCount: 300},
				fakeTextSink{enabled: true, byteCount: 200},
			},
			wantByteCount: 300,
		},
		{
			name: "some sinks disabled",
			sinks: []fakeTextSink{
				fakeTextSink{enabled: true, byteCount: 100},
				fakeTextSink{enabled: true, byteCount: 200},
				fakeTextSink{},
				fakeTextSink{},
			},
			wantByteCount: 200,
		},
		{
			name: "all sinks disabled",
			sinks: []fakeTextSink{
				fakeTextSink{},
				fakeTextSink{},
				fakeTextSink{},
			},
		},
		{
			name: "error",
			sinks: []fakeTextSink{
				fakeTextSink{enabled: true, byteCount: 100},
				fakeTextSink{enabled: true, err: errors.New("err")},
				fakeTextSink{enabled: true, byteCount: 200},
			},
			wantErr: true,
		},
	} {
		t.Run(test.name, func(t *testing.T) {
			wrapper := logsink.StructuredTextWrapper{}
			for i := range test.sinks {
				wrapper.TextSinks = append(wrapper.TextSinks, &test.sinks[i])
			}

			// Writing to reference sink.
			// Copy meta to prevent changes by the code under test.
			m := meta
			if _, err := logsink.Printf(&m, format, args); err != nil {
				t.Fatalf("failed to write to reference sink: %v", err)
			}

			// Writing to StructuredTextWrapper.
			// Copy meta to prevent changes by the code under test.
			m = meta
			n, err := wrapper.Printf(&m, format, args)

			if gotErr := err != nil; gotErr != test.wantErr {
				t.Fatalf("StructuredTextWrapper.Printf() returned err=%v, want err? %t", err, test.wantErr)
			}

			// If an error is expected, we are done.
			if err != nil {
				return
			}

			if n != test.wantByteCount {
				t.Fatalf("StructuredTextWrapper.Printf() returned n=%v, want %v", n, test.wantByteCount)
			}

			for i, sink := range test.sinks {
				if sink.enabled {
					if got, want := sink.calls, 1; got != want {
						t.Fatalf("sinks[%v].calls = %d, want %d", i, got, want)
					}

					if diff := cmp.Diff(&meta, sink.gotMeta); diff != "" {
						t.Errorf("sinks[%v].meta diff -want +got:\n%s", i, diff)
					}

					if got, want := sink.gotBytes, reference.gotBytes; bytes.Compare(got, want) != 0 {
						t.Errorf("sinks[%v].bytes = %s, want %s", i, got, want)
					}
				} else {
					if got, want := sink.calls, 0; got != want {
						t.Fatalf("sinks[%v].calls = %d, want %d", i, got, want)
					}
				}
			}
		})
	}
}

type fakeTextSink struct {
	// enabled is returned by Enabled().
	enabled bool
	// byteCount is returned by Emit().
	byteCount int
	// err is returned by Emit().
	err error
	// gotMeta is the Meta passed to the last Emit() call.
	gotMeta *logsink.Meta
	// gotBytes is the byte slice passed to the last Emit() call.
	gotBytes []byte
	// calls is a counter of the number of times Emit() has been called.
	calls int
}

func (s *fakeTextSink) Enabled(meta *logsink.Meta) bool {
	return s.enabled
}

func (s *fakeTextSink) Emit(meta *logsink.Meta, bytes []byte) (n int, err error) {
	s.gotMeta = meta
	s.gotBytes = bytes
	s.calls++
	return s.byteCount, s.err
}

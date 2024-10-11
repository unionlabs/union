package term_test

import (
	"bytes"
	"io"
	"io/ioutil"
	"strconv"
	"sync"
	"testing"

	"github.com/go-kit/log"
	"github.com/go-kit/log/term"
)

func TestColorLogger(t *testing.T) {
	var buf bytes.Buffer
	logger := newColorLogger(&buf)

	if err := logger.Log("hello", "world"); err != nil {
		t.Fatal(err)
	}
	if want, have := "hello=world\n", buf.String(); want != have {
		t.Errorf("\nwant %#v\nhave %#v", want, have)
	}

	buf.Reset()
	if err := logger.Log("a", 1); err != nil {
		t.Fatal(err)
	}
	if want, have := "\x1b[32;1m\x1b[47;1ma=1\n\x1b[39;49;22m", buf.String(); want != have {
		t.Errorf("\nwant %#v\nhave %#v", want, have)
	}
}

func newColorLogger(w io.Writer) log.Logger {
	return term.NewColorLogger(w, log.NewLogfmtLogger,
		func(keyvals ...interface{}) term.FgBgColor {
			if keyvals[0] == "a" {
				return term.FgBgColor{Fg: term.Green, Bg: term.White}
			}
			return term.FgBgColor{}
		})
}

func BenchmarkColorLoggerSimple(b *testing.B) {
	benchmarkRunner(b, newColorLogger(ioutil.Discard), baseMessage)
}

func BenchmarkColorLoggerContextual(b *testing.B) {
	benchmarkRunner(b, newColorLogger(ioutil.Discard), withMessage)
}

func TestColorLoggerConcurrency(t *testing.T) {
	testConcurrency(t, newColorLogger(ioutil.Discard))
}

// copied from log/benchmark_test.go
func benchmarkRunner(b *testing.B, logger log.Logger, f func(log.Logger)) {
	lc := log.With(logger, "common_key", "common_value")
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		f(lc)
	}
}

var (
	baseMessage = func(logger log.Logger) { logger.Log("foo_key", "foo_value") }
	withMessage = func(logger log.Logger) { log.With(logger, "a", "b").Log("c", "d") }
)

// copied from log/concurrency_test.go
func testConcurrency(t *testing.T, logger log.Logger) {
	for _, n := range []int{10, 100, 500} {
		wg := sync.WaitGroup{}
		wg.Add(n)
		for i := 0; i < n; i++ {
			go func() { spam(logger); wg.Done() }()
		}
		wg.Wait()
	}
}

func spam(logger log.Logger) {
	for i := 0; i < 100; i++ {
		logger.Log("a", strconv.FormatInt(int64(i), 10))
	}
}

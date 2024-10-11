package log

import (
	"bytes"
	"os"
	"strings"
	"testing"
)

// TestLoggingWithTrace checks that if BackTraceAt is set, then the
// gloghandler is capable of spitting out a stacktrace
func TestLoggingWithTrace(t *testing.T) {
	defer stackEnabled.Store(stackEnabled.Load())
	out := new(bytes.Buffer)
	logger := New()
	{
		glog := NewGlogHandler(StreamHandler(out, TerminalFormat(false)))
		glog.Verbosity(LvlTrace)
		if err := glog.BacktraceAt("logger_test.go:24"); err != nil {
			t.Fatal(err)
		}
		logger.SetHandler(glog)
	}
	logger.Trace("a message", "foo", "bar") // Will be bumped to INFO
	have := out.String()
	if !strings.HasPrefix(have, "INFO") {
		t.Fatalf("backtraceat should bump level to info: %s", have)
	}
	// The timestamp is locale-dependent, so we want to trim that off
	// "INFO [01-01|00:00:00.000] a messag ..." -> "a messag..."
	have = strings.Split(have, "]")[1]
	wantPrefix := " a message\n\ngoroutine"
	if !strings.HasPrefix(have, wantPrefix) {
		t.Errorf("\nhave: %q\nwant: %q\n", have, wantPrefix)
	}
}

// TestLoggingWithVmodule checks that vmodule works.
func TestLoggingWithVmodule(t *testing.T) {
	defer stackEnabled.Store(stackEnabled.Load())
	out := new(bytes.Buffer)
	logger := New()
	{
		glog := NewGlogHandler(StreamHandler(out, TerminalFormat(false)))
		glog.Verbosity(LvlCrit)
		logger.SetHandler(glog)
		logger.Warn("This should not be seen", "ignored", "true")
		glog.Vmodule("logger_test.go=5")
	}
	logger.Trace("a message", "foo", "bar")
	have := out.String()
	// The timestamp is locale-dependent, so we want to trim that off
	// "INFO [01-01|00:00:00.000] a messag ..." -> "a messag..."
	have = strings.Split(have, "]")[1]
	want := " a message                                foo=bar\n"
	if have != want {
		t.Errorf("\nhave: %q\nwant: %q\n", have, want)
	}
}

func BenchmarkTraceLogging(b *testing.B) {
	Root().SetHandler(LvlFilterHandler(LvlInfo, StreamHandler(os.Stderr, TerminalFormat(true))))
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		Trace("a message", "v", i)
	}
}

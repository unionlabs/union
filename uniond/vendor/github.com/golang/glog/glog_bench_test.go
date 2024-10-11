package glog

import (
	"flag"
	"io/ioutil"
	"runtime"
	"sync"
	"sync/atomic"
	"testing"
	"time"
)

// discarder is a flushSyncWriter that discards all data.
// Sync sleeps for 10ms to simulate a disk seek.
type discarder struct {
}

func (d *discarder) Write(data []byte) (int, error) {
	return len(data), nil
}

func (d *discarder) Flush() error {
	return nil
}

func (d *discarder) Sync() error {
	time.Sleep(10 * time.Millisecond)
	return nil
}

func (d *discarder) filenames() []string {
	return nil
}

// newDiscard sets the log writers to all new byte buffers and returns the old array.
func (s *fileSink) newDiscarders() severityWriters {
	return s.swap(severityWriters{new(discarder), new(discarder), new(discarder), new(discarder)})
}

func discardStderr() func() {
	se := sinks.stderr.w
	sinks.stderr.w = ioutil.Discard
	return func() { sinks.stderr.w = se }
}

const message = "benchmark log message"

func benchmarkLog(b *testing.B, log func(...any)) {
	defer sinks.file.swap(sinks.file.newDiscarders())
	defer discardStderr()()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		log(message)
	}
	b.StopTimer()
}

func benchmarkLogConcurrent(b *testing.B, log func(...any)) {
	defer sinks.file.swap(sinks.file.newDiscarders())
	defer discardStderr()()
	b.ResetTimer()
	concurrency := runtime.GOMAXPROCS(0)
	var wg sync.WaitGroup
	wg.Add(concurrency)
	for i := 0; i < concurrency; i++ {
		go func() {
			for i := 0; i < b.N; i++ {
				log(message)
			}
			wg.Done()
		}()
	}
	wg.Wait()
	b.StopTimer()
}

func BenchmarkInfo(b *testing.B) {
	benchmarkLog(b, Info)
}

func BenchmarkInfoConcurrent(b *testing.B) {
	benchmarkLogConcurrent(b, Info)
}

func BenchmarkWarning(b *testing.B) {
	benchmarkLog(b, Warning)
}

func BenchmarkWarningConcurrent(b *testing.B) {
	benchmarkLogConcurrent(b, Warning)
}

func BenchmarkError(b *testing.B) {
	benchmarkLog(b, Error)
}

func BenchmarkErrorConcurrent(b *testing.B) {
	benchmarkLogConcurrent(b, Error)
}

func mixer() func(...any) {
	var i int64
	return func(args ...any) {
		n := atomic.AddInt64(&i, 1)
		switch {
		case n%10000 == 0:
			Error(args...)
		case n%1000 == 0:
			Warning(args...)
		default:
			Info(args...)
		}
	}
}

func BenchmarkMix(b *testing.B) {
	benchmarkLog(b, mixer())
}

func BenchmarkMixConcurrent(b *testing.B) {
	benchmarkLogConcurrent(b, mixer())
}

func BenchmarkVLogDisabled(b *testing.B) {
	benchmarkLog(b, vlog)
}

func BenchmarkVLogDisabledConcurrent(b *testing.B) {
	benchmarkLogConcurrent(b, vlog)
}

func BenchmarkVLogModuleFlagSet(b *testing.B) {
	defer withVmodule("nonexistant=5")()
	benchmarkLog(b, vlog)
}

func BenchmarkVLogModuleFlagSetConcurrent(b *testing.B) {
	defer withVmodule("nonexistant=5")()
	benchmarkLogConcurrent(b, vlog)
}

func BenchmarkVLogEnabled(b *testing.B) {
	defer withVmodule("glog_bench_test=5")()
	if got := bool(V(3)); got != true {
		b.Fatalf("V(3) == %v, want %v", got, true)
	}
	benchmarkLog(b, vlog)
}

func BenchmarkVLogEnabledConcurrent(b *testing.B) {
	defer withVmodule("glog_bench_test=5")()
	benchmarkLogConcurrent(b, vlog)
}

func vlog(args ...any) {
	V(3).Info(args)
}

func withVmodule(val string) func() {
	if err := flag.Set("vmodule", val); err != nil {
		panic(err)
	}
	return func() { flag.Set("vmodule", "") }
}

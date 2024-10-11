// Copyright 2019+ Klaus Post. All rights reserved.
// License information can be found in the LICENSE file.

package zstd

import (
	"flag"
	"fmt"
	"io"
	"log"
	"os"
	"runtime"
	"runtime/pprof"
	"strings"
	"testing"
	"time"
)

var isRaceTest bool

// Fuzzing tweaks:
var fuzzStartF = flag.Int("fuzz-start", int(SpeedFastest), "Start fuzzing at this level")
var fuzzEndF = flag.Int("fuzz-end", int(SpeedBestCompression), "End fuzzing at this level (inclusive)")
var fuzzMaxF = flag.Int("fuzz-max", 1<<20, "Maximum input size")

func TestMain(m *testing.M) {
	flag.Parse()
	ec := m.Run()
	if ec == 0 && runtime.NumGoroutine() > 2 {
		n := 0
		for n < 15 {
			n++
			time.Sleep(time.Second)
			if runtime.NumGoroutine() == 2 {
				os.Exit(0)
			}
		}
		fmt.Println("goroutines:", runtime.NumGoroutine())
		pprof.Lookup("goroutine").WriteTo(os.Stderr, 1)
		os.Exit(1)
	}
	os.Exit(ec)
}

func TestMatchLen(t *testing.T) {
	a := make([]byte, 130)
	for i := range a {
		a[i] = byte(i)
	}
	b := append([]byte{}, a...)

	check := func(x, y []byte, l int) {
		if m := matchLen(x, y); m != l {
			t.Error("expected", l, "got", m)
		}
	}

	for l := range a {
		a[l] = ^a[l]
		check(a, b, l)
		check(a[:l], b, l)
		a[l] = ^a[l]
	}
}

func TestWriterMemUsage(t *testing.T) {
	testMem := func(t *testing.T, fn func()) {
		var before, after runtime.MemStats
		var w io.Writer
		if false {
			f, err := os.Create(strings.ReplaceAll(fmt.Sprintf("%s.pprof", t.Name()), "/", "_"))
			if err != nil {
				log.Fatal(err)
			}
			defer f.Close()
			w = f
			t.Logf("opened memory profile %s", t.Name())
		}
		runtime.GC()
		runtime.ReadMemStats(&before)
		fn()
		runtime.GC()
		runtime.ReadMemStats(&after)
		if w != nil {
			pprof.WriteHeapProfile(w)
		}
		t.Log("wrote profile")
		t.Logf("%s: Memory Used: %dMB, %d allocs", t.Name(), (after.HeapInuse-before.HeapInuse)/1024/1024, after.HeapObjects-before.HeapObjects)
	}
	data := make([]byte, 10<<20)

	t.Run("enc-all-lower", func(t *testing.T) {
		for level := SpeedFastest; level <= SpeedBestCompression; level++ {
			t.Run(fmt.Sprint("level-", level), func(t *testing.T) {
				var zr *Encoder
				var err error
				dst := make([]byte, 0, len(data)*2)
				testMem(t, func() {
					zr, err = NewWriter(io.Discard, WithEncoderConcurrency(32), WithEncoderLevel(level), WithLowerEncoderMem(false), WithWindowSize(1<<20))
					if err != nil {
						t.Fatal(err)
					}
					for i := 0; i < 100; i++ {
						_ = zr.EncodeAll(data, dst[:0])
					}
				})
				zr.Close()
			})
		}
	})
}

var data = []byte{1, 2, 3}

func newZstdWriter() (*Encoder, error) {
	return NewWriter(
		io.Discard,
		WithEncoderLevel(SpeedBetterCompression),
		WithEncoderConcurrency(16), // we implicitly get this concurrency level if we run on 16 core CPU
		WithLowerEncoderMem(false),
		WithWindowSize(1<<20),
	)
}

func BenchmarkMem(b *testing.B) {
	b.Run("flush", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			w, err := newZstdWriter()
			if err != nil {
				b.Fatal(err)
			}

			for j := 0; j < 16; j++ {
				w.Reset(io.Discard)

				if _, err := w.Write(data); err != nil {
					b.Fatal(err)
				}

				if err := w.Flush(); err != nil {
					b.Fatal(err)
				}

				if err := w.Close(); err != nil {
					b.Fatal(err)
				}
			}
		}
	})
	b.Run("no-flush", func(b *testing.B) {
		// Will use encodeAll for block.
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			w, err := newZstdWriter()
			if err != nil {
				b.Fatal(err)
			}

			for j := 0; j < 16; j++ {
				w.Reset(io.Discard)

				if _, err := w.Write(data); err != nil {
					b.Fatal(err)
				}

				if err := w.Close(); err != nil {
					b.Fatal(err)
				}
			}
		}
	})
}

//go:build go1.18
// +build go1.18

package flate

import (
	"bytes"
	"flag"
	"io"
	"os"
	"strconv"
	"testing"

	"github.com/klauspost/compress/internal/fuzz"
)

// Fuzzing tweaks:
var fuzzStartF = flag.Int("start", HuffmanOnly, "Start fuzzing at this level")
var fuzzEndF = flag.Int("end", BestCompression, "End fuzzing at this level (inclusive)")
var fuzzMaxF = flag.Int("max", 1<<20, "Maximum input size")
var fuzzSLF = flag.Bool("sl", true, "Include stateless encodes")
var fuzzWindow = flag.Bool("windows", true, "Include windowed encodes")

func TestMain(m *testing.M) {
	flag.Parse()
	os.Exit(m.Run())
}

func FuzzEncoding(f *testing.F) {
	fuzz.AddFromZip(f, "testdata/regression.zip", fuzz.TypeRaw, false)
	fuzz.AddFromZip(f, "testdata/fuzz/encode-raw-corpus.zip", fuzz.TypeRaw, testing.Short())
	fuzz.AddFromZip(f, "testdata/fuzz/FuzzEncoding.zip", fuzz.TypeGoFuzz, testing.Short())

	startFuzz := *fuzzStartF
	endFuzz := *fuzzEndF
	maxSize := *fuzzMaxF
	stateless := *fuzzSLF
	fuzzWindow := *fuzzWindow

	decoder := NewReader(nil)
	buf := new(bytes.Buffer)
	encs := make([]*Writer, endFuzz-startFuzz+1)
	for i := range encs {
		var err error
		encs[i], err = NewWriter(nil, i+startFuzz)
		if err != nil {
			f.Fatal(err.Error())
		}
	}

	f.Fuzz(func(t *testing.T, data []byte) {
		if len(data) > maxSize {
			return
		}
		for level := startFuzz; level <= endFuzz; level++ {
			msg := "level " + strconv.Itoa(level) + ":"
			buf.Reset()
			fw := encs[level-startFuzz]
			fw.Reset(buf)
			n, err := fw.Write(data)
			if n != len(data) {
				t.Fatal(msg + "short write")
			}
			if err != nil {
				t.Fatal(msg + err.Error())
			}
			err = fw.Close()
			if err != nil {
				t.Fatal(msg + err.Error())
			}
			decoder.(Resetter).Reset(buf, nil)
			data2, err := io.ReadAll(decoder)
			if err != nil {
				t.Fatal(msg + err.Error())
			}
			if !bytes.Equal(data, data2) {
				t.Fatal(msg + "not equal")
			}
			// Do it again...
			msg = "level " + strconv.Itoa(level) + " (reset):"
			buf.Reset()
			fw.Reset(buf)
			n, err = fw.Write(data)
			if n != len(data) {
				t.Fatal(msg + "short write")
			}
			if err != nil {
				t.Fatal(msg + err.Error())
			}
			err = fw.Close()
			if err != nil {
				t.Fatal(msg + err.Error())
			}
			decoder.(Resetter).Reset(buf, nil)
			data2, err = io.ReadAll(decoder)
			if err != nil {
				t.Fatal(msg + err.Error())
			}
			if !bytes.Equal(data, data2) {
				t.Fatal(msg + "not equal")
			}
		}
		if stateless {
			// Split into two and use history...
			msg := "stateless:"
			buf.Reset()
			err := StatelessDeflate(buf, data[:len(data)/2], false, nil)
			if err != nil {
				t.Error(err)
			}

			// Use top half as dictionary...
			dict := data[:len(data)/2]
			err = StatelessDeflate(buf, data[len(data)/2:], true, dict)
			if err != nil {
				t.Error(err)
			}

			decoder.(Resetter).Reset(buf, nil)
			data2, err := io.ReadAll(decoder)
			if err != nil {
				t.Error(err)
			}
			if !bytes.Equal(data, data2) {
				//fmt.Printf("want:%x\ngot: %x\n", data1, data2)
				t.Error(msg + "not equal")
			}
		}
		if fuzzWindow {
			msg := "windowed:"
			buf.Reset()
			fw, err := NewWriterWindow(buf, 1000)
			if err != nil {
				t.Fatal(msg + err.Error())
			}
			fw.Reset(buf)
			n, err := fw.Write(data)
			if n != len(data) {
				t.Fatal(msg + "short write")
			}
			if err != nil {
				t.Fatal(msg + err.Error())
			}
			err = fw.Close()
			if err != nil {
				t.Fatal(msg + err.Error())
			}
			decoder.(Resetter).Reset(buf, nil)
			data2, err := io.ReadAll(decoder)
			if err != nil {
				t.Fatal(msg + err.Error())
			}
			if !bytes.Equal(data, data2) {
				t.Fatal(msg + "not equal")
			}
			// Do it again...
			msg = msg + " (reset):"
			buf.Reset()
			fw.Reset(buf)
			n, err = fw.Write(data)
			if n != len(data) {
				t.Fatal(msg + "short write")
			}
			if err != nil {
				t.Fatal(msg + err.Error())
			}
			err = fw.Close()
			if err != nil {
				t.Fatal(msg + err.Error())
			}
			decoder.(Resetter).Reset(buf, nil)
			data2, err = io.ReadAll(decoder)
			if err != nil {
				t.Fatal(msg + err.Error())
			}
		}
	})
}

package zstd

import (
	"bytes"
	"encoding/csv"
	"fmt"
	"io"
	"os"
	"reflect"
	"strconv"
	"testing"
	"time"

	"github.com/klauspost/compress/zip"
)

func TestSequenceDecsAdjustOffset(t *testing.T) {
	type result struct {
		offset     int
		prevOffset [3]int
	}

	tc := []struct {
		offset     int
		litLen     int
		offsetB    uint8
		prevOffset [3]int

		res result
	}{{
		offset:     444,
		litLen:     0,
		offsetB:    42,
		prevOffset: [3]int{111, 222, 333},

		res: result{
			offset:     444,
			prevOffset: [3]int{444, 111, 222},
		},
	}, {
		offset:     0,
		litLen:     1,
		offsetB:    0,
		prevOffset: [3]int{111, 222, 333},

		res: result{
			offset:     111,
			prevOffset: [3]int{111, 222, 333},
		},
	}, {
		offset:     -1,
		litLen:     0,
		offsetB:    0,
		prevOffset: [3]int{111, 222, 333},

		res: result{
			offset:     111,
			prevOffset: [3]int{111, 222, 333},
		},
	}, {
		offset:     1,
		litLen:     1,
		offsetB:    0,
		prevOffset: [3]int{111, 222, 333},

		res: result{
			offset:     222,
			prevOffset: [3]int{222, 111, 333},
		},
	}, {
		offset:     2,
		litLen:     1,
		offsetB:    0,
		prevOffset: [3]int{111, 222, 333},

		res: result{
			offset:     333,
			prevOffset: [3]int{333, 111, 222},
		},
	}, {
		offset:     3,
		litLen:     1,
		offsetB:    0,
		prevOffset: [3]int{111, 222, 333},

		res: result{
			offset:     110, // s.prevOffset[0] - 1
			prevOffset: [3]int{110, 111, 222},
		},
	}, {
		offset:     3,
		litLen:     1,
		offsetB:    0,
		prevOffset: [3]int{1, 222, 333},

		res: result{
			offset:     1,
			prevOffset: [3]int{1, 1, 222},
		},
	},
	}

	for i := range tc {
		// given
		var sd sequenceDecs
		for j := 0; j < 3; j++ {
			sd.prevOffset[j] = tc[i].prevOffset[j]
		}

		// when
		offset := sd.adjustOffset(tc[i].offset, tc[i].litLen, tc[i].offsetB)

		// then
		if offset != tc[i].res.offset {
			t.Logf("result:   %d", offset)
			t.Logf("expected: %d", tc[i].res.offset)
			t.Errorf("testcase #%d: wrong function result", i)
		}

		for j := 0; j < 3; j++ {
			if sd.prevOffset[j] != tc[i].res.prevOffset[j] {
				t.Logf("result:   %v", sd.prevOffset)
				t.Logf("expected: %v", tc[i].res.prevOffset)
				t.Errorf("testcase #%d: sd.prevOffset got wrongly updated", i)
				break
			}
		}
	}
}

type testSequence struct {
	n, lits, win int
	prevOffsets  [3]int
}

func (s *testSequence) parse(fn string) (ok bool) {
	n, err := fmt.Sscanf(fn, "n-%d-lits-%d-prev-%d-%d-%d-win-%d.blk", &s.n, &s.lits, &s.prevOffsets[0], &s.prevOffsets[1], &s.prevOffsets[2], &s.win)
	ok = err == nil && n == 6
	if !ok {
		fmt.Println("Unable to parse:", err, n)
	}
	return ok
}

func readDecoders(tb testing.TB, buf *bytes.Buffer, ref testSequence) sequenceDecs {
	s := sequenceDecs{
		litLengths:   sequenceDec{fse: &fseDecoder{}},
		offsets:      sequenceDec{fse: &fseDecoder{}},
		matchLengths: sequenceDec{fse: &fseDecoder{}},
		prevOffset:   ref.prevOffsets,
		dict:         nil,
		literals:     make([]byte, ref.lits, ref.lits+compressedBlockOverAlloc),
		out:          nil,
		nSeqs:        ref.n,
		br:           nil,
		seqSize:      0,
		windowSize:   ref.win,
		maxBits:      0,
	}

	s.litLengths.fse.mustReadFrom(buf)
	s.matchLengths.fse.mustReadFrom(buf)
	s.offsets.fse.mustReadFrom(buf)

	s.maxBits = s.litLengths.fse.maxBits + s.offsets.fse.maxBits + s.matchLengths.fse.maxBits
	s.br = &bitReader{}
	return s
}

func Test_seqdec_decode_regression(t *testing.T) {
	zr := testCreateZipReader("testdata/decode-regression.zip", t)

	for _, tt := range zr.File {
		t.Run(tt.Name, func(t *testing.T) {
			f, err := tt.Open()
			if err != nil {
				t.Error(err)
				return
			}
			defer f.Close()

			// Note: make sure we create stream reader
			dec, err := NewReader(f, WithDecoderConcurrency(4))
			if err != nil {
				t.Error(err)
				return
			}

			var buf []byte
			_, err = io.ReadFull(dec, buf)
			if err != nil {
				t.Error(err)
				return
			}
		})
	}
}

func Test_seqdec_decoder(t *testing.T) {
	const writeWant = false
	var buf bytes.Buffer
	zw := zip.NewWriter(&buf)

	want := map[string][]seqVals{}
	var wantOffsets = map[string][3]int{}
	if !writeWant {
		zr := testCreateZipReader("testdata/seqs-want.zip", t)
		tb := t
		for _, tt := range zr.File {
			var ref testSequence
			if !ref.parse(tt.Name) {
				tb.Skip("unable to parse:", tt.Name)
			}
			o, err := tt.Open()
			if err != nil {
				t.Fatal(err)
			}
			r := csv.NewReader(o)
			recs, err := r.ReadAll()
			if err != nil {
				t.Fatal(err)
			}
			for i, rec := range recs {
				if i == 0 {
					var o [3]int
					o[0], _ = strconv.Atoi(rec[0])
					o[1], _ = strconv.Atoi(rec[1])
					o[2], _ = strconv.Atoi(rec[2])
					wantOffsets[tt.Name] = o
					continue
				}
				s := seqVals{}
				s.mo, _ = strconv.Atoi(rec[0])
				s.ml, _ = strconv.Atoi(rec[1])
				s.ll, _ = strconv.Atoi(rec[2])
				want[tt.Name] = append(want[tt.Name], s)
			}
			o.Close()
		}
	}
	zr := testCreateZipReader("testdata/seqs.zip", t)
	tb := t
	for _, tt := range zr.File {
		var ref testSequence
		if !ref.parse(tt.Name) {
			tb.Skip("unable to parse:", tt.Name)
		}
		r, err := tt.Open()
		if err != nil {
			tb.Error(err)
			return
		}

		seqData, err := io.ReadAll(r)
		if err != nil {
			tb.Error(err)
			return
		}
		var buf = bytes.NewBuffer(seqData)
		s := readDecoders(tb, buf, ref)
		seqs := make([]seqVals, ref.n)

		t.Run(tt.Name, func(t *testing.T) {
			fatalIf := func(err error) {
				if err != nil {
					t.Fatal(err)
				}
			}
			fatalIf(s.br.init(buf.Bytes()))
			fatalIf(s.litLengths.init(s.br))
			fatalIf(s.offsets.init(s.br))
			fatalIf(s.matchLengths.init(s.br))

			err := s.decode(seqs)
			if err != nil {
				t.Error(err)
			}
			if writeWant {
				w, err := zw.Create(tt.Name)
				fatalIf(err)
				c := csv.NewWriter(w)
				w.Write([]byte(fmt.Sprintf("%d,%d,%d\n", s.prevOffset[0], s.prevOffset[1], s.prevOffset[2])))
				for _, seq := range seqs {
					c.Write([]string{strconv.Itoa(seq.mo), strconv.Itoa(seq.ml), strconv.Itoa(seq.ll)})
				}
				c.Flush()
			} else {
				if s.prevOffset != wantOffsets[tt.Name] {
					t.Errorf("want offsets %v, got %v", wantOffsets[tt.Name], s.prevOffset)
				}

				if !reflect.DeepEqual(want[tt.Name], seqs) {
					t.Errorf("got %v\nwant %v", seqs, want[tt.Name])
				}
			}
		})
	}
	if writeWant {
		zw.Close()
		os.WriteFile("testdata/seqs-want.zip", buf.Bytes(), os.ModePerm)
	}
}

func Test_seqdec_execute(t *testing.T) {
	zr := testCreateZipReader("testdata/seqs.zip", t)
	tb := t
	for _, tt := range zr.File {
		var ref testSequence
		if !ref.parse(tt.Name) {
			tb.Skip("unable to parse:", tt.Name)
		}
		r, err := tt.Open()
		if err != nil {
			tb.Error(err)
			return
		}

		seqData, err := io.ReadAll(r)
		if err != nil {
			tb.Error(err)
			return
		}
		var buf = bytes.NewBuffer(seqData)
		s := readDecoders(tb, buf, ref)
		seqs := make([]seqVals, ref.n)

		fatalIf := func(err error) {
			if err != nil {
				tb.Fatal(err)
			}
		}
		fatalIf(s.br.init(buf.Bytes()))
		fatalIf(s.litLengths.init(s.br))
		fatalIf(s.offsets.init(s.br))
		fatalIf(s.matchLengths.init(s.br))

		fatalIf(s.decode(seqs))
		hist := make([]byte, ref.win)
		lits := s.literals

		t.Run(tt.Name, func(t *testing.T) {
			s.literals = lits
			if len(s.out) > 0 {
				s.out = s.out[:0]
			}
			err := s.execute(seqs, hist)
			if err != nil {
				t.Fatal(err)
			}
			if len(s.out) != s.seqSize {
				t.Errorf("want %d != got %d", s.seqSize, len(s.out))
			}
		})
	}
}

func Test_seqdec_decodeSync(t *testing.T) {
	zr := testCreateZipReader("testdata/seqs.zip", t)
	tb := t
	for _, tt := range zr.File {
		var ref testSequence
		if !ref.parse(tt.Name) {
			tb.Skip("unable to parse:", tt.Name)
		}
		r, err := tt.Open()
		if err != nil {
			tb.Error(err)
			return
		}

		seqData, err := io.ReadAll(r)
		if err != nil {
			tb.Error(err)
			return
		}
		var buf = bytes.NewBuffer(seqData)
		s := readDecoders(tb, buf, ref)

		lits := s.literals
		hist := make([]byte, ref.win)
		t.Run(tt.Name, func(t *testing.T) {
			fatalIf := func(err error) {
				if err != nil {
					t.Fatal(err)
				}
			}
			fatalIf(s.br.init(buf.Bytes()))
			fatalIf(s.litLengths.init(s.br))
			fatalIf(s.offsets.init(s.br))
			fatalIf(s.matchLengths.init(s.br))
			s.literals = lits
			if len(s.out) > 0 {
				s.out = s.out[:0]
			}
			err := s.decodeSync(hist)
			if err != nil {
				t.Fatal(err)
			}
		})
	}
}

func Benchmark_seqdec_decode(b *testing.B) {
	benchmark_seqdec_decode(b)
}

func benchmark_seqdec_decode(b *testing.B) {
	zr := testCreateZipReader("testdata/seqs.zip", b)
	tb := b
	for _, tt := range zr.File {
		var ref testSequence
		if !ref.parse(tt.Name) {
			tb.Skip("unable to parse:", tt.Name)
		}
		r, err := tt.Open()
		if err != nil {
			tb.Error(err)
			return
		}

		seqData, err := io.ReadAll(r)
		if err != nil {
			tb.Error(err)
			return
		}
		var buf = bytes.NewBuffer(seqData)
		s := readDecoders(tb, buf, ref)
		seqs := make([]seqVals, ref.n)

		b.Run(tt.Name, func(b *testing.B) {
			fatalIf := func(err error) {
				if err != nil {
					b.Fatal(err)
				}
			}
			b.ReportAllocs()
			b.ResetTimer()
			t := time.Now()
			decoded := 0
			remain := uint(0)
			for i := 0; i < b.N; i++ {
				fatalIf(s.br.init(buf.Bytes()))
				fatalIf(s.litLengths.init(s.br))
				fatalIf(s.offsets.init(s.br))
				fatalIf(s.matchLengths.init(s.br))
				remain = s.br.remain()
				err := s.decode(seqs)
				if err != nil {
					b.Fatal(err)
				}
				decoded += ref.n
			}
			b.ReportMetric(float64(decoded)/time.Since(t).Seconds(), "seq/s")
			b.ReportMetric(float64(remain)/float64(s.nSeqs), "b/seq")
		})
	}
}

func Benchmark_seqdec_execute(b *testing.B) {
	zr := testCreateZipReader("testdata/seqs.zip", b)
	tb := b
	for _, tt := range zr.File {
		var ref testSequence
		if !ref.parse(tt.Name) {
			tb.Skip("unable to parse:", tt.Name)
		}
		r, err := tt.Open()
		if err != nil {
			tb.Error(err)
			return
		}

		seqData, err := io.ReadAll(r)
		if err != nil {
			tb.Error(err)
			return
		}
		var buf = bytes.NewBuffer(seqData)
		s := readDecoders(tb, buf, ref)
		seqs := make([]seqVals, ref.n)

		fatalIf := func(err error) {
			if err != nil {
				b.Fatal(err)
			}
		}
		fatalIf(s.br.init(buf.Bytes()))
		fatalIf(s.litLengths.init(s.br))
		fatalIf(s.offsets.init(s.br))
		fatalIf(s.matchLengths.init(s.br))

		fatalIf(s.decode(seqs))
		hist := make([]byte, ref.win)
		lits := s.literals

		b.Run(tt.Name, func(b *testing.B) {
			b.ReportAllocs()
			b.SetBytes(int64(s.seqSize))
			b.ResetTimer()
			t := time.Now()
			decoded := 0
			for i := 0; i < b.N; i++ {
				s.literals = lits
				if len(s.out) > 0 {
					s.out = s.out[:0]
				}
				fatalIf(s.execute(seqs, hist))
				decoded += ref.n
			}
			b.ReportMetric(float64(decoded)/time.Since(t).Seconds(), "seq/s")
		})
	}
}

func Benchmark_seqdec_decodeSync(b *testing.B) {
	zr := testCreateZipReader("testdata/seqs.zip", b)
	tb := b
	for _, tt := range zr.File {
		var ref testSequence
		if !ref.parse(tt.Name) {
			tb.Skip("unable to parse:", tt.Name)
		}
		r, err := tt.Open()
		if err != nil {
			tb.Error(err)
			return
		}

		seqData, err := io.ReadAll(r)
		if err != nil {
			tb.Error(err)
			return
		}
		var buf = bytes.NewBuffer(seqData)
		s := readDecoders(tb, buf, ref)

		lits := s.literals
		hist := make([]byte, ref.win)
		b.Run(tt.Name, func(b *testing.B) {
			fatalIf := func(err error) {
				if err != nil {
					b.Fatal(err)
				}
			}
			decoded := 0
			b.ReportAllocs()
			b.ResetTimer()
			t := time.Now()

			for i := 0; i < b.N; i++ {
				fatalIf(s.br.init(buf.Bytes()))
				fatalIf(s.litLengths.init(s.br))
				fatalIf(s.offsets.init(s.br))
				fatalIf(s.matchLengths.init(s.br))
				s.literals = lits
				if len(s.out) > 0 {
					s.out = s.out[:0]
				}
				err := s.decodeSync(hist)
				if err != nil {
					b.Fatal(err)
				}
				b.SetBytes(int64(len(s.out)))
				decoded += ref.n
			}
			b.ReportMetric(float64(decoded)/time.Since(t).Seconds(), "seq/s")
		})
	}
}

func testCreateZipReader(path string, tb testing.TB) *zip.Reader {
	failOnError := func(err error) {
		if err != nil {
			tb.Fatal(err)
		}
	}

	data, err := os.ReadFile(path)
	failOnError(err)

	zr, err := zip.NewReader(bytes.NewReader(data), int64(len(data)))
	failOnError(err)

	return zr
}

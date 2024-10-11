//go:build amd64 && !appengine && !noasm && gc
// +build amd64,!appengine,!noasm,gc

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

	"github.com/klauspost/compress/internal/cpuinfo"
	"github.com/klauspost/compress/zip"
)

func Benchmark_seqdec_decodeNoBMI(b *testing.B) {
	if !cpuinfo.HasBMI2() {
		b.Skip("Already tested, platform does not have bmi2")
		return
	}
	defer cpuinfo.DisableBMI2()()

	benchmark_seqdec_decode(b)
}

func Test_sequenceDecs_decodeNoBMI(t *testing.T) {
	if !cpuinfo.HasBMI2() {
		t.Skip("Already tested, platform does not have bmi2")
		return
	}
	defer cpuinfo.DisableBMI2()()

	const writeWant = false
	var buf bytes.Buffer
	zw := zip.NewWriter(&buf)

	want := map[string][]seqVals{}
	var wantOffsets = map[string][3]int{}
	if !writeWant {
		fn := "testdata/seqs-want.zip"
		data, err := os.ReadFile(fn)
		tb := t
		if err != nil {
			tb.Fatal(err)
		}
		zr, err := zip.NewReader(bytes.NewReader(data), int64(len(data)))
		if err != nil {
			tb.Fatal(err)
		}
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
	fn := "testdata/seqs.zip"
	data, err := os.ReadFile(fn)
	tb := t
	if err != nil {
		tb.Fatal(err)
	}
	zr, err := zip.NewReader(bytes.NewReader(data), int64(len(data)))
	if err != nil {
		tb.Fatal(err)
	}
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

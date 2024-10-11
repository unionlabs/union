package huff0

import (
	"bytes"
	"fmt"
	"io"
	"os"
	"testing"

	"github.com/klauspost/compress/zip"
)

func TestDecompress1X(t *testing.T) {
	for _, test := range testfiles {
		t.Run(test.name, func(t *testing.T) {
			var s = &Scratch{}
			buf0, err := test.fn()
			if err != nil {
				t.Fatal(err)
			}
			if len(buf0) > BlockSizeMax {
				buf0 = buf0[:BlockSizeMax]
			}
			b, re, err := Compress1X(buf0, s)
			if err != test.err1X {
				t.Errorf("want error %v (%T), got %v (%T)", test.err1X, test.err1X, err, err)
			}
			if err != nil {
				t.Log(test.name, err.Error())
				return
			}
			if b == nil {
				t.Error("got no output")
				return
			}
			if len(s.OutTable) == 0 {
				t.Error("got no table definition")
			}
			if re {
				t.Error("claimed to have re-used.")
			}
			if len(s.OutData) == 0 {
				t.Error("got no data output")
			}

			wantRemain := len(s.OutData)
			t.Logf("%s: %d -> %d bytes (%.2f:1) %t (table: %d bytes)", test.name, len(buf0), len(b), float64(len(buf0))/float64(len(b)), re, len(s.OutTable))

			s.Out = nil
			var remain []byte
			s, remain, err = ReadTable(b, s)
			if err != nil {
				t.Error(err)
				return
			}
			var buf bytes.Buffer
			if s.matches(s.prevTable, &buf); buf.Len() > 0 {
				t.Error(buf.String())
			}
			if len(remain) != wantRemain {
				t.Fatalf("remain mismatch, want %d, got %d bytes", wantRemain, len(remain))
			}
			t.Logf("remain: %d bytes, ok", len(remain))
			dc, err := s.Decompress1X(remain)
			if err != nil {
				t.Error(err)
				return
			}
			if len(buf0) != len(dc) {
				t.Errorf(test.name+"decompressed, want size: %d, got %d", len(buf0), len(dc))
				if len(buf0) > len(dc) {
					buf0 = buf0[:len(dc)]
				} else {
					dc = dc[:len(buf0)]
				}
				if !bytes.Equal(buf0, dc) {
					if len(dc) > 1024 {
						t.Log(string(dc[:1024]))
						t.Errorf(test.name+"decompressed, got delta: \n(in)\t%02x !=\n(out)\t%02x\n", buf0[:1024], dc[:1024])
					} else {
						t.Log(string(dc))
						t.Errorf(test.name+"decompressed, got delta: (in) %v != (out) %v\n", buf0, dc)
					}
				}
				return
			}
			if !bytes.Equal(buf0, dc) {
				if len(buf0) > 1024 {
					t.Log(string(dc[:1024]))
				} else {
					t.Log(string(dc))
				}
				//t.Errorf(test.name+": decompressed, got delta: \n%s")
				t.Errorf(test.name + ": decompressed, got delta")
			}
			if !t.Failed() {
				t.Log("... roundtrip ok!")
			}
		})
	}
}

func TestDecompress1XRegression(t *testing.T) {
	data, err := os.ReadFile("testdata/decompress1x_regression.zip")
	if err != nil {
		t.Fatal(err)
	}
	zr, err := zip.NewReader(bytes.NewReader(data), int64(len(data)))
	if err != nil {
		t.Fatal(err)
	}
	for _, tt := range zr.File {
		if tt.UncompressedSize64 == 0 {
			continue
		}
		rc, err := tt.Open()
		if err != nil {
			t.Fatal(err)
		}
		data, err := io.ReadAll(rc)
		if err != nil {
			t.Fatal(err)
		}

		t.Run(tt.Name, func(t *testing.T) {
			s, rem, err := ReadTable(data, nil)
			if err != nil {
				t.Fatal(err)
			}
			_, err = s.Decompress1X(rem)
			if err == nil {
				t.Fatal("expected error to be returned")
			}

			t.Logf("returned error: %s", err)
		})
	}
}

func TestDecompress4X(t *testing.T) {
	for _, test := range testfiles {
		t.Run(test.name, func(t *testing.T) {
			for _, tl := range []uint8{0, 5, 6, 7, 8, 9, 10, 11} {
				t.Run(fmt.Sprintf("tablelog-%d", tl), func(t *testing.T) {
					var s = &Scratch{}
					s.TableLog = tl
					buf0, err := test.fn()
					if err != nil {
						t.Fatal(err)
					}
					if len(buf0) > BlockSizeMax {
						buf0 = buf0[:BlockSizeMax]
					}
					b, re, err := Compress4X(buf0, s)
					if err != test.err4X {
						t.Errorf("want error %v (%T), got %v (%T)", test.err1X, test.err1X, err, err)
					}
					if err != nil {
						t.Log(test.name, err.Error())
						return
					}
					if b == nil {
						t.Error("got no output")
						return
					}
					if len(s.OutTable) == 0 {
						t.Error("got no table definition")
					}
					if re {
						t.Error("claimed to have re-used.")
					}
					if len(s.OutData) == 0 {
						t.Error("got no data output")
					}

					wantRemain := len(s.OutData)
					t.Logf("%s: %d -> %d bytes (%.2f:1) %t (table: %d bytes)", test.name, len(buf0), len(b), float64(len(buf0))/float64(len(b)), re, len(s.OutTable))

					s.Out = nil
					var remain []byte
					s, remain, err = ReadTable(b, s)
					if err != nil {
						t.Error(err)
						return
					}
					var buf bytes.Buffer
					if s.matches(s.prevTable, &buf); buf.Len() > 0 {
						t.Error(buf.String())
					}
					if len(remain) != wantRemain {
						t.Fatalf("remain mismatch, want %d, got %d bytes", wantRemain, len(remain))
					}
					t.Logf("remain: %d bytes, ok", len(remain))
					dc, err := s.Decompress4X(remain, len(buf0))
					if err != nil {
						t.Error(err)
						return
					}
					if len(buf0) != len(dc) {
						t.Errorf(test.name+"decompressed, want size: %d, got %d", len(buf0), len(dc))
						if len(buf0) > len(dc) {
							buf0 = buf0[:len(dc)]
						} else {
							dc = dc[:len(buf0)]
						}
						if !bytes.Equal(buf0, dc) {
							if len(dc) > 1024 {
								t.Log(string(dc[:1024]))
								t.Errorf(test.name+"decompressed, got delta: \n(in)\t%02x !=\n(out)\t%02x\n", buf0[:1024], dc[:1024])
							} else {
								t.Log(string(dc))
								t.Errorf(test.name+"decompressed, got delta: (in) %v != (out) %v\n", buf0, dc)
							}
						}
						return
					}
					if !bytes.Equal(buf0, dc) {
						if len(buf0) > 1024 {
							t.Log(string(dc[:1024]))
						} else {
							t.Log(string(dc))
						}
						//t.Errorf(test.name+": decompressed, got delta: \n%s")
						t.Errorf(test.name + ": decompressed, got delta")
					}
					if !t.Failed() {
						t.Log("... roundtrip ok!")
					}

				})
			}
		})
	}
}

func TestRoundtrip1XFuzz(t *testing.T) {
	for _, test := range testfilesExtended {
		t.Run(test.name, func(t *testing.T) {
			var s = &Scratch{}
			buf0, err := test.fn()
			if err != nil {
				t.Fatal(err)
			}
			if len(buf0) > BlockSizeMax {
				buf0 = buf0[:BlockSizeMax]
			}
			b, re, err := Compress1X(buf0, s)
			if err != nil {
				if err == ErrIncompressible || err == ErrUseRLE || err == ErrTooBig {
					t.Log(test.name, err.Error())
					return
				}
				t.Error(test.name, err.Error())
				return
			}
			if b == nil {
				t.Error("got no output")
				return
			}
			if len(s.OutTable) == 0 {
				t.Error("got no table definition")
			}
			if re {
				t.Error("claimed to have re-used.")
			}
			if len(s.OutData) == 0 {
				t.Error("got no data output")
			}

			wantRemain := len(s.OutData)
			t.Logf("%s: %d -> %d bytes (%.2f:1) %t (table: %d bytes)", test.name, len(buf0), len(b), float64(len(buf0))/float64(len(b)), re, len(s.OutTable))

			s.Out = nil
			var remain []byte
			s, remain, err = ReadTable(b, s)
			if err != nil {
				t.Error(err)
				return
			}
			var buf bytes.Buffer
			if s.matches(s.prevTable, &buf); buf.Len() > 0 {
				t.Error(buf.String())
			}
			if len(remain) != wantRemain {
				t.Fatalf("remain mismatch, want %d, got %d bytes", wantRemain, len(remain))
			}
			t.Logf("remain: %d bytes, ok", len(remain))
			dc, err := s.Decompress1X(remain)
			if err != nil {
				t.Error(err)
				return
			}
			if len(buf0) != len(dc) {
				t.Errorf(test.name+"decompressed, want size: %d, got %d", len(buf0), len(dc))
				if len(buf0) > len(dc) {
					buf0 = buf0[:len(dc)]
				} else {
					dc = dc[:len(buf0)]
				}
				if !bytes.Equal(buf0, dc) {
					if len(dc) > 1024 {
						t.Log(string(dc[:1024]))
						t.Errorf(test.name+"decompressed, got delta: \n(in)\t%02x !=\n(out)\t%02x\n", buf0[:1024], dc[:1024])
					} else {
						t.Log(string(dc))
						t.Errorf(test.name+"decompressed, got delta: (in) %v != (out) %v\n", buf0, dc)
					}
				}
				return
			}
			if !bytes.Equal(buf0, dc) {
				if len(buf0) > 1024 {
					t.Log(string(dc[:1024]))
				} else {
					t.Log(string(dc))
				}
				//t.Errorf(test.name+": decompressed, got delta: \n%s")
				t.Errorf(test.name + ": decompressed, got delta")
			}
			if !t.Failed() {
				t.Log("... roundtrip ok!")
			}
		})
	}
}

func TestRoundtrip4XFuzz(t *testing.T) {
	for _, test := range testfilesExtended {
		t.Run(test.name, func(t *testing.T) {
			var s = &Scratch{}
			buf0, err := test.fn()
			if err != nil {
				t.Fatal(err)
			}
			if len(buf0) > BlockSizeMax {
				buf0 = buf0[:BlockSizeMax]
			}
			b, re, err := Compress4X(buf0, s)
			if err != nil {
				if err == ErrIncompressible || err == ErrUseRLE || err == ErrTooBig {
					t.Log(test.name, err.Error())
					return
				}
				t.Error(test.name, err.Error())
				return
			}
			if b == nil {
				t.Error("got no output")
				return
			}
			if len(s.OutTable) == 0 {
				t.Error("got no table definition")
			}
			if re {
				t.Error("claimed to have re-used.")
			}
			if len(s.OutData) == 0 {
				t.Error("got no data output")
			}

			wantRemain := len(s.OutData)
			t.Logf("%s: %d -> %d bytes (%.2f:1) %t (table: %d bytes)", test.name, len(buf0), len(b), float64(len(buf0))/float64(len(b)), re, len(s.OutTable))

			s.Out = nil
			var remain []byte
			s, remain, err = ReadTable(b, s)
			if err != nil {
				t.Error(err)
				return
			}
			var buf bytes.Buffer
			if s.matches(s.prevTable, &buf); buf.Len() > 0 {
				t.Error(buf.String())
			}
			if len(remain) != wantRemain {
				t.Fatalf("remain mismatch, want %d, got %d bytes", wantRemain, len(remain))
			}
			t.Logf("remain: %d bytes, ok", len(remain))
			dc, err := s.Decompress4X(remain, len(buf0))
			if err != nil {
				t.Error(err)
				return
			}
			if len(buf0) != len(dc) {
				t.Errorf(test.name+"decompressed, want size: %d, got %d", len(buf0), len(dc))
				if len(buf0) > len(dc) {
					buf0 = buf0[:len(dc)]
				} else {
					dc = dc[:len(buf0)]
				}
				if !bytes.Equal(buf0, dc) {
					if len(dc) > 1024 {
						t.Log(string(dc[:1024]))
						t.Errorf(test.name+"decompressed, got delta: \n(in)\t%02x !=\n(out)\t%02x\n", buf0[:1024], dc[:1024])
					} else {
						t.Log(string(dc))
						t.Errorf(test.name+"decompressed, got delta: (in) %v != (out) %v\n", buf0, dc)
					}
				}
				return
			}
			if !bytes.Equal(buf0, dc) {
				if len(buf0) > 1024 {
					t.Log(string(dc[:1024]))
				} else {
					t.Log(string(dc))
				}
				//t.Errorf(test.name+": decompressed, got delta: \n%s")
				t.Errorf(test.name + ": decompressed, got delta")
			}
			if !t.Failed() {
				t.Log("... roundtrip ok!")
			}
		})
	}
}

func BenchmarkDecompress1XTable(b *testing.B) {
	for _, tt := range testfiles {
		test := tt
		if test.err1X != nil {
			continue
		}
		b.Run(test.name, func(b *testing.B) {
			var s = &Scratch{}
			s.Reuse = ReusePolicyNone
			buf0, err := test.fn()
			if err != nil {
				b.Fatal(err)
			}
			if len(buf0) > BlockSizeMax {
				buf0 = buf0[:BlockSizeMax]
			}
			compressed, _, err := Compress1X(buf0, s)
			if err != test.err1X {
				b.Fatal("unexpected error:", err)
			}
			s.Out = nil
			s, remain, _ := ReadTable(compressed, s)
			s.Decompress1X(remain)
			b.ResetTimer()
			b.ReportAllocs()
			b.SetBytes(int64(len(buf0)))
			for i := 0; i < b.N; i++ {
				s, remain, err := ReadTable(compressed, s)
				if err != nil {
					b.Fatal(err)
				}
				_, err = s.Decompress1X(remain)
				if err != nil {
					b.Fatal(err)
				}
			}
		})
	}
}

func BenchmarkDecompress1XNoTable(b *testing.B) {
	for _, tt := range testfiles {
		test := tt
		if test.err1X != nil {
			continue
		}
		b.Run(test.name, func(b *testing.B) {
			for _, sz := range []int{1e2, 1e4, BlockSizeMax} {
				b.Run(fmt.Sprintf("%d", sz), func(b *testing.B) {
					var s = &Scratch{}
					s.Reuse = ReusePolicyNone
					buf0, err := test.fn()
					if err != nil {
						b.Fatal(err)
					}
					for len(buf0) < sz {
						buf0 = append(buf0, buf0...)
					}
					if len(buf0) > sz {
						buf0 = buf0[:sz]
					}
					compressed, _, err := Compress1X(buf0, s)
					if err != test.err1X {
						if err == ErrUseRLE {
							b.Skip("RLE")
							return
						}
						b.Fatal("unexpected error:", err)
					}
					s.Out = nil
					s, remain, _ := ReadTable(compressed, s)
					s.Decompress1X(remain)
					b.ResetTimer()
					b.ReportAllocs()
					b.SetBytes(int64(len(buf0)))
					for i := 0; i < b.N; i++ {
						_, err = s.Decompress1X(remain)
						if err != nil {
							b.Fatal(err)
						}
					}
					b.ReportMetric(float64(s.actualTableLog), "log")
					b.ReportMetric(100*float64(len(compressed))/float64(len(buf0)), "pct")
				})
			}
		})
	}
}

func BenchmarkDecompress4XNoTable(b *testing.B) {
	for _, tt := range testfiles {
		test := tt
		if test.err4X != nil {
			continue
		}
		b.Run(test.name, func(b *testing.B) {
			for _, sz := range []int{1e2, 1e4, BlockSizeMax} {
				b.Run(fmt.Sprintf("%d", sz), func(b *testing.B) {
					var s = &Scratch{}
					s.Reuse = ReusePolicyNone
					buf0, err := test.fn()
					if err != nil {
						b.Fatal(err)
					}
					for len(buf0) < sz {
						buf0 = append(buf0, buf0...)
					}
					if len(buf0) > sz {
						buf0 = buf0[:sz]
					}
					compressed, _, err := Compress4X(buf0, s)
					if err != test.err4X {
						if err == ErrUseRLE {
							b.Skip("RLE")
							return
						}
						b.Fatal("unexpected error:", err)
					}
					s.Out = nil
					s, remain, _ := ReadTable(compressed, s)
					s.Decompress4X(remain, len(buf0))
					b.ResetTimer()
					b.ReportAllocs()
					b.SetBytes(int64(len(buf0)))
					for i := 0; i < b.N; i++ {
						_, err = s.Decompress4X(remain, len(buf0))
						if err != nil {
							b.Fatal(err)
						}
					}
					b.ReportMetric(float64(s.actualTableLog), "log")
					b.ReportMetric(100*float64(len(compressed))/float64(len(buf0)), "pct")

				})
			}
		})
	}
}

func BenchmarkDecompress4XNoTableTableLog8(b *testing.B) {
	for _, tt := range testfiles[:1] {
		test := tt
		if test.err4X != nil {
			continue
		}
		b.Run(test.name, func(b *testing.B) {
			var s = &Scratch{}
			s.Reuse = ReusePolicyNone
			buf0, err := test.fn()
			if err != nil {
				b.Fatal(err)
			}
			if len(buf0) > BlockSizeMax {
				buf0 = buf0[:BlockSizeMax]
			}
			s.TableLog = 8
			compressed, _, err := Compress4X(buf0, s)
			if err != test.err1X {
				b.Fatal("unexpected error:", err)
			}
			s.Out = nil
			s, remain, _ := ReadTable(compressed, s)
			s.Decompress4X(remain, len(buf0))
			b.ResetTimer()
			b.ReportAllocs()
			b.SetBytes(int64(len(buf0)))
			for i := 0; i < b.N; i++ {
				_, err = s.Decompress4X(remain, len(buf0))
				if err != nil {
					b.Fatal(err)
				}
			}
		})
	}
}

func BenchmarkDecompress4XTable(b *testing.B) {
	for _, tt := range testfiles {
		test := tt
		if test.err4X != nil {
			continue
		}
		b.Run(test.name, func(b *testing.B) {
			var s = &Scratch{}
			s.Reuse = ReusePolicyNone
			buf0, err := test.fn()
			if err != nil {
				b.Fatal(err)
			}
			if len(buf0) > BlockSizeMax {
				buf0 = buf0[:BlockSizeMax]
			}
			compressed, _, err := Compress4X(buf0, s)
			if err != test.err1X {
				b.Fatal("unexpected error:", err)
			}
			s.Out = nil
			b.ResetTimer()
			b.ReportAllocs()
			b.SetBytes(int64(len(buf0)))
			for i := 0; i < b.N; i++ {
				s, remain, err := ReadTable(compressed, s)
				if err != nil {
					b.Fatal(err)
				}
				_, err = s.Decompress4X(remain, len(buf0))
				if err != nil {
					b.Fatal(err)
				}
			}
		})
	}
}

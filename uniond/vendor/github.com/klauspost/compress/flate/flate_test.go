// Copyright 2009 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// This test tests some internals of the flate package.
// The tests in package compress/gzip serve as the
// end-to-end test of the decompressor.

package flate

import (
	"archive/zip"
	"bytes"
	"compress/flate"
	"encoding/hex"
	"fmt"
	"io"
	"os"
	"testing"
)

// The following test should not panic.
func TestIssue5915(t *testing.T) {
	bits := []int{4, 0, 0, 6, 4, 3, 2, 3, 3, 4, 4, 5, 0, 0, 0, 0, 5, 5, 6,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 8, 6, 0, 11, 0, 8, 0, 6, 6, 10, 8}
	var h huffmanDecoder
	if h.init(bits) {
		t.Fatalf("Given sequence of bits is bad, and should not succeed.")
	}
}

// The following test should not panic.
func TestIssue5962(t *testing.T) {
	bits := []int{4, 0, 0, 6, 4, 3, 2, 3, 3, 4, 4, 5, 0, 0, 0, 0,
		5, 5, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 11}
	var h huffmanDecoder
	if h.init(bits) {
		t.Fatalf("Given sequence of bits is bad, and should not succeed.")
	}
}

// The following test should not panic.
func TestIssue6255(t *testing.T) {
	bits1 := []int{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 11}
	bits2 := []int{11, 13}
	var h huffmanDecoder
	if !h.init(bits1) {
		t.Fatalf("Given sequence of bits is good and should succeed.")
	}
	if h.init(bits2) {
		t.Fatalf("Given sequence of bits is bad and should not succeed.")
	}
}

func TestInvalidEncoding(t *testing.T) {
	// Initialize Huffman decoder to recognize "0".
	var h huffmanDecoder
	if !h.init([]int{1}) {
		t.Fatal("Failed to initialize Huffman decoder")
	}

	// Initialize decompressor with invalid Huffman coding.
	var f decompressor
	f.r = bytes.NewReader([]byte{0xff})

	_, err := f.huffSym(&h)
	if err == nil {
		t.Fatal("Should have rejected invalid bit sequence")
	}
}

func TestRegressions(t *testing.T) {
	// Test fuzzer regressions
	data, err := os.ReadFile("testdata/regression.zip")
	if err != nil {
		t.Fatal(err)
	}
	zr, err := zip.NewReader(bytes.NewReader(data), int64(len(data)))
	if err != nil {
		t.Fatal(err)
	}
	for _, tt := range zr.File {
		data, err := tt.Open()
		if err != nil {
			t.Fatal(err)
		}
		data1, err := io.ReadAll(data)
		if err != nil {
			t.Fatal(err)
		}
		t.Run(tt.Name, func(t *testing.T) {
			if testing.Short() && len(data1) > 10000 {
				t.SkipNow()
			}
			for level := 0; level <= 9; level++ {
				t.Run(fmt.Sprint(tt.Name+"-level", 1), func(t *testing.T) {
					buf := new(bytes.Buffer)
					fw, err := NewWriter(buf, level)
					if err != nil {
						t.Error(err)
					}
					n, err := fw.Write(data1)
					if n != len(data1) {
						t.Error("short write")
					}
					if err != nil {
						t.Error(err)
					}
					err = fw.Close()
					if err != nil {
						t.Error(err)
					}
					fr1 := NewReader(buf)
					data2, err := io.ReadAll(fr1)
					if err != nil {
						t.Error(err)
					}
					if !bytes.Equal(data1, data2) {
						t.Error("not equal")
					}
					// Do it again...
					buf.Reset()
					fw.Reset(buf)
					n, err = fw.Write(data1)
					if n != len(data1) {
						t.Error("short write")
					}
					if err != nil {
						t.Error(err)
					}
					err = fw.Close()
					if err != nil {
						t.Error(err)
					}
					fr1 = flate.NewReader(buf)
					data2, err = io.ReadAll(fr1)
					if err != nil {
						t.Error(err)
					}
					if !bytes.Equal(data1, data2) {
						t.Error("not equal")
					}
				})
			}
			t.Run(tt.Name+"stateless", func(t *testing.T) {
				// Split into two and use history...
				buf := new(bytes.Buffer)
				err = StatelessDeflate(buf, data1[:len(data1)/2], false, nil)
				if err != nil {
					t.Error(err)
				}

				// Use top half as dictionary...
				dict := data1[:len(data1)/2]
				err = StatelessDeflate(buf, data1[len(data1)/2:], true, dict)
				if err != nil {
					t.Error(err)
				}
				t.Log(buf.Len())
				fr1 := NewReader(buf)
				data2, err := io.ReadAll(fr1)
				if err != nil {
					t.Error(err)
				}
				if !bytes.Equal(data1, data2) {
					//fmt.Printf("want:%x\ngot: %x\n", data1, data2)
					t.Error("not equal")
				}
			})
		})
	}
}

func TestInvalidBits(t *testing.T) {
	oversubscribed := []int{1, 2, 3, 4, 4, 5}
	incomplete := []int{1, 2, 4, 4}
	var h huffmanDecoder
	if h.init(oversubscribed) {
		t.Fatal("Should reject oversubscribed bit-length set")
	}
	if h.init(incomplete) {
		t.Fatal("Should reject incomplete bit-length set")
	}
}

func TestStreams(t *testing.T) {
	// To verify any of these hexstrings as valid or invalid flate streams
	// according to the C zlib library, you can use the Python wrapper library:
	// >>> hex_string = "010100feff11"
	// >>> import zlib
	// >>> zlib.decompress(hex_string.decode("hex"), -15) # Negative means raw DEFLATE
	// '\x11'

	testCases := []struct {
		desc   string // Description of the stream
		stream string // Hexstring of the input DEFLATE stream
		want   string // Expected result. Use "fail" to expect failure
	}{{
		"degenerate HCLenTree",
		"05e0010000000000100000000000000000000000000000000000000000000000" +
			"00000000000000000004",
		"fail",
	}, {
		"complete HCLenTree, empty HLitTree, empty HDistTree",
		"05e0010400000000000000000000000000000000000000000000000000000000" +
			"00000000000000000010",
		"fail",
	}, {
		"empty HCLenTree",
		"05e0010000000000000000000000000000000000000000000000000000000000" +
			"00000000000000000010",
		"fail",
	}, {
		"complete HCLenTree, complete HLitTree, empty HDistTree, use missing HDist symbol",
		"000100feff000de0010400000000100000000000000000000000000000000000" +
			"0000000000000000000000000000002c",
		"fail",
	}, {
		"complete HCLenTree, complete HLitTree, degenerate HDistTree, use missing HDist symbol",
		"000100feff000de0010000000000000000000000000000000000000000000000" +
			"00000000000000000610000000004070",
		"fail",
	}, {
		"complete HCLenTree, empty HLitTree, empty HDistTree",
		"05e0010400000000100400000000000000000000000000000000000000000000" +
			"0000000000000000000000000008",
		"fail",
	}, {
		"complete HCLenTree, empty HLitTree, degenerate HDistTree",
		"05e0010400000000100400000000000000000000000000000000000000000000" +
			"0000000000000000000800000008",
		"fail",
	}, {
		"complete HCLenTree, degenerate HLitTree, degenerate HDistTree, use missing HLit symbol",
		"05e0010400000000100000000000000000000000000000000000000000000000" +
			"0000000000000000001c",
		"fail",
	}, {
		"complete HCLenTree, complete HLitTree, too large HDistTree",
		"edff870500000000200400000000000000000000000000000000000000000000" +
			"000000000000000000080000000000000004",
		"fail",
	}, {
		"complete HCLenTree, complete HLitTree, empty HDistTree, excessive repeater code",
		"edfd870500000000200400000000000000000000000000000000000000000000" +
			"000000000000000000e8b100",
		"fail",
	}, {
		"complete HCLenTree, complete HLitTree, empty HDistTree of normal length 30",
		"05fd01240000000000f8ffffffffffffffffffffffffffffffffffffffffffff" +
			"ffffffffffffffffff07000000fe01",
		"",
	}, {
		"complete HCLenTree, complete HLitTree, empty HDistTree of excessive length 31",
		"05fe01240000000000f8ffffffffffffffffffffffffffffffffffffffffffff" +
			"ffffffffffffffffff07000000fc03",
		"fail",
	}, {
		"complete HCLenTree, over-subscribed HLitTree, empty HDistTree",
		"05e001240000000000fcffffffffffffffffffffffffffffffffffffffffffff" +
			"ffffffffffffffffff07f00f",
		"fail",
	}, {
		"complete HCLenTree, under-subscribed HLitTree, empty HDistTree",
		"05e001240000000000fcffffffffffffffffffffffffffffffffffffffffffff" +
			"fffffffffcffffffff07f00f",
		"fail",
	}, {
		"complete HCLenTree, complete HLitTree with single code, empty HDistTree",
		"05e001240000000000f8ffffffffffffffffffffffffffffffffffffffffffff" +
			"ffffffffffffffffff07f00f",
		"01",
	}, {
		"complete HCLenTree, complete HLitTree with multiple codes, empty HDistTree",
		"05e301240000000000f8ffffffffffffffffffffffffffffffffffffffffffff" +
			"ffffffffffffffffff07807f",
		"01",
	}, {
		"complete HCLenTree, complete HLitTree, degenerate HDistTree, use valid HDist symbol",
		"000100feff000de0010400000000100000000000000000000000000000000000" +
			"0000000000000000000000000000003c",
		"00000000",
	}, {
		"complete HCLenTree, degenerate HLitTree, degenerate HDistTree",
		"05e0010400000000100000000000000000000000000000000000000000000000" +
			"0000000000000000000c",
		"",
	}, {
		"complete HCLenTree, degenerate HLitTree, empty HDistTree",
		"05e0010400000000100000000000000000000000000000000000000000000000" +
			"00000000000000000004",
		"",
	}, {
		"complete HCLenTree, complete HLitTree, empty HDistTree, spanning repeater code",
		"edfd870500000000200400000000000000000000000000000000000000000000" +
			"000000000000000000e8b000",
		"",
	}, {
		"complete HCLenTree with length codes, complete HLitTree, empty HDistTree",
		"ede0010400000000100000000000000000000000000000000000000000000000" +
			"0000000000000000000400004000",
		"",
	}, {
		"complete HCLenTree, complete HLitTree, degenerate HDistTree, use valid HLit symbol 284 with count 31",
		"000100feff00ede0010400000000100000000000000000000000000000000000" +
			"000000000000000000000000000000040000407f00",
		"0000000000000000000000000000000000000000000000000000000000000000" +
			"0000000000000000000000000000000000000000000000000000000000000000" +
			"0000000000000000000000000000000000000000000000000000000000000000" +
			"0000000000000000000000000000000000000000000000000000000000000000" +
			"0000000000000000000000000000000000000000000000000000000000000000" +
			"0000000000000000000000000000000000000000000000000000000000000000" +
			"0000000000000000000000000000000000000000000000000000000000000000" +
			"0000000000000000000000000000000000000000000000000000000000000000" +
			"000000",
	}, {
		"complete HCLenTree, complete HLitTree, degenerate HDistTree, use valid HLit and HDist symbols",
		"0cc2010d00000082b0ac4aff0eb07d27060000ffff",
		"616263616263",
	}, {
		"fixed block, use reserved symbol 287",
		"33180700",
		"fail",
	}, {
		"raw block",
		"010100feff11",
		"11",
	}, {
		"issue 10426 - over-subscribed HCLenTree causes a hang",
		"344c4a4e494d4b070000ff2e2eff2e2e2e2e2eff",
		"fail",
	}, {
		"issue 11030 - empty HDistTree unexpectedly leads to error",
		"05c0070600000080400fff37a0ca",
		"",
	}, {
		"issue 11033 - empty HDistTree unexpectedly leads to error",
		"050fb109c020cca5d017dcbca044881ee1034ec149c8980bbc413c2ab35be9dc" +
			"b1473449922449922411202306ee97b0383a521b4ffdcf3217f9f7d3adb701",
		"3130303634342068652e706870005d05355f7ed957ff084a90925d19e3ebc6d0" +
			"c6d7",
	}}

	for i, tc := range testCases {
		data, err := hex.DecodeString(tc.stream)
		if err != nil {
			t.Fatal(err)
		}
		data, err = io.ReadAll(NewReader(bytes.NewReader(data)))
		if tc.want == "fail" {
			if err == nil {
				t.Errorf("#%d (%s): got nil error, want non-nil", i, tc.desc)
			}
		} else {
			if err != nil {
				t.Errorf("#%d (%s): %v", i, tc.desc, err)
				continue
			}
			if got := hex.EncodeToString(data); got != tc.want {
				t.Errorf("#%d (%s):\ngot  %q\nwant %q", i, tc.desc, got, tc.want)
			}

		}
	}
}

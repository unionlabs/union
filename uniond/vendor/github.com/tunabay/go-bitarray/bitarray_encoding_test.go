// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"bytes"
	"encoding/hex"
	"fmt"
	"testing"

	"github.com/tunabay/go-bitarray"
)

func TestBitArray_MarshalBinary(t *testing.T) {
	tdt := []string{
		"", "",
		"0", "04",
		"1", "84",
		"00", "03",
		"11", "c3",
		"000", "02",
		"111", "e2",
		"0000", "01",
		"1111", "f1",
		"0000-0", "00",
		"1111-1", "f8",
		"0000-00", "0007",
		"1111-11", "fc07",
		"0000-000", "0006",
		"1111-111", "fe06",
		"0000-0000", "0005",
		"1111-1111", "ff05",
		"0000-0000 0", "0004",
		"1111-1111 1", "ff84",

		// 127 bits
		"0x_0000_0000_0000_0000_0000_0000_0000_000 000",
		"0000000000000000000000000000000006",
		"0x_ffff_ffff_ffff_ffff_ffff_ffff_ffff_fff 111",
		"fffffffffffffffffffffffffffffffe06",

		// 128 bits
		"0x_0000_0000_0000_0000_0000_0000_0000_0000",
		"0000000000000000000000000000000005",
		"0x_ffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff",
		"ffffffffffffffffffffffffffffffff05",

		// 129 bits
		"0x_0000_0000_0000_0000_0000_0000_0000_0000 0",
		"0000000000000000000000000000000004",
		"0x_ffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff 1",
		"ffffffffffffffffffffffffffffffff84",
	}
	for i := 0; i < len(tdt); i += 2 {
		baO := bitarray.MustParse(tdt[i]).ZOptimize()
		baE := baO.ZExpand()
		want, _ := hex.DecodeString(tdt[i+1])
		if got, _ := baO.MarshalBinary(); !bytes.Equal(got, want) {
			t.Error("unexpected result:")
			t.Logf(" src: %#b", baO)
			t.Logf(" src: %s", baO.D())
			t.Logf(" got: % x", got)
			t.Logf("want: % x", want)
		}
		if got, _ := baE.MarshalBinary(); !bytes.Equal(got, want) {
			t.Error("unexpected result:")
			t.Logf(" src: %#b", baE)
			t.Logf(" src: %s", baE.D())
			t.Logf(" got: % x", got)
			t.Logf("want: % x", want)
		}
	}
	var nilba *bitarray.BitArray
	if got, _ := nilba.MarshalBinary(); got == nil || len(got) != 0 {
		t.Errorf("unexpected result: got %+v, want empty", got)
	}
}

// also tests MarshalJSON, MarshalYAML
func TestBitArray_MarshalText(t *testing.T) {
	tdt := []string{
		"", "",
		"0", "0",
		"1", "1",
		"0000", "0000",
		"0000-0000", "00000000",
		"0000-0000 0", "000000000",
		"1111-1111", "11111111",
		"1111-111111", "1111111111",

		// 127 bits
		"0x_0000_0000_0000_0000_0000_0000_0000_000 000",
		"0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
		"0x_ffff_ffff_ffff_ffff_ffff_ffff_ffff_fff 111",
		"1111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111",

		// 128 bits
		"0x_0000_0000_0000_0000_0000_0000_0000_0000",
		"00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
		"0x_ffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff",
		"11111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111",

		// 129 bits
		"0x_0000_0000_0000_0000_0000_0000_0000_0000 0",
		"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
		"0x_ffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff 1",
		"111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111",
	}
	chk := func(ba *bitarray.BitArray, want string) {
		gotB, _ := ba.MarshalText()
		if string(gotB) != want {
			t.Error("MarshalText: unexpected result:")
			t.Logf(" src: %#b", ba)
			t.Logf(" src: %s", ba.D())
			t.Logf(" got: %s", gotB)
			t.Logf("want: %s", want)
		}
		gotB, _ = ba.MarshalJSON()
		jwant := fmt.Sprintf(`"%s"`, want)
		if string(gotB) != jwant {
			t.Error("MarshalJSON: unexpected result:")
			t.Logf(" src: %#b", ba)
			t.Logf(" src: %s", ba.D())
			t.Logf(" got: %s", gotB)
			t.Logf("want: %s", jwant)
		}
		yif, _ := ba.MarshalYAML()
		ystr, ok := yif.(string)
		if !ok {
			t.Fatalf("MarshalYAML: unexpected type: %T, %+v", yif, yif)
		}
		if ystr != want {
			t.Error("MarshalYAML: unexpected result:")
			t.Logf(" src: %#b", ba)
			t.Logf(" src: %s", ba.D())
			t.Logf(" got: %s", ystr)
			t.Logf("want: %s", want)
		}
	}
	for i := 0; i < len(tdt); i += 2 {
		baO := bitarray.MustParse(tdt[i]).ZOptimize()
		baE := baO.ZExpand()
		chk(baO, tdt[i+1])
		chk(baE, tdt[i+1])
	}
	var nilba *bitarray.BitArray
	if got, _ := nilba.MarshalText(); got == nil || len(got) != 0 {
		t.Errorf("unexpected result: got %+v, want empty", got)
	}
	if got, _ := nilba.MarshalJSON(); string(got) != `""` {
		t.Errorf(`unexpected result: got %s, want ""`, got)
	}
	if got, _ := nilba.MarshalYAML(); got != nil {
		t.Errorf("unexpected result: got (%T) %+v, want nil", got, got)
	}
}

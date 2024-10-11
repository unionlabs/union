// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray_test

import (
	"reflect"
	"strconv"
	"strings"
	"testing"

	"github.com/tunabay/go-bitarray"
)

func TestBitArray_HasPrefix(t *testing.T) {
	tcs := []struct {
		b, p string
		h    bool
	}{
		{"", "", true},
		{"", "0", false},
		{"", "1", false},
		{"0", "", true},
		{"1", "", true},
		{"0", "00", false},
		{"1", "11", false},
		{"0000-0000 0000", "0000-0000 000", true},
		{"0000-0000 0000", "0000-0000 0000", true},
		{"0000-0000 0000", "0000-0000 0000-0", false},
		{"1111-1111 1111", "1111-1111 111", true},
		{"1111-1111 1111", "1111-1111 1111-1", false},
		{"0111-1111 1111-1111 1111", "1111", false},
		{"1000-0000 0000-0000 0000", "0000", false},
		{"1111-0000 0000-0000 0000", "1111", true},
		{"0000-1111 1111-1111 1111", "0000", true},
		{"0000-0011 1111-1111 1111", "0000-00", true},
		{"1111-1100 0000-0000 0000", "1111-11", true},
		{"0000-0000 0000-1111 1111", "0000-0001", false},
		{"0000-0000 0000-1111 1111", "0000-0000 0000", true},
		{"1111-1111 1111-1000 0000", "1111-1111 1111", true},
		// TODO: more cases
	}
	chk := func(ba, prefix *bitarray.BitArray, want bool) {
		t.Helper()
		if got := ba.HasPrefix(prefix); got != want {
			t.Errorf("unexpected result: got %t, want %t", got, want)
			t.Logf("target: %#b", ba)
			t.Logf("prefix: %#b", prefix)
		}
	}
	for _, tc := range tcs {
		baO := bitarray.MustParse(tc.b).ZOptimize()
		baE := baO.ZExpand()
		preO := bitarray.MustParse(tc.p).ZOptimize()
		preE := preO.ZExpand()
		chk(baO, preO, tc.h)
		chk(baO, preE, tc.h)
		chk(baE, preO, tc.h)
		chk(baE, preE, tc.h)
	}

	var nilba *bitarray.BitArray
	zeroba := bitarray.New()
	ba := bitarray.MustParse("0101")
	switch {
	case !nilba.HasPrefix(nil): // untyped nil
		t.Error("unexpected result: got false, want true")
	case !nilba.HasPrefix(nilba):
		t.Error("unexpected result: got false, want true")
	case !nilba.HasPrefix(zeroba):
		t.Error("unexpected result: got false, want true")
	case !zeroba.HasPrefix(nilba):
		t.Error("unexpected result: got false, want true")
	case !ba.HasPrefix(nilba):
		t.Error("unexpected result: got false, want true")
	case nilba.HasPrefix(ba):
		t.Error("unexpected result: got true, want false")
	}
}

func TestBitArray_HasSuffix(t *testing.T) {
	tcs := []struct {
		b, s string
		h    bool
	}{
		{"", "", true},
		{"", "0", false},
		{"", "1", false},
		{"0", "", true},
		{"1", "", true},
		{"0", "00", false},
		{"1", "11", false},
		{"0000-0000 0000", "0000-0000 000", true},
		{"0000-0000 0000", "0000-0000 0000", true},
		{"0000-0000 0000", "0000-0000 0000-0", false},
		{"1111-1111 1111", "1111-1111 111", true},
		{"1111-1111 1111", "1111-1111 1111-1", false},
		{"1111-1111 1111-1111 1110", "1111", false},
		{"0000-0000 0000-0000 0001", "0000", false},
		{"0000-0000 0000-0000 1111", "1111", true},
		{"1111-1111 1111-1111 0000", "0000", true},
		{"1111-1111 1111-1100 0000", "0000-00", true},
		// TODO: more cases
	}
	chk := func(ba, suffix *bitarray.BitArray, want bool) {
		t.Helper()
		if got := ba.HasSuffix(suffix); got != want {
			t.Errorf("unexpected result: got %t, want %t", got, want)
			t.Logf("target: %#b", ba)
			t.Logf("suffix: %#b", suffix)
		}
	}
	for _, tc := range tcs {
		baO := bitarray.MustParse(tc.b).ZOptimize()
		baE := baO.ZExpand()
		sufO := bitarray.MustParse(tc.s).ZOptimize()
		sufE := sufO.ZExpand()
		chk(baO, sufO, tc.h)
		chk(baO, sufE, tc.h)
		chk(baE, sufO, tc.h)
		chk(baE, sufE, tc.h)
	}

	var nilba *bitarray.BitArray
	zeroba := bitarray.New()
	ba := bitarray.MustParse("0101")
	switch {
	case !nilba.HasSuffix(nil): // untyped nil
		t.Error("unexpected result: got false, want true")
	case !nilba.HasSuffix(nilba):
		t.Error("unexpected result: got false, want true")
	case !nilba.HasSuffix(zeroba):
		t.Error("unexpected result: got false, want true")
	case !zeroba.HasSuffix(nilba):
		t.Error("unexpected result: got false, want true")
	case !ba.HasSuffix(nilba):
		t.Error("unexpected result: got false, want true")
	case nilba.HasSuffix(ba):
		t.Error("unexpected result: got true, want false")
	}
}

func TestBitArray_Index(t *testing.T) {
	tcs := []struct {
		h, n string
		i    int
	}{
		{"", "", 0},
		{"", "0", -1},
		{"", "1", -1},
		{"0", "", 0},
		{"1", "", 0},
		{"0", "00", -1},
		{"1", "11", -1},
		{"0000-0000 0000", "0000-0000 000", 0},
		{"0000-0000 0000", "0000-0000 0000", 0},
		{"0000-0000 0000", "0000-0000 0000-0", -1},
		{"1111-1111 1111", "1111-1111 111", 0},
		{"1111-1111 1111", "1111-1111 1111-1", -1},
		{"1111-1111 1111-1111 1110", "1111", 0},
		{"1000-0000 0000-0000 0001", "0000", 1},
		{"0000-1111 0000-0000 1111", "1111", 4},
		{"1111-1111 1111-1111 0000", "0000", 16},
		{"1111-1111 1111-1100 0000", "0000-00", 14},
		// TODO: more cases
	}
	chk := func(haystack, needle *bitarray.BitArray, want int) {
		t.Helper()
		if got := haystack.Index(needle); got != want {
			t.Errorf("unexpected result: got %d, want %d", got, want)
			t.Logf("haystack: %#b", haystack)
			t.Logf("  needle: %#b", needle)
		}
	}
	for _, tc := range tcs {
		haystackO := bitarray.MustParse(tc.h).ZOptimize()
		haystackE := haystackO.ZExpand()
		needleO := bitarray.MustParse(tc.n).ZOptimize()
		needleE := needleO.ZExpand()
		chk(haystackO, needleO, tc.i)
		chk(haystackO, needleE, tc.i)
		chk(haystackE, needleO, tc.i)
		chk(haystackE, needleE, tc.i)
	}

	var nilba *bitarray.BitArray
	zeroba := bitarray.New()
	ba := bitarray.MustParse("0101")
	if got := nilba.Index(nil); got != 0 { // untyped nil
		t.Errorf("unexpected result: got %d, want 0", got)
	}
	if got := nilba.Index(nilba); got != 0 {
		t.Errorf("unexpected result: got %d, want 0", got)
	}
	if got := nilba.Index(zeroba); got != 0 {
		t.Errorf("unexpected result: got %d, want 0", got)
	}
	if got := zeroba.Index(nilba); got != 0 {
		t.Errorf("unexpected result: got %d, want 0", got)
	}
	if got := ba.Index(nilba); got != 0 {
		t.Errorf("unexpected result: got %d, want 0", got)
	}
	if got := nilba.Index(ba); got != -1 {
		t.Errorf("unexpected result: got %d, want -1", got)
	}
}

func TestBitArray_LastIndex(t *testing.T) {
	tcs := []struct {
		h, n string
		i    int
	}{
		{"", "", 0},
		{"", "0", -1},
		{"", "1", -1},
		{"0", "", 1},
		{"1", "", 1},
		{"0", "00", -1},
		{"1", "11", -1},
		{"0000-0000 0000", "0000-0000 000", 1},
		{"0000-0000 0000", "0000-0000 0000", 0},
		{"0000-0000 0000", "0000-0000 0000-0", -1},
		{"1111-1111 1111", "1111-1111 111", 1},
		{"1111-1111 1111", "1111-1111 1111-1", -1},
		{"1111-1111 1111-1111 1110", "1111", 15},
		{"1000-0000 0000-0000 0001", "0000", 15},
		{"0000-1111 0000-0000 1111", "1111", 16},
		{"1111-1111 1111-1111 0000", "0000", 16},
		{"1111-1111 1111-1100 0000", "0000-00", 14},
		// TODO: more cases
	}
	chk := func(haystack, needle *bitarray.BitArray, want int) {
		t.Helper()
		if got := haystack.LastIndex(needle); got != want {
			t.Errorf("unexpected result: got %d, want %d", got, want)
			t.Logf("haystack: %#b", haystack)
			t.Logf("  needle: %#b", needle)
		}
	}
	for _, tc := range tcs {
		haystackO := bitarray.MustParse(tc.h).ZOptimize()
		haystackE := haystackO.ZExpand()
		needleO := bitarray.MustParse(tc.n).ZOptimize()
		needleE := needleO.ZExpand()
		chk(haystackO, needleO, tc.i)
		chk(haystackO, needleE, tc.i)
		chk(haystackE, needleO, tc.i)
		chk(haystackE, needleE, tc.i)
	}

	var nilba *bitarray.BitArray
	zeroba := bitarray.New()
	ba := bitarray.MustParse("0101")
	if got := nilba.LastIndex(nil); got != 0 { // untyped nil
		t.Errorf("unexpected result: got %d, want 0", got)
	}
	if got := nilba.LastIndex(nilba); got != 0 {
		t.Errorf("unexpected result: got %d, want 0", got)
	}
	if got := nilba.LastIndex(zeroba); got != 0 {
		t.Errorf("unexpected result: got %d, want 0", got)
	}
	if got := zeroba.LastIndex(nilba); got != 0 {
		t.Errorf("unexpected result: got %d, want 0", got)
	}
	if got := ba.LastIndex(nilba); got != 4 {
		t.Errorf("unexpected result: got %d, want 0", got)
	}
	if got := nilba.LastIndex(ba); got != -1 {
		t.Errorf("unexpected result: got %d, want -1", got)
	}
}

func TestBitArray_AllIndex(t *testing.T) {
	tdt := []string{
		"", "", "0",
		"", "0", "",
		"", "1", "",
		"0", "", "0,1",
		"1", "", "0,1",
		"0", "00", "",
		"1", "11", "",
		"1010", "", "0,1,2,3,4",
		"0000-0000 0000", "0000-0000 000", "0,1",
		"0000-0000 0000", "0000-0000 0000", "0",
		"0000-0000 0000", "0000-0000 0000-0", "",
		"1111-1111 1111", "1111-1111 111", "0,1",
		"1111-1111 1111", "1111-1111 1111-1", "",
		"1111-1111 1011-1101 1111", "1111", "0,1,2,3,4,5,10,15,16",
		"1110-0000 0000-0000 0111", "00000", "3,4,5,6,7,8,9,10,11,12",
		"0000-1111 0000-0000 1111", "1111", "4,16",
		"1111-1111 1111-1111 0000", "0000", "16",
		"1111-1111 1111-1100 0000", "0000-00", "14",
		// TODO: more cases
	}
	chk := func(haystack, needle *bitarray.BitArray, want []int) {
		t.Helper()
		if got := haystack.AllIndex(needle); !reflect.DeepEqual(got, want) {
			t.Error("unexpected result:")
			t.Logf("haystack: %#b", haystack)
			t.Logf("  needle: %#b", needle)
			t.Logf("     got: %v", got)
			t.Logf("    want: %v", want)
		}
	}
	for i := 0; i < len(tdt); i += 3 {
		haystackO := bitarray.MustParse(tdt[i]).ZOptimize()
		haystackE := haystackO.ZExpand()
		needleO := bitarray.MustParse(tdt[i+1]).ZOptimize()
		needleE := needleO.ZExpand()
		wants := strings.Split(tdt[i+2], ",")
		want := make([]int, 0, len(wants))
		for _, ss := range wants {
			if s := strings.TrimSpace(ss); s != "" {
				if v, err := strconv.Atoi(s); err == nil {
					want = append(want, v)
				}
			}
		}
		chk(haystackO, needleO, want)
		chk(haystackO, needleE, want)
		chk(haystackE, needleO, want)
		chk(haystackE, needleE, want)
	}

	var nilba *bitarray.BitArray
	zeroba := bitarray.New()
	ba := bitarray.MustParse("0101")
	if got := nilba.AllIndex(nil); !reflect.DeepEqual(got, []int{0}) { // untyped nil
		t.Errorf("unexpected result: got %v, want [0]", got)
	}
	if got := nilba.AllIndex(nilba); !reflect.DeepEqual(got, []int{0}) {
		t.Errorf("unexpected result: got %v, want [0]", got)
	}
	if got := nilba.AllIndex(zeroba); !reflect.DeepEqual(got, []int{0}) {
		t.Errorf("unexpected result: got %v, want [0]", got)
	}
	if got := zeroba.AllIndex(nilba); !reflect.DeepEqual(got, []int{0}) {
		t.Errorf("unexpected result: got %v, want [0]", got)
	}
	if got := ba.AllIndex(nilba); !reflect.DeepEqual(got, []int{0, 1, 2, 3, 4}) {
		t.Errorf("unexpected result: got %v, want [0 1 2 3 4]", got)
	}
	if got := nilba.AllIndex(ba); !reflect.DeepEqual(got, []int{}) {
		t.Errorf("unexpected result: got %v, want []", got)
	}
}

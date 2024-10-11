// Copyright 2019 Gregory Petrosyan <gregory.petrosyan@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

package rapid

import (
	"math"
	"os"
	"testing"
)

func TestDataVis(t *testing.T) {
	t.Parallel()

	f, err := os.Create("vis-test.html")
	if err != nil {
		t.Fatalf("failed to create vis html file: %v", err)

	}
	defer func() { _ = f.Close() }()

	data := []uint64{
		0,
		0x55,
		0xaa,
		math.MaxUint8,
		0x5555,
		0xaaaa,
		math.MaxUint16,
		0x55555555,
		0xaaaaaaaa,
		math.MaxUint32,
		0x5555555555555555,
		0xaaaaaaaaaaaaaaaa,
		math.MaxUint64,
	}

	groups := []groupInfo{
		{begin: 0, end: 13, label: ""},
		{begin: 1, end: 1 + 3, label: "8-bit"},
		{begin: 3, end: 4, label: "0xff", discard: true},
		{begin: 4, end: 4 + 3, label: "16-bit"},
		{begin: 7, end: 13, label: "big integers"},
		{begin: 7, end: 7 + 3, label: "32-bit"},
		{begin: 10, end: 10 + 3, label: "64-bit"},
	}

	rd := []recordedBits{
		{data: data, groups: groups},
	}

	g := SliceOf(SliceOf(Uint().Filter(func(i uint) bool { return i%2 == 1 }))).Filter(func(s [][]uint) bool { return len(s) > 0 })
	for {
		s := newRandomBitStream(baseSeed(), true)
		_, err := recoverValue(g, newT(nil, s, false, nil))
		if err != nil && !err.isInvalidData() {
			t.Errorf("unexpected error %v", err)
		}

		rd = append(rd, recordedBits{data: s.data, groups: s.groups})

		if err == nil {
			break
		}
	}

	err = visWriteHTML(f, "test", rd)
	if err != nil {
		t.Errorf("visWriteHTML error: %v", err)
	}
}

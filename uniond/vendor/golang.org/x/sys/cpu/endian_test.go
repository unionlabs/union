// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package cpu_test

import (
	"testing"
	"unsafe"

	"golang.org/x/sys/cpu"
)

func TestIsBigEndian(t *testing.T) {
	b := uint16(0xff00)
	want := *(*byte)(unsafe.Pointer(&b)) == 0xff
	if cpu.IsBigEndian != want {
		t.Errorf("IsBigEndian = %t, want %t",
			cpu.IsBigEndian, want)
	}
}

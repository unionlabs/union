// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package unix

import "testing"

func TestSysctlUvmexp(t *testing.T) {
	uvm, err := SysctlUvmexp("vm.uvmexp2")
	if err != nil {
		t.Fatal(err)
	}
	t.Logf("%#v", uvm)
}

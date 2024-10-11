// Copyright 2014-2022 Ulrich Kunitz. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package xz_test

import (
	"bytes"
	"io/ioutil"
	"testing"

	"github.com/ulikunitz/xz"
)

func TestPanic(t *testing.T) {
	data := []byte([]uint8{253, 55, 122, 88, 90, 0, 0, 0, 255, 18, 217, 65, 0, 189, 191, 239, 189, 191, 239, 48})
	t.Logf("%q", string(data))
	t.Logf("0x%02x", data)
	r, err := xz.NewReader(bytes.NewReader(data))
	if err != nil {
		t.Logf("xz.NewReader error %s", err)
		return
	}
	_, err = ioutil.ReadAll(r)
	if err != nil {
		t.Logf("ioutil.ReadAll(r) error %s", err)
		return
	}
}

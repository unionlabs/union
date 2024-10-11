// Copyright 2014-2022 Ulrich Kunitz. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package lzma

import (
	"testing"
)

func TestNewDecoderDict(t *testing.T) {
	if _, err := newDecoderDict(0); err == nil {
		t.Fatalf("no error for zero dictionary capacity")
	}
	if _, err := newDecoderDict(8); err != nil {
		t.Fatalf("error %s", err)
	}
}

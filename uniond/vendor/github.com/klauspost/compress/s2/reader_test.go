// Copyright (c) 2019+ Klaus Post. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package s2

import (
	"bytes"
	"io"
	"testing"
)

func TestLeadingSkippableBlock(t *testing.T) {
	var buf bytes.Buffer
	w := NewWriter(&buf)
	if err := w.AddSkippableBlock(0x80, []byte("skippable block")); err != nil {
		t.Fatalf("w.AddSkippableBlock: %v", err)
	}
	if _, err := w.Write([]byte("some data")); err != nil {
		t.Fatalf("w.Write: %v", err)
	}
	if err := w.Close(); err != nil {
		t.Fatalf("w.Close: %v", err)
	}
	r := NewReader(&buf)
	var sb []byte
	r.SkippableCB(0x80, func(sr io.Reader) error {
		var err error
		sb, err = io.ReadAll(sr)
		return err
	})
	if _, err := r.Read([]byte{}); err != nil {
		t.Errorf("empty read failed: %v", err)
	}
	if !bytes.Equal(sb, []byte("skippable block")) {
		t.Errorf("didn't get correct data from skippable block: %q", string(sb))
	}
	data, err := io.ReadAll(r)
	if err != nil {
		t.Fatalf("r.Read: %v", err)
	}
	if !bytes.Equal(data, []byte("some data")) {
		t.Errorf("didn't get correct compressed data: %q", string(data))
	}
}

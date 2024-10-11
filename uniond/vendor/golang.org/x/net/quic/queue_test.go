// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"context"
	"io"
	"testing"
	"time"
)

func TestQueue(t *testing.T) {
	nonblocking, cancel := context.WithCancel(context.Background())
	cancel()

	q := newQueue[int]()
	if got, err := q.get(nonblocking, nil); err != context.Canceled {
		t.Fatalf("q.get() = %v, %v, want nil, contex.Canceled", got, err)
	}

	if !q.put(1) {
		t.Fatalf("q.put(1) = false, want true")
	}
	if !q.put(2) {
		t.Fatalf("q.put(2) = false, want true")
	}
	if got, err := q.get(nonblocking, nil); got != 1 || err != nil {
		t.Fatalf("q.get() = %v, %v, want 1, nil", got, err)
	}
	if got, err := q.get(nonblocking, nil); got != 2 || err != nil {
		t.Fatalf("q.get() = %v, %v, want 2, nil", got, err)
	}
	if got, err := q.get(nonblocking, nil); err != context.Canceled {
		t.Fatalf("q.get() = %v, %v, want nil, contex.Canceled", got, err)
	}

	go func() {
		time.Sleep(1 * time.Millisecond)
		q.put(3)
	}()
	if got, err := q.get(context.Background(), nil); got != 3 || err != nil {
		t.Fatalf("q.get() = %v, %v, want 3, nil", got, err)
	}

	if !q.put(4) {
		t.Fatalf("q.put(2) = false, want true")
	}
	q.close(io.EOF)
	if got, err := q.get(context.Background(), nil); got != 0 || err != io.EOF {
		t.Fatalf("q.get() = %v, %v, want 0, io.EOF", got, err)
	}
	if q.put(5) {
		t.Fatalf("q.put(5) = true, want false")
	}
}

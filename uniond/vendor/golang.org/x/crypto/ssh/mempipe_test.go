// Copyright 2013 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package ssh

import (
	"io"
	"sync"
	"testing"
)

// An in-memory packetConn. It is safe to call Close and writePacket
// from different goroutines.
type memTransport struct {
	eof        bool
	pending    [][]byte
	write      *memTransport
	writeCount uint64
	sync.Mutex
	*sync.Cond
}

func (t *memTransport) readPacket() ([]byte, error) {
	t.Lock()
	defer t.Unlock()
	for {
		if len(t.pending) > 0 {
			r := t.pending[0]
			t.pending = t.pending[1:]
			return r, nil
		}
		if t.eof {
			return nil, io.EOF
		}
		t.Cond.Wait()
	}
}

func (t *memTransport) closeSelf() error {
	t.Lock()
	defer t.Unlock()
	if t.eof {
		return io.EOF
	}
	t.eof = true
	t.Cond.Broadcast()
	return nil
}

func (t *memTransport) Close() error {
	err := t.write.closeSelf()
	t.closeSelf()
	return err
}

func (t *memTransport) writePacket(p []byte) error {
	t.write.Lock()
	defer t.write.Unlock()
	if t.write.eof {
		return io.EOF
	}
	c := make([]byte, len(p))
	copy(c, p)
	t.write.pending = append(t.write.pending, c)
	t.write.Cond.Signal()
	t.writeCount++
	return nil
}

func (t *memTransport) getWriteCount() uint64 {
	t.write.Lock()
	defer t.write.Unlock()
	return t.writeCount
}

func memPipe() (a, b packetConn) {
	t1 := memTransport{}
	t2 := memTransport{}
	t1.write = &t2
	t2.write = &t1
	t1.Cond = sync.NewCond(&t1.Mutex)
	t2.Cond = sync.NewCond(&t2.Mutex)
	return &t1, &t2
}

func TestMemPipe(t *testing.T) {
	a, b := memPipe()
	if err := a.writePacket([]byte{42}); err != nil {
		t.Fatalf("writePacket: %v", err)
	}
	if wc := a.(*memTransport).getWriteCount(); wc != 1 {
		t.Fatalf("got %v, want 1", wc)
	}
	if err := a.Close(); err != nil {
		t.Fatal("Close: ", err)
	}
	p, err := b.readPacket()
	if err != nil {
		t.Fatal("readPacket: ", err)
	}
	if len(p) != 1 || p[0] != 42 {
		t.Fatalf("got %v, want {42}", p)
	}
	p, err = b.readPacket()
	if err != io.EOF {
		t.Fatalf("got %v, %v, want EOF", p, err)
	}
	if wc := b.(*memTransport).getWriteCount(); wc != 0 {
		t.Fatalf("got %v, want 0", wc)
	}
}

func TestDoubleClose(t *testing.T) {
	a, _ := memPipe()
	err := a.Close()
	if err != nil {
		t.Errorf("Close: %v", err)
	}
	err = a.Close()
	if err != io.EOF {
		t.Errorf("expect EOF on double close.")
	}
}

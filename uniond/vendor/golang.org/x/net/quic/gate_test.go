// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"context"
	"testing"
	"time"
)

func TestGateLockAndUnlock(t *testing.T) {
	g := newGate()
	if set := g.lock(); set {
		t.Errorf("g.lock() of never-locked gate: true, want false")
	}
	unlockedc := make(chan struct{})
	donec := make(chan struct{})
	go func() {
		defer close(donec)
		set := g.lock()
		select {
		case <-unlockedc:
		default:
			t.Errorf("g.lock() succeeded while gate was held")
		}
		if !set {
			t.Errorf("g.lock() of set gate: false, want true")
		}
		g.unlock(false)
	}()
	time.Sleep(1 * time.Millisecond)
	close(unlockedc)
	g.unlock(true)
	<-donec
	if set := g.lock(); set {
		t.Errorf("g.lock() of unset gate: true, want false")
	}
}

func TestGateWaitAndLockContext(t *testing.T) {
	g := newGate()
	// waitAndLock is canceled
	ctx, cancel := context.WithCancel(context.Background())
	go func() {
		time.Sleep(1 * time.Millisecond)
		cancel()
	}()
	if err := g.waitAndLock(ctx, nil); err != context.Canceled {
		t.Errorf("g.waitAndLock() = %v, want context.Canceled", err)
	}
	// waitAndLock succeeds
	set := false
	go func() {
		time.Sleep(1 * time.Millisecond)
		g.lock()
		set = true
		g.unlock(true)
	}()
	if err := g.waitAndLock(context.Background(), nil); err != nil {
		t.Errorf("g.waitAndLock() = %v, want nil", err)
	}
	if !set {
		t.Errorf("g.waitAndLock() returned before gate was set")
	}
	g.unlock(true)
	// waitAndLock succeeds when the gate is set and the context is canceled
	if err := g.waitAndLock(ctx, nil); err != nil {
		t.Errorf("g.waitAndLock() = %v, want nil", err)
	}
}

func TestGateLockIfSet(t *testing.T) {
	g := newGate()
	if locked := g.lockIfSet(); locked {
		t.Errorf("g.lockIfSet() of unset gate = %v, want false", locked)
	}
	g.lock()
	g.unlock(true)
	if locked := g.lockIfSet(); !locked {
		t.Errorf("g.lockIfSet() of set gate = %v, want true", locked)
	}
}

func TestGateUnlockFunc(t *testing.T) {
	g := newGate()
	go func() {
		g.lock()
		defer g.unlockFunc(func() bool { return true })
	}()
	g.waitAndLock(context.Background(), nil)
}

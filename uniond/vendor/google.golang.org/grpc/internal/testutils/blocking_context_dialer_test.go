/*
 *
 * Copyright 2024 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package testutils

import (
	"context"
	"errors"
	"testing"
	"time"
)

const (
	testTimeout      = 5 * time.Second
	testShortTimeout = 10 * time.Millisecond
)

func (s) TestBlockingDialer_NoHold(t *testing.T) {
	lis, err := LocalTCPListener()
	if err != nil {
		t.Fatalf("Failed to listen: %v", err)
	}
	defer lis.Close()

	d := NewBlockingDialer()

	// This should not block.
	ctx, cancel := context.WithTimeout(context.Background(), testTimeout)
	defer cancel()
	conn, err := d.DialContext(ctx, lis.Addr().String())
	if err != nil {
		t.Fatalf("Failed to dial: %v", err)
	}
	conn.Close()
}

func (s) TestBlockingDialer_HoldWaitResume(t *testing.T) {
	lis, err := LocalTCPListener()
	if err != nil {
		t.Fatalf("Failed to listen: %v", err)
	}
	defer lis.Close()

	d := NewBlockingDialer()
	h := d.Hold(lis.Addr().String())

	if h.IsStarted() {
		t.Fatalf("hold.IsStarted() = true, want false")
	}

	done := make(chan struct{})
	ctx, cancel := context.WithTimeout(context.Background(), testTimeout)
	defer cancel()
	go func() {
		defer close(done)
		conn, err := d.DialContext(ctx, lis.Addr().String())
		if err != nil {
			t.Errorf("BlockingDialer.DialContext() got error: %v, want success", err)
			return
		}

		if !h.IsStarted() {
			t.Errorf("hold.IsStarted() = false, want true")
		}
		conn.Close()
	}()

	// This should block until the goroutine above is scheduled.
	if !h.Wait(ctx) {
		t.Fatalf("Timeout while waiting for a connection attempt to %q", h.addr)
	}

	if !h.IsStarted() {
		t.Errorf("hold.IsStarted() = false, want true")
	}

	select {
	case <-done:
		t.Fatalf("Expected dialer to be blocked.")
	case <-time.After(testShortTimeout):
	}

	h.Resume() // Unblock the above goroutine.

	select {
	case <-done:
	case <-ctx.Done():
		t.Errorf("Timeout waiting for connection attempt to resume.")
	}
}

func (s) TestBlockingDialer_HoldWaitFail(t *testing.T) {
	lis, err := LocalTCPListener()
	if err != nil {
		t.Fatalf("Failed to listen: %v", err)
	}
	defer lis.Close()

	d := NewBlockingDialer()
	h := d.Hold(lis.Addr().String())

	wantErr := errors.New("test error")

	dialError := make(chan error)
	ctx, cancel := context.WithTimeout(context.Background(), testTimeout)
	defer cancel()
	go func() {
		_, err := d.DialContext(ctx, lis.Addr().String())
		dialError <- err
	}()

	if !h.Wait(ctx) {
		t.Fatalf("Timeout while waiting for a connection attempt to " + h.addr)
	}
	select {
	case err = <-dialError:
		t.Errorf("DialContext got unblocked with err %v. Want DialContext to still be blocked after Wait()", err)
	case <-time.After(testShortTimeout):
	}

	h.Fail(wantErr)

	select {
	case err = <-dialError:
		if !errors.Is(err, wantErr) {
			t.Errorf("BlockingDialer.DialContext() after Fail(): got error %v, want %v", err, wantErr)
		}
	case <-ctx.Done():
		t.Errorf("Timeout waiting for connection attempt to fail.")
	}
}

func (s) TestBlockingDialer_ContextCanceled(t *testing.T) {
	lis, err := LocalTCPListener()
	if err != nil {
		t.Fatalf("Failed to listen: %v", err)
	}
	defer lis.Close()

	d := NewBlockingDialer()
	h := d.Hold(lis.Addr().String())

	dialErr := make(chan error)
	testCtx, cancel := context.WithTimeout(context.Background(), testTimeout)
	defer cancel()

	ctx, cancel := context.WithCancel(testCtx)
	defer cancel()
	go func() {
		_, err := d.DialContext(ctx, lis.Addr().String())
		dialErr <- err
	}()
	if !h.Wait(testCtx) {
		t.Errorf("Timeout while waiting for a connection attempt to %q", h.addr)
	}

	cancel()

	select {
	case err = <-dialErr:
		if !errors.Is(err, context.Canceled) {
			t.Errorf("BlockingDialer.DialContext() after context cancel: got error %v, want %v", err, context.Canceled)
		}
	case <-testCtx.Done():
		t.Errorf("Timeout while waiting for Wait to return.")
	}

	h.Resume() // noop, just make sure nothing bad happen.
}

func (s) TestBlockingDialer_CancelWait(t *testing.T) {
	lis, err := LocalTCPListener()
	if err != nil {
		t.Fatalf("Failed to listen: %v", err)
	}
	defer lis.Close()

	d := NewBlockingDialer()
	h := d.Hold(lis.Addr().String())

	testCtx, cancel := context.WithTimeout(context.Background(), testTimeout)
	defer cancel()

	ctx, cancel := context.WithCancel(testCtx)
	cancel()
	done := make(chan struct{})
	go func() {
		if h.Wait(ctx) {
			t.Errorf("Expected cancel to return false when context expires")
		}
		done <- struct{}{}
	}()

	select {
	case <-done:
	case <-testCtx.Done():
		t.Errorf("Timeout while waiting for Wait to return.")
	}
}

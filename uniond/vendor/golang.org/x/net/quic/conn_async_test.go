// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"context"
	"errors"
	"fmt"
	"path/filepath"
	"runtime"
	"sync"
)

// asyncTestState permits handling asynchronous operations in a synchronous test.
//
// For example, a test may want to write to a stream and observe that
// STREAM frames are sent with the contents of the write in response
// to MAX_STREAM_DATA frames received from the peer.
// The Stream.Write is an asynchronous operation, but the test is simpler
// if we can start the write, observe the first STREAM frame sent,
// send a MAX_STREAM_DATA frame, observe the next STREAM frame sent, etc.
//
// We do this by instrumenting points where operations can block.
// We start async operations like Write in a goroutine,
// and wait for the operation to either finish or hit a blocking point.
// When the connection event loop is idle, we check a list of
// blocked operations to see if any can be woken.
type asyncTestState struct {
	mu      sync.Mutex
	notify  chan struct{}
	blocked map[*blockedAsync]struct{}
}

// An asyncOp is an asynchronous operation that results in (T, error).
type asyncOp[T any] struct {
	v   T
	err error

	caller     string
	tc         *testConn
	donec      chan struct{}
	cancelFunc context.CancelFunc
}

// cancel cancels the async operation's context, and waits for
// the operation to complete.
func (a *asyncOp[T]) cancel() {
	select {
	case <-a.donec:
		return // already done
	default:
	}
	a.cancelFunc()
	<-a.tc.asyncTestState.notify
	select {
	case <-a.donec:
	default:
		panic(fmt.Errorf("%v: async op failed to finish after being canceled", a.caller))
	}
}

var errNotDone = errors.New("async op is not done")

// result returns the result of the async operation.
// It returns errNotDone if the operation is still in progress.
//
// Note that unlike a traditional async/await, this doesn't block
// waiting for the operation to complete. Since tests have full
// control over the progress of operations, an asyncOp can only
// become done in reaction to the test taking some action.
func (a *asyncOp[T]) result() (v T, err error) {
	a.tc.wait()
	select {
	case <-a.donec:
		return a.v, a.err
	default:
		return v, errNotDone
	}
}

// A blockedAsync is a blocked async operation.
type blockedAsync struct {
	until func() bool   // when this returns true, the operation is unblocked
	donec chan struct{} // closed when the operation is unblocked
}

type asyncContextKey struct{}

// runAsync starts an asynchronous operation.
//
// The function f should call a blocking function such as
// Stream.Write or Conn.AcceptStream and return its result.
// It must use the provided context.
func runAsync[T any](tc *testConn, f func(context.Context) (T, error)) *asyncOp[T] {
	as := &tc.asyncTestState
	if as.notify == nil {
		as.notify = make(chan struct{})
		as.mu.Lock()
		as.blocked = make(map[*blockedAsync]struct{})
		as.mu.Unlock()
	}
	_, file, line, _ := runtime.Caller(1)
	ctx := context.WithValue(context.Background(), asyncContextKey{}, true)
	ctx, cancel := context.WithCancel(ctx)
	a := &asyncOp[T]{
		tc:         tc,
		caller:     fmt.Sprintf("%v:%v", filepath.Base(file), line),
		donec:      make(chan struct{}),
		cancelFunc: cancel,
	}
	go func() {
		a.v, a.err = f(ctx)
		close(a.donec)
		as.notify <- struct{}{}
	}()
	tc.t.Cleanup(func() {
		if _, err := a.result(); err == errNotDone {
			tc.t.Errorf("%v: async operation is still executing at end of test", a.caller)
			a.cancel()
		}
	})
	// Wait for the operation to either finish or block.
	<-as.notify
	tc.wait()
	return a
}

// waitUntil waits for a blocked async operation to complete.
// The operation is complete when the until func returns true.
func (as *asyncTestState) waitUntil(ctx context.Context, until func() bool) error {
	if until() {
		return nil
	}
	if err := ctx.Err(); err != nil {
		// Context has already expired.
		return err
	}
	if ctx.Value(asyncContextKey{}) == nil {
		// Context is not one that we've created, and hasn't expired.
		// This probably indicates that we've tried to perform a
		// blocking operation without using the async test harness here,
		// which may have unpredictable results.
		panic("blocking async point with unexpected Context")
	}
	b := &blockedAsync{
		until: until,
		donec: make(chan struct{}),
	}
	// Record this as a pending blocking operation.
	as.mu.Lock()
	as.blocked[b] = struct{}{}
	as.mu.Unlock()
	// Notify the creator of the operation that we're blocked,
	// and wait to be woken up.
	as.notify <- struct{}{}
	select {
	case <-b.donec:
	case <-ctx.Done():
		return ctx.Err()
	}
	return nil
}

// wakeAsync tries to wake up a blocked async operation.
// It returns true if one was woken, false otherwise.
func (as *asyncTestState) wakeAsync() bool {
	as.mu.Lock()
	var woken *blockedAsync
	for w := range as.blocked {
		if w.until() {
			woken = w
			delete(as.blocked, w)
			break
		}
	}
	as.mu.Unlock()
	if woken == nil {
		return false
	}
	close(woken.donec)
	<-as.notify // must not hold as.mu while blocked here
	return true
}

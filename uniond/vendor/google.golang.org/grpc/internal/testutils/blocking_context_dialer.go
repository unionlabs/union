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
	"net"
	"sync"

	"google.golang.org/grpc/grpclog"
)

var logger = grpclog.Component("testutils")

// BlockingDialer is a dialer that waits for Resume() to be called before
// dialing.
type BlockingDialer struct {
	// mu protects holds.
	mu sync.Mutex
	// holds maps network addresses to a list of holds for that address.
	holds map[string][]*Hold
}

// NewBlockingDialer returns a dialer that waits for Resume() to be called
// before dialing.
func NewBlockingDialer() *BlockingDialer {
	return &BlockingDialer{
		holds: make(map[string][]*Hold),
	}
}

// DialContext implements a context dialer for use with grpc.WithContextDialer
// dial option for a BlockingDialer.
func (d *BlockingDialer) DialContext(ctx context.Context, addr string) (net.Conn, error) {
	d.mu.Lock()
	holds := d.holds[addr]
	if len(holds) == 0 {
		// No hold for this addr.
		d.mu.Unlock()
		return (&net.Dialer{}).DialContext(ctx, "tcp", addr)
	}
	hold := holds[0]
	d.holds[addr] = holds[1:]
	d.mu.Unlock()

	logger.Infof("Hold %p: Intercepted connection attempt to addr %q", hold, addr)
	close(hold.waitCh)
	select {
	case err := <-hold.blockCh:
		if err != nil {
			return nil, err
		}
		return (&net.Dialer{}).DialContext(ctx, "tcp", addr)
	case <-ctx.Done():
		logger.Infof("Hold %p: Connection attempt to addr %q timed out", hold, addr)
		return nil, ctx.Err()
	}
}

// Hold is a handle to a single connection attempt. It can be used to block,
// fail and succeed connection attempts.
type Hold struct {
	// dialer is the dialer that created this hold.
	dialer *BlockingDialer
	// waitCh is closed when a connection attempt is received.
	waitCh chan struct{}
	// blockCh receives the value to return from DialContext for this connection
	// attempt (nil on resume, an error on fail). It receives at most 1 value.
	blockCh chan error
	// addr is the address that this hold is for.
	addr string
}

// Hold blocks the dialer when a connection attempt is made to the given addr.
// A hold is valid for exactly one connection attempt. Multiple holds for an
// addr can be added, and they will apply in the order that the connections are
// attempted.
func (d *BlockingDialer) Hold(addr string) *Hold {
	d.mu.Lock()
	defer d.mu.Unlock()

	h := Hold{dialer: d, blockCh: make(chan error, 1), waitCh: make(chan struct{}), addr: addr}
	d.holds[addr] = append(d.holds[addr], &h)
	return &h
}

// Wait blocks until there is a connection attempt on this Hold, or the context
// expires. Return false if the context has expired, true otherwise.
func (h *Hold) Wait(ctx context.Context) bool {
	logger.Infof("Hold %p: Waiting for a connection attempt to addr %q", h, h.addr)
	select {
	case <-ctx.Done():
		return false
	case <-h.waitCh:
		return true
	}
}

// Resume unblocks the dialer for the given addr. Either Resume or Fail must be
// called at most once on a hold. Otherwise, Resume panics.
func (h *Hold) Resume() {
	logger.Infof("Hold %p: Resuming connection attempt to addr %q", h, h.addr)
	h.blockCh <- nil
	close(h.blockCh)
}

// Fail fails the connection attempt. Either Resume or Fail must be
// called at most once on a hold. Otherwise, Resume panics.
func (h *Hold) Fail(err error) {
	logger.Infof("Hold %p: Failing connection attempt to addr %q", h, h.addr)
	h.blockCh <- err
	close(h.blockCh)
}

// IsStarted returns true if this hold has received a connection attempt.
func (h *Hold) IsStarted() bool {
	select {
	case <-h.waitCh:
		return true
	default:
		return false
	}
}

// Copyright 2023 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package discover

import (
	"net"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/p2p/discover/v5wire"
	"github.com/ethereum/go-ethereum/p2p/enode"
)

// This is a limit for the number of concurrent talk requests.
const maxActiveTalkRequests = 1024

// This is the timeout for acquiring a handler execution slot for a talk request.
// The timeout should be short enough to fit within the request timeout.
const talkHandlerLaunchTimeout = 400 * time.Millisecond

// TalkRequestHandler callback processes a talk request and returns a response.
//
// Note that talk handlers are expected to come up with a response very quickly, within at
// most 200ms or so. If the handler takes longer than that, the remote end may time out
// and wont receive the response.
type TalkRequestHandler func(enode.ID, *net.UDPAddr, []byte) []byte

type talkSystem struct {
	transport *UDPv5

	mutex     sync.Mutex
	handlers  map[string]TalkRequestHandler
	slots     chan struct{}
	lastLog   time.Time
	dropCount int
}

func newTalkSystem(transport *UDPv5) *talkSystem {
	t := &talkSystem{
		transport: transport,
		handlers:  make(map[string]TalkRequestHandler),
		slots:     make(chan struct{}, maxActiveTalkRequests),
	}
	for i := 0; i < cap(t.slots); i++ {
		t.slots <- struct{}{}
	}
	return t
}

// register adds a protocol handler.
func (t *talkSystem) register(protocol string, handler TalkRequestHandler) {
	t.mutex.Lock()
	t.handlers[protocol] = handler
	t.mutex.Unlock()
}

// handleRequest handles a talk request.
func (t *talkSystem) handleRequest(id enode.ID, addr *net.UDPAddr, req *v5wire.TalkRequest) {
	t.mutex.Lock()
	handler, ok := t.handlers[req.Protocol]
	t.mutex.Unlock()

	if !ok {
		resp := &v5wire.TalkResponse{ReqID: req.ReqID}
		t.transport.sendResponse(id, addr, resp)
		return
	}

	// Wait for a slot to become available, then run the handler.
	timeout := time.NewTimer(talkHandlerLaunchTimeout)
	defer timeout.Stop()
	select {
	case <-t.slots:
		go func() {
			defer func() { t.slots <- struct{}{} }()
			respMessage := handler(id, addr, req.Message)
			resp := &v5wire.TalkResponse{ReqID: req.ReqID, Message: respMessage}
			t.transport.sendFromAnotherThread(id, addr, resp)
		}()
	case <-timeout.C:
		// Couldn't get it in time, drop the request.
		if time.Since(t.lastLog) > 5*time.Second {
			log.Warn("Dropping TALKREQ due to overload", "ndrop", t.dropCount)
			t.lastLog = time.Now()
			t.dropCount++
		}
	case <-t.transport.closeCtx.Done():
		// Transport closed, drop the request.
	}
}

// wait blocks until all active requests have finished, and prevents new request
// handlers from being launched.
func (t *talkSystem) wait() {
	for i := 0; i < cap(t.slots); i++ {
		<-t.slots
	}
}

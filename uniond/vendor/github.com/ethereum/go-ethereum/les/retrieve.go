// Copyright 2017 The go-ethereum Authors
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

package les

import (
	"context"
	"errors"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/light"
)

var (
	retryQueue         = time.Millisecond * 100
	hardRequestTimeout = time.Second * 10
)

// retrieveManager is a layer on top of requestDistributor which takes care of
// matching replies by request ID and handles timeouts and resends if necessary.
type retrieveManager struct {
	dist               *requestDistributor
	peers              *serverPeerSet
	softRequestTimeout func() time.Duration

	lock     sync.RWMutex
	sentReqs map[uint64]*sentReq
}

// validatorFunc is a function that processes a reply message
type validatorFunc func(distPeer, *Msg) error

// sentReq represents a request sent and tracked by retrieveManager
type sentReq struct {
	rm       *retrieveManager
	req      *distReq
	id       uint64
	validate validatorFunc

	eventsCh chan reqPeerEvent
	stopCh   chan struct{}
	stopped  bool
	err      error

	lock   sync.RWMutex // protect access to sentTo map
	sentTo map[distPeer]sentReqToPeer

	lastReqQueued bool     // last request has been queued but not sent
	lastReqSentTo distPeer // if not nil then last request has been sent to given peer but not timed out
	reqSrtoCount  int      // number of requests that reached soft (but not hard) timeout
}

// sentReqToPeer notifies the request-from-peer goroutine (tryRequest) about a response
// delivered by the given peer. Only one delivery is allowed per request per peer,
// after which delivered is set to true, the validity of the response is sent on the
// valid channel and no more responses are accepted.
type sentReqToPeer struct {
	delivered, frozen bool
	event             chan int
}

// reqPeerEvent is sent by the request-from-peer goroutine (tryRequest) to the
// request state machine (retrieveLoop) through the eventsCh channel.
type reqPeerEvent struct {
	event int
	peer  distPeer
}

const (
	rpSent = iota // if peer == nil, not sent (no suitable peers)
	rpSoftTimeout
	rpHardTimeout
	rpDeliveredValid
	rpDeliveredInvalid
	rpNotDelivered
)

// newRetrieveManager creates the retrieve manager
func newRetrieveManager(peers *serverPeerSet, dist *requestDistributor, srto func() time.Duration) *retrieveManager {
	return &retrieveManager{
		peers:              peers,
		dist:               dist,
		sentReqs:           make(map[uint64]*sentReq),
		softRequestTimeout: srto,
	}
}

// retrieve sends a request (to multiple peers if necessary) and waits for an answer
// that is delivered through the deliver function and successfully validated by the
// validator callback. It returns when a valid answer is delivered or the context is
// cancelled.
func (rm *retrieveManager) retrieve(ctx context.Context, reqID uint64, req *distReq, val validatorFunc, shutdown chan struct{}) error {
	sentReq := rm.sendReq(reqID, req, val)
	select {
	case <-sentReq.stopCh:
	case <-ctx.Done():
		sentReq.stop(ctx.Err())
	case <-shutdown:
		sentReq.stop(errors.New("client is shutting down"))
	}
	return sentReq.getError()
}

// sendReq starts a process that keeps trying to retrieve a valid answer for a
// request from any suitable peers until stopped or succeeded.
func (rm *retrieveManager) sendReq(reqID uint64, req *distReq, val validatorFunc) *sentReq {
	r := &sentReq{
		rm:       rm,
		req:      req,
		id:       reqID,
		sentTo:   make(map[distPeer]sentReqToPeer),
		stopCh:   make(chan struct{}),
		eventsCh: make(chan reqPeerEvent, 10),
		validate: val,
	}

	canSend := req.canSend
	req.canSend = func(p distPeer) bool {
		// add an extra check to canSend: the request has not been sent to the same peer before
		r.lock.RLock()
		_, sent := r.sentTo[p]
		r.lock.RUnlock()
		return !sent && canSend(p)
	}

	request := req.request
	req.request = func(p distPeer) func() {
		// before actually sending the request, put an entry into the sentTo map
		r.lock.Lock()
		r.sentTo[p] = sentReqToPeer{delivered: false, frozen: false, event: make(chan int, 1)}
		r.lock.Unlock()
		return request(p)
	}
	rm.lock.Lock()
	rm.sentReqs[reqID] = r
	rm.lock.Unlock()

	go r.retrieveLoop()
	return r
}

// deliver is called by the LES protocol manager to deliver reply messages to waiting requests
func (rm *retrieveManager) deliver(peer distPeer, msg *Msg) error {
	rm.lock.RLock()
	req, ok := rm.sentReqs[msg.ReqID]
	rm.lock.RUnlock()

	if ok {
		return req.deliver(peer, msg)
	}
	return errResp(ErrUnexpectedResponse, "reqID = %v", msg.ReqID)
}

// frozen is called by the LES protocol manager when a server has suspended its service and we
// should not expect an answer for the requests already sent there
func (rm *retrieveManager) frozen(peer distPeer) {
	rm.lock.RLock()
	defer rm.lock.RUnlock()

	for _, req := range rm.sentReqs {
		req.frozen(peer)
	}
}

// reqStateFn represents a state of the retrieve loop state machine
type reqStateFn func() reqStateFn

// retrieveLoop is the retrieval state machine event loop
func (r *sentReq) retrieveLoop() {
	go r.tryRequest()
	r.lastReqQueued = true
	state := r.stateRequesting

	for state != nil {
		state = state()
	}

	r.rm.lock.Lock()
	delete(r.rm.sentReqs, r.id)
	r.rm.lock.Unlock()
}

// stateRequesting: a request has been queued or sent recently; when it reaches soft timeout,
// a new request is sent to a new peer
func (r *sentReq) stateRequesting() reqStateFn {
	select {
	case ev := <-r.eventsCh:
		r.update(ev)
		switch ev.event {
		case rpSent:
			if ev.peer == nil {
				// request send failed, no more suitable peers
				if r.waiting() {
					// we are already waiting for sent requests which may succeed so keep waiting
					return r.stateNoMorePeers
				}
				// nothing to wait for, no more peers to ask, return with error
				r.stop(light.ErrNoPeers)
				// no need to go to stopped state because waiting() already returned false
				return nil
			}
		case rpSoftTimeout:
			// last request timed out, try asking a new peer
			go r.tryRequest()
			r.lastReqQueued = true
			return r.stateRequesting
		case rpDeliveredInvalid, rpNotDelivered:
			// if it was the last sent request (set to nil by update) then start a new one
			if !r.lastReqQueued && r.lastReqSentTo == nil {
				go r.tryRequest()
				r.lastReqQueued = true
			}
			return r.stateRequesting
		case rpDeliveredValid:
			r.stop(nil)
			return r.stateStopped
		}
		return r.stateRequesting
	case <-r.stopCh:
		return r.stateStopped
	}
}

// stateNoMorePeers: could not send more requests because no suitable peers are available.
// Peers may become suitable for a certain request later or new peers may appear so we
// keep trying.
func (r *sentReq) stateNoMorePeers() reqStateFn {
	select {
	case <-time.After(retryQueue):
		go r.tryRequest()
		r.lastReqQueued = true
		return r.stateRequesting
	case ev := <-r.eventsCh:
		r.update(ev)
		if ev.event == rpDeliveredValid {
			r.stop(nil)
			return r.stateStopped
		}
		if r.waiting() {
			return r.stateNoMorePeers
		}
		r.stop(light.ErrNoPeers)
		return nil
	case <-r.stopCh:
		return r.stateStopped
	}
}

// stateStopped: request succeeded or cancelled, just waiting for some peers
// to either answer or time out hard
func (r *sentReq) stateStopped() reqStateFn {
	for r.waiting() {
		r.update(<-r.eventsCh)
	}
	return nil
}

// update updates the queued/sent flags and timed out peers counter according to the event
func (r *sentReq) update(ev reqPeerEvent) {
	switch ev.event {
	case rpSent:
		r.lastReqQueued = false
		r.lastReqSentTo = ev.peer
	case rpSoftTimeout:
		r.lastReqSentTo = nil
		r.reqSrtoCount++
	case rpHardTimeout:
		r.reqSrtoCount--
	case rpDeliveredValid, rpDeliveredInvalid, rpNotDelivered:
		if ev.peer == r.lastReqSentTo {
			r.lastReqSentTo = nil
		} else {
			r.reqSrtoCount--
		}
	}
}

// waiting returns true if the retrieval mechanism is waiting for an answer from
// any peer
func (r *sentReq) waiting() bool {
	return r.lastReqQueued || r.lastReqSentTo != nil || r.reqSrtoCount > 0
}

// tryRequest tries to send the request to a new peer and waits for it to either
// succeed or time out if it has been sent. It also sends the appropriate reqPeerEvent
// messages to the request's event channel.
func (r *sentReq) tryRequest() {
	sent := r.rm.dist.queue(r.req)
	var p distPeer
	select {
	case p = <-sent:
	case <-r.stopCh:
		if r.rm.dist.cancel(r.req) {
			p = nil
		} else {
			p = <-sent
		}
	}

	r.eventsCh <- reqPeerEvent{rpSent, p}
	if p == nil {
		return
	}

	hrto := false

	r.lock.RLock()
	s, ok := r.sentTo[p]
	r.lock.RUnlock()
	if !ok {
		panic(nil)
	}

	defer func() {
		pp, ok := p.(*serverPeer)
		if hrto && ok {
			pp.Log().Debug("Request timed out hard")
			if r.rm.peers != nil {
				r.rm.peers.unregister(pp.id)
			}
		}
	}()

	select {
	case event := <-s.event:
		if event == rpNotDelivered {
			r.lock.Lock()
			delete(r.sentTo, p)
			r.lock.Unlock()
		}
		r.eventsCh <- reqPeerEvent{event, p}
		return
	case <-time.After(r.rm.softRequestTimeout()):
		r.eventsCh <- reqPeerEvent{rpSoftTimeout, p}
	}

	select {
	case event := <-s.event:
		if event == rpNotDelivered {
			r.lock.Lock()
			delete(r.sentTo, p)
			r.lock.Unlock()
		}
		r.eventsCh <- reqPeerEvent{event, p}
	case <-time.After(hardRequestTimeout):
		hrto = true
		r.eventsCh <- reqPeerEvent{rpHardTimeout, p}
	}
}

// deliver a reply belonging to this request
func (r *sentReq) deliver(peer distPeer, msg *Msg) error {
	r.lock.Lock()
	defer r.lock.Unlock()

	s, ok := r.sentTo[peer]
	if !ok || s.delivered {
		return errResp(ErrUnexpectedResponse, "reqID = %v", msg.ReqID)
	}
	if s.frozen {
		return nil
	}
	valid := r.validate(peer, msg) == nil
	r.sentTo[peer] = sentReqToPeer{delivered: true, frozen: false, event: s.event}
	if valid {
		s.event <- rpDeliveredValid
	} else {
		s.event <- rpDeliveredInvalid
	}
	if !valid {
		return errResp(ErrInvalidResponse, "reqID = %v", msg.ReqID)
	}
	return nil
}

// frozen sends a "not delivered" event to the peer event channel belonging to the
// given peer if the request has been sent there, causing the state machine to not
// expect an answer and potentially even send the request to the same peer again
// when canSend allows it.
func (r *sentReq) frozen(peer distPeer) {
	r.lock.Lock()
	defer r.lock.Unlock()

	s, ok := r.sentTo[peer]
	if ok && !s.delivered && !s.frozen {
		r.sentTo[peer] = sentReqToPeer{delivered: false, frozen: true, event: s.event}
		s.event <- rpNotDelivered
	}
}

// stop stops the retrieval process and sets an error code that will be returned
// by getError
func (r *sentReq) stop(err error) {
	r.lock.Lock()
	if !r.stopped {
		r.stopped = true
		r.err = err
		close(r.stopCh)
	}
	r.lock.Unlock()
}

// getError returns any retrieval error (either internally generated or set by the
// stop function) after stopCh has been closed
func (r *sentReq) getError() error {
	return r.err
}

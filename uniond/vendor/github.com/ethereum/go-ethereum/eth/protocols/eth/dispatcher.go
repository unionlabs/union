// Copyright 2021 The go-ethereum Authors
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

package eth

import (
	"errors"
	"fmt"
	"time"

	"github.com/ethereum/go-ethereum/p2p"
)

var (
	// errDisconnected is returned if a request is attempted to be made to a peer
	// that was already closed.
	errDisconnected = errors.New("disconnected")

	// errDanglingResponse is returned if a response arrives with a request id
	// which does not match to any existing pending requests.
	errDanglingResponse = errors.New("response to non-existent request")

	// errMismatchingResponseType is returned if the remote peer sent a different
	// packet type as a response to a request than what the local node expected.
	errMismatchingResponseType = errors.New("mismatching response type")
)

// Request is a pending request to allow tracking it and delivering a response
// back to the requester on their chosen channel.
type Request struct {
	peer *Peer  // Peer to which this request belogs for untracking
	id   uint64 // Request ID to match up replies to

	sink   chan *Response // Channel to deliver the response on
	cancel chan struct{}  // Channel to cancel requests ahead of time

	code uint64      // Message code of the request packet
	want uint64      // Message code of the response packet
	data interface{} // Data content of the request packet

	Peer string    // Demultiplexer if cross-peer requests are batched together
	Sent time.Time // Timestamp when the request was sent
}

// Close aborts an in-flight request. Although there's no way to notify the
// remote peer about the cancellation, this method notifies the dispatcher to
// discard any late responses.
func (r *Request) Close() error {
	if r.peer == nil { // Tests mock out the dispatcher, skip internal cancellation
		return nil
	}
	cancelOp := &cancel{
		id:   r.id,
		fail: make(chan error),
	}
	select {
	case r.peer.reqCancel <- cancelOp:
		if err := <-cancelOp.fail; err != nil {
			return err
		}
		close(r.cancel)
		return nil
	case <-r.peer.term:
		return errDisconnected
	}
}

// request is a wrapper around a client Request that has an error channel to
// signal on if sending the request already failed on a network level.
type request struct {
	req  *Request
	fail chan error
}

// cancel is a maintenance type on the dispatcher to stop tracking a pending
// request.
type cancel struct {
	id   uint64 // Request ID to stop tracking
	fail chan error
}

// Response is a reply packet to a previously created request. It is delivered
// on the channel assigned by the requester subsystem and contains the original
// request embedded to allow uniquely matching it caller side.
type Response struct {
	id   uint64    // Request ID to match up this reply to
	recv time.Time // Timestamp when the request was received
	code uint64    // Response packet type to cross validate with request

	Req  *Request      // Original request to cross-reference with
	Res  interface{}   // Remote response for the request query
	Meta interface{}   // Metadata generated locally on the receiver thread
	Time time.Duration // Time it took for the request to be served
	Done chan error    // Channel to signal message handling to the reader
}

// response is a wrapper around a remote Response that has an error channel to
// signal on if processing the response failed.
type response struct {
	res  *Response
	fail chan error
}

// dispatchRequest schedules the request to the dispatcher for tracking and
// network serialization, blocking until it's successfully sent.
//
// The returned Request must either be closed before discarding it, or the reply
// must be waited for and the Response's Done channel signalled.
func (p *Peer) dispatchRequest(req *Request) error {
	reqOp := &request{
		req:  req,
		fail: make(chan error),
	}
	req.cancel = make(chan struct{})
	req.peer = p
	req.Peer = p.id

	select {
	case p.reqDispatch <- reqOp:
		return <-reqOp.fail
	case <-p.term:
		return errDisconnected
	}
}

// dispatchRequest fulfils a pending request and delivers it to the requested
// sink.
func (p *Peer) dispatchResponse(res *Response, metadata func() interface{}) error {
	resOp := &response{
		res:  res,
		fail: make(chan error),
	}
	res.recv = time.Now()
	res.Done = make(chan error)

	select {
	case p.resDispatch <- resOp:
		// Ensure the response is accepted by the dispatcher
		if err := <-resOp.fail; err != nil {
			return nil
		}
		// Request was accepted, run any postprocessing step to generate metadata
		// on the receiver thread, not the sink thread
		if metadata != nil {
			res.Meta = metadata()
		}
		// Deliver the filled out response and wait until it's handled. This
		// path is a bit funky as Go's select has no order, so if a response
		// arrives to an already cancelled request, there's a 50-50% changes
		// of picking on channel or the other. To avoid such cases delivering
		// the packet upstream, check for cancellation first and only after
		// block on delivery.
		select {
		case <-res.Req.cancel:
			return nil // Request cancelled, silently discard response
		default:
			// Request not yet cancelled, attempt to deliver it, but do watch
			// for fresh cancellations too
			select {
			case res.Req.sink <- res:
				return <-res.Done // Response delivered, return any errors
			case <-res.Req.cancel:
				return nil // Request cancelled, silently discard response
			}
		}

	case <-p.term:
		return errDisconnected
	}
}

// dispatcher is a loop that accepts requests from higher layer packages, pushes
// it to the network and tracks and dispatches the responses back to the original
// requester.
func (p *Peer) dispatcher() {
	pending := make(map[uint64]*Request)

	for {
		select {
		case reqOp := <-p.reqDispatch:
			req := reqOp.req
			req.Sent = time.Now()

			requestTracker.Track(p.id, p.version, req.code, req.want, req.id)
			err := p2p.Send(p.rw, req.code, req.data)
			reqOp.fail <- err

			if err == nil {
				pending[req.id] = req
			}

		case cancelOp := <-p.reqCancel:
			// Retrieve the pending request to cancel and short circuit if it
			// has already been serviced and is not available anymore
			req := pending[cancelOp.id]
			if req == nil {
				cancelOp.fail <- nil
				continue
			}
			// Stop tracking the request
			delete(pending, cancelOp.id)
			cancelOp.fail <- nil

		case resOp := <-p.resDispatch:
			res := resOp.res
			res.Req = pending[res.id]

			// Independent if the request exists or not, track this packet
			requestTracker.Fulfil(p.id, p.version, res.code, res.id)

			switch {
			case res.Req == nil:
				// Response arrived with an untracked ID. Since even cancelled
				// requests are tracked until fulfilment, a dangling response
				// means the remote peer implements the protocol badly.
				resOp.fail <- errDanglingResponse

			case res.Req.want != res.code:
				// Response arrived, but it's a different packet type than the
				// one expected by the requester. Either the local code is bad,
				// or the remote peer send junk. In neither cases can we handle
				// the packet.
				resOp.fail <- fmt.Errorf("%w: have %d, want %d", errMismatchingResponseType, res.code, res.Req.want)

			default:
				// All dispatcher checks passed and the response was initialized
				// with the matching request. Signal to the delivery routine that
				// it can wait for a handler response and dispatch the data.
				res.Time = res.recv.Sub(res.Req.Sent)
				resOp.fail <- nil

				// Stop tracking the request, the response dispatcher will deliver
				delete(pending, res.id)
			}

		case <-p.term:
			return
		}
	}
}

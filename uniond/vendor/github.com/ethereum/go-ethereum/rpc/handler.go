// Copyright 2019 The go-ethereum Authors
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

package rpc

import (
	"context"
	"encoding/json"
	"reflect"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/log"
)

// handler handles JSON-RPC messages. There is one handler per connection. Note that
// handler is not safe for concurrent use. Message handling never blocks indefinitely
// because RPCs are processed on background goroutines launched by handler.
//
// The entry points for incoming messages are:
//
//	h.handleMsg(message)
//	h.handleBatch(message)
//
// Outgoing calls use the requestOp struct. Register the request before sending it
// on the connection:
//
//	op := &requestOp{ids: ...}
//	h.addRequestOp(op)
//
// Now send the request, then wait for the reply to be delivered through handleMsg:
//
//	if err := op.wait(...); err != nil {
//		h.removeRequestOp(op) // timeout, etc.
//	}
type handler struct {
	reg                  *serviceRegistry
	unsubscribeCb        *callback
	idgen                func() ID                      // subscription ID generator
	respWait             map[string]*requestOp          // active client requests
	clientSubs           map[string]*ClientSubscription // active client subscriptions
	callWG               sync.WaitGroup                 // pending call goroutines
	rootCtx              context.Context                // canceled by close()
	cancelRoot           func()                         // cancel function for rootCtx
	conn                 jsonWriter                     // where responses will be sent
	log                  log.Logger
	allowSubscribe       bool
	batchRequestLimit    int
	batchResponseMaxSize int

	subLock    sync.Mutex
	serverSubs map[ID]*Subscription
}

type callProc struct {
	ctx       context.Context
	notifiers []*Notifier
}

func newHandler(connCtx context.Context, conn jsonWriter, idgen func() ID, reg *serviceRegistry, batchRequestLimit, batchResponseMaxSize int) *handler {
	rootCtx, cancelRoot := context.WithCancel(connCtx)
	h := &handler{
		reg:                  reg,
		idgen:                idgen,
		conn:                 conn,
		respWait:             make(map[string]*requestOp),
		clientSubs:           make(map[string]*ClientSubscription),
		rootCtx:              rootCtx,
		cancelRoot:           cancelRoot,
		allowSubscribe:       true,
		serverSubs:           make(map[ID]*Subscription),
		log:                  log.Root(),
		batchRequestLimit:    batchRequestLimit,
		batchResponseMaxSize: batchResponseMaxSize,
	}
	if conn.remoteAddr() != "" {
		h.log = h.log.New("conn", conn.remoteAddr())
	}
	h.unsubscribeCb = newCallback(reflect.Value{}, reflect.ValueOf(h.unsubscribe))
	return h
}

// batchCallBuffer manages in progress call messages and their responses during a batch
// call. Calls need to be synchronized between the processing and timeout-triggering
// goroutines.
type batchCallBuffer struct {
	mutex sync.Mutex
	calls []*jsonrpcMessage
	resp  []*jsonrpcMessage
	wrote bool
}

// nextCall returns the next unprocessed message.
func (b *batchCallBuffer) nextCall() *jsonrpcMessage {
	b.mutex.Lock()
	defer b.mutex.Unlock()

	if len(b.calls) == 0 {
		return nil
	}
	// The popping happens in `pushAnswer`. The in progress call is kept
	// so we can return an error for it in case of timeout.
	msg := b.calls[0]
	return msg
}

// pushResponse adds the response to last call returned by nextCall.
func (b *batchCallBuffer) pushResponse(answer *jsonrpcMessage) {
	b.mutex.Lock()
	defer b.mutex.Unlock()

	if answer != nil {
		b.resp = append(b.resp, answer)
	}
	b.calls = b.calls[1:]
}

// write sends the responses.
func (b *batchCallBuffer) write(ctx context.Context, conn jsonWriter) {
	b.mutex.Lock()
	defer b.mutex.Unlock()

	b.doWrite(ctx, conn, false)
}

// respondWithError sends the responses added so far. For the remaining unanswered call
// messages, it responds with the given error.
func (b *batchCallBuffer) respondWithError(ctx context.Context, conn jsonWriter, err error) {
	b.mutex.Lock()
	defer b.mutex.Unlock()

	for _, msg := range b.calls {
		if !msg.isNotification() {
			b.resp = append(b.resp, msg.errorResponse(err))
		}
	}
	b.doWrite(ctx, conn, true)
}

// doWrite actually writes the response.
// This assumes b.mutex is held.
func (b *batchCallBuffer) doWrite(ctx context.Context, conn jsonWriter, isErrorResponse bool) {
	if b.wrote {
		return
	}
	b.wrote = true // can only write once
	if len(b.resp) > 0 {
		conn.writeJSON(ctx, b.resp, isErrorResponse)
	}
}

// handleBatch executes all messages in a batch and returns the responses.
func (h *handler) handleBatch(msgs []*jsonrpcMessage) {
	// Emit error response for empty batches:
	if len(msgs) == 0 {
		h.startCallProc(func(cp *callProc) {
			resp := errorMessage(&invalidRequestError{"empty batch"})
			h.conn.writeJSON(cp.ctx, resp, true)
		})
		return
	}
	// Apply limit on total number of requests.
	if h.batchRequestLimit != 0 && len(msgs) > h.batchRequestLimit {
		h.startCallProc(func(cp *callProc) {
			h.respondWithBatchTooLarge(cp, msgs)
		})
		return
	}

	// Handle non-call messages first.
	// Here we need to find the requestOp that sent the request batch.
	calls := make([]*jsonrpcMessage, 0, len(msgs))
	h.handleResponses(msgs, func(msg *jsonrpcMessage) {
		calls = append(calls, msg)
	})
	if len(calls) == 0 {
		return
	}

	// Process calls on a goroutine because they may block indefinitely:
	h.startCallProc(func(cp *callProc) {
		var (
			timer      *time.Timer
			cancel     context.CancelFunc
			callBuffer = &batchCallBuffer{calls: calls, resp: make([]*jsonrpcMessage, 0, len(calls))}
		)

		cp.ctx, cancel = context.WithCancel(cp.ctx)
		defer cancel()

		// Cancel the request context after timeout and send an error response. Since the
		// currently-running method might not return immediately on timeout, we must wait
		// for the timeout concurrently with processing the request.
		if timeout, ok := ContextRequestTimeout(cp.ctx); ok {
			timer = time.AfterFunc(timeout, func() {
				cancel()
				err := &internalServerError{errcodeTimeout, errMsgTimeout}
				callBuffer.respondWithError(cp.ctx, h.conn, err)
			})
		}

		responseBytes := 0
		for {
			// No need to handle rest of calls if timed out.
			if cp.ctx.Err() != nil {
				break
			}
			msg := callBuffer.nextCall()
			if msg == nil {
				break
			}
			resp := h.handleCallMsg(cp, msg)
			callBuffer.pushResponse(resp)
			if resp != nil && h.batchResponseMaxSize != 0 {
				responseBytes += len(resp.Result)
				if responseBytes > h.batchResponseMaxSize {
					err := &internalServerError{errcodeResponseTooLarge, errMsgResponseTooLarge}
					callBuffer.respondWithError(cp.ctx, h.conn, err)
					break
				}
			}
		}
		if timer != nil {
			timer.Stop()
		}

		h.addSubscriptions(cp.notifiers)
		callBuffer.write(cp.ctx, h.conn)
		for _, n := range cp.notifiers {
			n.activate()
		}
	})
}

func (h *handler) respondWithBatchTooLarge(cp *callProc, batch []*jsonrpcMessage) {
	resp := errorMessage(&invalidRequestError{errMsgBatchTooLarge})
	// Find the first call and add its "id" field to the error.
	// This is the best we can do, given that the protocol doesn't have a way
	// of reporting an error for the entire batch.
	for _, msg := range batch {
		if msg.isCall() {
			resp.ID = msg.ID
			break
		}
	}
	h.conn.writeJSON(cp.ctx, []*jsonrpcMessage{resp}, true)
}

// handleMsg handles a single non-batch message.
func (h *handler) handleMsg(msg *jsonrpcMessage) {
	msgs := []*jsonrpcMessage{msg}
	h.handleResponses(msgs, func(msg *jsonrpcMessage) {
		h.startCallProc(func(cp *callProc) {
			h.handleNonBatchCall(cp, msg)
		})
	})
}

func (h *handler) handleNonBatchCall(cp *callProc, msg *jsonrpcMessage) {
	var (
		responded sync.Once
		timer     *time.Timer
		cancel    context.CancelFunc
	)
	cp.ctx, cancel = context.WithCancel(cp.ctx)
	defer cancel()

	// Cancel the request context after timeout and send an error response. Since the
	// running method might not return immediately on timeout, we must wait for the
	// timeout concurrently with processing the request.
	if timeout, ok := ContextRequestTimeout(cp.ctx); ok {
		timer = time.AfterFunc(timeout, func() {
			cancel()
			responded.Do(func() {
				resp := msg.errorResponse(&internalServerError{errcodeTimeout, errMsgTimeout})
				h.conn.writeJSON(cp.ctx, resp, true)
			})
		})
	}

	answer := h.handleCallMsg(cp, msg)
	if timer != nil {
		timer.Stop()
	}
	h.addSubscriptions(cp.notifiers)
	if answer != nil {
		responded.Do(func() {
			h.conn.writeJSON(cp.ctx, answer, false)
		})
	}
	for _, n := range cp.notifiers {
		n.activate()
	}
}

// close cancels all requests except for inflightReq and waits for
// call goroutines to shut down.
func (h *handler) close(err error, inflightReq *requestOp) {
	h.cancelAllRequests(err, inflightReq)
	h.callWG.Wait()
	h.cancelRoot()
	h.cancelServerSubscriptions(err)
}

// addRequestOp registers a request operation.
func (h *handler) addRequestOp(op *requestOp) {
	for _, id := range op.ids {
		h.respWait[string(id)] = op
	}
}

// removeRequestOps stops waiting for the given request IDs.
func (h *handler) removeRequestOp(op *requestOp) {
	for _, id := range op.ids {
		delete(h.respWait, string(id))
	}
}

// cancelAllRequests unblocks and removes pending requests and active subscriptions.
func (h *handler) cancelAllRequests(err error, inflightReq *requestOp) {
	didClose := make(map[*requestOp]bool)
	if inflightReq != nil {
		didClose[inflightReq] = true
	}

	for id, op := range h.respWait {
		// Remove the op so that later calls will not close op.resp again.
		delete(h.respWait, id)

		if !didClose[op] {
			op.err = err
			close(op.resp)
			didClose[op] = true
		}
	}
	for id, sub := range h.clientSubs {
		delete(h.clientSubs, id)
		sub.close(err)
	}
}

func (h *handler) addSubscriptions(nn []*Notifier) {
	h.subLock.Lock()
	defer h.subLock.Unlock()

	for _, n := range nn {
		if sub := n.takeSubscription(); sub != nil {
			h.serverSubs[sub.ID] = sub
		}
	}
}

// cancelServerSubscriptions removes all subscriptions and closes their error channels.
func (h *handler) cancelServerSubscriptions(err error) {
	h.subLock.Lock()
	defer h.subLock.Unlock()

	for id, s := range h.serverSubs {
		s.err <- err
		close(s.err)
		delete(h.serverSubs, id)
	}
}

// startCallProc runs fn in a new goroutine and starts tracking it in the h.calls wait group.
func (h *handler) startCallProc(fn func(*callProc)) {
	h.callWG.Add(1)
	go func() {
		ctx, cancel := context.WithCancel(h.rootCtx)
		defer h.callWG.Done()
		defer cancel()
		fn(&callProc{ctx: ctx})
	}()
}

// handleResponse processes method call responses.
func (h *handler) handleResponses(batch []*jsonrpcMessage, handleCall func(*jsonrpcMessage)) {
	var resolvedops []*requestOp
	handleResp := func(msg *jsonrpcMessage) {
		op := h.respWait[string(msg.ID)]
		if op == nil {
			h.log.Debug("Unsolicited RPC response", "reqid", idForLog{msg.ID})
			return
		}
		resolvedops = append(resolvedops, op)
		delete(h.respWait, string(msg.ID))

		// For subscription responses, start the subscription if the server
		// indicates success. EthSubscribe gets unblocked in either case through
		// the op.resp channel.
		if op.sub != nil {
			if msg.Error != nil {
				op.err = msg.Error
			} else {
				op.err = json.Unmarshal(msg.Result, &op.sub.subid)
				if op.err == nil {
					go op.sub.run()
					h.clientSubs[op.sub.subid] = op.sub
				}
			}
		}

		if !op.hadResponse {
			op.hadResponse = true
			op.resp <- batch
		}
	}

	for _, msg := range batch {
		start := time.Now()
		switch {
		case msg.isResponse():
			handleResp(msg)
			h.log.Trace("Handled RPC response", "reqid", idForLog{msg.ID}, "duration", time.Since(start))

		case msg.isNotification():
			if strings.HasSuffix(msg.Method, notificationMethodSuffix) {
				h.handleSubscriptionResult(msg)
				continue
			}
			handleCall(msg)

		default:
			handleCall(msg)
		}
	}

	for _, op := range resolvedops {
		h.removeRequestOp(op)
	}
}

// handleSubscriptionResult processes subscription notifications.
func (h *handler) handleSubscriptionResult(msg *jsonrpcMessage) {
	var result subscriptionResult
	if err := json.Unmarshal(msg.Params, &result); err != nil {
		h.log.Debug("Dropping invalid subscription message")
		return
	}
	if h.clientSubs[result.ID] != nil {
		h.clientSubs[result.ID].deliver(result.Result)
	}
}

// handleCallMsg executes a call message and returns the answer.
func (h *handler) handleCallMsg(ctx *callProc, msg *jsonrpcMessage) *jsonrpcMessage {
	start := time.Now()
	switch {
	case msg.isNotification():
		h.handleCall(ctx, msg)
		h.log.Debug("Served "+msg.Method, "duration", time.Since(start))
		return nil

	case msg.isCall():
		resp := h.handleCall(ctx, msg)
		var ctx []interface{}
		ctx = append(ctx, "reqid", idForLog{msg.ID}, "duration", time.Since(start))
		if resp.Error != nil {
			ctx = append(ctx, "err", resp.Error.Message)
			if resp.Error.Data != nil {
				ctx = append(ctx, "errdata", resp.Error.Data)
			}
			h.log.Warn("Served "+msg.Method, ctx...)
		} else {
			h.log.Debug("Served "+msg.Method, ctx...)
		}
		return resp

	case msg.hasValidID():
		return msg.errorResponse(&invalidRequestError{"invalid request"})

	default:
		return errorMessage(&invalidRequestError{"invalid request"})
	}
}

// handleCall processes method calls.
func (h *handler) handleCall(cp *callProc, msg *jsonrpcMessage) *jsonrpcMessage {
	if msg.isSubscribe() {
		return h.handleSubscribe(cp, msg)
	}
	var callb *callback
	if msg.isUnsubscribe() {
		callb = h.unsubscribeCb
	} else {
		callb = h.reg.callback(msg.Method)
	}
	if callb == nil {
		return msg.errorResponse(&methodNotFoundError{method: msg.Method})
	}

	args, err := parsePositionalArguments(msg.Params, callb.argTypes)
	if err != nil {
		return msg.errorResponse(&invalidParamsError{err.Error()})
	}
	start := time.Now()
	answer := h.runMethod(cp.ctx, msg, callb, args)

	// Collect the statistics for RPC calls if metrics is enabled.
	// We only care about pure rpc call. Filter out subscription.
	if callb != h.unsubscribeCb {
		rpcRequestGauge.Inc(1)
		if answer.Error != nil {
			failedRequestGauge.Inc(1)
		} else {
			successfulRequestGauge.Inc(1)
		}
		rpcServingTimer.UpdateSince(start)
		updateServeTimeHistogram(msg.Method, answer.Error == nil, time.Since(start))
	}

	return answer
}

// handleSubscribe processes *_subscribe method calls.
func (h *handler) handleSubscribe(cp *callProc, msg *jsonrpcMessage) *jsonrpcMessage {
	if !h.allowSubscribe {
		return msg.errorResponse(ErrNotificationsUnsupported)
	}

	// Subscription method name is first argument.
	name, err := parseSubscriptionName(msg.Params)
	if err != nil {
		return msg.errorResponse(&invalidParamsError{err.Error()})
	}
	namespace := msg.namespace()
	callb := h.reg.subscription(namespace, name)
	if callb == nil {
		return msg.errorResponse(&subscriptionNotFoundError{namespace, name})
	}

	// Parse subscription name arg too, but remove it before calling the callback.
	argTypes := append([]reflect.Type{stringType}, callb.argTypes...)
	args, err := parsePositionalArguments(msg.Params, argTypes)
	if err != nil {
		return msg.errorResponse(&invalidParamsError{err.Error()})
	}
	args = args[1:]

	// Install notifier in context so the subscription handler can find it.
	n := &Notifier{h: h, namespace: namespace}
	cp.notifiers = append(cp.notifiers, n)
	ctx := context.WithValue(cp.ctx, notifierKey{}, n)

	return h.runMethod(ctx, msg, callb, args)
}

// runMethod runs the Go callback for an RPC method.
func (h *handler) runMethod(ctx context.Context, msg *jsonrpcMessage, callb *callback, args []reflect.Value) *jsonrpcMessage {
	result, err := callb.call(ctx, msg.Method, args)
	if err != nil {
		return msg.errorResponse(err)
	}
	return msg.response(result)
}

// unsubscribe is the callback function for all *_unsubscribe calls.
func (h *handler) unsubscribe(ctx context.Context, id ID) (bool, error) {
	h.subLock.Lock()
	defer h.subLock.Unlock()

	s := h.serverSubs[id]
	if s == nil {
		return false, ErrSubscriptionNotFound
	}
	close(s.err)
	delete(h.serverSubs, id)
	return true, nil
}

type idForLog struct{ json.RawMessage }

func (id idForLog) String() string {
	if s, err := strconv.Unquote(string(id.RawMessage)); err == nil {
		return s
	}
	return string(id.RawMessage)
}

// Copyright 2016 The go-ethereum Authors
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
	"errors"
	"fmt"
	"math/rand"
	"net"
	"net/http"
	"net/http/httptest"
	"os"
	"reflect"
	"runtime"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/davecgh/go-spew/spew"
	"github.com/ethereum/go-ethereum/log"
)

func TestClientRequest(t *testing.T) {
	server := newTestServer()
	defer server.Stop()
	client := DialInProc(server)
	defer client.Close()

	var resp echoResult
	if err := client.Call(&resp, "test_echo", "hello", 10, &echoArgs{"world"}); err != nil {
		t.Fatal(err)
	}
	if !reflect.DeepEqual(resp, echoResult{"hello", 10, &echoArgs{"world"}}) {
		t.Errorf("incorrect result %#v", resp)
	}
}

func TestClientResponseType(t *testing.T) {
	server := newTestServer()
	defer server.Stop()
	client := DialInProc(server)
	defer client.Close()

	if err := client.Call(nil, "test_echo", "hello", 10, &echoArgs{"world"}); err != nil {
		t.Errorf("Passing nil as result should be fine, but got an error: %v", err)
	}
	var resultVar echoResult
	// Note: passing the var, not a ref
	err := client.Call(resultVar, "test_echo", "hello", 10, &echoArgs{"world"})
	if err == nil {
		t.Error("Passing a var as result should be an error")
	}
}

// This test checks calling a method that returns 'null'.
func TestClientNullResponse(t *testing.T) {
	server := newTestServer()
	defer server.Stop()

	client := DialInProc(server)
	defer client.Close()

	var result json.RawMessage
	if err := client.Call(&result, "test_null"); err != nil {
		t.Fatal(err)
	}
	if result == nil {
		t.Fatal("Expected non-nil result")
	}
	if !reflect.DeepEqual(result, json.RawMessage("null")) {
		t.Errorf("Expected null, got %s", result)
	}
}

// This test checks that server-returned errors with code and data come out of Client.Call.
func TestClientErrorData(t *testing.T) {
	server := newTestServer()
	defer server.Stop()
	client := DialInProc(server)
	defer client.Close()

	var resp interface{}
	err := client.Call(&resp, "test_returnError")
	if err == nil {
		t.Fatal("expected error")
	}

	// Check code.
	// The method handler returns an error value which implements the rpc.Error
	// interface, i.e. it has a custom error code. The server returns this error code.
	expectedCode := testError{}.ErrorCode()
	if e, ok := err.(Error); !ok {
		t.Fatalf("client did not return rpc.Error, got %#v", e)
	} else if e.ErrorCode() != expectedCode {
		t.Fatalf("wrong error code %d, want %d", e.ErrorCode(), expectedCode)
	}

	// Check data.
	if e, ok := err.(DataError); !ok {
		t.Fatalf("client did not return rpc.DataError, got %#v", e)
	} else if e.ErrorData() != (testError{}.ErrorData()) {
		t.Fatalf("wrong error data %#v, want %#v", e.ErrorData(), testError{}.ErrorData())
	}
}

func TestClientBatchRequest(t *testing.T) {
	server := newTestServer()
	defer server.Stop()
	client := DialInProc(server)
	defer client.Close()

	batch := []BatchElem{
		{
			Method: "test_echo",
			Args:   []interface{}{"hello", 10, &echoArgs{"world"}},
			Result: new(echoResult),
		},
		{
			Method: "test_echo",
			Args:   []interface{}{"hello2", 11, &echoArgs{"world"}},
			Result: new(echoResult),
		},
		{
			Method: "no_such_method",
			Args:   []interface{}{1, 2, 3},
			Result: new(int),
		},
	}
	if err := client.BatchCall(batch); err != nil {
		t.Fatal(err)
	}
	wantResult := []BatchElem{
		{
			Method: "test_echo",
			Args:   []interface{}{"hello", 10, &echoArgs{"world"}},
			Result: &echoResult{"hello", 10, &echoArgs{"world"}},
		},
		{
			Method: "test_echo",
			Args:   []interface{}{"hello2", 11, &echoArgs{"world"}},
			Result: &echoResult{"hello2", 11, &echoArgs{"world"}},
		},
		{
			Method: "no_such_method",
			Args:   []interface{}{1, 2, 3},
			Result: new(int),
			Error:  &jsonError{Code: -32601, Message: "the method no_such_method does not exist/is not available"},
		},
	}
	if !reflect.DeepEqual(batch, wantResult) {
		t.Errorf("batch results mismatch:\ngot %swant %s", spew.Sdump(batch), spew.Sdump(wantResult))
	}
}

// This checks that, for HTTP connections, the length of batch responses is validated to
// match the request exactly.
func TestClientBatchRequest_len(t *testing.T) {
	b, err := json.Marshal([]jsonrpcMessage{
		{Version: "2.0", ID: json.RawMessage("1"), Result: json.RawMessage(`"0x1"`)},
		{Version: "2.0", ID: json.RawMessage("2"), Result: json.RawMessage(`"0x2"`)},
	})
	if err != nil {
		t.Fatal("failed to encode jsonrpc message:", err)
	}
	s := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, r *http.Request) {
		_, err := rw.Write(b)
		if err != nil {
			t.Error("failed to write response:", err)
		}
	}))
	t.Cleanup(s.Close)

	t.Run("too-few", func(t *testing.T) {
		client, err := Dial(s.URL)
		if err != nil {
			t.Fatal("failed to dial test server:", err)
		}
		defer client.Close()

		batch := []BatchElem{
			{Method: "foo", Result: new(string)},
			{Method: "bar", Result: new(string)},
			{Method: "baz", Result: new(string)},
		}
		ctx, cancelFn := context.WithTimeout(context.Background(), time.Second)
		defer cancelFn()

		if err := client.BatchCallContext(ctx, batch); err != nil {
			t.Fatal("error:", err)
		}
		for i, elem := range batch[:2] {
			if elem.Error != nil {
				t.Errorf("expected no error for batch element %d, got %q", i, elem.Error)
			}
		}
		for i, elem := range batch[2:] {
			if elem.Error != ErrMissingBatchResponse {
				t.Errorf("wrong error %q for batch element %d", elem.Error, i+2)
			}
		}
	})

	t.Run("too-many", func(t *testing.T) {
		client, err := Dial(s.URL)
		if err != nil {
			t.Fatal("failed to dial test server:", err)
		}
		defer client.Close()

		batch := []BatchElem{
			{Method: "foo", Result: new(string)},
		}
		ctx, cancelFn := context.WithTimeout(context.Background(), time.Second)
		defer cancelFn()

		if err := client.BatchCallContext(ctx, batch); err != nil {
			t.Fatal("error:", err)
		}
		for i, elem := range batch[:1] {
			if elem.Error != nil {
				t.Errorf("expected no error for batch element %d, got %q", i, elem.Error)
			}
		}
		for i, elem := range batch[1:] {
			if elem.Error != ErrMissingBatchResponse {
				t.Errorf("wrong error %q for batch element %d", elem.Error, i+2)
			}
		}
	})
}

// This checks that the client can handle the case where the server doesn't
// respond to all requests in a batch.
func TestClientBatchRequestLimit(t *testing.T) {
	server := newTestServer()
	defer server.Stop()
	server.SetBatchLimits(2, 100000)
	client := DialInProc(server)

	batch := []BatchElem{
		{Method: "foo"},
		{Method: "bar"},
		{Method: "baz"},
	}
	err := client.BatchCall(batch)
	if err != nil {
		t.Fatal("unexpected error:", err)
	}

	// Check that the first response indicates an error with batch size.
	var err0 Error
	if !errors.As(batch[0].Error, &err0) {
		t.Log("error zero:", batch[0].Error)
		t.Fatalf("batch elem 0 has wrong error type: %T", batch[0].Error)
	} else {
		if err0.ErrorCode() != -32600 || err0.Error() != errMsgBatchTooLarge {
			t.Fatalf("wrong error on batch elem zero: %v", err0)
		}
	}

	// Check that remaining response batch elements are reported as absent.
	for i, elem := range batch[1:] {
		if elem.Error != ErrMissingBatchResponse {
			t.Fatalf("batch elem %d has unexpected error: %v", i+1, elem.Error)
		}
	}
}

func TestClientNotify(t *testing.T) {
	server := newTestServer()
	defer server.Stop()
	client := DialInProc(server)
	defer client.Close()

	if err := client.Notify(context.Background(), "test_echo", "hello", 10, &echoArgs{"world"}); err != nil {
		t.Fatal(err)
	}
}

// func TestClientCancelInproc(t *testing.T) { testClientCancel("inproc", t) }
func TestClientCancelWebsocket(t *testing.T) { testClientCancel("ws", t) }
func TestClientCancelHTTP(t *testing.T)      { testClientCancel("http", t) }
func TestClientCancelIPC(t *testing.T)       { testClientCancel("ipc", t) }

// This test checks that requests made through CallContext can be canceled by canceling
// the context.
func testClientCancel(transport string, t *testing.T) {
	// These tests take a lot of time, run them all at once.
	// You probably want to run with -parallel 1 or comment out
	// the call to t.Parallel if you enable the logging.
	t.Parallel()

	server := newTestServer()
	defer server.Stop()

	// What we want to achieve is that the context gets canceled
	// at various stages of request processing. The interesting cases
	// are:
	//  - cancel during dial
	//  - cancel while performing a HTTP request
	//  - cancel while waiting for a response
	//
	// To trigger those, the times are chosen such that connections
	// are killed within the deadline for every other call (maxKillTimeout
	// is 2x maxCancelTimeout).
	//
	// Once a connection is dead, there is a fair chance it won't connect
	// successfully because the accept is delayed by 1s.
	maxContextCancelTimeout := 300 * time.Millisecond
	fl := &flakeyListener{
		maxAcceptDelay: 1 * time.Second,
		maxKillTimeout: 600 * time.Millisecond,
	}

	var client *Client
	switch transport {
	case "ws", "http":
		c, hs := httpTestClient(server, transport, fl)
		defer hs.Close()
		client = c
	case "ipc":
		c, l := ipcTestClient(server, fl)
		defer l.Close()
		client = c
	default:
		panic("unknown transport: " + transport)
	}

	// The actual test starts here.
	var (
		wg       sync.WaitGroup
		nreqs    = 10
		ncallers = 10
	)
	caller := func(index int) {
		defer wg.Done()
		for i := 0; i < nreqs; i++ {
			var (
				ctx     context.Context
				cancel  func()
				timeout = time.Duration(rand.Int63n(int64(maxContextCancelTimeout)))
			)
			if index < ncallers/2 {
				// For half of the callers, create a context without deadline
				// and cancel it later.
				ctx, cancel = context.WithCancel(context.Background())
				time.AfterFunc(timeout, cancel)
			} else {
				// For the other half, create a context with a deadline instead. This is
				// different because the context deadline is used to set the socket write
				// deadline.
				ctx, cancel = context.WithTimeout(context.Background(), timeout)
			}

			// Now perform a call with the context.
			// The key thing here is that no call will ever complete successfully.
			err := client.CallContext(ctx, nil, "test_block")
			switch {
			case err == nil:
				_, hasDeadline := ctx.Deadline()
				t.Errorf("no error for call with %v wait time (deadline: %v)", timeout, hasDeadline)
				// default:
				//	t.Logf("got expected error with %v wait time: %v", timeout, err)
			}
			cancel()
		}
	}
	wg.Add(ncallers)
	for i := 0; i < ncallers; i++ {
		go caller(i)
	}
	wg.Wait()
}

func TestClientSubscribeInvalidArg(t *testing.T) {
	server := newTestServer()
	defer server.Stop()
	client := DialInProc(server)
	defer client.Close()

	check := func(shouldPanic bool, arg interface{}) {
		defer func() {
			err := recover()
			if shouldPanic && err == nil {
				t.Errorf("EthSubscribe should've panicked for %#v", arg)
			}
			if !shouldPanic && err != nil {
				t.Errorf("EthSubscribe shouldn't have panicked for %#v", arg)
				buf := make([]byte, 1024*1024)
				buf = buf[:runtime.Stack(buf, false)]
				t.Error(err)
				t.Error(string(buf))
			}
		}()
		client.EthSubscribe(context.Background(), arg, "foo_bar")
	}
	check(true, nil)
	check(true, 1)
	check(true, (chan int)(nil))
	check(true, make(<-chan int))
	check(false, make(chan int))
	check(false, make(chan<- int))
}

func TestClientSubscribe(t *testing.T) {
	server := newTestServer()
	defer server.Stop()
	client := DialInProc(server)
	defer client.Close()

	nc := make(chan int)
	count := 10
	sub, err := client.Subscribe(context.Background(), "nftest", nc, "someSubscription", count, 0)
	if err != nil {
		t.Fatal("can't subscribe:", err)
	}
	for i := 0; i < count; i++ {
		if val := <-nc; val != i {
			t.Fatalf("value mismatch: got %d, want %d", val, i)
		}
	}

	sub.Unsubscribe()
	select {
	case v := <-nc:
		t.Fatal("received value after unsubscribe:", v)
	case err := <-sub.Err():
		if err != nil {
			t.Fatalf("Err returned a non-nil error after explicit unsubscribe: %q", err)
		}
	case <-time.After(1 * time.Second):
		t.Fatalf("subscription not closed within 1s after unsubscribe")
	}
}

// In this test, the connection drops while Subscribe is waiting for a response.
func TestClientSubscribeClose(t *testing.T) {
	server := newTestServer()
	service := &notificationTestService{
		gotHangSubscriptionReq:  make(chan struct{}),
		unblockHangSubscription: make(chan struct{}),
	}
	if err := server.RegisterName("nftest2", service); err != nil {
		t.Fatal(err)
	}

	defer server.Stop()
	client := DialInProc(server)
	defer client.Close()

	var (
		nc   = make(chan int)
		errc = make(chan error, 1)
		sub  *ClientSubscription
		err  error
	)
	go func() {
		sub, err = client.Subscribe(context.Background(), "nftest2", nc, "hangSubscription", 999)
		errc <- err
	}()

	<-service.gotHangSubscriptionReq
	client.Close()
	service.unblockHangSubscription <- struct{}{}

	select {
	case err := <-errc:
		if err == nil {
			t.Errorf("Subscribe returned nil error after Close")
		}
		if sub != nil {
			t.Error("Subscribe returned non-nil subscription after Close")
		}
	case <-time.After(1 * time.Second):
		t.Fatalf("Subscribe did not return within 1s after Close")
	}
}

// This test reproduces https://github.com/ethereum/go-ethereum/issues/17837 where the
// client hangs during shutdown when Unsubscribe races with Client.Close.
func TestClientCloseUnsubscribeRace(t *testing.T) {
	server := newTestServer()
	defer server.Stop()

	for i := 0; i < 20; i++ {
		client := DialInProc(server)
		nc := make(chan int)
		sub, err := client.Subscribe(context.Background(), "nftest", nc, "someSubscription", 3, 1)
		if err != nil {
			t.Fatal(err)
		}
		go client.Close()
		go sub.Unsubscribe()
		select {
		case <-sub.Err():
		case <-time.After(5 * time.Second):
			t.Fatal("subscription not closed within timeout")
		}
	}
}

// unsubscribeRecorder collects the subscription IDs of *_unsubscribe calls.
type unsubscribeRecorder struct {
	ServerCodec
	unsubscribes map[string]bool
}

func (r *unsubscribeRecorder) readBatch() ([]*jsonrpcMessage, bool, error) {
	if r.unsubscribes == nil {
		r.unsubscribes = make(map[string]bool)
	}

	msgs, batch, err := r.ServerCodec.readBatch()
	for _, msg := range msgs {
		if msg.isUnsubscribe() {
			var params []string
			if err := json.Unmarshal(msg.Params, &params); err != nil {
				panic("unsubscribe decode error: " + err.Error())
			}
			r.unsubscribes[params[0]] = true
		}
	}
	return msgs, batch, err
}

// This checks that Client calls the _unsubscribe method on the server when Unsubscribe is
// called on a subscription.
func TestClientSubscriptionUnsubscribeServer(t *testing.T) {
	t.Parallel()

	// Create the server.
	srv := NewServer()
	srv.RegisterName("nftest", new(notificationTestService))
	p1, p2 := net.Pipe()
	recorder := &unsubscribeRecorder{ServerCodec: NewCodec(p1)}
	go srv.ServeCodec(recorder, OptionMethodInvocation|OptionSubscriptions)
	defer srv.Stop()

	// Create the client on the other end of the pipe.
	cfg := new(clientConfig)
	client, _ := newClient(context.Background(), cfg, func(context.Context) (ServerCodec, error) {
		return NewCodec(p2), nil
	})
	defer client.Close()

	// Create the subscription.
	ch := make(chan int)
	sub, err := client.Subscribe(context.Background(), "nftest", ch, "someSubscription", 1, 1)
	if err != nil {
		t.Fatal(err)
	}

	// Unsubscribe and check that unsubscribe was called.
	sub.Unsubscribe()
	if !recorder.unsubscribes[sub.subid] {
		t.Fatal("client did not call unsubscribe method")
	}
	if _, open := <-sub.Err(); open {
		t.Fatal("subscription error channel not closed after unsubscribe")
	}
}

// This checks that the subscribed channel can be closed after Unsubscribe.
// It is the reproducer for https://github.com/ethereum/go-ethereum/issues/22322
func TestClientSubscriptionChannelClose(t *testing.T) {
	t.Parallel()

	var (
		srv     = NewServer()
		httpsrv = httptest.NewServer(srv.WebsocketHandler(nil))
		wsURL   = "ws:" + strings.TrimPrefix(httpsrv.URL, "http:")
	)
	defer srv.Stop()
	defer httpsrv.Close()

	srv.RegisterName("nftest", new(notificationTestService))
	client, _ := Dial(wsURL)

	for i := 0; i < 100; i++ {
		ch := make(chan int, 100)
		sub, err := client.Subscribe(context.Background(), "nftest", ch, "someSubscription", maxClientSubscriptionBuffer-1, 1)
		if err != nil {
			t.Fatal(err)
		}
		sub.Unsubscribe()
		close(ch)
	}
}

// This test checks that Client doesn't lock up when a single subscriber
// doesn't read subscription events.
func TestClientNotificationStorm(t *testing.T) {
	server := newTestServer()
	defer server.Stop()

	doTest := func(count int, wantError bool) {
		client := DialInProc(server)
		defer client.Close()
		ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
		defer cancel()

		// Subscribe on the server. It will start sending many notifications
		// very quickly.
		nc := make(chan int)
		sub, err := client.Subscribe(ctx, "nftest", nc, "someSubscription", count, 0)
		if err != nil {
			t.Fatal("can't subscribe:", err)
		}
		defer sub.Unsubscribe()

		// Process each notification, try to run a call in between each of them.
		for i := 0; i < count; i++ {
			select {
			case val := <-nc:
				if val != i {
					t.Fatalf("(%d/%d) unexpected value %d", i, count, val)
				}
			case err := <-sub.Err():
				if wantError && err != ErrSubscriptionQueueOverflow {
					t.Fatalf("(%d/%d) got error %q, want %q", i, count, err, ErrSubscriptionQueueOverflow)
				} else if !wantError {
					t.Fatalf("(%d/%d) got unexpected error %q", i, count, err)
				}
				return
			}
			var r int
			err := client.CallContext(ctx, &r, "nftest_echo", i)
			if err != nil {
				if !wantError {
					t.Fatalf("(%d/%d) call error: %v", i, count, err)
				}
				return
			}
		}
		if wantError {
			t.Fatalf("didn't get expected error")
		}
	}

	doTest(8000, false)
	doTest(24000, true)
}

func TestClientSetHeader(t *testing.T) {
	var gotHeader bool
	srv := newTestServer()
	httpsrv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Header.Get("test") == "ok" {
			gotHeader = true
		}
		srv.ServeHTTP(w, r)
	}))
	defer httpsrv.Close()
	defer srv.Stop()

	client, err := Dial(httpsrv.URL)
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()

	client.SetHeader("test", "ok")
	if _, err := client.SupportedModules(); err != nil {
		t.Fatal(err)
	}
	if !gotHeader {
		t.Fatal("client did not set custom header")
	}

	// Check that Content-Type can be replaced.
	client.SetHeader("content-type", "application/x-garbage")
	_, err = client.SupportedModules()
	if err == nil {
		t.Fatal("no error for invalid content-type header")
	} else if !strings.Contains(err.Error(), "Unsupported Media Type") {
		t.Fatalf("error is not related to content-type: %q", err)
	}
}

func TestClientHTTP(t *testing.T) {
	server := newTestServer()
	defer server.Stop()

	client, hs := httpTestClient(server, "http", nil)
	defer hs.Close()
	defer client.Close()

	// Launch concurrent requests.
	var (
		results    = make([]echoResult, 100)
		errc       = make(chan error, len(results))
		wantResult = echoResult{"a", 1, new(echoArgs)}
	)
	defer client.Close()
	for i := range results {
		i := i
		go func() {
			errc <- client.Call(&results[i], "test_echo", wantResult.String, wantResult.Int, wantResult.Args)
		}()
	}

	// Wait for all of them to complete.
	timeout := time.NewTimer(5 * time.Second)
	defer timeout.Stop()
	for i := range results {
		select {
		case err := <-errc:
			if err != nil {
				t.Fatal(err)
			}
		case <-timeout.C:
			t.Fatalf("timeout (got %d/%d) results)", i+1, len(results))
		}
	}

	// Check results.
	for i := range results {
		if !reflect.DeepEqual(results[i], wantResult) {
			t.Errorf("result %d mismatch: got %#v, want %#v", i, results[i], wantResult)
		}
	}
}

func TestClientReconnect(t *testing.T) {
	startServer := func(addr string) (*Server, net.Listener) {
		srv := newTestServer()
		l, err := net.Listen("tcp", addr)
		if err != nil {
			t.Fatal("can't listen:", err)
		}
		go http.Serve(l, srv.WebsocketHandler([]string{"*"}))
		return srv, l
	}

	ctx, cancel := context.WithTimeout(context.Background(), 12*time.Second)
	defer cancel()

	// Start a server and corresponding client.
	s1, l1 := startServer("127.0.0.1:0")
	client, err := DialContext(ctx, "ws://"+l1.Addr().String())
	if err != nil {
		t.Fatal("can't dial", err)
	}
	defer client.Close()

	// Perform a call. This should work because the server is up.
	var resp echoResult
	if err := client.CallContext(ctx, &resp, "test_echo", "", 1, nil); err != nil {
		t.Fatal(err)
	}

	// Shut down the server and allow for some cool down time so we can listen on the same
	// address again.
	l1.Close()
	s1.Stop()
	time.Sleep(2 * time.Second)

	// Try calling again. It shouldn't work.
	if err := client.CallContext(ctx, &resp, "test_echo", "", 2, nil); err == nil {
		t.Error("successful call while the server is down")
		t.Logf("resp: %#v", resp)
	}

	// Start it up again and call again. The connection should be reestablished.
	// We spawn multiple calls here to check whether this hangs somehow.
	s2, l2 := startServer(l1.Addr().String())
	defer l2.Close()
	defer s2.Stop()

	start := make(chan struct{})
	errors := make(chan error, 20)
	for i := 0; i < cap(errors); i++ {
		go func() {
			<-start
			var resp echoResult
			errors <- client.CallContext(ctx, &resp, "test_echo", "", 3, nil)
		}()
	}
	close(start)
	errcount := 0
	for i := 0; i < cap(errors); i++ {
		if err = <-errors; err != nil {
			errcount++
		}
	}
	t.Logf("%d errors, last error: %v", errcount, err)
	if errcount > 1 {
		t.Errorf("expected one error after disconnect, got %d", errcount)
	}
}

func httpTestClient(srv *Server, transport string, fl *flakeyListener) (*Client, *httptest.Server) {
	// Create the HTTP server.
	var hs *httptest.Server
	switch transport {
	case "ws":
		hs = httptest.NewUnstartedServer(srv.WebsocketHandler([]string{"*"}))
	case "http":
		hs = httptest.NewUnstartedServer(srv)
	default:
		panic("unknown HTTP transport: " + transport)
	}
	// Wrap the listener if required.
	if fl != nil {
		fl.Listener = hs.Listener
		hs.Listener = fl
	}
	// Connect the client.
	hs.Start()
	client, err := Dial(transport + "://" + hs.Listener.Addr().String())
	if err != nil {
		panic(err)
	}
	return client, hs
}

func ipcTestClient(srv *Server, fl *flakeyListener) (*Client, net.Listener) {
	// Listen on a random endpoint.
	endpoint := fmt.Sprintf("go-ethereum-test-ipc-%d-%d", os.Getpid(), rand.Int63())
	if runtime.GOOS == "windows" {
		endpoint = `\\.\pipe\` + endpoint
	} else {
		endpoint = os.TempDir() + "/" + endpoint
	}
	l, err := ipcListen(endpoint)
	if err != nil {
		panic(err)
	}
	// Connect the listener to the server.
	if fl != nil {
		fl.Listener = l
		l = fl
	}
	go srv.ServeListener(l)
	// Connect the client.
	client, err := Dial(endpoint)
	if err != nil {
		panic(err)
	}
	return client, l
}

// flakeyListener kills accepted connections after a random timeout.
type flakeyListener struct {
	net.Listener
	maxKillTimeout time.Duration
	maxAcceptDelay time.Duration
}

func (l *flakeyListener) Accept() (net.Conn, error) {
	delay := time.Duration(rand.Int63n(int64(l.maxAcceptDelay)))
	time.Sleep(delay)

	c, err := l.Listener.Accept()
	if err == nil {
		timeout := time.Duration(rand.Int63n(int64(l.maxKillTimeout)))
		time.AfterFunc(timeout, func() {
			log.Debug(fmt.Sprintf("killing conn %v after %v", c.LocalAddr(), timeout))
			c.Close()
		})
	}
	return c, err
}

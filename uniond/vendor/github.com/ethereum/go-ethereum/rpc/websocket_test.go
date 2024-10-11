// Copyright 2018 The go-ethereum Authors
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
	"errors"
	"net"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/gorilla/websocket"
)

func TestWebsocketClientHeaders(t *testing.T) {
	t.Parallel()

	endpoint, header, err := wsClientHeaders("wss://testuser:test-PASS_01@example.com:1234", "https://example.com")
	if err != nil {
		t.Fatalf("wsGetConfig failed: %s", err)
	}
	if endpoint != "wss://example.com:1234" {
		t.Fatal("User should have been stripped from the URL")
	}
	if header.Get("authorization") != "Basic dGVzdHVzZXI6dGVzdC1QQVNTXzAx" {
		t.Fatal("Basic auth header is incorrect")
	}
	if header.Get("origin") != "https://example.com" {
		t.Fatal("Origin not set")
	}
}

// This test checks that the server rejects connections from disallowed origins.
func TestWebsocketOriginCheck(t *testing.T) {
	t.Parallel()

	var (
		srv     = newTestServer()
		httpsrv = httptest.NewServer(srv.WebsocketHandler([]string{"http://example.com"}))
		wsURL   = "ws:" + strings.TrimPrefix(httpsrv.URL, "http:")
	)
	defer srv.Stop()
	defer httpsrv.Close()

	client, err := DialWebsocket(context.Background(), wsURL, "http://ekzample.com")
	if err == nil {
		client.Close()
		t.Fatal("no error for wrong origin")
	}
	wantErr := wsHandshakeError{websocket.ErrBadHandshake, "403 Forbidden"}
	if !errors.Is(err, wantErr) {
		t.Fatalf("wrong error for wrong origin: %q", err)
	}

	// Connections without origin header should work.
	client, err = DialWebsocket(context.Background(), wsURL, "")
	if err != nil {
		t.Fatalf("error for empty origin: %v", err)
	}
	client.Close()
}

// This test checks whether calls exceeding the request size limit are rejected.
func TestWebsocketLargeCall(t *testing.T) {
	t.Parallel()

	var (
		srv     = newTestServer()
		httpsrv = httptest.NewServer(srv.WebsocketHandler([]string{"*"}))
		wsURL   = "ws:" + strings.TrimPrefix(httpsrv.URL, "http:")
	)
	defer srv.Stop()
	defer httpsrv.Close()

	client, err := DialWebsocket(context.Background(), wsURL, "")
	if err != nil {
		t.Fatalf("can't dial: %v", err)
	}
	defer client.Close()

	// This call sends slightly less than the limit and should work.
	var result echoResult
	arg := strings.Repeat("x", maxRequestContentLength-200)
	if err := client.Call(&result, "test_echo", arg, 1); err != nil {
		t.Fatalf("valid call didn't work: %v", err)
	}
	if result.String != arg {
		t.Fatal("wrong string echoed")
	}

	// This call sends twice the allowed size and shouldn't work.
	arg = strings.Repeat("x", maxRequestContentLength*2)
	err = client.Call(&result, "test_echo", arg)
	if err == nil {
		t.Fatal("no error for too large call")
	}
}

// This test checks whether the wsMessageSizeLimit option is obeyed.
func TestWebsocketLargeRead(t *testing.T) {
	t.Parallel()

	var (
		srv     = newTestServer()
		httpsrv = httptest.NewServer(srv.WebsocketHandler([]string{"*"}))
		wsURL   = "ws:" + strings.TrimPrefix(httpsrv.URL, "http:")
	)
	defer srv.Stop()
	defer httpsrv.Close()

	testLimit := func(limit *int64) {
		opts := []ClientOption{}
		expLimit := int64(wsDefaultReadLimit)
		if limit != nil && *limit >= 0 {
			opts = append(opts, WithWebsocketMessageSizeLimit(*limit))
			if *limit > 0 {
				expLimit = *limit // 0 means infinite
			}
		}
		client, err := DialOptions(context.Background(), wsURL, opts...)
		if err != nil {
			t.Fatalf("can't dial: %v", err)
		}
		defer client.Close()
		// Remove some bytes for json encoding overhead.
		underLimit := int(expLimit - 128)
		overLimit := expLimit + 1
		if expLimit == wsDefaultReadLimit {
			// No point trying the full 32MB in tests. Just sanity-check that
			// it's not obviously limited.
			underLimit = 1024
			overLimit = -1
		}
		var res string
		// Check under limit
		if err = client.Call(&res, "test_repeat", "A", underLimit); err != nil {
			t.Fatalf("unexpected error with limit %d: %v", expLimit, err)
		}
		if len(res) != underLimit || strings.Count(res, "A") != underLimit {
			t.Fatal("incorrect data")
		}
		// Check over limit
		if overLimit > 0 {
			err = client.Call(&res, "test_repeat", "A", expLimit+1)
			if err == nil || err != websocket.ErrReadLimit {
				t.Fatalf("wrong error with limit %d: %v expecting %v", expLimit, err, websocket.ErrReadLimit)
			}
		}
	}
	ptr := func(v int64) *int64 { return &v }

	testLimit(ptr(-1)) // Should be ignored (use default)
	testLimit(ptr(0))  // Should be ignored (use default)
	testLimit(nil)     // Should be ignored (use default)
	testLimit(ptr(200))
	testLimit(ptr(wsDefaultReadLimit * 2))
}

func TestWebsocketPeerInfo(t *testing.T) {
	var (
		s     = newTestServer()
		ts    = httptest.NewServer(s.WebsocketHandler([]string{"origin.example.com"}))
		tsurl = "ws:" + strings.TrimPrefix(ts.URL, "http:")
	)
	defer s.Stop()
	defer ts.Close()

	ctx := context.Background()
	c, err := DialWebsocket(ctx, tsurl, "origin.example.com")
	if err != nil {
		t.Fatal(err)
	}

	// Request peer information.
	var connInfo PeerInfo
	if err := c.Call(&connInfo, "test_peerInfo"); err != nil {
		t.Fatal(err)
	}

	if connInfo.RemoteAddr == "" {
		t.Error("RemoteAddr not set")
	}
	if connInfo.Transport != "ws" {
		t.Errorf("wrong Transport %q", connInfo.Transport)
	}
	if connInfo.HTTP.UserAgent != "Go-http-client/1.1" {
		t.Errorf("wrong HTTP.UserAgent %q", connInfo.HTTP.UserAgent)
	}
	if connInfo.HTTP.Origin != "origin.example.com" {
		t.Errorf("wrong HTTP.Origin %q", connInfo.HTTP.UserAgent)
	}
}

// This test checks that client handles WebSocket ping frames correctly.
func TestClientWebsocketPing(t *testing.T) {
	t.Parallel()

	var (
		sendPing    = make(chan struct{})
		server      = wsPingTestServer(t, sendPing)
		ctx, cancel = context.WithTimeout(context.Background(), 2*time.Second)
	)
	defer cancel()
	defer server.Shutdown(ctx)

	client, err := DialContext(ctx, "ws://"+server.Addr)
	if err != nil {
		t.Fatalf("client dial error: %v", err)
	}
	defer client.Close()

	resultChan := make(chan int)
	sub, err := client.EthSubscribe(ctx, resultChan, "foo")
	if err != nil {
		t.Fatalf("client subscribe error: %v", err)
	}
	// Note: Unsubscribe is not called on this subscription because the mockup
	// server can't handle the request.

	// Wait for the context's deadline to be reached before proceeding.
	// This is important for reproducing https://github.com/ethereum/go-ethereum/issues/19798
	<-ctx.Done()
	close(sendPing)

	// Wait for the subscription result.
	timeout := time.NewTimer(5 * time.Second)
	defer timeout.Stop()
	for {
		select {
		case err := <-sub.Err():
			t.Error("client subscription error:", err)
		case result := <-resultChan:
			t.Log("client got result:", result)
			return
		case <-timeout.C:
			t.Error("didn't get any result within the test timeout")
			return
		}
	}
}

// This checks that the websocket transport can deal with large messages.
func TestClientWebsocketLargeMessage(t *testing.T) {
	var (
		srv     = NewServer()
		httpsrv = httptest.NewServer(srv.WebsocketHandler(nil))
		wsURL   = "ws:" + strings.TrimPrefix(httpsrv.URL, "http:")
	)
	defer srv.Stop()
	defer httpsrv.Close()

	respLength := wsDefaultReadLimit - 50
	srv.RegisterName("test", largeRespService{respLength})

	c, err := DialWebsocket(context.Background(), wsURL, "")
	if err != nil {
		t.Fatal(err)
	}

	var r string
	if err := c.Call(&r, "test_largeResp"); err != nil {
		t.Fatal("call failed:", err)
	}
	if len(r) != respLength {
		t.Fatalf("response has wrong length %d, want %d", len(r), respLength)
	}
}

// wsPingTestServer runs a WebSocket server which accepts a single subscription request.
// When a value arrives on sendPing, the server sends a ping frame, waits for a matching
// pong and finally delivers a single subscription result.
func wsPingTestServer(t *testing.T, sendPing <-chan struct{}) *http.Server {
	var srv http.Server
	shutdown := make(chan struct{})
	srv.RegisterOnShutdown(func() {
		close(shutdown)
	})
	srv.Handler = http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Upgrade to WebSocket.
		upgrader := websocket.Upgrader{
			CheckOrigin: func(r *http.Request) bool { return true },
		}
		conn, err := upgrader.Upgrade(w, r, nil)
		if err != nil {
			t.Errorf("server WS upgrade error: %v", err)
			return
		}
		defer conn.Close()

		// Handle the connection.
		wsPingTestHandler(t, conn, shutdown, sendPing)
	})

	// Start the server.
	listener, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatal("can't listen:", err)
	}
	srv.Addr = listener.Addr().String()
	go srv.Serve(listener)
	return &srv
}

func wsPingTestHandler(t *testing.T, conn *websocket.Conn, shutdown, sendPing <-chan struct{}) {
	// Canned responses for the eth_subscribe call in TestClientWebsocketPing.
	const (
		subResp   = `{"jsonrpc":"2.0","id":1,"result":"0x00"}`
		subNotify = `{"jsonrpc":"2.0","method":"eth_subscription","params":{"subscription":"0x00","result":1}}`
	)

	// Handle subscribe request.
	if _, _, err := conn.ReadMessage(); err != nil {
		t.Errorf("server read error: %v", err)
		return
	}
	if err := conn.WriteMessage(websocket.TextMessage, []byte(subResp)); err != nil {
		t.Errorf("server write error: %v", err)
		return
	}

	// Read from the connection to process control messages.
	var pongCh = make(chan string)
	conn.SetPongHandler(func(d string) error {
		t.Logf("server got pong: %q", d)
		pongCh <- d
		return nil
	})
	go func() {
		for {
			typ, msg, err := conn.ReadMessage()
			if err != nil {
				return
			}
			t.Logf("server got message (%d): %q", typ, msg)
		}
	}()

	// Write messages.
	var (
		wantPong string
		timer    = time.NewTimer(0)
	)
	defer timer.Stop()
	<-timer.C
	for {
		select {
		case _, open := <-sendPing:
			if !open {
				sendPing = nil
			}
			t.Logf("server sending ping")
			conn.WriteMessage(websocket.PingMessage, []byte("ping"))
			wantPong = "ping"
		case data := <-pongCh:
			if wantPong == "" {
				t.Errorf("unexpected pong")
			} else if data != wantPong {
				t.Errorf("got pong with wrong data %q", data)
			}
			wantPong = ""
			timer.Reset(200 * time.Millisecond)
		case <-timer.C:
			t.Logf("server sending response")
			conn.WriteMessage(websocket.TextMessage, []byte(subNotify))
		case <-shutdown:
			conn.Close()
			return
		}
	}
}

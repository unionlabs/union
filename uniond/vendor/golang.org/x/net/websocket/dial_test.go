// Copyright 2015 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package websocket

import (
	"context"
	"crypto/tls"
	"errors"
	"fmt"
	"log"
	"net"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

// This test depend on Go 1.3+ because in earlier versions the Dialer won't be
// used in TLS connections and a timeout won't be triggered.
func TestDialConfigTLSWithDialer(t *testing.T) {
	tlsServer := httptest.NewTLSServer(nil)
	tlsServerAddr := tlsServer.Listener.Addr().String()
	log.Print("Test TLS WebSocket server listening on ", tlsServerAddr)
	defer tlsServer.Close()
	config, _ := NewConfig(fmt.Sprintf("wss://%s/echo", tlsServerAddr), "http://localhost")
	config.Dialer = &net.Dialer{
		Deadline: time.Now().Add(-time.Minute),
	}
	config.TlsConfig = &tls.Config{
		InsecureSkipVerify: true,
	}
	_, err := DialConfig(config)
	dialerr, ok := err.(*DialError)
	if !ok {
		t.Fatalf("DialError expected, got %#v", err)
	}
	neterr, ok := dialerr.Err.(*net.OpError)
	if !ok {
		t.Fatalf("net.OpError error expected, got %#v", dialerr.Err)
	}
	if !neterr.Timeout() {
		t.Fatalf("expected timeout error, got %#v", neterr)
	}
}

func TestDialConfigTLSWithTimeouts(t *testing.T) {
	t.Parallel()

	finishedRequest := make(chan bool)

	// Context for cancellation
	ctx, cancel := context.WithCancel(context.Background())

	// This is a TLS server that blocks each request indefinitely (and cancels the context)
	tlsServer := httptest.NewTLSServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		cancel()
		<-finishedRequest
	}))

	tlsServerAddr := tlsServer.Listener.Addr().String()
	log.Print("Test TLS WebSocket server listening on ", tlsServerAddr)
	defer tlsServer.Close()
	defer close(finishedRequest)

	config, _ := NewConfig(fmt.Sprintf("wss://%s/echo", tlsServerAddr), "http://localhost")
	config.TlsConfig = &tls.Config{
		InsecureSkipVerify: true,
	}

	_, err := config.DialContext(ctx)
	dialerr, ok := err.(*DialError)
	if !ok {
		t.Fatalf("DialError expected, got %#v", err)
	}
	if !errors.Is(dialerr.Err, context.Canceled) {
		t.Fatalf("context.Canceled error expected, got %#v", dialerr.Err)
	}
}

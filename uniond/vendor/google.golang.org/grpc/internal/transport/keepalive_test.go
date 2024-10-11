/*
 *
 * Copyright 2019 gRPC authors.
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

// This file contains tests related to the following proposals:
// https://github.com/grpc/proposal/blob/master/A8-client-side-keepalive.md
// https://github.com/grpc/proposal/blob/master/A9-server-side-conn-mgt.md
// https://github.com/grpc/proposal/blob/master/A18-tcp-user-timeout.md
package transport

import (
	"context"
	"crypto/tls"
	"crypto/x509"
	"fmt"
	"io"
	"net"
	"os"
	"strings"
	"testing"
	"time"

	"golang.org/x/net/http2"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/internal/channelz"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/syscall"
	"google.golang.org/grpc/keepalive"
	"google.golang.org/grpc/testdata"
)

const defaultTestTimeout = 10 * time.Second

// TestMaxConnectionIdle tests that a server will send GoAway to an idle
// client. An idle client is one who doesn't make any RPC calls for a duration
// of MaxConnectionIdle time.
func (s) TestMaxConnectionIdle(t *testing.T) {
	serverConfig := &ServerConfig{
		KeepaliveParams: keepalive.ServerParameters{
			MaxConnectionIdle: 30 * time.Millisecond,
		},
	}
	server, client, cancel := setUpWithOptions(t, 0, serverConfig, suspended, ConnectOptions{})
	defer func() {
		client.Close(fmt.Errorf("closed manually by test"))
		server.stop()
		cancel()
	}()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	stream, err := client.NewStream(ctx, &CallHdr{})
	if err != nil {
		t.Fatalf("client.NewStream() failed: %v", err)
	}
	client.CloseStream(stream, io.EOF)

	// Verify the server sends a GoAway to client after MaxConnectionIdle timeout
	// kicks in.
	select {
	case <-ctx.Done():
		t.Fatalf("context expired before receiving GoAway from the server.")
	case <-client.GoAway():
		reason, debugMsg := client.GetGoAwayReason()
		if reason != GoAwayNoReason {
			t.Fatalf("GoAwayReason is %v, want %v", reason, GoAwayNoReason)
		}
		if !strings.Contains(debugMsg, "max_idle") {
			t.Fatalf("GoAwayDebugMessage is %v, want %v", debugMsg, "max_idle")
		}
	}
}

// TestMaxConnectionIdleBusyClient tests that a server will not send GoAway to
// a busy client.
func (s) TestMaxConnectionIdleBusyClient(t *testing.T) {
	serverConfig := &ServerConfig{
		KeepaliveParams: keepalive.ServerParameters{
			MaxConnectionIdle: 100 * time.Millisecond,
		},
	}
	server, client, cancel := setUpWithOptions(t, 0, serverConfig, suspended, ConnectOptions{})
	defer func() {
		client.Close(fmt.Errorf("closed manually by test"))
		server.stop()
		cancel()
	}()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	_, err := client.NewStream(ctx, &CallHdr{})
	if err != nil {
		t.Fatalf("client.NewStream() failed: %v", err)
	}

	// Verify the server does not send a GoAway to client even after MaxConnectionIdle
	// timeout kicks in.
	ctx, cancel = context.WithTimeout(context.Background(), time.Second)
	defer cancel()
	select {
	case <-client.GoAway():
		t.Fatalf("A busy client received a GoAway.")
	case <-ctx.Done():
	}
}

// TestMaxConnectionAge tests that a server will send GoAway after a duration
// of MaxConnectionAge.
func (s) TestMaxConnectionAge(t *testing.T) {
	maxConnAge := 100 * time.Millisecond
	serverConfig := &ServerConfig{
		KeepaliveParams: keepalive.ServerParameters{
			MaxConnectionAge:      maxConnAge,
			MaxConnectionAgeGrace: 10 * time.Millisecond,
		},
	}
	server, client, cancel := setUpWithOptions(t, 0, serverConfig, suspended, ConnectOptions{})
	defer func() {
		client.Close(fmt.Errorf("closed manually by test"))
		server.stop()
		cancel()
	}()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := client.NewStream(ctx, &CallHdr{}); err != nil {
		t.Fatalf("client.NewStream() failed: %v", err)
	}

	// Verify the server sends a GoAway to client even after client remains idle
	// for more than MaxConnectionIdle time.
	select {
	case <-client.GoAway():
		reason, debugMsg := client.GetGoAwayReason()
		if reason != GoAwayNoReason {
			t.Fatalf("GoAwayReason is %v, want %v", reason, GoAwayNoReason)
		}
		if !strings.Contains(debugMsg, "max_age") {
			t.Fatalf("GoAwayDebugMessage is %v, want %v", debugMsg, "max_age")
		}
	case <-ctx.Done():
		t.Fatalf("timed out before getting a GoAway from the server.")
	}
}

const (
	defaultWriteBufSize = 32 * 1024
	defaultReadBufSize  = 32 * 1024
)

// TestKeepaliveServerClosesUnresponsiveClient tests that a server closes
// the connection with a client that doesn't respond to keepalive pings.
//
// This test creates a regular net.Conn connection to the server and sends the
// clientPreface and the initial Settings frame, and then remains unresponsive.
func (s) TestKeepaliveServerClosesUnresponsiveClient(t *testing.T) {
	serverConfig := &ServerConfig{
		KeepaliveParams: keepalive.ServerParameters{
			Time:    100 * time.Millisecond,
			Timeout: 10 * time.Millisecond,
		},
	}
	server, client, cancel := setUpWithOptions(t, 0, serverConfig, suspended, ConnectOptions{})
	defer func() {
		client.Close(fmt.Errorf("closed manually by test"))
		server.stop()
		cancel()
	}()

	addr := server.addr()
	conn, err := net.Dial("tcp", addr)
	if err != nil {
		t.Fatalf("net.Dial(tcp, %v) failed: %v", addr, err)
	}
	defer conn.Close()

	if n, err := conn.Write(clientPreface); err != nil || n != len(clientPreface) {
		t.Fatalf("conn.Write(clientPreface) failed: n=%v, err=%v", n, err)
	}
	framer := newFramer(conn, defaultWriteBufSize, defaultReadBufSize, false, 0)
	if err := framer.fr.WriteSettings(http2.Setting{}); err != nil {
		t.Fatal("framer.WriteSettings(http2.Setting{}) failed:", err)
	}
	framer.writer.Flush()

	// We read from the net.Conn till we get an error, which is expected when
	// the server closes the connection as part of the keepalive logic.
	errCh := make(chan error, 1)
	go func() {
		b := make([]byte, 24)
		for {
			if _, err = conn.Read(b); err != nil {
				errCh <- err
				return
			}
		}
	}()

	// Server waits for KeepaliveParams.Time seconds before sending out a ping,
	// and then waits for KeepaliveParams.Timeout for a ping ack.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	select {
	case err := <-errCh:
		if err != io.EOF {
			t.Fatalf("client.Read(_) = _,%v, want io.EOF", err)

		}
	case <-ctx.Done():
		t.Fatalf("Test timed out before server closed the connection.")
	}
}

// TestKeepaliveServerWithResponsiveClient tests that a server doesn't close
// the connection with a client that responds to keepalive pings.
func (s) TestKeepaliveServerWithResponsiveClient(t *testing.T) {
	serverConfig := &ServerConfig{
		KeepaliveParams: keepalive.ServerParameters{
			Time:    100 * time.Millisecond,
			Timeout: 100 * time.Millisecond,
		},
	}
	server, client, cancel := setUpWithOptions(t, 0, serverConfig, suspended, ConnectOptions{})
	defer func() {
		client.Close(fmt.Errorf("closed manually by test"))
		server.stop()
		cancel()
	}()

	// Give keepalive logic some time by sleeping.
	time.Sleep(500 * time.Millisecond)

	if err := checkForHealthyStream(client); err != nil {
		t.Fatalf("Stream creation failed: %v", err)
	}
}

func channelzSubChannel(t *testing.T) *channelz.SubChannel {
	ch := channelz.RegisterChannel(nil, "test chan")
	sc := channelz.RegisterSubChannel(ch, "test subchan")
	t.Cleanup(func() {
		channelz.RemoveEntry(sc.ID)
		channelz.RemoveEntry(ch.ID)
	})
	return sc
}

// TestKeepaliveClientClosesUnresponsiveServer creates a server which does not
// respond to keepalive pings, and makes sure that the client closes the
// transport once the keepalive logic kicks in. Here, we set the
// `PermitWithoutStream` parameter to true which ensures that the keepalive
// logic is running even without any active streams.
func (s) TestKeepaliveClientClosesUnresponsiveServer(t *testing.T) {
	connCh := make(chan net.Conn, 1)
	copts := ConnectOptions{
		ChannelzParent: channelzSubChannel(t),
		KeepaliveParams: keepalive.ClientParameters{
			Time:                10 * time.Millisecond,
			Timeout:             10 * time.Millisecond,
			PermitWithoutStream: true,
		},
	}
	client, cancel := setUpWithNoPingServer(t, copts, connCh)
	defer cancel()
	defer client.Close(fmt.Errorf("closed manually by test"))

	conn, ok := <-connCh
	if !ok {
		t.Fatalf("Server didn't return connection object")
	}
	defer conn.Close()

	if err := pollForStreamCreationError(client); err != nil {
		t.Fatal(err)
	}
}

// TestKeepaliveClientOpenWithUnresponsiveServer creates a server which does
// not respond to keepalive pings, and makes sure that the client does not
// close the transport. Here, we do not set the `PermitWithoutStream` parameter
// to true which ensures that the keepalive logic is turned off without any
// active streams, and therefore the transport stays open.
func (s) TestKeepaliveClientOpenWithUnresponsiveServer(t *testing.T) {
	connCh := make(chan net.Conn, 1)
	copts := ConnectOptions{
		ChannelzParent: channelzSubChannel(t),
		KeepaliveParams: keepalive.ClientParameters{
			Time:    10 * time.Millisecond,
			Timeout: 10 * time.Millisecond,
		},
	}
	client, cancel := setUpWithNoPingServer(t, copts, connCh)
	defer cancel()
	defer client.Close(fmt.Errorf("closed manually by test"))

	conn, ok := <-connCh
	if !ok {
		t.Fatalf("Server didn't return connection object")
	}
	defer conn.Close()

	// Give keepalive some time.
	time.Sleep(500 * time.Millisecond)

	if err := checkForHealthyStream(client); err != nil {
		t.Fatalf("Stream creation failed: %v", err)
	}
}

// TestKeepaliveClientClosesWithActiveStreams creates a server which does not
// respond to keepalive pings, and makes sure that the client closes the
// transport even when there is an active stream.
func (s) TestKeepaliveClientClosesWithActiveStreams(t *testing.T) {
	connCh := make(chan net.Conn, 1)
	copts := ConnectOptions{
		ChannelzParent: channelzSubChannel(t),
		KeepaliveParams: keepalive.ClientParameters{
			Time:    500 * time.Millisecond,
			Timeout: 500 * time.Millisecond,
		},
	}
	// TODO(i/6099): Setup a server which can ping and no-ping based on a flag to
	// reduce the flakiness in this test.
	client, cancel := setUpWithNoPingServer(t, copts, connCh)
	defer cancel()
	defer client.Close(fmt.Errorf("closed manually by test"))

	conn, ok := <-connCh
	if !ok {
		t.Fatalf("Server didn't return connection object")
	}
	defer conn.Close()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	// Create a stream, but send no data on it.
	if _, err := client.NewStream(ctx, &CallHdr{}); err != nil {
		t.Fatalf("Stream creation failed: %v", err)
	}

	if err := pollForStreamCreationError(client); err != nil {
		t.Fatal(err)
	}
}

// TestKeepaliveClientStaysHealthyWithResponsiveServer creates a server which
// responds to keepalive pings, and makes sure than a client transport stays
// healthy without any active streams.
func (s) TestKeepaliveClientStaysHealthyWithResponsiveServer(t *testing.T) {
	server, client, cancel := setUpWithOptions(t, 0,
		&ServerConfig{
			KeepalivePolicy: keepalive.EnforcementPolicy{
				MinTime:             50 * time.Millisecond,
				PermitWithoutStream: true,
			},
		},
		normal,
		ConnectOptions{
			KeepaliveParams: keepalive.ClientParameters{
				Time:                55 * time.Millisecond,
				Timeout:             time.Second,
				PermitWithoutStream: true,
			}})
	defer func() {
		client.Close(fmt.Errorf("closed manually by test"))
		server.stop()
		cancel()
	}()

	// Give keepalive some time.
	time.Sleep(500 * time.Millisecond)

	if err := checkForHealthyStream(client); err != nil {
		t.Fatalf("Stream creation failed: %v", err)
	}
}

// TestKeepaliveClientFrequency creates a server which expects at most 1 client
// ping for every 100 ms, while the client is configured to send a ping
// every 50 ms. So, this configuration should end up with the client
// transport being closed. But we had a bug wherein the client was sending one
// ping every [Time+Timeout] instead of every [Time] period, and this test
// explicitly makes sure the fix works and the client sends a ping every [Time]
// period.
func (s) TestKeepaliveClientFrequency(t *testing.T) {
	grpctest.TLogger.ExpectError("Client received GoAway with error code ENHANCE_YOUR_CALM and debug data equal to ASCII \"too_many_pings\"")

	serverConfig := &ServerConfig{
		KeepalivePolicy: keepalive.EnforcementPolicy{
			MinTime:             100 * time.Millisecond,
			PermitWithoutStream: true,
		},
	}
	clientOptions := ConnectOptions{
		KeepaliveParams: keepalive.ClientParameters{
			Time:                50 * time.Millisecond,
			Timeout:             time.Second,
			PermitWithoutStream: true,
		},
	}
	server, client, cancel := setUpWithOptions(t, 0, serverConfig, normal, clientOptions)
	defer func() {
		client.Close(fmt.Errorf("closed manually by test"))
		server.stop()
		cancel()
	}()

	if err := waitForGoAwayTooManyPings(client); err != nil {
		t.Fatal(err)
	}
}

// TestKeepaliveServerEnforcementWithAbusiveClientNoRPC verifies that the
// server closes a client transport when it sends too many keepalive pings
// (when there are no active streams), based on the configured
// EnforcementPolicy.
func (s) TestKeepaliveServerEnforcementWithAbusiveClientNoRPC(t *testing.T) {
	grpctest.TLogger.ExpectError("Client received GoAway with error code ENHANCE_YOUR_CALM and debug data equal to ASCII \"too_many_pings\"")

	serverConfig := &ServerConfig{
		KeepalivePolicy: keepalive.EnforcementPolicy{
			MinTime: time.Second,
		},
	}
	clientOptions := ConnectOptions{
		KeepaliveParams: keepalive.ClientParameters{
			Time:                20 * time.Millisecond,
			Timeout:             100 * time.Millisecond,
			PermitWithoutStream: true,
		},
	}
	server, client, cancel := setUpWithOptions(t, 0, serverConfig, normal, clientOptions)
	defer func() {
		client.Close(fmt.Errorf("closed manually by test"))
		server.stop()
		cancel()
	}()

	if err := waitForGoAwayTooManyPings(client); err != nil {
		t.Fatal(err)
	}
}

// TestKeepaliveServerEnforcementWithAbusiveClientWithRPC verifies that the
// server closes a client transport when it sends too many keepalive pings
// (even when there is an active stream), based on the configured
// EnforcementPolicy.
func (s) TestKeepaliveServerEnforcementWithAbusiveClientWithRPC(t *testing.T) {
	grpctest.TLogger.ExpectError("Client received GoAway with error code ENHANCE_YOUR_CALM and debug data equal to ASCII \"too_many_pings\"")

	serverConfig := &ServerConfig{
		KeepalivePolicy: keepalive.EnforcementPolicy{
			MinTime: time.Second,
		},
	}
	clientOptions := ConnectOptions{
		KeepaliveParams: keepalive.ClientParameters{
			Time:    50 * time.Millisecond,
			Timeout: 100 * time.Millisecond,
		},
	}
	server, client, cancel := setUpWithOptions(t, 0, serverConfig, suspended, clientOptions)
	defer func() {
		client.Close(fmt.Errorf("closed manually by test"))
		server.stop()
		cancel()
	}()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := client.NewStream(ctx, &CallHdr{}); err != nil {
		t.Fatalf("Stream creation failed: %v", err)
	}

	if err := waitForGoAwayTooManyPings(client); err != nil {
		t.Fatal(err)
	}
}

// TestKeepaliveServerEnforcementWithObeyingClientNoRPC verifies that the
// server does not close a client transport (with no active streams) which
// sends keepalive pings in accordance to the configured keepalive
// EnforcementPolicy.
func (s) TestKeepaliveServerEnforcementWithObeyingClientNoRPC(t *testing.T) {
	serverConfig := &ServerConfig{
		KeepalivePolicy: keepalive.EnforcementPolicy{
			MinTime:             40 * time.Millisecond,
			PermitWithoutStream: true,
		},
	}
	clientOptions := ConnectOptions{
		KeepaliveParams: keepalive.ClientParameters{
			Time:                50 * time.Millisecond,
			Timeout:             time.Second,
			PermitWithoutStream: true,
		},
	}
	server, client, cancel := setUpWithOptions(t, 0, serverConfig, normal, clientOptions)
	defer func() {
		client.Close(fmt.Errorf("closed manually by test"))
		server.stop()
		cancel()
	}()

	// Sleep for client to send ~10 keepalive pings.
	time.Sleep(500 * time.Millisecond)

	// Verify that the server does not close the client transport.
	if err := checkForHealthyStream(client); err != nil {
		t.Fatalf("Stream creation failed: %v", err)
	}
}

// TestKeepaliveServerEnforcementWithObeyingClientWithRPC verifies that the
// server does not close a client transport (with active streams) which
// sends keepalive pings in accordance to the configured keepalive
// EnforcementPolicy.
func (s) TestKeepaliveServerEnforcementWithObeyingClientWithRPC(t *testing.T) {
	serverConfig := &ServerConfig{
		KeepalivePolicy: keepalive.EnforcementPolicy{
			MinTime: 40 * time.Millisecond,
		},
	}
	clientOptions := ConnectOptions{
		KeepaliveParams: keepalive.ClientParameters{
			Time: 50 * time.Millisecond,
		},
	}
	server, client, cancel := setUpWithOptions(t, 0, serverConfig, suspended, clientOptions)
	defer func() {
		client.Close(fmt.Errorf("closed manually by test"))
		server.stop()
		cancel()
	}()

	if err := checkForHealthyStream(client); err != nil {
		t.Fatalf("Stream creation failed: %v", err)
	}

	// Give keepalive enough time.
	time.Sleep(500 * time.Millisecond)

	if err := checkForHealthyStream(client); err != nil {
		t.Fatalf("Stream creation failed: %v", err)
	}
}

// TestKeepaliveServerEnforcementWithDormantKeepaliveOnClient verifies that the
// server does not closes a client transport, which has been configured to send
// more pings than allowed by the server's EnforcementPolicy. This client
// transport does not have any active streams and `PermitWithoutStream` is set
// to false. This should ensure that the keepalive functionality on the client
// side enters a dormant state.
func (s) TestKeepaliveServerEnforcementWithDormantKeepaliveOnClient(t *testing.T) {
	serverConfig := &ServerConfig{
		KeepalivePolicy: keepalive.EnforcementPolicy{
			MinTime: 100 * time.Millisecond,
		},
	}
	clientOptions := ConnectOptions{
		KeepaliveParams: keepalive.ClientParameters{
			Time:    10 * time.Millisecond,
			Timeout: 10 * time.Millisecond,
		},
	}
	server, client, cancel := setUpWithOptions(t, 0, serverConfig, normal, clientOptions)
	defer func() {
		client.Close(fmt.Errorf("closed manually by test"))
		server.stop()
		cancel()
	}()

	// No active streams on the client. Give keepalive enough time.
	time.Sleep(500 * time.Millisecond)

	if err := checkForHealthyStream(client); err != nil {
		t.Fatalf("Stream creation failed: %v", err)
	}
}

// TestTCPUserTimeout tests that the TCP_USER_TIMEOUT socket option is set to
// the keepalive timeout, as detailed in proposal A18.
func (s) TestTCPUserTimeout(t *testing.T) {
	tests := []struct {
		tls               bool
		time              time.Duration
		timeout           time.Duration
		clientWantTimeout time.Duration
		serverWantTimeout time.Duration
	}{
		{
			false,
			10 * time.Second,
			10 * time.Second,
			10 * 1000 * time.Millisecond,
			10 * 1000 * time.Millisecond,
		},
		{
			false,
			0,
			0,
			0,
			20 * 1000 * time.Millisecond,
		},
		{
			false,
			infinity,
			infinity,
			0,
			0,
		},
		{
			true,
			10 * time.Second,
			10 * time.Second,
			10 * 1000 * time.Millisecond,
			10 * 1000 * time.Millisecond,
		},
		{
			true,
			0,
			0,
			0,
			20 * 1000 * time.Millisecond,
		},
		{
			true,
			infinity,
			infinity,
			0,
			0,
		},
	}
	for _, tt := range tests {
		sopts := &ServerConfig{
			KeepaliveParams: keepalive.ServerParameters{
				Time:    tt.time,
				Timeout: tt.timeout,
			},
		}

		copts := ConnectOptions{
			KeepaliveParams: keepalive.ClientParameters{
				Time:    tt.time,
				Timeout: tt.timeout,
			},
		}

		if tt.tls {
			copts.TransportCredentials = makeTLSCreds(t, "x509/client1_cert.pem", "x509/client1_key.pem", "x509/server_ca_cert.pem")
			sopts.Credentials = makeTLSCreds(t, "x509/server1_cert.pem", "x509/server1_key.pem", "x509/client_ca_cert.pem")

		}

		server, client, cancel := setUpWithOptions(
			t,
			0,
			sopts,
			normal,
			copts,
		)
		defer func() {
			client.Close(fmt.Errorf("closed manually by test"))
			server.stop()
			cancel()
		}()

		var sc *http2Server
		var srawConn net.Conn
		// Wait until the server transport is setup.
		for {
			server.mu.Lock()
			if len(server.conns) == 0 {
				server.mu.Unlock()
				time.Sleep(time.Millisecond)
				continue
			}
			for k := range server.conns {
				var ok bool
				sc, ok = k.(*http2Server)
				if !ok {
					t.Fatalf("Failed to convert %v to *http2Server", k)
				}
				srawConn = server.conns[k]
			}
			server.mu.Unlock()
			break
		}

		ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
		defer cancel()
		stream, err := client.NewStream(ctx, &CallHdr{})
		if err != nil {
			t.Fatalf("client.NewStream() failed: %v", err)
		}
		client.CloseStream(stream, io.EOF)

		// check client TCP user timeout only when non TLS
		// TODO : find a way to get the underlying conn for client when TLS
		if !tt.tls {
			cltOpt, err := syscall.GetTCPUserTimeout(client.conn)
			if err != nil {
				t.Fatalf("syscall.GetTCPUserTimeout() failed: %v", err)
			}
			if cltOpt < 0 {
				t.Skipf("skipping test on unsupported environment")
			}
			if gotTimeout := time.Duration(cltOpt) * time.Millisecond; gotTimeout != tt.clientWantTimeout {
				t.Fatalf("syscall.GetTCPUserTimeout() = %d, want %d", gotTimeout, tt.clientWantTimeout)
			}
		}
		scConn := sc.conn
		if tt.tls {
			if _, ok := sc.conn.(*net.TCPConn); ok {
				t.Fatalf("sc.conn is should have wrapped conn with TLS")
			}
			scConn = srawConn
		}
		// verify the type of scConn (on which TCP user timeout will be got)
		if _, ok := scConn.(*net.TCPConn); !ok {
			t.Fatalf("server underlying conn is of type %T, want net.TCPConn", scConn)
		}
		srvOpt, err := syscall.GetTCPUserTimeout(scConn)
		if err != nil {
			t.Fatalf("syscall.GetTCPUserTimeout() failed: %v", err)
		}
		if gotTimeout := time.Duration(srvOpt) * time.Millisecond; gotTimeout != tt.serverWantTimeout {
			t.Fatalf("syscall.GetTCPUserTimeout() = %d, want %d", gotTimeout, tt.serverWantTimeout)
		}

	}
}

func makeTLSCreds(t *testing.T, certPath, keyPath, rootsPath string) credentials.TransportCredentials {
	cert, err := tls.LoadX509KeyPair(testdata.Path(certPath), testdata.Path(keyPath))
	if err != nil {
		t.Fatalf("tls.LoadX509KeyPair(%q, %q) failed: %v", certPath, keyPath, err)
	}
	b, err := os.ReadFile(testdata.Path(rootsPath))
	if err != nil {
		t.Fatalf("os.ReadFile(%q) failed: %v", rootsPath, err)
	}
	roots := x509.NewCertPool()
	if !roots.AppendCertsFromPEM(b) {
		t.Fatal("failed to append certificates")
	}
	return credentials.NewTLS(&tls.Config{
		Certificates:       []tls.Certificate{cert},
		RootCAs:            roots,
		InsecureSkipVerify: true,
	})
}

// checkForHealthyStream attempts to create a stream and return error if any.
// The stream created is closed right after to avoid any leakages.
func checkForHealthyStream(client *http2Client) error {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	stream, err := client.NewStream(ctx, &CallHdr{})
	client.CloseStream(stream, err)
	return err
}

func pollForStreamCreationError(client *http2Client) error {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	for {
		if _, err := client.NewStream(ctx, &CallHdr{}); err != nil {
			break
		}
		time.Sleep(50 * time.Millisecond)
	}
	if ctx.Err() != nil {
		return fmt.Errorf("test timed out before stream creation returned an error")
	}
	return nil
}

// waitForGoAwayTooManyPings waits for client to receive a GoAwayTooManyPings
// from server. It also asserts that stream creation fails after receiving a
// GoAway.
func waitForGoAwayTooManyPings(client *http2Client) error {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	select {
	case <-client.GoAway():
		if reason, _ := client.GetGoAwayReason(); reason != GoAwayTooManyPings {
			return fmt.Errorf("goAwayReason is %v, want %v", reason, GoAwayTooManyPings)
		}
	case <-ctx.Done():
		return fmt.Errorf("test timed out before getting GoAway with reason:GoAwayTooManyPings from server")
	}

	if _, err := client.NewStream(ctx, &CallHdr{}); err == nil {
		return fmt.Errorf("stream creation succeeded after receiving a GoAway from the server")
	}
	return nil
}

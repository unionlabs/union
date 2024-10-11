/*
 *
 * Copyright 2014 gRPC authors.
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

package grpc

import (
	"context"
	"errors"
	"fmt"
	"math"
	"net"
	"strings"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"golang.org/x/net/http2"
	"google.golang.org/grpc/backoff"
	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"
	internalbackoff "google.golang.org/grpc/internal/backoff"
	"google.golang.org/grpc/internal/grpcsync"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/transport"
	"google.golang.org/grpc/keepalive"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/resolver/manual"
	"google.golang.org/grpc/serviceconfig"
	"google.golang.org/grpc/testdata"
)

const (
	defaultTestTimeout         = 10 * time.Second
	stateRecordingBalancerName = "state_recording_balancer"
)

var testBalancerBuilder = newStateRecordingBalancerBuilder()

func init() {
	balancer.Register(testBalancerBuilder)
}

func parseCfg(r *manual.Resolver, s string) *serviceconfig.ParseResult {
	scpr := r.CC.ParseServiceConfig(s)
	if scpr.Err != nil {
		panic(fmt.Sprintf("Error parsing config %q: %v", s, scpr.Err))
	}
	return scpr
}

func (s) TestDialWithTimeout(t *testing.T) {
	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Error while listening. Err: %v", err)
	}
	defer lis.Close()
	lisAddr := resolver.Address{Addr: lis.Addr().String()}
	lisDone := make(chan struct{})
	dialDone := make(chan struct{})
	// 1st listener accepts the connection and then does nothing
	go func() {
		defer close(lisDone)
		conn, err := lis.Accept()
		if err != nil {
			t.Errorf("Error while accepting. Err: %v", err)
			return
		}
		framer := http2.NewFramer(conn, conn)
		if err := framer.WriteSettings(http2.Setting{}); err != nil {
			t.Errorf("Error while writing settings. Err: %v", err)
			return
		}
		<-dialDone // Close conn only after dial returns.
	}()

	r := manual.NewBuilderWithScheme("whatever")
	r.InitialState(resolver.State{Addresses: []resolver.Address{lisAddr}})
	client, err := Dial(r.Scheme()+":///test.server", WithTransportCredentials(insecure.NewCredentials()), WithResolvers(r), WithTimeout(5*time.Second))
	close(dialDone)
	if err != nil {
		t.Fatalf("Dial failed. Err: %v", err)
	}
	defer client.Close()
	timeout := time.After(1 * time.Second)
	select {
	case <-timeout:
		t.Fatal("timed out waiting for server to finish")
	case <-lisDone:
	}
}

func (s) TestDialWithMultipleBackendsNotSendingServerPreface(t *testing.T) {
	lis1, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Error while listening. Err: %v", err)
	}
	defer lis1.Close()
	lis1Addr := resolver.Address{Addr: lis1.Addr().String()}
	lis1Done := make(chan struct{})
	// 1st listener accepts the connection and immediately closes it.
	go func() {
		defer close(lis1Done)
		conn, err := lis1.Accept()
		if err != nil {
			t.Errorf("Error while accepting. Err: %v", err)
			return
		}
		conn.Close()
	}()

	lis2, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Error while listening. Err: %v", err)
	}
	defer lis2.Close()
	lis2Done := make(chan struct{})
	lis2Addr := resolver.Address{Addr: lis2.Addr().String()}
	// 2nd listener should get a connection attempt since the first one failed.
	go func() {
		defer close(lis2Done)
		_, err := lis2.Accept() // Closing the client will clean up this conn.
		if err != nil {
			t.Errorf("Error while accepting. Err: %v", err)
			return
		}
	}()

	r := manual.NewBuilderWithScheme("whatever")
	r.InitialState(resolver.State{Addresses: []resolver.Address{lis1Addr, lis2Addr}})
	client, err := Dial(r.Scheme()+":///test.server", WithTransportCredentials(insecure.NewCredentials()), WithResolvers(r))
	if err != nil {
		t.Fatalf("Dial failed. Err: %v", err)
	}
	defer client.Close()
	timeout := time.After(5 * time.Second)
	select {
	case <-timeout:
		t.Fatal("timed out waiting for server 1 to finish")
	case <-lis1Done:
	}
	select {
	case <-timeout:
		t.Fatal("timed out waiting for server 2 to finish")
	case <-lis2Done:
	}
}

func (s) TestDialWaitsForServerSettings(t *testing.T) {
	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Error while listening. Err: %v", err)
	}
	defer lis.Close()
	done := make(chan struct{})
	sent := make(chan struct{})
	dialDone := make(chan struct{})
	go func() { // Launch the server.
		defer func() {
			close(done)
		}()
		conn, err := lis.Accept()
		if err != nil {
			t.Errorf("Error while accepting. Err: %v", err)
			return
		}
		defer conn.Close()
		// Sleep for a little bit to make sure that Dial on client
		// side blocks until settings are received.
		time.Sleep(100 * time.Millisecond)
		framer := http2.NewFramer(conn, conn)
		close(sent)
		if err := framer.WriteSettings(http2.Setting{}); err != nil {
			t.Errorf("Error while writing settings. Err: %v", err)
			return
		}
		<-dialDone // Close conn only after dial returns.
	}()
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	client, err := DialContext(ctx, lis.Addr().String(), WithTransportCredentials(insecure.NewCredentials()), WithBlock())
	close(dialDone)
	if err != nil {
		t.Fatalf("Error while dialing. Err: %v", err)
	}
	defer client.Close()
	select {
	case <-sent:
	default:
		t.Fatalf("Dial returned before server settings were sent")
	}
	<-done
}

func (s) TestDialWaitsForServerSettingsAndFails(t *testing.T) {
	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Error while listening. Err: %v", err)
	}
	done := make(chan struct{})
	numConns := 0
	go func() { // Launch the server.
		defer func() {
			close(done)
		}()
		for {
			conn, err := lis.Accept()
			if err != nil {
				break
			}
			numConns++
			defer conn.Close()
		}
	}()
	ctx, cancel := context.WithTimeout(context.Background(), 1*time.Second)
	defer cancel()
	client, err := DialContext(ctx,
		lis.Addr().String(),
		WithTransportCredentials(insecure.NewCredentials()),
		WithReturnConnectionError(),
		WithConnectParams(ConnectParams{
			Backoff:           backoff.Config{},
			MinConnectTimeout: 250 * time.Millisecond,
		}))
	lis.Close()
	if err == nil {
		client.Close()
		t.Fatalf("Unexpected success (err=nil) while dialing")
	}
	expectedMsg := "server preface"
	if !strings.Contains(err.Error(), context.DeadlineExceeded.Error()) || !strings.Contains(err.Error(), expectedMsg) {
		t.Fatalf("DialContext(_) = %v; want a message that includes both %q and %q", err, context.DeadlineExceeded.Error(), expectedMsg)
	}
	<-done
	if numConns < 2 {
		t.Fatalf("dial attempts: %v; want > 1", numConns)
	}
}

// 1. Client connects to a server that doesn't send preface.
// 2. After minConnectTimeout(500 ms here), client disconnects and retries.
// 3. The new server sends its preface.
// 4. Client doesn't kill the connection this time.
func (s) TestCloseConnectionWhenServerPrefaceNotReceived(t *testing.T) {
	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Error while listening. Err: %v", err)
	}
	var (
		conn2 net.Conn
		over  uint32
	)
	defer func() {
		lis.Close()
		// conn2 shouldn't be closed until the client has
		// observed a successful test.
		if conn2 != nil {
			conn2.Close()
		}
	}()
	done := make(chan struct{})
	accepted := make(chan struct{})
	go func() { // Launch the server.
		defer close(done)
		conn1, err := lis.Accept()
		if err != nil {
			t.Errorf("Error while accepting. Err: %v", err)
			return
		}
		defer conn1.Close()
		// Don't send server settings and the client should close the connection and try again.
		conn2, err = lis.Accept() // Accept a reconnection request from client.
		if err != nil {
			t.Errorf("Error while accepting. Err: %v", err)
			return
		}
		close(accepted)
		framer := http2.NewFramer(conn2, conn2)
		if err = framer.WriteSettings(http2.Setting{}); err != nil {
			t.Errorf("Error while writing settings. Err: %v", err)
			return
		}
		b := make([]byte, 8)
		for {
			_, err = conn2.Read(b)
			if err == nil {
				continue
			}
			if atomic.LoadUint32(&over) == 1 {
				// The connection stayed alive for the timer.
				// Success.
				return
			}
			t.Errorf("Unexpected error while reading. Err: %v, want timeout error", err)
			break
		}
	}()
	client, err := Dial(lis.Addr().String(), WithTransportCredentials(insecure.NewCredentials()), withMinConnectDeadline(func() time.Duration { return time.Millisecond * 500 }))
	if err != nil {
		t.Fatalf("Error while dialing. Err: %v", err)
	}

	go stayConnected(client)

	// wait for connection to be accepted on the server.
	timer := time.NewTimer(time.Second * 10)
	select {
	case <-accepted:
	case <-timer.C:
		t.Fatalf("Client didn't make another connection request in time.")
	}
	// Make sure the connection stays alive for sometime.
	time.Sleep(time.Second)
	atomic.StoreUint32(&over, 1)
	client.Close()
	<-done
}

func (s) TestBackoffWhenNoServerPrefaceReceived(t *testing.T) {
	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Unexpected error from net.Listen(%q, %q): %v", "tcp", "localhost:0", err)
	}
	defer lis.Close()
	done := make(chan struct{})
	go func() { // Launch the server.
		defer close(done)
		conn, err := lis.Accept() // Accept the connection only to close it immediately.
		if err != nil {
			t.Errorf("Error while accepting. Err: %v", err)
			return
		}
		prevAt := time.Now()
		conn.Close()
		var prevDuration time.Duration
		// Make sure the retry attempts are backed off properly.
		for i := 0; i < 3; i++ {
			conn, err := lis.Accept()
			if err != nil {
				t.Errorf("Error while accepting. Err: %v", err)
				return
			}
			meow := time.Now()
			conn.Close()
			dr := meow.Sub(prevAt)
			if dr <= prevDuration {
				t.Errorf("Client backoff did not increase with retries. Previous duration: %v, current duration: %v", prevDuration, dr)
				return
			}
			prevDuration = dr
			prevAt = meow
		}
	}()
	bc := backoff.Config{
		BaseDelay:  200 * time.Millisecond,
		Multiplier: 2.0,
		Jitter:     0,
		MaxDelay:   120 * time.Second,
	}
	cp := ConnectParams{
		Backoff:           bc,
		MinConnectTimeout: 1 * time.Second,
	}
	cc, err := Dial(lis.Addr().String(), WithTransportCredentials(insecure.NewCredentials()), WithConnectParams(cp))
	if err != nil {
		t.Fatalf("Unexpected error from Dial(%v) = %v", lis.Addr(), err)
	}
	defer cc.Close()
	go stayConnected(cc)
	<-done
}

func (s) TestWithTimeout(t *testing.T) {
	conn, err := Dial("passthrough:///Non-Existent.Server:80",
		WithTimeout(time.Millisecond),
		WithBlock(),
		WithTransportCredentials(insecure.NewCredentials()))
	if err == nil {
		conn.Close()
	}
	if err != context.DeadlineExceeded {
		t.Fatalf("Dial(_, _) = %v, %v, want %v", conn, err, context.DeadlineExceeded)
	}
}

func (s) TestWithTransportCredentialsTLS(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), time.Millisecond)
	defer cancel()
	creds, err := credentials.NewClientTLSFromFile(testdata.Path("x509/server_ca_cert.pem"), "x.test.example.com")
	if err != nil {
		t.Fatalf("Failed to create credentials %v", err)
	}
	conn, err := DialContext(ctx, "passthrough:///Non-Existent.Server:80", WithTransportCredentials(creds), WithBlock())
	if err == nil {
		conn.Close()
	}
	if err != context.DeadlineExceeded {
		t.Fatalf("Dial(_, _) = %v, %v, want %v", conn, err, context.DeadlineExceeded)
	}
}

// When creating a transport configured with n addresses, only calculate the
// backoff once per "round" of attempts instead of once per address (n times
// per "round" of attempts).
func (s) TestDial_OneBackoffPerRetryGroup(t *testing.T) {
	var attempts uint32
	getMinConnectTimeout := func() time.Duration {
		if atomic.AddUint32(&attempts, 1) == 1 {
			// Once all addresses are exhausted, hang around and wait for the
			// client.Close to happen rather than re-starting a new round of
			// attempts.
			return time.Hour
		}
		t.Error("only one attempt backoff calculation, but got more")
		return 0
	}

	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()

	lis1, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Error while listening. Err: %v", err)
	}
	defer lis1.Close()

	lis2, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Error while listening. Err: %v", err)
	}
	defer lis2.Close()

	server1Done := make(chan struct{})
	server2Done := make(chan struct{})

	// Launch server 1.
	go func() {
		conn, err := lis1.Accept()
		if err != nil {
			t.Error(err)
			return
		}

		conn.Close()
		close(server1Done)
	}()
	// Launch server 2.
	go func() {
		conn, err := lis2.Accept()
		if err != nil {
			t.Error(err)
			return
		}
		conn.Close()
		close(server2Done)
	}()

	rb := manual.NewBuilderWithScheme("whatever")
	rb.InitialState(resolver.State{Addresses: []resolver.Address{
		{Addr: lis1.Addr().String()},
		{Addr: lis2.Addr().String()},
	}})
	client, err := DialContext(ctx, "whatever:///this-gets-overwritten",
		WithTransportCredentials(insecure.NewCredentials()),
		WithResolvers(rb),
		withMinConnectDeadline(getMinConnectTimeout))
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()

	timeout := time.After(15 * time.Second)

	select {
	case <-timeout:
		t.Fatal("timed out waiting for test to finish")
	case <-server1Done:
	}

	select {
	case <-timeout:
		t.Fatal("timed out waiting for test to finish")
	case <-server2Done:
	}
}

func (s) TestDialContextCancel(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	cancel()
	if _, err := DialContext(ctx, "Non-Existent.Server:80", WithBlock(), WithTransportCredentials(insecure.NewCredentials())); err != context.Canceled {
		t.Fatalf("DialContext(%v, _) = _, %v, want _, %v", ctx, err, context.Canceled)
	}
}

type failFastError struct{}

func (failFastError) Error() string   { return "failfast" }
func (failFastError) Temporary() bool { return false }

func (s) TestDialContextFailFast(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	failErr := failFastError{}
	dialer := func(string, time.Duration) (net.Conn, error) {
		return nil, failErr
	}

	_, err := DialContext(ctx, "Non-Existent.Server:80", WithBlock(), WithTransportCredentials(insecure.NewCredentials()), WithDialer(dialer), FailOnNonTempDialError(true))
	if terr, ok := err.(transport.ConnectionError); !ok || terr.Origin() != failErr {
		t.Fatalf("DialContext() = _, %v, want _, %v", err, failErr)
	}
}

// securePerRPCCredentials always requires transport security.
type securePerRPCCredentials struct {
	credentials.PerRPCCredentials
}

func (c securePerRPCCredentials) RequireTransportSecurity() bool {
	return true
}

type fakeBundleCreds struct {
	credentials.Bundle
	transportCreds credentials.TransportCredentials
}

func (b *fakeBundleCreds) TransportCredentials() credentials.TransportCredentials {
	return b.transportCreds
}

func (s) TestCredentialsMisuse(t *testing.T) {
	// Use of no transport creds and no creds bundle must fail.
	if _, err := Dial("passthrough:///Non-Existent.Server:80"); err != errNoTransportSecurity {
		t.Fatalf("Dial(_, _) = _, %v, want _, %v", err, errNoTransportSecurity)
	}

	// Use of both transport creds and creds bundle must fail.
	creds, err := credentials.NewClientTLSFromFile(testdata.Path("x509/server_ca_cert.pem"), "x.test.example.com")
	if err != nil {
		t.Fatalf("Failed to create authenticator %v", err)
	}
	dopts := []DialOption{
		WithTransportCredentials(creds),
		WithCredentialsBundle(&fakeBundleCreds{transportCreds: creds}),
	}
	if _, err := Dial("passthrough:///Non-Existent.Server:80", dopts...); err != errTransportCredsAndBundle {
		t.Fatalf("Dial(_, _) = _, %v, want _, %v", err, errTransportCredsAndBundle)
	}

	// Use of perRPC creds requiring transport security over an insecure
	// transport must fail.
	if _, err := Dial("passthrough:///Non-Existent.Server:80", WithPerRPCCredentials(securePerRPCCredentials{}), WithTransportCredentials(insecure.NewCredentials())); err != errTransportCredentialsMissing {
		t.Fatalf("Dial(_, _) = _, %v, want _, %v", err, errTransportCredentialsMissing)
	}

	// Use of a creds bundle with nil transport credentials must fail.
	if _, err := Dial("passthrough:///Non-Existent.Server:80", WithCredentialsBundle(&fakeBundleCreds{})); err != errNoTransportCredsInBundle {
		t.Fatalf("Dial(_, _) = _, %v, want _, %v", err, errTransportCredsAndBundle)
	}
}

func (s) TestWithBackoffConfigDefault(t *testing.T) {
	testBackoffConfigSet(t, internalbackoff.DefaultExponential)
}

func (s) TestWithBackoffConfig(t *testing.T) {
	b := BackoffConfig{MaxDelay: DefaultBackoffConfig.MaxDelay / 2}
	bc := backoff.DefaultConfig
	bc.MaxDelay = b.MaxDelay
	wantBackoff := internalbackoff.Exponential{Config: bc}
	testBackoffConfigSet(t, wantBackoff, WithBackoffConfig(b))
}

func (s) TestWithBackoffMaxDelay(t *testing.T) {
	md := DefaultBackoffConfig.MaxDelay / 2
	bc := backoff.DefaultConfig
	bc.MaxDelay = md
	wantBackoff := internalbackoff.Exponential{Config: bc}
	testBackoffConfigSet(t, wantBackoff, WithBackoffMaxDelay(md))
}

func (s) TestWithConnectParams(t *testing.T) {
	bd := 2 * time.Second
	mltpr := 2.0
	jitter := 0.0
	bc := backoff.Config{BaseDelay: bd, Multiplier: mltpr, Jitter: jitter}

	crt := ConnectParams{Backoff: bc}
	// MaxDelay is not set in the ConnectParams. So it should not be set on
	// internalbackoff.Exponential as well.
	wantBackoff := internalbackoff.Exponential{Config: bc}
	testBackoffConfigSet(t, wantBackoff, WithConnectParams(crt))
}

func testBackoffConfigSet(t *testing.T, wantBackoff internalbackoff.Exponential, opts ...DialOption) {
	opts = append(opts, WithTransportCredentials(insecure.NewCredentials()))
	conn, err := Dial("passthrough:///foo:80", opts...)
	if err != nil {
		t.Fatalf("unexpected error dialing connection: %v", err)
	}
	defer conn.Close()

	if conn.dopts.bs == nil {
		t.Fatalf("backoff config not set")
	}

	gotBackoff, ok := conn.dopts.bs.(internalbackoff.Exponential)
	if !ok {
		t.Fatalf("unexpected type of backoff config: %#v", conn.dopts.bs)
	}

	if gotBackoff != wantBackoff {
		t.Fatalf("unexpected backoff config on connection: %v, want %v", gotBackoff, wantBackoff)
	}
}

func (s) TestConnectParamsWithMinConnectTimeout(t *testing.T) {
	// Default value specified for minConnectTimeout in the spec is 20 seconds.
	mct := 1 * time.Minute
	conn, err := Dial("passthrough:///foo:80", WithTransportCredentials(insecure.NewCredentials()), WithConnectParams(ConnectParams{MinConnectTimeout: mct}))
	if err != nil {
		t.Fatalf("unexpected error dialing connection: %v", err)
	}
	defer conn.Close()

	if got := conn.dopts.minConnectTimeout(); got != mct {
		t.Errorf("unexpected minConnectTimeout on the connection: %v, want %v", got, mct)
	}
}

func (s) TestResolverServiceConfigBeforeAddressNotPanic(t *testing.T) {
	r := manual.NewBuilderWithScheme("whatever")

	cc, err := Dial(r.Scheme()+":///test.server", WithTransportCredentials(insecure.NewCredentials()), WithResolvers(r))
	if err != nil {
		t.Fatalf("failed to dial: %v", err)
	}
	defer cc.Close()

	// SwitchBalancer before NewAddress. There was no balancer created, this
	// makes sure we don't call close on nil balancerWrapper.
	r.UpdateState(resolver.State{ServiceConfig: parseCfg(r, `{"loadBalancingPolicy": "round_robin"}`)}) // This should not panic.

	time.Sleep(time.Second) // Sleep to make sure the service config is handled by ClientConn.
}

func (s) TestResolverServiceConfigWhileClosingNotPanic(t *testing.T) {
	for i := 0; i < 10; i++ { // Run this multiple times to make sure it doesn't panic.
		r := manual.NewBuilderWithScheme(fmt.Sprintf("whatever-%d", i))

		cc, err := Dial(r.Scheme()+":///test.server", WithTransportCredentials(insecure.NewCredentials()), WithResolvers(r))
		if err != nil {
			t.Fatalf("failed to dial: %v", err)
		}
		// Send a new service config while closing the ClientConn.
		go cc.Close()
		go r.UpdateState(resolver.State{ServiceConfig: parseCfg(r, `{"loadBalancingPolicy": "round_robin"}`)}) // This should not panic.
	}
}

func (s) TestResolverEmptyUpdateNotPanic(t *testing.T) {
	r := manual.NewBuilderWithScheme("whatever")

	cc, err := Dial(r.Scheme()+":///test.server", WithTransportCredentials(insecure.NewCredentials()), WithResolvers(r))
	if err != nil {
		t.Fatalf("failed to dial: %v", err)
	}
	defer cc.Close()

	// This make sure we don't create addrConn with empty address list.
	r.UpdateState(resolver.State{}) // This should not panic.

	time.Sleep(time.Second) // Sleep to make sure the service config is handled by ClientConn.
}

func (s) TestClientUpdatesParamsAfterGoAway(t *testing.T) {
	grpctest.TLogger.ExpectError("Client received GoAway with error code ENHANCE_YOUR_CALM and debug data equal to ASCII \"too_many_pings\"")

	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Failed to listen. Err: %v", err)
	}
	defer lis.Close()
	connected := grpcsync.NewEvent()
	defer connected.Fire()
	go func() {
		conn, err := lis.Accept()
		if err != nil {
			t.Errorf("error accepting connection: %v", err)
			return
		}
		defer conn.Close()
		f := http2.NewFramer(conn, conn)
		// Start a goroutine to read from the conn to prevent the client from
		// blocking after it writes its preface.
		go func() {
			for {
				if _, err := f.ReadFrame(); err != nil {
					return
				}
			}
		}()
		if err := f.WriteSettings(http2.Setting{}); err != nil {
			t.Errorf("error writing settings: %v", err)
			return
		}
		<-connected.Done()
		if err := f.WriteGoAway(0, http2.ErrCodeEnhanceYourCalm, []byte("too_many_pings")); err != nil {
			t.Errorf("error writing GOAWAY: %v", err)
			return
		}
	}()
	addr := lis.Addr().String()
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	cc, err := DialContext(ctx, addr, WithBlock(), WithTransportCredentials(insecure.NewCredentials()), WithKeepaliveParams(keepalive.ClientParameters{
		Time:                10 * time.Second,
		Timeout:             100 * time.Millisecond,
		PermitWithoutStream: true,
	}))
	if err != nil {
		t.Fatalf("Dial(%s, _) = _, %v, want _, <nil>", addr, err)
	}
	defer cc.Close()
	connected.Fire()
	for {
		time.Sleep(10 * time.Millisecond)
		cc.mu.RLock()
		v := cc.mkp.Time
		cc.mu.RUnlock()
		if v == 20*time.Second {
			// Success
			return
		}
		if ctx.Err() != nil {
			// Timeout
			t.Fatalf("cc.dopts.copts.Keepalive.Time = %v , want 20s", v)
		}
	}
}

func (s) TestDisableServiceConfigOption(t *testing.T) {
	r := manual.NewBuilderWithScheme("whatever")
	addr := r.Scheme() + ":///non.existent"
	cc, err := Dial(addr, WithTransportCredentials(insecure.NewCredentials()), WithResolvers(r), WithDisableServiceConfig())
	if err != nil {
		t.Fatalf("Dial(%s, _) = _, %v, want _, <nil>", addr, err)
	}
	defer cc.Close()
	r.UpdateState(resolver.State{ServiceConfig: parseCfg(r, `{
    "methodConfig": [
        {
            "name": [
                {
                    "service": "foo",
                    "method": "Bar"
                }
            ],
            "waitForReady": true
        }
    ]
}`)})
	time.Sleep(1 * time.Second)
	m := cc.GetMethodConfig("/foo/Bar")
	if m.WaitForReady != nil {
		t.Fatalf("want: method (\"/foo/bar/\") config to be empty, got: %+v", m)
	}
}

func (s) TestMethodConfigDefaultService(t *testing.T) {
	addr := "nonexist:///non.existent"
	cc, err := Dial(addr, WithTransportCredentials(insecure.NewCredentials()), WithDefaultServiceConfig(`{
  "methodConfig": [{
    "name": [
      {
        "service": ""
      }
    ],
    "waitForReady": true
  }]
}`))
	if err != nil {
		t.Fatalf("Dial(%s, _) = _, %v, want _, <nil>", addr, err)
	}
	defer cc.Close()

	m := cc.GetMethodConfig("/foo/Bar")
	if m.WaitForReady == nil {
		t.Fatalf("want: method (%q) config to fallback to the default service", "/foo/Bar")
	}
}

func (s) TestClientConnCanonicalTarget(t *testing.T) {
	tests := []struct {
		name                string
		addr                string
		canonicalTargetWant string
	}{
		{
			name:                "normal-case",
			addr:                "dns://a.server.com/google.com",
			canonicalTargetWant: "dns://a.server.com/google.com",
		},
		{
			name:                "canonical-target-not-specified",
			addr:                "no.scheme",
			canonicalTargetWant: "passthrough:///no.scheme",
		},
		{
			name:                "canonical-target-nonexistent",
			addr:                "nonexist:///non.existent",
			canonicalTargetWant: "passthrough:///nonexist:///non.existent",
		},
		{
			name:                "canonical-target-add-colon-slash",
			addr:                "dns:hostname:port",
			canonicalTargetWant: "dns:///hostname:port",
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			cc, err := Dial(test.addr, WithTransportCredentials(insecure.NewCredentials()))
			if err != nil {
				t.Fatalf("Dial(%s, _) = _, %v, want _, <nil>", test.addr, err)
			}
			defer cc.Close()
			if cc.Target() != test.addr {
				t.Fatalf("Target() = %s, want %s", cc.Target(), test.addr)
			}
			if cc.CanonicalTarget() != test.canonicalTargetWant {
				t.Fatalf("CanonicalTarget() = %s, want %s", cc.CanonicalTarget(), test.canonicalTargetWant)
			}
		})
	}
}

type backoffForever struct{}

func (b backoffForever) Backoff(int) time.Duration { return time.Duration(math.MaxInt64) }

func (s) TestResetConnectBackoff(t *testing.T) {
	dials := make(chan struct{})
	defer func() { // If we fail, let the http2client break out of dialing.
		select {
		case <-dials:
		default:
		}
	}()
	dialer := func(string, time.Duration) (net.Conn, error) {
		dials <- struct{}{}
		return nil, errors.New("failed to fake dial")
	}
	cc, err := Dial("any", WithTransportCredentials(insecure.NewCredentials()), WithDialer(dialer), withBackoff(backoffForever{}))
	if err != nil {
		t.Fatalf("Dial() = _, %v; want _, nil", err)
	}
	defer cc.Close()
	go stayConnected(cc)
	select {
	case <-dials:
	case <-time.NewTimer(10 * time.Second).C:
		t.Fatal("Failed to call dial within 10s")
	}

	select {
	case <-dials:
		t.Fatal("Dial called unexpectedly before resetting backoff")
	case <-time.NewTimer(100 * time.Millisecond).C:
	}

	cc.ResetConnectBackoff()

	select {
	case <-dials:
	case <-time.NewTimer(10 * time.Second).C:
		t.Fatal("Failed to call dial within 10s after resetting backoff")
	}
}

func (s) TestBackoffCancel(t *testing.T) {
	dialStrCh := make(chan string)
	cc, err := Dial("any", WithTransportCredentials(insecure.NewCredentials()), WithDialer(func(t string, _ time.Duration) (net.Conn, error) {
		dialStrCh <- t
		return nil, fmt.Errorf("test dialer, always error")
	}))
	if err != nil {
		t.Fatalf("Failed to create ClientConn: %v", err)
	}
	defer cc.Close()

	select {
	case <-time.After(defaultTestTimeout):
		t.Fatal("Timeout when waiting for custom dialer to be invoked during Dial")
	case <-dialStrCh:
	}
}

// TestUpdateAddresses_NoopIfCalledWithSameAddresses tests that UpdateAddresses
// should be noop if UpdateAddresses is called with the same list of addresses,
// even when the SubConn is in Connecting and doesn't have a current address.
func (s) TestUpdateAddresses_NoopIfCalledWithSameAddresses(t *testing.T) {
	lis1, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Error while listening. Err: %v", err)
	}
	defer lis1.Close()

	lis2, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Error while listening. Err: %v", err)
	}
	defer lis2.Close()

	lis3, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Error while listening. Err: %v", err)
	}
	defer lis3.Close()

	closeServer2 := make(chan struct{})
	exitCh := make(chan struct{})
	server1ContactedFirstTime := make(chan struct{})
	server1ContactedSecondTime := make(chan struct{})
	server2ContactedFirstTime := make(chan struct{})
	server2ContactedSecondTime := make(chan struct{})
	server3Contacted := make(chan struct{})

	defer close(exitCh)

	// Launch server 1.
	go func() {
		// First, let's allow the initial connection to go READY. We need to do
		// this because tryUpdateAddrs only works after there's some non-nil
		// address on the ac, and curAddress is only set after READY.
		conn1, err := lis1.Accept()
		if err != nil {
			t.Error(err)
			return
		}
		go keepReading(conn1)

		framer := http2.NewFramer(conn1, conn1)
		if err := framer.WriteSettings(http2.Setting{}); err != nil {
			t.Errorf("Error while writing settings frame. %v", err)
			return
		}

		// nextStateNotifier() is updated after balancerBuilder.Build(), which is
		// called by grpc.Dial. It's safe to do it here because lis1.Accept blocks
		// until balancer is built to process the addresses.
		stateNotifications := testBalancerBuilder.nextStateNotifier()
		// Wait for the transport to become ready.
		for {
			select {
			case st := <-stateNotifications:
				if st == connectivity.Ready {
					goto ready
				}
			case <-exitCh:
				return
			}
		}

	ready:
		// Once it's ready, curAddress has been set. So let's close this
		// connection prompting the first reconnect cycle.
		conn1.Close()

		// Accept and immediately close, causing it to go to server2.
		conn2, err := lis1.Accept()
		if err != nil {
			t.Error(err)
			return
		}
		close(server1ContactedFirstTime)
		conn2.Close()

		// Hopefully it picks this server after tryUpdateAddrs.
		lis1.Accept()
		close(server1ContactedSecondTime)
	}()
	// Launch server 2.
	go func() {
		// Accept and then hang waiting for the test call tryUpdateAddrs and
		// then signal to this server to close. After this server closes, it
		// should start from the top instead of trying server2 or continuing
		// to server3.
		conn, err := lis2.Accept()
		if err != nil {
			t.Error(err)
			return
		}

		close(server2ContactedFirstTime)
		<-closeServer2
		conn.Close()

		// After tryUpdateAddrs, it should NOT try server2.
		lis2.Accept()
		close(server2ContactedSecondTime)
	}()
	// Launch server 3.
	go func() {
		// After tryUpdateAddrs, it should NOT try server3. (or any other time)
		lis3.Accept()
		close(server3Contacted)
	}()

	addrsList := []resolver.Address{
		{Addr: lis1.Addr().String()},
		{Addr: lis2.Addr().String()},
		{Addr: lis3.Addr().String()},
	}
	rb := manual.NewBuilderWithScheme("whatever")
	rb.InitialState(resolver.State{Addresses: addrsList})

	client, err := Dial("whatever:///this-gets-overwritten",
		WithTransportCredentials(insecure.NewCredentials()),
		WithResolvers(rb),
		WithConnectParams(ConnectParams{
			Backoff:           backoff.Config{},
			MinConnectTimeout: time.Hour,
		}),
		WithDefaultServiceConfig(fmt.Sprintf(`{"loadBalancingConfig": [{"%s":{}}]}`, stateRecordingBalancerName)))
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()
	go stayConnected(client)

	timeout := time.After(5 * time.Second)

	// Wait for server1 to be contacted (which will immediately fail), then
	// server2 (which will hang waiting for our signal).
	select {
	case <-server1ContactedFirstTime:
	case <-timeout:
		t.Fatal("timed out waiting for server1 to be contacted")
	}
	select {
	case <-server2ContactedFirstTime:
	case <-timeout:
		t.Fatal("timed out waiting for server2 to be contacted")
	}

	// Grab the addrConn and call tryUpdateAddrs.
	var ac *addrConn
	client.mu.Lock()
	for clientAC := range client.conns {
		ac = clientAC
		break
	}
	client.mu.Unlock()

	// Call UpdateAddresses with the same list of addresses, it should be a noop
	// (even when the SubConn is Connecting, and doesn't have a curAddr).
	ac.acbw.UpdateAddresses(addrsList)

	// We've called tryUpdateAddrs - now let's make server2 close the
	// connection and check that it continues to server3.
	close(closeServer2)

	select {
	case <-server1ContactedSecondTime:
		t.Fatal("server1 was contacted a second time, but it should have continued to server 3")
	case <-server2ContactedSecondTime:
		t.Fatal("server2 was contacted a second time, but it should have continued to server 3")
	case <-server3Contacted:
	case <-timeout:
		t.Fatal("timed out waiting for any server to be contacted after tryUpdateAddrs")
	}
}

func (s) TestDefaultServiceConfig(t *testing.T) {
	const defaultSC = `
{
    "methodConfig": [
        {
            "name": [
                {
                    "service": "foo",
                    "method": "bar"
                }
            ],
            "waitForReady": true
        }
    ]
}`
	tests := []struct {
		name  string
		testF func(t *testing.T, r *manual.Resolver, addr, sc string)
		sc    string
	}{
		{
			name:  "invalid-service-config",
			testF: testInvalidDefaultServiceConfig,
			sc:    "",
		},
		{
			name:  "resolver-service-config-disabled",
			testF: testDefaultServiceConfigWhenResolverServiceConfigDisabled,
			sc:    defaultSC,
		},
		{
			name:  "resolver-does-not-return-service-config",
			testF: testDefaultServiceConfigWhenResolverDoesNotReturnServiceConfig,
			sc:    defaultSC,
		},
		{
			name:  "resolver-returns-invalid-service-config",
			testF: testDefaultServiceConfigWhenResolverReturnInvalidServiceConfig,
			sc:    defaultSC,
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			r := manual.NewBuilderWithScheme(test.name)
			addr := r.Scheme() + ":///non.existent"
			test.testF(t, r, addr, test.sc)
		})
	}
}

func verifyWaitForReadyEqualsTrue(cc *ClientConn) bool {
	var i int
	for i = 0; i < 10; i++ {
		mc := cc.GetMethodConfig("/foo/bar")
		if mc.WaitForReady != nil && *mc.WaitForReady == true {
			break
		}
		time.Sleep(100 * time.Millisecond)
	}
	return i != 10
}

func testInvalidDefaultServiceConfig(t *testing.T, r *manual.Resolver, addr, sc string) {
	_, err := Dial(addr, WithTransportCredentials(insecure.NewCredentials()), WithResolvers(r), WithDefaultServiceConfig(sc))
	if !strings.Contains(err.Error(), invalidDefaultServiceConfigErrPrefix) {
		t.Fatalf("Dial got err: %v, want err contains: %v", err, invalidDefaultServiceConfigErrPrefix)
	}
}

func testDefaultServiceConfigWhenResolverServiceConfigDisabled(t *testing.T, r *manual.Resolver, addr string, js string) {
	cc, err := Dial(addr, WithTransportCredentials(insecure.NewCredentials()), WithDisableServiceConfig(), WithResolvers(r), WithDefaultServiceConfig(js))
	if err != nil {
		t.Fatalf("Dial(%s, _) = _, %v, want _, <nil>", addr, err)
	}
	defer cc.Close()
	// Resolver service config gets ignored since resolver service config is disabled.
	r.UpdateState(resolver.State{
		Addresses:     []resolver.Address{{Addr: addr}},
		ServiceConfig: parseCfg(r, "{}"),
	})
	if !verifyWaitForReadyEqualsTrue(cc) {
		t.Fatal("default service config failed to be applied after 1s")
	}
}

func testDefaultServiceConfigWhenResolverDoesNotReturnServiceConfig(t *testing.T, r *manual.Resolver, addr string, js string) {
	cc, err := Dial(addr, WithTransportCredentials(insecure.NewCredentials()), WithResolvers(r), WithDefaultServiceConfig(js))
	if err != nil {
		t.Fatalf("Dial(%s, _) = _, %v, want _, <nil>", addr, err)
	}
	defer cc.Close()
	r.UpdateState(resolver.State{
		Addresses: []resolver.Address{{Addr: addr}},
	})
	if !verifyWaitForReadyEqualsTrue(cc) {
		t.Fatal("default service config failed to be applied after 1s")
	}
}

func testDefaultServiceConfigWhenResolverReturnInvalidServiceConfig(t *testing.T, r *manual.Resolver, addr string, js string) {
	cc, err := Dial(addr, WithTransportCredentials(insecure.NewCredentials()), WithResolvers(r), WithDefaultServiceConfig(js))
	if err != nil {
		t.Fatalf("Dial(%s, _) = _, %v, want _, <nil>", addr, err)
	}
	defer cc.Close()
	r.UpdateState(resolver.State{
		Addresses: []resolver.Address{{Addr: addr}},
	})
	if !verifyWaitForReadyEqualsTrue(cc) {
		t.Fatal("default service config failed to be applied after 1s")
	}
}

type stateRecordingBalancer struct {
	balancer.Balancer
}

func (b *stateRecordingBalancer) UpdateSubConnState(sc balancer.SubConn, s balancer.SubConnState) {
	panic(fmt.Sprintf("UpdateSubConnState(%v, %+v) called unexpectedly", sc, s))
}

func (b *stateRecordingBalancer) Close() {
	b.Balancer.Close()
}

type stateRecordingBalancerBuilder struct {
	mu       sync.Mutex
	notifier chan connectivity.State // The notifier used in the last Balancer.
}

func newStateRecordingBalancerBuilder() *stateRecordingBalancerBuilder {
	return &stateRecordingBalancerBuilder{}
}

func (b *stateRecordingBalancerBuilder) Name() string {
	return stateRecordingBalancerName
}

func (b *stateRecordingBalancerBuilder) Build(cc balancer.ClientConn, opts balancer.BuildOptions) balancer.Balancer {
	stateNotifications := make(chan connectivity.State, 10)
	b.mu.Lock()
	b.notifier = stateNotifications
	b.mu.Unlock()
	return &stateRecordingBalancer{
		Balancer: balancer.Get("pick_first").Build(&stateRecordingCCWrapper{cc, stateNotifications}, opts),
	}
}

func (b *stateRecordingBalancerBuilder) nextStateNotifier() <-chan connectivity.State {
	b.mu.Lock()
	defer b.mu.Unlock()
	ret := b.notifier
	b.notifier = nil
	return ret
}

type stateRecordingCCWrapper struct {
	balancer.ClientConn
	notifier chan<- connectivity.State
}

func (ccw *stateRecordingCCWrapper) NewSubConn(addrs []resolver.Address, opts balancer.NewSubConnOptions) (balancer.SubConn, error) {
	oldListener := opts.StateListener
	opts.StateListener = func(s balancer.SubConnState) {
		ccw.notifier <- s.ConnectivityState
		oldListener(s)
	}
	return ccw.ClientConn.NewSubConn(addrs, opts)
}

// Keep reading until something causes the connection to die (EOF, server
// closed, etc). Useful as a tool for mindlessly keeping the connection
// healthy, since the client will error if things like client prefaces are not
// accepted in a timely fashion.
func keepReading(conn net.Conn) {
	buf := make([]byte, 1024)
	for _, err := conn.Read(buf); err == nil; _, err = conn.Read(buf) {
	}
}

// stayConnected makes cc stay connected by repeatedly calling cc.Connect()
// until the state becomes Shutdown or until 10 seconds elapses.
func stayConnected(cc *ClientConn) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	for {
		state := cc.GetState()
		switch state {
		case connectivity.Idle:
			cc.Connect()
		case connectivity.Shutdown:
			return
		}
		if !cc.WaitForStateChange(ctx, state) {
			return
		}
	}
}

func (s) TestURLAuthorityEscape(t *testing.T) {
	tests := []struct {
		name      string
		authority string
		want      string
	}{
		{
			name:      "ipv6_authority",
			authority: "[::1]",
			want:      "[::1]",
		},
		{
			name:      "with_user_and_host",
			authority: "userinfo@host:10001",
			want:      "userinfo@host:10001",
		},
		{
			name:      "with_multiple_slashes",
			authority: "projects/123/network/abc/service",
			want:      "projects%2F123%2Fnetwork%2Fabc%2Fservice",
		},
		{
			name:      "all_possible_allowed_chars",
			authority: "abc123-._~!$&'()*+,;=@:[]",
			want:      "abc123-._~!$&'()*+,;=@:[]",
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			if got, want := encodeAuthority(test.authority), test.want; got != want {
				t.Errorf("encodeAuthority(%s) = %s, want %s", test.authority, got, test.want)
			}
		})
	}
}

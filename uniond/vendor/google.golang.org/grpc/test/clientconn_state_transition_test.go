/*
 *
 * Copyright 2018 gRPC authors.
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

package test

import (
	"context"
	"fmt"
	"net"
	"sync"
	"testing"
	"time"

	"golang.org/x/net/http2"
	"google.golang.org/grpc"
	"google.golang.org/grpc/backoff"
	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/balancer/stub"
	"google.golang.org/grpc/internal/grpcsync"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/resolver/manual"
)

const stateRecordingBalancerName = "state_recording_balancer"

var testBalancerBuilder = newStateRecordingBalancerBuilder()

func init() {
	balancer.Register(testBalancerBuilder)
}

// These tests use a pipeListener. This listener is similar to net.Listener
// except that it is unbuffered, so each read and write will wait for the other
// side's corresponding write or read.
func (s) TestStateTransitions_SingleAddress(t *testing.T) {
	for _, test := range []struct {
		desc   string
		want   []connectivity.State
		server func(net.Listener) net.Conn
	}{
		{
			desc: "When the server returns server preface, the client enters READY.",
			want: []connectivity.State{
				connectivity.Connecting,
				connectivity.Ready,
			},
			server: func(lis net.Listener) net.Conn {
				conn, err := lis.Accept()
				if err != nil {
					t.Error(err)
					return nil
				}

				go keepReading(conn)

				framer := http2.NewFramer(conn, conn)
				if err := framer.WriteSettings(http2.Setting{}); err != nil {
					t.Errorf("Error while writing settings frame. %v", err)
					return nil
				}

				return conn
			},
		},
		{
			desc: "When the connection is closed before the preface is sent, the client enters TRANSIENT FAILURE.",
			want: []connectivity.State{
				connectivity.Connecting,
				connectivity.TransientFailure,
			},
			server: func(lis net.Listener) net.Conn {
				conn, err := lis.Accept()
				if err != nil {
					t.Error(err)
					return nil
				}

				conn.Close()
				return nil
			},
		},
		{
			desc: `When the server sends its connection preface, but the connection dies before the client can write its
connection preface, the client enters TRANSIENT FAILURE.`,
			want: []connectivity.State{
				connectivity.Connecting,
				connectivity.TransientFailure,
			},
			server: func(lis net.Listener) net.Conn {
				conn, err := lis.Accept()
				if err != nil {
					t.Error(err)
					return nil
				}

				framer := http2.NewFramer(conn, conn)
				if err := framer.WriteSettings(http2.Setting{}); err != nil {
					t.Errorf("Error while writing settings frame. %v", err)
					return nil
				}

				conn.Close()
				return nil
			},
		},
		{
			desc: `When the server reads the client connection preface but does not send its connection preface, the
client enters TRANSIENT FAILURE.`,
			want: []connectivity.State{
				connectivity.Connecting,
				connectivity.TransientFailure,
			},
			server: func(lis net.Listener) net.Conn {
				conn, err := lis.Accept()
				if err != nil {
					t.Error(err)
					return nil
				}

				go keepReading(conn)

				return conn
			},
		},
	} {
		t.Log(test.desc)
		testStateTransitionSingleAddress(t, test.want, test.server)
	}
}

func testStateTransitionSingleAddress(t *testing.T, want []connectivity.State, server func(net.Listener) net.Conn) {
	pl := testutils.NewPipeListener()
	defer pl.Close()

	// Launch the server.
	var conn net.Conn
	var connMu sync.Mutex
	go func() {
		connMu.Lock()
		conn = server(pl)
		connMu.Unlock()
	}()

	client, err := grpc.Dial("",
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithDefaultServiceConfig(fmt.Sprintf(`{"loadBalancingConfig": [{"%s":{}}]}`, stateRecordingBalancerName)),
		grpc.WithDialer(pl.Dialer()),
		grpc.WithConnectParams(grpc.ConnectParams{
			Backoff:           backoff.Config{},
			MinConnectTimeout: 100 * time.Millisecond,
		}))
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	go testutils.StayConnected(ctx, client)

	stateNotifications := testBalancerBuilder.nextStateNotifier()
	for i := 0; i < len(want); i++ {
		select {
		case <-time.After(defaultTestTimeout):
			t.Fatalf("timed out waiting for state %d (%v) in flow %v", i, want[i], want)
		case seen := <-stateNotifications:
			if seen != want[i] {
				t.Fatalf("expected to see %v at position %d in flow %v, got %v", want[i], i, want, seen)
			}
		}
	}

	connMu.Lock()
	defer connMu.Unlock()
	if conn != nil {
		err = conn.Close()
		if err != nil {
			t.Fatal(err)
		}
	}
}

// When a READY connection is closed, the client enters IDLE then CONNECTING.
func (s) TestStateTransitions_ReadyToConnecting(t *testing.T) {
	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Error while listening. Err: %v", err)
	}
	defer lis.Close()

	sawReady := make(chan struct{}, 1)
	defer close(sawReady)

	// Launch the server.
	go func() {
		conn, err := lis.Accept()
		if err != nil {
			t.Error(err)
			return
		}

		go keepReading(conn)

		framer := http2.NewFramer(conn, conn)
		if err := framer.WriteSettings(http2.Setting{}); err != nil {
			t.Errorf("Error while writing settings frame. %v", err)
			return
		}

		// Prevents race between onPrefaceReceipt and onClose.
		<-sawReady

		conn.Close()
	}()

	client, err := grpc.Dial(lis.Addr().String(),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithDefaultServiceConfig(fmt.Sprintf(`{"loadBalancingConfig": [{"%s":{}}]}`, stateRecordingBalancerName)))
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	go testutils.StayConnected(ctx, client)

	stateNotifications := testBalancerBuilder.nextStateNotifier()

	want := []connectivity.State{
		connectivity.Connecting,
		connectivity.Ready,
		connectivity.Idle,
		connectivity.Connecting,
	}
	for i := 0; i < len(want); i++ {
		select {
		case <-time.After(defaultTestTimeout):
			t.Fatalf("timed out waiting for state %d (%v) in flow %v", i, want[i], want)
		case seen := <-stateNotifications:
			if seen == connectivity.Ready {
				sawReady <- struct{}{}
			}
			if seen != want[i] {
				t.Fatalf("expected to see %v at position %d in flow %v, got %v", want[i], i, want, seen)
			}
		}
	}
}

// When the first connection is closed, the client stays in CONNECTING until it
// tries the second address (which succeeds, and then it enters READY).
func (s) TestStateTransitions_TriesAllAddrsBeforeTransientFailure(t *testing.T) {
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

		go keepReading(conn)

		framer := http2.NewFramer(conn, conn)
		if err := framer.WriteSettings(http2.Setting{}); err != nil {
			t.Errorf("Error while writing settings frame. %v", err)
			return
		}

		close(server2Done)
	}()

	rb := manual.NewBuilderWithScheme("whatever")
	rb.InitialState(resolver.State{Addresses: []resolver.Address{
		{Addr: lis1.Addr().String()},
		{Addr: lis2.Addr().String()},
	}})
	client, err := grpc.Dial("whatever:///this-gets-overwritten",
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithDefaultServiceConfig(fmt.Sprintf(`{"loadBalancingConfig": [{"%s":{}}]}`, stateRecordingBalancerName)),
		grpc.WithResolvers(rb))
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()

	stateNotifications := testBalancerBuilder.nextStateNotifier()
	want := []connectivity.State{
		connectivity.Connecting,
		connectivity.Ready,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	for i := 0; i < len(want); i++ {
		select {
		case <-ctx.Done():
			t.Fatalf("timed out waiting for state %d (%v) in flow %v", i, want[i], want)
		case seen := <-stateNotifications:
			if seen != want[i] {
				t.Fatalf("expected to see %v at position %d in flow %v, got %v", want[i], i, want, seen)
			}
		}
	}
	select {
	case <-ctx.Done():
		t.Fatal("saw the correct state transitions, but timed out waiting for client to finish interactions with server 1")
	case <-server1Done:
	}
	select {
	case <-ctx.Done():
		t.Fatal("saw the correct state transitions, but timed out waiting for client to finish interactions with server 2")
	case <-server2Done:
	}
}

// When there are multiple addresses, and we enter READY on one of them, a
// later closure should cause the client to enter CONNECTING
func (s) TestStateTransitions_MultipleAddrsEntersReady(t *testing.T) {
	lis1, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Error while listening. Err: %v", err)
	}
	defer lis1.Close()

	// Never actually gets used; we just want it to be alive so that the resolver has two addresses to target.
	lis2, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Error while listening. Err: %v", err)
	}
	defer lis2.Close()

	server1Done := make(chan struct{})
	sawReady := make(chan struct{}, 1)
	defer close(sawReady)

	// Launch server 1.
	go func() {
		conn, err := lis1.Accept()
		if err != nil {
			t.Error(err)
			return
		}

		go keepReading(conn)

		framer := http2.NewFramer(conn, conn)
		if err := framer.WriteSettings(http2.Setting{}); err != nil {
			t.Errorf("Error while writing settings frame. %v", err)
			return
		}

		<-sawReady

		conn.Close()

		close(server1Done)
	}()

	rb := manual.NewBuilderWithScheme("whatever")
	rb.InitialState(resolver.State{Addresses: []resolver.Address{
		{Addr: lis1.Addr().String()},
		{Addr: lis2.Addr().String()},
	}})
	client, err := grpc.Dial("whatever:///this-gets-overwritten",
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithDefaultServiceConfig(fmt.Sprintf(`{"loadBalancingConfig": [{"%s":{}}]}`, stateRecordingBalancerName)),
		grpc.WithResolvers(rb))
	if err != nil {
		t.Fatal(err)
	}
	defer client.Close()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	go testutils.StayConnected(ctx, client)

	stateNotifications := testBalancerBuilder.nextStateNotifier()
	want := []connectivity.State{
		connectivity.Connecting,
		connectivity.Ready,
		connectivity.Idle,
		connectivity.Connecting,
	}
	for i := 0; i < len(want); i++ {
		select {
		case <-ctx.Done():
			t.Fatalf("timed out waiting for state %d (%v) in flow %v", i, want[i], want)
		case seen := <-stateNotifications:
			if seen == connectivity.Ready {
				sawReady <- struct{}{}
			}
			if seen != want[i] {
				t.Fatalf("expected to see %v at position %d in flow %v, got %v", want[i], i, want, seen)
			}
		}
	}
	select {
	case <-ctx.Done():
		t.Fatal("saw the correct state transitions, but timed out waiting for client to finish interactions with server 1")
	case <-server1Done:
	}
}

type stateRecordingBalancer struct {
	balancer.Balancer
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

type funcConnectivityStateSubscriber struct {
	onMsg func(connectivity.State)
}

func (f *funcConnectivityStateSubscriber) OnMessage(msg any) {
	f.onMsg(msg.(connectivity.State))
}

// TestConnectivityStateSubscriber confirms updates sent by the balancer in
// rapid succession are not missed by the subscriber.
func (s) TestConnectivityStateSubscriber(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	sendStates := []connectivity.State{
		connectivity.Connecting,
		connectivity.Ready,
		connectivity.Idle,
		connectivity.Connecting,
		connectivity.Idle,
		connectivity.Connecting,
		connectivity.Ready,
	}
	wantStates := append(sendStates, connectivity.Shutdown)

	const testBalName = "any"
	bf := stub.BalancerFuncs{
		UpdateClientConnState: func(bd *stub.BalancerData, _ balancer.ClientConnState) error {
			// Send the expected states in rapid succession.
			for _, s := range sendStates {
				t.Logf("Sending state update %s", s)
				bd.ClientConn.UpdateState(balancer.State{ConnectivityState: s})
			}
			return nil
		},
	}
	stub.Register(testBalName, bf)

	// Create the ClientConn.
	const testResName = "any"
	rb := manual.NewBuilderWithScheme(testResName)
	cc, err := grpc.Dial(testResName+":///",
		grpc.WithResolvers(rb),
		grpc.WithDefaultServiceConfig(fmt.Sprintf(`{"loadBalancingConfig": [{"%s":{}}]}`, testBalName)),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		t.Fatalf("Unexpected error from grpc.Dial: %v", err)
	}

	// Subscribe to state updates.  Use a buffer size of 1 to allow the
	// Shutdown state to go into the channel when Close()ing.
	connCh := make(chan connectivity.State, 1)
	s := &funcConnectivityStateSubscriber{
		onMsg: func(s connectivity.State) {
			select {
			case connCh <- s:
			case <-ctx.Done():
			}
			if s == connectivity.Shutdown {
				close(connCh)
			}
		},
	}

	internal.SubscribeToConnectivityStateChanges.(func(cc *grpc.ClientConn, s grpcsync.Subscriber) func())(cc, s)

	// Send an update from the resolver that will trigger the LB policy's UpdateClientConnState.
	go rb.UpdateState(resolver.State{})

	// Verify the resulting states.
	for i, want := range wantStates {
		if i == len(sendStates) {
			// Trigger Shutdown to be sent by the channel.  Use a goroutine to
			// ensure the operation does not block.
			cc.Close()
		}
		select {
		case got := <-connCh:
			if got != want {
				t.Errorf("Update %v was %s; want %s", i, got, want)
			} else {
				t.Logf("Update %v was %s as expected", i, got)
			}
		case <-ctx.Done():
			t.Fatalf("Timed out waiting for state update %v: %s", i, want)
		}
	}
}

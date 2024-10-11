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

package handshaker

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	grpc "google.golang.org/grpc"
	core "google.golang.org/grpc/credentials/alts/internal"
	altspb "google.golang.org/grpc/credentials/alts/internal/proto/grpc_gcp"
	"google.golang.org/grpc/credentials/alts/internal/testutil"
	"google.golang.org/grpc/internal/envconfig"
	"google.golang.org/grpc/internal/grpctest"
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

var (
	testRecordProtocol = rekeyRecordProtocolName
	testKey            = []byte{
		// 44 arbitrary bytes.
		0x1f, 0x8b, 0x08, 0x00, 0x00, 0x09, 0x6e, 0x88, 0x02, 0xff, 0xe2, 0xd2, 0x4c, 0xce, 0x4f, 0x49,
		0x08, 0x00, 0x00, 0x09, 0x6e, 0x88, 0x02, 0xff, 0xe2, 0xd2, 0x4c, 0xce, 0x4f, 0x49, 0x1f, 0x8b,
		0xd2, 0x4c, 0xce, 0x08, 0x00, 0x00, 0x09, 0x6e, 0x88, 0x02, 0xff, 0xe2,
	}
	testServiceAccount        = "test_service_account"
	testTargetServiceAccounts = []string{testServiceAccount}
	testClientIdentity        = &altspb.Identity{
		IdentityOneof: &altspb.Identity_Hostname{
			Hostname: "i_am_a_client",
		},
	}
)

const defaultTestTimeout = 10 * time.Second

// testRPCStream mimics a altspb.HandshakerService_DoHandshakeClient object.
type testRPCStream struct {
	grpc.ClientStream
	t        *testing.T
	isClient bool
	// The resp expected to be returned by Recv(). Make sure this is set to
	// the content the test requires before Recv() is invoked.
	recvBuf *altspb.HandshakerResp
	// false if it is the first access to Handshaker service on Envelope.
	first bool
	// useful for testing concurrent calls.
	delay time.Duration
	// The minimum expected value of the network_latency_ms field in a
	// NextHandshakeMessageReq.
	minExpectedNetworkLatency time.Duration
}

func (t *testRPCStream) Recv() (*altspb.HandshakerResp, error) {
	resp := t.recvBuf
	t.recvBuf = nil
	return resp, nil
}

func (t *testRPCStream) Send(req *altspb.HandshakerReq) error {
	var resp *altspb.HandshakerResp
	if !t.first {
		// Generate the bytes to be returned by Recv() for the initial
		// handshaking.
		t.first = true
		if t.isClient {
			resp = &altspb.HandshakerResp{
				OutFrames: testutil.MakeFrame("ClientInit"),
				// Simulate consuming ServerInit.
				BytesConsumed: 14,
			}
		} else {
			resp = &altspb.HandshakerResp{
				OutFrames: testutil.MakeFrame("ServerInit"),
				// Simulate consuming ClientInit.
				BytesConsumed: 14,
			}
		}
	} else {
		switch req := req.ReqOneof.(type) {
		case *altspb.HandshakerReq_Next:
			// Compare the network_latency_ms field to the minimum expected network
			// latency.
			if nl := time.Duration(req.Next.NetworkLatencyMs) * time.Millisecond; nl < t.minExpectedNetworkLatency {
				return fmt.Errorf("networkLatency (%v) is smaller than expected min network latency (%v)", nl, t.minExpectedNetworkLatency)
			}
		default:
			return fmt.Errorf("handshake request has unexpected type: %v", req)
		}

		// Add delay to test concurrent calls.
		cleanup := stat.Update()
		defer cleanup()
		time.Sleep(t.delay)

		// Generate the response to be returned by Recv() for the
		// follow-up handshaking.
		result := &altspb.HandshakerResult{
			RecordProtocol: testRecordProtocol,
			KeyData:        testKey,
		}
		resp = &altspb.HandshakerResp{
			Result: result,
			// Simulate consuming ClientFinished or ServerFinished.
			BytesConsumed: 18,
		}
	}
	t.recvBuf = resp
	return nil
}

func (t *testRPCStream) CloseSend() error {
	return nil
}

var stat testutil.Stats

func (s) TestClientHandshake(t *testing.T) {
	for _, testCase := range []struct {
		delay              time.Duration
		numberOfHandshakes int
		readLatency        time.Duration
	}{
		{0 * time.Millisecond, 1, time.Duration(0)},
		{0 * time.Millisecond, 1, 2 * time.Millisecond},
		{100 * time.Millisecond, 10 * int(envconfig.ALTSMaxConcurrentHandshakes), time.Duration(0)},
	} {
		errc := make(chan error)
		stat.Reset()

		ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
		defer cancel()

		for i := 0; i < testCase.numberOfHandshakes; i++ {
			stream := &testRPCStream{
				t:                         t,
				isClient:                  true,
				minExpectedNetworkLatency: testCase.readLatency,
			}
			// Preload the inbound frames.
			f1 := testutil.MakeFrame("ServerInit")
			f2 := testutil.MakeFrame("ServerFinished")
			in := bytes.NewBuffer(f1)
			in.Write(f2)
			out := new(bytes.Buffer)
			tc := testutil.NewTestConnWithReadLatency(in, out, testCase.readLatency)
			chs := &altsHandshaker{
				stream: stream,
				conn:   tc,
				clientOpts: &ClientHandshakerOptions{
					TargetServiceAccounts: testTargetServiceAccounts,
					ClientIdentity:        testClientIdentity,
				},
				side: core.ClientSide,
			}
			go func() {
				_, context, err := chs.ClientHandshake(ctx)
				if err == nil && context == nil {
					errc <- errors.New("expected non-nil ALTS context")
					return
				}
				errc <- err
				chs.Close()
			}()
		}

		// Ensure that there are no errors.
		for i := 0; i < testCase.numberOfHandshakes; i++ {
			if err := <-errc; err != nil {
				t.Errorf("ClientHandshake() = _, %v, want _, <nil>", err)
			}
		}

		// Ensure that there are no concurrent calls more than the limit.
		if stat.MaxConcurrentCalls > int(envconfig.ALTSMaxConcurrentHandshakes) {
			t.Errorf("Observed %d concurrent handshakes; want <= %d", stat.MaxConcurrentCalls, envconfig.ALTSMaxConcurrentHandshakes)
		}
	}
}

func (s) TestServerHandshake(t *testing.T) {
	for _, testCase := range []struct {
		delay              time.Duration
		numberOfHandshakes int
	}{
		{0 * time.Millisecond, 1},
		{100 * time.Millisecond, 10 * int(envconfig.ALTSMaxConcurrentHandshakes)},
	} {
		errc := make(chan error)
		stat.Reset()

		ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
		defer cancel()

		for i := 0; i < testCase.numberOfHandshakes; i++ {
			stream := &testRPCStream{
				t:        t,
				isClient: false,
			}
			// Preload the inbound frames.
			f1 := testutil.MakeFrame("ClientInit")
			f2 := testutil.MakeFrame("ClientFinished")
			in := bytes.NewBuffer(f1)
			in.Write(f2)
			out := new(bytes.Buffer)
			tc := testutil.NewTestConn(in, out)
			shs := &altsHandshaker{
				stream:     stream,
				conn:       tc,
				serverOpts: DefaultServerHandshakerOptions(),
				side:       core.ServerSide,
			}
			go func() {
				_, context, err := shs.ServerHandshake(ctx)
				if err == nil && context == nil {
					errc <- errors.New("expected non-nil ALTS context")
					return
				}
				errc <- err
				shs.Close()
			}()
		}

		// Ensure that there are no errors.
		for i := 0; i < testCase.numberOfHandshakes; i++ {
			if err := <-errc; err != nil {
				t.Errorf("ServerHandshake() = _, %v, want _, <nil>", err)
			}
		}

		// Ensure that there are no concurrent calls more than the limit.
		if stat.MaxConcurrentCalls > int(envconfig.ALTSMaxConcurrentHandshakes) {
			t.Errorf("Observed %d concurrent handshakes; want <= %d", stat.MaxConcurrentCalls, envconfig.ALTSMaxConcurrentHandshakes)
		}
	}
}

// testUnresponsiveRPCStream is used for testing the PeerNotResponding case.
type testUnresponsiveRPCStream struct {
	grpc.ClientStream
}

func (t *testUnresponsiveRPCStream) Recv() (*altspb.HandshakerResp, error) {
	return &altspb.HandshakerResp{}, nil
}

func (t *testUnresponsiveRPCStream) Send(*altspb.HandshakerReq) error {
	return nil
}

func (t *testUnresponsiveRPCStream) CloseSend() error {
	return nil
}

func (s) TestPeerNotResponding(t *testing.T) {
	stream := &testUnresponsiveRPCStream{}
	chs := &altsHandshaker{
		stream: stream,
		conn:   testutil.NewUnresponsiveTestConn(),
		clientOpts: &ClientHandshakerOptions{
			TargetServiceAccounts: testTargetServiceAccounts,
			ClientIdentity:        testClientIdentity,
		},
		side: core.ClientSide,
	}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	_, context, err := chs.ClientHandshake(ctx)
	chs.Close()
	if context != nil {
		t.Error("expected non-nil ALTS context")
	}
	if got, want := err, core.PeerNotRespondingError; got != want {
		t.Errorf("ClientHandshake() = %v, want %v", got, want)
	}
}

func (s) TestNewClientHandshaker(t *testing.T) {
	conn := testutil.NewTestConn(nil, nil)
	clientConn := &grpc.ClientConn{}
	opts := &ClientHandshakerOptions{}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	hs, err := NewClientHandshaker(ctx, clientConn, conn, opts)
	if err != nil {
		t.Errorf("NewClientHandshaker returned unexpected error: %v", err)
	}
	expectedHs := &altsHandshaker{
		stream:     nil,
		conn:       conn,
		clientConn: clientConn,
		clientOpts: opts,
		serverOpts: nil,
		side:       core.ClientSide,
	}
	cmpOpts := []cmp.Option{
		cmp.AllowUnexported(altsHandshaker{}),
		cmpopts.IgnoreFields(altsHandshaker{}, "conn", "clientConn"),
	}
	if got, want := hs.(*altsHandshaker), expectedHs; !cmp.Equal(got, want, cmpOpts...) {
		t.Errorf("NewClientHandshaker() returned unexpected handshaker: got: %v, want: %v", got, want)
	}
	if hs.(*altsHandshaker).stream != nil {
		t.Errorf("NewClientHandshaker() returned handshaker with non-nil stream")
	}
	if hs.(*altsHandshaker).clientConn != clientConn {
		t.Errorf("NewClientHandshaker() returned handshaker with unexpected clientConn")
	}
	hs.Close()
}

func (s) TestNewServerHandshaker(t *testing.T) {
	conn := testutil.NewTestConn(nil, nil)
	clientConn := &grpc.ClientConn{}
	opts := &ServerHandshakerOptions{}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	hs, err := NewServerHandshaker(ctx, clientConn, conn, opts)
	if err != nil {
		t.Errorf("NewServerHandshaker returned unexpected error: %v", err)
	}
	expectedHs := &altsHandshaker{
		stream:     nil,
		conn:       conn,
		clientConn: clientConn,
		clientOpts: nil,
		serverOpts: opts,
		side:       core.ServerSide,
	}
	cmpOpts := []cmp.Option{
		cmp.AllowUnexported(altsHandshaker{}),
		cmpopts.IgnoreFields(altsHandshaker{}, "conn", "clientConn"),
	}
	if got, want := hs.(*altsHandshaker), expectedHs; !cmp.Equal(got, want, cmpOpts...) {
		t.Errorf("NewServerHandshaker() returned unexpected handshaker: got: %v, want: %v", got, want)
	}
	if hs.(*altsHandshaker).stream != nil {
		t.Errorf("NewServerHandshaker() returned handshaker with non-nil stream")
	}
	if hs.(*altsHandshaker).clientConn != clientConn {
		t.Errorf("NewServerHandshaker() returned handshaker with unexpected clientConn")
	}
	hs.Close()
}

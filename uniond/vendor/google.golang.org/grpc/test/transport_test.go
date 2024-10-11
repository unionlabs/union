/*
*
* Copyright 2023 gRPC authors.
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
	"io"
	"net"
	"sync"
	"testing"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/internal/grpcsync"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/transport"
	"google.golang.org/grpc/status"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

// connWrapperWithCloseCh wraps a net.Conn and fires an event when closed.
type connWrapperWithCloseCh struct {
	net.Conn
	close *grpcsync.Event
}

// Close closes the connection and sends a value on the close channel.
func (cw *connWrapperWithCloseCh) Close() error {
	cw.close.Fire()
	return cw.Conn.Close()
}

// These custom creds are used for storing the connections made by the client.
// The closeCh in conn can be used to detect when conn is closed.
type transportRestartCheckCreds struct {
	mu          sync.Mutex
	connections []*connWrapperWithCloseCh
}

func (c *transportRestartCheckCreds) ServerHandshake(rawConn net.Conn) (net.Conn, credentials.AuthInfo, error) {
	return rawConn, nil, nil
}
func (c *transportRestartCheckCreds) ClientHandshake(_ context.Context, _ string, rawConn net.Conn) (net.Conn, credentials.AuthInfo, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	conn := &connWrapperWithCloseCh{Conn: rawConn, close: grpcsync.NewEvent()}
	c.connections = append(c.connections, conn)
	return conn, nil, nil
}
func (c *transportRestartCheckCreds) Info() credentials.ProtocolInfo {
	return credentials.ProtocolInfo{}
}
func (c *transportRestartCheckCreds) Clone() credentials.TransportCredentials {
	return c
}
func (c *transportRestartCheckCreds) OverrideServerName(string) error {
	return nil
}

// Tests that the client transport drains and restarts when next stream ID exceeds
// MaxStreamID. This test also verifies that subsequent RPCs use a new client
// transport and the old transport is closed.
func (s) TestClientTransportRestartsAfterStreamIDExhausted(t *testing.T) {
	// Set the transport's MaxStreamID to 4 to cause connection to drain after 2 RPCs.
	originalMaxStreamID := transport.MaxStreamID
	transport.MaxStreamID = 4
	defer func() {
		transport.MaxStreamID = originalMaxStreamID
	}()

	ss := &stubserver.StubServer{
		FullDuplexCallF: func(stream testgrpc.TestService_FullDuplexCallServer) error {
			if _, err := stream.Recv(); err != nil {
				return status.Errorf(codes.Internal, "unexpected error receiving: %v", err)
			}
			if err := stream.Send(&testpb.StreamingOutputCallResponse{}); err != nil {
				return status.Errorf(codes.Internal, "unexpected error sending: %v", err)
			}
			if recv, err := stream.Recv(); err != io.EOF {
				return status.Errorf(codes.Internal, "Recv = %v, %v; want _, io.EOF", recv, err)
			}
			return nil
		},
	}

	creds := &transportRestartCheckCreds{}
	if err := ss.Start(nil, grpc.WithTransportCredentials(creds)); err != nil {
		t.Fatalf("Starting stubServer: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	var streams []testgrpc.TestService_FullDuplexCallClient

	const numStreams = 3
	// expected number of conns when each stream is created i.e., 3rd stream is created
	// on a new connection.
	expectedNumConns := [numStreams]int{1, 1, 2}

	// Set up 3 streams.
	for i := 0; i < numStreams; i++ {
		s, err := ss.Client.FullDuplexCall(ctx)
		if err != nil {
			t.Fatalf("Creating FullDuplex stream: %v", err)
		}
		streams = append(streams, s)
		// Verify expected num of conns after each stream is created.
		if len(creds.connections) != expectedNumConns[i] {
			t.Fatalf("Got number of connections created: %v, want: %v", len(creds.connections), expectedNumConns[i])
		}
	}

	// Verify all streams still work.
	for i, stream := range streams {
		if err := stream.Send(&testpb.StreamingOutputCallRequest{}); err != nil {
			t.Fatalf("Sending on stream %d: %v", i, err)
		}
		if _, err := stream.Recv(); err != nil {
			t.Fatalf("Receiving on stream %d: %v", i, err)
		}
	}

	for i, stream := range streams {
		if err := stream.CloseSend(); err != nil {
			t.Fatalf("CloseSend() on stream %d: %v", i, err)
		}
	}

	// Verifying first connection was closed.
	select {
	case <-creds.connections[0].close.Done():
	case <-ctx.Done():
		t.Fatal("Timeout expired when waiting for first client transport to close")
	}
}

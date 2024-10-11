//go:build linux || windows
// +build linux windows

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

package alts

import (
	"context"
	"reflect"
	"sync"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/alts/internal/handshaker"
	"google.golang.org/grpc/credentials/alts/internal/handshaker/service"
	altsgrpc "google.golang.org/grpc/credentials/alts/internal/proto/grpc_gcp"
	altspb "google.golang.org/grpc/credentials/alts/internal/proto/grpc_gcp"
	"google.golang.org/grpc/credentials/alts/internal/testutil"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/testutils"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"
)

const (
	defaultTestLongTimeout  = 60 * time.Second
	defaultTestShortTimeout = 10 * time.Millisecond
)

type s struct {
	grpctest.Tester
}

func init() {
	// The vmOnGCP global variable MUST be forced to true. Otherwise, if
	// this test is run anywhere except on a GCP VM, then an ALTS handshake
	// will immediately fail.
	once.Do(func() {})
	vmOnGCP = true
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

func (s) TestInfoServerName(t *testing.T) {
	// This is not testing any handshaker functionality, so it's fine to only
	// use NewServerCreds and not NewClientCreds.
	alts := NewServerCreds(DefaultServerOptions())
	if got, want := alts.Info().ServerName, ""; got != want {
		t.Fatalf("%v.Info().ServerName = %v, want %v", alts, got, want)
	}
}

func (s) TestOverrideServerName(t *testing.T) {
	wantServerName := "server.name"
	// This is not testing any handshaker functionality, so it's fine to only
	// use NewServerCreds and not NewClientCreds.
	c := NewServerCreds(DefaultServerOptions())
	c.OverrideServerName(wantServerName)
	if got, want := c.Info().ServerName, wantServerName; got != want {
		t.Fatalf("c.Info().ServerName = %v, want %v", got, want)
	}
}

func (s) TestCloneClient(t *testing.T) {
	wantServerName := "server.name"
	opt := DefaultClientOptions()
	opt.TargetServiceAccounts = []string{"not", "empty"}
	c := NewClientCreds(opt)
	c.OverrideServerName(wantServerName)
	cc := c.Clone()
	if got, want := cc.Info().ServerName, wantServerName; got != want {
		t.Fatalf("cc.Info().ServerName = %v, want %v", got, want)
	}
	cc.OverrideServerName("")
	if got, want := c.Info().ServerName, wantServerName; got != want {
		t.Fatalf("Change in clone should not affect the original, c.Info().ServerName = %v, want %v", got, want)
	}
	if got, want := cc.Info().ServerName, ""; got != want {
		t.Fatalf("cc.Info().ServerName = %v, want %v", got, want)
	}

	ct := c.(*altsTC)
	cct := cc.(*altsTC)

	if ct.side != cct.side {
		t.Errorf("cc.side = %q, want %q", cct.side, ct.side)
	}
	if ct.hsAddress != cct.hsAddress {
		t.Errorf("cc.hsAddress = %q, want %q", cct.hsAddress, ct.hsAddress)
	}
	if !reflect.DeepEqual(ct.accounts, cct.accounts) {
		t.Errorf("cc.accounts = %q, want %q", cct.accounts, ct.accounts)
	}
}

func (s) TestCloneServer(t *testing.T) {
	wantServerName := "server.name"
	c := NewServerCreds(DefaultServerOptions())
	c.OverrideServerName(wantServerName)
	cc := c.Clone()
	if got, want := cc.Info().ServerName, wantServerName; got != want {
		t.Fatalf("cc.Info().ServerName = %v, want %v", got, want)
	}
	cc.OverrideServerName("")
	if got, want := c.Info().ServerName, wantServerName; got != want {
		t.Fatalf("Change in clone should not affect the original, c.Info().ServerName = %v, want %v", got, want)
	}
	if got, want := cc.Info().ServerName, ""; got != want {
		t.Fatalf("cc.Info().ServerName = %v, want %v", got, want)
	}

	ct := c.(*altsTC)
	cct := cc.(*altsTC)

	if ct.side != cct.side {
		t.Errorf("cc.side = %q, want %q", cct.side, ct.side)
	}
	if ct.hsAddress != cct.hsAddress {
		t.Errorf("cc.hsAddress = %q, want %q", cct.hsAddress, ct.hsAddress)
	}
	if !reflect.DeepEqual(ct.accounts, cct.accounts) {
		t.Errorf("cc.accounts = %q, want %q", cct.accounts, ct.accounts)
	}
}

func (s) TestInfo(t *testing.T) {
	// This is not testing any handshaker functionality, so it's fine to only
	// use NewServerCreds and not NewClientCreds.
	c := NewServerCreds(DefaultServerOptions())
	info := c.Info()
	if got, want := info.ProtocolVersion, ""; got != want {
		t.Errorf("info.ProtocolVersion=%v, want %v", got, want)
	}
	if got, want := info.SecurityProtocol, "alts"; got != want {
		t.Errorf("info.SecurityProtocol=%v, want %v", got, want)
	}
	if got, want := info.SecurityVersion, "1.0"; got != want {
		t.Errorf("info.SecurityVersion=%v, want %v", got, want)
	}
	if got, want := info.ServerName, ""; got != want {
		t.Errorf("info.ServerName=%v, want %v", got, want)
	}
}

func (s) TestCompareRPCVersions(t *testing.T) {
	for _, tc := range []struct {
		v1     *altspb.RpcProtocolVersions_Version
		v2     *altspb.RpcProtocolVersions_Version
		output int
	}{
		{
			version(3, 2),
			version(2, 1),
			1,
		},
		{
			version(3, 2),
			version(3, 1),
			1,
		},
		{
			version(2, 1),
			version(3, 2),
			-1,
		},
		{
			version(3, 1),
			version(3, 2),
			-1,
		},
		{
			version(3, 2),
			version(3, 2),
			0,
		},
	} {
		if got, want := compareRPCVersions(tc.v1, tc.v2), tc.output; got != want {
			t.Errorf("compareRPCVersions(%v, %v)=%v, want %v", tc.v1, tc.v2, got, want)
		}
	}
}

func (s) TestCheckRPCVersions(t *testing.T) {
	for _, tc := range []struct {
		desc             string
		local            *altspb.RpcProtocolVersions
		peer             *altspb.RpcProtocolVersions
		output           bool
		maxCommonVersion *altspb.RpcProtocolVersions_Version
	}{
		{
			"local.max > peer.max and local.min > peer.min",
			versions(2, 1, 3, 2),
			versions(1, 2, 2, 1),
			true,
			version(2, 1),
		},
		{
			"local.max > peer.max and local.min < peer.min",
			versions(1, 2, 3, 2),
			versions(2, 1, 2, 1),
			true,
			version(2, 1),
		},
		{
			"local.max > peer.max and local.min = peer.min",
			versions(2, 1, 3, 2),
			versions(2, 1, 2, 1),
			true,
			version(2, 1),
		},
		{
			"local.max < peer.max and local.min > peer.min",
			versions(2, 1, 2, 1),
			versions(1, 2, 3, 2),
			true,
			version(2, 1),
		},
		{
			"local.max = peer.max and local.min > peer.min",
			versions(2, 1, 2, 1),
			versions(1, 2, 2, 1),
			true,
			version(2, 1),
		},
		{
			"local.max < peer.max and local.min < peer.min",
			versions(1, 2, 2, 1),
			versions(2, 1, 3, 2),
			true,
			version(2, 1),
		},
		{
			"local.max < peer.max and local.min = peer.min",
			versions(1, 2, 2, 1),
			versions(1, 2, 3, 2),
			true,
			version(2, 1),
		},
		{
			"local.max = peer.max and local.min < peer.min",
			versions(1, 2, 2, 1),
			versions(2, 1, 2, 1),
			true,
			version(2, 1),
		},
		{
			"all equal",
			versions(2, 1, 2, 1),
			versions(2, 1, 2, 1),
			true,
			version(2, 1),
		},
		{
			"max is smaller than min",
			versions(2, 1, 1, 2),
			versions(2, 1, 1, 2),
			false,
			nil,
		},
		{
			"no overlap, local > peer",
			versions(4, 3, 6, 5),
			versions(1, 0, 2, 1),
			false,
			nil,
		},
		{
			"no overlap, local < peer",
			versions(1, 0, 2, 1),
			versions(4, 3, 6, 5),
			false,
			nil,
		},
		{
			"no overlap, max < min",
			versions(6, 5, 4, 3),
			versions(2, 1, 1, 0),
			false,
			nil,
		},
	} {
		output, maxCommonVersion := checkRPCVersions(tc.local, tc.peer)
		if got, want := output, tc.output; got != want {
			t.Errorf("%v: checkRPCVersions(%v, %v)=(%v, _), want (%v, _)", tc.desc, tc.local, tc.peer, got, want)
		}
		if got, want := maxCommonVersion, tc.maxCommonVersion; !proto.Equal(got, want) {
			t.Errorf("%v: checkRPCVersions(%v, %v)=(_, %v), want (_, %v)", tc.desc, tc.local, tc.peer, got, want)
		}
	}
}

// TestFullHandshake performs a full ALTS handshake between a test client and
// server, where both client and server offload to a local, fake handshaker
// service.
func (s) TestFullHandshake(t *testing.T) {
	// Start the fake handshaker service and the server.
	var wait sync.WaitGroup
	defer wait.Wait()
	stopHandshaker, handshakerAddress := startFakeHandshakerService(t, &wait)
	defer stopHandshaker()
	stopServer, serverAddress := startServer(t, handshakerAddress, &wait)
	defer stopServer()

	// Ping the server, authenticating with ALTS.
	establishAltsConnection(t, handshakerAddress, serverAddress)

	// Close open connections to the fake handshaker service.
	if err := service.CloseForTesting(); err != nil {
		t.Errorf("service.CloseForTesting() failed: %v", err)
	}
}

// TestConcurrentHandshakes performs a several, concurrent ALTS handshakes
// between a test client and server, where both client and server offload to a
// local, fake handshaker service.
func (s) TestConcurrentHandshakes(t *testing.T) {
	// Set the max number of concurrent handshakes to 3, so that we can
	// test the handshaker behavior when handshakes are queued by
	// performing more than 3 concurrent handshakes (specifically, 10).
	handshaker.ResetConcurrentHandshakeSemaphoreForTesting(3)

	// Start the fake handshaker service and the server.
	var wait sync.WaitGroup
	defer wait.Wait()
	stopHandshaker, handshakerAddress := startFakeHandshakerService(t, &wait)
	defer stopHandshaker()
	stopServer, serverAddress := startServer(t, handshakerAddress, &wait)
	defer stopServer()

	// Ping the server, authenticating with ALTS.
	var waitForConnections sync.WaitGroup
	for i := 0; i < 10; i++ {
		waitForConnections.Add(1)
		go func() {
			establishAltsConnection(t, handshakerAddress, serverAddress)
			waitForConnections.Done()
		}()
	}
	waitForConnections.Wait()

	// Close open connections to the fake handshaker service.
	if err := service.CloseForTesting(); err != nil {
		t.Errorf("service.CloseForTesting() failed: %v", err)
	}
}

func version(major, minor uint32) *altspb.RpcProtocolVersions_Version {
	return &altspb.RpcProtocolVersions_Version{
		Major: major,
		Minor: minor,
	}
}

func versions(minMajor, minMinor, maxMajor, maxMinor uint32) *altspb.RpcProtocolVersions {
	return &altspb.RpcProtocolVersions{
		MinRpcVersion: version(minMajor, minMinor),
		MaxRpcVersion: version(maxMajor, maxMinor),
	}
}

func establishAltsConnection(t *testing.T, handshakerAddress, serverAddress string) {
	clientCreds := NewClientCreds(&ClientOptions{HandshakerServiceAddress: handshakerAddress})
	conn, err := grpc.NewClient(serverAddress, grpc.WithTransportCredentials(clientCreds))
	if err != nil {
		t.Fatalf("grpc.NewClient(%v) failed: %v", serverAddress, err)
	}
	defer conn.Close()
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestLongTimeout)
	defer cancel()
	c := testgrpc.NewTestServiceClient(conn)
	var peer peer.Peer
	success := false
	for ; ctx.Err() == nil; <-time.After(defaultTestShortTimeout) {
		_, err = c.UnaryCall(ctx, &testpb.SimpleRequest{}, grpc.Peer(&peer))
		if err == nil {
			success = true
			break
		}
		if code := status.Code(err); code == codes.Unavailable || code == codes.DeadlineExceeded {
			// The server is not ready yet or there were too many concurrent handshakes.
			// Try again.
			continue
		}
		t.Fatalf("c.UnaryCall() failed: %v", err)
	}
	if !success {
		t.Fatalf("c.UnaryCall() timed out after %v", defaultTestShortTimeout)
	}

	// Check that peer.AuthInfo was populated with an ALTS AuthInfo
	// instance. As a sanity check, also verify that the AuthType() and
	// ApplicationProtocol() have the expected values.
	if got, want := peer.AuthInfo.AuthType(), "alts"; got != want {
		t.Errorf("authInfo.AuthType() = %s, want = %s", got, want)
	}
	authInfo, err := AuthInfoFromPeer(&peer)
	if err != nil {
		t.Errorf("AuthInfoFromPeer failed: %v", err)
	}
	if got, want := authInfo.ApplicationProtocol(), "grpc"; got != want {
		t.Errorf("authInfo.ApplicationProtocol() = %s, want = %s", got, want)
	}
}

func startFakeHandshakerService(t *testing.T, wait *sync.WaitGroup) (stop func(), address string) {
	listener, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("LocalTCPListener() failed: %v", err)
	}
	s := grpc.NewServer()
	altsgrpc.RegisterHandshakerServiceServer(s, &testutil.FakeHandshaker{})
	wait.Add(1)
	go func() {
		defer wait.Done()
		if err := s.Serve(listener); err != nil {
			t.Errorf("failed to serve: %v", err)
		}
	}()
	return func() { s.Stop() }, listener.Addr().String()
}

func startServer(t *testing.T, handshakerServiceAddress string, wait *sync.WaitGroup) (stop func(), address string) {
	listener, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("LocalTCPListener() failed: %v", err)
	}
	serverOpts := &ServerOptions{HandshakerServiceAddress: handshakerServiceAddress}
	creds := NewServerCreds(serverOpts)
	s := grpc.NewServer(grpc.Creds(creds))
	testgrpc.RegisterTestServiceServer(s, &testServer{})
	wait.Add(1)
	go func() {
		defer wait.Done()
		if err := s.Serve(listener); err != nil {
			t.Errorf("s.Serve(%v) failed: %v", listener, err)
		}
	}()
	return func() { s.Stop() }, listener.Addr().String()
}

type testServer struct {
	testgrpc.UnimplementedTestServiceServer
}

func (s *testServer) UnaryCall(_ context.Context, _ *testpb.SimpleRequest) (*testpb.SimpleResponse, error) {
	return &testpb.SimpleResponse{
		Payload: &testpb.Payload{},
	}, nil
}

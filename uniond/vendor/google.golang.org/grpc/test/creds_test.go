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
	"errors"
	"fmt"
	"net"
	"strings"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/resolver/manual"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/tap"
	"google.golang.org/grpc/testdata"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

const (
	bundlePerRPCOnly = "perRPCOnly"
	bundleTLSOnly    = "tlsOnly"
)

type testCredsBundle struct {
	t    *testing.T
	mode string
}

func (c *testCredsBundle) TransportCredentials() credentials.TransportCredentials {
	if c.mode == bundlePerRPCOnly {
		return insecure.NewCredentials()
	}

	creds, err := credentials.NewClientTLSFromFile(testdata.Path("x509/server_ca_cert.pem"), "x.test.example.com")
	if err != nil {
		c.t.Logf("Failed to load credentials: %v", err)
		return nil
	}
	return creds
}

func (c *testCredsBundle) PerRPCCredentials() credentials.PerRPCCredentials {
	if c.mode == bundleTLSOnly {
		return nil
	}
	return testPerRPCCredentials{authdata: authdata}
}

func (c *testCredsBundle) NewWithMode(mode string) (credentials.Bundle, error) {
	return &testCredsBundle{mode: mode}, nil
}

func (s) TestCredsBundleBoth(t *testing.T) {
	te := newTest(t, env{name: "creds-bundle", network: "tcp", security: "empty"})
	te.tapHandle = authHandle
	te.customDialOptions = []grpc.DialOption{
		grpc.WithCredentialsBundle(&testCredsBundle{t: t}),
	}
	creds, err := credentials.NewServerTLSFromFile(testdata.Path("x509/server1_cert.pem"), testdata.Path("x509/server1_key.pem"))
	if err != nil {
		t.Fatalf("Failed to generate credentials %v", err)
	}
	te.customServerOptions = []grpc.ServerOption{
		grpc.Creds(creds),
	}
	te.startServer(&testServer{})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("Test failed. Reason: %v", err)
	}
}

func (s) TestCredsBundleTransportCredentials(t *testing.T) {
	te := newTest(t, env{name: "creds-bundle", network: "tcp", security: "empty"})
	te.customDialOptions = []grpc.DialOption{
		grpc.WithCredentialsBundle(&testCredsBundle{t: t, mode: bundleTLSOnly}),
	}
	creds, err := credentials.NewServerTLSFromFile(testdata.Path("x509/server1_cert.pem"), testdata.Path("x509/server1_key.pem"))
	if err != nil {
		t.Fatalf("Failed to generate credentials %v", err)
	}
	te.customServerOptions = []grpc.ServerOption{
		grpc.Creds(creds),
	}
	te.startServer(&testServer{})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("Test failed. Reason: %v", err)
	}
}

func (s) TestCredsBundlePerRPCCredentials(t *testing.T) {
	te := newTest(t, env{name: "creds-bundle", network: "tcp", security: "empty"})
	te.tapHandle = authHandle
	te.customDialOptions = []grpc.DialOption{
		grpc.WithCredentialsBundle(&testCredsBundle{t: t, mode: bundlePerRPCOnly}),
	}
	te.startServer(&testServer{})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("Test failed. Reason: %v", err)
	}
}

type clientTimeoutCreds struct {
	credentials.TransportCredentials
	timeoutReturned bool
}

func (c *clientTimeoutCreds) ClientHandshake(_ context.Context, _ string, rawConn net.Conn) (net.Conn, credentials.AuthInfo, error) {
	if !c.timeoutReturned {
		c.timeoutReturned = true
		return nil, nil, context.DeadlineExceeded
	}
	return rawConn, nil, nil
}

func (c *clientTimeoutCreds) Info() credentials.ProtocolInfo {
	return credentials.ProtocolInfo{}
}

func (c *clientTimeoutCreds) Clone() credentials.TransportCredentials {
	return nil
}

func (s) TestNonFailFastRPCSucceedOnTimeoutCreds(t *testing.T) {
	te := newTest(t, env{name: "timeout-cred", network: "tcp", security: "empty"})
	te.userAgent = testAppUA
	te.startServer(&testServer{security: te.e.security})
	defer te.tearDown()

	cc := te.clientConn(grpc.WithTransportCredentials(&clientTimeoutCreds{}))
	tc := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	// This unary call should succeed, because ClientHandshake will succeed for the second time.
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		te.t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want <nil>", err)
	}
}

type methodTestCreds struct{}

func (m *methodTestCreds) GetRequestMetadata(ctx context.Context, _ ...string) (map[string]string, error) {
	ri, _ := credentials.RequestInfoFromContext(ctx)
	return nil, status.Errorf(codes.Unknown, ri.Method)
}

func (m *methodTestCreds) RequireTransportSecurity() bool { return false }

func (s) TestGRPCMethodAccessibleToCredsViaContextRequestInfo(t *testing.T) {
	const wantMethod = "/grpc.testing.TestService/EmptyCall"
	te := newTest(t, env{name: "context-request-info", network: "tcp"})
	te.userAgent = testAppUA
	te.startServer(&testServer{security: te.e.security})
	defer te.tearDown()

	cc := te.clientConn(grpc.WithPerRPCCredentials(&methodTestCreds{}))
	tc := testgrpc.NewTestServiceClient(cc)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); status.Convert(err).Message() != wantMethod {
		t.Fatalf("ss.client.EmptyCall(_, _) = _, %v; want _, _.Message()=%q", err, wantMethod)
	}

	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); status.Convert(err).Message() != wantMethod {
		t.Fatalf("ss.client.EmptyCall(_, _) = _, %v; want _, _.Message()=%q", err, wantMethod)
	}
}

const clientAlwaysFailCredErrorMsg = "clientAlwaysFailCred always fails"

type clientAlwaysFailCred struct {
	credentials.TransportCredentials
}

func (c clientAlwaysFailCred) ClientHandshake(context.Context, string, net.Conn) (net.Conn, credentials.AuthInfo, error) {
	return nil, nil, errors.New(clientAlwaysFailCredErrorMsg)
}
func (c clientAlwaysFailCred) Info() credentials.ProtocolInfo {
	return credentials.ProtocolInfo{}
}
func (c clientAlwaysFailCred) Clone() credentials.TransportCredentials {
	return nil
}

func (s) TestFailFastRPCErrorOnBadCertificates(t *testing.T) {
	te := newTest(t, env{name: "bad-cred", network: "tcp", security: "empty", balancer: "round_robin"})
	te.startServer(&testServer{security: te.e.security})
	defer te.tearDown()

	opts := []grpc.DialOption{grpc.WithTransportCredentials(clientAlwaysFailCred{})}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	cc, err := grpc.DialContext(ctx, te.srvAddr, opts...)
	if err != nil {
		t.Fatalf("Dial(_) = %v, want %v", err, nil)
	}
	defer cc.Close()

	tc := testgrpc.NewTestServiceClient(cc)
	for i := 0; i < 1000; i++ {
		// This loop runs for at most 1 second. The first several RPCs will fail
		// with Unavailable because the connection hasn't started. When the
		// first connection failed with creds error, the next RPC should also
		// fail with the expected error.
		if _, err = tc.EmptyCall(ctx, &testpb.Empty{}); strings.Contains(err.Error(), clientAlwaysFailCredErrorMsg) {
			return
		}
		time.Sleep(time.Millisecond)
	}
	te.t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want err.Error() contains %q", err, clientAlwaysFailCredErrorMsg)
}

func (s) TestWaitForReadyRPCErrorOnBadCertificates(t *testing.T) {
	te := newTest(t, env{name: "bad-cred", network: "tcp", security: "empty", balancer: "round_robin"})
	te.startServer(&testServer{security: te.e.security})
	defer te.tearDown()

	opts := []grpc.DialOption{grpc.WithTransportCredentials(clientAlwaysFailCred{})}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	cc, err := grpc.DialContext(ctx, te.srvAddr, opts...)
	if err != nil {
		t.Fatalf("Dial(_) = %v, want %v", err, nil)
	}
	defer cc.Close()

	tc := testgrpc.NewTestServiceClient(cc)
	ctx, cancel = context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer cancel()
	if _, err = tc.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); !strings.Contains(err.Error(), clientAlwaysFailCredErrorMsg) {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want err.Error() contains %q", err, clientAlwaysFailCredErrorMsg)
	}
}

var (
	// test authdata
	authdata = map[string]string{
		"test-key":      "test-value",
		"test-key2-bin": string([]byte{1, 2, 3}),
	}
)

type testPerRPCCredentials struct {
	authdata map[string]string
	errChan  chan error
}

func (cr testPerRPCCredentials) GetRequestMetadata(context.Context, ...string) (map[string]string, error) {
	var err error
	if cr.errChan != nil {
		err = <-cr.errChan
	}
	return cr.authdata, err
}

func (cr testPerRPCCredentials) RequireTransportSecurity() bool {
	return false
}

func authHandle(ctx context.Context, _ *tap.Info) (context.Context, error) {
	md, ok := metadata.FromIncomingContext(ctx)
	if !ok {
		return ctx, fmt.Errorf("didn't find metadata in context")
	}
	for k, vwant := range authdata {
		vgot, ok := md[k]
		if !ok {
			return ctx, fmt.Errorf("didn't find authdata key %v in context", k)
		}
		if vgot[0] != vwant {
			return ctx, fmt.Errorf("for key %v, got value %v, want %v", k, vgot, vwant)
		}
	}
	return ctx, nil
}

func (s) TestPerRPCCredentialsViaDialOptions(t *testing.T) {
	for _, e := range listTestEnv() {
		testPerRPCCredentialsViaDialOptions(t, e)
	}
}

func testPerRPCCredentialsViaDialOptions(t *testing.T, e env) {
	te := newTest(t, e)
	te.tapHandle = authHandle
	te.perRPCCreds = testPerRPCCredentials{authdata: authdata}
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("Test failed. Reason: %v", err)
	}
}

func (s) TestPerRPCCredentialsViaCallOptions(t *testing.T) {
	for _, e := range listTestEnv() {
		testPerRPCCredentialsViaCallOptions(t, e)
	}
}

func testPerRPCCredentialsViaCallOptions(t *testing.T, e env) {
	te := newTest(t, e)
	te.tapHandle = authHandle
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}, grpc.PerRPCCredentials(testPerRPCCredentials{authdata: authdata})); err != nil {
		t.Fatalf("Test failed. Reason: %v", err)
	}
}

func (s) TestPerRPCCredentialsViaDialOptionsAndCallOptions(t *testing.T) {
	for _, e := range listTestEnv() {
		testPerRPCCredentialsViaDialOptionsAndCallOptions(t, e)
	}
}

func testPerRPCCredentialsViaDialOptionsAndCallOptions(t *testing.T, e env) {
	te := newTest(t, e)
	te.perRPCCreds = testPerRPCCredentials{authdata: authdata}
	// When credentials are provided via both dial options and call options,
	// we apply both sets.
	te.tapHandle = func(ctx context.Context, _ *tap.Info) (context.Context, error) {
		md, ok := metadata.FromIncomingContext(ctx)
		if !ok {
			return ctx, fmt.Errorf("couldn't find metadata in context")
		}
		for k, vwant := range authdata {
			vgot, ok := md[k]
			if !ok {
				return ctx, fmt.Errorf("couldn't find metadata for key %v", k)
			}
			if len(vgot) != 2 {
				return ctx, fmt.Errorf("len of value for key %v was %v, want 2", k, len(vgot))
			}
			if vgot[0] != vwant || vgot[1] != vwant {
				return ctx, fmt.Errorf("value for %v was %v, want [%v, %v]", k, vgot, vwant, vwant)
			}
		}
		return ctx, nil
	}
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}, grpc.PerRPCCredentials(testPerRPCCredentials{authdata: authdata})); err != nil {
		t.Fatalf("Test failed. Reason: %v", err)
	}
}

const testAuthority = "test.auth.ori.ty"

type authorityCheckCreds struct {
	credentials.TransportCredentials
	got string
}

func (c *authorityCheckCreds) ClientHandshake(_ context.Context, authority string, rawConn net.Conn) (net.Conn, credentials.AuthInfo, error) {
	c.got = authority
	return rawConn, nil, nil
}
func (c *authorityCheckCreds) Info() credentials.ProtocolInfo {
	return credentials.ProtocolInfo{}
}
func (c *authorityCheckCreds) Clone() credentials.TransportCredentials {
	return c
}

// This test makes sure that the authority client handshake gets is the endpoint
// in dial target, not the resolved ip address.
func (s) TestCredsHandshakeAuthority(t *testing.T) {
	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatal(err)
	}
	cred := &authorityCheckCreds{}
	s := grpc.NewServer()
	go s.Serve(lis)
	defer s.Stop()

	r := manual.NewBuilderWithScheme("whatever")

	cc, err := grpc.Dial(r.Scheme()+":///"+testAuthority, grpc.WithTransportCredentials(cred), grpc.WithResolvers(r))
	if err != nil {
		t.Fatalf("grpc.Dial(%q) = %v", lis.Addr().String(), err)
	}
	defer cc.Close()
	r.UpdateState(resolver.State{Addresses: []resolver.Address{{Addr: lis.Addr().String()}}})

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	testutils.AwaitState(ctx, t, cc, connectivity.Ready)

	if cred.got != testAuthority {
		t.Fatalf("client creds got authority: %q, want: %q", cred.got, testAuthority)
	}
}

// This test makes sure that the authority client handshake gets is the endpoint
// of the ServerName of the address when it is set.
func (s) TestCredsHandshakeServerNameAuthority(t *testing.T) {
	const testServerName = "test.server.name"

	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatal(err)
	}
	cred := &authorityCheckCreds{}
	s := grpc.NewServer()
	go s.Serve(lis)
	defer s.Stop()

	r := manual.NewBuilderWithScheme("whatever")

	cc, err := grpc.Dial(r.Scheme()+":///"+testAuthority, grpc.WithTransportCredentials(cred), grpc.WithResolvers(r))
	if err != nil {
		t.Fatalf("grpc.Dial(%q) = %v", lis.Addr().String(), err)
	}
	defer cc.Close()
	r.UpdateState(resolver.State{Addresses: []resolver.Address{{Addr: lis.Addr().String(), ServerName: testServerName}}})

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	testutils.AwaitState(ctx, t, cc, connectivity.Ready)

	if cred.got != testServerName {
		t.Fatalf("client creds got authority: %q, want: %q", cred.got, testAuthority)
	}
}

type serverDispatchCred struct {
	rawConnCh chan net.Conn
}

func (c *serverDispatchCred) ClientHandshake(_ context.Context, _ string, rawConn net.Conn) (net.Conn, credentials.AuthInfo, error) {
	return rawConn, nil, nil
}
func (c *serverDispatchCred) ServerHandshake(rawConn net.Conn) (net.Conn, credentials.AuthInfo, error) {
	select {
	case c.rawConnCh <- rawConn:
	default:
	}
	return nil, nil, credentials.ErrConnDispatched
}
func (c *serverDispatchCred) Info() credentials.ProtocolInfo {
	return credentials.ProtocolInfo{}
}
func (c *serverDispatchCred) Clone() credentials.TransportCredentials {
	return nil
}
func (c *serverDispatchCred) OverrideServerName(string) error {
	return nil
}
func (c *serverDispatchCred) getRawConn() net.Conn {
	return <-c.rawConnCh
}

func (s) TestServerCredsDispatch(t *testing.T) {
	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatal(err)
	}
	cred := &serverDispatchCred{
		rawConnCh: make(chan net.Conn, 1),
	}
	s := grpc.NewServer(grpc.Creds(cred))
	go s.Serve(lis)
	defer s.Stop()

	cc, err := grpc.Dial(lis.Addr().String(), grpc.WithTransportCredentials(cred))
	if err != nil {
		t.Fatalf("grpc.Dial(%q) = %v", lis.Addr().String(), err)
	}
	defer cc.Close()

	rawConn := cred.getRawConn()
	// Give grpc a chance to see the error and potentially close the connection.
	// And check that connection is not closed after that.
	time.Sleep(100 * time.Millisecond)
	// Check rawConn is not closed.
	if n, err := rawConn.Write([]byte{0}); n <= 0 || err != nil {
		t.Errorf("Read() = %v, %v; want n>0, <nil>", n, err)
	}
}

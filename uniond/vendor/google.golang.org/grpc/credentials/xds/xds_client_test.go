/*
 *
 * Copyright 2020 gRPC authors.
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

package xds

import (
	"context"
	"crypto/tls"
	"crypto/x509"
	"errors"
	"fmt"
	"net"
	"os"
	"strings"
	"sync/atomic"
	"testing"
	"time"
	"unsafe"

	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/tls/certprovider"
	icredentials "google.golang.org/grpc/internal/credentials"
	xdsinternal "google.golang.org/grpc/internal/credentials/xds"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/xds/matcher"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/testdata"
)

const (
	defaultTestTimeout      = 1 * time.Second
	defaultTestShortTimeout = 10 * time.Millisecond
	defaultTestCertSAN      = "abc.test.example.com"
	authority               = "authority"
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

// Helper function to create a real TLS client credentials which is used as
// fallback credentials from multiple tests.
func makeFallbackClientCreds(t *testing.T) credentials.TransportCredentials {
	creds, err := credentials.NewClientTLSFromFile(testdata.Path("x509/server_ca_cert.pem"), "x.test.example.com")
	if err != nil {
		t.Fatal(err)
	}
	return creds
}

// testServer is a no-op server which listens on a local TCP port for incoming
// connections, and performs a manual TLS handshake on the received raw
// connection using a user specified handshake function. It then makes the
// result of the handshake operation available through a channel for tests to
// inspect. Tests should stop the testServer as part of their cleanup.
type testServer struct {
	lis           net.Listener
	address       string             // Listening address of the test server.
	handshakeFunc testHandshakeFunc  // Test specified handshake function.
	hsResult      *testutils.Channel // Channel to deliver handshake results.
}

// handshakeResult wraps the result of the handshake operation on the test
// server. It consists of TLS connection state and an error, if the handshake
// failed. This result is delivered on the `hsResult` channel on the testServer.
type handshakeResult struct {
	connState tls.ConnectionState
	err       error
}

// Configurable handshake function for the testServer. Tests can set this to
// simulate different conditions like handshake success, failure, timeout etc.
type testHandshakeFunc func(net.Conn) handshakeResult

// newTestServerWithHandshakeFunc starts a new testServer which listens for
// connections on a local TCP port, and uses the provided custom handshake
// function to perform TLS handshake.
func newTestServerWithHandshakeFunc(f testHandshakeFunc) *testServer {
	ts := &testServer{
		handshakeFunc: f,
		hsResult:      testutils.NewChannel(),
	}
	ts.start()
	return ts
}

// starts actually starts listening on a local TCP port, and spawns a goroutine
// to handle new connections.
func (ts *testServer) start() error {
	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		return err
	}
	ts.lis = lis
	ts.address = lis.Addr().String()
	go ts.handleConn()
	return nil
}

// handleconn accepts a new raw connection, and invokes the test provided
// handshake function to perform TLS handshake, and returns the result on the
// `hsResult` channel.
func (ts *testServer) handleConn() {
	for {
		rawConn, err := ts.lis.Accept()
		if err != nil {
			// Once the listeners closed, Accept() will return with an error.
			return
		}
		hsr := ts.handshakeFunc(rawConn)
		ts.hsResult.Send(hsr)
	}
}

// stop closes the associated listener which causes the connection handling
// goroutine to exit.
func (ts *testServer) stop() {
	ts.lis.Close()
}

// A handshake function which simulates a successful handshake without client
// authentication (server does not request for client certificate during the
// handshake here).
func testServerTLSHandshake(rawConn net.Conn) handshakeResult {
	cert, err := tls.LoadX509KeyPair(testdata.Path("x509/server1_cert.pem"), testdata.Path("x509/server1_key.pem"))
	if err != nil {
		return handshakeResult{err: err}
	}
	cfg := &tls.Config{
		Certificates: []tls.Certificate{cert},
		NextProtos:   []string{"h2"},
	}
	conn := tls.Server(rawConn, cfg)
	if err := conn.Handshake(); err != nil {
		return handshakeResult{err: err}
	}
	return handshakeResult{connState: conn.ConnectionState()}
}

// A handshake function which simulates a successful handshake with mutual
// authentication.
func testServerMutualTLSHandshake(rawConn net.Conn) handshakeResult {
	cert, err := tls.LoadX509KeyPair(testdata.Path("x509/server1_cert.pem"), testdata.Path("x509/server1_key.pem"))
	if err != nil {
		return handshakeResult{err: err}
	}
	pemData, err := os.ReadFile(testdata.Path("x509/client_ca_cert.pem"))
	if err != nil {
		return handshakeResult{err: err}
	}
	roots := x509.NewCertPool()
	roots.AppendCertsFromPEM(pemData)
	cfg := &tls.Config{
		Certificates: []tls.Certificate{cert},
		ClientCAs:    roots,
	}
	conn := tls.Server(rawConn, cfg)
	if err := conn.Handshake(); err != nil {
		return handshakeResult{err: err}
	}
	return handshakeResult{connState: conn.ConnectionState()}
}

// fakeProvider is an implementation of the certprovider.Provider interface
// which returns the configured key material and error in calls to
// KeyMaterial().
type fakeProvider struct {
	km  *certprovider.KeyMaterial
	err error
}

func (f *fakeProvider) KeyMaterial(context.Context) (*certprovider.KeyMaterial, error) {
	return f.km, f.err
}

func (f *fakeProvider) Close() {}

// makeIdentityProvider creates a new instance of the fakeProvider returning the
// identity key material specified in the provider file paths.
func makeIdentityProvider(t *testing.T, certPath, keyPath string) certprovider.Provider {
	t.Helper()
	cert, err := tls.LoadX509KeyPair(testdata.Path(certPath), testdata.Path(keyPath))
	if err != nil {
		t.Fatal(err)
	}
	return &fakeProvider{km: &certprovider.KeyMaterial{Certs: []tls.Certificate{cert}}}
}

// makeRootProvider creates a new instance of the fakeProvider returning the
// root key material specified in the provider file paths.
func makeRootProvider(t *testing.T, caPath string) *fakeProvider {
	pemData, err := os.ReadFile(testdata.Path(caPath))
	if err != nil {
		t.Fatal(err)
	}
	roots := x509.NewCertPool()
	roots.AppendCertsFromPEM(pemData)
	return &fakeProvider{km: &certprovider.KeyMaterial{Roots: roots}}
}

// newTestContextWithHandshakeInfo returns a copy of parent with HandshakeInfo
// context value added to it.
func newTestContextWithHandshakeInfo(parent context.Context, root, identity certprovider.Provider, sanExactMatch string) context.Context {
	// Creating the HandshakeInfo and adding it to the attributes is very
	// similar to what the CDS balancer would do when it intercepts calls to
	// NewSubConn().
	var sms []matcher.StringMatcher
	if sanExactMatch != "" {
		sms = []matcher.StringMatcher{matcher.StringMatcherForTesting(newStringP(sanExactMatch), nil, nil, nil, nil, false)}
	}
	info := xdsinternal.NewHandshakeInfo(root, identity, sms, false)
	uPtr := unsafe.Pointer(info)
	addr := xdsinternal.SetHandshakeInfo(resolver.Address{}, &uPtr)

	// Moving the attributes from the resolver.Address to the context passed to
	// the handshaker is done in the transport layer. Since we directly call the
	// handshaker in these tests, we need to do the same here.
	return icredentials.NewClientHandshakeInfoContext(parent, credentials.ClientHandshakeInfo{Attributes: addr.Attributes})
}

// compareAuthInfo compares the AuthInfo received on the client side after a
// successful handshake with the authInfo available on the testServer.
func compareAuthInfo(ctx context.Context, ts *testServer, ai credentials.AuthInfo) error {
	if ai.AuthType() != "tls" {
		return fmt.Errorf("ClientHandshake returned authType %q, want %q", ai.AuthType(), "tls")
	}
	info, ok := ai.(credentials.TLSInfo)
	if !ok {
		return fmt.Errorf("ClientHandshake returned authInfo of type %T, want %T", ai, credentials.TLSInfo{})
	}
	gotState := info.State

	// Read the handshake result from the testServer which contains the TLS
	// connection state and compare it with the one received on the client-side.
	val, err := ts.hsResult.Receive(ctx)
	if err != nil {
		return fmt.Errorf("testServer failed to return handshake result: %v", err)
	}
	hsr := val.(handshakeResult)
	if hsr.err != nil {
		return fmt.Errorf("testServer handshake failure: %v", hsr.err)
	}
	// AuthInfo contains a variety of information. We only verify a subset here.
	// This is the same subset which is verified in TLS credentials tests.
	if err := compareConnState(gotState, hsr.connState); err != nil {
		return err
	}
	return nil
}

func compareConnState(got, want tls.ConnectionState) error {
	switch {
	case got.Version != want.Version:
		return fmt.Errorf("TLS.ConnectionState got Version: %v, want: %v", got.Version, want.Version)
	case got.HandshakeComplete != want.HandshakeComplete:
		return fmt.Errorf("TLS.ConnectionState got HandshakeComplete: %v, want: %v", got.HandshakeComplete, want.HandshakeComplete)
	case got.CipherSuite != want.CipherSuite:
		return fmt.Errorf("TLS.ConnectionState got CipherSuite: %v, want: %v", got.CipherSuite, want.CipherSuite)
	case got.NegotiatedProtocol != want.NegotiatedProtocol:
		return fmt.Errorf("TLS.ConnectionState got NegotiatedProtocol: %v, want: %v", got.NegotiatedProtocol, want.NegotiatedProtocol)
	}
	return nil
}

// TestClientCredsWithoutFallback verifies that the call to
// NewClientCredentials() fails when no fallback is specified.
func (s) TestClientCredsWithoutFallback(t *testing.T) {
	if _, err := NewClientCredentials(ClientOptions{}); err == nil {
		t.Fatal("NewClientCredentials() succeeded without specifying fallback")
	}
}

// TestClientCredsInvalidHandshakeInfo verifies scenarios where the passed in
// HandshakeInfo is invalid because it does not contain the expected certificate
// providers.
func (s) TestClientCredsInvalidHandshakeInfo(t *testing.T) {
	opts := ClientOptions{FallbackCreds: makeFallbackClientCreds(t)}
	creds, err := NewClientCredentials(opts)
	if err != nil {
		t.Fatalf("NewClientCredentials(%v) failed: %v", opts, err)
	}

	pCtx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	ctx := newTestContextWithHandshakeInfo(pCtx, nil, &fakeProvider{}, "")
	if _, _, err := creds.ClientHandshake(ctx, authority, nil); err == nil {
		t.Fatal("ClientHandshake succeeded without root certificate provider in HandshakeInfo")
	}
}

// TestClientCredsProviderFailure verifies the cases where an expected
// certificate provider is missing in the HandshakeInfo value in the context.
func (s) TestClientCredsProviderFailure(t *testing.T) {
	opts := ClientOptions{FallbackCreds: makeFallbackClientCreds(t)}
	creds, err := NewClientCredentials(opts)
	if err != nil {
		t.Fatalf("NewClientCredentials(%v) failed: %v", opts, err)
	}

	tests := []struct {
		desc             string
		rootProvider     certprovider.Provider
		identityProvider certprovider.Provider
		wantErr          string
	}{
		{
			desc:         "erroring root provider",
			rootProvider: &fakeProvider{err: errors.New("root provider error")},
			wantErr:      "root provider error",
		},
		{
			desc:             "erroring identity provider",
			rootProvider:     &fakeProvider{km: &certprovider.KeyMaterial{}},
			identityProvider: &fakeProvider{err: errors.New("identity provider error")},
			wantErr:          "identity provider error",
		},
	}
	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			ctx = newTestContextWithHandshakeInfo(ctx, test.rootProvider, test.identityProvider, "")
			if _, _, err := creds.ClientHandshake(ctx, authority, nil); err == nil || !strings.Contains(err.Error(), test.wantErr) {
				t.Fatalf("ClientHandshake() returned error: %q, wantErr: %q", err, test.wantErr)
			}
		})
	}
}

// TestClientCredsSuccess verifies successful client handshake cases.
func (s) TestClientCredsSuccess(t *testing.T) {
	tests := []struct {
		desc             string
		handshakeFunc    testHandshakeFunc
		handshakeInfoCtx func(ctx context.Context) context.Context
	}{
		{
			desc:          "fallback",
			handshakeFunc: testServerTLSHandshake,
			handshakeInfoCtx: func(ctx context.Context) context.Context {
				// Since we don't add a HandshakeInfo to the context, the
				// ClientHandshake() method will delegate to the fallback.
				return ctx
			},
		},
		{
			desc:          "TLS",
			handshakeFunc: testServerTLSHandshake,
			handshakeInfoCtx: func(ctx context.Context) context.Context {
				return newTestContextWithHandshakeInfo(ctx, makeRootProvider(t, "x509/server_ca_cert.pem"), nil, defaultTestCertSAN)
			},
		},
		{
			desc:          "mTLS",
			handshakeFunc: testServerMutualTLSHandshake,
			handshakeInfoCtx: func(ctx context.Context) context.Context {
				return newTestContextWithHandshakeInfo(ctx, makeRootProvider(t, "x509/server_ca_cert.pem"), makeIdentityProvider(t, "x509/server1_cert.pem", "x509/server1_key.pem"), defaultTestCertSAN)
			},
		},
		{
			desc:          "mTLS with no acceptedSANs specified",
			handshakeFunc: testServerMutualTLSHandshake,
			handshakeInfoCtx: func(ctx context.Context) context.Context {
				return newTestContextWithHandshakeInfo(ctx, makeRootProvider(t, "x509/server_ca_cert.pem"), makeIdentityProvider(t, "x509/server1_cert.pem", "x509/server1_key.pem"), "")
			},
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			ts := newTestServerWithHandshakeFunc(test.handshakeFunc)
			defer ts.stop()

			opts := ClientOptions{FallbackCreds: makeFallbackClientCreds(t)}
			creds, err := NewClientCredentials(opts)
			if err != nil {
				t.Fatalf("NewClientCredentials(%v) failed: %v", opts, err)
			}

			conn, err := net.Dial("tcp", ts.address)
			if err != nil {
				t.Fatalf("net.Dial(%s) failed: %v", ts.address, err)
			}
			defer conn.Close()

			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			_, ai, err := creds.ClientHandshake(test.handshakeInfoCtx(ctx), authority, conn)
			if err != nil {
				t.Fatalf("ClientHandshake() returned failed: %q", err)
			}
			if err := compareAuthInfo(ctx, ts, ai); err != nil {
				t.Fatal(err)
			}
		})
	}
}

func (s) TestClientCredsHandshakeTimeout(t *testing.T) {
	clientDone := make(chan struct{})
	// A handshake function which simulates a handshake timeout from the
	// server-side by simply blocking on the client-side handshake to timeout
	// and not writing any handshake data.
	hErr := errors.New("server handshake error")
	ts := newTestServerWithHandshakeFunc(func(net.Conn) handshakeResult {
		<-clientDone
		return handshakeResult{err: hErr}
	})
	defer ts.stop()

	opts := ClientOptions{FallbackCreds: makeFallbackClientCreds(t)}
	creds, err := NewClientCredentials(opts)
	if err != nil {
		t.Fatalf("NewClientCredentials(%v) failed: %v", opts, err)
	}

	conn, err := net.Dial("tcp", ts.address)
	if err != nil {
		t.Fatalf("net.Dial(%s) failed: %v", ts.address, err)
	}
	defer conn.Close()

	sCtx, sCancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer sCancel()
	ctx := newTestContextWithHandshakeInfo(sCtx, makeRootProvider(t, "x509/server_ca_cert.pem"), nil, defaultTestCertSAN)
	if _, _, err := creds.ClientHandshake(ctx, authority, conn); err == nil {
		t.Fatal("ClientHandshake() succeeded when expected to timeout")
	}
	close(clientDone)

	// Read the handshake result from the testServer and make sure the expected
	// error is returned.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	val, err := ts.hsResult.Receive(ctx)
	if err != nil {
		t.Fatalf("testServer failed to return handshake result: %v", err)
	}
	hsr := val.(handshakeResult)
	if hsr.err != hErr {
		t.Fatalf("testServer handshake returned error: %v, want: %v", hsr.err, hErr)
	}
}

// TestClientCredsHandshakeFailure verifies different handshake failure cases.
func (s) TestClientCredsHandshakeFailure(t *testing.T) {
	tests := []struct {
		desc          string
		handshakeFunc testHandshakeFunc
		rootProvider  certprovider.Provider
		san           string
		wantErr       string
	}{
		{
			desc:          "cert validation failure",
			handshakeFunc: testServerTLSHandshake,
			rootProvider:  makeRootProvider(t, "x509/client_ca_cert.pem"),
			san:           defaultTestCertSAN,
			wantErr:       "x509: certificate signed by unknown authority",
		},
		{
			desc:          "SAN mismatch",
			handshakeFunc: testServerTLSHandshake,
			rootProvider:  makeRootProvider(t, "x509/server_ca_cert.pem"),
			san:           "bad-san",
			wantErr:       "do not match any of the accepted SANs",
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			ts := newTestServerWithHandshakeFunc(test.handshakeFunc)
			defer ts.stop()

			opts := ClientOptions{FallbackCreds: makeFallbackClientCreds(t)}
			creds, err := NewClientCredentials(opts)
			if err != nil {
				t.Fatalf("NewClientCredentials(%v) failed: %v", opts, err)
			}

			conn, err := net.Dial("tcp", ts.address)
			if err != nil {
				t.Fatalf("net.Dial(%s) failed: %v", ts.address, err)
			}
			defer conn.Close()

			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			ctx = newTestContextWithHandshakeInfo(ctx, test.rootProvider, nil, test.san)
			if _, _, err := creds.ClientHandshake(ctx, authority, conn); err == nil || !strings.Contains(err.Error(), test.wantErr) {
				t.Fatalf("ClientHandshake() returned %q, wantErr %q", err, test.wantErr)
			}
		})
	}
}

// TestClientCredsProviderSwitch verifies the case where the first attempt of
// ClientHandshake fails because of a handshake failure. Then we update the
// certificate provider and the second attempt succeeds. This is an
// approximation of the flow of events when the control plane specifies new
// security config which results in new certificate providers being used.
func (s) TestClientCredsProviderSwitch(t *testing.T) {
	ts := newTestServerWithHandshakeFunc(testServerTLSHandshake)
	defer ts.stop()

	opts := ClientOptions{FallbackCreds: makeFallbackClientCreds(t)}
	creds, err := NewClientCredentials(opts)
	if err != nil {
		t.Fatalf("NewClientCredentials(%v) failed: %v", opts, err)
	}

	conn, err := net.Dial("tcp", ts.address)
	if err != nil {
		t.Fatalf("net.Dial(%s) failed: %v", ts.address, err)
	}
	defer conn.Close()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	// Create a root provider which will fail the handshake because it does not
	// use the correct trust roots.
	root1 := makeRootProvider(t, "x509/client_ca_cert.pem")
	handshakeInfo := xdsinternal.NewHandshakeInfo(root1, nil, []matcher.StringMatcher{matcher.StringMatcherForTesting(newStringP(defaultTestCertSAN), nil, nil, nil, nil, false)}, false)
	// We need to repeat most of what newTestContextWithHandshakeInfo() does
	// here because we need access to the underlying HandshakeInfo so that we
	// can update it before the next call to ClientHandshake().
	uPtr := unsafe.Pointer(handshakeInfo)
	addr := xdsinternal.SetHandshakeInfo(resolver.Address{}, &uPtr)
	ctx = icredentials.NewClientHandshakeInfoContext(ctx, credentials.ClientHandshakeInfo{Attributes: addr.Attributes})
	if _, _, err := creds.ClientHandshake(ctx, authority, conn); err == nil {
		t.Fatal("ClientHandshake() succeeded when expected to fail")
	}
	// Drain the result channel on the test server so that we can inspect the
	// result for the next handshake.
	_, err = ts.hsResult.Receive(ctx)
	if err != nil {
		t.Errorf("testServer failed to return handshake result: %v", err)
	}

	conn, err = net.Dial("tcp", ts.address)
	if err != nil {
		t.Fatalf("net.Dial(%s) failed: %v", ts.address, err)
	}
	defer conn.Close()

	// Create a new root provider which uses the correct trust roots. And update
	// the HandshakeInfo with the new provider.
	root2 := makeRootProvider(t, "x509/server_ca_cert.pem")
	handshakeInfo = xdsinternal.NewHandshakeInfo(root2, nil, []matcher.StringMatcher{matcher.StringMatcherForTesting(newStringP(defaultTestCertSAN), nil, nil, nil, nil, false)}, false)
	// Update the existing pointer, which address attribute will continue to
	// point to.
	atomic.StorePointer(&uPtr, unsafe.Pointer(handshakeInfo))
	_, ai, err := creds.ClientHandshake(ctx, authority, conn)
	if err != nil {
		t.Fatalf("ClientHandshake() returned failed: %q", err)
	}
	if err := compareAuthInfo(ctx, ts, ai); err != nil {
		t.Fatal(err)
	}
}

// TestClientClone verifies the Clone() method on client credentials.
func (s) TestClientClone(t *testing.T) {
	opts := ClientOptions{FallbackCreds: makeFallbackClientCreds(t)}
	orig, err := NewClientCredentials(opts)
	if err != nil {
		t.Fatalf("NewClientCredentials(%v) failed: %v", opts, err)
	}

	// The credsImpl does not have any exported fields, and it does not make
	// sense to use any cmp options to look deep into. So, all we make sure here
	// is that the cloned object points to a different location in memory.
	if clone := orig.Clone(); clone == orig {
		t.Fatal("return value from Clone() doesn't point to new credentials instance")
	}
}

func newStringP(s string) *string {
	return &s
}

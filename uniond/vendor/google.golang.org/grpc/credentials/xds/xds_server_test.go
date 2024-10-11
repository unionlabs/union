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
	"testing"
	"time"

	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/tls/certprovider"
	xdsinternal "google.golang.org/grpc/internal/credentials/xds"
	"google.golang.org/grpc/testdata"
)

func makeClientTLSConfig(t *testing.T, mTLS bool) *tls.Config {
	t.Helper()

	pemData, err := os.ReadFile(testdata.Path("x509/server_ca_cert.pem"))
	if err != nil {
		t.Fatal(err)
	}
	roots := x509.NewCertPool()
	roots.AppendCertsFromPEM(pemData)

	var certs []tls.Certificate
	if mTLS {
		cert, err := tls.LoadX509KeyPair(testdata.Path("x509/client1_cert.pem"), testdata.Path("x509/client1_key.pem"))
		if err != nil {
			t.Fatal(err)
		}
		certs = append(certs, cert)
	}

	return &tls.Config{
		Certificates: certs,
		RootCAs:      roots,
		ServerName:   "*.test.example.com",
		// Setting this to true completely turns off the certificate validation
		// on the client side. So, the client side handshake always seems to
		// succeed. But if we want to turn this ON, we will need to generate
		// certificates which work with localhost, or supply a custom
		// verification function. So, the server credentials tests will rely
		// solely on the success/failure of the server-side handshake.
		InsecureSkipVerify: true,
		NextProtos:         []string{"h2"},
	}
}

// Helper function to create a real TLS server credentials which is used as
// fallback credentials from multiple tests.
func makeFallbackServerCreds(t *testing.T) credentials.TransportCredentials {
	t.Helper()

	creds, err := credentials.NewServerTLSFromFile(testdata.Path("x509/server1_cert.pem"), testdata.Path("x509/server1_key.pem"))
	if err != nil {
		t.Fatal(err)
	}
	return creds
}

type errorCreds struct {
	credentials.TransportCredentials
}

// TestServerCredsWithoutFallback verifies that the call to
// NewServerCredentials() fails when no fallback is specified.
func (s) TestServerCredsWithoutFallback(t *testing.T) {
	if _, err := NewServerCredentials(ServerOptions{}); err == nil {
		t.Fatal("NewServerCredentials() succeeded without specifying fallback")
	}
}

type wrapperConn struct {
	net.Conn
	xdsHI            *xdsinternal.HandshakeInfo
	deadline         time.Time
	handshakeInfoErr error
}

func (wc *wrapperConn) XDSHandshakeInfo() (*xdsinternal.HandshakeInfo, error) {
	return wc.xdsHI, wc.handshakeInfoErr
}

func (wc *wrapperConn) GetDeadline() time.Time {
	return wc.deadline
}

func newWrappedConn(conn net.Conn, xdsHI *xdsinternal.HandshakeInfo, deadline time.Time) *wrapperConn {
	return &wrapperConn{Conn: conn, xdsHI: xdsHI, deadline: deadline}
}

// TestServerCredsInvalidHandshakeInfo verifies scenarios where the passed in
// HandshakeInfo is invalid because it does not contain the expected certificate
// providers.
func (s) TestServerCredsInvalidHandshakeInfo(t *testing.T) {
	opts := ServerOptions{FallbackCreds: &errorCreds{}}
	creds, err := NewServerCredentials(opts)
	if err != nil {
		t.Fatalf("NewServerCredentials(%v) failed: %v", opts, err)
	}

	info := xdsinternal.NewHandshakeInfo(&fakeProvider{}, nil, nil, false)
	conn := newWrappedConn(nil, info, time.Time{})
	if _, _, err := creds.ServerHandshake(conn); err == nil {
		t.Fatal("ServerHandshake succeeded without identity certificate provider in HandshakeInfo")
	}
}

// TestServerCredsProviderFailure verifies the cases where an expected
// certificate provider is missing in the HandshakeInfo value in the context.
func (s) TestServerCredsProviderFailure(t *testing.T) {
	opts := ServerOptions{FallbackCreds: &errorCreds{}}
	creds, err := NewServerCredentials(opts)
	if err != nil {
		t.Fatalf("NewServerCredentials(%v) failed: %v", opts, err)
	}

	tests := []struct {
		desc             string
		rootProvider     certprovider.Provider
		identityProvider certprovider.Provider
		wantErr          string
	}{
		{
			desc:             "erroring identity provider",
			identityProvider: &fakeProvider{err: errors.New("identity provider error")},
			wantErr:          "identity provider error",
		},
		{
			desc:             "erroring root provider",
			identityProvider: &fakeProvider{km: &certprovider.KeyMaterial{}},
			rootProvider:     &fakeProvider{err: errors.New("root provider error")},
			wantErr:          "root provider error",
		},
	}
	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			info := xdsinternal.NewHandshakeInfo(test.rootProvider, test.identityProvider, nil, false)
			conn := newWrappedConn(nil, info, time.Time{})
			if _, _, err := creds.ServerHandshake(conn); err == nil || !strings.Contains(err.Error(), test.wantErr) {
				t.Fatalf("ServerHandshake() returned error: %q, wantErr: %q", err, test.wantErr)
			}
		})
	}
}

// TestServerCredsHandshake_XDSHandshakeInfoError verifies the case where the
// call to XDSHandshakeInfo() from the ServerHandshake() method returns an
// error, and the test verifies that the ServerHandshake() fails with the
// expected error.
func (s) TestServerCredsHandshake_XDSHandshakeInfoError(t *testing.T) {
	opts := ServerOptions{FallbackCreds: &errorCreds{}}
	creds, err := NewServerCredentials(opts)
	if err != nil {
		t.Fatalf("NewServerCredentials(%v) failed: %v", opts, err)
	}

	// Create a test server which uses the xDS server credentials created above
	// to perform TLS handshake on incoming connections.
	ts := newTestServerWithHandshakeFunc(func(rawConn net.Conn) handshakeResult {
		// Create a wrapped conn which returns a nil HandshakeInfo and a non-nil error.
		conn := newWrappedConn(rawConn, nil, time.Now().Add(defaultTestTimeout))
		hiErr := errors.New("xdsHandshakeInfo error")
		conn.handshakeInfoErr = hiErr

		// Invoke the ServerHandshake() method on the xDS credentials and verify
		// that the error returned by the XDSHandshakeInfo() method on the
		// wrapped conn is returned here.
		_, _, err := creds.ServerHandshake(conn)
		if !errors.Is(err, hiErr) {
			return handshakeResult{err: fmt.Errorf("ServerHandshake() returned err: %v, wantErr: %v", err, hiErr)}
		}
		return handshakeResult{}
	})
	defer ts.stop()

	// Dial the test server, but don't trigger the TLS handshake. This will
	// cause ServerHandshake() to fail.
	rawConn, err := net.Dial("tcp", ts.address)
	if err != nil {
		t.Fatalf("net.Dial(%s) failed: %v", ts.address, err)
	}
	defer rawConn.Close()

	// Read handshake result from the testServer which will return an error if
	// the handshake succeeded.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	val, err := ts.hsResult.Receive(ctx)
	if err != nil {
		t.Fatalf("testServer failed to return handshake result: %v", err)
	}
	hsr := val.(handshakeResult)
	if hsr.err != nil {
		t.Fatalf("testServer handshake failure: %v", hsr.err)
	}
}

// TestServerCredsHandshakeTimeout verifies the case where the client does not
// send required handshake data before the deadline set on the net.Conn passed
// to ServerHandshake().
func (s) TestServerCredsHandshakeTimeout(t *testing.T) {
	opts := ServerOptions{FallbackCreds: &errorCreds{}}
	creds, err := NewServerCredentials(opts)
	if err != nil {
		t.Fatalf("NewServerCredentials(%v) failed: %v", opts, err)
	}

	// Create a test server which uses the xDS server credentials created above
	// to perform TLS handshake on incoming connections.
	ts := newTestServerWithHandshakeFunc(func(rawConn net.Conn) handshakeResult {
		hi := xdsinternal.NewHandshakeInfo(makeRootProvider(t, "x509/client_ca_cert.pem"), makeIdentityProvider(t, "x509/server2_cert.pem", "x509/server2_key.pem"), nil, true)

		// Create a wrapped conn which can return the HandshakeInfo created
		// above with a very small deadline.
		d := time.Now().Add(defaultTestShortTimeout)
		rawConn.SetDeadline(d)
		conn := newWrappedConn(rawConn, hi, d)

		// ServerHandshake() on the xDS credentials is expected to fail.
		if _, _, err := creds.ServerHandshake(conn); err == nil {
			return handshakeResult{err: errors.New("ServerHandshake() succeeded when expected to timeout")}
		}
		return handshakeResult{}
	})
	defer ts.stop()

	// Dial the test server, but don't trigger the TLS handshake. This will
	// cause ServerHandshake() to fail.
	rawConn, err := net.Dial("tcp", ts.address)
	if err != nil {
		t.Fatalf("net.Dial(%s) failed: %v", ts.address, err)
	}
	defer rawConn.Close()

	// Read handshake result from the testServer and expect a failure result.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	val, err := ts.hsResult.Receive(ctx)
	if err != nil {
		t.Fatalf("testServer failed to return handshake result: %v", err)
	}
	hsr := val.(handshakeResult)
	if hsr.err != nil {
		t.Fatalf("testServer handshake failure: %v", hsr.err)
	}
}

// TestServerCredsHandshakeFailure verifies the case where the server-side
// credentials uses a root certificate which does not match the certificate
// presented by the client, and hence the handshake must fail.
func (s) TestServerCredsHandshakeFailure(t *testing.T) {
	opts := ServerOptions{FallbackCreds: &errorCreds{}}
	creds, err := NewServerCredentials(opts)
	if err != nil {
		t.Fatalf("NewServerCredentials(%v) failed: %v", opts, err)
	}

	// Create a test server which uses the xDS server credentials created above
	// to perform TLS handshake on incoming connections.
	ts := newTestServerWithHandshakeFunc(func(rawConn net.Conn) handshakeResult {
		// Create a HandshakeInfo which has a root provider which does not match
		// the certificate sent by the client.
		hi := xdsinternal.NewHandshakeInfo(makeRootProvider(t, "x509/server_ca_cert.pem"), makeIdentityProvider(t, "x509/client2_cert.pem", "x509/client2_key.pem"), nil, true)

		// Create a wrapped conn which can return the HandshakeInfo and
		// configured deadline to the xDS credentials' ServerHandshake()
		// method.
		conn := newWrappedConn(rawConn, hi, time.Now().Add(defaultTestTimeout))

		// ServerHandshake() on the xDS credentials is expected to fail.
		if _, _, err := creds.ServerHandshake(conn); err == nil {
			return handshakeResult{err: errors.New("ServerHandshake() succeeded when expected to fail")}
		}
		return handshakeResult{}
	})
	defer ts.stop()

	// Dial the test server, and trigger the TLS handshake.
	rawConn, err := net.Dial("tcp", ts.address)
	if err != nil {
		t.Fatalf("net.Dial(%s) failed: %v", ts.address, err)
	}
	defer rawConn.Close()
	tlsConn := tls.Client(rawConn, makeClientTLSConfig(t, true))
	tlsConn.SetDeadline(time.Now().Add(defaultTestTimeout))
	if err := tlsConn.Handshake(); err != nil {
		t.Fatal(err)
	}

	// Read handshake result from the testServer which will return an error if
	// the handshake succeeded.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	val, err := ts.hsResult.Receive(ctx)
	if err != nil {
		t.Fatalf("testServer failed to return handshake result: %v", err)
	}
	hsr := val.(handshakeResult)
	if hsr.err != nil {
		t.Fatalf("testServer handshake failure: %v", hsr.err)
	}
}

// TestServerCredsHandshakeSuccess verifies success handshake cases.
func (s) TestServerCredsHandshakeSuccess(t *testing.T) {
	tests := []struct {
		desc              string
		fallbackCreds     credentials.TransportCredentials
		rootProvider      certprovider.Provider
		identityProvider  certprovider.Provider
		requireClientCert bool
	}{
		{
			desc:          "fallback",
			fallbackCreds: makeFallbackServerCreds(t),
		},
		{
			desc:             "TLS",
			fallbackCreds:    &errorCreds{},
			identityProvider: makeIdentityProvider(t, "x509/server2_cert.pem", "x509/server2_key.pem"),
		},
		{
			desc:              "mTLS",
			fallbackCreds:     &errorCreds{},
			identityProvider:  makeIdentityProvider(t, "x509/server2_cert.pem", "x509/server2_key.pem"),
			rootProvider:      makeRootProvider(t, "x509/client_ca_cert.pem"),
			requireClientCert: true,
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			// Create an xDS server credentials.
			opts := ServerOptions{FallbackCreds: test.fallbackCreds}
			creds, err := NewServerCredentials(opts)
			if err != nil {
				t.Fatalf("NewServerCredentials(%v) failed: %v", opts, err)
			}

			// Create a test server which uses the xDS server credentials
			// created above to perform TLS handshake on incoming connections.
			ts := newTestServerWithHandshakeFunc(func(rawConn net.Conn) handshakeResult {
				// Create a HandshakeInfo with information from the test table.
				hi := xdsinternal.NewHandshakeInfo(test.rootProvider, test.identityProvider, nil, test.requireClientCert)

				// Create a wrapped conn which can return the HandshakeInfo and
				// configured deadline to the xDS credentials' ServerHandshake()
				// method.
				conn := newWrappedConn(rawConn, hi, time.Now().Add(defaultTestTimeout))

				// Invoke the ServerHandshake() method on the xDS credentials
				// and make some sanity checks before pushing the result for
				// inspection by the main test body.
				_, ai, err := creds.ServerHandshake(conn)
				if err != nil {
					return handshakeResult{err: fmt.Errorf("ServerHandshake() failed: %v", err)}
				}
				if ai.AuthType() != "tls" {
					return handshakeResult{err: fmt.Errorf("ServerHandshake returned authType %q, want %q", ai.AuthType(), "tls")}
				}
				info, ok := ai.(credentials.TLSInfo)
				if !ok {
					return handshakeResult{err: fmt.Errorf("ServerHandshake returned authInfo of type %T, want %T", ai, credentials.TLSInfo{})}
				}
				return handshakeResult{connState: info.State}
			})
			defer ts.stop()

			// Dial the test server, and trigger the TLS handshake.
			rawConn, err := net.Dial("tcp", ts.address)
			if err != nil {
				t.Fatalf("net.Dial(%s) failed: %v", ts.address, err)
			}
			defer rawConn.Close()
			tlsConn := tls.Client(rawConn, makeClientTLSConfig(t, test.requireClientCert))
			tlsConn.SetDeadline(time.Now().Add(defaultTestTimeout))
			if err := tlsConn.Handshake(); err != nil {
				t.Fatal(err)
			}

			// Read the handshake result from the testServer which contains the
			// TLS connection state on the server-side and compare it with the
			// one received on the client-side.
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			val, err := ts.hsResult.Receive(ctx)
			if err != nil {
				t.Fatalf("testServer failed to return handshake result: %v", err)
			}
			hsr := val.(handshakeResult)
			if hsr.err != nil {
				t.Fatalf("testServer handshake failure: %v", hsr.err)
			}

			// AuthInfo contains a variety of information. We only verify a
			// subset here. This is the same subset which is verified in TLS
			// credentials tests.
			if err := compareConnState(tlsConn.ConnectionState(), hsr.connState); err != nil {
				t.Fatal(err)
			}
		})
	}
}

func (s) TestServerCredsProviderSwitch(t *testing.T) {
	opts := ServerOptions{FallbackCreds: &errorCreds{}}
	creds, err := NewServerCredentials(opts)
	if err != nil {
		t.Fatalf("NewServerCredentials(%v) failed: %v", opts, err)
	}

	// The first time the handshake function is invoked, it returns a
	// HandshakeInfo which is expected to fail. Further invocations return a
	// HandshakeInfo which is expected to succeed.
	cnt := 0
	// Create a test server which uses the xDS server credentials created above
	// to perform TLS handshake on incoming connections.
	ts := newTestServerWithHandshakeFunc(func(rawConn net.Conn) handshakeResult {
		cnt++
		var hi *xdsinternal.HandshakeInfo
		if cnt == 1 {
			// Create a HandshakeInfo which has a root provider which does not match
			// the certificate sent by the client.
			hi = xdsinternal.NewHandshakeInfo(makeRootProvider(t, "x509/server_ca_cert.pem"), makeIdentityProvider(t, "x509/client2_cert.pem", "x509/client2_key.pem"), nil, true)

			// Create a wrapped conn which can return the HandshakeInfo and
			// configured deadline to the xDS credentials' ServerHandshake()
			// method.
			conn := newWrappedConn(rawConn, hi, time.Now().Add(defaultTestTimeout))

			// ServerHandshake() on the xDS credentials is expected to fail.
			if _, _, err := creds.ServerHandshake(conn); err == nil {
				return handshakeResult{err: errors.New("ServerHandshake() succeeded when expected to fail")}
			}
			return handshakeResult{}
		}

		hi = xdsinternal.NewHandshakeInfo(makeRootProvider(t, "x509/client_ca_cert.pem"), makeIdentityProvider(t, "x509/server1_cert.pem", "x509/server1_key.pem"), nil, true)

		// Create a wrapped conn which can return the HandshakeInfo and
		// configured deadline to the xDS credentials' ServerHandshake()
		// method.
		conn := newWrappedConn(rawConn, hi, time.Now().Add(defaultTestTimeout))

		// Invoke the ServerHandshake() method on the xDS credentials
		// and make some sanity checks before pushing the result for
		// inspection by the main test body.
		_, ai, err := creds.ServerHandshake(conn)
		if err != nil {
			return handshakeResult{err: fmt.Errorf("ServerHandshake() failed: %v", err)}
		}
		if ai.AuthType() != "tls" {
			return handshakeResult{err: fmt.Errorf("ServerHandshake returned authType %q, want %q", ai.AuthType(), "tls")}
		}
		info, ok := ai.(credentials.TLSInfo)
		if !ok {
			return handshakeResult{err: fmt.Errorf("ServerHandshake returned authInfo of type %T, want %T", ai, credentials.TLSInfo{})}
		}
		return handshakeResult{connState: info.State}
	})
	defer ts.stop()

	for i := 0; i < 5; i++ {
		// Dial the test server, and trigger the TLS handshake.
		rawConn, err := net.Dial("tcp", ts.address)
		if err != nil {
			t.Fatalf("net.Dial(%s) failed: %v", ts.address, err)
		}
		defer rawConn.Close()
		tlsConn := tls.Client(rawConn, makeClientTLSConfig(t, true))
		tlsConn.SetDeadline(time.Now().Add(defaultTestTimeout))
		if err := tlsConn.Handshake(); err != nil {
			t.Fatal(err)
		}

		// Read the handshake result from the testServer which contains the
		// TLS connection state on the server-side and compare it with the
		// one received on the client-side.
		ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
		defer cancel()
		val, err := ts.hsResult.Receive(ctx)
		if err != nil {
			t.Fatalf("testServer failed to return handshake result: %v", err)
		}
		hsr := val.(handshakeResult)
		if hsr.err != nil {
			t.Fatalf("testServer handshake failure: %v", hsr.err)
		}
		if i == 0 {
			// We expect the first handshake to fail. So, we skip checks which
			// compare connection state.
			continue
		}
		// AuthInfo contains a variety of information. We only verify a
		// subset here. This is the same subset which is verified in TLS
		// credentials tests.
		if err := compareConnState(tlsConn.ConnectionState(), hsr.connState); err != nil {
			t.Fatal(err)
		}
	}
}

// TestServerClone verifies the Clone() method on client credentials.
func (s) TestServerClone(t *testing.T) {
	opts := ServerOptions{FallbackCreds: makeFallbackServerCreds(t)}
	orig, err := NewServerCredentials(opts)
	if err != nil {
		t.Fatalf("NewServerCredentials(%v) failed: %v", opts, err)
	}

	// The credsImpl does not have any exported fields, and it does not make
	// sense to use any cmp options to look deep into. So, all we make sure here
	// is that the cloned object points to a different location in memory.
	if clone := orig.Clone(); clone == orig {
		t.Fatal("return value from Clone() doesn't point to new credentials instance")
	}
}

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

// Package xds provides a transport credentials implementation where the
// security configuration is pushed by a management server using xDS APIs.
package xds

import (
	"context"
	"crypto/tls"
	"crypto/x509"
	"errors"
	"fmt"
	"net"
	"sync/atomic"
	"time"

	"google.golang.org/grpc/credentials"
	credinternal "google.golang.org/grpc/internal/credentials"
	xdsinternal "google.golang.org/grpc/internal/credentials/xds"
)

// ClientOptions contains parameters to configure a new client-side xDS
// credentials implementation.
type ClientOptions struct {
	// FallbackCreds specifies the fallback credentials to be used when either
	// the `xds` scheme is not used in the user's dial target or when the
	// management server does not return any security configuration. Attempts to
	// create client credentials without fallback credentials will fail.
	FallbackCreds credentials.TransportCredentials
}

// NewClientCredentials returns a new client-side transport credentials
// implementation which uses xDS APIs to fetch its security configuration.
func NewClientCredentials(opts ClientOptions) (credentials.TransportCredentials, error) {
	if opts.FallbackCreds == nil {
		return nil, errors.New("missing fallback credentials")
	}
	return &credsImpl{
		isClient: true,
		fallback: opts.FallbackCreds,
	}, nil
}

// ServerOptions contains parameters to configure a new server-side xDS
// credentials implementation.
type ServerOptions struct {
	// FallbackCreds specifies the fallback credentials to be used when the
	// management server does not return any security configuration. Attempts to
	// create server credentials without fallback credentials will fail.
	FallbackCreds credentials.TransportCredentials
}

// NewServerCredentials returns a new server-side transport credentials
// implementation which uses xDS APIs to fetch its security configuration.
func NewServerCredentials(opts ServerOptions) (credentials.TransportCredentials, error) {
	if opts.FallbackCreds == nil {
		return nil, errors.New("missing fallback credentials")
	}
	return &credsImpl{
		isClient: false,
		fallback: opts.FallbackCreds,
	}, nil
}

// credsImpl is an implementation of the credentials.TransportCredentials
// interface which uses xDS APIs to fetch its security configuration.
type credsImpl struct {
	isClient bool
	fallback credentials.TransportCredentials
}

// ClientHandshake performs the TLS handshake on the client-side.
//
// It looks for the presence of a HandshakeInfo value in the passed in context
// (added using a call to NewContextWithHandshakeInfo()), and retrieves identity
// and root certificates from there. It also retrieves a list of acceptable SANs
// and uses a custom verification function to validate the certificate presented
// by the peer. It uses fallback credentials if no HandshakeInfo is present in
// the passed in context.
func (c *credsImpl) ClientHandshake(ctx context.Context, authority string, rawConn net.Conn) (net.Conn, credentials.AuthInfo, error) {
	if !c.isClient {
		return nil, nil, errors.New("ClientHandshake() is not supported for server credentials")
	}

	// The CDS balancer constructs a new HandshakeInfo using a call to
	// NewHandshakeInfo(), and then adds it to the attributes field of the
	// resolver.Address when handling calls to NewSubConn(). The transport layer
	// takes care of shipping these attributes in the context to this handshake
	// function. We first read the credentials.ClientHandshakeInfo type from the
	// context, which contains the attributes added by the CDS balancer. We then
	// read the HandshakeInfo from the attributes to get to the actual data that
	// we need here for the handshake.
	chi := credentials.ClientHandshakeInfoFromContext(ctx)
	// If there are no attributes in the received context or the attributes does
	// not contain a HandshakeInfo, it could either mean that the user did not
	// specify an `xds` scheme in their dial target or that the xDS server did
	// not provide any security configuration. In both of these cases, we use
	// the fallback credentials specified by the user.
	if chi.Attributes == nil {
		return c.fallback.ClientHandshake(ctx, authority, rawConn)
	}

	uPtr := xdsinternal.GetHandshakeInfo(chi.Attributes)
	hi := (*xdsinternal.HandshakeInfo)(atomic.LoadPointer(uPtr))
	if hi.UseFallbackCreds() {
		return c.fallback.ClientHandshake(ctx, authority, rawConn)
	}

	// We build the tls.Config with the following values
	// 1. Root certificate as returned by the root provider.
	// 2. Identity certificate as returned by the identity provider. This may be
	//    empty on the client side, if the client is not doing mTLS.
	// 3. InsecureSkipVerify to true. Certificates used in Mesh environments
	//    usually contains the identity of the workload presenting the
	//    certificate as a SAN (instead of a hostname in the CommonName field).
	//    This means that normal certificate verification as done by the
	//    standard library will fail.
	// 4. Key usage to match whether client/server usage.
	// 5. A `VerifyPeerCertificate` function which performs normal peer
	// 	  cert verification using configured roots, and the custom SAN checks.
	cfg, err := hi.ClientSideTLSConfig(ctx)
	if err != nil {
		return nil, nil, err
	}
	cfg.VerifyPeerCertificate = func(rawCerts [][]byte, _ [][]*x509.Certificate) error {
		// Parse all raw certificates presented by the peer.
		var certs []*x509.Certificate
		for _, rc := range rawCerts {
			cert, err := x509.ParseCertificate(rc)
			if err != nil {
				return err
			}
			certs = append(certs, cert)
		}

		// Build the intermediates list and verify that the leaf certificate
		// is signed by one of the root certificates.
		intermediates := x509.NewCertPool()
		for _, cert := range certs[1:] {
			intermediates.AddCert(cert)
		}
		opts := x509.VerifyOptions{
			Roots:         cfg.RootCAs,
			Intermediates: intermediates,
			KeyUsages:     []x509.ExtKeyUsage{x509.ExtKeyUsageServerAuth},
		}
		if _, err := certs[0].Verify(opts); err != nil {
			return err
		}
		// The SANs sent by the MeshCA are encoded as SPIFFE IDs. We need to
		// only look at the SANs on the leaf cert.
		if cert := certs[0]; !hi.MatchingSANExists(cert) {
			// TODO: Print the complete certificate once the x509 package
			// supports a String() method on the Certificate type.
			return fmt.Errorf("xds: received SANs {DNSNames: %v, EmailAddresses: %v, IPAddresses: %v, URIs: %v} do not match any of the accepted SANs", cert.DNSNames, cert.EmailAddresses, cert.IPAddresses, cert.URIs)
		}
		return nil
	}

	// Perform the TLS handshake with the tls.Config that we have. We run the
	// actual Handshake() function in a goroutine because we need to respect the
	// deadline specified on the passed in context, and we need a way to cancel
	// the handshake if the context is cancelled.
	conn := tls.Client(rawConn, cfg)
	errCh := make(chan error, 1)
	go func() {
		errCh <- conn.Handshake()
		close(errCh)
	}()
	select {
	case err := <-errCh:
		if err != nil {
			conn.Close()
			return nil, nil, err
		}
	case <-ctx.Done():
		conn.Close()
		return nil, nil, ctx.Err()
	}
	info := credentials.TLSInfo{
		State: conn.ConnectionState(),
		CommonAuthInfo: credentials.CommonAuthInfo{
			SecurityLevel: credentials.PrivacyAndIntegrity,
		},
		SPIFFEID: credinternal.SPIFFEIDFromState(conn.ConnectionState()),
	}
	return credinternal.WrapSyscallConn(rawConn, conn), info, nil
}

// ServerHandshake performs the TLS handshake on the server-side.
func (c *credsImpl) ServerHandshake(rawConn net.Conn) (net.Conn, credentials.AuthInfo, error) {
	if c.isClient {
		return nil, nil, errors.New("ServerHandshake is not supported for client credentials")
	}

	// An xds-enabled gRPC server wraps the underlying raw net.Conn in a type
	// that provides a way to retrieve `HandshakeInfo`, which contains the
	// certificate providers to be used during the handshake. If the net.Conn
	// passed to this function does not implement this interface, or if the
	// `HandshakeInfo` does not contain the information we are looking for, we
	// delegate the handshake to the fallback credentials.
	hiConn, ok := rawConn.(interface {
		XDSHandshakeInfo() (*xdsinternal.HandshakeInfo, error)
	})
	if !ok {
		return c.fallback.ServerHandshake(rawConn)
	}
	hi, err := hiConn.XDSHandshakeInfo()
	if err != nil {
		return nil, nil, err
	}
	if hi.UseFallbackCreds() {
		return c.fallback.ServerHandshake(rawConn)
	}

	// An xds-enabled gRPC server is expected to wrap the underlying raw
	// net.Conn in a type which provides a way to retrieve the deadline set on
	// it. If we cannot retrieve the deadline here, we fail (by setting deadline
	// to time.Now()), instead of using a default deadline and possibly taking
	// longer to eventually fail.
	deadline := time.Now()
	if dConn, ok := rawConn.(interface{ GetDeadline() time.Time }); ok {
		deadline = dConn.GetDeadline()
	}
	ctx, cancel := context.WithDeadline(context.Background(), deadline)
	defer cancel()
	cfg, err := hi.ServerSideTLSConfig(ctx)
	if err != nil {
		return nil, nil, err
	}

	conn := tls.Server(rawConn, cfg)
	if err := conn.Handshake(); err != nil {
		conn.Close()
		return nil, nil, err
	}
	info := credentials.TLSInfo{
		State: conn.ConnectionState(),
		CommonAuthInfo: credentials.CommonAuthInfo{
			SecurityLevel: credentials.PrivacyAndIntegrity,
		},
	}
	info.SPIFFEID = credinternal.SPIFFEIDFromState(conn.ConnectionState())
	return credinternal.WrapSyscallConn(rawConn, conn), info, nil
}

// Info provides the ProtocolInfo of this TransportCredentials.
func (c *credsImpl) Info() credentials.ProtocolInfo {
	return credentials.ProtocolInfo{SecurityProtocol: "tls"}
}

// Clone makes a copy of this TransportCredentials.
func (c *credsImpl) Clone() credentials.TransportCredentials {
	clone := *c
	return &clone
}

func (c *credsImpl) OverrideServerName(_ string) error {
	return errors.New("serverName for peer validation must be configured as a list of acceptable SANs")
}

// UsesXDS returns true if c uses xDS to fetch security configuration
// used at handshake time, and false otherwise.
func (c *credsImpl) UsesXDS() bool {
	return true
}

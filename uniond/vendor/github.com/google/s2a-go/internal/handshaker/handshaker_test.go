/*
 *
 * Copyright 2021 Google LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
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
	"io"
	"net"
	"strings"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	commonpb "github.com/google/s2a-go/internal/proto/common_go_proto"
	s2apb "github.com/google/s2a-go/internal/proto/s2a_go_proto"
	"github.com/google/s2a-go/internal/tokenmanager"
	"golang.org/x/sync/errgroup"
	grpc "google.golang.org/grpc"
	"google.golang.org/protobuf/testing/protocmp"
)

var (
	testAccessToken = "test_access_token"

	// testHSAddr is the handshaker service address used for testing
	testHSAddr = "handshaker_address"

	// testHostname is the hostname of the server used for testing.
	testHostname = "localhost"

	// testClientHandshakerOptions are the client-side handshaker options used for
	// testing.
	testClientHandshakerOptions = &ClientHandshakerOptions{
		MinTLSVersion: commonpb.TLSVersion_TLS1_2,
		MaxTLSVersion: commonpb.TLSVersion_TLS1_3,
		TLSCiphersuites: []commonpb.Ciphersuite{
			commonpb.Ciphersuite_AES_128_GCM_SHA256,
			commonpb.Ciphersuite_AES_256_GCM_SHA384,
			commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
		},
		TargetIdentities: []*commonpb.Identity{
			{
				IdentityOneof: &commonpb.Identity_SpiffeId{
					SpiffeId: "target_spiffe_id",
				},
			},
			{
				IdentityOneof: &commonpb.Identity_Hostname{
					Hostname: "target_hostname",
				},
			},
		},
		LocalIdentity: &commonpb.Identity{
			IdentityOneof: &commonpb.Identity_SpiffeId{
				SpiffeId: "client_local_spiffe_id",
			},
		},
		TargetName: testHostname + ":1234",
	}

	// testClientStart is the ClientSessionStartReq message that the S2A expects
	// to receive first from the test client.
	testClientStart = &s2apb.ClientSessionStartReq{
		ApplicationProtocols: []string{"grpc"},
		MinTlsVersion:        commonpb.TLSVersion_TLS1_2,
		MaxTlsVersion:        commonpb.TLSVersion_TLS1_3,
		TlsCiphersuites: []commonpb.Ciphersuite{
			commonpb.Ciphersuite_AES_128_GCM_SHA256,
			commonpb.Ciphersuite_AES_256_GCM_SHA384,
			commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
		},
		TargetIdentities: []*commonpb.Identity{
			{
				IdentityOneof: &commonpb.Identity_SpiffeId{
					SpiffeId: "target_spiffe_id",
				},
			},
			{
				IdentityOneof: &commonpb.Identity_Hostname{
					Hostname: "target_hostname",
				},
			},
		},
		LocalIdentity: &commonpb.Identity{
			IdentityOneof: &commonpb.Identity_SpiffeId{
				SpiffeId: "client_local_spiffe_id",
			},
		},
		TargetName: testHostname,
	}

	// testClientNext is the SessionNextReq message that the S2A expects
	// to receive second from the test client.
	testClientNext = &s2apb.SessionNextReq{
		InBytes: []byte("ServerHelloServerFinished"),
	}

	// testServerHandshakerOptions are the server-side handshaker options used
	// for testing.
	testServerHandshakerOptions = &ServerHandshakerOptions{
		MinTLSVersion: commonpb.TLSVersion_TLS1_2,
		MaxTLSVersion: commonpb.TLSVersion_TLS1_3,
		TLSCiphersuites: []commonpb.Ciphersuite{
			commonpb.Ciphersuite_AES_128_GCM_SHA256,
			commonpb.Ciphersuite_AES_256_GCM_SHA384,
			commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
		},
		LocalIdentities: []*commonpb.Identity{
			{
				IdentityOneof: &commonpb.Identity_SpiffeId{
					SpiffeId: "server_local_spiffe_id",
				},
			},
			{
				IdentityOneof: &commonpb.Identity_Hostname{
					Hostname: "server_local_hostname",
				},
			},
		},
	}

	// testServerStart is the ServerSessionStartReq message that the S2A expects
	// to receive from the test server.
	testServerStart = &s2apb.ServerSessionStartReq{
		ApplicationProtocols: []string{"grpc"},
		MinTlsVersion:        commonpb.TLSVersion_TLS1_2,
		MaxTlsVersion:        commonpb.TLSVersion_TLS1_3,
		TlsCiphersuites: []commonpb.Ciphersuite{
			commonpb.Ciphersuite_AES_128_GCM_SHA256,
			commonpb.Ciphersuite_AES_256_GCM_SHA384,
			commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
		},
		LocalIdentities: []*commonpb.Identity{
			{
				IdentityOneof: &commonpb.Identity_SpiffeId{
					SpiffeId: "server_local_spiffe_id",
				},
			},
			{
				IdentityOneof: &commonpb.Identity_Hostname{
					Hostname: "server_local_hostname",
				},
			},
		},
		InBytes: []byte("ClientHello"),
	}

	// testServerNext is the SessionNextReq message that the S2A expects to
	// receive second from the test server.
	testServerNext = &s2apb.SessionNextReq{
		InBytes: []byte("ClientFinished"),
	}

	testClientSessionResult = &s2apb.SessionResult{
		ApplicationProtocol: "grpc",
		State: &s2apb.SessionState{
			TlsVersion:     commonpb.TLSVersion_TLS1_3,
			TlsCiphersuite: commonpb.Ciphersuite_AES_128_GCM_SHA256,
			InSequence:     0,
			OutSequence:    0,
			InKey:          make([]byte, 32),
			OutKey:         make([]byte, 32),
		},
		PeerIdentity: &commonpb.Identity{
			IdentityOneof: &commonpb.Identity_SpiffeId{
				SpiffeId: "client_local_spiffe_id",
			},
		},
		LocalIdentity: &commonpb.Identity{
			IdentityOneof: &commonpb.Identity_SpiffeId{
				SpiffeId: "server_local_spiffe_id",
			},
		},
		LocalCertFingerprint: []byte("client_cert_fingerprint"),
		PeerCertFingerprint:  []byte("server_cert_fingerprint"),
	}

	testServerSessionResult = &s2apb.SessionResult{
		ApplicationProtocol: "grpc",
		State: &s2apb.SessionState{
			TlsVersion:     commonpb.TLSVersion_TLS1_3,
			TlsCiphersuite: commonpb.Ciphersuite_AES_128_GCM_SHA256,
			InSequence:     0,
			OutSequence:    0,
			InKey:          make([]byte, 32),
			OutKey:         make([]byte, 32),
		},
		PeerIdentity: &commonpb.Identity{
			IdentityOneof: &commonpb.Identity_SpiffeId{
				SpiffeId: "server_local_spiffe_id",
			},
		},
		LocalIdentity: &commonpb.Identity{
			IdentityOneof: &commonpb.Identity_SpiffeId{
				SpiffeId: "client_local_spiffe_id",
			},
		},
		LocalCertFingerprint: []byte("server_cert_fingerprint"),
		PeerCertFingerprint:  []byte("client_cert_fingerprint"),
	}
	testResultWithoutLocalIdentity = &s2apb.SessionResult{
		ApplicationProtocol: "grpc",
		State: &s2apb.SessionState{
			TlsVersion:     commonpb.TLSVersion_TLS1_3,
			TlsCiphersuite: commonpb.Ciphersuite_AES_128_GCM_SHA256,
			InSequence:     0,
			OutSequence:    0,
			InKey:          make([]byte, 32),
			OutKey:         make([]byte, 32),
		},
		PeerIdentity: &commonpb.Identity{
			IdentityOneof: &commonpb.Identity_SpiffeId{
				SpiffeId: "server_local_spiffe_id",
			},
		},
		LocalCertFingerprint: []byte("server_cert_fingerprint"),
		PeerCertFingerprint:  []byte("client_cert_fingerprint"),
	}
)

// fakeConn is a fake implementation of the net.Conn interface that is used for
// testing.
type fakeConn struct {
	net.Conn
	in  *bytes.Buffer
	out *bytes.Buffer
}

func (fc *fakeConn) Read(b []byte) (n int, err error)  { return fc.in.Read(b) }
func (fc *fakeConn) Write(b []byte) (n int, err error) { return fc.out.Write(b) }
func (fc *fakeConn) Close() error                      { return nil }

// fakeInvalidConn is a fake implementation of a invalid net.Conn interface
// that is used for testing.
type fakeInvalidConn struct {
	net.Conn
}

func (fc *fakeInvalidConn) Read(_ []byte) (n int, err error)  { return 0, io.EOF }
func (fc *fakeInvalidConn) Write(_ []byte) (n int, err error) { return 0, nil }
func (fc *fakeInvalidConn) Close() error                      { return nil }

// fakeStream is a fake implementation of the grpc.ClientStream interface that
// is used for testing.
type fakeStream struct {
	grpc.ClientStream
	t                   *testing.T
	fc                  *fakeConn
	expectedClientStart *s2apb.ClientSessionStartReq
	expectedServerStart *s2apb.ServerSessionStartReq
	expectToken         bool
	// expectedResp is the expected SessionResp message from the handshaker
	// service.
	expectedResp *s2apb.SessionResp
	// isFirstAccess indicates whether the first call to the handshaker service
	// has been made or not.
	isFirstAccess          bool
	isClient               bool
	isLocalIdentityMissing bool
}

func (fs *fakeStream) Recv() (*s2apb.SessionResp, error) {
	resp := fs.expectedResp
	fs.expectedResp = nil
	return resp, nil
}
func (fs *fakeStream) Send(req *s2apb.SessionReq) error {
	var resp *s2apb.SessionResp
	if fs.expectToken {
		if len(req.GetAuthMechanisms()) == 0 {
			return fmt.Errorf("request to S2A did not contain any tokens")
		}
		// Ensure that every token appearing in the request has a valid token.
		for _, authMechanism := range req.GetAuthMechanisms() {
			if authMechanism.GetToken() != testAccessToken {
				return fmt.Errorf("request to S2A contained invalid token")
			}
		}
	}
	if !fs.isFirstAccess {
		// Generate the bytes to be returned by Recv() for the first handshake
		// message.
		fs.isFirstAccess = true
		if fs.isClient {
			if diff := cmp.Diff(req.GetClientStart(), fs.expectedClientStart, protocmp.Transform()); diff != "" {
				return fmt.Errorf("client start message is incorrect, (-want +got):\n%s", diff)
			}
			resp = &s2apb.SessionResp{
				OutFrames: []byte("ClientHello"),
				// There are no consumed bytes for a client start message
				BytesConsumed: 0,
			}
		} else {
			// Expect a server start message.
			if req.GetServerStart() == nil {
				return errors.New("first request from server does not have server start")
			}
			if diff := cmp.Diff(req.GetServerStart(), fs.expectedServerStart, protocmp.Transform()); diff != "" {
				return fmt.Errorf("server start message is incorrect, (-want +got):\n%s", diff)
			}
			fs.fc.in.Write([]byte("ClientFinished"))
			resp = &s2apb.SessionResp{
				OutFrames: []byte("ServerHelloServerFinished"),
				// Simulate consuming the ClientHello message.
				BytesConsumed: uint32(len("ClientHello")),
			}
		}
	} else {
		// Construct a SessionResp message that contains the handshake result.
		if fs.isClient {
			// Expect next message with "ServerHelloServerFinished".
			if req.GetNext() == nil {
				return errors.New("second request from client does not have next")
			}
			if got, want := cmp.Equal(req.GetNext(), testClientNext, protocmp.Transform()), true; got != want {
				return errors.New("client next message is incorrect")
			}
			if fs.isLocalIdentityMissing {
				resp = &s2apb.SessionResp{
					Result:        testResultWithoutLocalIdentity,
					BytesConsumed: uint32(len("ClientFinished")),
				}
			} else {
				resp = &s2apb.SessionResp{
					Result:        testClientSessionResult,
					BytesConsumed: uint32(len("ServerHelloServerFinished")),
				}
			}
		} else {
			// Expect next message with "ClientFinished".
			if req.GetNext() == nil {
				return errors.New("second request from server does not have next")
			}
			if got, want := cmp.Equal(req.GetNext(), testServerNext, protocmp.Transform()), true; got != want {
				return errors.New("server next message is incorrect")
			}
			if fs.isLocalIdentityMissing {
				resp = &s2apb.SessionResp{
					Result:        testResultWithoutLocalIdentity,
					BytesConsumed: uint32(len("ClientFinished")),
				}
			} else {
				resp = &s2apb.SessionResp{
					Result:        testServerSessionResult,
					BytesConsumed: uint32(len("ClientFinished")),
				}
			}
		}
	}
	fs.expectedResp = resp
	return nil
}

func (*fakeStream) CloseSend() error { return nil }

// fakeInvalidStream is a fake implementation of an invalid grpc.ClientStream
// interface that is used for testing.
type fakeInvalidStream struct {
	grpc.ClientStream
}

func (*fakeInvalidStream) Recv() (*s2apb.SessionResp, error) { return &s2apb.SessionResp{}, nil }
func (*fakeInvalidStream) Send(*s2apb.SessionReq) error      { return nil }
func (*fakeInvalidStream) CloseSend() error                  { return nil }

type fakeAccessTokenManager struct {
	acceptedIdentity   *commonpb.Identity
	accessToken        string
	allowEmptyIdentity bool
}

func (m *fakeAccessTokenManager) DefaultToken() (string, error) {
	if !m.allowEmptyIdentity {
		return "", fmt.Errorf("not allowed to get token for empty identity")
	}
	return m.accessToken, nil
}

func (m *fakeAccessTokenManager) Token(identity *commonpb.Identity) (string, error) {
	if identity == nil || cmp.Equal(identity, &commonpb.Identity{}, protocmp.Transform()) {
		if !m.allowEmptyIdentity {
			return "", fmt.Errorf("not allowed to get token for empty identity")
		}
		return m.accessToken, nil
	}
	if cmp.Equal(identity, m.acceptedIdentity, protocmp.Transform()) {
		return m.accessToken, nil
	}
	return "", fmt.Errorf("unable to get token")
}

// TestNewClientHandshaker creates a fake stream, and ensures that
// newClientHandshaker returns a valid client-side handshaker instance.
func TestNewClientHandshaker(t *testing.T) {
	stream := &fakeStream{}
	c := &fakeConn{}
	chs := newClientHandshaker(stream, c, testHSAddr, testClientHandshakerOptions, &fakeAccessTokenManager{})
	if chs.clientOpts != testClientHandshakerOptions || chs.conn != c {
		t.Errorf("handshaker parameters incorrect")
	}
}

// TestNewServerHandshaker creates a fake stream, and ensures that
// newServerHandshaker returns a valid server-side handshaker instance.
func TestNewServerHandshaker(t *testing.T) {
	stream := &fakeStream{}
	c := &fakeConn{}
	shs := newServerHandshaker(stream, c, testHSAddr, testServerHandshakerOptions, &fakeAccessTokenManager{})
	if shs.serverOpts != testServerHandshakerOptions || shs.conn != c {
		t.Errorf("handshaker parameters incorrect")
	}
}

func TestClientHandshakeSuccess(t *testing.T) {
	for _, tc := range []struct {
		description         string
		options             *ClientHandshakerOptions
		tokenManager        tokenmanager.AccessTokenManager
		expectedClientStart *s2apb.ClientSessionStartReq
	}{
		{
			description:         "full client options",
			options:             testClientHandshakerOptions,
			expectedClientStart: testClientStart,
		},
		{
			description: "full client options with no port in target name",
			options: &ClientHandshakerOptions{
				MinTLSVersion: commonpb.TLSVersion_TLS1_2,
				MaxTLSVersion: commonpb.TLSVersion_TLS1_3,
				TLSCiphersuites: []commonpb.Ciphersuite{
					commonpb.Ciphersuite_AES_128_GCM_SHA256,
					commonpb.Ciphersuite_AES_256_GCM_SHA384,
					commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
				},
				TargetIdentities: []*commonpb.Identity{
					{
						IdentityOneof: &commonpb.Identity_SpiffeId{
							SpiffeId: "target_spiffe_id",
						},
					},
					{
						IdentityOneof: &commonpb.Identity_Hostname{
							Hostname: "target_hostname",
						},
					},
				},
				LocalIdentity: &commonpb.Identity{
					IdentityOneof: &commonpb.Identity_SpiffeId{
						SpiffeId: "client_local_spiffe_id",
					},
				},
				TargetName: testHostname,
			},
			expectedClientStart: &s2apb.ClientSessionStartReq{
				ApplicationProtocols: []string{"grpc"},
				MinTlsVersion:        commonpb.TLSVersion_TLS1_2,
				MaxTlsVersion:        commonpb.TLSVersion_TLS1_3,
				TlsCiphersuites: []commonpb.Ciphersuite{
					commonpb.Ciphersuite_AES_128_GCM_SHA256,
					commonpb.Ciphersuite_AES_256_GCM_SHA384,
					commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
				},
				TargetIdentities: []*commonpb.Identity{
					{
						IdentityOneof: &commonpb.Identity_SpiffeId{
							SpiffeId: "target_spiffe_id",
						},
					},
					{
						IdentityOneof: &commonpb.Identity_Hostname{
							Hostname: "target_hostname",
						},
					},
				},
				LocalIdentity: &commonpb.Identity{
					IdentityOneof: &commonpb.Identity_SpiffeId{
						SpiffeId: "client_local_spiffe_id",
					},
				},
				TargetName: testHostname,
			},
		},
		{
			description: "full client options with no local identity",
			options: &ClientHandshakerOptions{
				MinTLSVersion: commonpb.TLSVersion_TLS1_2,
				MaxTLSVersion: commonpb.TLSVersion_TLS1_3,
				TLSCiphersuites: []commonpb.Ciphersuite{
					commonpb.Ciphersuite_AES_128_GCM_SHA256,
					commonpb.Ciphersuite_AES_256_GCM_SHA384,
					commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
				},
				TargetIdentities: []*commonpb.Identity{
					{
						IdentityOneof: &commonpb.Identity_SpiffeId{
							SpiffeId: "target_spiffe_id",
						},
					},
					{
						IdentityOneof: &commonpb.Identity_Hostname{
							Hostname: "target_hostname",
						},
					},
				},
				TargetName: testHostname + ":1234",
			},
			expectedClientStart: &s2apb.ClientSessionStartReq{
				ApplicationProtocols: []string{"grpc"},
				MinTlsVersion:        commonpb.TLSVersion_TLS1_2,
				MaxTlsVersion:        commonpb.TLSVersion_TLS1_3,
				TlsCiphersuites: []commonpb.Ciphersuite{
					commonpb.Ciphersuite_AES_128_GCM_SHA256,
					commonpb.Ciphersuite_AES_256_GCM_SHA384,
					commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
				},
				TargetIdentities: []*commonpb.Identity{
					{
						IdentityOneof: &commonpb.Identity_SpiffeId{
							SpiffeId: "target_spiffe_id",
						},
					},
					{
						IdentityOneof: &commonpb.Identity_Hostname{
							Hostname: "target_hostname",
						},
					},
				},
				TargetName: testHostname,
			},
		},
		{
			description:         "full client options, sending tokens",
			options:             testClientHandshakerOptions,
			expectedClientStart: testClientStart,
			tokenManager: &fakeAccessTokenManager{
				accessToken: testAccessToken,
				acceptedIdentity: &commonpb.Identity{
					IdentityOneof: &commonpb.Identity_SpiffeId{
						SpiffeId: "client_local_spiffe_id",
					},
				},
			},
		},
		{
			description: "full client options with no local identity, sending tokens",
			options: &ClientHandshakerOptions{
				MinTLSVersion: commonpb.TLSVersion_TLS1_2,
				MaxTLSVersion: commonpb.TLSVersion_TLS1_3,
				TLSCiphersuites: []commonpb.Ciphersuite{
					commonpb.Ciphersuite_AES_128_GCM_SHA256,
					commonpb.Ciphersuite_AES_256_GCM_SHA384,
					commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
				},
				TargetIdentities: []*commonpb.Identity{
					{
						IdentityOneof: &commonpb.Identity_SpiffeId{
							SpiffeId: "target_spiffe_id",
						},
					},
					{
						IdentityOneof: &commonpb.Identity_Hostname{
							Hostname: "target_hostname",
						},
					},
				},
				TargetName: testHostname + ":1234",
			},
			expectedClientStart: &s2apb.ClientSessionStartReq{
				ApplicationProtocols: []string{"grpc"},
				MinTlsVersion:        commonpb.TLSVersion_TLS1_2,
				MaxTlsVersion:        commonpb.TLSVersion_TLS1_3,
				TlsCiphersuites: []commonpb.Ciphersuite{
					commonpb.Ciphersuite_AES_128_GCM_SHA256,
					commonpb.Ciphersuite_AES_256_GCM_SHA384,
					commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
				},
				TargetIdentities: []*commonpb.Identity{
					{
						IdentityOneof: &commonpb.Identity_SpiffeId{
							SpiffeId: "target_spiffe_id",
						},
					},
					{
						IdentityOneof: &commonpb.Identity_Hostname{
							Hostname: "target_hostname",
						},
					},
				},
				TargetName: testHostname,
			},
			tokenManager: &fakeAccessTokenManager{
				accessToken:        testAccessToken,
				allowEmptyIdentity: true,
			},
		},
	} {
		t.Run(tc.description, func(t *testing.T) {
			// Set up all fakes and input data.
			var errg errgroup.Group
			stream := &fakeStream{
				t:                   t,
				isClient:            true,
				expectedClientStart: tc.expectedClientStart,
				expectToken:         (tc.tokenManager != nil),
			}
			in := bytes.NewBuffer([]byte("ServerHelloServerFinished"))
			c := &fakeConn{
				in:  in,
				out: new(bytes.Buffer),
			}

			// Do the handshake.
			chs := newClientHandshaker(stream, c, testHSAddr, tc.options, tc.tokenManager)
			errg.Go(func() error {
				newConn, auth, err := chs.ClientHandshake(context.Background())
				if err != nil {
					return err
				}
				if auth.AuthType() != "s2a" {
					return errors.New("s2a auth type incorrect")
				}
				if newConn == nil {
					return errors.New("expected non-nil net.Conn")
				}
				if err := chs.Close(); err != nil {
					t.Errorf("chs.Close() failed: %v", err)
				}
				return nil
			})

			if err := errg.Wait(); err != nil {
				t.Errorf("client handshake failed: %v", err)
			}
		})
	}
}

func TestServerHandshakeSuccess(t *testing.T) {
	for _, tc := range []struct {
		description         string
		options             *ServerHandshakerOptions
		tokenManager        tokenmanager.AccessTokenManager
		expectedServerStart *s2apb.ServerSessionStartReq
	}{
		{
			description:         "full server options",
			options:             testServerHandshakerOptions,
			expectedServerStart: testServerStart,
		},
		{
			description: "full server options with no local identities",
			options: &ServerHandshakerOptions{
				MinTLSVersion: commonpb.TLSVersion_TLS1_2,
				MaxTLSVersion: commonpb.TLSVersion_TLS1_3,
				TLSCiphersuites: []commonpb.Ciphersuite{
					commonpb.Ciphersuite_AES_128_GCM_SHA256,
					commonpb.Ciphersuite_AES_256_GCM_SHA384,
					commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
				},
			},
			expectedServerStart: &s2apb.ServerSessionStartReq{
				ApplicationProtocols: []string{"grpc"},
				MinTlsVersion:        commonpb.TLSVersion_TLS1_2,
				MaxTlsVersion:        commonpb.TLSVersion_TLS1_3,
				TlsCiphersuites: []commonpb.Ciphersuite{
					commonpb.Ciphersuite_AES_128_GCM_SHA256,
					commonpb.Ciphersuite_AES_256_GCM_SHA384,
					commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
				},
				InBytes: []byte("ClientHello"),
			},
		},
		{
			description:         "full server options, sending tokens",
			options:             testServerHandshakerOptions,
			expectedServerStart: testServerStart,
			tokenManager: &fakeAccessTokenManager{
				accessToken: testAccessToken,
				acceptedIdentity: &commonpb.Identity{
					IdentityOneof: &commonpb.Identity_SpiffeId{
						SpiffeId: "server_local_spiffe_id",
					},
				},
			},
		},
		{
			description: "full server options with no local identity, sending tokens",
			options: &ServerHandshakerOptions{
				MinTLSVersion: commonpb.TLSVersion_TLS1_2,
				MaxTLSVersion: commonpb.TLSVersion_TLS1_3,
				TLSCiphersuites: []commonpb.Ciphersuite{
					commonpb.Ciphersuite_AES_128_GCM_SHA256,
					commonpb.Ciphersuite_AES_256_GCM_SHA384,
					commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
				},
			},
			expectedServerStart: &s2apb.ServerSessionStartReq{
				ApplicationProtocols: []string{"grpc"},
				MinTlsVersion:        commonpb.TLSVersion_TLS1_2,
				MaxTlsVersion:        commonpb.TLSVersion_TLS1_3,
				TlsCiphersuites: []commonpb.Ciphersuite{
					commonpb.Ciphersuite_AES_128_GCM_SHA256,
					commonpb.Ciphersuite_AES_256_GCM_SHA384,
					commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
				},
				InBytes: []byte("ClientHello"),
			},
			tokenManager: &fakeAccessTokenManager{
				accessToken:        testAccessToken,
				allowEmptyIdentity: true,
			},
		},
	} {
		t.Run(tc.description, func(t *testing.T) {
			// Set up all fakes and input data.
			var errg errgroup.Group
			in := bytes.NewBuffer([]byte("ClientHello"))
			c := &fakeConn{
				in:  in,
				out: new(bytes.Buffer),
			}
			stream := &fakeStream{
				t:                   t,
				fc:                  c,
				isClient:            false,
				expectedServerStart: tc.expectedServerStart,
				expectToken:         (tc.tokenManager != nil),
			}

			// Do the handshake.
			shs := newServerHandshaker(stream, c, testHSAddr, tc.options, tc.tokenManager)
			errg.Go(func() error {
				newConn, auth, err := shs.ServerHandshake(context.Background())
				if err != nil {
					return err
				}
				if auth.AuthType() != "s2a" {
					return errors.New("s2a auth type incorrect")
				}
				if newConn == nil {
					return errors.New("expected non-nil net.Conn")
				}
				if err = shs.Close(); err != nil {
					t.Errorf("shs.Close() failed: %v", err)
				}
				return nil
			})

			if err := errg.Wait(); err != nil {
				t.Errorf("server handshake failed: %v", err)
			}
		})
	}
}

// Note that there is no need to test the case where S2A is expecting a token
// and the application does not send a token, because this case is functionally
// the same as the application sending an invalid token.
func TestS2ARejectsTokenFromClient(t *testing.T) {
	stream := &fakeStream{
		t:           t,
		isClient:    true,
		expectToken: true,
	}
	in := bytes.NewBuffer([]byte("ServerHelloServerFinished"))
	c := &fakeConn{
		in:  in,
		out: new(bytes.Buffer),
	}
	tokenManager := &fakeAccessTokenManager{
		accessToken: "bad_access_token",
		acceptedIdentity: &commonpb.Identity{
			IdentityOneof: &commonpb.Identity_SpiffeId{
				SpiffeId: "client_local_spiffe_id",
			},
		},
	}

	chs := newClientHandshaker(stream, c, testHSAddr, testClientHandshakerOptions, tokenManager)
	_, _, err := chs.ClientHandshake(context.Background())
	if err == nil {
		t.Errorf("expected non-nil error from call to chs.ClientHandshake()")
	}
	if !strings.Contains(err.Error(), "request to S2A contained invalid token") {
		t.Errorf("chs.ClientHandshake() produced unexpected error: %v", err)
	}
}

func TestS2ARejectsTokenFromServer(t *testing.T) {
	stream := &fakeStream{
		t:           t,
		isClient:    false,
		expectToken: true,
	}
	in := bytes.NewBuffer([]byte("ClientHello"))
	c := &fakeConn{
		in:  in,
		out: new(bytes.Buffer),
	}
	tokenManager := &fakeAccessTokenManager{
		accessToken: "bad_access_token",
		acceptedIdentity: &commonpb.Identity{
			IdentityOneof: &commonpb.Identity_SpiffeId{
				SpiffeId: "server_local_spiffe_id",
			},
		},
	}

	chs := newServerHandshaker(stream, c, testHSAddr, testServerHandshakerOptions, tokenManager)
	_, _, err := chs.ServerHandshake(context.Background())
	if err == nil {
		t.Errorf("expected non-nil error from call to chs.ServerHandshake()")
	}
	if !strings.Contains(err.Error(), "request to S2A contained invalid token") {
		t.Errorf("chs.ServerHandshake() produced unexpected error: %v", err)
	}
}

func TestInvalidHandshaker(t *testing.T) {
	emptyCHS := &s2aHandshaker{
		isClient: false,
	}
	_, _, err := emptyCHS.ClientHandshake(context.Background())
	if err == nil {
		t.Error("ClientHandshake() should fail with server-side handshaker service")
	}
	emptySHS := &s2aHandshaker{
		isClient: true,
	}
	_, _, err = emptySHS.ServerHandshake(context.Background())
	if err == nil {
		t.Error("ServerHandshake() should fail with client-side handshaker service")
	}
}

// TestPeerNotResponding uses an invalid net.Conn instance and performs a
// client-side handshake to test the case when the peer is not responding.
func TestPeerNotResponding(t *testing.T) {
	stream := &fakeInvalidStream{}
	c := &fakeInvalidConn{}
	chs := &s2aHandshaker{
		stream:     stream,
		conn:       c,
		clientOpts: testClientHandshakerOptions,
		isClient:   true,
		hsAddr:     testHSAddr,
	}
	_, authInfo, err := chs.ClientHandshake(context.Background())
	if authInfo != nil {
		t.Error("expected non-nil S2A authInfo")
	}
	if got, want := err, errPeerNotResponding; got != want {
		t.Errorf("ClientHandshake() = %v, want %v", got, want)
	}
	if err = chs.Close(); err != nil {
		t.Errorf("chs.Close() failed: %v", err)
	}
}

// TestLocalIdentityNotSet performs a client-side handshake that fails
// because the local identity is not set in the handshake result.
func TestLocalIdentityNotSet(t *testing.T) {
	var errg errgroup.Group
	stream := &fakeStream{
		t:                      t,
		isClient:               true,
		isLocalIdentityMissing: true,
	}
	in := bytes.NewBuffer([]byte("ServerHelloServerFinished"))
	c := &fakeConn{
		in:  in,
		out: new(bytes.Buffer),
	}
	chs := &s2aHandshaker{
		stream:     stream,
		conn:       c,
		clientOpts: testClientHandshakerOptions,
		isClient:   true,
		hsAddr:     testHSAddr,
	}
	errg.Go(func() error {
		newConn, auth, err := chs.ClientHandshake(context.Background())
		if cmp.Equal(err, errors.New("local identity must be populated in session result"), cmpopts.EquateErrors()) {
			return fmt.Errorf("unexpected error: %v", err)
		}
		if auth != nil {
			return errors.New("expected nil credentials.AuthInfo")
		}
		if newConn != nil {
			return errors.New("expected nil net.Conn")
		}
		return nil
	})

	if err := errg.Wait(); err != nil {
		t.Errorf("client handshake failed: %v", err)
	}
}

func TestGetAuthMechanismsForClient(t *testing.T) {
	sortProtos := cmpopts.SortSlices(func(m1, m2 *s2apb.AuthenticationMechanism) bool { return m1.String() < m2.String() })
	for _, tc := range []struct {
		description            string
		options                *ClientHandshakerOptions
		tokenManager           tokenmanager.AccessTokenManager
		expectedAuthMechanisms []*s2apb.AuthenticationMechanism
	}{
		{
			description:            "token manager is nil",
			tokenManager:           nil,
			expectedAuthMechanisms: nil,
		},
		{
			description: "token manager expects empty identity",
			tokenManager: &fakeAccessTokenManager{
				accessToken:        testAccessToken,
				allowEmptyIdentity: true,
			},
			expectedAuthMechanisms: []*s2apb.AuthenticationMechanism{
				{
					MechanismOneof: &s2apb.AuthenticationMechanism_Token{
						Token: testAccessToken,
					},
				},
			},
		},
		{
			description: "token manager does not expect empty identity",
			tokenManager: &fakeAccessTokenManager{
				allowEmptyIdentity: false,
			},
			expectedAuthMechanisms: nil,
		},
		{
			description: "token manager expects SPIFFE ID",
			options: &ClientHandshakerOptions{
				LocalIdentity: &commonpb.Identity{
					IdentityOneof: &commonpb.Identity_SpiffeId{
						SpiffeId: "allowed_spiffe_id",
					},
				},
			},
			tokenManager: &fakeAccessTokenManager{
				accessToken: testAccessToken,
				acceptedIdentity: &commonpb.Identity{
					IdentityOneof: &commonpb.Identity_SpiffeId{
						SpiffeId: "allowed_spiffe_id",
					},
				},
			},
			expectedAuthMechanisms: []*s2apb.AuthenticationMechanism{
				{
					Identity: &commonpb.Identity{
						IdentityOneof: &commonpb.Identity_SpiffeId{
							SpiffeId: "allowed_spiffe_id",
						},
					},
					MechanismOneof: &s2apb.AuthenticationMechanism_Token{
						Token: testAccessToken,
					},
				},
			},
		},
		{
			description: "token manager does not expect hostname",
			options: &ClientHandshakerOptions{
				LocalIdentity: &commonpb.Identity{
					IdentityOneof: &commonpb.Identity_Hostname{
						Hostname: "disallowed_hostname",
					},
				},
			},
			tokenManager:           &fakeAccessTokenManager{},
			expectedAuthMechanisms: nil,
		},
	} {
		t.Run(tc.description, func(t *testing.T) {
			handshaker := newClientHandshaker(nil, nil, "", tc.options, tc.tokenManager)
			authMechanisms := handshaker.getAuthMechanisms()
			if got, want := (authMechanisms == nil), (tc.expectedAuthMechanisms == nil); got != want {
				t.Errorf("authMechanisms == nil: %t, tc.expectedAuthMechanisms == nil: %t", got, want)
			}
			if authMechanisms != nil && tc.expectedAuthMechanisms != nil {
				if diff := cmp.Diff(authMechanisms, tc.expectedAuthMechanisms, protocmp.Transform(), sortProtos); diff != "" {
					t.Errorf("handshaker.getAuthMechanisms() returned incorrect slice, (-want +got):\n%s", diff)
				}
			}
		})
	}
}

func TestGetAuthMechanismsForServer(t *testing.T) {
	sortProtos := cmpopts.SortSlices(func(m1, m2 *s2apb.AuthenticationMechanism) bool { return m1.String() < m2.String() })
	for _, tc := range []struct {
		description            string
		options                *ServerHandshakerOptions
		tokenManager           tokenmanager.AccessTokenManager
		expectedAuthMechanisms []*s2apb.AuthenticationMechanism
	}{
		{
			description:            "token manager is nil",
			tokenManager:           nil,
			expectedAuthMechanisms: nil,
		},
		{
			description: "token manager expects empty identity",
			tokenManager: &fakeAccessTokenManager{
				accessToken:        testAccessToken,
				allowEmptyIdentity: true,
			},
			expectedAuthMechanisms: []*s2apb.AuthenticationMechanism{
				{
					MechanismOneof: &s2apb.AuthenticationMechanism_Token{
						Token: testAccessToken,
					},
				},
			},
		},
		{
			description: "token manager does not expect empty identity",
			tokenManager: &fakeAccessTokenManager{
				allowEmptyIdentity: false,
			},
			expectedAuthMechanisms: nil,
		},
		{
			description: "token manager expects 2 SPIFFE IDs",
			options: &ServerHandshakerOptions{
				LocalIdentities: []*commonpb.Identity{
					{
						IdentityOneof: &commonpb.Identity_SpiffeId{
							SpiffeId: "allowed_spiffe_id",
						},
					},
					{
						IdentityOneof: &commonpb.Identity_SpiffeId{
							SpiffeId: "allowed_spiffe_id",
						},
					},
				},
			},
			tokenManager: &fakeAccessTokenManager{
				accessToken: testAccessToken,
				acceptedIdentity: &commonpb.Identity{
					IdentityOneof: &commonpb.Identity_SpiffeId{
						SpiffeId: "allowed_spiffe_id",
					},
				},
			},
			expectedAuthMechanisms: []*s2apb.AuthenticationMechanism{
				{
					Identity: &commonpb.Identity{
						IdentityOneof: &commonpb.Identity_SpiffeId{
							SpiffeId: "allowed_spiffe_id",
						},
					},
					MechanismOneof: &s2apb.AuthenticationMechanism_Token{
						Token: testAccessToken,
					},
				},
				{
					Identity: &commonpb.Identity{
						IdentityOneof: &commonpb.Identity_SpiffeId{
							SpiffeId: "allowed_spiffe_id",
						},
					},
					MechanismOneof: &s2apb.AuthenticationMechanism_Token{
						Token: testAccessToken,
					},
				},
			},
		},
		{
			description: "token manager expects a SPIFFE ID but does not expect hostname",
			options: &ServerHandshakerOptions{
				LocalIdentities: []*commonpb.Identity{
					{
						IdentityOneof: &commonpb.Identity_SpiffeId{
							SpiffeId: "allowed_spiffe_id",
						},
					},
					{
						IdentityOneof: &commonpb.Identity_Hostname{
							Hostname: "disallowed_hostname",
						},
					},
				},
			},
			tokenManager: &fakeAccessTokenManager{
				accessToken: testAccessToken,
				acceptedIdentity: &commonpb.Identity{
					IdentityOneof: &commonpb.Identity_SpiffeId{
						SpiffeId: "allowed_spiffe_id",
					},
				},
			},
			expectedAuthMechanisms: []*s2apb.AuthenticationMechanism{
				{
					Identity: &commonpb.Identity{
						IdentityOneof: &commonpb.Identity_SpiffeId{
							SpiffeId: "allowed_spiffe_id",
						},
					},
					MechanismOneof: &s2apb.AuthenticationMechanism_Token{
						Token: testAccessToken,
					},
				},
			},
		},
	} {
		t.Run(tc.description, func(t *testing.T) {
			handshaker := newServerHandshaker(nil, nil, "", tc.options, tc.tokenManager)
			authMechanisms := handshaker.getAuthMechanisms()
			if got, want := (authMechanisms == nil), (tc.expectedAuthMechanisms == nil); got != want {
				t.Errorf("authMechanisms == nil: %t, tc.expectedAuthMechanisms == nil: %t", got, want)
			}
			if authMechanisms != nil && tc.expectedAuthMechanisms != nil {
				if diff := cmp.Diff(authMechanisms, tc.expectedAuthMechanisms, protocmp.Transform(), sortProtos); diff != "" {
					t.Errorf("handshaker.getAuthMechanisms() returned incorrect slice, (-want +got):\n%s", diff)
				}
			}
		})
	}
}

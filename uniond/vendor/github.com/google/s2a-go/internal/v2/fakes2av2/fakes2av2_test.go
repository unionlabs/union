/*
 *
 * Copyright 2022 Google LLC
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

package fakes2av2

import (
	"context"
	"crypto"
	"crypto/rand"
	"crypto/sha256"
	"crypto/tls"
	"errors"
	"fmt"
	"log"
	"net"
	"sync"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/protobuf/testing/protocmp"

	commonpb "github.com/google/s2a-go/internal/proto/v2/common_go_proto"
	s2av2ctx "github.com/google/s2a-go/internal/proto/v2/s2a_context_go_proto"
	s2av2pb "github.com/google/s2a-go/internal/proto/v2/s2a_go_proto"
)

const (
	defaultTimeout = 10.0 * time.Second
)

func startFakeS2Av2Server(wg *sync.WaitGroup) (address string, stop func(), err error) {
	// Pick unused port.
	listener, err := net.Listen("tcp", ":0")
	address = listener.Addr().String()
	if err != nil {
		log.Fatalf("Failed to listen on address %s: %v", listener.Addr().String(), err)
	}
	s := grpc.NewServer()
	log.Printf("Server: started gRPC Fake S2Av2 Server on address: %s", listener.Addr().String())
	s2av2pb.RegisterS2AServiceServer(s, &Server{ExpectedToken: "valid_token"})
	go func() {
		wg.Done()
		if err := s.Serve(listener); err != nil {
			log.Printf("Failed to serve: %v", err)
		}
	}()
	return address, func() { s.Stop() }, nil
}

func TestSetUpSession(t *testing.T) {
	// Start up fake S2Av2 server.
	var wg sync.WaitGroup
	wg.Add(1)
	address, stop, err := startFakeS2Av2Server(&wg)
	wg.Wait()
	if err != nil {
		log.Fatalf("Failed to set up fake S2Av2 server.")
	}

	for _, tc := range []struct {
		description      string
		request          *s2av2pb.SessionReq
		expErr           error
		expectedResponse *s2av2pb.SessionResp
	}{
		{
			description: "Get TLS config for client.",
			request: &s2av2pb.SessionReq{
				AuthenticationMechanisms: []*s2av2pb.AuthenticationMechanism{
					{
						MechanismOneof: &s2av2pb.AuthenticationMechanism_Token{Token: "valid_token"},
					},
				},
				ReqOneof: &s2av2pb.SessionReq_GetTlsConfigurationReq{
					GetTlsConfigurationReq: &s2av2pb.GetTlsConfigurationReq{
						ConnectionSide: commonpb.ConnectionSide_CONNECTION_SIDE_CLIENT,
					},
				},
			},
			expectedResponse: &s2av2pb.SessionResp{
				Status: &s2av2pb.Status{
					Code: uint32(codes.OK),
				},
				RespOneof: &s2av2pb.SessionResp_GetTlsConfigurationResp{
					GetTlsConfigurationResp: &s2av2pb.GetTlsConfigurationResp{
						TlsConfiguration: &s2av2pb.GetTlsConfigurationResp_ClientTlsConfiguration_{
							ClientTlsConfiguration: &s2av2pb.GetTlsConfigurationResp_ClientTlsConfiguration{
								CertificateChain: []string{
									string(clientCert),
								},
								MinTlsVersion: commonpb.TLSVersion_TLS_VERSION_1_3,
								MaxTlsVersion: commonpb.TLSVersion_TLS_VERSION_1_3,
							},
						},
					},
				},
			},
		},
		{
			description: "Get TLS config for server.",
			request: &s2av2pb.SessionReq{
				AuthenticationMechanisms: []*s2av2pb.AuthenticationMechanism{
					{
						MechanismOneof: &s2av2pb.AuthenticationMechanism_Token{Token: "valid_token"},
					},
				},
				ReqOneof: &s2av2pb.SessionReq_GetTlsConfigurationReq{
					GetTlsConfigurationReq: &s2av2pb.GetTlsConfigurationReq{
						ConnectionSide: commonpb.ConnectionSide_CONNECTION_SIDE_SERVER,
					},
				},
			},
			expectedResponse: &s2av2pb.SessionResp{
				Status: &s2av2pb.Status{
					Code: uint32(codes.OK),
				},
				RespOneof: &s2av2pb.SessionResp_GetTlsConfigurationResp{
					GetTlsConfigurationResp: &s2av2pb.GetTlsConfigurationResp{
						TlsConfiguration: &s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration_{
							ServerTlsConfiguration: &s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration{
								CertificateChain: []string{
									string(serverCert),
								},
								MinTlsVersion:            commonpb.TLSVersion_TLS_VERSION_1_3,
								MaxTlsVersion:            commonpb.TLSVersion_TLS_VERSION_1_3,
								TlsResumptionEnabled:     false,
								RequestClientCertificate: s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration_REQUEST_AND_REQUIRE_CLIENT_CERTIFICATE_AND_VERIFY,
								MaxOverheadOfTicketAead:  0,
							},
						},
					},
				},
			},
		},
		{
			description: "Get TLS config error -- invalid connection side",
			request: &s2av2pb.SessionReq{
				AuthenticationMechanisms: []*s2av2pb.AuthenticationMechanism{
					{
						MechanismOneof: &s2av2pb.AuthenticationMechanism_Token{Token: "valid_token"},
					},
				},
				ReqOneof: &s2av2pb.SessionReq_GetTlsConfigurationReq{
					GetTlsConfigurationReq: &s2av2pb.GetTlsConfigurationReq{
						ConnectionSide: commonpb.ConnectionSide_CONNECTION_SIDE_UNSPECIFIED,
					},
				},
			},
			expectedResponse: &s2av2pb.SessionResp{
				Status: &s2av2pb.Status{
					Code:    uint32(codes.InvalidArgument),
					Details: "unknown ConnectionSide: CONNECTION_SIDE_UNSPECIFIED",
				},
			},
		},
		{
			description: "Get TLS config error -- invalid token",
			request: &s2av2pb.SessionReq{
				AuthenticationMechanisms: []*s2av2pb.AuthenticationMechanism{
					{
						MechanismOneof: &s2av2pb.AuthenticationMechanism_Token{Token: "invalid_token"},
					},
				},
				ReqOneof: &s2av2pb.SessionReq_GetTlsConfigurationReq{
					GetTlsConfigurationReq: &s2av2pb.GetTlsConfigurationReq{
						ConnectionSide: commonpb.ConnectionSide_CONNECTION_SIDE_UNSPECIFIED,
					},
				},
			},
			expErr: errors.New("rpc error: code = Unknown desc = SessionReq has no AuthenticationMechanism with a valid token"),
		},
		{
			description: "Get server TLS config -- empty authmechanisms (S2A_ACCESS_TOKEN env var not set)",
			request: &s2av2pb.SessionReq{
				AuthenticationMechanisms: []*s2av2pb.AuthenticationMechanism{},
				ReqOneof: &s2av2pb.SessionReq_GetTlsConfigurationReq{
					GetTlsConfigurationReq: &s2av2pb.GetTlsConfigurationReq{
						ConnectionSide: commonpb.ConnectionSide_CONNECTION_SIDE_SERVER,
					},
				},
			},
			expectedResponse: &s2av2pb.SessionResp{
				Status: &s2av2pb.Status{
					Code: uint32(codes.OK),
				},
				RespOneof: &s2av2pb.SessionResp_GetTlsConfigurationResp{
					GetTlsConfigurationResp: &s2av2pb.GetTlsConfigurationResp{
						TlsConfiguration: &s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration_{
							ServerTlsConfiguration: &s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration{
								CertificateChain: []string{
									string(serverCert),
								},
								MinTlsVersion:            commonpb.TLSVersion_TLS_VERSION_1_3,
								MaxTlsVersion:            commonpb.TLSVersion_TLS_VERSION_1_3,
								TlsResumptionEnabled:     false,
								RequestClientCertificate: s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration_REQUEST_AND_REQUIRE_CLIENT_CERTIFICATE_AND_VERIFY,
								MaxOverheadOfTicketAead:  0,
							},
						},
					},
				},
			},
		},
		{
			description: "Client Peer Verification",
			request: &s2av2pb.SessionReq{
				ReqOneof: &s2av2pb.SessionReq_ValidatePeerCertificateChainReq{
					ValidatePeerCertificateChainReq: &s2av2pb.ValidatePeerCertificateChainReq{
						Mode: s2av2pb.ValidatePeerCertificateChainReq_SPIFFE,
						PeerOneof: &s2av2pb.ValidatePeerCertificateChainReq_ClientPeer_{
							ClientPeer: &s2av2pb.ValidatePeerCertificateChainReq_ClientPeer{
								CertificateChain: [][]byte{clientDERCert},
							},
						},
					},
				},
			},
			expectedResponse: &s2av2pb.SessionResp{
				Status: &s2av2pb.Status{
					Code:    uint32(codes.OK),
					Details: "",
				},
				RespOneof: &s2av2pb.SessionResp_ValidatePeerCertificateChainResp{
					ValidatePeerCertificateChainResp: &s2av2pb.ValidatePeerCertificateChainResp{
						ValidationResult:  s2av2pb.ValidatePeerCertificateChainResp_SUCCESS,
						ValidationDetails: "client peer verification succeeded",
						Context:           &s2av2ctx.S2AContext{},
					},
				},
			},
		},
		{
			description: "Client Peer Verification -- failure",
			request: &s2av2pb.SessionReq{
				ReqOneof: &s2av2pb.SessionReq_ValidatePeerCertificateChainReq{
					ValidatePeerCertificateChainReq: &s2av2pb.ValidatePeerCertificateChainReq{
						Mode: s2av2pb.ValidatePeerCertificateChainReq_SPIFFE,
						PeerOneof: &s2av2pb.ValidatePeerCertificateChainReq_ClientPeer_{
							ClientPeer: &s2av2pb.ValidatePeerCertificateChainReq_ClientPeer{
								CertificateChain: [][]byte{},
							},
						},
					},
				},
			},
			expectedResponse: &s2av2pb.SessionResp{
				Status: &s2av2pb.Status{
					Code:    uint32(codes.OK),
					Details: "",
				},
				RespOneof: &s2av2pb.SessionResp_ValidatePeerCertificateChainResp{
					ValidatePeerCertificateChainResp: &s2av2pb.ValidatePeerCertificateChainResp{
						ValidationResult:  s2av2pb.ValidatePeerCertificateChainResp_FAILURE,
						ValidationDetails: "client peer verification failed: client cert chain is empty",
						Context:           &s2av2ctx.S2AContext{},
					},
				},
			},
		},
		{
			description: "Server Peer Verification",
			request: &s2av2pb.SessionReq{
				ReqOneof: &s2av2pb.SessionReq_ValidatePeerCertificateChainReq{
					ValidatePeerCertificateChainReq: &s2av2pb.ValidatePeerCertificateChainReq{
						Mode: s2av2pb.ValidatePeerCertificateChainReq_SPIFFE,
						PeerOneof: &s2av2pb.ValidatePeerCertificateChainReq_ServerPeer_{
							ServerPeer: &s2av2pb.ValidatePeerCertificateChainReq_ServerPeer{
								CertificateChain: [][]byte{serverDERCert},
							},
						},
					},
				},
			},
			expectedResponse: &s2av2pb.SessionResp{
				Status: &s2av2pb.Status{
					Code:    uint32(codes.OK),
					Details: "",
				},
				RespOneof: &s2av2pb.SessionResp_ValidatePeerCertificateChainResp{
					ValidatePeerCertificateChainResp: &s2av2pb.ValidatePeerCertificateChainResp{
						ValidationResult:  s2av2pb.ValidatePeerCertificateChainResp_SUCCESS,
						ValidationDetails: "server peer verification succeeded",
						Context:           &s2av2ctx.S2AContext{},
					},
				},
			},
		},
	} {
		t.Run(tc.description, func(t *testing.T) {
			// Create new stream to server.
			opts := []grpc.DialOption{
				grpc.WithTransportCredentials(insecure.NewCredentials()),
				grpc.WithReturnConnectionError(),
				grpc.WithBlock(),
			}
			conn, err := grpc.Dial(address, opts...)
			if err != nil {
				t.Fatalf("Client: failed to connect: %v", err)
			}
			defer conn.Close()
			c := s2av2pb.NewS2AServiceClient(conn)
			log.Printf("Client: connected to: %s", address)
			ctx, cancel := context.WithTimeout(context.Background(), defaultTimeout)
			defer cancel()

			// Setup bidrectional streaming session.
			callOpts := []grpc.CallOption{}
			cstream, err := c.SetUpSession(ctx, callOpts...)
			if err != nil {
				t.Fatalf("Client: failed to setup bidirectional streaming RPC session: %v", err)
			}
			log.Printf("Client: set up bidirectional streaming RPC session.")

			// Send request.
			if err := cstream.Send(tc.request); err != nil {
				t.Fatalf("Client: failed to send SessionReq: %v", err)
			}
			log.Printf("Client: sent SessionReq")

			// Get the response.
			resp, err := cstream.Recv()
			if err != tc.expErr {
				if (err != nil) && (tc.expErr != nil) {
					if err.Error() != tc.expErr.Error() {
						t.Fatalf("err = %v, expErr = %v", err.Error(), tc.expErr.Error())
					}
				} else {
					t.Fatalf("err = %v, expErr = %v", err, tc.expErr)
				}
			}
			log.Printf("Client: received SessionResp")
			if diff := cmp.Diff(tc.expectedResponse, resp, protocmp.Transform()); diff != "" {
				t.Errorf("cstream.Recv() returned incorrect SessionResp, (-want +got):\n%s", diff)
			}
			log.Printf("resp matches tc.expectedResponse")
		})
	}
	stop()
}

func TestSetUpSessionPrivateKeyOperation(t *testing.T) {
	// Start up fake S2Av2 server.
	var wg sync.WaitGroup
	wg.Add(1)
	address, stop, err := startFakeS2Av2Server(&wg)
	wg.Wait()
	if err != nil {
		log.Fatalf("Failed to set up fake S2Av2 server.")
	}

	// Setup for client and server offloadPrivateKeyOperation test.
	clientTLSCert, err := tls.X509KeyPair(clientCert, clientKey)
	if err != nil {
		log.Fatalf("Failed during test setup: %v", err)
	}

	serverTLSCert, err := tls.X509KeyPair(serverCert, serverKey)
	if err != nil {
		log.Fatalf("Failed during test setup: %v", err)
	}

	testString := "Generate hash and sign this."

	// TODO(rmehta19): Investigate whether go crypto libraries compute hash.
	// If so, remove this line, and just pass testString to Sign and as InBytes.
	hsha256 := sha256.Sum256([]byte(testString))

	var opts crypto.Hash = crypto.SHA256
	signedWithClientKey, err := clientTLSCert.PrivateKey.(crypto.Signer).Sign(rand.Reader, hsha256[:], opts)
	if err != nil {
		log.Fatalf("Failed during test setup: %v", err)
	}
	signedWithServerKey, err := serverTLSCert.PrivateKey.(crypto.Signer).Sign(rand.Reader, hsha256[:], opts)
	if err != nil {
		log.Fatalf("Failed during test setup: %v", err)
	}

	for _, tc := range []struct {
		description      string
		connSide         commonpb.ConnectionSide
		request          *s2av2pb.SessionReq
		expectedResponse *s2av2pb.SessionResp
	}{

		{
			description: "client side private key operation",
			connSide:    commonpb.ConnectionSide_CONNECTION_SIDE_CLIENT,
			request: &s2av2pb.SessionReq{
				ReqOneof: &s2av2pb.SessionReq_OffloadPrivateKeyOperationReq{
					OffloadPrivateKeyOperationReq: &s2av2pb.OffloadPrivateKeyOperationReq{
						Operation:          s2av2pb.OffloadPrivateKeyOperationReq_SIGN,
						SignatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_RSA_PKCS1_SHA256,
						InBytes: &s2av2pb.OffloadPrivateKeyOperationReq_Sha256Digest{
							Sha256Digest: []byte(hsha256[:]),
						},
					},
				},
			},
			expectedResponse: &s2av2pb.SessionResp{
				Status: &s2av2pb.Status{
					Code: uint32(codes.OK),
				},
				RespOneof: &s2av2pb.SessionResp_OffloadPrivateKeyOperationResp{
					OffloadPrivateKeyOperationResp: &s2av2pb.OffloadPrivateKeyOperationResp{
						OutBytes: signedWithClientKey,
					},
				},
			},
		},
		{
			description: "server side private key operation",
			connSide:    commonpb.ConnectionSide_CONNECTION_SIDE_SERVER,
			request: &s2av2pb.SessionReq{
				ReqOneof: &s2av2pb.SessionReq_OffloadPrivateKeyOperationReq{
					OffloadPrivateKeyOperationReq: &s2av2pb.OffloadPrivateKeyOperationReq{
						Operation:          s2av2pb.OffloadPrivateKeyOperationReq_SIGN,
						SignatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_RSA_PKCS1_SHA256,
						InBytes: &s2av2pb.OffloadPrivateKeyOperationReq_Sha256Digest{
							Sha256Digest: []byte(hsha256[:]),
						},
					},
				},
			},
			expectedResponse: &s2av2pb.SessionResp{
				Status: &s2av2pb.Status{
					Code: uint32(codes.OK),
				},
				RespOneof: &s2av2pb.SessionResp_OffloadPrivateKeyOperationResp{
					OffloadPrivateKeyOperationResp: &s2av2pb.OffloadPrivateKeyOperationResp{
						OutBytes: signedWithServerKey,
					},
				},
			},
		},
		{
			description: "client side private key operation -- invalid signature algorithm",
			connSide:    commonpb.ConnectionSide_CONNECTION_SIDE_CLIENT,
			request: &s2av2pb.SessionReq{
				ReqOneof: &s2av2pb.SessionReq_OffloadPrivateKeyOperationReq{
					OffloadPrivateKeyOperationReq: &s2av2pb.OffloadPrivateKeyOperationReq{
						Operation:          s2av2pb.OffloadPrivateKeyOperationReq_SIGN,
						SignatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_UNSPECIFIED,
						InBytes: &s2av2pb.OffloadPrivateKeyOperationReq_Sha256Digest{
							Sha256Digest: []byte(hsha256[:]),
						},
					},
				},
			},
			expectedResponse: &s2av2pb.SessionResp{
				Status: &s2av2pb.Status{
					Code:    uint32(codes.InvalidArgument),
					Details: fmt.Sprintf("invalid signature algorithm: %v", s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_UNSPECIFIED),
				},
			},
		},
	} {
		t.Run(tc.description, func(t *testing.T) {
			// Create new stream to server.
			opts := []grpc.DialOption{
				grpc.WithTransportCredentials(insecure.NewCredentials()),
				grpc.WithReturnConnectionError(),
				grpc.WithBlock(),
			}
			conn, err := grpc.Dial(address, opts...)
			if err != nil {
				t.Fatalf("Client: failed to connect: %v", err)
			}
			defer conn.Close()
			c := s2av2pb.NewS2AServiceClient(conn)
			log.Printf("Client: connected to: %s", address)
			ctx, cancel := context.WithTimeout(context.Background(), defaultTimeout)
			defer cancel()

			// Setup bidrectional streaming session.
			callOpts := []grpc.CallOption{}
			cstream, err := c.SetUpSession(ctx, callOpts...)
			if err != nil {
				t.Fatalf("Client: failed to setup bidirectional streaming RPC session: %v", err)
			}
			log.Printf("Client: set up bidirectional streaming RPC session.")

			// Send first SessionReq for TLS Config. Sets isClientSide to ensure correct
			// private key used to sign transcript.
			if err := cstream.Send(&s2av2pb.SessionReq{
				AuthenticationMechanisms: []*s2av2pb.AuthenticationMechanism{
					{
						MechanismOneof: &s2av2pb.AuthenticationMechanism_Token{
							Token: "valid_token",
						},
					},
				},
				ReqOneof: &s2av2pb.SessionReq_GetTlsConfigurationReq{
					GetTlsConfigurationReq: &s2av2pb.GetTlsConfigurationReq{
						ConnectionSide: tc.connSide,
					},
				},
			}); err != nil {
				t.Fatalf("Setup failed: failed to send initial SessionReq for TLS config: %v", err)
			}

			if _, err := cstream.Recv(); err != nil {
				t.Fatalf("Setup failed: failed to receive initial SessionResp for TLS config: %v", err)
			}

			// Send request.
			if err := cstream.Send(tc.request); err != nil {
				t.Fatalf("Client: failed to send SessionReq: %v", err)
			}
			log.Printf("Client: sent SessionReq")

			// Get the response.
			resp, err := cstream.Recv()
			if err != nil {
				t.Fatalf("Client: failed to receive SessionResp: %v", err)
			}
			log.Printf("Client: received SessionResp")
			if diff := cmp.Diff(tc.expectedResponse, resp, protocmp.Transform()); diff != "" {
				t.Errorf("cstream.Recv() returned incorrect SessionResp, (-want +got):\n%s", diff)
			}
			log.Printf("resp matches tc.expectedResponse")
		})
	}
	stop()
}

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

// Package fakes2av2 is a fake S2Av2 Go implementation.
package fakes2av2

import (
	"bytes"
	"crypto"
	"crypto/rand"
	"crypto/rsa"
	"crypto/tls"
	"crypto/x509"
	"errors"
	"fmt"
	"log"
	"time"

	"google.golang.org/grpc/codes"

	_ "embed"

	commonpb "github.com/google/s2a-go/internal/proto/v2/common_go_proto"
	s2av2ctx "github.com/google/s2a-go/internal/proto/v2/s2a_context_go_proto"
	s2av2pb "github.com/google/s2a-go/internal/proto/v2/s2a_go_proto"
)

var (
	//go:embed testdata/client_root_cert.pem
	clientCert []byte
	//go:embed testdata/client_root_cert.der
	clientDERCert []byte
	//go:embed testdata/client_root_key.pem
	clientKey []byte
	//go:embed testdata/server_root_cert.pem
	serverCert []byte
	//go:embed testdata/server_root_cert.der
	serverDERCert []byte
	//go:embed testdata/server_root_key.pem
	serverKey []byte
)

// Server is a fake S2A Server for testing.
type Server struct {
	s2av2pb.UnimplementedS2AServiceServer
	// ExpectedToken is the token S2Av2 expects to be attached to the SessionReq.
	ExpectedToken string
	// ShouldNotReturnClientCredentials indicates whether the fake S2Av2 should
	// not return credentials when GetTlsConfiguration is called by a client.
	ShouldNotReturnClientCredentials bool
	isAssistingClientSide            bool
	ServerAuthorizationPolicy        []byte
	// TODO(rmehta19): Decide whether to also store validationResult (bool).
	// Set this after validating token attached to first SessionReq. Check
	// this field before completing subsequent SessionReq.
}

// SetUpSession receives SessionReq, performs request, and returns a
// SessionResp, all on the server stream.
func (s *Server) SetUpSession(stream s2av2pb.S2AService_SetUpSessionServer) error {
	for {
		req, err := stream.Recv()
		if err != nil {
			log.Printf("Fake S2A Service: failed to receive SessionReq: %v", err)
			return err
		}
		// Call one of the 4 possible RespOneof's
		// TODO(rmehta19): Consider validating the body of the request.
		var resp *s2av2pb.SessionResp
		switch x := req.ReqOneof.(type) {
		case *s2av2pb.SessionReq_GetTlsConfigurationReq:
			if err := s.hasValidToken(req.GetAuthenticationMechanisms()); err != nil {
				log.Printf("Fake S2A Service: authentication error: %v", err)
				return err
			}
			if err := s.findConnectionSide(req); err != nil {
				resp = &s2av2pb.SessionResp{
					Status: &s2av2pb.Status{
						Code:    uint32(codes.InvalidArgument),
						Details: err.Error(),
					},
				}
				break
			}
			resp, err = getTLSConfiguration(req.GetGetTlsConfigurationReq(), s.ShouldNotReturnClientCredentials)
			if err != nil {
				log.Printf("Fake S2A Service: failed to build SessionResp with GetTlsConfigurationResp: %v", err)
				return err
			}
		case *s2av2pb.SessionReq_OffloadPrivateKeyOperationReq:
			resp, err = offloadPrivateKeyOperation(req.GetOffloadPrivateKeyOperationReq(), s.isAssistingClientSide)
			if err != nil {
				log.Printf("Fake S2A Service: failed to build SessionResp with OffloadPrivateKeyOperationResp: %v", err)
				return err
			}
		case *s2av2pb.SessionReq_OffloadResumptionKeyOperationReq:
			// TODO(rmehta19): Implement fake.
		case *s2av2pb.SessionReq_ValidatePeerCertificateChainReq:
			resp, err = validatePeerCertificateChain(req.GetValidatePeerCertificateChainReq(), s.ServerAuthorizationPolicy)
			if err != nil {
				log.Printf("Fake S2A Service: failed to build SessionResp with ValidatePeerCertificateChainResp: %v", err)
				return err
			}
		default:
			return fmt.Errorf("SessionReq.ReqOneof has unexpected type %T", x)
		}
		if err := stream.Send(resp); err != nil {
			log.Printf("Fake S2A Service: failed to send SessionResp: %v", err)
			return err
		}
	}
}

func (s *Server) findConnectionSide(req *s2av2pb.SessionReq) error {
	switch connSide := req.GetGetTlsConfigurationReq().GetConnectionSide(); connSide {
	case commonpb.ConnectionSide_CONNECTION_SIDE_CLIENT:
		s.isAssistingClientSide = true
	case commonpb.ConnectionSide_CONNECTION_SIDE_SERVER:
		s.isAssistingClientSide = false
	default:
		return fmt.Errorf("unknown ConnectionSide: %v", connSide)
	}
	return nil
}

func (s *Server) hasValidToken(authMechanisms []*s2av2pb.AuthenticationMechanism) error {
	if len(authMechanisms) == 0 {
		return nil
	}
	for _, v := range authMechanisms {
		token := v.GetToken()
		if token == s.ExpectedToken {
			return nil
		}
	}
	return errors.New("SessionReq has no AuthenticationMechanism with a valid token")
}

func offloadPrivateKeyOperation(req *s2av2pb.OffloadPrivateKeyOperationReq, isAssistingClientSide bool) (*s2av2pb.SessionResp, error) {
	switch x := req.GetOperation(); x {
	case s2av2pb.OffloadPrivateKeyOperationReq_SIGN:
		var root tls.Certificate
		var err error
		// Retrieve S2Av2 implementation of crypto.Signer.
		if isAssistingClientSide {
			root, err = tls.X509KeyPair(clientCert, clientKey)
			if err != nil {
				return nil, err
			}
		} else {
			root, err = tls.X509KeyPair(serverCert, serverKey)
			if err != nil {
				return nil, err
			}
		}
		var signedBytes []byte
		if req.GetSignatureAlgorithm() == s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_RSA_PKCS1_SHA256 {
			signedBytes, err = root.PrivateKey.(crypto.Signer).Sign(rand.Reader, req.GetSha256Digest(), crypto.SHA256)
			if err != nil {
				return nil, err
			}
		} else if req.GetSignatureAlgorithm() == s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_RSA_PSS_RSAE_SHA256 {
			opts := &rsa.PSSOptions{SaltLength: rsa.PSSSaltLengthEqualsHash, Hash: crypto.SHA256}
			signedBytes, err = root.PrivateKey.(crypto.Signer).Sign(rand.Reader, req.GetSha256Digest(), opts)
			if err != nil {
				return nil, err
			}
		} else {
			return &s2av2pb.SessionResp{
				Status: &s2av2pb.Status{
					Code:    uint32(codes.InvalidArgument),
					Details: fmt.Sprintf("invalid signature algorithm: %v", req.GetSignatureAlgorithm()),
				},
			}, nil
		}
		return &s2av2pb.SessionResp{
			Status: &s2av2pb.Status{
				Code: uint32(codes.OK),
			},
			RespOneof: &s2av2pb.SessionResp_OffloadPrivateKeyOperationResp{
				OffloadPrivateKeyOperationResp: &s2av2pb.OffloadPrivateKeyOperationResp{
					OutBytes: signedBytes,
				},
			},
		}, nil
	case s2av2pb.OffloadPrivateKeyOperationReq_DECRYPT:
		return nil, errors.New("decrypt operation not implemented yet")
	default:
		return nil, fmt.Errorf("unspecified private key operation requested: %d", x)
	}
}

func validatePeerCertificateChain(req *s2av2pb.ValidatePeerCertificateChainReq, serverAuthorizationPolicy []byte) (*s2av2pb.SessionResp, error) {
	switch x := req.PeerOneof.(type) {
	case *s2av2pb.ValidatePeerCertificateChainReq_ClientPeer_:
		return verifyClientPeer(req)
	case *s2av2pb.ValidatePeerCertificateChainReq_ServerPeer_:
		return verifyServerPeer(req, serverAuthorizationPolicy)
	default:
		err := fmt.Errorf("peer verification failed: invalid Peer type %T", x)
		return buildValidatePeerCertificateChainSessionResp(uint32(codes.InvalidArgument), err.Error(), s2av2pb.ValidatePeerCertificateChainResp_FAILURE, err.Error(), &s2av2ctx.S2AContext{}), err
	}
}

// TODO(rmehta19): Update this to return ciphersuites in Client/Server TlsConfiguration.
func getTLSConfiguration(req *s2av2pb.GetTlsConfigurationReq, shouldNotReturnClientCredentials bool) (*s2av2pb.SessionResp, error) {
	if req.GetConnectionSide() == commonpb.ConnectionSide_CONNECTION_SIDE_CLIENT {
		if shouldNotReturnClientCredentials {
			return &s2av2pb.SessionResp{
				Status: &s2av2pb.Status{
					Code: uint32(codes.OK),
				},
				RespOneof: &s2av2pb.SessionResp_GetTlsConfigurationResp{
					GetTlsConfigurationResp: &s2av2pb.GetTlsConfigurationResp{
						TlsConfiguration: &s2av2pb.GetTlsConfigurationResp_ClientTlsConfiguration_{
							ClientTlsConfiguration: &s2av2pb.GetTlsConfigurationResp_ClientTlsConfiguration{
								MinTlsVersion: commonpb.TLSVersion_TLS_VERSION_1_3,
								MaxTlsVersion: commonpb.TLSVersion_TLS_VERSION_1_3,
							},
						},
					},
				},
			}, nil
		}
		return &s2av2pb.SessionResp{
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
		}, nil
	} else if req.GetConnectionSide() == commonpb.ConnectionSide_CONNECTION_SIDE_SERVER {
		return &s2av2pb.SessionResp{
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
		}, nil
	}
	return nil, fmt.Errorf("unspecified connection side: %v", req.GetConnectionSide())
}

func buildValidatePeerCertificateChainSessionResp(StatusCode uint32, StatusDetails string, ValidationResult s2av2pb.ValidatePeerCertificateChainResp_ValidationResult, ValidationDetails string, Context *s2av2ctx.S2AContext) *s2av2pb.SessionResp {
	return &s2av2pb.SessionResp{
		Status: &s2av2pb.Status{
			Code:    StatusCode,
			Details: StatusDetails,
		},
		RespOneof: &s2av2pb.SessionResp_ValidatePeerCertificateChainResp{
			ValidatePeerCertificateChainResp: &s2av2pb.ValidatePeerCertificateChainResp{
				ValidationResult:  ValidationResult,
				ValidationDetails: ValidationDetails,
				Context:           Context,
			},
		},
	}
}

func verifyClientPeer(req *s2av2pb.ValidatePeerCertificateChainReq) (*s2av2pb.SessionResp, error) {
	derCertChain := req.GetClientPeer().CertificateChain
	if len(derCertChain) == 0 {
		s := "client peer verification failed: client cert chain is empty"
		return buildValidatePeerCertificateChainSessionResp(uint32(codes.OK), "", s2av2pb.ValidatePeerCertificateChainResp_FAILURE, s, &s2av2ctx.S2AContext{}), nil
	}

	// Obtain the set of root certificates.
	rootCertPool := x509.NewCertPool()
	if ok := rootCertPool.AppendCertsFromPEM(clientCert); ok != true {
		err := errors.New("client peer verification failed: S2Av2 could not obtain/parse roots")
		return buildValidatePeerCertificateChainSessionResp(uint32(codes.Internal), err.Error(), s2av2pb.ValidatePeerCertificateChainResp_FAILURE, err.Error(), &s2av2ctx.S2AContext{}), err
	}

	// Set the Intermediates: certs between leaf and root, excluding the leaf and root.
	intermediateCertPool := x509.NewCertPool()
	for i := 1; i < (len(derCertChain)); i++ {
		x509Cert, err := x509.ParseCertificate(derCertChain[i])
		if err != nil {
			return buildValidatePeerCertificateChainSessionResp(uint32(codes.InvalidArgument), err.Error(), s2av2pb.ValidatePeerCertificateChainResp_FAILURE, err.Error(), &s2av2ctx.S2AContext{}), err
		}
		intermediateCertPool.AddCert(x509Cert)
	}

	// Verify the leaf certificate.
	opts := x509.VerifyOptions{
		CurrentTime:   time.Now(),
		Roots:         rootCertPool,
		Intermediates: intermediateCertPool,
	}
	x509LeafCert, err := x509.ParseCertificate(derCertChain[0])
	if err != nil {
		s := fmt.Sprintf("client peer verification failed: %v", err)
		return buildValidatePeerCertificateChainSessionResp(uint32(codes.InvalidArgument), s, s2av2pb.ValidatePeerCertificateChainResp_FAILURE, s, &s2av2ctx.S2AContext{}), err
	}
	if _, err := x509LeafCert.Verify(opts); err != nil {
		s := fmt.Sprintf("client peer verification failed: %v", err)
		return buildValidatePeerCertificateChainSessionResp(uint32(codes.InvalidArgument), s, s2av2pb.ValidatePeerCertificateChainResp_FAILURE, s, &s2av2ctx.S2AContext{}), nil
	}
	return buildValidatePeerCertificateChainSessionResp(uint32(codes.OK), "", s2av2pb.ValidatePeerCertificateChainResp_SUCCESS, "client peer verification succeeded", &s2av2ctx.S2AContext{}), nil
}

func verifyServerPeer(req *s2av2pb.ValidatePeerCertificateChainReq, serverAuthorizationPolicy []byte) (*s2av2pb.SessionResp, error) {
	if serverAuthorizationPolicy != nil {
		if got := req.GetServerPeer().SerializedUnrestrictedClientPolicy; !bytes.Equal(got, serverAuthorizationPolicy) {
			err := fmt.Errorf("server peer verification failed: invalid server authorization policy, expected: %s, got: %s",
				serverAuthorizationPolicy, got)
			return buildValidatePeerCertificateChainSessionResp(uint32(codes.Internal), err.Error(), s2av2pb.ValidatePeerCertificateChainResp_FAILURE, err.Error(), &s2av2ctx.S2AContext{}), err
		}
	}
	derCertChain := req.GetServerPeer().CertificateChain
	if len(derCertChain) == 0 {
		s := "server peer verification failed: server cert chain is empty"
		return buildValidatePeerCertificateChainSessionResp(uint32(codes.OK), "", s2av2pb.ValidatePeerCertificateChainResp_FAILURE, s, &s2av2ctx.S2AContext{}), nil
	}

	// Obtain the set of root certificates.
	rootCertPool := x509.NewCertPool()
	if ok := rootCertPool.AppendCertsFromPEM(serverCert); ok != true {
		err := errors.New("server peer verification failed: S2Av2 could not obtain/parse roots")
		return buildValidatePeerCertificateChainSessionResp(uint32(codes.Internal), err.Error(), s2av2pb.ValidatePeerCertificateChainResp_FAILURE, err.Error(), &s2av2ctx.S2AContext{}), err
	}

	// Set the Intermediates: certs between leaf and root, excluding the leaf and root.
	intermediateCertPool := x509.NewCertPool()
	for i := 1; i < (len(derCertChain)); i++ {
		x509Cert, err := x509.ParseCertificate(derCertChain[i])
		if err != nil {
			return buildValidatePeerCertificateChainSessionResp(uint32(codes.InvalidArgument), err.Error(), s2av2pb.ValidatePeerCertificateChainResp_FAILURE, err.Error(), &s2av2ctx.S2AContext{}), err
		}
		intermediateCertPool.AddCert(x509Cert)
	}

	// Verify the leaf certificate.
	opts := x509.VerifyOptions{
		CurrentTime:   time.Now(),
		Roots:         rootCertPool,
		Intermediates: intermediateCertPool,
	}
	x509LeafCert, err := x509.ParseCertificate(derCertChain[0])
	if err != nil {
		s := fmt.Sprintf("server peer verification failed: %v", err)
		return buildValidatePeerCertificateChainSessionResp(uint32(codes.InvalidArgument), s, s2av2pb.ValidatePeerCertificateChainResp_FAILURE, s, &s2av2ctx.S2AContext{}), err
	}
	if _, err := x509LeafCert.Verify(opts); err != nil {
		s := fmt.Sprintf("server peer verification failed: %v", err)
		return buildValidatePeerCertificateChainSessionResp(uint32(codes.InvalidArgument), s, s2av2pb.ValidatePeerCertificateChainResp_FAILURE, s, &s2av2ctx.S2AContext{}), nil
	}

	return buildValidatePeerCertificateChainSessionResp(uint32(codes.OK), "", s2av2pb.ValidatePeerCertificateChainResp_SUCCESS, "server peer verification succeeded", &s2av2ctx.S2AContext{}), nil
}

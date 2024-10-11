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

package remotesigner

import (
	"bytes"
	"context"
	"crypto"
	"crypto/rand"
	"crypto/rsa"
	"crypto/sha256"
	"crypto/tls"
	"crypto/x509"
	"fmt"
	"log"
	"net"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/google/s2a-go/internal/v2/fakes2av2"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/protobuf/testing/protocmp"

	_ "embed"

	commonpb "github.com/google/s2a-go/internal/proto/v2/common_go_proto"
	s2av2pb "github.com/google/s2a-go/internal/proto/v2/s2a_go_proto"
)

const (
	defaultTimeout = 10.0 * time.Second
)

func startFakeS2Av2Server(wg *sync.WaitGroup, expToken string) (stop func(), address string, err error) {
	listener, err := net.Listen("tcp", ":0")
	if err != nil {
		log.Fatalf("Failed to listen on address %s: %v", address, err)
	}
	address = listener.Addr().String()
	s := grpc.NewServer()
	log.Printf("Server: started gRPC fake S2Av2 Server on address: %s", address)
	s2av2pb.RegisterS2AServiceServer(s, &fakes2av2.Server{ExpectedToken: expToken})
	go func() {
		wg.Done()
		if err := s.Serve(listener); err != nil {
			log.Printf("Failed to serve: %v", err)
		}
	}()
	return func() { s.Stop() }, address, nil
}

var (
	//go:embed testdata/client_cert.pem
	clientCertPEM []byte
	//go:embed testdata/client_cert.der
	clientCertDER []byte
	//go:embed testdata/client_key.pem
	clientKeyPEM []byte
	//go:embed testdata/server_cert.pem
	serverCertPEM []byte
	//go:embed testdata/server_cert.der
	serverCertDER []byte
	//go:embed testdata/server_key.pem
	serverKeyPEM []byte
)

func TestSign(t *testing.T) {
	// Start up fake S2Av2 server.
	var wg sync.WaitGroup
	wg.Add(1)
	stop, address, err := startFakeS2Av2Server(&wg, "TestSign_token")
	wg.Wait()
	if err != nil {
		t.Fatalf("Error starting fake S2Av2 Server: %v", err)
	}

	for _, tc := range []struct {
		description string
		PEMCert     []byte
		DERCert     []byte
		PEMKey      []byte
		connSide    commonpb.ConnectionSide
	}{
		{
			description: "Sign with client key",
			PEMCert:     clientCertPEM,
			DERCert:     clientCertDER,
			PEMKey:      clientKeyPEM,
			connSide:    commonpb.ConnectionSide_CONNECTION_SIDE_CLIENT,
		},
		{
			description: "Sign with server key",
			PEMCert:     serverCertPEM,
			DERCert:     serverCertDER,
			PEMKey:      serverKeyPEM,
			connSide:    commonpb.ConnectionSide_CONNECTION_SIDE_SERVER,
		},
	} {
		t.Run(tc.description, func(t *testing.T) {
			// Create stream to S2Av2.
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
							Token: "TestSign_token",
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

			// Setup data for testing Sign.
			TLSCert, err := tls.X509KeyPair(tc.PEMCert, tc.PEMKey)
			if err != nil {
				t.Fatalf("tls.X509KeyPair failed: %v", err)
			}
			x509Cert, err := x509.ParseCertificate(tc.DERCert)
			if err != nil {
				t.Fatalf("Failed to parse cert: %v", err)
			}
			testInBytes := []byte("Test data.")

			// Hash testInBytes because caller of Sign is expected to do so.
			hsha256 := sha256.Sum256([]byte(testInBytes))

			// Test RSA PKCS1v15 signature algorithm.
			s := New(x509Cert, cstream)

			gotSignedBytes, err := s.Sign(rand.Reader, hsha256[:], crypto.SHA256)
			if err != nil {
				t.Errorf("Call to remote signer Sign API failed: %v", err)
			}
			wantSignedBytes, err := TLSCert.PrivateKey.(crypto.Signer).Sign(rand.Reader, hsha256[:], crypto.SHA256)
			if err != nil {
				t.Errorf("Call to Sign API failed: %v", err)
			}
			if !bytes.Equal(gotSignedBytes, wantSignedBytes) {
				t.Errorf("gotSignedBytes = %v, wantSignedBytes = %v", gotSignedBytes, wantSignedBytes)
			}
			if err = rsa.VerifyPKCS1v15(x509Cert.PublicKey.(*rsa.PublicKey), crypto.SHA256, hsha256[:], gotSignedBytes); err != nil {
				t.Errorf("Failed to verify RSA PKCS #1 v1.5 signature: %v", err)
			}

			// Test RSA PSS signature algorithm.
			s = New(x509Cert, cstream)
			pssSignerOpts := &rsa.PSSOptions{SaltLength: rsa.PSSSaltLengthEqualsHash, Hash: crypto.SHA256}

			gotSignedBytes, err = s.Sign(rand.Reader, hsha256[:], pssSignerOpts)
			if err != nil {
				t.Errorf("Failed to generate gotSignedBytes using RSA PSS: %v", err)
			}
			if err = rsa.VerifyPSS(x509Cert.PublicKey.(*rsa.PublicKey), crypto.SHA256, hsha256[:], gotSignedBytes, pssSignerOpts); err != nil {
				t.Errorf("Failed to verify RSA PSS signature: %v", err)
			}
		})
	}
	stop()
}

// TestNew runs unit test for New.
func TestNew(t *testing.T) {
	// Setup data for testing New.
	clientx509Cert, err := x509.ParseCertificate(clientCertDER)
	if err != nil {
		t.Errorf("Failed to parse cert: %v", err)
	}
	var cstream s2av2pb.S2AService_SetUpSessionClient

	got := New(clientx509Cert, cstream)
	if v := got.(*remoteSigner).getCert(); v != clientx509Cert {
		t.Errorf("RemoteSigner leafCert field is incorrect. got: %v, want: %v", v, clientx509Cert)
	}
	if v := got.(*remoteSigner).getStream(); v != cstream {
		t.Errorf("RemoteSigner cstream field is incorrect. got: %v, want: %v", v, cstream)
	}
}

// Test GetSignatureAlgorithm runs unit test for getSignatureAlgorithm.
func TestGetSignatureAlgorithm(t *testing.T) {
	for _, tc := range []struct {
		description            string
		leafCert               *x509.Certificate
		signerOpts             crypto.SignerOpts
		wantSignatureAlgorithm s2av2pb.SignatureAlgorithm
		wantError              error
	}{
		{
			description: "Leaf certificate is nil",
			leafCert:    nil,
			signerOpts:  crypto.SHA256,
			wantError:   fmt.Errorf("unknown signature algorithm"),
		},
		{
			description: "Signer options are nil",
			leafCert:    &x509.Certificate{},
			signerOpts:  nil,
			wantError:   fmt.Errorf("unknown signature algorithm"),
		},
		{
			description:            "RSA PSS SHA256",
			leafCert:               &x509.Certificate{PublicKeyAlgorithm: x509.RSA},
			signerOpts:             &rsa.PSSOptions{SaltLength: rsa.PSSSaltLengthEqualsHash, Hash: crypto.SHA256},
			wantSignatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_RSA_PSS_RSAE_SHA256,
		},
		{
			description:            "RSA PSS SHA384",
			leafCert:               &x509.Certificate{PublicKeyAlgorithm: x509.RSA},
			signerOpts:             &rsa.PSSOptions{SaltLength: rsa.PSSSaltLengthEqualsHash, Hash: crypto.SHA384},
			wantSignatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_RSA_PSS_RSAE_SHA384,
		},
		{
			description:            "RSA PSS SHA512",
			leafCert:               &x509.Certificate{PublicKeyAlgorithm: x509.RSA},
			signerOpts:             &rsa.PSSOptions{SaltLength: rsa.PSSSaltLengthEqualsHash, Hash: crypto.SHA512},
			wantSignatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_RSA_PSS_RSAE_SHA512,
		},
		{
			description: "RSA PSS with unsupported hash",
			leafCert:    &x509.Certificate{PublicKeyAlgorithm: x509.RSA},
			signerOpts:  &rsa.PSSOptions{SaltLength: rsa.PSSSaltLengthEqualsHash, Hash: crypto.MD5},
			wantError:   fmt.Errorf("unknown signature algorithm"),
		},
		{
			description:            "RSA PKCS1 SHA256",
			leafCert:               &x509.Certificate{PublicKeyAlgorithm: x509.RSA},
			signerOpts:             crypto.SHA256,
			wantSignatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_RSA_PKCS1_SHA256,
		},
		{
			description:            "RSA PKCS1 SHA384",
			leafCert:               &x509.Certificate{PublicKeyAlgorithm: x509.RSA},
			signerOpts:             crypto.SHA384,
			wantSignatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_RSA_PKCS1_SHA384,
		},
		{
			description:            "RSA PKCS1 SHA512",
			leafCert:               &x509.Certificate{PublicKeyAlgorithm: x509.RSA},
			signerOpts:             crypto.SHA512,
			wantSignatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_RSA_PKCS1_SHA512,
		},
		{
			description: "RSA PKCS1 with unsupported hash",
			leafCert:    &x509.Certificate{PublicKeyAlgorithm: x509.RSA},
			signerOpts:  crypto.MD5,
			wantError:   fmt.Errorf("unknown signature algorithm"),
		},
		{
			description:            "ECDSA SHA256",
			leafCert:               &x509.Certificate{PublicKeyAlgorithm: x509.ECDSA},
			signerOpts:             crypto.SHA256,
			wantSignatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_ECDSA_SECP256R1_SHA256,
		},
		{
			description:            "ECDSA SHA384",
			leafCert:               &x509.Certificate{PublicKeyAlgorithm: x509.ECDSA},
			signerOpts:             crypto.SHA384,
			wantSignatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_ECDSA_SECP384R1_SHA384,
		},
		{
			description:            "ECDSA SHA512",
			leafCert:               &x509.Certificate{PublicKeyAlgorithm: x509.ECDSA},
			signerOpts:             crypto.SHA512,
			wantSignatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_ECDSA_SECP521R1_SHA512,
		},
		{
			description: "ECDSA with unsupported hash",
			leafCert:    &x509.Certificate{PublicKeyAlgorithm: x509.ECDSA},
			signerOpts:  crypto.MD5,
			wantError:   fmt.Errorf("unknown signature algorithm"),
		},
		{
			description:            "ED25519",
			leafCert:               &x509.Certificate{PublicKeyAlgorithm: x509.Ed25519},
			signerOpts:             crypto.SHA256,
			wantSignatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_ED25519,
		},
		{
			description: "DSA",
			leafCert:    &x509.Certificate{PublicKeyAlgorithm: x509.DSA},
			signerOpts:  crypto.SHA256,
			wantError:   fmt.Errorf("unknown signature algorithm: \"DSA\""),
		},
		{
			description: "Unknown public key algorithm",
			leafCert:    &x509.Certificate{PublicKeyAlgorithm: x509.UnknownPublicKeyAlgorithm},
			signerOpts:  crypto.SHA256,
			wantError:   fmt.Errorf("unknown signature algorithm: \"0\""),
		},
		{
			description: "No public key algorithm",
			leafCert:    &x509.Certificate{},
			signerOpts:  crypto.SHA256,
			wantError:   fmt.Errorf("unknown signature algorithm: \"0\""),
		},
	} {
		t.Run(tc.description, func(t *testing.T) {
			algorithm, err := getSignatureAlgorithm(tc.signerOpts, tc.leafCert)

			if tc.wantError != nil {
				if got, want := algorithm, s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_UNSPECIFIED; got != want {
					t.Errorf("Signature algorithm, got: %v, want: %v", got, want)
				}
				if !strings.Contains(tc.wantError.Error(), err.Error()) {
					t.Errorf("Unexpected error, got: %v, want: %v", err, tc.wantError)
				}
			} else {
				if got, want := algorithm, tc.wantSignatureAlgorithm; got != want {
					t.Errorf("Signature algorithm, got: %v, want: %v", got, want)
				}
				if err != nil {
					t.Errorf("Unexpected error: %v", err)
				}
			}
		})
	}
}

func TestGetSignReq(t *testing.T) {
	for _, tc := range []struct {
		description        string
		signatureAlgorithm s2av2pb.SignatureAlgorithm
		expReq             *s2av2pb.OffloadPrivateKeyOperationReq
		expErr             error
	}{
		{
			description:        "Unspecified",
			signatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_UNSPECIFIED,
			expErr:             fmt.Errorf("unknown signature algorithm: %v", s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_UNSPECIFIED),
		},
		{
			description:        "RSA PKCS1 SHA256",
			signatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_RSA_PKCS1_SHA256,
			expReq: &s2av2pb.OffloadPrivateKeyOperationReq{
				Operation:          s2av2pb.OffloadPrivateKeyOperationReq_SIGN,
				SignatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_RSA_PKCS1_SHA256,
				InBytes: &s2av2pb.OffloadPrivateKeyOperationReq_Sha256Digest{
					Sha256Digest: []byte(""),
				},
			},
		},
		{
			description:        "RSA PSS SHA256",
			signatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_RSA_PSS_RSAE_SHA256,
			expReq: &s2av2pb.OffloadPrivateKeyOperationReq{
				Operation:          s2av2pb.OffloadPrivateKeyOperationReq_SIGN,
				SignatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_RSA_PSS_RSAE_SHA256,
				InBytes: &s2av2pb.OffloadPrivateKeyOperationReq_Sha256Digest{
					Sha256Digest: []byte(""),
				},
			},
		},
		{
			description:        "ECDSA SHA256",
			signatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_ECDSA_SECP256R1_SHA256,
			expReq: &s2av2pb.OffloadPrivateKeyOperationReq{
				Operation:          s2av2pb.OffloadPrivateKeyOperationReq_SIGN,
				SignatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_ECDSA_SECP256R1_SHA256,
				InBytes: &s2av2pb.OffloadPrivateKeyOperationReq_Sha256Digest{
					Sha256Digest: []byte(""),
				},
			},
		},
		{
			description:        "RSA PKCS1 SHA384",
			signatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_RSA_PKCS1_SHA384,
			expReq: &s2av2pb.OffloadPrivateKeyOperationReq{
				Operation:          s2av2pb.OffloadPrivateKeyOperationReq_SIGN,
				SignatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_RSA_PKCS1_SHA384,
				InBytes: &s2av2pb.OffloadPrivateKeyOperationReq_Sha384Digest{
					Sha384Digest: []byte(""),
				},
			},
		},
		{
			description:        "RSA PSS SHA384",
			signatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_RSA_PSS_RSAE_SHA384,
			expReq: &s2av2pb.OffloadPrivateKeyOperationReq{
				Operation:          s2av2pb.OffloadPrivateKeyOperationReq_SIGN,
				SignatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_RSA_PSS_RSAE_SHA384,
				InBytes: &s2av2pb.OffloadPrivateKeyOperationReq_Sha384Digest{
					Sha384Digest: []byte(""),
				},
			},
		},
		{
			description:        "ECDSA SHA384",
			signatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_ECDSA_SECP384R1_SHA384,
			expReq: &s2av2pb.OffloadPrivateKeyOperationReq{
				Operation:          s2av2pb.OffloadPrivateKeyOperationReq_SIGN,
				SignatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_ECDSA_SECP384R1_SHA384,
				InBytes: &s2av2pb.OffloadPrivateKeyOperationReq_Sha384Digest{
					Sha384Digest: []byte(""),
				},
			},
		},
		{
			description:        "RSA PKCS1 SHA512",
			signatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_RSA_PKCS1_SHA512,
			expReq: &s2av2pb.OffloadPrivateKeyOperationReq{
				Operation:          s2av2pb.OffloadPrivateKeyOperationReq_SIGN,
				SignatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_RSA_PKCS1_SHA512,
				InBytes: &s2av2pb.OffloadPrivateKeyOperationReq_Sha512Digest{
					Sha512Digest: []byte(""),
				},
			},
		},
		{
			description:        "RSA PSS SHA512",
			signatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_RSA_PSS_RSAE_SHA512,
			expReq: &s2av2pb.OffloadPrivateKeyOperationReq{
				Operation:          s2av2pb.OffloadPrivateKeyOperationReq_SIGN,
				SignatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_RSA_PSS_RSAE_SHA512,
				InBytes: &s2av2pb.OffloadPrivateKeyOperationReq_Sha512Digest{
					Sha512Digest: []byte(""),
				},
			},
		},
		{
			description:        "ECDSA SHA512",
			signatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_ECDSA_SECP521R1_SHA512,
			expReq: &s2av2pb.OffloadPrivateKeyOperationReq{
				Operation:          s2av2pb.OffloadPrivateKeyOperationReq_SIGN,
				SignatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_ECDSA_SECP521R1_SHA512,
				InBytes: &s2av2pb.OffloadPrivateKeyOperationReq_Sha512Digest{
					Sha512Digest: []byte(""),
				},
			},
		},
		{
			description:        "ED25519",
			signatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_ED25519,
			expReq: &s2av2pb.OffloadPrivateKeyOperationReq{
				Operation:          s2av2pb.OffloadPrivateKeyOperationReq_SIGN,
				SignatureAlgorithm: s2av2pb.SignatureAlgorithm_S2A_SSL_SIGN_ED25519,
				InBytes: &s2av2pb.OffloadPrivateKeyOperationReq_Sha512Digest{
					Sha512Digest: []byte(""),
				},
			},
		},
	} {
		t.Run(tc.description, func(t *testing.T) {
			gotReq, gotErr := getSignReq(tc.signatureAlgorithm, []byte(""))
			if gotErr != tc.expErr {
				if (gotErr == nil) || (tc.expErr == nil) {
					t.Errorf("gotErr = %v, expErr = %v", gotErr, tc.expErr)
				}
				if gotErr.Error() != tc.expErr.Error() {
					t.Errorf("gotErr = %v, expErr = %v", gotErr, tc.expErr)
				}
			}
			if diff := cmp.Diff(tc.expReq, gotReq, protocmp.Transform()); diff != "" {
				t.Errorf("getSignReq returned incorrect OffloadPrivateKeyOperationReq, (-want +got):\n%s", diff)
			}
		})
	}
}

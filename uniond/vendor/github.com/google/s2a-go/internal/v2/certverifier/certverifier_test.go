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

package certverifier

import (
	"context"
	"errors"
	"fmt"
	"log"
	"net"
	"sync"
	"testing"
	"time"

	_ "embed"

	"github.com/google/s2a-go/internal/v2/fakes2av2"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	s2av2pb "github.com/google/s2a-go/internal/proto/v2/s2a_go_proto"
)

const (
	defaultTimeout                = 10.0 * time.Second
	fakeServerAuthorizationPolicy = "fake server authorization policy"
)

var (
	//go:embed testdata/client_root_cert.der
	clientRootDERCert []byte
	//go:embed testdata/client_intermediate_cert.der
	clientIntermediateDERCert []byte
	//go:embed testdata/client_leaf_cert.der
	clientLeafDERCert []byte
	//go:embed testdata/server_root_cert.der
	serverRootDERCert []byte
	//go:embed testdata/server_intermediate_cert.der
	serverIntermediateDERCert []byte
	//go:embed testdata/server_leaf_cert.der
	serverLeafDERCert []byte
)

func startFakeS2Av2Server(wg *sync.WaitGroup, enableServerAuthorizationPolicyCheck bool) (stop func(), address string, err error) {
	listener, err := net.Listen("tcp", ":0")
	if err != nil {
		log.Fatalf("Failed to listen on address %s: %v", address, err)
	}
	address = listener.Addr().String()
	s := grpc.NewServer()
	log.Printf("Server: started gRPC fake S2Av2 Server on address: %s", address)
	if enableServerAuthorizationPolicyCheck {
		s2av2pb.RegisterS2AServiceServer(s, &fakes2av2.Server{
			ServerAuthorizationPolicy: []byte(fakeServerAuthorizationPolicy),
		})
	} else {
		s2av2pb.RegisterS2AServiceServer(s, &fakes2av2.Server{})
	}
	go func() {
		wg.Done()
		if err := s.Serve(listener); err != nil {
			log.Printf("Failed to serve: %v", err)
		}
	}()
	return func() { s.Stop() }, address, nil
}

// TestVerifyClientCertChain runs unit tests for VerifyClientCertificateChain.
func TestVerifyClientCertChain(t *testing.T) {
	// Start up fake S2Av2 server.
	var wg sync.WaitGroup
	wg.Add(1)
	stop, address, err := startFakeS2Av2Server(&wg, false)
	wg.Wait()
	if err != nil {
		t.Fatalf("Error starting fake S2Av2 Server: %v", err)
	}

	for _, tc := range []struct {
		description string
		rawCerts    [][]byte
		expectedErr error
	}{
		{
			description: "empty chain",
			rawCerts:    nil,
			expectedErr: errors.New("client cert verification failed: client peer verification failed: client cert chain is empty"),
		},
		{
			description: "chain of length 1",
			rawCerts:    [][]byte{clientRootDERCert},
			expectedErr: nil,
		},
		{
			description: "chain of length 2 correct",
			rawCerts:    [][]byte{clientLeafDERCert, clientIntermediateDERCert},
			expectedErr: nil,
		},
		{
			description: "chain of length 2 error: missing intermediate",
			rawCerts:    [][]byte{clientLeafDERCert, clientRootDERCert},
			expectedErr: errors.New("failed to offload client cert verification to S2A: 3, client peer verification failed: x509: certificate signed by unknown authority (possibly because of \"crypto/rsa: verification error\" while trying to verify candidate authority certificate \"s2a_test_cert\")"),
		},
	} {
		t.Run(tc.description, func(t *testing.T) {
			// Create new stream to S2Av2.
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

			// TODO(rmehta19): Add verificationMode to struct, and vary between tests.
			VerifyPeerCertificateFunc := VerifyClientCertificateChain(s2av2pb.ValidatePeerCertificateChainReq_CONNECT_TO_GOOGLE, cstream)
			got, want := VerifyPeerCertificateFunc(tc.rawCerts, nil), tc.expectedErr
			if want == nil {
				if got != nil {
					t.Errorf("Peer certificate verification failed, got: %v, want: %v", got, want)
				}
			} else {
				if got == nil {
					t.Errorf("Peer certificate verification failed, got: %v, want: %v", got, want)
				}
				if got.Error() != want.Error() {
					t.Errorf("Peer certificate verification failed, got: %v, want: %v", got, want)
				}
			}
		})
	}
	stop()
}

// TestVerifyServerCertChainWithServerAuthorizationPolicy runs unit tests for VerifyServerCertificateChain with server authorization policy check.
func TestVerifyServerCertChainWithServerAuthorizationPolicy(t *testing.T) {
	// Start up fake S2Av2 server.
	var wg sync.WaitGroup
	wg.Add(1)
	stop, address, err := startFakeS2Av2Server(&wg, true)
	wg.Wait()
	if err != nil {
		t.Fatalf("Error starting fake S2Av2 Server: %v", err)
	}

	for _, tc := range []struct {
		description               string
		hostname                  string
		rawCerts                  [][]byte
		expectedErr               error
		serverAuthorizationPolicy []byte
	}{
		{
			description:               "empty chain",
			hostname:                  "host",
			rawCerts:                  nil,
			expectedErr:               errors.New("server cert verification failed: server peer verification failed: server cert chain is empty"),
			serverAuthorizationPolicy: []byte(fakeServerAuthorizationPolicy),
		},
		{
			description:               "invalid server authorization policy",
			hostname:                  "host",
			rawCerts:                  [][]byte{serverRootDERCert},
			expectedErr:               fmt.Errorf("rpc error: code = Unknown desc = server peer verification failed: invalid server authorization policy, expected: %s, got: ", fakeServerAuthorizationPolicy),
			serverAuthorizationPolicy: nil,
		},
		{
			description:               "chain of length 1",
			hostname:                  "host",
			rawCerts:                  [][]byte{serverRootDERCert},
			expectedErr:               nil,
			serverAuthorizationPolicy: []byte(fakeServerAuthorizationPolicy),
		},
		{
			description:               "chain of length 2 correct",
			hostname:                  "host",
			rawCerts:                  [][]byte{serverLeafDERCert, serverIntermediateDERCert},
			expectedErr:               nil,
			serverAuthorizationPolicy: []byte(fakeServerAuthorizationPolicy),
		},
	} {
		t.Run(tc.description, func(t *testing.T) {
			// Create new stream to S2Av2.
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

			// TODO(rmehta19): Add verificationMode to struct, and vary between tests.
			VerifyPeerCertificateFunc := VerifyServerCertificateChain(tc.hostname, s2av2pb.ValidatePeerCertificateChainReq_CONNECT_TO_GOOGLE, cstream, tc.serverAuthorizationPolicy)
			got, want := VerifyPeerCertificateFunc(tc.rawCerts, nil), tc.expectedErr
			if want == nil {
				if got != nil {
					t.Errorf("Peer certificate verification failed, got: %v, want: %v", got, want)
				}
			} else {
				if got == nil {
					t.Errorf("Peer certificate verification failed, got: %v, want: %v", got, want)
				}
				if got.Error() != want.Error() {
					t.Errorf("Peer certificate verification failed, got: %v, want: %v", got, want)
				}
			}
		})
	}
	stop()
}

// TestVerifyServerCertChainWithoutServerAuthorizationPolicy runs unit tests for VerifyServerCertificateChain without server authorization policy check.
func TestVerifyServerCertChainWithoutServerAuthorizationPolicy(t *testing.T) {
	// Start up fake S2Av2 server.
	var wg sync.WaitGroup
	wg.Add(1)
	stop, address, err := startFakeS2Av2Server(&wg, false)
	wg.Wait()
	if err != nil {
		t.Fatalf("Error starting fake S2Av2 Server: %v", err)
	}

	for _, tc := range []struct {
		description               string
		hostname                  string
		rawCerts                  [][]byte
		expectedErr               error
		serverAuthorizationPolicy []byte
	}{
		{
			description:               "empty chain",
			hostname:                  "host",
			rawCerts:                  nil,
			expectedErr:               errors.New("server cert verification failed: server peer verification failed: server cert chain is empty"),
			serverAuthorizationPolicy: []byte(fakeServerAuthorizationPolicy),
		},
		{
			description:               "chain of length 1",
			hostname:                  "host",
			rawCerts:                  [][]byte{serverRootDERCert},
			expectedErr:               nil,
			serverAuthorizationPolicy: nil,
		},
		{
			description:               "chain of length 2 correct",
			hostname:                  "host",
			rawCerts:                  [][]byte{serverLeafDERCert, serverIntermediateDERCert},
			expectedErr:               nil,
			serverAuthorizationPolicy: nil,
		},
	} {
		t.Run(tc.description, func(t *testing.T) {
			// Create new stream to S2Av2.
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

			VerifyPeerCertificateFunc := VerifyServerCertificateChain(tc.hostname, s2av2pb.ValidatePeerCertificateChainReq_CONNECT_TO_GOOGLE, cstream, tc.serverAuthorizationPolicy)
			got, want := VerifyPeerCertificateFunc(tc.rawCerts, nil), tc.expectedErr
			if want == nil {
				if got != nil {
					t.Errorf("Peer certificate verification failed, got: %v, want: %v", got, want)
				}
			} else {
				if got == nil {
					t.Errorf("Peer certificate verification failed, got: %v, want: %v", got, want)
				}
				if got.Error() != want.Error() {
					t.Errorf("Peer certificate verification failed, got: %v, want: %v", got, want)
				}
			}
		})
	}
	stop()
}

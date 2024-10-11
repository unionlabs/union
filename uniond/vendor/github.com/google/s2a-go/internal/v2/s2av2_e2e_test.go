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

package v2

import (
	"bytes"
	"context"
	"crypto/tls"
	"fmt"
	"io/ioutil"
	"net"
	"os"
	"path/filepath"
	"testing"
	"time"

	_ "embed"

	"github.com/google/s2a-go/fallback"
	"github.com/google/s2a-go/internal/tokenmanager"
	"github.com/google/s2a-go/internal/v2/fakes2av2"
	"github.com/google/s2a-go/retry"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/grpclog"

	grpc "google.golang.org/grpc"

	commonpbv1 "github.com/google/s2a-go/internal/proto/common_go_proto"
	helloworldpb "github.com/google/s2a-go/internal/proto/examples/helloworld_go_proto"
	s2av2pb "github.com/google/s2a-go/internal/proto/v2/s2a_go_proto"
)

const (
	accessTokenEnvVariable = "S2A_ACCESS_TOKEN"
	defaultE2ETimeout      = time.Second * 5
	clientMessage          = "echo"
)

var (
	//go:embed testdata/client_cert.pem
	clientCertpem []byte
	//go:embed testdata/client_key.pem
	clientKeypem []byte
	//go:embed testdata/server_cert.pem
	serverCertpem []byte
	//go:embed testdata/server_key.pem
	serverKeypem []byte
)

// server implements the helloworld.GreeterServer.
type server struct {
	helloworldpb.UnimplementedGreeterServer
}

// SayHello implements helloworld.GreeterServer.
func (s *server) SayHello(_ context.Context, in *helloworldpb.HelloRequest) (*helloworldpb.HelloReply, error) {
	return &helloworldpb.HelloReply{Message: "Hello " + in.GetName()}, nil
}

// startFakeS2A starts up a fake S2A and returns the address that it is
// listening on.
func startFakeS2A(t *testing.T, expToken string) string {
	lis, err := net.Listen("tcp", ":")
	if err != nil {
		t.Errorf("net.Listen(tcp, :0) failed: %v", err)
	}
	s := grpc.NewServer()
	s2av2pb.RegisterS2AServiceServer(s, &fakes2av2.Server{ExpectedToken: expToken})
	go func() {
		if err := s.Serve(lis); err != nil {
			t.Errorf("s.Serve(%v) failed: %v", lis, err)
		}
	}()
	return lis.Addr().String()
}

// startFakeS2A starts up a fake S2A on UDS and returns the address that it is
// listening on.
func startFakeS2AOnUDS(t *testing.T, expToken string) string {
	dir, err := ioutil.TempDir("/tmp", "socket_dir")
	if err != nil {
		t.Errorf("Unable to create temporary directory: %v", err)
	}
	udsAddress := filepath.Join(dir, "socket")
	lis, err := net.Listen("unix", filepath.Join(dir, "socket"))
	if err != nil {
		t.Errorf("net.Listen(unix, %s) failed: %v", udsAddress, err)
	}
	s := grpc.NewServer()
	s2av2pb.RegisterS2AServiceServer(s, &fakes2av2.Server{ExpectedToken: expToken})
	go func() {
		if err := s.Serve(lis); err != nil {
			t.Errorf("s.Serve(%v) failed: %v", lis, err)
		}
	}()
	return fmt.Sprintf("unix://%s", lis.Addr().String())
}

// startServer starts up a server and returns the address that it is listening
// on.
func startServer(t *testing.T, s2aAddress string, localIdentities []*commonpbv1.Identity) string {
	// TODO(rmehta19): Pass verificationMode as a parameter to startServer.
	creds, err := NewServerCreds(s2aAddress, nil, localIdentities, s2av2pb.ValidatePeerCertificateChainReq_CONNECT_TO_GOOGLE, nil)
	if err != nil {
		t.Errorf("NewServerCreds(%s) failed: %v", s2aAddress, err)
	}

	lis, err := net.Listen("tcp", ":0")
	if err != nil {
		t.Errorf("net.Listen(tcp, :0) failed: %v", err)
	}
	s := grpc.NewServer(grpc.Creds(creds))
	helloworldpb.RegisterGreeterServer(s, &server{})
	go func() {
		if err := s.Serve(lis); err != nil {
			t.Errorf("s.Serve(%v) failed: %v", lis, err)
		}
	}()
	return lis.Addr().String()
}

// startFallbackServer runs a GRPC echo testing server and returns the address.
// It's used to test the default fallback logic upon S2A failure.
func startFallbackServer(t *testing.T) string {
	lis, err := net.Listen("tcp", ":0")
	if err != nil {
		t.Errorf("net.Listen(tcp, :0) failed: %v", err)
	}
	cert, err := tls.X509KeyPair(serverCertpem, serverKeypem)
	if err != nil {
		t.Errorf("failure initializing tls.certificate: %v", err)
	}
	// Client certs are not required for the fallback server.
	creds := credentials.NewTLS(&tls.Config{
		MinVersion:   tls.VersionTLS13,
		MaxVersion:   tls.VersionTLS13,
		Certificates: []tls.Certificate{cert},
	})
	s := grpc.NewServer(grpc.Creds(creds))
	helloworldpb.RegisterGreeterServer(s, &server{})
	go func() {
		if err := s.Serve(lis); err != nil {
			t.Errorf("s.Serve(%v) failed: %v", lis, err)
		}
	}()
	return lis.Addr().String()
}

// runClient starts up a client and calls the server.
func runClient(ctx context.Context, t *testing.T, clientS2AAddress, serverAddr string, localIdentity *commonpbv1.Identity, fallbackHandshake fallback.ClientHandshake) {
	creds, err := NewClientCreds(clientS2AAddress, nil, localIdentity, s2av2pb.ValidatePeerCertificateChainReq_CONNECT_TO_GOOGLE, fallbackHandshake, nil, nil)
	if err != nil {
		t.Errorf("NewClientCreds(%s) failed: %v", clientS2AAddress, err)
	}
	dialOptions := []grpc.DialOption{
		grpc.WithTransportCredentials(creds),
		grpc.WithBlock(),
	}

	grpclog.Info("Client dialing server at address: %v", serverAddr)
	// Establish a connection to the server.
	conn, err := grpc.Dial(serverAddr, dialOptions...)
	if err != nil {
		t.Errorf("grpc.Dial(%v, %v) failed: %v", serverAddr, dialOptions, err)
	}
	defer conn.Close()

	// Contact the server.
	c := helloworldpb.NewGreeterClient(conn)
	req := &helloworldpb.HelloRequest{Name: clientMessage}
	grpclog.Infof("Client calling SayHello with request: %v", req)
	resp, err := c.SayHello(ctx, req, grpc.WaitForReady(true))
	if err != nil {
		t.Errorf("c.SayHello(%v, %v) failed: %v", ctx, req, err)
	}
	if got, want := resp.GetMessage(), "Hello "+clientMessage; got != want {
		t.Errorf("r.GetMessage() = %v, want %v", got, want)
	}
	grpclog.Infof("Client received message from server: %s", resp.GetMessage())
}

func TestEndToEndUsingFakeS2AOverTCP(t *testing.T) {
	os.Setenv(accessTokenEnvVariable, "TestE2ETCP_token")
	oldRetry := retry.NewRetryer
	defer func() { retry.NewRetryer = oldRetry }()
	testRetryer := retry.NewRetryer()
	retry.NewRetryer = func() *retry.S2ARetryer {
		return testRetryer
	}
	// Start the fake S2As for the client and server.
	serverS2AAddr := startFakeS2A(t, "TestE2ETCP_token")
	grpclog.Infof("Fake handshaker for server running at address: %v", serverS2AAddr)
	clientS2AAddr := startFakeS2A(t, "TestE2ETCP_token")
	grpclog.Infof("Fake handshaker for client running at address: %v", clientS2AAddr)

	// Start the server.
	localIdentities := []*commonpbv1.Identity{
		{
			IdentityOneof: &commonpbv1.Identity_Hostname{
				Hostname: "test_rsa_server_identity",
			},
		},
	}
	serverAddr := startServer(t, serverS2AAddr, localIdentities)
	grpclog.Infof("Server running at address: %v", serverAddr)

	// Finally, start up the client.
	ctx, cancel := context.WithTimeout(context.Background(), defaultE2ETimeout)
	defer cancel()
	runClient(ctx, t, clientS2AAddr, serverAddr, &commonpbv1.Identity{
		IdentityOneof: &commonpbv1.Identity_Hostname{
			Hostname: "test_rsa_client_identity",
		},
	}, nil)
	if got, want := testRetryer.Attempts(), 0; got != want {
		t.Errorf("expecting retryer attempts count:[%v], got [%v]", want, got)
	}
}

func TestEndToEndUsingFakeS2AOverTCPEmptyId(t *testing.T) {
	os.Setenv(accessTokenEnvVariable, "TestE2ETCP_token")
	// Start the fake S2As for the client and server.
	serverS2AAddr := startFakeS2A(t, "TestE2ETCP_token")
	grpclog.Infof("Fake handshaker for server running at address: %v", serverS2AAddr)
	clientS2AAddr := startFakeS2A(t, "TestE2ETCP_token")
	grpclog.Infof("Fake handshaker for client running at address: %v", clientS2AAddr)

	// Start the server.
	var localIdentities []*commonpbv1.Identity
	localIdentities = append(localIdentities, nil)
	serverAddr := startServer(t, serverS2AAddr, localIdentities)
	grpclog.Infof("Server running at address: %v", serverAddr)

	// Finally, start up the client.
	ctx, cancel := context.WithTimeout(context.Background(), defaultE2ETimeout)
	defer cancel()
	runClient(ctx, t, clientS2AAddr, serverAddr, nil, nil)
}

func TestEndToEndUsingFakeS2AOnUDS(t *testing.T) {
	os.Setenv(accessTokenEnvVariable, "TestE2EUDS_token")
	// Start fake S2As for use by the client and server.
	serverS2AAddr := startFakeS2AOnUDS(t, "TestE2EUDS_token")
	grpclog.Infof("Fake S2A for server listening on UDS at address: %v", serverS2AAddr)
	clientS2AAddr := startFakeS2AOnUDS(t, "TestE2EUDS_token")
	grpclog.Infof("Fake S2A for client listening on UDS at address: %v", clientS2AAddr)

	// Start the server.
	localIdentities := []*commonpbv1.Identity{
		{
			IdentityOneof: &commonpbv1.Identity_Hostname{
				Hostname: "test_rsa_server_identity",
			},
		},
	}
	serverAddr := startServer(t, serverS2AAddr, localIdentities)
	grpclog.Infof("Server running at address: %v", serverAddr)

	// Finally, start up the client.
	ctx, cancel := context.WithTimeout(context.Background(), defaultE2ETimeout)
	defer cancel()
	runClient(ctx, t, clientS2AAddr, serverAddr, &commonpbv1.Identity{
		IdentityOneof: &commonpbv1.Identity_Hostname{
			Hostname: "test_rsa_client_identity",
		},
	}, nil)
}

func TestEndToEndUsingFakeS2AOnUDSEmptyId(t *testing.T) {
	os.Setenv(accessTokenEnvVariable, "TestE2EUDS_token")
	// Start fake S2As for use by the client and server.
	serverS2AAddr := startFakeS2AOnUDS(t, "TestE2EUDS_token")
	grpclog.Infof("Fake S2A for server listening on UDS at address: %v", serverS2AAddr)
	clientS2AAddr := startFakeS2AOnUDS(t, "TestE2EUDS_token")
	grpclog.Infof("Fake S2A for client listening on UDS at address: %v", clientS2AAddr)

	// Start the server.
	var localIdentities []*commonpbv1.Identity
	localIdentities = append(localIdentities, nil)
	serverAddr := startServer(t, serverS2AAddr, localIdentities)
	grpclog.Infof("Server running at address: %v", serverAddr)

	// Finally, start up the client.
	ctx, cancel := context.WithTimeout(context.Background(), defaultE2ETimeout)
	defer cancel()
	runClient(ctx, t, clientS2AAddr, serverAddr, nil, nil)
}

func TestGRPCFallbackEndToEndUsingFakeS2AOverTCP(t *testing.T) {
	// Set for testing only.
	fallback.FallbackTLSConfigGRPC.InsecureSkipVerify = true
	os.Setenv(accessTokenEnvVariable, "TestE2ETCP_token")
	oldRetry := retry.NewRetryer
	defer func() { retry.NewRetryer = oldRetry }()
	testRetryer := retry.NewRetryer()
	retry.NewRetryer = func() *retry.S2ARetryer {
		return testRetryer
	}

	// Start the fake S2A for the server.
	serverS2AAddr := startFakeS2A(t, "TestE2ETCP_token")
	t.Logf("Fake handshaker for server running at address: %v", serverS2AAddr)

	// Start the server.
	localIdentities := []*commonpbv1.Identity{
		{
			IdentityOneof: &commonpbv1.Identity_Hostname{
				Hostname: "test_rsa_server_identity",
			},
		},
	}
	serverAddr := startServer(t, serverS2AAddr, localIdentities)
	fallbackServerAddr := startFallbackServer(t)
	t.Logf("server running at address: %v", serverAddr)
	t.Logf("fallback server running at address: %v", fallbackServerAddr)

	// Finally, start up the client.
	ctx, cancel := context.WithTimeout(context.Background(), defaultE2ETimeout)
	defer cancel()
	fallbackHandshake, err := fallback.DefaultFallbackClientHandshakeFunc(fallbackServerAddr)
	if err != nil {
		t.Errorf("error creating fallback handshake function: %v", err)
	}
	fallbackCalled := false
	fallbackHandshakeWrapper := func(ctx context.Context, targetServer string, conn net.Conn, err error) (net.Conn, credentials.AuthInfo, error) {
		fallbackCalled = true
		return fallbackHandshake(ctx, targetServer, conn, err)
	}
	// Set wrong S2A address for client to trigger S2A failure and fallback.
	runClient(ctx, t, "not_exist", serverAddr, &commonpbv1.Identity{
		IdentityOneof: &commonpbv1.Identity_Hostname{
			Hostname: "test_rsa_client_identity",
		},
	}, fallbackHandshakeWrapper)

	if !fallbackCalled {
		t.Errorf("fallbackHandshake is not called")
	}
	if got, want := testRetryer.Attempts(), 5; got != want {
		t.Errorf("expecting retryer attempts count:[%v], got [%v]", want, got)
	}
}

func TestGRPCRetryAndFallbackEndToEndUsingFakeS2AOverTCP(t *testing.T) {
	// Set for testing only.
	fallback.FallbackTLSConfigGRPC.InsecureSkipVerify = true
	// Set an invalid token to trigger failures and retries when talking to S2A.
	os.Setenv(accessTokenEnvVariable, "invalid_token")
	oldRetry := retry.NewRetryer
	defer func() { retry.NewRetryer = oldRetry }()
	testRetryer := retry.NewRetryer()
	retry.NewRetryer = func() *retry.S2ARetryer {
		return testRetryer
	}

	clientS2AAddr := startFakeS2A(t, "TestE2ETCP_token")
	grpclog.Infof("Fake handshaker for client running at address: %v", clientS2AAddr)
	serverS2AAddr := startFakeS2A(t, "TestE2ETCP_token")
	grpclog.Infof("Fake handshaker for server running at address: %v", serverS2AAddr)

	// Start the server.
	localIdentities := []*commonpbv1.Identity{
		{
			IdentityOneof: &commonpbv1.Identity_Hostname{
				Hostname: "test_rsa_server_identity",
			},
		},
	}
	serverAddr := startServer(t, serverS2AAddr, localIdentities)
	fallbackServerAddr := startFallbackServer(t)
	t.Logf("server running at address: %v", serverAddr)
	t.Logf("fallback server running at address: %v", fallbackServerAddr)

	// Finally, start up the client.
	ctx, cancel := context.WithTimeout(context.Background(), defaultE2ETimeout)
	defer cancel()
	fallbackHandshake, err := fallback.DefaultFallbackClientHandshakeFunc(fallbackServerAddr)
	if err != nil {
		t.Errorf("error creating fallback handshake function: %v", err)
	}
	fallbackCalled := false
	fallbackHandshakeWrapper := func(ctx context.Context, targetServer string, conn net.Conn, err error) (net.Conn, credentials.AuthInfo, error) {
		fallbackCalled = true
		return fallbackHandshake(ctx, targetServer, conn, err)
	}
	runClient(ctx, t, clientS2AAddr, serverAddr, &commonpbv1.Identity{
		IdentityOneof: &commonpbv1.Identity_Hostname{
			Hostname: "test_rsa_client_identity",
		},
	}, fallbackHandshakeWrapper)

	if !fallbackCalled {
		t.Errorf("fallbackHandshake is not called")
	}
	if got, want := testRetryer.Attempts(), 5; got != want {
		t.Errorf("expecting retryer attempts count:[%v], got [%v]", want, got)
	}
}

func TestNewClientTlsConfigWithTokenManager(t *testing.T) {
	os.Setenv(accessTokenEnvVariable, "TestNewClientTlsConfig_token")
	s2AAddr := startFakeS2A(t, "TestNewClientTlsConfig_token")
	accessTokenManager, err := tokenmanager.NewSingleTokenAccessTokenManager()
	if err != nil {
		t.Errorf("tokenmanager.NewSingleTokenAccessTokenManager() failed: %v", err)
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultE2ETimeout)
	defer cancel()
	config, err := NewClientTLSConfig(ctx, s2AAddr, nil, accessTokenManager, s2av2pb.ValidatePeerCertificateChainReq_CONNECT_TO_GOOGLE, "test_server_name", nil)
	if err != nil {
		t.Errorf("NewClientTLSConfig() failed: %v", err)
	}

	cert, err := tls.X509KeyPair(clientCertpem, clientKeypem)
	if err != nil {
		t.Fatalf("tls.X509KeyPair failed: %v", err)
	}
	if got, want := config.Certificates[0].Certificate[0], cert.Certificate[0]; !bytes.Equal(got, want) {
		t.Errorf("tls.Config has unexpected certificate: got: %v, want: %v", got, want)
	}
}

func TestNewClientTlsConfigWithoutTokenManager(t *testing.T) {
	os.Unsetenv(accessTokenEnvVariable)
	s2AAddr := startFakeS2A(t, "ignored-value")
	var tokenManager tokenmanager.AccessTokenManager
	ctx, cancel := context.WithTimeout(context.Background(), defaultE2ETimeout)
	defer cancel()
	config, err := NewClientTLSConfig(ctx, s2AAddr, nil, tokenManager, s2av2pb.ValidatePeerCertificateChainReq_CONNECT_TO_GOOGLE, "test_server_name", nil)
	if err != nil {
		t.Errorf("NewClientTLSConfig() failed: %v", err)
	}

	cert, err := tls.X509KeyPair(clientCertpem, clientKeypem)
	if err != nil {
		t.Fatalf("tls.X509KeyPair failed: %v", err)
	}
	if got, want := config.Certificates[0].Certificate[0], cert.Certificate[0]; !bytes.Equal(got, want) {
		t.Errorf("tls.Config has unexpected certificate: got: %v, want: %v", got, want)
	}
}

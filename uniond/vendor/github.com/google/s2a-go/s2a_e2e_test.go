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

package s2a

import (
	"bytes"
	"context"
	"crypto/tls"
	"crypto/x509"
	"fmt"
	"io"
	"io/ioutil"
	"net"
	"net/http"
	"os"
	"path/filepath"
	"testing"
	"time"

	_ "embed"

	"github.com/google/s2a-go/fallback"
	"github.com/google/s2a-go/internal/fakehandshaker/service"
	"github.com/google/s2a-go/internal/v2/fakes2av2"
	"github.com/google/s2a-go/retry"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/grpclog"
	"google.golang.org/grpc/peer"

	grpc "google.golang.org/grpc"

	commonpb "github.com/google/s2a-go/internal/proto/common_go_proto"
	helloworldpb "github.com/google/s2a-go/internal/proto/examples/helloworld_go_proto"
	s2apb "github.com/google/s2a-go/internal/proto/s2a_go_proto"
	s2av2pb "github.com/google/s2a-go/internal/proto/v2/s2a_go_proto"
)

const (
	accessTokenEnvVariable = "S2A_ACCESS_TOKEN"
	testAccessToken        = "test_access_token"
	testV2AccessToken      = "valid_token"

	applicationProtocol   = "grpc"
	authType              = "s2a"
	clientHostname        = "test_client_hostname"
	serverSpiffeID        = "test_server_spiffe_id"
	clientMessage         = "echo"
	defaultE2ETestTimeout = time.Second * 5
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

	//go:embed testdata/mds_root_cert.pem
	mdsRootCertPem []byte
	//go:embed testdata/mds_server_cert.pem
	mdsServerCertPem []byte
	//go:embed testdata/mds_server_key.pem
	mdsServerKeyPem []byte
	//go:embed testdata/mds_client_cert.pem
	mdsClientCertPem []byte
	//go:embed testdata/mds_client_key.pem
	mdsClientKeyPem []byte
	//go:embed testdata/self_signed_cert.pem
	selfSignedCertPem []byte
	//go:embed testdata/self_signed_key.pem
	selfSignedKeyPem []byte
)

// server is used to implement helloworld.GreeterServer.
type server struct {
	helloworldpb.UnimplementedGreeterServer
}

// SayHello implements helloworld.GreeterServer.
func (s *server) SayHello(_ context.Context, in *helloworldpb.HelloRequest) (*helloworldpb.HelloReply, error) {
	return &helloworldpb.HelloReply{Message: "Hello " + in.GetName()}, nil
}

// startFakeS2A starts up a fake S2A and returns the address that it is
// listening on.
func startFakeS2A(t *testing.T, enableLegacyMode bool, expToken string, serverTransportCreds credentials.TransportCredentials) string {
	lis, err := net.Listen("tcp", ":")
	if err != nil {
		t.Errorf("net.Listen(tcp, :0) failed: %v", err)
	}

	var s *grpc.Server
	if serverTransportCreds != nil {
		s = grpc.NewServer(grpc.Creds(serverTransportCreds))
	} else {
		s = grpc.NewServer()
	}

	if enableLegacyMode {
		s2apb.RegisterS2AServiceServer(s, &service.FakeHandshakerService{})
	} else {
		s2av2pb.RegisterS2AServiceServer(s, &fakes2av2.Server{ExpectedToken: expToken})
	}
	go func() {
		if err := s.Serve(lis); err != nil {
			t.Errorf("s.Serve(%v) failed: %v", lis, err)
		}
	}()
	return lis.Addr().String()
}

// startFakeS2AOnUDS starts up a fake S2A on UDS and returns the address that
// it is listening on.
func startFakeS2AOnUDS(t *testing.T, enableLegacyMode bool, expToken string) string {
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
	if enableLegacyMode {
		s2apb.RegisterS2AServiceServer(s, &service.FakeHandshakerService{})
	} else {
		s2av2pb.RegisterS2AServiceServer(s, &fakes2av2.Server{ExpectedToken: expToken})
	}
	go func() {
		if err := s.Serve(lis); err != nil {
			t.Errorf("s.Serve(%v) failed: %v", lis, err)
		}
	}()
	return fmt.Sprintf("unix://%s", lis.Addr().String())
}

// startServer starts up a server and returns the address that it is listening
// on.
func startServer(t *testing.T, s2aAddress string, transportCreds credentials.TransportCredentials, enableLegacyMode bool) string {
	serverOpts := &ServerOptions{
		LocalIdentities:  []Identity{NewSpiffeID(serverSpiffeID)},
		S2AAddress:       s2aAddress,
		TransportCreds:   transportCreds,
		EnableLegacyMode: enableLegacyMode,
	}
	creds, err := NewServerCreds(serverOpts)
	if err != nil {
		t.Errorf("NewServerCreds(%v) failed: %v", serverOpts, err)
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

// runClient starts up a client and calls the server.
func runClient(ctx context.Context, t *testing.T, clientS2AAddress string, transportCreds credentials.TransportCredentials, serverAddr string, enableLegacyMode bool, fallbackHandshake fallback.ClientHandshake) {
	clientOpts := &ClientOptions{
		TargetIdentities: []Identity{NewSpiffeID(serverSpiffeID)},
		LocalIdentity:    NewHostname(clientHostname),
		S2AAddress:       clientS2AAddress,
		TransportCreds:   transportCreds,
		EnableLegacyMode: enableLegacyMode,
		FallbackOpts: &FallbackOptions{
			FallbackClientHandshakeFunc: fallbackHandshake,
		},
	}
	creds, err := NewClientCreds(clientOpts)
	if err != nil {
		t.Errorf("NewClientCreds(%v) failed: %v", clientOpts, err)
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
	peer := new(peer.Peer)
	c := helloworldpb.NewGreeterClient(conn)
	req := &helloworldpb.HelloRequest{Name: clientMessage}
	grpclog.Infof("Client calling SayHello with request: %v", req)
	resp, err := c.SayHello(ctx, req, grpc.Peer(peer), grpc.WaitForReady(true))
	if err != nil {
		t.Errorf("c.SayHello(%v, %v) failed: %v", ctx, req, err)
	}
	if got, want := resp.GetMessage(), "Hello "+clientMessage; got != want {
		t.Errorf("r.GetMessage() = %v, want %v", got, want)
	}
	grpclog.Infof("Client received message from server: %s", resp.GetMessage())

	if enableLegacyMode {
		// Check the auth info.
		authInfo, err := AuthInfoFromPeer(peer)
		if err != nil {
			t.Errorf("AuthInfoFromContext(peer) failed: %v", err)
		}
		s2aAuthInfo, ok := authInfo.(AuthInfo)
		if !ok {
			t.Errorf("authInfo is not an s2a.AuthInfo")
		}
		if got, want := s2aAuthInfo.AuthType(), authType; got != want {
			t.Errorf("s2aAuthInfo.AuthType() = %v, want %v", got, want)
		}
		if got, want := s2aAuthInfo.ApplicationProtocol(), applicationProtocol; got != want {
			t.Errorf("s2aAuthInfo.ApplicationProtocol() = %v, want %v", got, want)
		}
		if got, want := s2aAuthInfo.TLSVersion(), commonpb.TLSVersion_TLS1_3; got != want {
			t.Errorf("s2aAuthInfo.TLSVersion() = %v, want %v", got, want)
		}
		if got, want := s2aAuthInfo.IsHandshakeResumed(), false; got != want {
			t.Errorf("s2aAuthInfo.IsHandshakeResumed() = %v, want %v", got, want)
		}
		if got, want := s2aAuthInfo.SecurityLevel(), credentials.PrivacyAndIntegrity; got != want {
			t.Errorf("s2aAuthInfo.SecurityLevel() = %v, want %v", got, want)
		}
	}
}

func TestV1EndToEndUsingFakeS2AOverTCP(t *testing.T) {
	os.Setenv(accessTokenEnvVariable, "")

	// Start the fake S2As for the client and server.
	serverHandshakerAddr := startFakeS2A(t, true, "", nil)
	grpclog.Infof("Fake handshaker for server running at address: %v", serverHandshakerAddr)
	clientHandshakerAddr := startFakeS2A(t, true, "", nil)
	grpclog.Infof("Fake handshaker for client running at address: %v", clientHandshakerAddr)

	// Start the server.
	serverAddr := startServer(t, serverHandshakerAddr, nil, true)
	grpclog.Infof("Server running at address: %v", serverAddr)

	// Finally, start up the client.
	ctx, cancel := context.WithTimeout(context.Background(), defaultE2ETestTimeout)
	defer cancel()
	runClient(ctx, t, clientHandshakerAddr, nil, serverAddr, true, nil)
}

func TestV2EndToEndUsingFakeS2AOverTCP(t *testing.T) {
	os.Setenv(accessTokenEnvVariable, testV2AccessToken)
	oldRetry := retry.NewRetryer
	defer func() { retry.NewRetryer = oldRetry }()
	testRetryer := retry.NewRetryer()
	retry.NewRetryer = func() *retry.S2ARetryer {
		return testRetryer
	}
	// Start the fake S2As for the client and server.
	serverHandshakerAddr := startFakeS2A(t, false, testV2AccessToken, nil)
	grpclog.Infof("Fake handshaker for server running at address: %v", serverHandshakerAddr)
	clientHandshakerAddr := startFakeS2A(t, false, testV2AccessToken, nil)
	grpclog.Infof("Fake handshaker for client running at address: %v", clientHandshakerAddr)

	// Start the server.
	serverAddr := startServer(t, serverHandshakerAddr, nil, false)
	grpclog.Infof("Server running at address: %v", serverAddr)

	// Finally, start up the client.
	ctx, cancel := context.WithTimeout(context.Background(), defaultE2ETestTimeout)
	defer cancel()
	runClient(ctx, t, clientHandshakerAddr, nil, serverAddr, false, nil)
	if got, want := testRetryer.Attempts(), 0; got != want {
		t.Errorf("expecting retryer attempts count:[%v], got [%v]", want, got)
	}
}

func TestV2EndToEndUsingFakeMTLSS2AOverTCP(t *testing.T) {
	os.Setenv(accessTokenEnvVariable, "")
	oldRetry := retry.NewRetryer
	defer func() { retry.NewRetryer = oldRetry }()
	testRetryer := retry.NewRetryer()
	retry.NewRetryer = func() *retry.S2ARetryer {
		return testRetryer
	}
	serverTransportCreds := loadServerTransportCreds(t, mdsServerCertPem, mdsServerKeyPem)
	// Start the fake S2As for the client and server.
	serverHandshakerAddr := startFakeS2A(t, false, "", serverTransportCreds)
	grpclog.Infof("Fake handshaker for server running at address: %v", serverHandshakerAddr)
	clientHandshakerAddr := startFakeS2A(t, false, "", serverTransportCreds)
	grpclog.Infof("Fake handshaker for client running at address: %v", clientHandshakerAddr)

	clientTransportCreds := loadClientTransportCreds(t, mdsClientCertPem, mdsClientKeyPem)
	// Start the server.
	serverAddr := startServer(t, serverHandshakerAddr, clientTransportCreds, false)
	grpclog.Infof("Server running at address: %v", serverAddr)

	// Finally, start up the client.
	ctx, cancel := context.WithTimeout(context.Background(), defaultE2ETestTimeout)
	defer cancel()
	runClient(ctx, t, clientHandshakerAddr, clientTransportCreds, serverAddr, false, nil)
	if got, want := testRetryer.Attempts(), 0; got != want {
		t.Errorf("expecting retryer attempts count:[%v], got [%v]", want, got)
	}
}

func TestV2EndToEndUsingFakeMTLSS2AOverTCP_SelfSignedClientTransportCreds(t *testing.T) {
	os.Setenv(accessTokenEnvVariable, "")
	fallback.FallbackTLSConfigGRPC.InsecureSkipVerify = true
	oldRetry := retry.NewRetryer
	defer func() { retry.NewRetryer = oldRetry }()
	testRetryer := retry.NewRetryer()
	retry.NewRetryer = func() *retry.S2ARetryer {
		return testRetryer
	}
	serverTransportCreds := loadServerTransportCreds(t, mdsServerCertPem, mdsServerKeyPem)
	// Start the fake S2As for the client and server.
	serverHandshakerAddr := startFakeS2A(t, false, "", serverTransportCreds)
	grpclog.Infof("Fake handshaker for server running at address: %v", serverHandshakerAddr)
	clientHandshakerAddr := startFakeS2A(t, false, "", serverTransportCreds)
	grpclog.Infof("Fake handshaker for client running at address: %v", clientHandshakerAddr)

	clientTransportCreds := loadClientTransportCreds(t, mdsClientCertPem, mdsClientKeyPem)
	// Load self-signed client credentials.
	selfSignedClientTransportCreds := loadClientTransportCreds(t, selfSignedCertPem, selfSignedKeyPem)
	// Start the server.
	serverAddr := startServer(t, serverHandshakerAddr, clientTransportCreds, false)
	fallbackServerAddr := startFallbackServer(t)
	t.Logf("server running at address: %v", serverAddr)
	t.Logf("fallback server running at address: %v", fallbackServerAddr)

	// Finally, start up the client.
	ctx, cancel := context.WithTimeout(context.Background(), defaultE2ETestTimeout)
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

	// Use self-signed cert to trigger handshake failure when connecting to MTLS-S2A gRPC server.
	// This should cause retries and eventually fallback.
	runClient(ctx, t, clientHandshakerAddr, selfSignedClientTransportCreds, serverAddr, false, fallbackHandshakeWrapper)
	if !fallbackCalled {
		t.Errorf("fallbackHandshake is not called")
	}
	if got, want := testRetryer.Attempts(), 5; got != want {
		t.Errorf("expecting retryer attempts count:[%v], got [%v]", want, got)
	}
}

func loadServerTransportCreds(t *testing.T, cert, key []byte) credentials.TransportCredentials {
	certificate, err := tls.X509KeyPair(cert, key)
	if err != nil {
		t.Errorf("failed to load S2A server cert/key: %v", err)
	}
	caPool := x509.NewCertPool()
	if !caPool.AppendCertsFromPEM(mdsRootCertPem) {
		t.Errorf("failed to add ca cert")
	}
	tlsConfig := &tls.Config{
		ClientAuth:   tls.RequireAndVerifyClientCert,
		Certificates: []tls.Certificate{certificate},
		ClientCAs:    caPool,
	}
	return credentials.NewTLS(tlsConfig)
}

func loadClientTransportCreds(t *testing.T, cert, key []byte) credentials.TransportCredentials {
	certificate, err := tls.X509KeyPair(cert, key)
	if err != nil {
		t.Errorf("failed to load S2A client cert/key: %v", err)
	}
	caPool := x509.NewCertPool()
	if !caPool.AppendCertsFromPEM(mdsRootCertPem) {
		t.Errorf("failed to add ca cert")
	}
	tlsConfig := &tls.Config{
		Certificates: []tls.Certificate{certificate},
		RootCAs:      caPool,
	}
	return credentials.NewTLS(tlsConfig)
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
func TestV2GRPCFallbackEndToEndUsingFakeS2AOverTCP(t *testing.T) {
	// Set for testing only.
	fallback.FallbackTLSConfigGRPC.InsecureSkipVerify = true
	os.Setenv(accessTokenEnvVariable, testV2AccessToken)
	oldRetry := retry.NewRetryer
	defer func() { retry.NewRetryer = oldRetry }()
	testRetryer := retry.NewRetryer()
	retry.NewRetryer = func() *retry.S2ARetryer {
		return testRetryer
	}
	// Start the fake S2A for the server.
	serverHandshakerAddr := startFakeS2A(t, false, testV2AccessToken, nil)
	grpclog.Infof("fake handshaker for server running at address: %v", serverHandshakerAddr)

	// Start the server.
	serverAddr := startServer(t, serverHandshakerAddr, nil, false)
	fallbackServerAddr := startFallbackServer(t)
	t.Logf("server running at address: %v", serverAddr)
	t.Logf("fallback server running at address: %v", fallbackServerAddr)

	// Finally, start up the client.
	ctx, cancel := context.WithTimeout(context.Background(), defaultE2ETestTimeout)
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
	runClient(ctx, t, "not_exist", nil, serverAddr, false, fallbackHandshakeWrapper)
	if !fallbackCalled {
		t.Errorf("fallbackHandshake is not called")
	}
	if got, want := testRetryer.Attempts(), 5; got != want {
		t.Errorf("expecting retryer attempts count:[%v], got [%v]", want, got)
	}
}

func TestV2GRPCRetryAndFallbackEndToEndUsingFakeS2AOverTCP(t *testing.T) {
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
	// Start the fake S2A for the server and client.
	serverHandshakerAddr := startFakeS2A(t, false, testV2AccessToken, nil)
	grpclog.Infof("fake handshaker for server running at address: %v", serverHandshakerAddr)
	clientHandshakerAddr := startFakeS2A(t, false, testV2AccessToken, nil)
	grpclog.Infof("Fake handshaker for client running at address: %v", clientHandshakerAddr)

	// Start the server.
	serverAddr := startServer(t, serverHandshakerAddr, nil, false)
	fallbackServerAddr := startFallbackServer(t)
	t.Logf("server running at address: %v", serverAddr)
	t.Logf("fallback server running at address: %v", fallbackServerAddr)

	// Finally, start up the client.
	ctx, cancel := context.WithTimeout(context.Background(), defaultE2ETestTimeout)
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
	runClient(ctx, t, clientHandshakerAddr, nil, serverAddr, false, fallbackHandshakeWrapper)
	if !fallbackCalled {
		t.Errorf("fallbackHandshake is not called")
	}
	if got, want := testRetryer.Attempts(), 5; got != want {
		t.Errorf("expecting retryer attempts count:[%v], got [%v]", want, got)
	}
}

func TestV1EndToEndUsingTokens(t *testing.T) {
	os.Setenv(accessTokenEnvVariable, testAccessToken)

	// Start the handshaker servers for the client and server.
	serverS2AAddress := startFakeS2A(t, true, "", nil)
	grpclog.Infof("Fake S2A for server running at address: %v", serverS2AAddress)
	clientS2AAddress := startFakeS2A(t, true, "", nil)
	grpclog.Infof("Fake S2A for client running at address: %v", clientS2AAddress)

	// Start the server.
	serverAddr := startServer(t, serverS2AAddress, nil, true)
	grpclog.Infof("Server running at address: %v", serverAddr)

	// Finally, start up the client.
	ctx, cancel := context.WithTimeout(context.Background(), defaultE2ETestTimeout)
	defer cancel()
	runClient(ctx, t, clientS2AAddress, nil, serverAddr, true, nil)
}

func TestV2EndToEndUsingTokens(t *testing.T) {
	os.Setenv(accessTokenEnvVariable, testV2AccessToken)

	// Start the handshaker servers for the client and server.
	serverS2AAddress := startFakeS2A(t, false, testV2AccessToken, nil)
	grpclog.Infof("Fake S2A for server running at address: %v", serverS2AAddress)
	clientS2AAddress := startFakeS2A(t, false, testV2AccessToken, nil)
	grpclog.Infof("Fake S2A for client running at address: %v", clientS2AAddress)

	// Start the server.
	serverAddr := startServer(t, serverS2AAddress, nil, false)
	grpclog.Infof("Server running at address: %v", serverAddr)

	// Finally, start up the client.
	ctx, cancel := context.WithTimeout(context.Background(), defaultE2ETestTimeout)
	defer cancel()
	runClient(ctx, t, clientS2AAddress, nil, serverAddr, false, nil)
}

func TestV2EndToEndEmptyToken(t *testing.T) {
	os.Unsetenv(accessTokenEnvVariable)

	// Start the handshaker servers for the client and server.
	serverS2AAddress := startFakeS2A(t, false, testV2AccessToken, nil)
	grpclog.Infof("Fake S2A for server running at address: %v", serverS2AAddress)
	clientS2AAddress := startFakeS2A(t, false, testV2AccessToken, nil)
	grpclog.Infof("Fake S2A for client running at address: %v", clientS2AAddress)

	// Start the server.
	serverAddr := startServer(t, serverS2AAddress, nil, false)
	grpclog.Infof("Server running at address: %v", serverAddr)

	// Finally, start up the client.
	ctx, cancel := context.WithTimeout(context.Background(), defaultE2ETestTimeout)
	defer cancel()
	runClient(ctx, t, clientS2AAddress, nil, serverAddr, false, nil)
}

func TestV1EndToEndUsingFakeS2AOnUDS(t *testing.T) {
	os.Setenv(accessTokenEnvVariable, "")

	// Start fake S2As for use by the client and server.
	serverS2AAddress := startFakeS2AOnUDS(t, true, "")
	grpclog.Infof("Fake S2A for server listening on UDS at address: %v", serverS2AAddress)
	clientS2AAddress := startFakeS2AOnUDS(t, true, "")
	grpclog.Infof("Fake S2A for client listening on UDS at address: %v", clientS2AAddress)

	// Start the server.
	serverAddress := startServer(t, serverS2AAddress, nil, true)
	grpclog.Infof("Server running at address: %v", serverS2AAddress)

	// Finally, start up the client.
	ctx, cancel := context.WithTimeout(context.Background(), defaultE2ETestTimeout)
	defer cancel()
	runClient(ctx, t, clientS2AAddress, nil, serverAddress, true, nil)
}

func TestV2EndToEndUsingFakeS2AOnUDS(t *testing.T) {
	os.Setenv(accessTokenEnvVariable, testV2AccessToken)

	// Start fake S2As for use by the client and server.
	serverS2AAddress := startFakeS2AOnUDS(t, false, testV2AccessToken)
	grpclog.Infof("Fake S2A for server listening on UDS at address: %v", serverS2AAddress)
	clientS2AAddress := startFakeS2AOnUDS(t, false, testV2AccessToken)
	grpclog.Infof("Fake S2A for client listening on UDS at address: %v", clientS2AAddress)

	// Start the server.
	serverAddress := startServer(t, serverS2AAddress, nil, false)
	grpclog.Infof("Server running at address: %v", serverS2AAddress)

	// Finally, start up the client.
	ctx, cancel := context.WithTimeout(context.Background(), defaultE2ETestTimeout)
	defer cancel()
	runClient(ctx, t, clientS2AAddress, nil, serverAddress, false, nil)
}

func TestNewTLSClientConfigFactoryWithTokenManager(t *testing.T) {
	os.Setenv(accessTokenEnvVariable, "TestNewTLSClientConfigFactory_token")
	s2AAddr := startFakeS2A(t, false, "TestNewTLSClientConfigFactory_token", nil)
	ctx, cancel := context.WithTimeout(context.Background(), defaultE2ETestTimeout)
	defer cancel()

	factory, err := NewTLSClientConfigFactory(&ClientOptions{
		S2AAddress: s2AAddr,
	})
	if err != nil {
		t.Errorf("NewTLSClientConfigFactory() failed: %v", err)
	}

	config, err := factory.Build(ctx, nil)
	if err != nil {
		t.Errorf("Build tls config failed: %v", err)
	}

	cert, err := tls.X509KeyPair(clientCertpem, clientKeypem)
	if err != nil {
		t.Fatalf("tls.X509KeyPair failed: %v", err)
	}

	if got, want := config.Certificates[0].Certificate[0], cert.Certificate[0]; !bytes.Equal(got, want) {
		t.Errorf("tls.Config has unexpected certificate: got: %v, want: %v", got, want)
	}
}

func TestNewTLSClientConfigFactoryWithoutTokenManager(t *testing.T) {
	os.Unsetenv(accessTokenEnvVariable)
	s2AAddr := startFakeS2A(t, false, "ignored-value", nil)
	ctx, cancel := context.WithTimeout(context.Background(), defaultE2ETestTimeout)
	defer cancel()

	factory, err := NewTLSClientConfigFactory(&ClientOptions{
		S2AAddress: s2AAddr,
	})
	if err != nil {
		t.Errorf("NewTLSClientConfigFactory() failed: %v", err)
	}

	config, err := factory.Build(ctx, nil)
	if err != nil {
		t.Errorf("Build tls config failed: %v", err)
	}

	cert, err := tls.X509KeyPair(clientCertpem, clientKeypem)
	if err != nil {
		t.Fatalf("tls.X509KeyPair failed: %v", err)
	}
	if got, want := config.Certificates[0].Certificate[0], cert.Certificate[0]; !bytes.Equal(got, want) {
		t.Errorf("tls.Config has unexpected certificate: got: %v, want: %v", got, want)
	}
}

// startHTTPServer runs an HTTP server on a random local port and serves a /hello endpoint.
// The response of the /hello endpoint should be passed in via the `resp` parameter.
// It returns the address of the server.
func startHTTPServer(t *testing.T, resp string) string {
	cert, _ := tls.X509KeyPair(serverCertpem, serverKeypem)
	tlsConfig := tls.Config{
		MinVersion:   tls.VersionTLS13,
		MaxVersion:   tls.VersionTLS13,
		Certificates: []tls.Certificate{cert},
	}
	s := http.NewServeMux()
	s.HandleFunc("/hello", func(w http.ResponseWriter, req *http.Request) {
		fmt.Fprintf(w, resp)
	})
	lis, err := tls.Listen("tcp", ":0", &tlsConfig)
	if err != nil {
		t.Errorf("net.Listen(tcp, :0) failed: %v", err)
	}
	go func() {
		http.Serve(lis, s)
	}()
	return lis.Addr().String()
}

// runHTTPClient starts an HTTP client and talks to an HTTP server using S2A.
// It returns the response from the /hello endpoint.
func runHTTPClient(t *testing.T, clientS2AAddress string, transportCreds credentials.TransportCredentials, serverAddr string, fallbackOpts *FallbackOptions) string {
	dialTLSContext := NewS2ADialTLSContextFunc(&ClientOptions{
		S2AAddress:     clientS2AAddress,
		TransportCreds: transportCreds,
		FallbackOpts:   fallbackOpts,
	})

	tr := http.Transport{
		DialTLSContext: dialTLSContext,
	}

	client := &http.Client{Transport: &tr}
	reqURL := fmt.Sprintf("https://%s/hello", serverAddr)
	t.Logf("reqURL is set to: %v", reqURL)
	req, err := http.NewRequest(http.MethodGet, reqURL, nil)
	if err != nil {
		t.Errorf("error creating new HTTP request: %v", err)
	}
	resp, err := client.Do(req)
	if err != nil {
		t.Errorf("error making client HTTP request: %v", err)
	}
	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		t.Errorf("error reading HTTP response: %v", err)
	}
	return string(respBody)
}
func TestHTTPEndToEndUsingFakeS2AOverTCP(t *testing.T) {
	os.Setenv(accessTokenEnvVariable, testV2AccessToken)
	oldRetry := retry.NewRetryer
	defer func() { retry.NewRetryer = oldRetry }()
	testRetryer := retry.NewRetryer()
	retry.NewRetryer = func() *retry.S2ARetryer {
		return testRetryer
	}

	// Start the fake S2As for the client.
	clientHandshakerAddr := startFakeS2A(t, false, testV2AccessToken, nil)
	t.Logf("fake handshaker for client running at address: %v", clientHandshakerAddr)

	// Start the server.
	serverAddr := startHTTPServer(t, "hello")
	t.Logf("HTTP server running at address: %v", serverAddr)

	// Finally, start up the client.
	resp := runHTTPClient(t, clientHandshakerAddr, nil, serverAddr, nil)

	if got, want := resp, "hello"; got != want {
		t.Errorf("expecting HTTP response:[%s], got [%s]", want, got)
	}
	if got, want := testRetryer.Attempts(), 0; got != want {
		t.Errorf("expecting retryer attempts count:[%v], got [%v]", want, got)
	}
}

func TestHTTPEndToEndSUsingFakeMTLSS2AOverTCP(t *testing.T) {
	os.Setenv(accessTokenEnvVariable, "")
	oldRetry := retry.NewRetryer
	defer func() { retry.NewRetryer = oldRetry }()
	testRetryer := retry.NewRetryer()
	retry.NewRetryer = func() *retry.S2ARetryer {
		return testRetryer
	}

	// Start the fake S2As for the client.
	serverTransportCreds := loadServerTransportCreds(t, mdsServerCertPem, mdsServerKeyPem)
	clientHandshakerAddr := startFakeS2A(t, false, "", serverTransportCreds)
	t.Logf("fake handshaker for client running at address: %v", clientHandshakerAddr)

	// Start the server.
	serverAddr := startHTTPServer(t, "hello")
	t.Logf("HTTP server running at address: %v", serverAddr)

	// Finally, start up the client.
	clientTransportCreds := loadClientTransportCreds(t, mdsClientCertPem, mdsClientKeyPem)
	resp := runHTTPClient(t, clientHandshakerAddr, clientTransportCreds, serverAddr, nil)

	if got, want := resp, "hello"; got != want {
		t.Errorf("expecting HTTP response:[%s], got [%s]", want, got)
	}
	if got, want := testRetryer.Attempts(), 0; got != want {
		t.Errorf("expecting retryer attempts count:[%v], got [%v]", want, got)
	}
}

func TestHTTPEndToEndSUsingFakeMTLSS2AOverTCP_SelfSignedClientTransportCreds(t *testing.T) {
	fallback.FallbackTLSConfigHTTP.InsecureSkipVerify = true
	os.Setenv(accessTokenEnvVariable, "")
	oldRetry := retry.NewRetryer
	defer func() { retry.NewRetryer = oldRetry }()
	testRetryer := retry.NewRetryer()
	retry.NewRetryer = func() *retry.S2ARetryer {
		return testRetryer
	}

	// Start the fake S2As for the client.
	serverTransportCreds := loadServerTransportCreds(t, mdsServerCertPem, mdsServerKeyPem)
	clientHandshakerAddr := startFakeS2A(t, false, "", serverTransportCreds)
	t.Logf("fake handshaker for client running at address: %v", clientHandshakerAddr)

	serverAddr := startHTTPServer(t, "hello")
	t.Logf("HTTP server running at address: %v", serverAddr)

	fallbackServerAddr := startHTTPServer(t, "hello fallback")
	t.Logf("fallback HTTP server running at address: %v", fallbackServerAddr)

	// Configure fallback options.
	fbDialer, fbAddr, err := fallback.DefaultFallbackDialerAndAddress(fallbackServerAddr)
	if err != nil {
		t.Errorf("error creating fallback dialer: %v", err)
	}
	fallbackOpts := &FallbackOptions{
		FallbackDialer: &FallbackDialer{
			Dialer:     fbDialer,
			ServerAddr: fbAddr,
		},
	}
	// Load self-signed client credentials.
	selfSignedClientTransportCreds := loadClientTransportCreds(t, selfSignedCertPem, selfSignedKeyPem)
	// Use self-signed cert to trigger handshake failure when connecting to MTLS-S2A gRPC server.
	// This should cause retries and eventually fallback.
	resp := runHTTPClient(t, clientHandshakerAddr, selfSignedClientTransportCreds, serverAddr, fallbackOpts)
	if got, want := resp, "hello fallback"; got != want {
		t.Errorf("expecting HTTP response:[%s], got [%s]", want, got)
	}

	if got, want := testRetryer.Attempts(), 5; got != want {
		t.Errorf("expecting retryer attempts count:[%v], got [%v]", want, got)
	}
}

func TestHTTPFallbackEndToEndUsingFakeS2AOverTCP(t *testing.T) {
	fallback.FallbackTLSConfigHTTP.InsecureSkipVerify = true
	os.Setenv(accessTokenEnvVariable, testV2AccessToken)
	oldRetry := retry.NewRetryer
	defer func() { retry.NewRetryer = oldRetry }()
	testRetryer := retry.NewRetryer()
	retry.NewRetryer = func() *retry.S2ARetryer {
		return testRetryer
	}

	// Start the server.
	serverAddr := startHTTPServer(t, "hello")
	t.Logf("HTTP server running at address: %v", serverAddr)

	fallbackServerAddr := startHTTPServer(t, "hello fallback")
	t.Logf("fallback HTTP server running at address: %v", fallbackServerAddr)

	// Configure fallback options.
	fbDialer, fbAddr, err := fallback.DefaultFallbackDialerAndAddress(fallbackServerAddr)
	if err != nil {
		t.Errorf("error creating fallback dialer: %v", err)
	}

	fallbackOpts := &FallbackOptions{
		FallbackDialer: &FallbackDialer{
			Dialer:     fbDialer,
			ServerAddr: fbAddr,
		},
	}
	// Set wrong client S2A address to trigger S2A failure and fallback.
	resp := runHTTPClient(t, "not_exist", nil, serverAddr, fallbackOpts)

	if got, want := resp, "hello fallback"; got != want {
		t.Errorf("expecting HTTP response:[%s], got [%s]", want, got)
	}

	if got, want := testRetryer.Attempts(), 5; got != want {
		t.Errorf("expecting retryer attempts count:[%v], got [%v]", want, got)
	}
}

func TestHTTPRetryAndFallbackEndToEndUsingFakeS2AOverTCP(t *testing.T) {
	fallback.FallbackTLSConfigHTTP.InsecureSkipVerify = true
	// Set an invalid token to trigger failures and retries when talking to S2A.
	os.Setenv(accessTokenEnvVariable, "invalid_token")
	oldRetry := retry.NewRetryer
	defer func() { retry.NewRetryer = oldRetry }()
	testRetryer := retry.NewRetryer()
	retry.NewRetryer = func() *retry.S2ARetryer {
		return testRetryer
	}

	// Start the fake S2As for the client.
	clientHandshakerAddr := startFakeS2A(t, false, testV2AccessToken, nil)
	t.Logf("fake handshaker for client running at address: %v", clientHandshakerAddr)

	serverAddr := startHTTPServer(t, "hello")
	t.Logf("HTTP server running at address: %v", serverAddr)

	fallbackServerAddr := startHTTPServer(t, "hello fallback")
	t.Logf("fallback HTTP server running at address: %v", fallbackServerAddr)

	// Configure fallback options.
	fbDialer, fbAddr, err := fallback.DefaultFallbackDialerAndAddress(fallbackServerAddr)
	if err != nil {
		t.Errorf("error creating fallback dialer: %v", err)
	}

	fallbackOpts := &FallbackOptions{
		FallbackDialer: &FallbackDialer{
			Dialer:     fbDialer,
			ServerAddr: fbAddr,
		},
	}

	resp := runHTTPClient(t, clientHandshakerAddr, nil, serverAddr, fallbackOpts)

	if got, want := resp, "hello fallback"; got != want {
		t.Errorf("expecting HTTP response:[%s], got [%s]", want, got)
	}

	if got, want := testRetryer.Attempts(), 5; got != want {
		t.Errorf("expecting retryer attempts count:[%v], got [%v]", want, got)
	}
}

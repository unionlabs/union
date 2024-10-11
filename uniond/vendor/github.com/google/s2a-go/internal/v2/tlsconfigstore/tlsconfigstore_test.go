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

package tlsconfigstore

import (
	"bytes"
	"context"
	"crypto/tls"
	"errors"
	"fmt"
	"log"
	"net"
	"sync"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"github.com/google/s2a-go/internal/tokenmanager"
	"github.com/google/s2a-go/internal/v2/fakes2av2"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/protobuf/testing/protocmp"

	_ "embed"

	commonpbv1 "github.com/google/s2a-go/internal/proto/common_go_proto"
	commonpb "github.com/google/s2a-go/internal/proto/v2/common_go_proto"
	s2av2pb "github.com/google/s2a-go/internal/proto/v2/s2a_go_proto"
)

const (
	defaultTimeout = 10.0 * time.Second
)

var (
	//go:embed testdata/client_cert.pem
	clientCertpem []byte
	//go:embed testdata/server_cert.pem
	serverCertpem []byte
	//go:embed testdata/client_key.pem
	clientKeypem []byte
	//go:embed testdata/server_key.pem
	serverKeypem []byte
)

// fakeAccessTokenManager implements the AccessTokenManager interface.
type fakeAccessTokenManager struct {
	acceptedIdentity   *commonpbv1.Identity
	accessToken        string
	allowEmptyIdentity bool
}

// DefaultToken returns the token managed by the fakeAccessTokenManager.
func (m *fakeAccessTokenManager) DefaultToken() (string, error) {
	if !m.allowEmptyIdentity {
		return "", fmt.Errorf("not allowed to get token for empty identity")
	}
	return m.accessToken, nil
}

// Token returns the token managed by the fakeAccessTokenManager.
func (m *fakeAccessTokenManager) Token(identity *commonpbv1.Identity) (string, error) {
	if identity == nil || cmp.Equal(identity, &commonpbv1.Identity{}, protocmp.Transform()) {
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

func startFakeS2Av2Server(wg *sync.WaitGroup, expToken string, shouldNotReturnClientCreds bool) (stop func(), address string, err error) {
	listener, err := net.Listen("tcp", ":0")
	if err != nil {
		log.Fatalf("Failed to listen on address %s: %v", address, err)
	}
	address = listener.Addr().String()
	s := grpc.NewServer()
	log.Printf("Server: started gRPC fake S2Av2 Server on address: %s", address)
	s2av2pb.RegisterS2AServiceServer(s, &fakes2av2.Server{
		ExpectedToken:                    expToken,
		ShouldNotReturnClientCredentials: shouldNotReturnClientCreds,
	})
	go func() {
		wg.Done()
		if err := s.Serve(listener); err != nil {
			log.Printf("Failed to serve: %v", err)
		}
	}()
	return func() { s.Stop() }, address, nil
}

// TestTLSConfigStoreClient runs unit tests for GetTLSConfigurationForClient.
func TestTLSConfigStoreClient(t *testing.T) {
	// Setup for static client test.
	cert, err := tls.X509KeyPair(clientCertpem, clientKeypem)
	if err != nil {
		t.Fatalf("tls.X509KeyPair failed: %v", err)
	}

	// Start up fake S2Av2 server.
	var wg sync.WaitGroup
	wg.Add(1)
	stop, address, err := startFakeS2Av2Server(&wg, "TestTlsConfigStoreClient_token", false)
	wg.Wait()
	if err != nil {
		t.Fatalf("Error starting fake S2Av2 Server: %v", err)
	}

	accessTokenManager := &fakeAccessTokenManager{
		allowEmptyIdentity: true,
		accessToken:        "TestTlsConfigStoreClient_token",
	}
	for _, tc := range []struct {
		description            string
		tokenManager           tokenmanager.AccessTokenManager
		Certificates           []tls.Certificate
		ServerName             string
		InsecureSkipVerify     bool
		SessionTicketsDisabled bool
		ClientSessionCache     tls.ClientSessionCache
		MinVersion             uint16
		MaxVersion             uint16
		NextProtos             []string
	}{
		{
			description:            "static - nil tokenManager",
			tokenManager:           nil,
			Certificates:           []tls.Certificate{cert},
			ServerName:             "host",
			InsecureSkipVerify:     true, // NOLINT
			SessionTicketsDisabled: true,
			ClientSessionCache:     nil,
			MinVersion:             tls.VersionTLS13,
			MaxVersion:             tls.VersionTLS13,
			NextProtos:             []string{"h2"},
		},
		{
			description:            "static - non-nil tokenManager",
			tokenManager:           accessTokenManager,
			Certificates:           []tls.Certificate{cert},
			ServerName:             "host",
			InsecureSkipVerify:     true, // NOLINT
			SessionTicketsDisabled: true,
			ClientSessionCache:     nil,
			MinVersion:             tls.VersionTLS13,
			MaxVersion:             tls.VersionTLS13,
			NextProtos:             []string{"h2"},
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
			config, err := GetTLSConfigurationForClient(tc.ServerName, cstream, tc.tokenManager, nil, s2av2pb.ValidatePeerCertificateChainReq_CONNECT_TO_GOOGLE, nil)
			if err != nil {
				t.Errorf("GetTLSConfigurationForClient failed: %v", err)
			}
			if got, want := config.Certificates[0].Certificate[0], tc.Certificates[0].Certificate[0]; !bytes.Equal(got, want) {
				t.Errorf("config.Certificates[0].Certificate[0] = %v, want %v", got, want)
			}
			if got, want := config.InsecureSkipVerify, tc.InsecureSkipVerify; got != want {
				t.Errorf("config.InsecureSkipVerify = %v, want %v", got, want)
			}
			if got, want := config.SessionTicketsDisabled, tc.SessionTicketsDisabled; got != want {
				t.Errorf("config.SessionTicketsDisabled = %v, want %v", got, want)
			}
			if got, want := config.ClientSessionCache, tc.ClientSessionCache; got != want {
				t.Errorf("config.ClientSessionCache = %v, want %v", got, want)
			}
			if got, want := config.MinVersion, tc.MinVersion; got != want {
				t.Errorf("config.MinVersion = %v, want %v", got, want)
			}
			if got, want := config.MaxVersion, tc.MaxVersion; got != want {
				t.Errorf("config.MaxVersion = %v, want %v", got, want)
			}
			if !compareNextProtos(config.NextProtos, tc.NextProtos) {
				t.Errorf("config.NextProtos = %v, want %v", config.NextProtos, tc.NextProtos)
			}
		})
	}
	stop()
}

// TestTLSConfigStoreClientWithoutCredentials runs unit tests for
// GetTLSConfigurationForClient, but does not expect the TLS configuration
// to contain any client credentials.
func TestTLSConfigStoreClientWithoutCredentials(t *testing.T) {
	// Start up fake S2Av2 server.
	var wg sync.WaitGroup
	wg.Add(1)
	stop, address, err := startFakeS2Av2Server(&wg, "", true)
	wg.Wait()
	if err != nil {
		t.Fatalf("Error starting fake S2Av2 Server: %v", err)
	}

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
	config, err := GetTLSConfigurationForClient("hostname", cstream, nil, nil, s2av2pb.ValidatePeerCertificateChainReq_CONNECT_TO_GOOGLE, nil)
	if err != nil {
		t.Errorf("GetTLSConfigurationForClient failed: %v", err)
	}
	if len(config.Certificates) > 0 {
		t.Errorf("config had unexpected number of certificates")
	}
	if got, want := config.InsecureSkipVerify, true; got != want {
		t.Errorf("config.InsecureSkipVerify = %v, want %v", got, want)
	}
	if got, want := config.SessionTicketsDisabled, true; got != want {
		t.Errorf("config.SessionTicketsDisabled = %v, want %v", got, want)
	}
	if config.ClientSessionCache != nil {
		t.Errorf("config.ClientSessionCache expected to be nil")
	}
	if got, want := int(config.MinVersion), tls.VersionTLS13; got != want {
		t.Errorf("config.MinVersion = %v, want %v", got, want)
	}
	if got, want := int(config.MaxVersion), tls.VersionTLS13; got != want {
		t.Errorf("config.MaxVersion = %v, want %v", got, want)
	}
	if !compareNextProtos(config.NextProtos, []string{"h2"}) {
		t.Errorf("config.NextProtos = %v, want %v", config.NextProtos, []string{"h2"})
	}
	stop()
}

// TestTLSConfigStoreServer runs unit tests for GetTLSConfigurationForServer.
func TestTLSConfigStoreServer(t *testing.T) {
	// Setup for static server test.
	cert, err := tls.X509KeyPair(serverCertpem, serverKeypem)
	if err != nil {
		t.Fatalf("tls.X509KeyPair failed: %v", err)
	}
	// Start up fake S2Av2 server.
	var wg sync.WaitGroup
	wg.Add(1)
	stop, address, err := startFakeS2Av2Server(&wg, "TestTlsConfigStoreServer_token", false)
	wg.Wait()
	if err != nil {
		t.Fatalf("Error starting fake S2Av2 Server: %v", err)
	}

	accessTokenManager := &fakeAccessTokenManager{
		allowEmptyIdentity: true,
		accessToken:        "TestTlsConfigStoreServer_token",
	}
	var identities []*commonpbv1.Identity
	identities = append(identities, nil)
	for _, tc := range []struct {
		description            string
		tokenManager           tokenmanager.AccessTokenManager
		Certificates           []tls.Certificate
		SessionTicketsDisabled bool
		ClientAuth             tls.ClientAuthType
		MinVersion             uint16
		MaxVersion             uint16
		NextProtos             []string
	}{
		{
			description:            "static - nil tokenManager",
			tokenManager:           nil,
			Certificates:           []tls.Certificate{cert},
			SessionTicketsDisabled: true,
			ClientAuth:             tls.RequireAnyClientCert,
			MinVersion:             tls.VersionTLS13,
			MaxVersion:             tls.VersionTLS13,
			NextProtos:             []string{"h2"},
		},
		{
			description:            "static - non-nil tokenManager",
			tokenManager:           accessTokenManager,
			Certificates:           []tls.Certificate{cert},
			SessionTicketsDisabled: true,
			ClientAuth:             tls.RequireAnyClientCert,
			MinVersion:             tls.VersionTLS13,
			MaxVersion:             tls.VersionTLS13,
			NextProtos:             []string{"h2"},
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
			config, err := GetTLSConfigurationForServer(cstream, tc.tokenManager, identities, s2av2pb.ValidatePeerCertificateChainReq_CONNECT_TO_GOOGLE)
			if err != nil {
				t.Errorf("GetTLSConfigurationForClient failed: %v", err)
			}
			clientConfigFunc := config.GetConfigForClient
			config, err = clientConfigFunc(&tls.ClientHelloInfo{
				ServerName: "host_1",
			})
			if err != nil {
				t.Errorf("ClientConfig failed: %v", err)
			}
			if got, want := config.Certificates[0].Certificate[0], tc.Certificates[0].Certificate[0]; !bytes.Equal(got, want) {
				t.Errorf("config.Certificates[0].Certificate[0] = %v, want %v", got, want)
			}
			if got, want := config.SessionTicketsDisabled, tc.SessionTicketsDisabled; got != want {
				t.Errorf("config.SessionTicketsDisabled = %v, want %v", got, want)
			}
			if got, want := config.ClientAuth, tc.ClientAuth; got != want {
				t.Errorf("config.ClientAuth = %v, want %v", got, want)
			}
			if got, want := config.MinVersion, tc.MinVersion; got != want {
				t.Errorf("config.MinVersion = %v, want %v", got, want)
			}
			if got, want := config.MaxVersion, tc.MaxVersion; got != want {
				t.Errorf("config.MaxVersion = %v, want %v", got, want)
			}
			if !compareNextProtos(config.NextProtos, tc.NextProtos) {
				t.Errorf("config.NextProtos = %v, want %v", config.NextProtos, tc.NextProtos)
			}
		})
	}
	stop()
}

func TestGetTLSMinMaxVersionsClient(t *testing.T) {
	m := makeMapOfTLSVersions()
	for min := commonpb.TLSVersion_TLS_VERSION_1_0; min <= commonpb.TLSVersion_TLS_VERSION_1_3; min++ {
		for max := commonpb.TLSVersion_TLS_VERSION_1_0; max <= commonpb.TLSVersion_TLS_VERSION_1_3; max++ {
			tlsConfig := &s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration{
				MinTlsVersion: min,
				MaxTlsVersion: max,
			}
			tlsMin, tlsMax, err := getTLSMinMaxVersionsServer(tlsConfig)
			if err != nil {
				if min <= max {
					t.Errorf("err = %v, expected err = nil", err)
				} else {
					if m[min] != tlsMin {
						t.Errorf("tlsMin = %v, expected %v", tlsMin, m[min])
					}
					if m[max] != tlsMax {
						t.Errorf("tlsMax = %v, expected %v", tlsMax, m[max])
					}
				}
			} else {
				if min > max {
					t.Errorf("err = nil, expected err = S2Av2 provided minVersion > maxVersion.")
				} else {
					if m[min] != tlsMin {
						t.Errorf("tlsMin = %v, expected %v", tlsMin, m[min])
					}
					if m[max] != tlsMax {
						t.Errorf("tlsMax = %v, expected %v", tlsMax, m[max])
					}
				}
			}
		}
	}
	// Test invalid input.
	tlsConfig := &s2av2pb.GetTlsConfigurationResp_ClientTlsConfiguration{
		MinTlsVersion: commonpb.TLSVersion_TLS_VERSION_1_0 - 1,
		MaxTlsVersion: commonpb.TLSVersion_TLS_VERSION_1_3,
	}
	expErr := fmt.Errorf("S2Av2 provided invalid MinTlsVersion: %v", tlsConfig.MinTlsVersion)
	_, _, err := getTLSMinMaxVersionsClient(tlsConfig)
	if (err == nil) || (err.Error() != expErr.Error()) {
		t.Errorf("err = %v, expErr = %v", err, expErr)
	}

	tlsConfig = &s2av2pb.GetTlsConfigurationResp_ClientTlsConfiguration{
		MinTlsVersion: commonpb.TLSVersion_TLS_VERSION_1_0,
		MaxTlsVersion: commonpb.TLSVersion_TLS_VERSION_1_3 + 1,
	}
	expErr = fmt.Errorf("S2Av2 provided invalid MaxTlsVersion: %v", tlsConfig.MaxTlsVersion)
	_, _, err = getTLSMinMaxVersionsClient(tlsConfig)
	if (err == nil) || (err.Error() != expErr.Error()) {
		t.Errorf("err = %v, expErr = %v", err, expErr)
	}
}

func TestGetTLSMinMaxVersionsServer(t *testing.T) {
	m := makeMapOfTLSVersions()
	for min := commonpb.TLSVersion_TLS_VERSION_1_0; min <= commonpb.TLSVersion_TLS_VERSION_1_3; min++ {
		for max := commonpb.TLSVersion_TLS_VERSION_1_0; max <= commonpb.TLSVersion_TLS_VERSION_1_3; max++ {
			tlsConfig := &s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration{
				MinTlsVersion: min,
				MaxTlsVersion: max,
			}
			tlsMin, tlsMax, err := getTLSMinMaxVersionsServer(tlsConfig)
			if err != nil {
				if min <= max {
					t.Errorf("err = %v, expected err = nil", err)
				} else {
					if m[min] != tlsMin {
						t.Errorf("tlsMin = %v, expected %v", tlsMin, m[min])
					}
					if m[max] != tlsMax {
						t.Errorf("tlsMax = %v, expected %v", tlsMax, m[max])
					}
				}
			} else {
				if min > max {
					t.Errorf("err = nil, expected err = S2Av2 provided minVersion > maxVersion.")
				} else {
					if m[min] != tlsMin {
						t.Errorf("tlsMin = %v, expected %v", tlsMin, m[min])
					}
					if m[max] != tlsMax {
						t.Errorf("tlsMax = %v, expected %v", tlsMax, m[max])
					}
				}
			}
		}
	}

	// Test invalid input.
	tlsConfig := &s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration{
		MinTlsVersion: commonpb.TLSVersion_TLS_VERSION_1_0 - 1,
		MaxTlsVersion: commonpb.TLSVersion_TLS_VERSION_1_3,
	}
	expErr := fmt.Errorf("S2Av2 provided invalid MinTlsVersion: %v", tlsConfig.MinTlsVersion)
	_, _, err := getTLSMinMaxVersionsServer(tlsConfig)
	if (err == nil) || (err.Error() != expErr.Error()) {
		t.Errorf("err = %v, expErr = %v", err, expErr)
	}

	tlsConfig = &s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration{
		MinTlsVersion: commonpb.TLSVersion_TLS_VERSION_1_0,
		MaxTlsVersion: commonpb.TLSVersion_TLS_VERSION_1_3 + 1,
	}
	expErr = fmt.Errorf("S2Av2 provided invalid MaxTlsVersion: %v", tlsConfig.MaxTlsVersion)
	_, _, err = getTLSMinMaxVersionsServer(tlsConfig)
	if (err == nil) || (err.Error() != expErr.Error()) {
		t.Errorf("err = %v, expErr = %v", err, expErr)
	}
}

func TestGetAuthMechanisms(t *testing.T) {
	// Setup data for test.
	sortProtos := cmpopts.SortSlices(func(m1, m2 *s2av2pb.AuthenticationMechanism) bool { return m1.String() < m2.String() })

	// TODO(rmehta19): Add additional tests.
	for _, tc := range []struct {
		description            string
		tokenManager           tokenmanager.AccessTokenManager
		localIdentities        []*commonpbv1.Identity
		expectedAuthMechanisms []*s2av2pb.AuthenticationMechanism
	}{
		{
			description:            "fake token manager is nil",
			tokenManager:           nil,
			expectedAuthMechanisms: nil,
		},
		{
			description: "fake token manager allows empty identity",
			tokenManager: &fakeAccessTokenManager{
				accessToken:        "TestGetAuthMechanisms_s2a_access_token",
				allowEmptyIdentity: true,
			},
			expectedAuthMechanisms: []*s2av2pb.AuthenticationMechanism{
				{
					MechanismOneof: &s2av2pb.AuthenticationMechanism_Token{
						Token: "TestGetAuthMechanisms_s2a_access_token",
					},
				},
			},
		},
		{
			description: "fake token manager does not allow empty identity",
			tokenManager: &fakeAccessTokenManager{
				allowEmptyIdentity: false,
			},
			expectedAuthMechanisms: nil,
		},
	} {
		t.Run(tc.description, func(t *testing.T) {
			authMechanisms := getAuthMechanisms(tc.tokenManager, tc.localIdentities)
			if got, want := (authMechanisms == nil), (tc.expectedAuthMechanisms == nil); got != want {
				t.Errorf("authMechanisms == nil: %t, tc.expectedAuthMechanisms == nil: %t", got, want)
			}
			if authMechanisms != nil && tc.expectedAuthMechanisms != nil {
				if diff := cmp.Diff(authMechanisms, tc.expectedAuthMechanisms, protocmp.Transform(), sortProtos); diff != "" {
					t.Errorf("getAuthMechanisms(%v, %v) returned incorrect slice, (-want +got):\n%s", tc.tokenManager, tc.localIdentities, diff)
				}
			}
		})
	}
}
func TestGetServerConfigFromS2Av2(t *testing.T) {
	// Start up fake S2Av2 server.
	var wg sync.WaitGroup
	wg.Add(1)
	stop, address, err := startFakeS2Av2Server(&wg, "TestGetServerConfigFromS2Av2_token", false)
	wg.Wait()
	if err != nil {
		t.Fatalf("Error starting fake S2Av2 Server: %v", err)
	}
	for _, tc := range []struct {
		description     string
		tokenManager    tokenmanager.AccessTokenManager
		localIdentities []*commonpbv1.Identity
		expTLSConfig    *s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration
		expErr          error
	}{
		{
			description: "empty accessToken and empty localIdentities",
			tokenManager: &fakeAccessTokenManager{
				allowEmptyIdentity: true,
				accessToken:        "",
			},
			localIdentities: nil,
			expErr:          errors.New("rpc error: code = Unknown desc = SessionReq has no AuthenticationMechanism with a valid token"),
		},
		{
			description: "invalid accessToken",
			tokenManager: &fakeAccessTokenManager{
				acceptedIdentity: &commonpbv1.Identity{
					IdentityOneof: &commonpbv1.Identity_Hostname{
						Hostname: "server_hostname",
					},
				},
				allowEmptyIdentity: true,
				accessToken:        "invalid_access_token",
			},
			localIdentities: []*commonpbv1.Identity{
				{
					IdentityOneof: &commonpbv1.Identity_Hostname{
						Hostname: "server_hostname",
					},
				},
			},
			expErr: errors.New("rpc error: code = Unknown desc = SessionReq has no AuthenticationMechanism with a valid token"),
		},
		{
			description: "correct accessToken and non - empty localIdentities",
			tokenManager: &fakeAccessTokenManager{
				acceptedIdentity: &commonpbv1.Identity{
					IdentityOneof: &commonpbv1.Identity_Hostname{
						Hostname: "server_hostname",
					},
				},
				allowEmptyIdentity: true,
				accessToken:        "TestGetServerConfigFromS2Av2_token",
			},
			localIdentities: []*commonpbv1.Identity{
				{
					IdentityOneof: &commonpbv1.Identity_Hostname{
						Hostname: "server_hostname",
					},
				},
			},
			expTLSConfig: &s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration{
				CertificateChain: []string{
					string(serverCertpem),
				},
				MinTlsVersion:            commonpb.TLSVersion_TLS_VERSION_1_3,
				MaxTlsVersion:            commonpb.TLSVersion_TLS_VERSION_1_3,
				TlsResumptionEnabled:     false,
				RequestClientCertificate: s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration_REQUEST_AND_REQUIRE_CLIENT_CERTIFICATE_AND_VERIFY,
				MaxOverheadOfTicketAead:  0,
			},
			expErr: nil,
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
			gotTLSConfig, gotErr := getServerConfigFromS2Av2(tc.tokenManager, tc.localIdentities, "test_server_name", cstream)
			if gotErr != tc.expErr {
				if (gotErr == nil) || (tc.expErr == nil) {
					t.Errorf("gotErr = %v,  tc.expErr = %v", gotErr, tc.expErr)
				} else if gotErr.Error() != tc.expErr.Error() {
					t.Errorf("gotErr = %v, tc.expErr = %v", gotErr, tc.expErr)
				}
			}
			if (gotErr == nil) && (tc.expErr == nil) {
				if diff := cmp.Diff(gotTLSConfig, tc.expTLSConfig, protocmp.Transform()); diff != "" {
					t.Errorf("getServerConfigFromS2Av2 returned incorrect GetTlsConfigurationResp_ServerTlsConfiguration, (-want +got):\n%s", diff)
				}
			}
		})
	}
	stop()
}

func TestGetClientConfig(t *testing.T) {
	// Setup test.
	cert, err := tls.X509KeyPair(serverCertpem, serverKeypem)
	if err != nil {
		t.Fatalf("tls.X509KeyPair failed: %v", err)
	}
	// Start up fake S2Av2 server.
	var wg sync.WaitGroup
	wg.Add(1)
	stop, address, err := startFakeS2Av2Server(&wg, "TestGetClientConfig_token", false)
	wg.Wait()
	if err != nil {
		t.Fatalf("Error starting fake S2Av2 Server: %v", err)
	}

	accessTokenManager := &fakeAccessTokenManager{
		accessToken:        "TestGetClientConfig_token",
		allowEmptyIdentity: true,
	}
	var identities []*commonpbv1.Identity
	identities = append(identities, nil)
	for _, tc := range []struct {
		description            string
		Certificates           []tls.Certificate
		SessionTicketsDisabled bool
		ClientAuth             tls.ClientAuthType
		MinVersion             uint16
		MaxVersion             uint16
		NextProtos             []string
	}{
		{
			description:            "static",
			Certificates:           []tls.Certificate{cert},
			SessionTicketsDisabled: true,
			ClientAuth:             tls.RequireAnyClientCert,
			MinVersion:             tls.VersionTLS13,
			MaxVersion:             tls.VersionTLS13,
			NextProtos:             []string{"h2"},
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
			clientConfigFunc := ClientConfig(accessTokenManager, identities, s2av2pb.ValidatePeerCertificateChainReq_CONNECT_TO_GOOGLE, cstream)
			config, err := clientConfigFunc(&tls.ClientHelloInfo{
				ServerName: "host_1",
			})
			if err != nil {
				t.Errorf("ClientConfig failed: %v", err)
			}
			if got, want := config.Certificates[0].Certificate[0], tc.Certificates[0].Certificate[0]; !bytes.Equal(got, want) {
				t.Errorf("config.Certificates[0].Certificate[0] = %v, want %v", got, want)
			}
			if got, want := config.SessionTicketsDisabled, tc.SessionTicketsDisabled; got != want {
				t.Errorf("config.SessionTicketsDisabled = %v, want %v", got, want)
			}
			if got, want := config.ClientAuth, tc.ClientAuth; got != want {
				t.Errorf("config.ClientAuth = %v, want %v", got, want)
			}
			if got, want := config.MinVersion, tc.MinVersion; got != want {
				t.Errorf("config.MinVersion = %v, want %v", got, want)
			}
			if got, want := config.MaxVersion, tc.MaxVersion; got != want {
				t.Errorf("config.MaxVersion = %v, want %v", got, want)
			}
			if !compareNextProtos(config.NextProtos, tc.NextProtos) {
				t.Errorf("config.NextProtos = %v, want %v", config.NextProtos, tc.NextProtos)
			}
		})
	}
	stop()
}

func TestGetTLSClientAuthType(t *testing.T) {
	for _, tc := range []struct {
		description       string
		tlsConfig         *s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration
		expClientAuthType tls.ClientAuthType
	}{
		{
			description: "Don't request client cert",
			tlsConfig: &s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration{
				RequestClientCertificate: s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration_DONT_REQUEST_CLIENT_CERTIFICATE,
			},
			expClientAuthType: tls.NoClientCert,
		},
		{
			description: "Request client cert, but don't verify",
			tlsConfig: &s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration{
				RequestClientCertificate: s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration_REQUEST_CLIENT_CERTIFICATE_BUT_DONT_VERIFY,
			},
			expClientAuthType: tls.RequestClientCert,
		},
		{
			description: "Request client cert and verify",
			tlsConfig: &s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration{
				RequestClientCertificate: s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration_REQUEST_CLIENT_CERTIFICATE_AND_VERIFY,
			},
			expClientAuthType: tls.RequireAnyClientCert,
		},
		{
			description: "Request and Require client cert, but don't verify",
			tlsConfig: &s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration{
				RequestClientCertificate: s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration_REQUEST_AND_REQUIRE_CLIENT_CERTIFICATE_BUT_DONT_VERIFY,
			},
			expClientAuthType: tls.RequireAnyClientCert,
		},
		{
			description: "Request and Require client cert and verify",
			tlsConfig: &s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration{
				RequestClientCertificate: s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration_REQUEST_AND_REQUIRE_CLIENT_CERTIFICATE_AND_VERIFY,
			},
			expClientAuthType: tls.RequireAnyClientCert,
		},
		{
			description: "default case",
			tlsConfig: &s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration{
				RequestClientCertificate: s2av2pb.GetTlsConfigurationResp_ServerTlsConfiguration_UNSPECIFIED,
			},
			expClientAuthType: tls.RequireAnyClientCert,
		},
	} {
		t.Run(tc.description, func(t *testing.T) {
			if got, want := getTLSClientAuthType(tc.tlsConfig), tc.expClientAuthType; got != want {
				t.Errorf("getClientAuthType(%v) returned %v, want = %v", tc.tlsConfig, got, want)
			}
		})
	}
}

func TestGetTLSCipherSuite(t *testing.T) {
	for _, tc := range []struct {
		description string
		inp         commonpb.Ciphersuite
		expOut      uint16
	}{
		{
			description: "ECDSA with AES 128 GCM SHA256",
			inp:         commonpb.Ciphersuite_CIPHERSUITE_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
			expOut:      tls.TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256,
		},
		{
			description: "ECDSA with AES 256 GCM SHA384",
			inp:         commonpb.Ciphersuite_CIPHERSUITE_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
			expOut:      tls.TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384,
		},
		{
			description: "ECDSA with CHACHA20 Poly 1305 SHA256",
			inp:         commonpb.Ciphersuite_CIPHERSUITE_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
			expOut:      tls.TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256,
		},
		{
			description: "RSA with AES 128 GCM SHA256",
			inp:         commonpb.Ciphersuite_CIPHERSUITE_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
			expOut:      tls.TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,
		},
		{
			description: "RSA with AES 256 GCM SHA384",
			inp:         commonpb.Ciphersuite_CIPHERSUITE_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
			expOut:      tls.TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384,
		},
		{
			description: "RSA with CHACHA20 Poly1305 SHA256",
			inp:         commonpb.Ciphersuite_CIPHERSUITE_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
			expOut:      tls.TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256,
		},
		{
			description: "unspecified",
			inp:         commonpb.Ciphersuite_CIPHERSUITE_UNSPECIFIED,
			expOut:      0xffff,
		},
	} {
		t.Run(tc.description, func(t *testing.T) {
			if got, want := getTLSCipherSuite(tc.inp), tc.expOut; got != want {
				t.Errorf("getTLSCipherSuite(%v) returned %v, want %v", tc.inp, got, want)
			}
		})
	}
}

func TestGetCipherSuites(t *testing.T) {
	for _, tc := range []struct {
		description string
		inp         []commonpb.Ciphersuite
		expOut      []uint16
	}{
		{
			description: "empty input",
			inp:         []commonpb.Ciphersuite{},
			expOut:      []uint16{},
		},
		{
			description: "non - empty input array of size 1",
			inp:         []commonpb.Ciphersuite{commonpb.Ciphersuite_CIPHERSUITE_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256},
			expOut:      []uint16{tls.TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256},
		},
		{
			description: "non - empty input array of size 2",
			inp:         []commonpb.Ciphersuite{commonpb.Ciphersuite_CIPHERSUITE_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256, commonpb.Ciphersuite_CIPHERSUITE_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384},
			expOut:      []uint16{tls.TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256, tls.TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384},
		},
		{
			description: "unspecified array of size 1",
			inp:         []commonpb.Ciphersuite{commonpb.Ciphersuite_CIPHERSUITE_UNSPECIFIED},
			expOut:      []uint16{},
		},
		{
			description: "unspecified array of size 2",
			inp:         []commonpb.Ciphersuite{commonpb.Ciphersuite_CIPHERSUITE_UNSPECIFIED, commonpb.Ciphersuite_CIPHERSUITE_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256},
			expOut:      []uint16{tls.TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256},
		},
	} {
		t.Run(tc.description, func(t *testing.T) {
			if got, want := getCipherSuites(tc.inp), tc.expOut; !compareCipherSuites(got, want) {
				t.Errorf("getCipherSuites(%+v) returned %+v, want %+v", tc.inp, got, want)
			}
		})
	}
}

func makeMapOfTLSVersions() map[commonpb.TLSVersion]uint16 {
	m := make(map[commonpb.TLSVersion]uint16)
	m[commonpb.TLSVersion_TLS_VERSION_1_0] = tls.VersionTLS10
	m[commonpb.TLSVersion_TLS_VERSION_1_1] = tls.VersionTLS11
	m[commonpb.TLSVersion_TLS_VERSION_1_2] = tls.VersionTLS12
	m[commonpb.TLSVersion_TLS_VERSION_1_3] = tls.VersionTLS13
	return m
}

func compareNextProtos(a, b []string) bool {
	if len(a) != len(b) {
		return false
	}
	for i, v := range a {
		if v != b[i] {
			return false
		}
	}
	return true
}

func compareCipherSuites(a, b []uint16) bool {
	if len(a) != len(b) {
		return false
	}
	for i, v := range a {
		if v != b[i] {
			return false
		}
	}
	return true
}

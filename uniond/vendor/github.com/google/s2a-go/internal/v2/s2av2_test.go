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
	"context"
	"os"
	"reflect"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/google/s2a-go/fallback"
	"github.com/google/s2a-go/internal/tokenmanager"
	"github.com/google/s2a-go/stream"
	"google.golang.org/protobuf/testing/protocmp"

	commonpbv1 "github.com/google/s2a-go/internal/proto/common_go_proto"
	s2av2pb "github.com/google/s2a-go/internal/proto/v2/s2a_go_proto"
)

var (
	fakes2av2Address = "0.0.0.0:0"
)

func TestNewClientCreds(t *testing.T) {
	os.Setenv("S2A_ACCESS_TOKEN", "TestNewClientCreds_s2a_access_token")
	for _, tc := range []struct {
		description string
	}{
		{
			description: "static",
		},
	} {
		t.Run(tc.description, func(t *testing.T) {
			c, err := NewClientCreds(fakes2av2Address, nil, &commonpbv1.Identity{
				IdentityOneof: &commonpbv1.Identity_Hostname{
					Hostname: "test_rsa_client_identity",
				},
			}, s2av2pb.ValidatePeerCertificateChainReq_CONNECT_TO_GOOGLE, nil, nil, nil)
			if err != nil {
				t.Fatalf("NewClientCreds() failed: %v", err)
			}
			if got, want := c.Info().SecurityProtocol, s2aSecurityProtocol; got != want {
				t.Errorf("c.Info().SecurityProtocol = %v, want %v", got, want)
			}
			_, ok := c.(*s2av2TransportCreds)
			if !ok {
				t.Fatal("The created creds is not of type s2av2TransportCreds")
			}
		})
	}
}

func TestNewServerCreds(t *testing.T) {
	os.Setenv("S2A_ACCESS_TOKEN", "TestNewServerCreds_s2a_access_token")
	for _, tc := range []struct {
		description string
	}{
		{
			description: "static",
		},
	} {
		t.Run(tc.description, func(t *testing.T) {
			localIdentities := []*commonpbv1.Identity{
				{
					IdentityOneof: &commonpbv1.Identity_Hostname{
						Hostname: "test_rsa_server_identity",
					},
				},
			}
			c, err := NewServerCreds(fakes2av2Address, nil, localIdentities, s2av2pb.ValidatePeerCertificateChainReq_CONNECT_TO_GOOGLE, nil)
			if err != nil {
				t.Fatalf("NewServerCreds() failed: %v", err)
			}
			if got, want := c.Info().SecurityProtocol, s2aSecurityProtocol; got != want {
				t.Errorf("c.Info().SecurityProtocol = %v, want %v", got, want)
			}
			_, ok := c.(*s2av2TransportCreds)
			if !ok {
				t.Fatal("The created creds is not of type s2av2TransportCreds")
			}
		})
	}
}

func TestClientHandshakeFail(t *testing.T) {
	cc := &s2av2TransportCreds{isClient: false}
	if _, _, err := cc.ClientHandshake(context.Background(), "", nil); err == nil {
		t.Errorf("c.ClientHandshake(nil, \"\", nil) should fail with incorrect transport credentials")
	}
}

func TestServerHandshakeFail(t *testing.T) {
	sc := &s2av2TransportCreds{isClient: true}
	if _, _, err := sc.ServerHandshake(nil); err == nil {
		t.Errorf("c.ServerHandshake(nil) should fail with incorrect transport credentials")
	}
}

func TestInfo(t *testing.T) {
	os.Setenv("S2A_ACCESS_TOKEN", "TestInfo_s2a_access_token")
	c, err := NewClientCreds(fakes2av2Address, nil, &commonpbv1.Identity{
		IdentityOneof: &commonpbv1.Identity_Hostname{
			Hostname: "test_rsa_client_identity",
		},
	}, s2av2pb.ValidatePeerCertificateChainReq_CONNECT_TO_GOOGLE, nil, nil, nil)
	if err != nil {
		t.Fatalf("NewClientCreds() failed: %v", err)
	}
	info := c.Info()
	if got, want := info.SecurityProtocol, "tls"; got != want {
		t.Errorf("info.SecurityProtocol=%v, want %v", got, want)
	}
}

func TestCloneClient(t *testing.T) {
	os.Setenv("S2A_ACCESS_TOKEN", "TestCloneClient_s2a_access_token")
	fallbackFunc, err := fallback.DefaultFallbackClientHandshakeFunc("example.com")
	if err != nil {
		t.Errorf("error creating fallback handshake function: %v", err)
	}
	c, err := NewClientCreds(fakes2av2Address, nil, &commonpbv1.Identity{
		IdentityOneof: &commonpbv1.Identity_Hostname{
			Hostname: "test_rsa_client_identity",
		},
	}, s2av2pb.ValidatePeerCertificateChainReq_CONNECT_TO_GOOGLE, fallbackFunc, nil, nil)
	if err != nil {
		t.Fatalf("NewClientCreds() failed: %v", err)
	}
	cc := c.Clone()
	s2av2Creds, ok := c.(*s2av2TransportCreds)
	if !ok {
		t.Fatal("The created creds is not of type s2av2TransportCreds")
	}
	s2av2CloneCreds, ok := cc.(*s2av2TransportCreds)
	if !ok {
		t.Fatal("The created clone creds is not of type s2aTransportCreds")
	}
	if got, want := cmp.Equal(s2av2Creds, s2av2CloneCreds, protocmp.Transform(), cmp.AllowUnexported(s2av2TransportCreds{}), cmp.Comparer(func(x, y tokenmanager.AccessTokenManager) bool {
		xToken, err := x.DefaultToken()
		if err != nil {
			t.Errorf("Failed to compare cloned creds: %v", err)
		}
		yToken, err := y.DefaultToken()
		if err != nil {
			t.Errorf("Failed to compare cloned creds: %v", err)
		}
		if xToken == yToken {
			return true
		}
		return false
	}), cmp.Comparer(func(x, y fallback.ClientHandshake) bool {
		return reflect.ValueOf(x) == reflect.ValueOf(y)
	})), true; got != want {
		t.Errorf("cmp.Equal(%+v, %+v) = %v, want %v", s2av2Creds, s2av2CloneCreds, got, want)
	}
	// Change the values and verify the creds were deep copied.
	s2av2CloneCreds.info.SecurityProtocol = "s2a"
	if got, want := cmp.Equal(s2av2Creds, s2av2CloneCreds, protocmp.Transform(), cmp.AllowUnexported(s2av2TransportCreds{}), cmp.Comparer(func(x, y tokenmanager.AccessTokenManager) bool {
		xToken, err := x.DefaultToken()
		if err != nil {
			t.Errorf("Failed to compare cloned creds: %v", err)
		}
		yToken, err := y.DefaultToken()
		if err != nil {
			t.Errorf("Failed to compare cloned creds: %v", err)
		}
		if xToken == yToken {
			return true
		}
		return false
	})), false; got != want {
		t.Errorf("cmp.Equal(%+v, %+v) = %v, want %v", s2av2Creds, s2av2CloneCreds, got, want)
	}
}

func TestCloneServer(t *testing.T) {
	os.Setenv("S2A_ACCESS_TOKEN", "TestCloneServer_s2a_access_token")
	localIdentities := []*commonpbv1.Identity{
		{
			IdentityOneof: &commonpbv1.Identity_Hostname{
				Hostname: "test_rsa_server_identity",
			},
		},
	}
	c, err := NewServerCreds(fakes2av2Address, nil, localIdentities, s2av2pb.ValidatePeerCertificateChainReq_CONNECT_TO_GOOGLE, nil)
	if err != nil {
		t.Fatalf("NewServerCreds() failed: %v", err)
	}
	cc := c.Clone()
	s2av2Creds, ok := c.(*s2av2TransportCreds)
	if !ok {
		t.Fatal("The created creds is not of type s2av2TransportCreds")
	}
	s2av2CloneCreds, ok := cc.(*s2av2TransportCreds)
	if !ok {
		t.Fatal("The created clone creds is not of type s2aTransportCreds")
	}
	if got, want := cmp.Equal(s2av2Creds, s2av2CloneCreds, protocmp.Transform(), cmp.AllowUnexported(s2av2TransportCreds{}), cmp.Comparer(func(x, y tokenmanager.AccessTokenManager) bool {
		xToken, err := x.DefaultToken()
		if err != nil {
			t.Errorf("Failed to compare cloned creds: %v", err)
		}
		yToken, err := y.DefaultToken()
		if err != nil {
			t.Errorf("Failed to compare cloned creds: %v", err)
		}
		if xToken == yToken {
			return true
		}
		return false
	})), true; got != want {
		t.Errorf("cmp.Equal(%+v, %+v) = %v, want %v", s2av2Creds, s2av2CloneCreds, got, want)
	}
	// Change the values and verify the creds were deep copied.
	s2av2CloneCreds.info.SecurityProtocol = "s2a"
	if got, want := cmp.Equal(s2av2Creds, s2av2CloneCreds, protocmp.Transform(), cmp.AllowUnexported(s2av2TransportCreds{}), cmp.Comparer(func(x, y tokenmanager.AccessTokenManager) bool {
		xToken, err := x.DefaultToken()
		if err != nil {
			t.Errorf("Failed to compare cloned creds: %v", err)
		}
		yToken, err := y.DefaultToken()
		if err != nil {
			t.Errorf("Failed to compare cloned creds: %v", err)
		}
		if xToken == yToken {
			return true
		}
		return false
	})), false; got != want {
		t.Errorf("cmp.Equal(%+v, %+v) = %v, want %v", s2av2Creds, s2av2CloneCreds, got, want)
	}
}

func TestOverrideServerName(t *testing.T) {
	// Setup test.
	os.Setenv("S2A_ACCESS_TOKEN", "TestOverrideServerName_s2a_access_token")
	c, err := NewClientCreds(fakes2av2Address, nil, &commonpbv1.Identity{
		IdentityOneof: &commonpbv1.Identity_Hostname{
			Hostname: "test_rsa_client_identity",
		},
	}, s2av2pb.ValidatePeerCertificateChainReq_CONNECT_TO_GOOGLE, nil, nil, nil)
	s2av2Creds, ok := c.(*s2av2TransportCreds)
	if !ok {
		t.Fatal("The created creds is not of type s2av2TransportCreds")
	}
	if err != nil {
		t.Fatalf("NewClientCreds() failed: %v", err)
	}
	if got, want := c.Info().ServerName, ""; got != want {
		t.Errorf("c.Info().ServerName = %v, want %v", got, want)
	}
	if got, want := s2av2Creds.serverName, ""; got != want {
		t.Errorf("c.serverName = %v, want %v", got, want)
	}
	for _, tc := range []struct {
		description    string
		override       string
		wantServerName string
		expectError    bool
	}{
		{
			description:    "empty string",
			override:       "",
			wantServerName: "",
		},
		{
			description:    "host only",
			override:       "server.name",
			wantServerName: "server.name",
		},
		{
			description:    "invalid syntax",
			override:       "server::",
			wantServerName: "server::",
		},
		{
			description:    "split host port",
			override:       "host:port",
			wantServerName: "host",
		},
	} {
		t.Run(tc.description, func(t *testing.T) {
			c.OverrideServerName(tc.override)
			if got, want := c.Info().ServerName, tc.wantServerName; got != want {
				t.Errorf("c.Info().ServerName = %v, want %v", got, want)
			}
			if got, want := s2av2Creds.serverName, tc.wantServerName; got != want {
				t.Errorf("c.serverName = %v, want %v", got, want)
			}
		})
	}
}

type s2ATestStream struct {
	debug string
}

func (x s2ATestStream) Send(m *s2av2pb.SessionReq) error {
	return nil
}

func (x s2ATestStream) Recv() (*s2av2pb.SessionResp, error) {
	return nil, nil
}

func (x s2ATestStream) CloseSend() error {
	return nil
}

func TestCreateStream(t *testing.T) {
	for _, tc := range []struct {
		description string
	}{
		{
			description: "static",
		},
	} {
		t.Run(tc.description, func(t *testing.T) {
			s2AStream, err := createStream(context.TODO(), "fake address", nil, func(ctx context.Context, s2av2Address string) (stream.S2AStream, error) {
				return s2ATestStream{debug: "test s2a stream"}, nil
			})
			if err != nil {
				t.Fatalf("New S2AStream failed: %v", err)
			}
			testStream, ok := s2AStream.(s2ATestStream)
			if !ok {
				t.Fatal("The created stream is not of type s2ATestStream")
			}
			if testStream.debug != "test s2a stream" {
				t.Errorf("The created stream is not the intended stream")
			}
		})
	}
}

func TestGetS2ATimeout(t *testing.T) {
	oldEnvValue := os.Getenv(s2aTimeoutEnv)
	defer os.Setenv(s2aTimeoutEnv, oldEnvValue)

	// Unset the environment var
	os.Unsetenv(s2aTimeoutEnv)
	if got, want := GetS2ATimeout(), defaultS2ATimeout; got != want {
		t.Fatalf("GetS2ATimeout should return default if S2A_TIMEOUT is not set")
	}

	// Set the environment var to empty string
	os.Setenv(s2aTimeoutEnv, "")
	if got, want := GetS2ATimeout(), defaultS2ATimeout; got != want {
		t.Fatalf("GetS2ATimeout should return default if S2A_TIMEOUT is set to empty string")
	}

	// Set a valid duration string
	os.Setenv(s2aTimeoutEnv, "5s")
	if got, want := GetS2ATimeout(), 5*time.Second; got != want {
		t.Fatalf("expected timeout to be 5s")
	}

	// Set an invalid duration string
	os.Setenv(s2aTimeoutEnv, "5abc")
	if got, want := GetS2ATimeout(), defaultS2ATimeout; got != want {
		t.Fatalf("expected timeout to be default if the set timeout is invalid")
	}
}

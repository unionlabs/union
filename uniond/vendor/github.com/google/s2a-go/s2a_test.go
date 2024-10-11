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
	"context"
	"testing"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/protobuf/testing/protocmp"

	s2apb "github.com/google/s2a-go/internal/proto/common_go_proto"
	s2av2pb "github.com/google/s2a-go/internal/proto/v2/s2a_go_proto"
)

func TestNewClientCreds(t *testing.T) {
	for _, tc := range []struct {
		desc                string
		opts                *ClientOptions
		outMinTLSVersion    s2apb.TLSVersion
		outMaxTLSVersion    s2apb.TLSVersion
		outTLSCiphersuites  []s2apb.Ciphersuite
		outLocalIdentity    *s2apb.Identity
		outTargetIdentities []*s2apb.Identity
		outS2AAddress       string
	}{
		{
			desc: "only hostnames",
			opts: &ClientOptions{
				TargetIdentities: []Identity{
					&hostname{"test_server_hostname"},
				},
				LocalIdentity:    &hostname{"test_client_hostname"},
				S2AAddress:       "test_s2a_address",
				EnableLegacyMode: true,
			},
			outMinTLSVersion: s2apb.TLSVersion_TLS1_3,
			outMaxTLSVersion: s2apb.TLSVersion_TLS1_3,
			outTLSCiphersuites: []s2apb.Ciphersuite{
				s2apb.Ciphersuite_AES_128_GCM_SHA256,
				s2apb.Ciphersuite_AES_256_GCM_SHA384,
				s2apb.Ciphersuite_CHACHA20_POLY1305_SHA256},
			outTargetIdentities: []*s2apb.Identity{
				{
					IdentityOneof: &s2apb.Identity_Hostname{
						Hostname: "test_server_hostname",
					},
				},
			},
			outLocalIdentity: &s2apb.Identity{
				IdentityOneof: &s2apb.Identity_Hostname{
					Hostname: "test_client_hostname",
				},
			},
			outS2AAddress: "test_s2a_address",
		},
		{
			desc: "only spiffe IDs",
			opts: &ClientOptions{
				TargetIdentities: []Identity{
					&spiffeID{"test_server_spiffe_id"},
				},
				LocalIdentity:    &spiffeID{"test_client_spiffe_id"},
				S2AAddress:       "test_s2a_address",
				EnableLegacyMode: true,
			},
			outMinTLSVersion: s2apb.TLSVersion_TLS1_3,
			outMaxTLSVersion: s2apb.TLSVersion_TLS1_3,
			outTLSCiphersuites: []s2apb.Ciphersuite{
				s2apb.Ciphersuite_AES_128_GCM_SHA256,
				s2apb.Ciphersuite_AES_256_GCM_SHA384,
				s2apb.Ciphersuite_CHACHA20_POLY1305_SHA256},
			outTargetIdentities: []*s2apb.Identity{
				{
					IdentityOneof: &s2apb.Identity_SpiffeId{
						SpiffeId: "test_server_spiffe_id",
					},
				},
			},
			outLocalIdentity: &s2apb.Identity{
				IdentityOneof: &s2apb.Identity_SpiffeId{
					SpiffeId: "test_client_spiffe_id",
				},
			},
			outS2AAddress: "test_s2a_address",
		},
		{
			desc: "only UIDs",
			opts: &ClientOptions{
				TargetIdentities: []Identity{
					&uid{"test_server_uid"},
				},
				LocalIdentity:    &uid{"test_client_uid"},
				S2AAddress:       "test_s2a_address",
				EnableLegacyMode: true,
			},
			outMinTLSVersion: s2apb.TLSVersion_TLS1_3,
			outMaxTLSVersion: s2apb.TLSVersion_TLS1_3,
			outTLSCiphersuites: []s2apb.Ciphersuite{
				s2apb.Ciphersuite_AES_128_GCM_SHA256,
				s2apb.Ciphersuite_AES_256_GCM_SHA384,
				s2apb.Ciphersuite_CHACHA20_POLY1305_SHA256},
			outTargetIdentities: []*s2apb.Identity{
				{
					IdentityOneof: &s2apb.Identity_Uid{
						Uid: "test_server_uid",
					},
				},
			},
			outLocalIdentity: &s2apb.Identity{
				IdentityOneof: &s2apb.Identity_Uid{
					Uid: "test_client_uid",
				},
			},
			outS2AAddress: "test_s2a_address",
		},
		{
			desc: "mixed identities",
			opts: &ClientOptions{
				TargetIdentities: []Identity{
					&spiffeID{"test_server_spiffe_id"},
					&hostname{"test_server_hostname"},
					&uid{"test_server_uid"},
				},
				LocalIdentity:    &spiffeID{"test_client_spiffe_id"},
				S2AAddress:       "test_s2a_address",
				EnableLegacyMode: true,
			},
			outMinTLSVersion: s2apb.TLSVersion_TLS1_3,
			outMaxTLSVersion: s2apb.TLSVersion_TLS1_3,
			outTLSCiphersuites: []s2apb.Ciphersuite{
				s2apb.Ciphersuite_AES_128_GCM_SHA256,
				s2apb.Ciphersuite_AES_256_GCM_SHA384,
				s2apb.Ciphersuite_CHACHA20_POLY1305_SHA256},
			outTargetIdentities: []*s2apb.Identity{
				{
					IdentityOneof: &s2apb.Identity_SpiffeId{
						SpiffeId: "test_server_spiffe_id",
					},
				},
				{
					IdentityOneof: &s2apb.Identity_Hostname{
						Hostname: "test_server_hostname",
					},
				},
				{
					IdentityOneof: &s2apb.Identity_Uid{
						Uid: "test_server_uid",
					},
				},
			},
			outLocalIdentity: &s2apb.Identity{
				IdentityOneof: &s2apb.Identity_SpiffeId{
					SpiffeId: "test_client_spiffe_id",
				},
			},
			outS2AAddress: "test_s2a_address",
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			c, err := NewClientCreds(tc.opts)
			if err != nil {
				t.Fatalf("NewClientCreds(_) failed: %v", err)
			}
			if got, want := c.Info().SecurityProtocol, s2aSecurityProtocol; got != want {
				t.Errorf("c.Info().SecurityProtocol = %v, want %v", got, want)
			}
			s2aCreds, ok := c.(*s2aTransportCreds)
			if !ok {
				t.Fatal("The created creds is not of type s2aTransportCreds")
			}
			if got, want := s2aCreds.minTLSVersion, tc.outMinTLSVersion; got != want {
				t.Errorf("s2aCreds.minTLSVersion = %v, want %v", got, want)
			}
			if got, want := s2aCreds.maxTLSVersion, tc.outMaxTLSVersion; got != want {
				t.Errorf("s2aCreds.maxTLSVersion = %v, want %v", got, want)
			}
			if got, want := s2aCreds.tlsCiphersuites, tc.outTLSCiphersuites; !cmp.Equal(got, want) {
				t.Errorf("s2aCreds.tlsCiphersuites = %v, want %v", got, want)
			}
			if got, want := s2aCreds.targetIdentities, tc.outTargetIdentities; !cmp.Equal(got, want, protocmp.Transform()) {
				t.Errorf("s2aCreds.targetIdentities = %v, want %v", got, want)
			}
			if got, want := s2aCreds.localIdentity, tc.outLocalIdentity; !cmp.Equal(got, want, protocmp.Transform()) {
				t.Errorf("s2aCreds.localIdentity = %v, want %v", got, want)
			}
			if got, want := s2aCreds.s2aAddr, tc.outS2AAddress; got != want {
				t.Errorf("s2aCreds.s2aAddr = %v, want %v", got, want)
			}
		})
	}
}

func TestNewServerCreds(t *testing.T) {
	for _, tc := range []struct {
		desc               string
		opts               *ServerOptions
		outMinTLSVersion   s2apb.TLSVersion
		outMaxTLSVersion   s2apb.TLSVersion
		outTLSCiphersuites []s2apb.Ciphersuite
		outLocalIdentities []*s2apb.Identity
		outS2AAddress      string
	}{
		{
			desc: "only hostnames",
			opts: &ServerOptions{
				LocalIdentities: []Identity{
					&hostname{"test_server_hostname"},
				},
				S2AAddress:       "test_s2a_address",
				EnableLegacyMode: true,
			},
			outMinTLSVersion: s2apb.TLSVersion_TLS1_3,
			outMaxTLSVersion: s2apb.TLSVersion_TLS1_3,
			outTLSCiphersuites: []s2apb.Ciphersuite{
				s2apb.Ciphersuite_AES_128_GCM_SHA256,
				s2apb.Ciphersuite_AES_256_GCM_SHA384,
				s2apb.Ciphersuite_CHACHA20_POLY1305_SHA256},
			outLocalIdentities: []*s2apb.Identity{
				{
					IdentityOneof: &s2apb.Identity_Hostname{
						Hostname: "test_server_hostname",
					},
				},
			},
			outS2AAddress: "test_s2a_address",
		},
		{
			desc: "only spiffe IDs",
			opts: &ServerOptions{
				LocalIdentities: []Identity{
					&spiffeID{"test_server_spiffe_id"},
				},
				S2AAddress:       "test_s2a_address",
				EnableLegacyMode: true,
			},
			outMinTLSVersion: s2apb.TLSVersion_TLS1_3,
			outMaxTLSVersion: s2apb.TLSVersion_TLS1_3,
			outTLSCiphersuites: []s2apb.Ciphersuite{
				s2apb.Ciphersuite_AES_128_GCM_SHA256,
				s2apb.Ciphersuite_AES_256_GCM_SHA384,
				s2apb.Ciphersuite_CHACHA20_POLY1305_SHA256},
			outLocalIdentities: []*s2apb.Identity{
				{
					IdentityOneof: &s2apb.Identity_SpiffeId{
						SpiffeId: "test_server_spiffe_id",
					},
				},
			},
			outS2AAddress: "test_s2a_address",
		},
		{
			desc: "only UIDs",
			opts: &ServerOptions{
				LocalIdentities: []Identity{
					&uid{"test_server_uid"},
				},
				S2AAddress:       "test_s2a_address",
				EnableLegacyMode: true,
			},
			outMinTLSVersion: s2apb.TLSVersion_TLS1_3,
			outMaxTLSVersion: s2apb.TLSVersion_TLS1_3,
			outTLSCiphersuites: []s2apb.Ciphersuite{
				s2apb.Ciphersuite_AES_128_GCM_SHA256,
				s2apb.Ciphersuite_AES_256_GCM_SHA384,
				s2apb.Ciphersuite_CHACHA20_POLY1305_SHA256},
			outLocalIdentities: []*s2apb.Identity{
				{
					IdentityOneof: &s2apb.Identity_Uid{
						Uid: "test_server_uid",
					},
				},
			},
			outS2AAddress: "test_s2a_address",
		},
		{
			desc: "mixed identities",
			opts: &ServerOptions{
				LocalIdentities: []Identity{
					&spiffeID{"test_server_spiffe_id"},
					&hostname{"test_server_hostname"},
					&uid{"test_server_uid"},
				},
				S2AAddress:       "test_s2a_address",
				EnableLegacyMode: true,
			},
			outMinTLSVersion: s2apb.TLSVersion_TLS1_3,
			outMaxTLSVersion: s2apb.TLSVersion_TLS1_3,
			outTLSCiphersuites: []s2apb.Ciphersuite{
				s2apb.Ciphersuite_AES_128_GCM_SHA256,
				s2apb.Ciphersuite_AES_256_GCM_SHA384,
				s2apb.Ciphersuite_CHACHA20_POLY1305_SHA256},
			outLocalIdentities: []*s2apb.Identity{
				{
					IdentityOneof: &s2apb.Identity_SpiffeId{
						SpiffeId: "test_server_spiffe_id",
					},
				},
				{
					IdentityOneof: &s2apb.Identity_Hostname{
						Hostname: "test_server_hostname",
					},
				},
				{
					IdentityOneof: &s2apb.Identity_Uid{
						Uid: "test_server_uid",
					},
				},
			},
			outS2AAddress: "test_s2a_address",
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			c, err := NewServerCreds(tc.opts)
			if err != nil {
				t.Fatalf("NewServerCreds(_) failed: %v", err)
			}
			if got, want := c.Info().SecurityProtocol, s2aSecurityProtocol; got != want {
				t.Errorf("c.Info().SecurityProtocol = %v, want %v", got, want)
			}
			s2aCreds, ok := c.(*s2aTransportCreds)
			if !ok {
				t.Fatal("The created creds is not of type s2aTransportCreds")
			}
			if got, want := s2aCreds.minTLSVersion, tc.outMinTLSVersion; got != want {
				t.Errorf("s2aCreds.minTLSVersion = %v, want %v", got, want)
			}
			if got, want := s2aCreds.maxTLSVersion, tc.outMaxTLSVersion; got != want {
				t.Errorf("s2aCreds.maxTLSVersion = %v, want %v", got, want)
			}
			if got, want := s2aCreds.tlsCiphersuites, tc.outTLSCiphersuites; !cmp.Equal(got, want) {
				t.Errorf("s2aCreds.tlsCiphersuites = %v, want %v", got, want)
			}
			if got, want := s2aCreds.localIdentities, tc.outLocalIdentities; !cmp.Equal(got, want, protocmp.Transform()) {
				t.Errorf("s2aCreds.localIdentities = %v, want %v", got, want)
			}
			if got, want := s2aCreds.s2aAddr, tc.outS2AAddress; got != want {
				t.Errorf("s2aCreds.s2aAddr = %v, want %v", got, want)
			}
		})
	}
}

func TestHandshakeFail(t *testing.T) {
	cc := &s2aTransportCreds{isClient: false}
	if _, _, err := cc.ClientHandshake(context.Background(), "", nil); err == nil {
		t.Errorf("c.ClientHandshake(nil, \"\", nil) should fail with incorrect transport credentials")
	}
	sc := &s2aTransportCreds{isClient: true}
	if _, _, err := sc.ServerHandshake(nil); err == nil {
		t.Errorf("c.ServerHandshake(nil) should fail with incorrect transport credentials")
	}
}

func TestInfo(t *testing.T) {
	// This is not testing any handshaker functionality, so it's fine to only
	// use NewServerCreds and not NewClientCreds.
	c, err := NewServerCreds(&ServerOptions{})
	if err != nil {
		t.Fatalf("NewServerCreds(&ServerOptions{}) failed: %v", err)
	}
	info := c.Info()
	if got, want := info.ProtocolVersion, ""; got != want {
		t.Errorf("info.ProtocolVersion=%v, want %v", got, want)
	}
	if got, want := info.SecurityProtocol, "tls"; got != want {
		t.Errorf("info.SecurityProtocol=%v, want %v", got, want)
	}
	if got, want := info.ServerName, ""; got != want {
		t.Errorf("info.ServerName=%v, want %v", got, want)
	}
}

func TestCloneClient(t *testing.T) {
	opt := &ClientOptions{
		TargetIdentities: []Identity{
			&spiffeID{"test_server_spiffe_id"},
			&hostname{"test_server_hostname"},
		},
		LocalIdentity:    &hostname{"test_client_hostname"},
		S2AAddress:       "test_s2a_address",
		EnableLegacyMode: true,
	}
	c, err := NewClientCreds(opt)
	if err != nil {
		t.Fatalf("NewClientCreds(%v) failed: %v", opt, err)
	}
	cc := c.Clone()
	s2aCreds, ok := c.(*s2aTransportCreds)
	if !ok {
		t.Fatal("The created creds is not of type s2aTransportCreds")
	}
	s2aCloneCreds, ok := cc.(*s2aTransportCreds)
	if !ok {
		t.Fatal("The created cloned creds is not of type s2aTransportCreds")
	}
	if got, want := cmp.Equal(s2aCreds, s2aCloneCreds, protocmp.Transform(), cmp.AllowUnexported(s2aTransportCreds{})), true; got != want {
		t.Errorf("cmp.Equal(%v, %v) = %v, want %v", s2aCreds, s2aCloneCreds, got, want)
	}
	// Change the values and verify that the creds were deep copied.
	s2aCloneCreds.targetIdentities[0] = &s2apb.Identity{
		IdentityOneof: &s2apb.Identity_SpiffeId{
			SpiffeId: "new_spiffe_id",
		},
	}
	if got, want := cmp.Equal(s2aCreds, s2aCloneCreds, protocmp.Transform(), cmp.AllowUnexported(s2aTransportCreds{})), false; got != want {
		t.Errorf("cmp.Equal(%v, %v) = %v, want %v", s2aCreds, s2aCloneCreds, got, want)
	}
}

func TestCloneServer(t *testing.T) {
	c, err := NewServerCreds(&ServerOptions{
		LocalIdentities: []Identity{
			&spiffeID{"test_server_spiffe_id"},
			&hostname{"test_server_hostname"},
		},
		S2AAddress:       "test_s2a_address",
		EnableLegacyMode: true,
	})
	if err != nil {
		t.Fatalf("NewServerCreds(&ServerOptions{}) failed: %v", err)
	}
	cc := c.Clone()
	s2aCreds, ok := c.(*s2aTransportCreds)
	if !ok {
		t.Fatal("The created creds is not of type s2aTransportCreds")
	}
	s2aCloneCreds, ok := cc.(*s2aTransportCreds)
	if !ok {
		t.Fatal("The created cloned creds is not of type s2aTransportCreds")
	}
	if got, want := cmp.Equal(s2aCreds, s2aCloneCreds, protocmp.Transform(), cmp.AllowUnexported(s2aTransportCreds{})), true; got != want {
		t.Errorf("cmp.Equal(%v, %v) = %v, want %v", s2aCreds, s2aCloneCreds, got, want)
	}
	// Change the values and verify that the creds were deep copied.
	s2aCloneCreds.localIdentities[0] = &s2apb.Identity{
		IdentityOneof: &s2apb.Identity_SpiffeId{
			SpiffeId: "new_spiffe_id",
		},
	}
	if got, want := cmp.Equal(s2aCreds, s2aCloneCreds, protocmp.Transform(), cmp.AllowUnexported(s2aTransportCreds{})), false; got != want {
		t.Errorf("cmp.Equal(%v, %v) = %v, want %v", s2aCreds, s2aCloneCreds, got, want)
	}
}

func TestOverrideServerName(t *testing.T) {
	wantServerName := "server.name"
	// This is not testing any handshaker functionality, so it's fine to only
	// use NewServerCreds and not NewClientCreds.
	c, err := NewServerCreds(&ServerOptions{})
	if err != nil {
		t.Fatalf("NewServerCreds(&ServerOptions{}) failed: %v", err)
	}
	if got, want := c.Info().ServerName, ""; got != want {
		t.Errorf("c.Info().ServerName = %v, want %v", got, want)
	}
	if err := c.OverrideServerName(wantServerName); err != nil {
		t.Fatalf("c.OverrideServerName(%v) failed: %v", wantServerName, err)
	}
	if got, want := c.Info().ServerName, wantServerName; got != want {
		t.Errorf("c.Info().ServerName = %v, want %v", got, want)
	}
}

func TestGetVerificationMode(t *testing.T) {
	for _, tc := range []struct {
		description         string
		verificationMode    VerificationModeType
		expVerificationMode s2av2pb.ValidatePeerCertificateChainReq_VerificationMode
	}{
		{
			description:         "connect to google",
			verificationMode:    ConnectToGoogle,
			expVerificationMode: s2av2pb.ValidatePeerCertificateChainReq_CONNECT_TO_GOOGLE,
		},
		{
			description:         "spiffe",
			verificationMode:    Spiffe,
			expVerificationMode: s2av2pb.ValidatePeerCertificateChainReq_SPIFFE,
		},
		{
			description:         "unspecified",
			verificationMode:    Unspecified,
			expVerificationMode: s2av2pb.ValidatePeerCertificateChainReq_UNSPECIFIED,
		},
	} {
		t.Run(tc.description, func(t *testing.T) {
			if got, want := getVerificationMode(tc.verificationMode), tc.expVerificationMode; got != want {
				t.Errorf("got = %v, want = %v", got, want)
			}
		})
	}
}

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

package authinfo

import (
	"bytes"
	"testing"

	"google.golang.org/grpc/credentials"

	commonpb "github.com/google/s2a-go/internal/proto/common_go_proto"
	grpcpb "github.com/google/s2a-go/internal/proto/s2a_go_proto"
)

func TestS2AAuthInfo(t *testing.T) {
	for _, tc := range []struct {
		desc                    string
		sessionResult           *grpcpb.SessionResult
		outAppProtocol          string
		outTLSVersion           commonpb.TLSVersion
		outCiphersuite          commonpb.Ciphersuite
		outPeerIdentity         *commonpb.Identity
		outLocalIdentity        *commonpb.Identity
		outPeerCertFingerprint  []byte
		outLocalCertFingerprint []byte
		outIsHandshakeResumed   bool
		outErr                  bool
	}{
		{
			desc: "basic 1",
			sessionResult: &grpcpb.SessionResult{
				ApplicationProtocol: "app protocol",
				State: &grpcpb.SessionState{
					TlsVersion:         commonpb.TLSVersion_TLS1_3,
					TlsCiphersuite:     commonpb.Ciphersuite_AES_128_GCM_SHA256,
					IsHandshakeResumed: true,
				},
				PeerIdentity: &commonpb.Identity{
					IdentityOneof: &commonpb.Identity_SpiffeId{
						SpiffeId: "peer spiffe identity",
					},
				},
				LocalIdentity: &commonpb.Identity{
					IdentityOneof: &commonpb.Identity_Hostname{
						Hostname: "local hostname",
					},
				},
				PeerCertFingerprint:  []byte("peer cert fingerprint"),
				LocalCertFingerprint: []byte("local cert fingerprint"),
			},
			outAppProtocol: "app protocol",
			outTLSVersion:  commonpb.TLSVersion_TLS1_3,
			outCiphersuite: commonpb.Ciphersuite_AES_128_GCM_SHA256,
			outPeerIdentity: &commonpb.Identity{
				IdentityOneof: &commonpb.Identity_SpiffeId{
					SpiffeId: "peer spiffe identity",
				},
			},
			outLocalIdentity: &commonpb.Identity{
				IdentityOneof: &commonpb.Identity_Hostname{
					Hostname: "local hostname",
				},
			},
			outPeerCertFingerprint:  []byte("peer cert fingerprint"),
			outLocalCertFingerprint: []byte("local cert fingerprint"),
			outIsHandshakeResumed:   true,
		},
		{
			desc: "basic 2",
			sessionResult: &grpcpb.SessionResult{
				ApplicationProtocol: "app protocol",
				State: &grpcpb.SessionState{
					TlsVersion:     commonpb.TLSVersion_TLS1_2,
					TlsCiphersuite: commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
				},
				PeerIdentity: &commonpb.Identity{
					IdentityOneof: &commonpb.Identity_Hostname{
						Hostname: "local hostname",
					},
				},
				LocalIdentity: &commonpb.Identity{
					IdentityOneof: &commonpb.Identity_SpiffeId{
						SpiffeId: "peer spiffe identity",
					},
				},
				PeerCertFingerprint:  []byte("peer cert fingerprint"),
				LocalCertFingerprint: []byte("local cert fingerprint"),
			},
			outAppProtocol: "app protocol",
			outTLSVersion:  commonpb.TLSVersion_TLS1_2,
			outCiphersuite: commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			outPeerIdentity: &commonpb.Identity{
				IdentityOneof: &commonpb.Identity_Hostname{
					Hostname: "local hostname",
				},
			},
			outLocalIdentity: &commonpb.Identity{
				IdentityOneof: &commonpb.Identity_SpiffeId{
					SpiffeId: "peer spiffe identity",
				},
			},
			outPeerCertFingerprint:  []byte("peer cert fingerprint"),
			outLocalCertFingerprint: []byte("local cert fingerprint"),
		},
		{
			desc: "nil identities and fingerprints",
			sessionResult: &grpcpb.SessionResult{
				ApplicationProtocol: "app protocol",
				State: &grpcpb.SessionState{
					TlsVersion:     commonpb.TLSVersion_TLS1_3,
					TlsCiphersuite: commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
				},
			},
			outAppProtocol: "app protocol",
			outTLSVersion:  commonpb.TLSVersion_TLS1_3,
			outCiphersuite: commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
		},
		{
			desc:   "nil session result",
			outErr: true,
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			authInfo, err := newS2AAuthInfo(tc.sessionResult)
			if got, want := err == nil, !tc.outErr; got != want {
				t.Errorf("NewS2AAuthInfo(%v) = (err=nil) = %v, want %v", tc.sessionResult, got, want)
			}
			if err == nil {
				if got, want := authInfo.AuthType(), s2aAuthType; got != want {
					t.Errorf("authInfo.AuthType() = %v, want %v", got, want)
				}
				if got, want := authInfo.ApplicationProtocol(), tc.outAppProtocol; got != want {
					t.Errorf("authInfo.ApplicationProtocol() = %v, want %v", got, want)
				}
				if got, want := authInfo.TLSVersion(), tc.outTLSVersion; got != want {
					t.Errorf("authInfo.TLSVersion() = %v, want %v", got, want)
				}
				if got, want := authInfo.Ciphersuite(), tc.outCiphersuite; got != want {
					t.Errorf("authInfo.Ciphersuite() = %v, want %v", got, want)
				}
				if got, want := authInfo.PeerIdentity().String(), tc.outPeerIdentity.String(); got != want {
					t.Errorf("authInfo.PeerIdentity() = %v, want %v", got, want)
				}
				if got, want := authInfo.LocalIdentity().String(), tc.outLocalIdentity.String(); got != want {
					t.Errorf("authInfo.LocalIdentity() = %v, want %v", got, want)
				}
				if got, want := authInfo.PeerCertFingerprint(), tc.outPeerCertFingerprint; !bytes.Equal(got, want) {
					t.Errorf("authinfo.PeerCertFingerprint() = %v, want %v", got, want)
				}
				if got, want := authInfo.LocalCertFingerprint(), tc.outLocalCertFingerprint; !bytes.Equal(got, want) {
					t.Errorf("authinfo.LocalCertFingerprint() = %v, want %v", got, want)
				}
				if got, want := authInfo.IsHandshakeResumed(), tc.outIsHandshakeResumed; got != want {
					t.Errorf("authinfo.IsHandshakeResumed() = %v, want %v", got, want)
				}
				if got, want := authInfo.SecurityLevel(), credentials.PrivacyAndIntegrity; got != want {
					t.Errorf("authInfo.SecurityLevel() = %v, want %v", got, want)
				}
			}
		})
	}
}

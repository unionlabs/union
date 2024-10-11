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

	commonpb "github.com/google/s2a-go/internal/proto/common_go_proto"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/peer"
)

func TestAuthInfoFromContext(t *testing.T) {
	ctx := context.Background()
	s2aAuthInfo := &fakeS2AAuthInfo{}
	p := &peer.Peer{
		AuthInfo: s2aAuthInfo,
	}
	for _, tc := range []struct {
		desc    string
		ctx     context.Context
		success bool
		out     AuthInfo
	}{
		{
			"working case",
			peer.NewContext(ctx, p),
			true,
			s2aAuthInfo,
		},
	} {
		authInfo, err := AuthInfoFromContext(tc.ctx)
		if got, want := (err == nil), tc.success; got != want {
			t.Errorf("%v: AuthInfoFromContext(_)=(err=nil)=%v, want %v", tc.desc, got, want)
		}
		if got, want := authInfo, tc.out; got != want {
			t.Errorf("%v:, AuthInfoFromContext(_)=(%v, _), want (%v, _)", tc.desc, got, want)
		}
	}
}

func TestAuthInfoFromPeer(t *testing.T) {
	s2aAuthInfo := &fakeS2AAuthInfo{}
	p := &peer.Peer{
		AuthInfo: s2aAuthInfo,
	}
	for _, tc := range []struct {
		desc    string
		p       *peer.Peer
		success bool
		out     AuthInfo
	}{
		{
			"working case",
			p,
			true,
			s2aAuthInfo,
		},
	} {
		authInfo, err := AuthInfoFromPeer(tc.p)
		if got, want := (err == nil), tc.success; got != want {
			t.Errorf("%v: AuthInfoFromPeer(_)=(err=nil)=%v, want %v", tc.desc, got, want)
		}
		if got, want := authInfo, tc.out; got != want {
			t.Errorf("%v:, AuthInfoFromPeer(_)=(%v, _), want (%v, _)", tc.desc, got, want)
		}
	}
}

type fakeS2AAuthInfo struct{}

func (*fakeS2AAuthInfo) AuthType() string                { return "" }
func (*fakeS2AAuthInfo) ApplicationProtocol() string     { return "" }
func (*fakeS2AAuthInfo) TLSVersion() commonpb.TLSVersion { return commonpb.TLSVersion_TLS1_3 }
func (*fakeS2AAuthInfo) Ciphersuite() commonpb.Ciphersuite {
	return commonpb.Ciphersuite_AES_128_GCM_SHA256
}
func (*fakeS2AAuthInfo) PeerIdentity() *commonpb.Identity  { return nil }
func (*fakeS2AAuthInfo) LocalIdentity() *commonpb.Identity { return nil }
func (*fakeS2AAuthInfo) PeerCertFingerprint() []byte       { return nil }
func (*fakeS2AAuthInfo) LocalCertFingerprint() []byte      { return nil }
func (*fakeS2AAuthInfo) IsHandshakeResumed() bool          { return false }
func (*fakeS2AAuthInfo) SecurityLevel() credentials.SecurityLevel {
	return credentials.PrivacyAndIntegrity
}

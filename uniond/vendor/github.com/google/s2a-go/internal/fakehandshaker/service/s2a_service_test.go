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

package service

import (
	"errors"
	"os"
	"strings"
	"testing"

	"github.com/google/go-cmp/cmp"
	commonpb "github.com/google/s2a-go/internal/proto/common_go_proto"
	s2apb "github.com/google/s2a-go/internal/proto/s2a_go_proto"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/protobuf/testing/protocmp"
)

const (
	testAccessToken = "test_access_token"
)

type fakeS2ASetupSessionServer struct {
	grpc.ServerStream
	recvCount int
	reqs      []*s2apb.SessionReq
	resps     []*s2apb.SessionResp
}

func (f *fakeS2ASetupSessionServer) Send(resp *s2apb.SessionResp) error {
	f.resps = append(f.resps, resp)
	return nil
}

func (f *fakeS2ASetupSessionServer) Recv() (*s2apb.SessionReq, error) {
	if f.recvCount == len(f.reqs) {
		return nil, errors.New("request buffer was fully exhausted")
	}
	req := f.reqs[f.recvCount]
	f.recvCount++
	return req, nil
}

func TestSetupSession(t *testing.T) {
	os.Setenv(accessTokenEnvVariable, "")
	for _, tc := range []struct {
		desc string
		// Note that outResps[i] is the output for reqs[i].
		reqs           []*s2apb.SessionReq
		outResps       []*s2apb.SessionResp
		hasNonOKStatus bool
	}{
		{
			desc: "client failure no app protocols",
			reqs: []*s2apb.SessionReq{
				{
					ReqOneof: &s2apb.SessionReq_ClientStart{
						ClientStart: &s2apb.ClientSessionStartReq{},
					},
				},
			},
			hasNonOKStatus: true,
		},
		{
			desc: "client failure non initial state",
			reqs: []*s2apb.SessionReq{
				{
					ReqOneof: &s2apb.SessionReq_ClientStart{
						ClientStart: &s2apb.ClientSessionStartReq{
							ApplicationProtocols: []string{grpcAppProtocol},
							MinTlsVersion:        commonpb.TLSVersion_TLS1_3,
							MaxTlsVersion:        commonpb.TLSVersion_TLS1_3,
							TlsCiphersuites: []commonpb.Ciphersuite{
								commonpb.Ciphersuite_AES_128_GCM_SHA256,
								commonpb.Ciphersuite_AES_256_GCM_SHA384,
								commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
							},
						},
					},
				},
				{
					ReqOneof: &s2apb.SessionReq_ClientStart{
						ClientStart: &s2apb.ClientSessionStartReq{
							ApplicationProtocols: []string{grpcAppProtocol},
							MinTlsVersion:        commonpb.TLSVersion_TLS1_3,
							MaxTlsVersion:        commonpb.TLSVersion_TLS1_3,
							TlsCiphersuites: []commonpb.Ciphersuite{
								commonpb.Ciphersuite_AES_128_GCM_SHA256,
								commonpb.Ciphersuite_AES_256_GCM_SHA384,
								commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
							},
						},
					},
				},
			},
			outResps: []*s2apb.SessionResp{
				{
					OutFrames: []byte(clientHelloFrame),
					Status: &s2apb.SessionStatus{
						Code: uint32(codes.OK),
					},
				},
			},
			hasNonOKStatus: true,
		},
		{
			desc: "client test",
			reqs: []*s2apb.SessionReq{
				{
					ReqOneof: &s2apb.SessionReq_ClientStart{
						ClientStart: &s2apb.ClientSessionStartReq{
							ApplicationProtocols: []string{grpcAppProtocol},
							MinTlsVersion:        commonpb.TLSVersion_TLS1_3,
							MaxTlsVersion:        commonpb.TLSVersion_TLS1_3,
							TlsCiphersuites: []commonpb.Ciphersuite{
								commonpb.Ciphersuite_AES_128_GCM_SHA256,
								commonpb.Ciphersuite_AES_256_GCM_SHA384,
								commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
							},
							LocalIdentity: &commonpb.Identity{
								IdentityOneof: &commonpb.Identity_Hostname{Hostname: "local hostname"},
							},
							TargetIdentities: []*commonpb.Identity{
								{
									IdentityOneof: &commonpb.Identity_SpiffeId{SpiffeId: "peer spiffe identity"},
								},
							},
						},
					},
				},
				{
					ReqOneof: &s2apb.SessionReq_Next{
						Next: &s2apb.SessionNextReq{
							InBytes: []byte(serverFrame),
						},
					},
				},
			},
			outResps: []*s2apb.SessionResp{
				{
					OutFrames: []byte(clientHelloFrame),
					Status: &s2apb.SessionStatus{
						Code: uint32(codes.OK),
					},
				},
				{
					OutFrames:     []byte(clientFinishedFrame),
					BytesConsumed: uint32(len(serverFrame)),
					Result: &s2apb.SessionResult{
						ApplicationProtocol: grpcAppProtocol,
						State: &s2apb.SessionState{
							TlsVersion:     commonpb.TLSVersion_TLS1_3,
							TlsCiphersuite: commonpb.Ciphersuite_AES_128_GCM_SHA256,
							InKey:          []byte(inKey),
							OutKey:         []byte(outKey),
						},
						PeerIdentity: &commonpb.Identity{
							IdentityOneof: &commonpb.Identity_SpiffeId{SpiffeId: "peer spiffe identity"},
						},
						LocalIdentity: &commonpb.Identity{
							IdentityOneof: &commonpb.Identity_Hostname{Hostname: "local hostname"},
						},
					},
					Status: &s2apb.SessionStatus{
						Code: uint32(codes.OK),
					},
				},
			},
		},
		{
			desc: "server failure no app protocols",
			reqs: []*s2apb.SessionReq{
				{
					ReqOneof: &s2apb.SessionReq_ServerStart{
						ServerStart: &s2apb.ServerSessionStartReq{},
					},
				},
			},
			hasNonOKStatus: true,
		},
		{
			desc: "server failure non initial state",
			reqs: []*s2apb.SessionReq{
				{
					ReqOneof: &s2apb.SessionReq_ServerStart{
						ServerStart: &s2apb.ServerSessionStartReq{
							ApplicationProtocols: []string{grpcAppProtocol},
							MinTlsVersion:        commonpb.TLSVersion_TLS1_3,
							MaxTlsVersion:        commonpb.TLSVersion_TLS1_3,
							TlsCiphersuites: []commonpb.Ciphersuite{
								commonpb.Ciphersuite_AES_128_GCM_SHA256,
								commonpb.Ciphersuite_AES_256_GCM_SHA384,
								commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
							},
						},
					},
				},
				{
					ReqOneof: &s2apb.SessionReq_ServerStart{
						ServerStart: &s2apb.ServerSessionStartReq{
							ApplicationProtocols: []string{grpcAppProtocol},
							MinTlsVersion:        commonpb.TLSVersion_TLS1_3,
							MaxTlsVersion:        commonpb.TLSVersion_TLS1_3,
							TlsCiphersuites: []commonpb.Ciphersuite{
								commonpb.Ciphersuite_AES_128_GCM_SHA256,
								commonpb.Ciphersuite_AES_256_GCM_SHA384,
								commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
							},
						},
					},
				},
			},
			outResps: []*s2apb.SessionResp{
				{
					Status: &s2apb.SessionStatus{
						Code: uint32(codes.OK),
					},
				},
			},
			hasNonOKStatus: true,
		},
		{
			desc: "server test",
			reqs: []*s2apb.SessionReq{
				{
					ReqOneof: &s2apb.SessionReq_ServerStart{
						ServerStart: &s2apb.ServerSessionStartReq{
							ApplicationProtocols: []string{grpcAppProtocol},
							MinTlsVersion:        commonpb.TLSVersion_TLS1_3,
							MaxTlsVersion:        commonpb.TLSVersion_TLS1_3,
							TlsCiphersuites: []commonpb.Ciphersuite{
								commonpb.Ciphersuite_AES_128_GCM_SHA256,
								commonpb.Ciphersuite_AES_256_GCM_SHA384,
								commonpb.Ciphersuite_CHACHA20_POLY1305_SHA256,
							},
							InBytes: []byte(clientHelloFrame),
							LocalIdentities: []*commonpb.Identity{
								{
									IdentityOneof: &commonpb.Identity_Hostname{Hostname: "local hostname"},
								},
							},
						},
					},
				},
				{
					ReqOneof: &s2apb.SessionReq_Next{
						Next: &s2apb.SessionNextReq{
							InBytes: []byte(clientFinishedFrame),
						},
					},
				},
			},
			outResps: []*s2apb.SessionResp{
				{
					OutFrames:     []byte(serverFrame),
					BytesConsumed: uint32(len(clientHelloFrame)),
					Status: &s2apb.SessionStatus{
						Code: uint32(codes.OK),
					},
				},
				{
					BytesConsumed: uint32(len(clientFinishedFrame)),
					Result: &s2apb.SessionResult{
						ApplicationProtocol: grpcAppProtocol,
						State: &s2apb.SessionState{
							TlsVersion:     commonpb.TLSVersion_TLS1_3,
							TlsCiphersuite: commonpb.Ciphersuite_AES_128_GCM_SHA256,
							InKey:          []byte(inKey),
							OutKey:         []byte(outKey),
						},
						LocalIdentity: &commonpb.Identity{
							IdentityOneof: &commonpb.Identity_Hostname{Hostname: "local hostname"},
						},
					},
					Status: &s2apb.SessionStatus{
						Code: uint32(codes.OK),
					},
				},
			},
		},
		{
			desc: "resumption ticket test",
			reqs: []*s2apb.SessionReq{
				{
					ReqOneof: &s2apb.SessionReq_ResumptionTicket{
						ResumptionTicket: &s2apb.ResumptionTicketReq{
							ConnectionId: 1234,
							LocalIdentity: &commonpb.Identity{
								IdentityOneof: &commonpb.Identity_Hostname{Hostname: "local hostname"},
							},
						},
					},
				},
			},
			outResps: []*s2apb.SessionResp{
				{
					Status: &s2apb.SessionStatus{
						Code: uint32(codes.OK),
					},
				},
			},
			hasNonOKStatus: false,
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			hs := FakeHandshakerService{}
			stream := &fakeS2ASetupSessionServer{reqs: tc.reqs}
			if got, want := hs.SetUpSession(stream) == nil, !tc.hasNonOKStatus; got != want {
				t.Errorf("hs.SetUpSession(%v) = (err=nil) = %v, want %v", stream, got, want)
			}
			hasNonOKStatus := false
			for i := range tc.reqs {
				if stream.resps[i].GetStatus().GetCode() != uint32(codes.OK) {
					hasNonOKStatus = true
					break
				}
				if got, want := stream.resps[i], tc.outResps[i]; !cmp.Equal(got, want, protocmp.Transform()) {
					t.Fatalf("stream.resps[%d] = %v, want %v", i, got, want)
				}
			}
			if got, want := hasNonOKStatus, tc.hasNonOKStatus; got != want {
				t.Errorf("hasNonOKStatus = %v, want %v", got, want)
			}
		})
	}
}

func TestAuthenticateRequest(t *testing.T) {
	for _, tc := range []struct {
		description   string
		acceptedToken string
		request       *s2apb.SessionReq
		expectedError string
	}{
		{
			description: "access token env variable is not set",
		},
		{
			description:   "request contains valid token",
			acceptedToken: testAccessToken,
			request: &s2apb.SessionReq{
				AuthMechanisms: []*s2apb.AuthenticationMechanism{
					{
						MechanismOneof: &s2apb.AuthenticationMechanism_Token{
							Token: testAccessToken,
						},
					},
				},
			},
		},
		{
			description:   "request contains invalid token",
			acceptedToken: testAccessToken,
			request: &s2apb.SessionReq{
				AuthMechanisms: []*s2apb.AuthenticationMechanism{
					{
						MechanismOneof: &s2apb.AuthenticationMechanism_Token{
							Token: "bad_access_token",
						},
					},
				},
			},
			expectedError: "received token: bad_access_token, expected token: test_access_token",
		},
		{
			description:   "request contains valid and invalid tokens",
			acceptedToken: testAccessToken,
			request: &s2apb.SessionReq{
				AuthMechanisms: []*s2apb.AuthenticationMechanism{
					{
						MechanismOneof: &s2apb.AuthenticationMechanism_Token{
							Token: testAccessToken,
						},
					},
					{
						MechanismOneof: &s2apb.AuthenticationMechanism_Token{
							Token: "bad_access_token",
						},
					},
				},
			},
			expectedError: "received token: bad_access_token, expected token: test_access_token",
		},
	} {
		t.Run(tc.description, func(t *testing.T) {
			os.Setenv(accessTokenEnvVariable, tc.acceptedToken)
			hs := &FakeHandshakerService{}
			err := hs.authenticateRequest(tc.request)
			if got, want := (err == nil), (tc.expectedError == ""); got != want {
				t.Errorf("(err == nil): %t, (tc.expectedError == \"\"): %t", got, want)
			}
			if err != nil && !strings.Contains(err.Error(), tc.expectedError) {
				t.Errorf("hs.authenticateRequest(%v)=%v, expected error to have substring: %v", tc.request, err, tc.expectedError)
			}
		})
	}
}

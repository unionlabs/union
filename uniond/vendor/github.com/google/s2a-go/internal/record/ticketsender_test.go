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

package record

import (
	"errors"
	"fmt"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	commonpb "github.com/google/s2a-go/internal/proto/common_go_proto"
	s2apb "github.com/google/s2a-go/internal/proto/s2a_go_proto"
	"github.com/google/s2a-go/internal/tokenmanager"
	"google.golang.org/grpc/codes"
	"google.golang.org/protobuf/testing/protocmp"
)

const (
	testAccessToken = "test_access_token"
)

type fakeStream struct {
	// returnInvalid is a flag indicating whether the return status of Recv is
	// OK or not.
	returnInvalid bool
	// returnRecvErr is a flag indicating whether an error should be returned by
	// Recv.
	returnRecvErr bool
}

func (fs *fakeStream) Send(req *s2apb.SessionReq) error {
	if len(req.GetResumptionTicket().GetInBytes()) == 0 {
		return errors.New("fakeStream Send received an empty InBytes")
	}
	if req.GetResumptionTicket().GetConnectionId() == 0 {
		return errors.New("fakeStream Send received a 0 ConnectionId")
	}
	if req.GetResumptionTicket().GetLocalIdentity() == nil {
		return errors.New("fakeStream Send received an empty LocalIdentity")
	}
	return nil
}

func (fs *fakeStream) Recv() (*s2apb.SessionResp, error) {
	if fs.returnRecvErr {
		return nil, errors.New("fakeStream Recv error")
	}
	if fs.returnInvalid {
		return &s2apb.SessionResp{
			Status: &s2apb.SessionStatus{Code: uint32(codes.InvalidArgument)},
		}, nil
	}
	return &s2apb.SessionResp{
		Status: &s2apb.SessionStatus{Code: uint32(codes.OK)},
	}, nil
}

type fakeAccessTokenManager struct {
	acceptedIdentity   *commonpb.Identity
	accessToken        string
	allowEmptyIdentity bool
}

func (m *fakeAccessTokenManager) DefaultToken() (string, error) {
	if !m.allowEmptyIdentity {
		return "", fmt.Errorf("not allowed to get token for empty identity")
	}
	return m.accessToken, nil
}

func (m *fakeAccessTokenManager) Token(identity *commonpb.Identity) (string, error) {
	if identity == nil || cmp.Equal(identity, &commonpb.Identity{}, protocmp.Transform()) {
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

func TestWriteTicketsToStream(t *testing.T) {
	for _, tc := range []struct {
		returnInvalid   bool
		returnRecvError bool
	}{
		{
			// Both flags are set to false.
		},
		{
			returnInvalid: true,
		},
		{
			returnRecvError: true,
		},
	} {
		sender := ticketSender{
			connectionID: 1,
			localIdentity: &commonpb.Identity{
				IdentityOneof: &commonpb.Identity_SpiffeId{
					SpiffeId: "test_spiffe_id",
				},
			},
		}
		fs := &fakeStream{returnInvalid: tc.returnInvalid, returnRecvErr: tc.returnRecvError}
		if got, want := sender.writeTicketsToStream(fs, make([][]byte, 1)) == nil, !tc.returnRecvError && !tc.returnInvalid; got != want {
			t.Errorf("sender.writeTicketsToStream(%v, _) = (err=nil) = %v, want %v", fs, got, want)
		}
	}
}

func TestGetAuthMechanism(t *testing.T) {
	sortProtos := cmpopts.SortSlices(func(m1, m2 *s2apb.AuthenticationMechanism) bool { return m1.String() < m2.String() })
	for _, tc := range []struct {
		description            string
		localIdentity          *commonpb.Identity
		tokenManager           tokenmanager.AccessTokenManager
		expectedAuthMechanisms []*s2apb.AuthenticationMechanism
	}{
		{
			description:            "token manager is nil",
			tokenManager:           nil,
			expectedAuthMechanisms: nil,
		},
		{
			description: "token manager expects empty identity",
			tokenManager: &fakeAccessTokenManager{
				accessToken:        testAccessToken,
				allowEmptyIdentity: true,
			},
			expectedAuthMechanisms: []*s2apb.AuthenticationMechanism{
				{
					MechanismOneof: &s2apb.AuthenticationMechanism_Token{
						Token: testAccessToken,
					},
				},
			},
		},
		{
			description: "token manager does not expect empty identity",
			tokenManager: &fakeAccessTokenManager{
				allowEmptyIdentity: false,
			},
			expectedAuthMechanisms: nil,
		},
		{
			description: "token manager expects SPIFFE ID",
			localIdentity: &commonpb.Identity{
				IdentityOneof: &commonpb.Identity_SpiffeId{
					SpiffeId: "allowed_spiffe_id",
				},
			},
			tokenManager: &fakeAccessTokenManager{
				accessToken: testAccessToken,
				acceptedIdentity: &commonpb.Identity{
					IdentityOneof: &commonpb.Identity_SpiffeId{
						SpiffeId: "allowed_spiffe_id",
					},
				},
			},
			expectedAuthMechanisms: []*s2apb.AuthenticationMechanism{
				{
					Identity: &commonpb.Identity{
						IdentityOneof: &commonpb.Identity_SpiffeId{
							SpiffeId: "allowed_spiffe_id",
						},
					},
					MechanismOneof: &s2apb.AuthenticationMechanism_Token{
						Token: testAccessToken,
					},
				},
			},
		},
		{
			description: "token manager does not expect hostname",

			localIdentity: &commonpb.Identity{
				IdentityOneof: &commonpb.Identity_Hostname{
					Hostname: "disallowed_hostname",
				},
			},
			tokenManager:           &fakeAccessTokenManager{},
			expectedAuthMechanisms: nil,
		},
	} {
		t.Run(tc.description, func(t *testing.T) {
			ticketSender := &ticketSender{
				localIdentity: tc.localIdentity,
				tokenManager:  tc.tokenManager,
			}
			authMechanisms := ticketSender.getAuthMechanisms()
			if got, want := (authMechanisms == nil), (tc.expectedAuthMechanisms == nil); got != want {
				t.Errorf("authMechanisms == nil: %t, tc.expectedAuthMechanisms == nil: %t", got, want)
			}
			if authMechanisms != nil && tc.expectedAuthMechanisms != nil {
				if diff := cmp.Diff(authMechanisms, tc.expectedAuthMechanisms, protocmp.Transform(), sortProtos); diff != "" {
					t.Errorf("ticketSender.getAuthMechanisms() returned incorrect slice, (-want +got):\n%s", diff)
				}
			}
		})
	}
}

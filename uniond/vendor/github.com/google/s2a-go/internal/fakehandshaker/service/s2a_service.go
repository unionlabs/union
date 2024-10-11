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

// Package service is a fake S2A handshaker service.
package service

import (
	"bytes"
	"fmt"
	"os"

	"google.golang.org/grpc/codes"

	commonpb "github.com/google/s2a-go/internal/proto/common_go_proto"
	s2apb "github.com/google/s2a-go/internal/proto/s2a_go_proto"
)

type handshakeState int

const (
	// initial is the state of the handshaker service before any handshake
	// message has been received.
	initial handshakeState = 0
	// started is the state of the handshaker service when the handshake has
	// been initiated but no bytes have been sent or received.
	started handshakeState = 1
	// sent is the state of the handshaker service when the handshake has been
	// initiated and bytes have been sent.
	sent handshakeState = 2
	// completed is the state of the handshaker service when the handshake has
	// been completed.
	completed handshakeState = 3
)

const (
	accessTokenEnvVariable = "S2A_ACCESS_TOKEN"
	grpcAppProtocol        = "grpc"
	clientHelloFrame       = "ClientHello"
	clientFinishedFrame    = "ClientFinished"
	serverFrame            = "ServerHelloAndFinished"
)

const (
	inKey  = "kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk"
	outKey = "kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk"
)

// FakeHandshakerService implements the s2apb.S2AServiceServer. The fake
// handshaker service should not be used by more than 1 application at a time.
type FakeHandshakerService struct {
	s2apb.S2AServiceServer

	assistingClient bool
	state           handshakeState
	peerIdentity    *commonpb.Identity
	localIdentity   *commonpb.Identity
}

// SetUpSession sets up the S2A session.
func (hs *FakeHandshakerService) SetUpSession(stream s2apb.S2AService_SetUpSessionServer) error {
	for {
		sessionReq, err := stream.Recv()
		if err != nil {
			return fmt.Errorf("stream recv failed: %v", err)
		}
		if err := hs.authenticateRequest(sessionReq); err != nil {
			return fmt.Errorf("S2A cannot authenticate the request: %v", err)
		}

		var resp *s2apb.SessionResp
		receivedTicket := false
		switch req := sessionReq.ReqOneof.(type) {
		case *s2apb.SessionReq_ClientStart:
			resp = hs.processClientStart(req)
		case *s2apb.SessionReq_ServerStart:
			resp = hs.processServerStart(req)
		case *s2apb.SessionReq_Next:
			resp = hs.processNext(req)
		case *s2apb.SessionReq_ResumptionTicket:
			resp = hs.processResumptionTicket(req)
			receivedTicket = true
		default:
			return fmt.Errorf("session request has unexpected type %T", req)
		}

		if err = stream.Send(resp); err != nil {
			return fmt.Errorf("stream send failed: %v", err)
		}

		if receivedTicket || resp.GetResult() != nil {
			return nil
		}
	}
}

// processClientStart processes a ClientSessionStartReq.
func (hs *FakeHandshakerService) processClientStart(req *s2apb.SessionReq_ClientStart) *s2apb.SessionResp {
	resp := s2apb.SessionResp{}
	if hs.state != initial {
		resp.Status = &s2apb.SessionStatus{
			Code:    uint32(codes.FailedPrecondition),
			Details: "client start handshake not in initial state",
		}
		return &resp
	}
	if len(req.ClientStart.GetApplicationProtocols()) != 1 ||
		req.ClientStart.GetApplicationProtocols()[0] != grpcAppProtocol {
		resp.Status = &s2apb.SessionStatus{
			Code:    uint32(codes.InvalidArgument),
			Details: "application protocol was not grpc",
		}
		return &resp
	}
	if req.ClientStart.GetMaxTlsVersion() != commonpb.TLSVersion_TLS1_3 {
		resp.Status = &s2apb.SessionStatus{
			Code:    uint32(codes.InvalidArgument),
			Details: "max TLS version must be 1.3",
		}
		return &resp
	}
	if req.ClientStart.GetMinTlsVersion() != commonpb.TLSVersion_TLS1_3 {
		resp.Status = &s2apb.SessionStatus{
			Code:    uint32(codes.InvalidArgument),
			Details: "min TLS version must be 1.3",
		}
		return &resp
	}
	resp.OutFrames = []byte(clientHelloFrame)
	resp.BytesConsumed = 0
	resp.Status = &s2apb.SessionStatus{Code: uint32(codes.OK)}
	hs.localIdentity = req.ClientStart.LocalIdentity
	if len(req.ClientStart.TargetIdentities) > 0 {
		hs.peerIdentity = req.ClientStart.TargetIdentities[0]
	}
	hs.assistingClient = true
	hs.state = sent
	return &resp
}

// processServerStart processes a ServerSessionStartReq.
func (hs *FakeHandshakerService) processServerStart(req *s2apb.SessionReq_ServerStart) *s2apb.SessionResp {
	resp := s2apb.SessionResp{}
	if hs.state != initial {
		resp.Status = &s2apb.SessionStatus{
			Code:    uint32(codes.FailedPrecondition),
			Details: "server start handshake not in initial state",
		}
		return &resp
	}
	if len(req.ServerStart.GetApplicationProtocols()) != 1 ||
		req.ServerStart.GetApplicationProtocols()[0] != grpcAppProtocol {
		resp.Status = &s2apb.SessionStatus{
			Code:    uint32(codes.InvalidArgument),
			Details: "application protocol was not grpc",
		}
		return &resp
	}
	if req.ServerStart.GetMaxTlsVersion() != commonpb.TLSVersion_TLS1_3 {
		resp.Status = &s2apb.SessionStatus{
			Code:    uint32(codes.InvalidArgument),
			Details: "max TLS version must be 1.3",
		}
		return &resp
	}
	if req.ServerStart.GetMinTlsVersion() != commonpb.TLSVersion_TLS1_3 {
		resp.Status = &s2apb.SessionStatus{
			Code:    uint32(codes.InvalidArgument),
			Details: "min TLS version must be 1.3",
		}
		return &resp
	}
	if len(req.ServerStart.InBytes) == 0 {
		resp.BytesConsumed = 0
		hs.state = started
	} else if bytes.Equal(req.ServerStart.InBytes, []byte(clientHelloFrame)) {
		resp.OutFrames = []byte(serverFrame)
		resp.BytesConsumed = uint32(len(clientHelloFrame))
		hs.state = sent
	} else {
		resp.Status = &s2apb.SessionStatus{
			Code:    uint32(codes.Internal),
			Details: "server start request did not have the correct input bytes",
		}
		return &resp
	}

	resp.Status = &s2apb.SessionStatus{Code: uint32(codes.OK)}
	if len(req.ServerStart.LocalIdentities) > 0 {
		hs.localIdentity = req.ServerStart.LocalIdentities[0]
	}
	hs.assistingClient = false
	return &resp
}

// processNext processes a SessionNext request.
func (hs *FakeHandshakerService) processNext(req *s2apb.SessionReq_Next) *s2apb.SessionResp {
	resp := s2apb.SessionResp{}
	if hs.assistingClient {
		if hs.state != sent {
			resp.Status = &s2apb.SessionStatus{
				Code:    uint32(codes.FailedPrecondition),
				Details: "client handshake was not in sent state",
			}
			return &resp
		}
		if !bytes.Equal(req.Next.InBytes, []byte(serverFrame)) {
			resp.Status = &s2apb.SessionStatus{
				Code:    uint32(codes.Internal),
				Details: "client request did not match server frame",
			}
			return &resp
		}
		resp.OutFrames = []byte(clientFinishedFrame)
		resp.BytesConsumed = uint32(len(serverFrame))
		hs.state = completed
	} else {
		if hs.state == started {
			if !bytes.Equal(req.Next.InBytes, []byte(clientHelloFrame)) {
				resp.Status = &s2apb.SessionStatus{
					Code:    uint32(codes.Internal),
					Details: "server request did not match client hello frame",
				}
				return &resp
			}
			resp.OutFrames = []byte(serverFrame)
			resp.BytesConsumed = uint32(len(clientHelloFrame))
			hs.state = sent
		} else if hs.state == sent {
			if !bytes.Equal(req.Next.InBytes[:len(clientFinishedFrame)], []byte(clientFinishedFrame)) {
				resp.Status = &s2apb.SessionStatus{
					Code:    uint32(codes.Internal),
					Details: "server request did not match client finished frame",
				}
				return &resp
			}
			resp.BytesConsumed = uint32(len(clientFinishedFrame))
			hs.state = completed
		} else {
			resp.Status = &s2apb.SessionStatus{
				Code:    uint32(codes.FailedPrecondition),
				Details: "server request was not in expected state",
			}
			return &resp
		}
	}
	resp.Status = &s2apb.SessionStatus{Code: uint32(codes.OK)}
	if hs.state == completed {
		resp.Result = hs.getSessionResult()
	}
	return &resp
}

// processResumptionTicket processes a ResumptionTicketReq request.
func (hs *FakeHandshakerService) processResumptionTicket(req *s2apb.SessionReq_ResumptionTicket) *s2apb.SessionResp {
	return &s2apb.SessionResp{
		Status: &s2apb.SessionStatus{Code: uint32(codes.OK)},
	}
}

// getSessionResult returns a dummy SessionResult.
func (hs *FakeHandshakerService) getSessionResult() *s2apb.SessionResult {
	res := s2apb.SessionResult{}
	res.ApplicationProtocol = grpcAppProtocol
	res.State = &s2apb.SessionState{
		TlsVersion:     commonpb.TLSVersion_TLS1_3,
		TlsCiphersuite: commonpb.Ciphersuite_AES_128_GCM_SHA256,
		InKey:          []byte(inKey),
		OutKey:         []byte(outKey),
	}
	res.PeerIdentity = hs.peerIdentity
	res.LocalIdentity = hs.localIdentity
	return &res
}

func (hs *FakeHandshakerService) authenticateRequest(request *s2apb.SessionReq) error {
	// If the S2A_ACCESS_TOKEN environment variable has not been set, then do not
	// enforce anything on the request.
	acceptedToken := os.Getenv(accessTokenEnvVariable)
	if acceptedToken == "" {
		return nil
	}
	if len(request.GetAuthMechanisms()) == 0 {
		return fmt.Errorf("expected token but none was received")
	}
	for _, authMechanism := range request.GetAuthMechanisms() {
		if authMechanism.GetToken() != acceptedToken {
			return fmt.Errorf("received token: %s, expected token: %s", authMechanism.GetToken(), acceptedToken)
		}
	}
	return nil
}

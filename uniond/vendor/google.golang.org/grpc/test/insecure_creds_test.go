/*
 *
 * Copyright 2020 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package test

import (
	"context"
	"net"
	"strings"
	"testing"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/status"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

// testLegacyPerRPCCredentials is a PerRPCCredentials that has yet incorporated security level.
type testLegacyPerRPCCredentials struct{}

func (cr testLegacyPerRPCCredentials) GetRequestMetadata(context.Context, ...string) (map[string]string, error) {
	return nil, nil
}

func (cr testLegacyPerRPCCredentials) RequireTransportSecurity() bool {
	return true
}

func getSecurityLevel(ai credentials.AuthInfo) credentials.SecurityLevel {
	if c, ok := ai.(interface {
		GetCommonAuthInfo() credentials.CommonAuthInfo
	}); ok {
		return c.GetCommonAuthInfo().SecurityLevel
	}
	return credentials.InvalidSecurityLevel
}

// TestInsecureCreds tests the use of insecure creds on the server and client
// side, and verifies that expect security level and auth info are returned.
// Also verifies that this credential can interop with existing `WithInsecure`
// DialOption.
func (s) TestInsecureCreds(t *testing.T) {
	tests := []struct {
		desc                string
		clientInsecureCreds bool
		serverInsecureCreds bool
	}{
		{
			desc:                "client and server insecure creds",
			clientInsecureCreds: true,
			serverInsecureCreds: true,
		},
		{
			desc:                "client only insecure creds",
			clientInsecureCreds: true,
		},
		{
			desc:                "server only insecure creds",
			serverInsecureCreds: true,
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			ss := &stubserver.StubServer{
				EmptyCallF: func(ctx context.Context, _ *testpb.Empty) (*testpb.Empty, error) {
					if !test.serverInsecureCreds {
						return &testpb.Empty{}, nil
					}

					pr, ok := peer.FromContext(ctx)
					if !ok {
						return nil, status.Error(codes.DataLoss, "Failed to get peer from ctx")
					}
					// Check security level.
					secLevel := getSecurityLevel(pr.AuthInfo)
					if secLevel == credentials.InvalidSecurityLevel {
						return nil, status.Errorf(codes.Unauthenticated, "peer.AuthInfo does not implement GetCommonAuthInfo()")
					}
					if secLevel != credentials.NoSecurity {
						return nil, status.Errorf(codes.Unauthenticated, "Wrong security level: got %q, want %q", secLevel, credentials.NoSecurity)
					}
					return &testpb.Empty{}, nil
				},
			}

			sOpts := []grpc.ServerOption{}
			if test.serverInsecureCreds {
				sOpts = append(sOpts, grpc.Creds(insecure.NewCredentials()))
			}
			s := grpc.NewServer(sOpts...)
			defer s.Stop()

			testgrpc.RegisterTestServiceServer(s, ss)

			lis, err := net.Listen("tcp", "localhost:0")
			if err != nil {
				t.Fatalf("net.Listen(tcp, localhost:0) failed: %v", err)
			}

			go s.Serve(lis)

			addr := lis.Addr().String()
			opts := []grpc.DialOption{grpc.WithTransportCredentials(insecure.NewCredentials())}
			if test.clientInsecureCreds {
				opts = []grpc.DialOption{grpc.WithTransportCredentials(insecure.NewCredentials())}
			}
			cc, err := grpc.NewClient(addr, opts...)
			if err != nil {
				t.Fatalf("grpc.NewClient(%q) failed: %v", addr, err)
			}
			defer cc.Close()

			c := testgrpc.NewTestServiceClient(cc)
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			if _, err = c.EmptyCall(ctx, &testpb.Empty{}); err != nil {
				t.Fatalf("EmptyCall(_, _) = _, %v; want _, <nil>", err)
			}
		})
	}
}

func (s) TestInsecureCreds_WithPerRPCCredentials_AsCallOption(t *testing.T) {
	ss := &stubserver.StubServer{
		EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) {
			return &testpb.Empty{}, nil
		},
	}

	s := grpc.NewServer(grpc.Creds(insecure.NewCredentials()))
	defer s.Stop()
	testgrpc.RegisterTestServiceServer(s, ss)

	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("net.Listen(tcp, localhost:0) failed: %v", err)
	}
	go s.Serve(lis)

	addr := lis.Addr().String()
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	dopts := []grpc.DialOption{grpc.WithTransportCredentials(insecure.NewCredentials())}
	copts := []grpc.CallOption{grpc.PerRPCCredentials(testLegacyPerRPCCredentials{})}
	cc, err := grpc.NewClient(addr, dopts...)
	if err != nil {
		t.Fatalf("grpc.NewClient(%q) failed: %v", addr, err)
	}
	defer cc.Close()

	const wantErr = "transport: cannot send secure credentials on an insecure connection"
	c := testgrpc.NewTestServiceClient(cc)
	if _, err = c.EmptyCall(ctx, &testpb.Empty{}, copts...); err == nil || !strings.Contains(err.Error(), wantErr) {
		t.Fatalf("insecure credentials with per-RPC credentials requiring transport security returned error: %v; want %s", err, wantErr)
	}
}

func (s) TestInsecureCreds_WithPerRPCCredentials_AsDialOption(t *testing.T) {
	ss := &stubserver.StubServer{
		EmptyCallF: func(_ context.Context, _ *testpb.Empty) (*testpb.Empty, error) {
			return &testpb.Empty{}, nil
		},
	}

	s := grpc.NewServer(grpc.Creds(insecure.NewCredentials()))
	defer s.Stop()
	testgrpc.RegisterTestServiceServer(s, ss)

	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("net.Listen(tcp, localhost:0) failed: %v", err)
	}
	go s.Serve(lis)

	addr := lis.Addr().String()
	dopts := []grpc.DialOption{
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithPerRPCCredentials(testLegacyPerRPCCredentials{}),
	}
	const wantErr = "the credentials require transport level security"
	if _, err := grpc.NewClient(addr, dopts...); err == nil || !strings.Contains(err.Error(), wantErr) {
		t.Fatalf("grpc.NewClient(%q) returned err %v, want: %v", addr, err, wantErr)
	}
}

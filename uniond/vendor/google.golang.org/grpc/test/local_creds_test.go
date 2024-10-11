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
	"fmt"
	"net"
	"strings"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/credentials/local"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/status"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

func testLocalCredsE2ESucceed(network, address string) error {
	ss := &stubserver.StubServer{
		EmptyCallF: func(ctx context.Context, _ *testpb.Empty) (*testpb.Empty, error) {
			pr, ok := peer.FromContext(ctx)
			if !ok {
				return nil, status.Error(codes.DataLoss, "Failed to get peer from ctx")
			}
			type internalInfo interface {
				GetCommonAuthInfo() credentials.CommonAuthInfo
			}
			var secLevel credentials.SecurityLevel
			if info, ok := (pr.AuthInfo).(internalInfo); ok {
				secLevel = info.GetCommonAuthInfo().SecurityLevel
			} else {
				return nil, status.Errorf(codes.Unauthenticated, "peer.AuthInfo does not implement GetCommonAuthInfo()")
			}
			// Check security level
			switch network {
			case "unix":
				if secLevel != credentials.PrivacyAndIntegrity {
					return nil, status.Errorf(codes.Unauthenticated, "Wrong security level: got %q, want %q", secLevel, credentials.PrivacyAndIntegrity)
				}
			case "tcp":
				if secLevel != credentials.NoSecurity {
					return nil, status.Errorf(codes.Unauthenticated, "Wrong security level: got %q, want %q", secLevel, credentials.NoSecurity)
				}
			}
			return &testpb.Empty{}, nil
		},
	}

	sopts := []grpc.ServerOption{grpc.Creds(local.NewCredentials())}
	s := grpc.NewServer(sopts...)
	defer s.Stop()

	testgrpc.RegisterTestServiceServer(s, ss)

	lis, err := net.Listen(network, address)
	if err != nil {
		return fmt.Errorf("Failed to create listener: %v", err)
	}

	go s.Serve(lis)

	var cc *grpc.ClientConn
	lisAddr := lis.Addr().String()

	switch network {
	case "unix":
		cc, err = grpc.Dial(lisAddr, grpc.WithTransportCredentials(local.NewCredentials()), grpc.WithContextDialer(
			func(_ context.Context, addr string) (net.Conn, error) {
				return net.Dial("unix", addr)
			}))
	case "tcp":
		cc, err = grpc.NewClient(lisAddr, grpc.WithTransportCredentials(local.NewCredentials()))
	default:
		return fmt.Errorf("unsupported network %q", network)
	}
	if err != nil {
		return fmt.Errorf("Failed to dial server: %v, %v", err, lisAddr)
	}
	defer cc.Close()

	c := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	if _, err = c.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		return fmt.Errorf("EmptyCall(_, _) = _, %v; want _, <nil>", err)
	}
	return nil
}

func (s) TestLocalCredsLocalhost(t *testing.T) {
	if err := testLocalCredsE2ESucceed("tcp", "localhost:0"); err != nil {
		t.Fatalf("Failed e2e test for localhost: %v", err)
	}
}

func (s) TestLocalCredsUDS(t *testing.T) {
	addr := fmt.Sprintf("/tmp/grpc_fullstck_test%d", time.Now().UnixNano())
	if err := testLocalCredsE2ESucceed("unix", addr); err != nil {
		t.Fatalf("Failed e2e test for UDS: %v", err)
	}
}

type connWrapper struct {
	net.Conn
	remote net.Addr
}

func (c connWrapper) RemoteAddr() net.Addr {
	return c.remote
}

type lisWrapper struct {
	net.Listener
	remote net.Addr
}

func spoofListener(l net.Listener, remote net.Addr) net.Listener {
	return &lisWrapper{l, remote}
}

func (l *lisWrapper) Accept() (net.Conn, error) {
	c, err := l.Listener.Accept()
	if err != nil {
		return nil, err
	}
	return connWrapper{c, l.remote}, nil
}

func spoofDialer(addr net.Addr) func(target string, t time.Duration) (net.Conn, error) {
	return func(t string, d time.Duration) (net.Conn, error) {
		c, err := net.DialTimeout("tcp", t, d)
		if err != nil {
			return nil, err
		}
		return connWrapper{c, addr}, nil
	}
}

func testLocalCredsE2EFail(dopts []grpc.DialOption) error {
	ss := &stubserver.StubServer{
		EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) {
			return &testpb.Empty{}, nil
		},
	}

	sopts := []grpc.ServerOption{grpc.Creds(local.NewCredentials())}
	s := grpc.NewServer(sopts...)
	defer s.Stop()

	testgrpc.RegisterTestServiceServer(s, ss)

	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		return fmt.Errorf("Failed to create listener: %v", err)
	}

	var fakeClientAddr, fakeServerAddr net.Addr
	fakeClientAddr = &net.IPAddr{
		IP:   net.ParseIP("10.8.9.10"),
		Zone: "",
	}
	fakeServerAddr = &net.IPAddr{
		IP:   net.ParseIP("10.8.9.11"),
		Zone: "",
	}

	go s.Serve(spoofListener(lis, fakeClientAddr))

	cc, err := grpc.NewClient(lis.Addr().String(), append(dopts, grpc.WithDialer(spoofDialer(fakeServerAddr)))...)
	if err != nil {
		return fmt.Errorf("Failed to dial server: %v, %v", err, lis.Addr().String())
	}
	defer cc.Close()

	c := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	_, err = c.EmptyCall(ctx, &testpb.Empty{})
	return err
}

func isExpected(got, want error) bool {
	return status.Code(got) == status.Code(want) && strings.Contains(status.Convert(got).Message(), status.Convert(want).Message())
}

func (s) TestLocalCredsClientFail(t *testing.T) {
	// Use local creds at client-side which should lead to client-side failure.
	opts := []grpc.DialOption{grpc.WithTransportCredentials(local.NewCredentials())}
	want := status.Error(codes.Unavailable, "transport: authentication handshake failed: local credentials rejected connection to non-local address")
	if err := testLocalCredsE2EFail(opts); !isExpected(err, want) {
		t.Fatalf("testLocalCredsE2EFail() = %v; want %v", err, want)
	}
}

func (s) TestLocalCredsServerFail(t *testing.T) {
	// Use insecure at client-side which should lead to server-side failure.
	opts := []grpc.DialOption{grpc.WithTransportCredentials(insecure.NewCredentials())}
	if err := testLocalCredsE2EFail(opts); status.Code(err) != codes.Unavailable {
		t.Fatalf("testLocalCredsE2EFail() = %v; want %v", err, codes.Unavailable)
	}
}

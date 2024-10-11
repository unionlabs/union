/*
 *
 * Copyright 2021 gRPC authors.
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

package authz_test

import (
	"context"
	"crypto/tls"
	"crypto/x509"
	"io"
	"net"
	"os"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/authz"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/testdata"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

type testServer struct {
	testgrpc.UnimplementedTestServiceServer
}

func (s *testServer) UnaryCall(ctx context.Context, req *testpb.SimpleRequest) (*testpb.SimpleResponse, error) {
	return &testpb.SimpleResponse{}, nil
}

func (s *testServer) StreamingInputCall(stream testgrpc.TestService_StreamingInputCallServer) error {
	for {
		_, err := stream.Recv()
		if err == io.EOF {
			return stream.SendAndClose(&testpb.StreamingInputCallResponse{})
		}
		if err != nil {
			return err
		}
	}
}

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

var authzTests = map[string]struct {
	authzPolicy string
	md          metadata.MD
	wantStatus  *status.Status
}{
	"DeniesRPCMatchInDenyNoMatchInAllow": {
		authzPolicy: `{
				"name": "authz",
				"allow_rules": 
				[
					{
						"name": "allow_StreamingOutputCall",
						"request": {
							"paths":
							[
								"/grpc.testing.TestService/StreamingOutputCall"
							]
						}
					}
				],
				"deny_rules":
				[
					{
						"name": "deny_TestServiceCalls",
						"request": {
							"paths":
							[
								"/grpc.testing.TestService/*"
							],
							"headers":
							[
								{
									"key": "key-abc",
									"values":
									[
										"val-abc",
										"val-def"
									]
								}
							]
						}
					}
				]
			}`,
		md:         metadata.Pairs("key-abc", "val-abc"),
		wantStatus: status.New(codes.PermissionDenied, "unauthorized RPC request rejected"),
	},
	"DeniesRPCMatchInDenyAndAllow": {
		authzPolicy: `{
				"name": "authz",
				"allow_rules":
				[
					{
						"name": "allow_all",
						"request": {
							"paths":
							[
								"*"
							]
						}
					}
				],
				"deny_rules":
				[
					{
						"name": "deny_all",
						"request": {
							"paths":
							[
								"*"
							]
						}
					}
				]
			}`,
		wantStatus: status.New(codes.PermissionDenied, "unauthorized RPC request rejected"),
	},
	"AllowsRPCNoMatchInDenyMatchInAllow": {
		authzPolicy: `{
				"name": "authz",
				"allow_rules":
				[
					{
						"name": "allow_all"
					}
				],
				"deny_rules":
				[
					{
						"name": "deny_TestServiceCalls",
						"request": {
							"paths":
							[
								"/grpc.testing.TestService/UnaryCall",
								"/grpc.testing.TestService/StreamingInputCall"
							],
							"headers": 
							[
								{
									"key": "key-abc",
									"values": 
									[
										"val-abc",
										"val-def"
									]
								}
							]
						}
					}
				]
			}`,
		md:         metadata.Pairs("key-xyz", "val-xyz"),
		wantStatus: status.New(codes.OK, ""),
	},
	"DeniesRPCNoMatchInDenyAndAllow": {
		authzPolicy: `{
				"name": "authz",
				"allow_rules":
				[
					{
						"name": "allow_some_user",
						"source": {
							"principals":
							[
								"some_user"
							]
						}
					}
				],
				"deny_rules":
				[
					{
						"name": "deny_StreamingOutputCall",
						"request": {
							"paths":
							[
								"/grpc.testing.TestService/StreamingOutputCall"
							]
						}
					}
				]
			}`,
		wantStatus: status.New(codes.PermissionDenied, "unauthorized RPC request rejected"),
	},
	"AllowsRPCEmptyDenyMatchInAllow": {
		authzPolicy: `{
				"name": "authz",
				"allow_rules":
				[
					{
						"name": "allow_UnaryCall",
						"request":
						{
							"paths":
							[
								"/grpc.testing.TestService/UnaryCall"
							]
						}
					},
					{
						"name": "allow_StreamingInputCall",
						"request":
						{
							"paths":
							[
								"/grpc.testing.TestService/StreamingInputCall"
							]
						}
					}
				]
			}`,
		wantStatus: status.New(codes.OK, ""),
	},
	"DeniesRPCEmptyDenyNoMatchInAllow": {
		authzPolicy: `{
				"name": "authz",
				"allow_rules":
				[
					{
						"name": "allow_StreamingOutputCall",
						"request": 
						{
							"paths":
							[
								"/grpc.testing.TestService/StreamingOutputCall"
							]
						}
					}
				]
			}`,
		wantStatus: status.New(codes.PermissionDenied, "unauthorized RPC request rejected"),
	},
	"DeniesRPCRequestWithPrincipalsFieldOnUnauthenticatedConnection": {
		authzPolicy: `{
				"name": "authz",
				"allow_rules":
				[
					{
						"name": "allow_authenticated",
						"source": {
							"principals": ["*", ""]
						}
					}
				]
			}`,
		wantStatus: status.New(codes.PermissionDenied, "unauthorized RPC request rejected"),
	},
	"DeniesRPCRequestNoMatchInAllowFailsPresenceMatch": {
		authzPolicy: `{
				"name": "authz",
				"allow_rules":
				[
					{
						"name": "allow_TestServiceCalls",
						"request": {
							"paths":
							[
								"/grpc.testing.TestService/*"
							],
							"headers":
							[
								{
									"key": "key-abc",
									"values":
									[
										"*"
									]
								}
							]
						}
					}
				]
			}`,
		md:         metadata.Pairs("key-abc", ""),
		wantStatus: status.New(codes.PermissionDenied, "unauthorized RPC request rejected"),
	},
}

func (s) TestStaticPolicyEnd2End(t *testing.T) {
	for name, test := range authzTests {
		t.Run(name, func(t *testing.T) {
			// Start a gRPC server with gRPC authz unary and stream server interceptors.
			i, _ := authz.NewStatic(test.authzPolicy)
			s := grpc.NewServer(
				grpc.ChainUnaryInterceptor(i.UnaryInterceptor),
				grpc.ChainStreamInterceptor(i.StreamInterceptor))
			defer s.Stop()
			testgrpc.RegisterTestServiceServer(s, &testServer{})

			lis, err := net.Listen("tcp", "localhost:0")
			if err != nil {
				t.Fatalf("error listening: %v", err)
			}
			go s.Serve(lis)

			// Establish a connection to the server.
			clientConn, err := grpc.NewClient(lis.Addr().String(), grpc.WithTransportCredentials(insecure.NewCredentials()))
			if err != nil {
				t.Fatalf("grpc.NewClient(%v) failed: %v", lis.Addr().String(), err)
			}
			defer clientConn.Close()
			client := testgrpc.NewTestServiceClient(clientConn)

			ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
			defer cancel()
			ctx = metadata.NewOutgoingContext(ctx, test.md)

			// Verifying authorization decision for Unary RPC.
			_, err = client.UnaryCall(ctx, &testpb.SimpleRequest{})
			if got := status.Convert(err); got.Code() != test.wantStatus.Code() || got.Message() != test.wantStatus.Message() {
				t.Fatalf("[UnaryCall] error want:{%v} got:{%v}", test.wantStatus.Err(), got.Err())
			}

			// Verifying authorization decision for Streaming RPC.
			stream, err := client.StreamingInputCall(ctx)
			if err != nil {
				t.Fatalf("failed StreamingInputCall err: %v", err)
			}
			req := &testpb.StreamingInputCallRequest{
				Payload: &testpb.Payload{
					Body: []byte("hi"),
				},
			}
			if err := stream.Send(req); err != nil && err != io.EOF {
				t.Fatalf("failed stream.Send err: %v", err)
			}
			_, err = stream.CloseAndRecv()
			if got := status.Convert(err); got.Code() != test.wantStatus.Code() || got.Message() != test.wantStatus.Message() {
				t.Fatalf("[StreamingCall] error want:{%v} got:{%v}", test.wantStatus.Err(), got.Err())
			}
		})
	}
}

func (s) TestAllowsRPCRequestWithPrincipalsFieldOnTLSAuthenticatedConnection(t *testing.T) {
	authzPolicy := `{
				"name": "authz",
				"allow_rules":
				[
					{
						"name": "allow_authenticated",
						"source": {
							"principals": ["*", ""]
						}
					}
				]
			}`
	// Start a gRPC server with gRPC authz unary server interceptor.
	i, _ := authz.NewStatic(authzPolicy)
	creds, err := credentials.NewServerTLSFromFile(testdata.Path("x509/server1_cert.pem"), testdata.Path("x509/server1_key.pem"))
	if err != nil {
		t.Fatalf("failed to generate credentials: %v", err)
	}
	s := grpc.NewServer(
		grpc.Creds(creds),
		grpc.ChainUnaryInterceptor(i.UnaryInterceptor))
	defer s.Stop()
	testgrpc.RegisterTestServiceServer(s, &testServer{})

	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("error listening: %v", err)
	}
	go s.Serve(lis)

	// Establish a connection to the server.
	creds, err = credentials.NewClientTLSFromFile(testdata.Path("x509/server_ca_cert.pem"), "x.test.example.com")
	if err != nil {
		t.Fatalf("failed to load credentials: %v", err)
	}
	clientConn, err := grpc.NewClient(lis.Addr().String(), grpc.WithTransportCredentials(creds))
	if err != nil {
		t.Fatalf("grpc.NewClient(%v) failed: %v", lis.Addr().String(), err)
	}
	defer clientConn.Close()
	client := testgrpc.NewTestServiceClient(clientConn)

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	// Verifying authorization decision.
	if _, err = client.UnaryCall(ctx, &testpb.SimpleRequest{}); err != nil {
		t.Fatalf("client.UnaryCall(_, _) = %v; want nil", err)
	}
}

func (s) TestAllowsRPCRequestWithPrincipalsFieldOnMTLSAuthenticatedConnection(t *testing.T) {
	authzPolicy := `{
				"name": "authz",
				"allow_rules":
				[
					{
						"name": "allow_authenticated",
						"source": {
							"principals": ["*", ""]
						}
					}
				]
			}`
	// Start a gRPC server with gRPC authz unary server interceptor.
	i, _ := authz.NewStatic(authzPolicy)
	cert, err := tls.LoadX509KeyPair(testdata.Path("x509/server1_cert.pem"), testdata.Path("x509/server1_key.pem"))
	if err != nil {
		t.Fatalf("tls.LoadX509KeyPair(x509/server1_cert.pem, x509/server1_key.pem) failed: %v", err)
	}
	ca, err := os.ReadFile(testdata.Path("x509/client_ca_cert.pem"))
	if err != nil {
		t.Fatalf("os.ReadFile(x509/client_ca_cert.pem) failed: %v", err)
	}
	certPool := x509.NewCertPool()
	if !certPool.AppendCertsFromPEM(ca) {
		t.Fatal("failed to append certificates")
	}
	creds := credentials.NewTLS(&tls.Config{
		ClientAuth:   tls.RequireAndVerifyClientCert,
		Certificates: []tls.Certificate{cert},
		ClientCAs:    certPool,
	})
	s := grpc.NewServer(
		grpc.Creds(creds),
		grpc.ChainUnaryInterceptor(i.UnaryInterceptor))
	defer s.Stop()
	testgrpc.RegisterTestServiceServer(s, &testServer{})

	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("error listening: %v", err)
	}
	go s.Serve(lis)

	// Establish a connection to the server.
	cert, err = tls.LoadX509KeyPair(testdata.Path("x509/client1_cert.pem"), testdata.Path("x509/client1_key.pem"))
	if err != nil {
		t.Fatalf("tls.LoadX509KeyPair(x509/client1_cert.pem, x509/client1_key.pem) failed: %v", err)
	}
	ca, err = os.ReadFile(testdata.Path("x509/server_ca_cert.pem"))
	if err != nil {
		t.Fatalf("os.ReadFile(x509/server_ca_cert.pem) failed: %v", err)
	}
	roots := x509.NewCertPool()
	if !roots.AppendCertsFromPEM(ca) {
		t.Fatal("failed to append certificates")
	}
	creds = credentials.NewTLS(&tls.Config{
		Certificates: []tls.Certificate{cert},
		RootCAs:      roots,
		ServerName:   "x.test.example.com",
	})
	clientConn, err := grpc.NewClient(lis.Addr().String(), grpc.WithTransportCredentials(creds))
	if err != nil {
		t.Fatalf("grpc.NewClient(%v) failed: %v", lis.Addr().String(), err)
	}
	defer clientConn.Close()
	client := testgrpc.NewTestServiceClient(clientConn)

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	// Verifying authorization decision.
	if _, err = client.UnaryCall(ctx, &testpb.SimpleRequest{}); err != nil {
		t.Fatalf("client.UnaryCall(_, _) = %v; want nil", err)
	}
}

func (s) TestFileWatcherEnd2End(t *testing.T) {
	for name, test := range authzTests {
		t.Run(name, func(t *testing.T) {
			file := createTmpPolicyFile(t, name, []byte(test.authzPolicy))
			i, _ := authz.NewFileWatcher(file, 1*time.Second)
			defer i.Close()

			// Start a gRPC server with gRPC authz unary and stream server interceptors.
			s := grpc.NewServer(
				grpc.ChainUnaryInterceptor(i.UnaryInterceptor),
				grpc.ChainStreamInterceptor(i.StreamInterceptor))
			defer s.Stop()
			testgrpc.RegisterTestServiceServer(s, &testServer{})

			lis, err := net.Listen("tcp", "localhost:0")
			if err != nil {
				t.Fatalf("error listening: %v", err)
			}
			defer lis.Close()
			go s.Serve(lis)

			// Establish a connection to the server.
			clientConn, err := grpc.NewClient(lis.Addr().String(), grpc.WithTransportCredentials(insecure.NewCredentials()))
			if err != nil {
				t.Fatalf("grpc.NewClient(%v) failed: %v", lis.Addr().String(), err)
			}
			defer clientConn.Close()
			client := testgrpc.NewTestServiceClient(clientConn)

			ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
			defer cancel()
			ctx = metadata.NewOutgoingContext(ctx, test.md)

			// Verifying authorization decision for Unary RPC.
			_, err = client.UnaryCall(ctx, &testpb.SimpleRequest{})
			if got := status.Convert(err); got.Code() != test.wantStatus.Code() || got.Message() != test.wantStatus.Message() {
				t.Fatalf("[UnaryCall] error want:{%v} got:{%v}", test.wantStatus.Err(), got.Err())
			}

			// Verifying authorization decision for Streaming RPC.
			stream, err := client.StreamingInputCall(ctx)
			if err != nil {
				t.Fatalf("failed StreamingInputCall err: %v", err)
			}
			req := &testpb.StreamingInputCallRequest{
				Payload: &testpb.Payload{
					Body: []byte("hi"),
				},
			}
			if err := stream.Send(req); err != nil && err != io.EOF {
				t.Fatalf("failed stream.Send err: %v", err)
			}
			_, err = stream.CloseAndRecv()
			if got := status.Convert(err); got.Code() != test.wantStatus.Code() || got.Message() != test.wantStatus.Message() {
				t.Fatalf("[StreamingCall] error want:{%v} got:{%v}", test.wantStatus.Err(), got.Err())
			}
		})
	}
}

func retryUntil(ctx context.Context, tsc testgrpc.TestServiceClient, want *status.Status) (lastErr error) {
	for ctx.Err() == nil {
		_, lastErr = tsc.UnaryCall(ctx, &testpb.SimpleRequest{})
		if s := status.Convert(lastErr); s.Code() == want.Code() && s.Message() == want.Message() {
			return nil
		}
		time.Sleep(20 * time.Millisecond)
	}
	return lastErr
}

func (s) TestFileWatcher_ValidPolicyRefresh(t *testing.T) {
	valid1 := authzTests["DeniesRPCMatchInDenyAndAllow"]
	file := createTmpPolicyFile(t, "valid_policy_refresh", []byte(valid1.authzPolicy))
	i, _ := authz.NewFileWatcher(file, 100*time.Millisecond)
	defer i.Close()

	// Start a gRPC server with gRPC authz unary server interceptor.
	s := grpc.NewServer(
		grpc.ChainUnaryInterceptor(i.UnaryInterceptor))
	defer s.Stop()
	testgrpc.RegisterTestServiceServer(s, &testServer{})

	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("error listening: %v", err)
	}
	defer lis.Close()
	go s.Serve(lis)

	// Establish a connection to the server.
	clientConn, err := grpc.NewClient(lis.Addr().String(), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("grpc.NewClient(%v) failed: %v", lis.Addr().String(), err)
	}
	defer clientConn.Close()
	client := testgrpc.NewTestServiceClient(clientConn)

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	// Verifying authorization decision.
	_, err = client.UnaryCall(ctx, &testpb.SimpleRequest{})
	if got := status.Convert(err); got.Code() != valid1.wantStatus.Code() || got.Message() != valid1.wantStatus.Message() {
		t.Fatalf("client.UnaryCall(_, _) = %v; want = %v", got.Err(), valid1.wantStatus.Err())
	}

	// Rewrite the file with a different valid authorization policy.
	valid2 := authzTests["AllowsRPCEmptyDenyMatchInAllow"]
	if err := os.WriteFile(file, []byte(valid2.authzPolicy), os.ModePerm); err != nil {
		t.Fatalf("os.WriteFile(%q) failed: %v", file, err)
	}

	// Verifying authorization decision.
	if got := retryUntil(ctx, client, valid2.wantStatus); got != nil {
		t.Fatalf("client.UnaryCall(_, _) = %v; want = %v", got, valid2.wantStatus.Err())
	}
}

func (s) TestFileWatcher_InvalidPolicySkipReload(t *testing.T) {
	valid := authzTests["DeniesRPCMatchInDenyAndAllow"]
	file := createTmpPolicyFile(t, "invalid_policy_skip_reload", []byte(valid.authzPolicy))
	i, _ := authz.NewFileWatcher(file, 20*time.Millisecond)
	defer i.Close()

	// Start a gRPC server with gRPC authz unary server interceptors.
	s := grpc.NewServer(
		grpc.ChainUnaryInterceptor(i.UnaryInterceptor))
	defer s.Stop()
	testgrpc.RegisterTestServiceServer(s, &testServer{})

	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("error listening: %v", err)
	}
	defer lis.Close()
	go s.Serve(lis)

	// Establish a connection to the server.
	clientConn, err := grpc.NewClient(lis.Addr().String(), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("grpc.NewClient(%v) failed: %v", lis.Addr().String(), err)
	}
	defer clientConn.Close()
	client := testgrpc.NewTestServiceClient(clientConn)

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	// Verifying authorization decision.
	_, err = client.UnaryCall(ctx, &testpb.SimpleRequest{})
	if got := status.Convert(err); got.Code() != valid.wantStatus.Code() || got.Message() != valid.wantStatus.Message() {
		t.Fatalf("client.UnaryCall(_, _) = %v; want = %v", got.Err(), valid.wantStatus.Err())
	}

	// Skips the invalid policy update, and continues to use the valid policy.
	if err := os.WriteFile(file, []byte("{}"), os.ModePerm); err != nil {
		t.Fatalf("os.WriteFile(%q) failed: %v", file, err)
	}

	// Wait 40 ms for background go routine to read updated files.
	time.Sleep(40 * time.Millisecond)

	// Verifying authorization decision.
	_, err = client.UnaryCall(ctx, &testpb.SimpleRequest{})
	if got := status.Convert(err); got.Code() != valid.wantStatus.Code() || got.Message() != valid.wantStatus.Message() {
		t.Fatalf("client.UnaryCall(_, _) = %v; want = %v", got.Err(), valid.wantStatus.Err())
	}
}

func (s) TestFileWatcher_RecoversFromReloadFailure(t *testing.T) {
	valid1 := authzTests["DeniesRPCMatchInDenyAndAllow"]
	file := createTmpPolicyFile(t, "recovers_from_reload_failure", []byte(valid1.authzPolicy))
	i, _ := authz.NewFileWatcher(file, 100*time.Millisecond)
	defer i.Close()

	// Start a gRPC server with gRPC authz unary server interceptors.
	s := grpc.NewServer(
		grpc.ChainUnaryInterceptor(i.UnaryInterceptor))
	defer s.Stop()
	testgrpc.RegisterTestServiceServer(s, &testServer{})

	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("error listening: %v", err)
	}
	defer lis.Close()
	go s.Serve(lis)

	// Establish a connection to the server.
	clientConn, err := grpc.NewClient(lis.Addr().String(), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("grpc.NewClient(%v) failed: %v", lis.Addr().String(), err)
	}
	defer clientConn.Close()
	client := testgrpc.NewTestServiceClient(clientConn)

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	// Verifying authorization decision.
	_, err = client.UnaryCall(ctx, &testpb.SimpleRequest{})
	if got := status.Convert(err); got.Code() != valid1.wantStatus.Code() || got.Message() != valid1.wantStatus.Message() {
		t.Fatalf("client.UnaryCall(_, _) = %v; want = %v", got.Err(), valid1.wantStatus.Err())
	}

	// Skips the invalid policy update, and continues to use the valid policy.
	if err := os.WriteFile(file, []byte("{}"), os.ModePerm); err != nil {
		t.Fatalf("os.WriteFile(%q) failed: %v", file, err)
	}

	// Wait 120 ms for background go routine to read updated files.
	time.Sleep(120 * time.Millisecond)

	// Verifying authorization decision.
	_, err = client.UnaryCall(ctx, &testpb.SimpleRequest{})
	if got := status.Convert(err); got.Code() != valid1.wantStatus.Code() || got.Message() != valid1.wantStatus.Message() {
		t.Fatalf("client.UnaryCall(_, _) = %v; want = %v", got.Err(), valid1.wantStatus.Err())
	}

	// Rewrite the file with a different valid authorization policy.
	valid2 := authzTests["AllowsRPCEmptyDenyMatchInAllow"]
	if err := os.WriteFile(file, []byte(valid2.authzPolicy), os.ModePerm); err != nil {
		t.Fatalf("os.WriteFile(%q) failed: %v", file, err)
	}

	// Verifying authorization decision.
	if got := retryUntil(ctx, client, valid2.wantStatus); got != nil {
		t.Fatalf("client.UnaryCall(_, _) = %v; want = %v", got, valid2.wantStatus.Err())
	}
}

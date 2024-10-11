/*
 *
 * Copyright 2023 gRPC authors.
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

package audit_test

import (
	"context"
	"crypto/tls"
	"crypto/x509"
	"encoding/json"
	"io"
	"net"
	"os"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc"
	"google.golang.org/grpc/authz"
	"google.golang.org/grpc/authz/audit"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/stubserver"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/testdata"

	_ "google.golang.org/grpc/authz/audit/stdout"
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

type statAuditLogger struct {
	authzDecisionStat map[bool]int // Map to hold the counts of authorization decisions
	lastEvent         *audit.Event // Field to store last received event
}

func (s *statAuditLogger) Log(event *audit.Event) {
	s.authzDecisionStat[event.Authorized]++
	*s.lastEvent = *event
}

type loggerBuilder struct {
	authzDecisionStat map[bool]int
	lastEvent         *audit.Event
}

func (loggerBuilder) Name() string {
	return "stat_logger"
}

func (lb *loggerBuilder) Build(audit.LoggerConfig) audit.Logger {
	return &statAuditLogger{
		authzDecisionStat: lb.authzDecisionStat,
		lastEvent:         lb.lastEvent,
	}
}

func (*loggerBuilder) ParseLoggerConfig(config json.RawMessage) (audit.LoggerConfig, error) {
	return nil, nil
}

// TestAuditLogger examines audit logging invocations using four different
// authorization policies. It covers scenarios including a disabled audit,
// auditing both 'allow' and 'deny' outcomes, and separately auditing 'allow'
// and 'deny' outcomes. Additionally, it checks if SPIFFE ID from a certificate
// is propagated correctly.
func (s) TestAuditLogger(t *testing.T) {
	// Each test data entry contains an authz policy for a grpc server,
	// how many 'allow' and 'deny' outcomes we expect (each test case makes 2
	// unary calls and one client-streaming call), and a structure to check if
	// the audit.Event fields are properly populated. Additionally, we specify
	// directly which authz outcome we expect from each type of call.
	tests := []struct {
		name                  string
		authzPolicy           string
		wantAuthzOutcomes     map[bool]int
		eventContent          *audit.Event
		wantUnaryCallCode     codes.Code
		wantStreamingCallCode codes.Code
	}{
		{
			name: "No audit",
			authzPolicy: `{
				"name": "authz",
				"allow_rules": [
					{
						"name": "allow_UnaryCall",
						"request": {
							"paths": [
								"/grpc.testing.TestService/UnaryCall"
							]
						}
					}
				],
				"audit_logging_options": {
					"audit_condition": "NONE",
					"audit_loggers": [
						{
							"name": "stat_logger",
							"config": {},
							"is_optional": false
						}
					]
				}
			}`,
			wantAuthzOutcomes:     map[bool]int{true: 0, false: 0},
			wantUnaryCallCode:     codes.OK,
			wantStreamingCallCode: codes.PermissionDenied,
		},
		{
			name: "Allow All Deny Streaming - Audit All",
			authzPolicy: `{
				"name": "authz",
				"allow_rules": [
					{
						"name": "allow_all",
						"request": {
							"paths": [
								"*"
							]
						}
					}
				],
				"deny_rules": [
					{
						"name": "deny_all",
						"request": {
							"paths": [
								"/grpc.testing.TestService/StreamingInputCall"
							]
						}
					}
				],
				"audit_logging_options": {
					"audit_condition": "ON_DENY_AND_ALLOW",
					"audit_loggers": [
						{
							"name": "stat_logger",
							"config": {},
							"is_optional": false
						},
						{
							"name": "stdout_logger",
							"is_optional": false
						}
					]
				}
			}`,
			wantAuthzOutcomes: map[bool]int{true: 2, false: 1},
			eventContent: &audit.Event{
				FullMethodName: "/grpc.testing.TestService/StreamingInputCall",
				Principal:      "spiffe://foo.bar.com/client/workload/1",
				PolicyName:     "authz",
				MatchedRule:    "authz_deny_all",
				Authorized:     false,
			},
			wantUnaryCallCode:     codes.OK,
			wantStreamingCallCode: codes.PermissionDenied,
		},
		{
			name: "Allow Unary - Audit Allow",
			authzPolicy: `{
				"name": "authz",
				"allow_rules": [
					{
						"name": "allow_UnaryCall",
						"request": {
							"paths": [
								"/grpc.testing.TestService/UnaryCall"
							]
						}
					}
				],
				"audit_logging_options": {
					"audit_condition": "ON_ALLOW",
					"audit_loggers": [
						{
							"name": "stat_logger",
							"config": {},
							"is_optional": false
						}
					]
				}
			}`,
			wantAuthzOutcomes:     map[bool]int{true: 2, false: 0},
			wantUnaryCallCode:     codes.OK,
			wantStreamingCallCode: codes.PermissionDenied,
		},
		{
			name: "Allow Typo - Audit Deny",
			authzPolicy: `{
				"name": "authz",
				"allow_rules": [
					{
						"name": "allow_UnaryCall",
						"request": {
							"paths": [
								"/grpc.testing.TestService/UnaryCall_Z"
							]
						}
					}
				],
				"audit_logging_options": {
					"audit_condition": "ON_DENY",
					"audit_loggers": [
						{
							"name": "stat_logger",
							"config": {},
							"is_optional": false
						}
					]
				}
			}`,
			wantAuthzOutcomes:     map[bool]int{true: 0, false: 3},
			wantUnaryCallCode:     codes.PermissionDenied,
			wantStreamingCallCode: codes.PermissionDenied,
		},
	}
	// Construct the credentials for the tests and the stub server
	serverCreds := loadServerCreds(t)
	clientCreds := loadClientCreds(t)
	ss := &stubserver.StubServer{
		UnaryCallF: func(ctx context.Context, in *testpb.SimpleRequest) (*testpb.SimpleResponse, error) {
			return &testpb.SimpleResponse{}, nil
		},
		FullDuplexCallF: func(stream testgrpc.TestService_FullDuplexCallServer) error {
			_, err := stream.Recv()
			if err != io.EOF {
				return err
			}
			return nil
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			// Setup test statAuditLogger, gRPC test server with authzPolicy, unary
			// and stream interceptors.
			lb := &loggerBuilder{
				authzDecisionStat: map[bool]int{true: 0, false: 0},
				lastEvent:         &audit.Event{},
			}
			audit.RegisterLoggerBuilder(lb)
			i, _ := authz.NewStatic(test.authzPolicy)

			s := grpc.NewServer(
				grpc.Creds(serverCreds),
				grpc.ChainUnaryInterceptor(i.UnaryInterceptor),
				grpc.ChainStreamInterceptor(i.StreamInterceptor))
			defer s.Stop()
			testgrpc.RegisterTestServiceServer(s, ss)
			lis, err := net.Listen("tcp", "localhost:0")
			if err != nil {
				t.Fatalf("Error listening: %v", err)
			}
			go s.Serve(lis)

			// Setup gRPC test client with certificates containing a SPIFFE Id.
			clientConn, err := grpc.NewClient(lis.Addr().String(), grpc.WithTransportCredentials(clientCreds))
			if err != nil {
				t.Fatalf("grpc.NewClient(%v) failed: %v", lis.Addr().String(), err)
			}
			defer clientConn.Close()
			client := testgrpc.NewTestServiceClient(clientConn)
			ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
			defer cancel()

			if _, err := client.UnaryCall(ctx, &testpb.SimpleRequest{}); status.Code(err) != test.wantUnaryCallCode {
				t.Errorf("Unexpected UnaryCall fail: got %v want %v", err, test.wantUnaryCallCode)
			}
			if _, err := client.UnaryCall(ctx, &testpb.SimpleRequest{}); status.Code(err) != test.wantUnaryCallCode {
				t.Errorf("Unexpected UnaryCall fail: got %v want %v", err, test.wantUnaryCallCode)
			}
			stream, err := client.StreamingInputCall(ctx)
			if err != nil {
				t.Fatalf("StreamingInputCall failed:%v", err)
			}
			req := &testpb.StreamingInputCallRequest{
				Payload: &testpb.Payload{
					Body: []byte("hi"),
				},
			}
			if err := stream.Send(req); err != nil && err != io.EOF {
				t.Fatalf("stream.Send failed:%v", err)
			}
			if _, err := stream.CloseAndRecv(); status.Code(err) != test.wantStreamingCallCode {
				t.Errorf("Unexpected stream.CloseAndRecv fail: got %v want %v", err, test.wantStreamingCallCode)
			}

			// Compare expected number of allows/denies with content of the internal
			// map of statAuditLogger.
			if diff := cmp.Diff(lb.authzDecisionStat, test.wantAuthzOutcomes); diff != "" {
				t.Errorf("Authorization decisions do not match\ndiff (-got +want):\n%s", diff)
			}
			// Compare last event received by statAuditLogger with expected event.
			if test.eventContent != nil {
				if diff := cmp.Diff(lb.lastEvent, test.eventContent); diff != "" {
					t.Errorf("Unexpected message\ndiff (-got +want):\n%s", diff)
				}
			}
		})
	}
}

// loadServerCreds constructs TLS containing server certs and CA
func loadServerCreds(t *testing.T) credentials.TransportCredentials {
	t.Helper()
	cert := loadKeys(t, "x509/server1_cert.pem", "x509/server1_key.pem")
	certPool := loadCACerts(t, "x509/client_ca_cert.pem")
	return credentials.NewTLS(&tls.Config{
		ClientAuth:   tls.RequireAndVerifyClientCert,
		Certificates: []tls.Certificate{cert},
		ClientCAs:    certPool,
	})
}

// loadClientCreds constructs TLS containing client certs and CA
func loadClientCreds(t *testing.T) credentials.TransportCredentials {
	t.Helper()
	cert := loadKeys(t, "x509/client_with_spiffe_cert.pem", "x509/client_with_spiffe_key.pem")
	roots := loadCACerts(t, "x509/server_ca_cert.pem")
	return credentials.NewTLS(&tls.Config{
		Certificates: []tls.Certificate{cert},
		RootCAs:      roots,
		ServerName:   "x.test.example.com",
	})

}

// loadKeys loads X509 key pair from the provided file paths.
// It is used for loading both client and server certificates for the test
func loadKeys(t *testing.T, certPath, key string) tls.Certificate {
	t.Helper()
	cert, err := tls.LoadX509KeyPair(testdata.Path(certPath), testdata.Path(key))
	if err != nil {
		t.Fatalf("tls.LoadX509KeyPair(%q, %q) failed: %v", certPath, key, err)
	}
	return cert
}

// loadCACerts loads CA certificates and constructs x509.CertPool
// It is used for loading both client and server CAs for the test
func loadCACerts(t *testing.T, certPath string) *x509.CertPool {
	t.Helper()
	ca, err := os.ReadFile(testdata.Path(certPath))
	if err != nil {
		t.Fatalf("os.ReadFile(%q) failed: %v", certPath, err)
	}
	roots := x509.NewCertPool()
	if !roots.AppendCertsFromPEM(ca) {
		t.Fatal("Failed to append certificates")
	}
	return roots
}

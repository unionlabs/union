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

package tlscreds_test

import (
	"context"
	"crypto/tls"
	"encoding/json"
	"fmt"
	"os"
	"strings"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/xds/bootstrap/tlscreds"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/testdata"
)

const defaultTestTimeout = 5 * time.Second

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

type Closable interface {
	Close()
}

func (s) TestValidTlsBuilder(t *testing.T) {
	caCert := testdata.Path("x509/server_ca_cert.pem")
	clientCert := testdata.Path("x509/client1_cert.pem")
	clientKey := testdata.Path("x509/client1_key.pem")
	tests := []struct {
		name string
		jd   string
	}{
		{
			name: "Absent configuration",
			jd:   `null`,
		},
		{
			name: "Empty configuration",
			jd:   `{}`,
		},
		{
			name: "Only CA certificate chain",
			jd:   fmt.Sprintf(`{"ca_certificate_file": "%s"}`, caCert),
		},
		{
			name: "Only private key and certificate chain",
			jd:   fmt.Sprintf(`{"certificate_file":"%s","private_key_file":"%s"}`, clientCert, clientKey),
		},
		{
			name: "CA chain, private key and certificate chain",
			jd:   fmt.Sprintf(`{"ca_certificate_file":"%s","certificate_file":"%s","private_key_file":"%s"}`, caCert, clientCert, clientKey),
		},
		{
			name: "Only refresh interval", jd: `{"refresh_interval": "1s"}`,
		},
		{
			name: "Refresh interval and CA certificate chain",
			jd:   fmt.Sprintf(`{"refresh_interval": "1s","ca_certificate_file": "%s"}`, caCert),
		},
		{
			name: "Refresh interval, private key and certificate chain",
			jd:   fmt.Sprintf(`{"refresh_interval": "1s","certificate_file":"%s","private_key_file":"%s"}`, clientCert, clientKey),
		},
		{
			name: "Refresh interval, CA chain, private key and certificate chain",
			jd:   fmt.Sprintf(`{"refresh_interval": "1s","ca_certificate_file":"%s","certificate_file":"%s","private_key_file":"%s"}`, caCert, clientCert, clientKey),
		},
		{
			name: "Unknown field",
			jd:   `{"unknown_field": "foo"}`,
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			msg := json.RawMessage(test.jd)
			_, stop, err := tlscreds.NewBundle(msg)
			if err != nil {
				t.Fatalf("NewBundle(%s) returned error %s when expected to succeed", test.jd, err)
			}
			stop()
		})
	}
}

func (s) TestInvalidTlsBuilder(t *testing.T) {
	tests := []struct {
		name, jd, wantErrPrefix string
	}{
		{
			name:          "Wrong type in json",
			jd:            `{"ca_certificate_file": 1}`,
			wantErrPrefix: "failed to unmarshal config:"},
		{
			name:          "Missing private key",
			jd:            fmt.Sprintf(`{"certificate_file":"%s"}`, testdata.Path("x509/server_cert.pem")),
			wantErrPrefix: "pemfile: private key file and identity cert file should be both specified or not specified",
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			msg := json.RawMessage(test.jd)
			_, stop, err := tlscreds.NewBundle(msg)
			if err == nil || !strings.HasPrefix(err.Error(), test.wantErrPrefix) {
				if stop != nil {
					stop()
				}
				t.Fatalf("NewBundle(%s): got error %s, want an error with prefix %s", msg, err, test.wantErrPrefix)
			}
		})
	}
}

func (s) TestCaReloading(t *testing.T) {
	serverCa, err := os.ReadFile(testdata.Path("x509/server_ca_cert.pem"))
	if err != nil {
		t.Fatalf("Failed to read test CA cert: %s", err)
	}

	// Write CA certs to a temporary file so that we can modify it later.
	caPath := t.TempDir() + "/ca.pem"
	if err = os.WriteFile(caPath, serverCa, 0644); err != nil {
		t.Fatalf("Failed to write test CA cert: %v", err)
	}
	cfg := fmt.Sprintf(`{
		"ca_certificate_file": "%s",
		"refresh_interval": ".01s"
	}`, caPath)
	tlsBundle, stop, err := tlscreds.NewBundle([]byte(cfg))
	if err != nil {
		t.Fatalf("Failed to create TLS bundle: %v", err)
	}
	defer stop()

	serverCredentials := grpc.Creds(testutils.CreateServerTLSCredentials(t, tls.NoClientCert))
	server := stubserver.StartTestService(t, nil, serverCredentials)

	conn, err := grpc.NewClient(
		server.Address,
		grpc.WithCredentialsBundle(tlsBundle),
		grpc.WithAuthority("x.test.example.com"),
	)
	if err != nil {
		t.Fatalf("Error dialing: %v", err)
	}
	defer conn.Close()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	client := testgrpc.NewTestServiceClient(conn)
	if _, err = client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Errorf("Error calling EmptyCall: %v", err)
	}
	// close the server and create a new one to force client to do a new
	// handshake.
	server.Stop()

	invalidCa, err := os.ReadFile(testdata.Path("ca.pem"))
	if err != nil {
		t.Fatalf("Failed to read test CA cert: %v", err)
	}
	// unload root cert
	err = os.WriteFile(caPath, invalidCa, 0644)
	if err != nil {
		t.Fatalf("Failed to write test CA cert: %v", err)
	}

	for ; ctx.Err() == nil; <-time.After(10 * time.Millisecond) {
		ss := stubserver.StubServer{
			Address:    server.Address,
			EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) { return &testpb.Empty{}, nil },
		}
		server = stubserver.StartTestService(t, &ss, serverCredentials)

		// Client handshake should eventually fail because the client CA was
		// reloaded, and thus the server cert is signed by an unknown CA.
		t.Log(server)
		_, err = client.EmptyCall(ctx, &testpb.Empty{})
		const wantErr = "certificate signed by unknown authority"
		if status.Code(err) == codes.Unavailable && strings.Contains(err.Error(), wantErr) {
			// Certs have reloaded.
			server.Stop()
			break
		}
		t.Logf("EmptyCall() got err: %s, want code: %s, want err: %s", err, codes.Unavailable, wantErr)
		server.Stop()
	}
	if ctx.Err() != nil {
		t.Errorf("Timed out waiting for CA certs reloading")
	}
}

func (s) TestMTLS(t *testing.T) {
	s := stubserver.StartTestService(t, nil, grpc.Creds(testutils.CreateServerTLSCredentials(t, tls.RequireAndVerifyClientCert)))
	defer s.Stop()

	cfg := fmt.Sprintf(`{
		"ca_certificate_file": "%s",
		"certificate_file": "%s",
		"private_key_file": "%s"
	}`,
		testdata.Path("x509/server_ca_cert.pem"),
		testdata.Path("x509/client1_cert.pem"),
		testdata.Path("x509/client1_key.pem"))
	tlsBundle, stop, err := tlscreds.NewBundle([]byte(cfg))
	if err != nil {
		t.Fatalf("Failed to create TLS bundle: %v", err)
	}
	defer stop()
	conn, err := grpc.NewClient(s.Address, grpc.WithCredentialsBundle(tlsBundle), grpc.WithAuthority("x.test.example.com"))
	if err != nil {
		t.Fatalf("Error dialing: %v", err)
	}
	defer conn.Close()
	client := testgrpc.NewTestServiceClient(conn)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err = client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Errorf("EmptyCall(): got error %v when expected to succeed", err)
	}
}

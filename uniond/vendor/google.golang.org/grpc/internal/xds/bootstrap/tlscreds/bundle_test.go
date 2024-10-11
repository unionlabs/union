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

package tlscreds

import (
	"context"
	"crypto/tls"
	"errors"
	"fmt"
	"strings"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/tls/certprovider"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/testdata"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

const defaultTestTimeout = 5 * time.Second

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

type failingProvider struct{}

func (f failingProvider) KeyMaterial(context.Context) (*certprovider.KeyMaterial, error) {
	return nil, errors.New("test error")
}

func (f failingProvider) Close() {}

func (s) TestFailingProvider(t *testing.T) {
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
	tlsBundle, stop, err := NewBundle([]byte(cfg))
	if err != nil {
		t.Fatalf("Failed to create TLS bundle: %v", err)
	}
	stop()

	// Force a provider that returns an error, and make sure the client fails
	// the handshake.
	creds, ok := tlsBundle.TransportCredentials().(*reloadingCreds)
	if !ok {
		t.Fatalf("Got %T, expected reloadingCreds", tlsBundle.TransportCredentials())
	}
	creds.provider = &failingProvider{}

	conn, err := grpc.NewClient(s.Address, grpc.WithCredentialsBundle(tlsBundle), grpc.WithAuthority("x.test.example.com"))
	if err != nil {
		t.Fatalf("Error dialing: %v", err)
	}
	defer conn.Close()

	client := testgrpc.NewTestServiceClient(conn)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	_, err = client.EmptyCall(ctx, &testpb.Empty{})
	if wantErr := "test error"; err == nil || !strings.Contains(err.Error(), wantErr) {
		t.Errorf("EmptyCall() got err: %s, want err to contain: %s", err, wantErr)
	}
}

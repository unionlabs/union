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

package grpc

import (
	"context"
	"net"
	"testing"

	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/testdata"
)

func (s) TestClientConnAuthority(t *testing.T) {
	serverNameOverride := "over.write.server.name"
	creds, err := credentials.NewClientTLSFromFile(testdata.Path("x509/server_ca_cert.pem"), serverNameOverride)
	if err != nil {
		t.Fatalf("credentials.NewClientTLSFromFile(_, %q) failed: %v", err, serverNameOverride)
	}

	tests := []struct {
		name          string
		target        string
		opts          []DialOption
		wantAuthority string
	}{
		{
			name:          "default",
			target:        "Non-Existent.Server:8080",
			opts:          []DialOption{WithTransportCredentials(insecure.NewCredentials())},
			wantAuthority: "Non-Existent.Server:8080",
		},
		{
			name:          "override-via-creds",
			target:        "Non-Existent.Server:8080",
			opts:          []DialOption{WithTransportCredentials(creds)},
			wantAuthority: serverNameOverride,
		},
		{
			name:          "override-via-WithAuthority",
			target:        "Non-Existent.Server:8080",
			opts:          []DialOption{WithTransportCredentials(insecure.NewCredentials()), WithAuthority("authority-override")},
			wantAuthority: "authority-override",
		},
		{
			name:          "override-via-creds-and-WithAuthority",
			target:        "Non-Existent.Server:8080",
			opts:          []DialOption{WithTransportCredentials(creds), WithAuthority(serverNameOverride)},
			wantAuthority: serverNameOverride,
		},
		{
			name:          "unix relative",
			target:        "unix:sock.sock",
			opts:          []DialOption{WithTransportCredentials(insecure.NewCredentials())},
			wantAuthority: "localhost",
		},
		{
			name:   "unix relative with custom dialer",
			target: "unix:sock.sock",
			opts: []DialOption{WithTransportCredentials(insecure.NewCredentials()), WithContextDialer(func(ctx context.Context, addr string) (net.Conn, error) {
				return (&net.Dialer{}).DialContext(ctx, "", addr)
			})},
			wantAuthority: "localhost",
		},
		{
			name:          "unix absolute",
			target:        "unix:/sock.sock",
			opts:          []DialOption{WithTransportCredentials(insecure.NewCredentials())},
			wantAuthority: "localhost",
		},
		{
			name:   "unix absolute with custom dialer",
			target: "unix:///sock.sock",
			opts: []DialOption{WithTransportCredentials(insecure.NewCredentials()), WithContextDialer(func(ctx context.Context, addr string) (net.Conn, error) {
				return (&net.Dialer{}).DialContext(ctx, "", addr)
			})},
			wantAuthority: "localhost",
		},
		{
			name:          "localhost colon port",
			target:        "localhost:50051",
			opts:          []DialOption{WithTransportCredentials(insecure.NewCredentials())},
			wantAuthority: "localhost:50051",
		},
		{
			name:          "colon port",
			target:        ":50051",
			opts:          []DialOption{WithTransportCredentials(insecure.NewCredentials())},
			wantAuthority: "localhost:50051",
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			cc, err := Dial(test.target, test.opts...)
			if err != nil {
				t.Fatalf("Dial(%q) failed: %v", test.target, err)
			}
			defer cc.Close()
			if cc.authority != test.wantAuthority {
				t.Fatalf("cc.authority = %q, want %q", cc.authority, test.wantAuthority)
			}
		})
	}
}

func (s) TestClientConnAuthority_CredsAndDialOptionMismatch(t *testing.T) {
	serverNameOverride := "over.write.server.name"
	creds, err := credentials.NewClientTLSFromFile(testdata.Path("x509/server_ca_cert.pem"), serverNameOverride)
	if err != nil {
		t.Fatalf("credentials.NewClientTLSFromFile(_, %q) failed: %v", err, serverNameOverride)
	}
	opts := []DialOption{WithTransportCredentials(creds), WithAuthority("authority-override")}
	if cc, err := NewClient("Non-Existent.Server:8000", opts...); err == nil {
		cc.Close()
		t.Fatal("grpc.NewClient() succeeded when expected to fail")
	}
}

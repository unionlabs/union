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

package credentials

import (
	"crypto/tls"
	"crypto/x509"
	"encoding/pem"
	"net/url"
	"os"
	"testing"

	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/testdata"
)

const wantURI = "spiffe://foo.bar.com/client/workload/1"

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

func (s) TestSPIFFEIDFromState(t *testing.T) {
	tests := []struct {
		name string
		urls []*url.URL
		// If we expect a SPIFFE ID to be returned.
		wantID bool
	}{
		{
			name:   "empty URIs",
			urls:   []*url.URL{},
			wantID: false,
		},
		{
			name: "good SPIFFE ID",
			urls: []*url.URL{
				{
					Scheme:  "spiffe",
					Host:    "foo.bar.com",
					Path:    "workload/wl1",
					RawPath: "workload/wl1",
				},
			},
			wantID: true,
		},
		{
			name: "invalid host",
			urls: []*url.URL{
				{
					Scheme:  "spiffe",
					Host:    "",
					Path:    "workload/wl1",
					RawPath: "workload/wl1",
				},
			},
			wantID: false,
		},
		{
			name: "invalid path",
			urls: []*url.URL{
				{
					Scheme:  "spiffe",
					Host:    "foo.bar.com",
					Path:    "",
					RawPath: "",
				},
			},
			wantID: false,
		},
		{
			name: "large path",
			urls: []*url.URL{
				{
					Scheme:  "spiffe",
					Host:    "foo.bar.com",
					Path:    string(make([]byte, 2050)),
					RawPath: string(make([]byte, 2050)),
				},
			},
			wantID: false,
		},
		{
			name: "large host",
			urls: []*url.URL{
				{
					Scheme:  "spiffe",
					Host:    string(make([]byte, 256)),
					Path:    "workload/wl1",
					RawPath: "workload/wl1",
				},
			},
			wantID: false,
		},
		{
			name: "multiple URI SANs",
			urls: []*url.URL{
				{
					Scheme:  "spiffe",
					Host:    "foo.bar.com",
					Path:    "workload/wl1",
					RawPath: "workload/wl1",
				},
				{
					Scheme:  "spiffe",
					Host:    "bar.baz.com",
					Path:    "workload/wl2",
					RawPath: "workload/wl2",
				},
				{
					Scheme:  "https",
					Host:    "foo.bar.com",
					Path:    "workload/wl1",
					RawPath: "workload/wl1",
				},
			},
			wantID: false,
		},
		{
			name: "multiple URI SANs without SPIFFE ID",
			urls: []*url.URL{
				{
					Scheme:  "https",
					Host:    "foo.bar.com",
					Path:    "workload/wl1",
					RawPath: "workload/wl1",
				},
				{
					Scheme:  "ssh",
					Host:    "foo.bar.com",
					Path:    "workload/wl1",
					RawPath: "workload/wl1",
				},
			},
			wantID: false,
		},
		{
			name: "multiple URI SANs with one SPIFFE ID",
			urls: []*url.URL{
				{
					Scheme:  "spiffe",
					Host:    "foo.bar.com",
					Path:    "workload/wl1",
					RawPath: "workload/wl1",
				},
				{
					Scheme:  "https",
					Host:    "foo.bar.com",
					Path:    "workload/wl1",
					RawPath: "workload/wl1",
				},
			},
			wantID: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			state := tls.ConnectionState{PeerCertificates: []*x509.Certificate{{URIs: tt.urls}}}
			id := SPIFFEIDFromState(state)
			if got, want := id != nil, tt.wantID; got != want {
				t.Errorf("want wantID = %v, but SPIFFE ID is %v", want, id)
			}
		})
	}
}

func (s) TestSPIFFEIDFromCert(t *testing.T) {
	tests := []struct {
		name     string
		dataPath string
		// If we expect a SPIFFE ID to be returned.
		wantID bool
	}{
		{
			name:     "good certificate with SPIFFE ID",
			dataPath: "x509/spiffe_cert.pem",
			wantID:   true,
		},
		{
			name:     "bad certificate with SPIFFE ID and another URI",
			dataPath: "x509/multiple_uri_cert.pem",
			wantID:   false,
		},
		{
			name:     "certificate without SPIFFE ID",
			dataPath: "x509/client1_cert.pem",
			wantID:   false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			data, err := os.ReadFile(testdata.Path(tt.dataPath))
			if err != nil {
				t.Fatalf("os.ReadFile(%s) failed: %v", testdata.Path(tt.dataPath), err)
			}
			block, _ := pem.Decode(data)
			if block == nil {
				t.Fatalf("Failed to parse the certificate: byte block is nil")
			}
			cert, err := x509.ParseCertificate(block.Bytes)
			if err != nil {
				t.Fatalf("x509.ParseCertificate(%b) failed: %v", block.Bytes, err)
			}
			uri := SPIFFEIDFromCert(cert)
			if (uri != nil) != tt.wantID {
				t.Fatalf("wantID got and want mismatch, got %t, want %t", uri != nil, tt.wantID)
			}
			if uri != nil && uri.String() != wantURI {
				t.Fatalf("SPIFFE ID not expected, got %s, want %s", uri.String(), wantURI)
			}
		})
	}
}

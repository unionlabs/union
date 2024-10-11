// Copyright 2015 The Prometheus Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package api

import (
	"bytes"
	"context"
	"fmt"
	"net/http"
	"net/http/httptest"
	"net/url"
	"testing"
)

func TestConfig(t *testing.T) {
	c := Config{}
	if c.roundTripper() != DefaultRoundTripper {
		t.Fatalf("expected default roundtripper for nil RoundTripper field")
	}
}

func TestClientURL(t *testing.T) {
	tests := []struct {
		address  string
		endpoint string
		args     map[string]string
		expected string
	}{
		{
			address:  "http://localhost:9090",
			endpoint: "/test",
			expected: "http://localhost:9090/test",
		},
		{
			address:  "http://localhost",
			endpoint: "/test",
			expected: "http://localhost/test",
		},
		{
			address:  "http://localhost:9090",
			endpoint: "test",
			expected: "http://localhost:9090/test",
		},
		{
			address:  "http://localhost:9090/prefix",
			endpoint: "/test",
			expected: "http://localhost:9090/prefix/test",
		},
		{
			address:  "https://localhost:9090/",
			endpoint: "/test/",
			expected: "https://localhost:9090/test",
		},
		{
			address:  "http://localhost:9090",
			endpoint: "/test/:param",
			args: map[string]string{
				"param": "content",
			},
			expected: "http://localhost:9090/test/content",
		},
		{
			address:  "http://localhost:9090",
			endpoint: "/test/:param/more/:param",
			args: map[string]string{
				"param": "content",
			},
			expected: "http://localhost:9090/test/content/more/content",
		},
		{
			address:  "http://localhost:9090",
			endpoint: "/test/:param/more/:foo",
			args: map[string]string{
				"param": "content",
				"foo":   "bar",
			},
			expected: "http://localhost:9090/test/content/more/bar",
		},
		{
			address:  "http://localhost:9090",
			endpoint: "/test/:param",
			args: map[string]string{
				"nonexistent": "content",
			},
			expected: "http://localhost:9090/test/:param",
		},
	}

	for _, test := range tests {
		ep, err := url.Parse(test.address)
		if err != nil {
			t.Fatal(err)
		}

		hclient := &httpClient{
			endpoint: ep,
			client:   http.Client{Transport: DefaultRoundTripper},
		}

		u := hclient.URL(test.endpoint, test.args)
		if u.String() != test.expected {
			t.Errorf("unexpected result: got %s, want %s", u, test.expected)
			continue
		}
	}
}

// Serve any http request with a response of N KB of spaces.
type serveSpaces struct {
	sizeKB int
}

func (t serveSpaces) ServeHTTP(w http.ResponseWriter, req *http.Request) {
	kb := bytes.Repeat([]byte{' '}, 1024)
	for i := 0; i < t.sizeKB; i++ {
		w.Write(kb)
	}
}

func BenchmarkClient(b *testing.B) {
	b.ReportAllocs()
	ctx := context.Background()

	for _, sizeKB := range []int{4, 50, 1000, 2000} {
		b.Run(fmt.Sprintf("%dKB", sizeKB), func(b *testing.B) {
			testServer := httptest.NewServer(serveSpaces{sizeKB})
			defer testServer.Close()

			client, err := NewClient(Config{
				Address: testServer.URL,
			})
			if err != nil {
				b.Fatalf("Failed to initialize client: %v", err)
			}
			url, err := url.Parse(testServer.URL + "/prometheus/api/v1/query?query=up")
			if err != nil {
				b.Fatalf("Failed to parse url: %v", err)
			}
			req := &http.Request{
				URL: url,
			}
			b.ResetTimer()
			for i := 0; i < b.N; i++ {
				_, _, err := client.Do(ctx, req)
				if err != nil {
					b.Fatalf("Query failed: %v", err)
				}
			}
			b.StopTimer()
		})
	}
}

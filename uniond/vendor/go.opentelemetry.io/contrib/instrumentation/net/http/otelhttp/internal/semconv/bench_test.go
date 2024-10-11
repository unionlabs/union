// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package semconv

import (
	"net/http"
	"net/url"
	"testing"

	"go.opentelemetry.io/otel/attribute"
)

var benchHTTPServerRequestResults []attribute.KeyValue

// BenchmarkHTTPServerRequest allows comparison between different version of the HTTP server.
// To use an alternative start this test with OTEL_HTTP_CLIENT_COMPATIBILITY_MODE set to the
// version under test.
func BenchmarkHTTPServerRequest(b *testing.B) {
	// Request was generated from TestHTTPServerRequest request.
	req := &http.Request{
		Method: http.MethodGet,
		URL: &url.URL{
			Path: "/",
		},
		Proto:      "HTTP/1.1",
		ProtoMajor: 1,
		ProtoMinor: 1,
		Header: http.Header{
			"User-Agent":      []string{"Go-http-client/1.1"},
			"Accept-Encoding": []string{"gzip"},
		},
		Body:       http.NoBody,
		Host:       "127.0.0.1:39093",
		RemoteAddr: "127.0.0.1:38738",
		RequestURI: "/",
	}
	serv := NewHTTPServer()

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		benchHTTPServerRequestResults = serv.RequestTraceAttrs("", req)
	}
}

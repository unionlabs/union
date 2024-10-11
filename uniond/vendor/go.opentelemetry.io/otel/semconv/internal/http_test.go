// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package internal

import (
	"crypto/tls"
	"net/http"
	"net/url"
	"strings"
	"testing"

	"go.opentelemetry.io/otel/trace"

	"github.com/google/go-cmp/cmp"
	"github.com/stretchr/testify/assert"

	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
)

type tlsOption int

const (
	noTLS tlsOption = iota
	withTLS
)

var sc = &SemanticConventions{
	EnduserIDKey:                attribute.Key("enduser.id"),
	HTTPClientIPKey:             attribute.Key("http.client_ip"),
	HTTPFlavorKey:               attribute.Key("http.flavor"),
	HTTPHostKey:                 attribute.Key("http.host"),
	HTTPMethodKey:               attribute.Key("http.method"),
	HTTPRequestContentLengthKey: attribute.Key("http.request_content_length"),
	HTTPRouteKey:                attribute.Key("http.route"),
	HTTPSchemeHTTP:              attribute.String("http.scheme", "http"),
	HTTPSchemeHTTPS:             attribute.String("http.scheme", "https"),
	HTTPServerNameKey:           attribute.Key("http.server_name"),
	HTTPStatusCodeKey:           attribute.Key("http.status_code"),
	HTTPTargetKey:               attribute.Key("http.target"),
	HTTPURLKey:                  attribute.Key("http.url"),
	HTTPUserAgentKey:            attribute.Key("http.user_agent"),
	NetHostIPKey:                attribute.Key("net.host.ip"),
	NetHostNameKey:              attribute.Key("net.host.name"),
	NetHostPortKey:              attribute.Key("net.host.port"),
	NetPeerIPKey:                attribute.Key("net.peer.ip"),
	NetPeerNameKey:              attribute.Key("net.peer.name"),
	NetPeerPortKey:              attribute.Key("net.peer.port"),
	NetTransportIP:              attribute.String("net.transport", "ip"),
	NetTransportOther:           attribute.String("net.transport", "other"),
	NetTransportTCP:             attribute.String("net.transport", "ip_tcp"),
	NetTransportUDP:             attribute.String("net.transport", "ip_udp"),
	NetTransportUnix:            attribute.String("net.transport", "unix"),
}

func TestNetAttributesFromHTTPRequest(t *testing.T) {
	type testcase struct {
		name string

		network string

		method     string
		requestURI string
		proto      string
		remoteAddr string
		host       string
		url        *url.URL
		header     http.Header

		expected []attribute.KeyValue
	}
	testcases := []testcase{
		{
			name:       "stripped, tcp",
			network:    "tcp",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "ip_tcp"),
			},
		},
		{
			name:       "stripped, udp",
			network:    "udp",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "ip_udp"),
			},
		},
		{
			name:       "stripped, ip",
			network:    "ip",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "ip"),
			},
		},
		{
			name:       "stripped, unix",
			network:    "unix",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "unix"),
			},
		},
		{
			name:       "stripped, other",
			network:    "nih",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "other"),
			},
		},
		{
			name:       "with remote ipv4 and port",
			network:    "tcp",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "1.2.3.4:56",
			host:       "",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "ip_tcp"),
				attribute.String("net.peer.ip", "1.2.3.4"),
				attribute.Int("net.peer.port", 56),
			},
		},
		{
			name:       "with remote ipv6 and port",
			network:    "tcp",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "[fe80::0202:b3ff:fe1e:8329]:56",
			host:       "",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "ip_tcp"),
				attribute.String("net.peer.ip", "fe80::202:b3ff:fe1e:8329"),
				attribute.Int("net.peer.port", 56),
			},
		},
		{
			name:       "with remote ipv4-in-v6 and port",
			network:    "tcp",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "[::ffff:192.168.0.1]:56",
			host:       "",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "ip_tcp"),
				attribute.String("net.peer.ip", "192.168.0.1"),
				attribute.Int("net.peer.port", 56),
			},
		},
		{
			name:       "with remote name and port",
			network:    "tcp",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "example.com:56",
			host:       "",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "ip_tcp"),
				attribute.String("net.peer.name", "example.com"),
				attribute.Int("net.peer.port", 56),
			},
		},
		{
			name:       "with remote ipv4 only",
			network:    "tcp",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "1.2.3.4",
			host:       "",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "ip_tcp"),
				attribute.String("net.peer.ip", "1.2.3.4"),
			},
		},
		{
			name:       "with remote ipv6 only",
			network:    "tcp",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "fe80::0202:b3ff:fe1e:8329",
			host:       "",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "ip_tcp"),
				attribute.String("net.peer.ip", "fe80::202:b3ff:fe1e:8329"),
			},
		},
		{
			name:       "with remote ipv4_in_v6 only",
			network:    "tcp",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "::ffff:192.168.0.1", // section 2.5.5.2 of RFC4291
			host:       "",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "ip_tcp"),
				attribute.String("net.peer.ip", "192.168.0.1"),
			},
		},
		{
			name:       "with remote name only",
			network:    "tcp",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "example.com",
			host:       "",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "ip_tcp"),
				attribute.String("net.peer.name", "example.com"),
			},
		},
		{
			name:       "with remote port only",
			network:    "tcp",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: ":56",
			host:       "",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "ip_tcp"),
				attribute.Int("net.peer.port", 56),
			},
		},
		{
			name:       "with host name only",
			network:    "tcp",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "1.2.3.4:56",
			host:       "example.com",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "ip_tcp"),
				attribute.String("net.peer.ip", "1.2.3.4"),
				attribute.Int("net.peer.port", 56),
				attribute.String("net.host.name", "example.com"),
			},
		},
		{
			name:       "with host ipv4 only",
			network:    "tcp",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "1.2.3.4:56",
			host:       "4.3.2.1",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "ip_tcp"),
				attribute.String("net.peer.ip", "1.2.3.4"),
				attribute.Int("net.peer.port", 56),
				attribute.String("net.host.ip", "4.3.2.1"),
			},
		},
		{
			name:       "with host ipv6 only",
			network:    "tcp",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "1.2.3.4:56",
			host:       "fe80::0202:b3ff:fe1e:8329",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "ip_tcp"),
				attribute.String("net.peer.ip", "1.2.3.4"),
				attribute.Int("net.peer.port", 56),
				attribute.String("net.host.ip", "fe80::202:b3ff:fe1e:8329"),
			},
		},
		{
			name:       "with host name and port",
			network:    "tcp",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "1.2.3.4:56",
			host:       "example.com:78",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "ip_tcp"),
				attribute.String("net.peer.ip", "1.2.3.4"),
				attribute.Int("net.peer.port", 56),
				attribute.String("net.host.name", "example.com"),
				attribute.Int("net.host.port", 78),
			},
		},
		{
			name:       "with host ipv4 and port",
			network:    "tcp",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "1.2.3.4:56",
			host:       "4.3.2.1:78",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "ip_tcp"),
				attribute.String("net.peer.ip", "1.2.3.4"),
				attribute.Int("net.peer.port", 56),
				attribute.String("net.host.ip", "4.3.2.1"),
				attribute.Int("net.host.port", 78),
			},
		},
		{
			name:       "with host ipv6 and port",
			network:    "tcp",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "1.2.3.4:56",
			host:       "[fe80::202:b3ff:fe1e:8329]:78",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "ip_tcp"),
				attribute.String("net.peer.ip", "1.2.3.4"),
				attribute.Int("net.peer.port", 56),
				attribute.String("net.host.ip", "fe80::202:b3ff:fe1e:8329"),
				attribute.Int("net.host.port", 78),
			},
		},
		{
			name:       "with host name and bogus port",
			network:    "tcp",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "1.2.3.4:56",
			host:       "example.com:qwerty",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "ip_tcp"),
				attribute.String("net.peer.ip", "1.2.3.4"),
				attribute.Int("net.peer.port", 56),
				attribute.String("net.host.name", "example.com"),
			},
		},
		{
			name:       "with host ipv4 and bogus port",
			network:    "tcp",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "1.2.3.4:56",
			host:       "4.3.2.1:qwerty",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "ip_tcp"),
				attribute.String("net.peer.ip", "1.2.3.4"),
				attribute.Int("net.peer.port", 56),
				attribute.String("net.host.ip", "4.3.2.1"),
			},
		},
		{
			name:       "with host ipv6 and bogus port",
			network:    "tcp",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "1.2.3.4:56",
			host:       "[fe80::202:b3ff:fe1e:8329]:qwerty",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "ip_tcp"),
				attribute.String("net.peer.ip", "1.2.3.4"),
				attribute.Int("net.peer.port", 56),
				attribute.String("net.host.ip", "fe80::202:b3ff:fe1e:8329"),
			},
		},
		{
			name:       "with empty host and port",
			network:    "tcp",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "1.2.3.4:56",
			host:       ":80",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "ip_tcp"),
				attribute.String("net.peer.ip", "1.2.3.4"),
				attribute.Int("net.peer.port", 56),
				attribute.Int("net.host.port", 80),
			},
		},
		{
			name:       "with host ip and port in headers",
			network:    "tcp",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "1.2.3.4:56",
			host:       "",
			url: &url.URL{
				Path: "/user/123",
			},
			header: http.Header{
				"Host": []string{"4.3.2.1:78"},
			},
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "ip_tcp"),
				attribute.String("net.peer.ip", "1.2.3.4"),
				attribute.Int("net.peer.port", 56),
				attribute.String("net.host.ip", "4.3.2.1"),
				attribute.Int("net.host.port", 78),
			},
		},
		{
			name:       "with host ipv4 and port in url",
			network:    "tcp",
			method:     "GET",
			requestURI: "http://4.3.2.1:78/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "1.2.3.4:56",
			host:       "",
			url: &url.URL{
				Host: "4.3.2.1:78",
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "ip_tcp"),
				attribute.String("net.peer.ip", "1.2.3.4"),
				attribute.Int("net.peer.port", 56),
				attribute.String("net.host.ip", "4.3.2.1"),
				attribute.Int("net.host.port", 78),
			},
		},
		{
			name:       "with host ipv6 and port in url",
			network:    "tcp",
			method:     "GET",
			requestURI: "http://4.3.2.1:78/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "1.2.3.4:56",
			host:       "",
			url: &url.URL{
				Host: "[fe80::202:b3ff:fe1e:8329]:78",
				Path: "/user/123",
			},
			header: nil,
			expected: []attribute.KeyValue{
				attribute.String("net.transport", "ip_tcp"),
				attribute.String("net.peer.ip", "1.2.3.4"),
				attribute.Int("net.peer.port", 56),
				attribute.String("net.host.ip", "fe80::202:b3ff:fe1e:8329"),
				attribute.Int("net.host.port", 78),
			},
		},
	}
	for _, tc := range testcases {
		t.Run(tc.name, func(t *testing.T) {
			r := testRequest(tc.method, tc.requestURI, tc.proto, tc.remoteAddr, tc.host, tc.url, tc.header, noTLS)
			got := sc.NetAttributesFromHTTPRequest(tc.network, r)
			if diff := cmp.Diff(
				tc.expected,
				got,
				cmp.AllowUnexported(attribute.Value{})); diff != "" {
				t.Fatalf("attributes differ: diff %+v,", diff)
			}
		})
	}
}

func TestEndUserAttributesFromHTTPRequest(t *testing.T) {
	r := testRequest("GET", "/user/123", "HTTP/1.1", "", "", nil, http.Header{}, withTLS)
	var expected []attribute.KeyValue
	got := sc.EndUserAttributesFromHTTPRequest(r)
	assert.ElementsMatch(t, expected, got)
	r.SetBasicAuth("admin", "password")
	expected = []attribute.KeyValue{attribute.String("enduser.id", "admin")}
	got = sc.EndUserAttributesFromHTTPRequest(r)
	assert.ElementsMatch(t, expected, got)
}

func TestHTTPServerAttributesFromHTTPRequest(t *testing.T) {
	type testcase struct {
		name string

		serverName string
		route      string

		method        string
		requestURI    string
		proto         string
		remoteAddr    string
		host          string
		url           *url.URL
		header        http.Header
		tls           tlsOption
		contentLength int64

		expected []attribute.KeyValue
	}
	testcases := []testcase{
		{
			name:       "stripped",
			serverName: "",
			route:      "",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			tls:    noTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.target", "/user/123"),
				attribute.String("http.scheme", "http"),
				attribute.String("http.flavor", "1.0"),
			},
		},
		{
			name:       "with server name",
			serverName: "my-server-name",
			route:      "",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			tls:    noTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.target", "/user/123"),
				attribute.String("http.scheme", "http"),
				attribute.String("http.flavor", "1.0"),
				attribute.String("http.server_name", "my-server-name"),
			},
		},
		{
			name:       "with tls",
			serverName: "my-server-name",
			route:      "",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			tls:    withTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.target", "/user/123"),
				attribute.String("http.scheme", "https"),
				attribute.String("http.flavor", "1.0"),
				attribute.String("http.server_name", "my-server-name"),
			},
		},
		{
			name:       "with route",
			serverName: "my-server-name",
			route:      "/user/:id",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			tls:    withTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.target", "/user/123"),
				attribute.String("http.scheme", "https"),
				attribute.String("http.flavor", "1.0"),
				attribute.String("http.server_name", "my-server-name"),
				attribute.String("http.route", "/user/:id"),
			},
		},
		{
			name:       "with host",
			serverName: "my-server-name",
			route:      "/user/:id",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "example.com",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			tls:    withTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.target", "/user/123"),
				attribute.String("http.scheme", "https"),
				attribute.String("http.flavor", "1.0"),
				attribute.String("http.server_name", "my-server-name"),
				attribute.String("http.route", "/user/:id"),
				attribute.String("http.host", "example.com"),
			},
		},
		{
			name:       "with host fallback",
			serverName: "my-server-name",
			route:      "/user/:id",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "",
			url: &url.URL{
				Host: "example.com",
				Path: "/user/123",
			},
			header: nil,
			tls:    withTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.target", "/user/123"),
				attribute.String("http.scheme", "https"),
				attribute.String("http.flavor", "1.0"),
				attribute.String("http.server_name", "my-server-name"),
				attribute.String("http.route", "/user/:id"),
				attribute.String("http.host", "example.com"),
			},
		},
		{
			name:       "with user agent",
			serverName: "my-server-name",
			route:      "/user/:id",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "example.com",
			url: &url.URL{
				Path: "/user/123",
			},
			header: http.Header{
				"User-Agent": []string{"foodownloader"},
			},
			tls: withTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.target", "/user/123"),
				attribute.String("http.scheme", "https"),
				attribute.String("http.flavor", "1.0"),
				attribute.String("http.server_name", "my-server-name"),
				attribute.String("http.route", "/user/:id"),
				attribute.String("http.host", "example.com"),
				attribute.String("http.user_agent", "foodownloader"),
			},
		},
		{
			name:       "with proxy info",
			serverName: "my-server-name",
			route:      "/user/:id",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "example.com",
			url: &url.URL{
				Path: "/user/123",
			},
			header: http.Header{
				"User-Agent":      []string{"foodownloader"},
				"X-Forwarded-For": []string{"203.0.113.195, 70.41.3.18, 150.172.238.178"},
			},
			tls: withTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.target", "/user/123"),
				attribute.String("http.scheme", "https"),
				attribute.String("http.flavor", "1.0"),
				attribute.String("http.server_name", "my-server-name"),
				attribute.String("http.route", "/user/:id"),
				attribute.String("http.host", "example.com"),
				attribute.String("http.user_agent", "foodownloader"),
				attribute.String("http.client_ip", "203.0.113.195"),
			},
		},
		{
			name:       "with http 1.1",
			serverName: "my-server-name",
			route:      "/user/:id",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.1",
			remoteAddr: "",
			host:       "example.com",
			url: &url.URL{
				Path: "/user/123",
			},
			header: http.Header{
				"User-Agent":      []string{"foodownloader"},
				"X-Forwarded-For": []string{"1.2.3.4"},
			},
			tls: withTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.target", "/user/123"),
				attribute.String("http.scheme", "https"),
				attribute.String("http.flavor", "1.1"),
				attribute.String("http.server_name", "my-server-name"),
				attribute.String("http.route", "/user/:id"),
				attribute.String("http.host", "example.com"),
				attribute.String("http.user_agent", "foodownloader"),
				attribute.String("http.client_ip", "1.2.3.4"),
			},
		},
		{
			name:       "with http 2",
			serverName: "my-server-name",
			route:      "/user/:id",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/2.0",
			remoteAddr: "",
			host:       "example.com",
			url: &url.URL{
				Path: "/user/123",
			},
			header: http.Header{
				"User-Agent":      []string{"foodownloader"},
				"X-Forwarded-For": []string{"1.2.3.4"},
			},
			tls: withTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.target", "/user/123"),
				attribute.String("http.scheme", "https"),
				attribute.String("http.flavor", "2"),
				attribute.String("http.server_name", "my-server-name"),
				attribute.String("http.route", "/user/:id"),
				attribute.String("http.host", "example.com"),
				attribute.String("http.user_agent", "foodownloader"),
				attribute.String("http.client_ip", "1.2.3.4"),
			},
		},
		{
			name:          "with content length",
			method:        "GET",
			requestURI:    "/user/123",
			contentLength: 100,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.target", "/user/123"),
				attribute.String("http.scheme", "http"),
				attribute.Int64("http.request_content_length", 100),
			},
		},
	}
	for idx, tc := range testcases {
		r := testRequest(tc.method, tc.requestURI, tc.proto, tc.remoteAddr, tc.host, tc.url, tc.header, tc.tls)
		r.ContentLength = tc.contentLength
		got := sc.HTTPServerAttributesFromHTTPRequest(tc.serverName, tc.route, r)
		assertElementsMatch(t, tc.expected, got, "testcase %d - %s", idx, tc.name)
	}
}

func TestHTTPAttributesFromHTTPStatusCode(t *testing.T) {
	expected := []attribute.KeyValue{
		attribute.Int("http.status_code", 404),
	}
	got := sc.HTTPAttributesFromHTTPStatusCode(http.StatusNotFound)
	assertElementsMatch(t, expected, got, "with valid HTTP status code")
	assert.ElementsMatch(t, expected, got)
	expected = []attribute.KeyValue{
		attribute.Int("http.status_code", 499),
	}
	got = sc.HTTPAttributesFromHTTPStatusCode(499)
	assertElementsMatch(t, expected, got, "with invalid HTTP status code")
}

func TestSpanStatusFromHTTPStatusCode(t *testing.T) {
	for code := 0; code < 1000; code++ {
		expected := getExpectedCodeForHTTPCode(code, trace.SpanKindClient)
		got, msg := SpanStatusFromHTTPStatusCode(code)
		assert.Equalf(t, expected, got, "%s vs %s", expected, got)

		_, valid := validateHTTPStatusCode(code)
		if !valid {
			assert.NotEmpty(t, msg, "message should be set if error cannot be inferred from code")
		} else {
			assert.Empty(t, msg, "message should not be set if error can be inferred from code")
		}
	}
}

func TestSpanStatusFromHTTPStatusCodeAndSpanKind(t *testing.T) {
	for code := 0; code < 1000; code++ {
		expected := getExpectedCodeForHTTPCode(code, trace.SpanKindClient)
		got, msg := SpanStatusFromHTTPStatusCodeAndSpanKind(code, trace.SpanKindClient)
		assert.Equalf(t, expected, got, "%s vs %s", expected, got)

		_, valid := validateHTTPStatusCode(code)
		if !valid {
			assert.NotEmpty(t, msg, "message should be set if error cannot be inferred from code")
		} else {
			assert.Empty(t, msg, "message should not be set if error can be inferred from code")
		}
	}
	code, _ := SpanStatusFromHTTPStatusCodeAndSpanKind(400, trace.SpanKindServer)
	assert.Equalf(t, codes.Unset, code, "message should be set if error cannot be inferred from code")
}

func getExpectedCodeForHTTPCode(code int, spanKind trace.SpanKind) codes.Code {
	if http.StatusText(code) == "" {
		return codes.Error
	}
	switch code {
	case
		http.StatusUnauthorized,
		http.StatusForbidden,
		http.StatusNotFound,
		http.StatusTooManyRequests,
		http.StatusNotImplemented,
		http.StatusServiceUnavailable,
		http.StatusGatewayTimeout:
		return codes.Error
	}
	category := code / 100
	if category > 0 && category < 4 {
		return codes.Unset
	}
	if spanKind == trace.SpanKindServer && category == 4 {
		return codes.Unset
	}
	return codes.Error
}

func assertElementsMatch(t *testing.T, expected, got []attribute.KeyValue, format string, args ...interface{}) {
	if !assert.ElementsMatchf(t, expected, got, format, args...) {
		t.Log("expected:", kvStr(expected))
		t.Log("got:", kvStr(got))
	}
}

func testRequest(method, requestURI, proto, remoteAddr, host string, u *url.URL, header http.Header, tlsopt tlsOption) *http.Request {
	major, minor := protoToInts(proto)
	var tlsConn *tls.ConnectionState
	switch tlsopt {
	case noTLS:
	case withTLS:
		tlsConn = &tls.ConnectionState{}
	}
	return &http.Request{
		Method:     method,
		URL:        u,
		Proto:      proto,
		ProtoMajor: major,
		ProtoMinor: minor,
		Header:     header,
		Host:       host,
		RemoteAddr: remoteAddr,
		RequestURI: requestURI,
		TLS:        tlsConn,
	}
}

func protoToInts(proto string) (int, int) {
	switch proto {
	case "HTTP/1.0":
		return 1, 0
	case "HTTP/1.1":
		return 1, 1
	case "HTTP/2.0":
		return 2, 0
	}
	// invalid proto
	return 13, 42
}

func kvStr(kvs []attribute.KeyValue) string {
	sb := strings.Builder{}
	_, _ = sb.WriteRune('[')
	for idx, attr := range kvs {
		if idx > 0 {
			_, _ = sb.WriteString(", ")
		}
		_, _ = sb.WriteString((string)(attr.Key))
		_, _ = sb.WriteString(": ")
		_, _ = sb.WriteString(attr.Value.Emit())
	}
	_, _ = sb.WriteRune(']')
	return sb.String()
}

func TestHTTPClientAttributesFromHTTPRequest(t *testing.T) {
	testCases := []struct {
		name string

		method        string
		requestURI    string
		proto         string
		remoteAddr    string
		host          string
		url           *url.URL
		header        http.Header
		tls           tlsOption
		contentLength int64

		expected []attribute.KeyValue
	}{
		{
			name:       "stripped",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			tls:    noTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.url", "/user/123"),
				attribute.String("http.scheme", "http"),
				attribute.String("http.flavor", "1.0"),
			},
		},
		{
			name:       "with tls",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			tls:    withTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.url", "/user/123"),
				attribute.String("http.scheme", "https"),
				attribute.String("http.flavor", "1.0"),
			},
		},
		{
			name:       "with host",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "example.com",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			tls:    withTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.url", "/user/123"),
				attribute.String("http.scheme", "https"),
				attribute.String("http.flavor", "1.0"),
				attribute.String("http.host", "example.com"),
			},
		},
		{
			name:       "with host fallback",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "",
			url: &url.URL{
				Scheme: "https",
				Host:   "example.com",
				Path:   "/user/123",
			},
			header: nil,
			tls:    withTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.url", "https://example.com/user/123"),
				attribute.String("http.scheme", "https"),
				attribute.String("http.flavor", "1.0"),
				attribute.String("http.host", "example.com"),
			},
		},
		{
			name:       "with user agent",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "example.com",
			url: &url.URL{
				Path: "/user/123",
			},
			header: http.Header{
				"User-Agent": []string{"foodownloader"},
			},
			tls: withTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.url", "/user/123"),
				attribute.String("http.scheme", "https"),
				attribute.String("http.flavor", "1.0"),
				attribute.String("http.host", "example.com"),
				attribute.String("http.user_agent", "foodownloader"),
			},
		},
		{
			name:       "with http 1.1",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.1",
			remoteAddr: "",
			host:       "example.com",
			url: &url.URL{
				Path: "/user/123",
			},
			header: http.Header{
				"User-Agent": []string{"foodownloader"},
			},
			tls: withTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.url", "/user/123"),
				attribute.String("http.scheme", "https"),
				attribute.String("http.flavor", "1.1"),
				attribute.String("http.host", "example.com"),
				attribute.String("http.user_agent", "foodownloader"),
			},
		},
		{
			name:       "with http 2",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/2.0",
			remoteAddr: "",
			host:       "example.com",
			url: &url.URL{
				Path: "/user/123",
			},
			header: http.Header{
				"User-Agent": []string{"foodownloader"},
			},
			tls: withTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.url", "/user/123"),
				attribute.String("http.scheme", "https"),
				attribute.String("http.flavor", "2"),
				attribute.String("http.host", "example.com"),
				attribute.String("http.user_agent", "foodownloader"),
			},
		},
		{
			name:   "with content length",
			method: "GET",
			url: &url.URL{
				Path: "/user/123",
			},
			contentLength: 100,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.url", "/user/123"),
				attribute.String("http.scheme", "http"),
				attribute.Int64("http.request_content_length", 100),
			},
		},
		{
			name:   "with empty method (fallback to GET)",
			method: "",
			url: &url.URL{
				Path: "/user/123",
			},
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.url", "/user/123"),
				attribute.String("http.scheme", "http"),
			},
		},
		{
			name:   "authentication information is stripped",
			method: "",
			url: &url.URL{
				Path: "/user/123",
				User: url.UserPassword("foo", "bar"),
			},
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.url", "/user/123"),
				attribute.String("http.scheme", "http"),
			},
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			r := testRequest(tc.method, tc.requestURI, tc.proto, tc.remoteAddr, tc.host, tc.url, tc.header, tc.tls)
			r.ContentLength = tc.contentLength
			got := sc.HTTPClientAttributesFromHTTPRequest(r)
			assert.ElementsMatch(t, tc.expected, got)
		})
	}
}

func TestHTTPServerMetricAttributesFromHTTPRequest(t *testing.T) {
	type testcase struct {
		name          string
		serverName    string
		method        string
		requestURI    string
		proto         string
		remoteAddr    string
		host          string
		url           *url.URL
		header        http.Header
		tls           tlsOption
		contentLength int64
		expected      []attribute.KeyValue
	}
	testcases := []testcase{
		{
			name:       "stripped",
			serverName: "",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			tls:    noTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.scheme", "http"),
				attribute.String("http.flavor", "1.0"),
			},
		},
		{
			name:       "with server name",
			serverName: "my-server-name",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			tls:    noTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.scheme", "http"),
				attribute.String("http.flavor", "1.0"),
				attribute.String("http.server_name", "my-server-name"),
			},
		},
		{
			name:       "with tls",
			serverName: "my-server-name",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			tls:    withTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.scheme", "https"),
				attribute.String("http.flavor", "1.0"),
				attribute.String("http.server_name", "my-server-name"),
			},
		},
		{
			name:       "with route",
			serverName: "my-server-name",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			tls:    withTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.scheme", "https"),
				attribute.String("http.flavor", "1.0"),
				attribute.String("http.server_name", "my-server-name"),
			},
		},
		{
			name:       "with host",
			serverName: "my-server-name",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "example.com",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			tls:    withTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.scheme", "https"),
				attribute.String("http.flavor", "1.0"),
				attribute.String("http.server_name", "my-server-name"),
				attribute.String("http.host", "example.com"),
			},
		},
		{
			name:       "with host fallback",
			serverName: "my-server-name",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "",
			url: &url.URL{
				Host: "example.com",
				Path: "/user/123",
			},
			header: nil,
			tls:    withTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.scheme", "https"),
				attribute.String("http.flavor", "1.0"),
				attribute.String("http.server_name", "my-server-name"),
				attribute.String("http.host", "example.com"),
			},
		},
		{
			name:       "with user agent",
			serverName: "my-server-name",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "example.com",
			url: &url.URL{
				Path: "/user/123",
			},
			header: http.Header{
				"User-Agent": []string{"foodownloader"},
			},
			tls: withTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.scheme", "https"),
				attribute.String("http.flavor", "1.0"),
				attribute.String("http.server_name", "my-server-name"),
				attribute.String("http.host", "example.com"),
			},
		},
		{
			name:       "with proxy info",
			serverName: "my-server-name",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "example.com",
			url: &url.URL{
				Path: "/user/123",
			},
			header: http.Header{
				"User-Agent":      []string{"foodownloader"},
				"X-Forwarded-For": []string{"203.0.113.195, 70.41.3.18, 150.172.238.178"},
			},
			tls: withTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.scheme", "https"),
				attribute.String("http.flavor", "1.0"),
				attribute.String("http.server_name", "my-server-name"),
				attribute.String("http.host", "example.com"),
			},
		},
		{
			name:       "with http 1.1",
			serverName: "my-server-name",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.1",
			remoteAddr: "",
			host:       "example.com",
			url: &url.URL{
				Path: "/user/123",
			},
			header: http.Header{
				"User-Agent":      []string{"foodownloader"},
				"X-Forwarded-For": []string{"1.2.3.4"},
			},
			tls: withTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.scheme", "https"),
				attribute.String("http.flavor", "1.1"),
				attribute.String("http.server_name", "my-server-name"),
				attribute.String("http.host", "example.com"),
			},
		},
		{
			name:       "with http 2",
			serverName: "my-server-name",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/2.0",
			remoteAddr: "",
			host:       "example.com",
			url: &url.URL{
				Path: "/user/123",
			},
			header: http.Header{
				"User-Agent":      []string{"foodownloader"},
				"X-Forwarded-For": []string{"1.2.3.4"},
			},
			tls: withTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.scheme", "https"),
				attribute.String("http.flavor", "2"),
				attribute.String("http.server_name", "my-server-name"),
				attribute.String("http.host", "example.com"),
			},
		},
	}
	for idx, tc := range testcases {
		r := testRequest(tc.method, tc.requestURI, tc.proto, tc.remoteAddr, tc.host, tc.url, tc.header, tc.tls)
		r.ContentLength = tc.contentLength
		got := sc.HTTPServerMetricAttributesFromHTTPRequest(tc.serverName, r)
		assertElementsMatch(t, tc.expected, got, "testcase %d - %s", idx, tc.name)
	}
}

func TestHttpBasicAttributesFromHTTPRequest(t *testing.T) {
	type testcase struct {
		name          string
		method        string
		requestURI    string
		proto         string
		remoteAddr    string
		host          string
		url           *url.URL
		header        http.Header
		tls           tlsOption
		contentLength int64
		expected      []attribute.KeyValue
	}
	testcases := []testcase{
		{
			name:       "stripped",
			method:     "GET",
			requestURI: "/user/123",
			proto:      "HTTP/1.0",
			remoteAddr: "",
			host:       "example.com",
			url: &url.URL{
				Path: "/user/123",
			},
			header: nil,
			tls:    noTLS,
			expected: []attribute.KeyValue{
				attribute.String("http.method", "GET"),
				attribute.String("http.scheme", "http"),
				attribute.String("http.flavor", "1.0"),
				attribute.String("http.host", "example.com"),
			},
		},
	}
	for idx, tc := range testcases {
		r := testRequest(tc.method, tc.requestURI, tc.proto, tc.remoteAddr, tc.host, tc.url, tc.header, tc.tls)
		r.ContentLength = tc.contentLength
		got := sc.httpBasicAttributesFromHTTPRequest(r)
		assertElementsMatch(t, tc.expected, got, "testcase %d - %s", idx, tc.name)
	}
}

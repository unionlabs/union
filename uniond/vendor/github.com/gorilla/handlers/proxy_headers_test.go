package handlers

import (
	"net/http"
	"net/http/httptest"
	"testing"
)

type headerTable struct {
	key      string // header key
	val      string // header val
	expected string // expected result
}

func TestGetIP(t *testing.T) {
	headers := []headerTable{
		{xForwardedFor, "8.8.8.8", "8.8.8.8"},                                   // Single address
		{xForwardedFor, "8.8.8.8, 8.8.4.4", "8.8.8.8"},                          // Multiple
		{xForwardedFor, "[2001:db8:cafe::17]:4711", "[2001:db8:cafe::17]:4711"}, // IPv6 address
		{xForwardedFor, "", ""},                                                 // None
		{xRealIP, "8.8.8.8", "8.8.8.8"},                                         // Single address
		{xRealIP, "8.8.8.8, 8.8.4.4", "8.8.8.8, 8.8.4.4"},                       // Multiple
		{xRealIP, "[2001:db8:cafe::17]:4711", "[2001:db8:cafe::17]:4711"},       // IPv6 address
		{xRealIP, "", ""},                       // None
		{forwarded, `for="_gazonk"`, "_gazonk"}, // Hostname
		{forwarded, `For="[2001:db8:cafe::17]:4711`, `[2001:db8:cafe::17]:4711`},      // IPv6 address
		{forwarded, `for=192.0.2.60;proto=http;by=203.0.113.43`, `192.0.2.60`},        // Multiple params
		{forwarded, `for=192.0.2.43, for=198.51.100.17`, "192.0.2.43"},                // Multiple params
		{forwarded, `for="workstation.local",for=198.51.100.17`, "workstation.local"}, // Hostname
	}

	for _, v := range headers {
		req := &http.Request{
			Header: http.Header{
				v.key: []string{v.val},
			},
		}
		res := getIP(req)
		if res != v.expected {
			t.Fatalf("wrong header for %s: got %s want %s", v.key, res,
				v.expected)
		}
	}
}

func TestGetScheme(t *testing.T) {
	headers := []headerTable{
		{xForwardedProto, "https", "https"},
		{xForwardedProto, "http", "http"},
		{xForwardedProto, "HTTP", "http"},
		{xForwardedScheme, "https", "https"},
		{xForwardedScheme, "http", "http"},
		{xForwardedScheme, "HTTP", "http"},
		{forwarded, `For="[2001:db8:cafe::17]:4711`, ""},                      // No proto
		{forwarded, `for=192.0.2.43, for=198.51.100.17;proto=https`, "https"}, // Multiple params before proto
		{forwarded, `for=172.32.10.15; proto=https;by=127.0.0.1`, "https"},    // Space before proto
		{forwarded, `for=192.0.2.60;proto=http;by=203.0.113.43`, "http"},      // Multiple params
	}

	for _, v := range headers {
		req := &http.Request{
			Header: http.Header{
				v.key: []string{v.val},
			},
		}
		res := getScheme(req)
		if res != v.expected {
			t.Fatalf("wrong header for %s: got %s want %s", v.key, res,
				v.expected)
		}
	}
}

// Test the middleware end-to-end.
func TestProxyHeaders(t *testing.T) {
	rr := httptest.NewRecorder()
	r := newRequest(http.MethodGet, "/")

	r.Header.Set(xForwardedFor, "8.8.8.8")
	r.Header.Set(xForwardedProto, "https")
	r.Header.Set(xForwardedHost, "google.com")
	var (
		addr  string
		proto string
		host  string
	)
	ProxyHeaders(http.HandlerFunc(
		func(w http.ResponseWriter, r *http.Request) {
			addr = r.RemoteAddr
			proto = r.URL.Scheme
			host = r.Host
		})).ServeHTTP(rr, r)

	if rr.Code != http.StatusOK {
		t.Fatalf("bad status: got %d want %d", rr.Code, http.StatusOK)
	}

	if addr != r.Header.Get(xForwardedFor) {
		t.Fatalf("wrong address: got %s want %s", addr,
			r.Header.Get(xForwardedFor))
	}

	if proto != r.Header.Get(xForwardedProto) {
		t.Fatalf("wrong address: got %s want %s", proto,
			r.Header.Get(xForwardedProto))
	}
	if host != r.Header.Get(xForwardedHost) {
		t.Fatalf("wrong address: got %s want %s", host,
			r.Header.Get(xForwardedHost))
	}
}

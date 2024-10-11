//go:build go1.23

package traceutils

import (
	"net/http"
	"net/url"
	"testing"
)

func TestGetHTTPSpanName(t *testing.T) {
	tests := []struct {
		name string
		got  string
		want string
	}{
		{
			name: "Without Pattern",
			got:  GetHTTPSpanName(&http.Request{Method: "GET", URL: &url.URL{Path: "/"}}),
			want: "GET /",
		},
		{
			name: "Pattern with method",
			got:  GetHTTPSpanName(&http.Request{Method: "GET", URL: &url.URL{Path: "/"}, Pattern: "POST /foo/{bar}"}),
			want: "POST /foo/{bar}",
		},
		{
			name: "Pattern without method",
			got:  GetHTTPSpanName(&http.Request{Method: "GET", URL: &url.URL{Path: "/"}, Pattern: "/foo/{bar}"}),
			want: "GET /foo/{bar}",
		},
		{
			name: "Pattern without slash",
			got:  GetHTTPSpanName(&http.Request{Method: "GET", URL: &url.URL{Path: "/"}, Pattern: "example.com/"}),
			want: "GET example.com/",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if tt.got != tt.want {
				t.Errorf("got %q; want %q", tt.got, tt.want)
			}
		})
	}
}

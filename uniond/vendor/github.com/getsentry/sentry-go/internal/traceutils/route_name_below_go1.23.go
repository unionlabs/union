//go:build !go1.23

package traceutils

import "net/http"

// GetHTTPSpanName grab needed fields from *http.Request to generate a span name for `http.server` span op.
func GetHTTPSpanName(r *http.Request) string {
	return r.Method + " " + r.URL.Path
}

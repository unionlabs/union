//go:build go1.23

package traceutils

import (
	"net/http"
	"strings"
)

// GetHTTPSpanName grab needed fields from *http.Request to generate a span name for `http.server` span op.
func GetHTTPSpanName(r *http.Request) string {
	if r.Pattern != "" {
		// If value does not start with HTTP methods, add them.
		// The method and the path should be separated by a space.
		if parts := strings.SplitN(r.Pattern, " ", 2); len(parts) == 1 {
			return r.Method + " " + r.Pattern
		}

		return r.Pattern
	}

	return r.Method + " " + r.URL.Path
}

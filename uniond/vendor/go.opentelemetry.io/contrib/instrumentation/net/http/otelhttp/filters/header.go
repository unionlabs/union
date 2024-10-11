// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package filters // import "go.opentelemetry.io/contrib/instrumentation/net/http/otelhttp/filters"

import (
	"net/http"
	"net/textproto"
	"strings"

	"go.opentelemetry.io/contrib/instrumentation/net/http/otelhttp"
)

// Header returns a Filter that returns true if the request
// includes a header k with a value equal to v.
func Header(k, v string) otelhttp.Filter {
	return func(r *http.Request) bool {
		for _, hv := range r.Header[textproto.CanonicalMIMEHeaderKey(k)] {
			if v == hv {
				return true
			}
		}
		return false
	}
}

// HeaderContains returns a Filter that returns true if the request
// includes a header k with a value that contains v.
func HeaderContains(k, v string) otelhttp.Filter {
	return func(r *http.Request) bool {
		for _, hv := range r.Header[textproto.CanonicalMIMEHeaderKey(k)] {
			if strings.Contains(hv, v) {
				return true
			}
		}
		return false
	}
}

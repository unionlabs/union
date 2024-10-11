// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// Package filters provides a set of filters useful with the
// otelhttp.WithFilter() option to control which inbound requests are traced.
package filters // import "go.opentelemetry.io/contrib/instrumentation/net/http/otelhttp/filters"

import (
	"net/http"
	"strings"

	"go.opentelemetry.io/contrib/instrumentation/net/http/otelhttp"
)

// Any takes a list of Filters and returns a Filter that
// returns true if any Filter in the list returns true.
func Any(fs ...otelhttp.Filter) otelhttp.Filter {
	return func(r *http.Request) bool {
		for _, f := range fs {
			if f(r) {
				return true
			}
		}
		return false
	}
}

// All takes a list of Filters and returns a Filter that
// returns true only if all Filters in the list return true.
func All(fs ...otelhttp.Filter) otelhttp.Filter {
	return func(r *http.Request) bool {
		for _, f := range fs {
			if !f(r) {
				return false
			}
		}
		return true
	}
}

// None takes a list of Filters and returns a Filter that returns
// true only if none of the Filters in the list return true.
func None(fs ...otelhttp.Filter) otelhttp.Filter {
	return func(r *http.Request) bool {
		for _, f := range fs {
			if f(r) {
				return false
			}
		}
		return true
	}
}

// Not provides a convenience mechanism for inverting a Filter.
func Not(f otelhttp.Filter) otelhttp.Filter {
	return func(r *http.Request) bool {
		return !f(r)
	}
}

// Hostname returns a Filter that returns true if the request's
// hostname matches the provided string.
func Hostname(h string) otelhttp.Filter {
	return func(r *http.Request) bool {
		return r.URL.Hostname() == h
	}
}

// Path returns a Filter that returns true if the request's
// path matches the provided string.
func Path(p string) otelhttp.Filter {
	return func(r *http.Request) bool {
		return r.URL.Path == p
	}
}

// PathPrefix returns a Filter that returns true if the request's
// path starts with the provided string.
func PathPrefix(p string) otelhttp.Filter {
	return func(r *http.Request) bool {
		return strings.HasPrefix(r.URL.Path, p)
	}
}

// Query returns a Filter that returns true if the request
// includes a query parameter k with a value equal to v.
func Query(k, v string) otelhttp.Filter {
	return func(r *http.Request) bool {
		for _, qv := range r.URL.Query()[k] {
			if v == qv {
				return true
			}
		}
		return false
	}
}

// QueryContains returns a Filter that returns true if the request
// includes a query parameter k with a value that contains v.
func QueryContains(k, v string) otelhttp.Filter {
	return func(r *http.Request) bool {
		for _, qv := range r.URL.Query()[k] {
			if strings.Contains(qv, v) {
				return true
			}
		}
		return false
	}
}

// Method returns a Filter that returns true if the request
// method is equal to the provided value.
func Method(m string) otelhttp.Filter {
	return func(r *http.Request) bool {
		return m == r.Method
	}
}

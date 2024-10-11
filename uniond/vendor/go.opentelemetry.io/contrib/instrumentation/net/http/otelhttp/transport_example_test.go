// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package otelhttp

import (
	"net/http"
)

func ExampleNewTransport() {
	// Create an http.Client that uses the (ot)http.Transport
	// wrapped around the http.DefaultTransport
	_ = http.Client{
		Transport: NewTransport(http.DefaultTransport),
	}
}

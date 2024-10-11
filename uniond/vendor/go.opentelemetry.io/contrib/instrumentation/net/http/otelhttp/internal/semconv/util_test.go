// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package semconv

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestSplitHostPort(t *testing.T) {
	tests := []struct {
		hostport string
		host     string
		port     int
	}{
		{"", "", -1},
		{":8080", "", 8080},
		{"127.0.0.1", "127.0.0.1", -1},
		{"www.example.com", "www.example.com", -1},
		{"127.0.0.1%25en0", "127.0.0.1%25en0", -1},
		{"[]", "", -1}, // Ensure this doesn't panic.
		{"[fe80::1", "", -1},
		{"[fe80::1]", "fe80::1", -1},
		{"[fe80::1%25en0]", "fe80::1%25en0", -1},
		{"[fe80::1]:8080", "fe80::1", 8080},
		{"[fe80::1]::", "", -1}, // Too many colons.
		{"127.0.0.1:", "127.0.0.1", -1},
		{"127.0.0.1:port", "127.0.0.1", -1},
		{"127.0.0.1:8080", "127.0.0.1", 8080},
		{"www.example.com:8080", "www.example.com", 8080},
		{"127.0.0.1%25en0:8080", "127.0.0.1%25en0", 8080},
	}

	for _, test := range tests {
		h, p := splitHostPort(test.hostport)
		assert.Equal(t, test.host, h, test.hostport)
		assert.Equal(t, test.port, p, test.hostport)
	}
}

// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package internal // import "go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc/internal"

import (
	"testing"

	"go.opentelemetry.io/otel/attribute"
	semconv "go.opentelemetry.io/otel/semconv/v1.17.0"

	"github.com/stretchr/testify/assert"
)

func TestParseFullMethod(t *testing.T) {
	cases := []struct {
		input         string
		expectedName  string
		expectedAttrs []attribute.KeyValue
	}{
		{
			input:        "no_slash/method",
			expectedName: "no_slash/method",
		},
		{
			input:        "/slash_but_no_second_slash",
			expectedName: "slash_but_no_second_slash",
		},
		{
			input:        "/service_only/",
			expectedName: "service_only/",
			expectedAttrs: []attribute.KeyValue{
				semconv.RPCService("service_only"),
			},
		},
		{
			input:        "//method_only",
			expectedName: "/method_only",
			expectedAttrs: []attribute.KeyValue{
				semconv.RPCMethod("method_only"),
			},
		},
		{
			input:        "/service/method",
			expectedName: "service/method",
			expectedAttrs: []attribute.KeyValue{
				semconv.RPCService("service"),
				semconv.RPCMethod("method"),
			},
		},
	}
	for _, tc := range cases {
		t.Run(tc.input, func(t *testing.T) {
			name, attrs := ParseFullMethod(tc.input)
			assert.Equal(t, tc.expectedName, name)
			assert.Equal(t, tc.expectedAttrs, attrs)
		})
	}
}

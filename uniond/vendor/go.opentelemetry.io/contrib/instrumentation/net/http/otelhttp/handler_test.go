// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package otelhttp_test

import (
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"go.opentelemetry.io/contrib/instrumentation/net/http/otelhttp"
)

func TestHandler(t *testing.T) {
	testCases := []struct {
		name               string
		handler            func(*testing.T) http.Handler
		requestBody        io.Reader
		expectedStatusCode int
	}{
		{
			name: "implements flusher",
			handler: func(t *testing.T) http.Handler {
				return otelhttp.NewHandler(
					http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
						assert.Implements(t, (*http.Flusher)(nil), w)
					}), "test_handler",
				)
			},
			expectedStatusCode: http.StatusOK,
		},
		{
			name: "succeeds",
			handler: func(t *testing.T) http.Handler {
				return otelhttp.NewHandler(
					http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
						assert.NotNil(t, r.Body)

						b, err := io.ReadAll(r.Body)
						assert.NoError(t, err)
						assert.Equal(t, "hello world", string(b))
					}), "test_handler",
				)
			},
			requestBody:        strings.NewReader("hello world"),
			expectedStatusCode: http.StatusOK,
		},
		{
			name: "succeeds with a nil body",
			handler: func(t *testing.T) http.Handler {
				return otelhttp.NewHandler(
					http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
						assert.Nil(t, r.Body)
					}), "test_handler",
				)
			},
			expectedStatusCode: http.StatusOK,
		},
		{
			name: "succeeds with an http.NoBody",
			handler: func(t *testing.T) http.Handler {
				return otelhttp.NewHandler(
					http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
						assert.Equal(t, http.NoBody, r.Body)
					}), "test_handler",
				)
			},
			requestBody:        http.NoBody,
			expectedStatusCode: http.StatusOK,
		},
	}
	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			r, err := http.NewRequest(http.MethodGet, "http://localhost/", tc.requestBody)
			require.NoError(t, err)

			rr := httptest.NewRecorder()
			tc.handler(t).ServeHTTP(rr, r)
			assert.Equal(t, tc.expectedStatusCode, rr.Result().StatusCode)
		})
	}
}

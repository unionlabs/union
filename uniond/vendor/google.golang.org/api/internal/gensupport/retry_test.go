// Copyright 2024 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package gensupport

import (
	"errors"
	"fmt"
	"io"
	"net"
	"net/url"
	"testing"
)

func TestShouldRetry(t *testing.T) {
	t.Parallel()

	for _, test := range []struct {
		desc        string
		code        int
		inputErr    error
		shouldRetry bool
	}{
		{
			desc:        "nil error",
			inputErr:    nil,
			shouldRetry: false,
		},
		{
			desc:        "429 error",
			inputErr:    fmt.Errorf("too many requests"),
			code:        429,
			shouldRetry: true,
		},
		{
			desc:        "408 error",
			inputErr:    fmt.Errorf("request timed out"),
			code:        408,
			shouldRetry: true,
		},
		{
			desc:        "unknown error",
			inputErr:    errors.New("foo"),
			shouldRetry: false,
		},
		{
			desc:        "503 error",
			inputErr:    fmt.Errorf("service unavailable"),
			code:        503,
			shouldRetry: true,
		},
		{
			desc:        "403 error",
			inputErr:    fmt.Errorf("forbidden"),
			code:        403,
			shouldRetry: false,
		},
		{
			desc:        "connection refused",
			inputErr:    &url.Error{Op: "blah", URL: "blah", Err: errors.New("connection refused")},
			shouldRetry: true,
		},
		{
			desc:        "connection reset",
			inputErr:    &net.OpError{Op: "blah", Net: "tcp", Err: errors.New("connection reset by peer")},
			shouldRetry: true,
		},
		{
			desc:        "io.ErrUnexpectedEOF",
			inputErr:    io.ErrUnexpectedEOF,
			shouldRetry: true,
		},
		{
			desc:        "wrapped retryable error",
			inputErr:    fmt.Errorf("Test unwrapping of a temporary error: %w", io.ErrUnexpectedEOF),
			shouldRetry: true,
		},
		{
			desc:        "wrapped non-retryable error",
			inputErr:    fmt.Errorf("Test unwrapping of a non-retriable error: %w", io.EOF),
			shouldRetry: false,
		},
		{
			desc:        "wrapped net.ErrClosed",
			inputErr:    &net.OpError{Err: net.ErrClosed},
			shouldRetry: true,
		},
	} {
		t.Run(test.desc, func(s *testing.T) {
			got := shouldRetry(test.code, test.inputErr)
			if got != test.shouldRetry {
				s.Errorf("got %v, want %v", got, test.shouldRetry)
			}
		})
	}
}

// Copyright 2020 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package storage

import (
	"context"
	"errors"
	"fmt"
	"io"
	"net"
	"net/http"
	"net/url"
	"regexp"
	"strings"
	"testing"
	"time"

	"github.com/googleapis/gax-go/v2"
	"github.com/googleapis/gax-go/v2/callctx"
	"golang.org/x/xerrors"
	"google.golang.org/api/googleapi"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

func TestInvoke(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	// Time-based tests are flaky. We just make sure that invoke eventually
	// returns with the right error.

	for _, test := range []struct {
		desc              string
		count             int   // Number of times to return retryable error.
		initialErr        error // Error to return initially.
		finalErr          error // Error to return after count returns of retryCode.
		retry             *retryConfig
		isIdempotentValue bool
		expectFinalErr    bool
	}{
		{
			desc:              "test fn never returns initial error with count=0",
			count:             0,
			initialErr:        &googleapi.Error{Code: 0}, //non-retryable
			finalErr:          nil,
			isIdempotentValue: true,
			expectFinalErr:    true,
		},
		{
			desc:              "non-retryable error is returned without retrying",
			count:             1,
			initialErr:        &googleapi.Error{Code: 0},
			finalErr:          nil,
			isIdempotentValue: true,
			expectFinalErr:    false,
		},
		{
			desc:              "retryable error is retried",
			count:             1,
			initialErr:        &googleapi.Error{Code: 429},
			finalErr:          nil,
			isIdempotentValue: true,
			expectFinalErr:    true,
		},
		{
			desc:              "retryable gRPC error is retried",
			count:             1,
			initialErr:        status.Error(codes.ResourceExhausted, "rate limit"),
			finalErr:          nil,
			isIdempotentValue: true,
			expectFinalErr:    true,
		},
		{
			desc:              "returns non-retryable error after retryable error",
			count:             1,
			initialErr:        &googleapi.Error{Code: 429},
			finalErr:          errors.New("bar"),
			isIdempotentValue: true,
			expectFinalErr:    true,
		},
		{
			desc:              "retryable 5xx error is retried",
			count:             2,
			initialErr:        &googleapi.Error{Code: 518},
			finalErr:          nil,
			isIdempotentValue: true,
			expectFinalErr:    true,
		},
		{
			desc:              "retriable error not retried when non-idempotent",
			count:             2,
			initialErr:        &googleapi.Error{Code: 599},
			finalErr:          nil,
			isIdempotentValue: false,
			expectFinalErr:    false,
		},
		{
			desc:              "non-idempotent retriable error retried when policy is RetryAlways",
			count:             2,
			initialErr:        &googleapi.Error{Code: 500},
			finalErr:          nil,
			isIdempotentValue: false,
			retry:             &retryConfig{policy: RetryAlways},
			expectFinalErr:    true,
		},
		{
			desc:              "retriable error not retried when policy is RetryNever",
			count:             2,
			initialErr:        &url.Error{Op: "blah", URL: "blah", Err: errors.New("connection refused")},
			finalErr:          nil,
			isIdempotentValue: true,
			retry:             &retryConfig{policy: RetryNever},
			expectFinalErr:    false,
		},
		{
			desc:              "non-retriable error not retried when policy is RetryAlways",
			count:             2,
			initialErr:        xerrors.Errorf("non-retriable error: %w", &googleapi.Error{Code: 400}),
			finalErr:          nil,
			isIdempotentValue: true,
			retry:             &retryConfig{policy: RetryAlways},
			expectFinalErr:    false,
		},
		{
			desc:              "non-retriable error retried with custom fn",
			count:             2,
			initialErr:        io.ErrNoProgress,
			finalErr:          nil,
			isIdempotentValue: true,
			retry: &retryConfig{
				shouldRetry: func(err error) bool {
					return err == io.ErrNoProgress
				},
			},
			expectFinalErr: true,
		},
		{
			desc:              "retriable error not retried with custom fn",
			count:             2,
			initialErr:        io.ErrUnexpectedEOF,
			finalErr:          nil,
			isIdempotentValue: true,
			retry: &retryConfig{
				shouldRetry: func(err error) bool {
					return err == io.ErrNoProgress
				},
			},
			expectFinalErr: false,
		},
		{
			desc:              "error not retried when policy is RetryNever despite custom fn",
			count:             2,
			initialErr:        io.ErrUnexpectedEOF,
			finalErr:          nil,
			isIdempotentValue: true,
			retry: &retryConfig{
				shouldRetry: func(err error) bool {
					return err == io.ErrUnexpectedEOF
				},
				policy: RetryNever,
			},
			expectFinalErr: false,
		},
		{
			desc:              "non-idempotent retriable error retried when policy is RetryAlways till maxAttempts",
			count:             4,
			initialErr:        &googleapi.Error{Code: 500},
			finalErr:          nil,
			isIdempotentValue: false,
			retry:             &retryConfig{policy: RetryAlways, maxAttempts: expectedAttempts(2)},
			expectFinalErr:    false,
		},
		{
			desc:              "non-idempotent retriable error not retried when policy is RetryNever with maxAttempts set",
			count:             4,
			initialErr:        &googleapi.Error{Code: 500},
			finalErr:          nil,
			isIdempotentValue: false,
			retry:             &retryConfig{policy: RetryNever, maxAttempts: expectedAttempts(2)},
			expectFinalErr:    false,
		},
		{
			desc:              "non-retriable error retried with custom fn till maxAttempts",
			count:             4,
			initialErr:        io.ErrNoProgress,
			finalErr:          nil,
			isIdempotentValue: true,
			retry: &retryConfig{
				shouldRetry: func(err error) bool {
					return err == io.ErrNoProgress
				},
				maxAttempts: expectedAttempts(2),
			},
			expectFinalErr: false,
		},
		{
			desc:              "non-idempotent retriable error retried when policy is RetryAlways till maxAttempts where count equals to maxAttempts-1",
			count:             3,
			initialErr:        &googleapi.Error{Code: 500},
			finalErr:          nil,
			isIdempotentValue: false,
			retry:             &retryConfig{policy: RetryAlways, maxAttempts: expectedAttempts(4)},
			expectFinalErr:    true,
		},
		{
			desc:              "non-idempotent retriable error retried when policy is RetryAlways till maxAttempts where count equals to maxAttempts",
			count:             4,
			initialErr:        &googleapi.Error{Code: 500},
			finalErr:          nil,
			isIdempotentValue: true,
			retry:             &retryConfig{policy: RetryAlways, maxAttempts: expectedAttempts(4)},
			expectFinalErr:    false,
		},
		{
			desc:              "non-idempotent retriable error not retried when policy is RetryAlways with maxAttempts equals to zero",
			count:             4,
			initialErr:        &googleapi.Error{Code: 500},
			finalErr:          nil,
			isIdempotentValue: true,
			retry:             &retryConfig{maxAttempts: expectedAttempts(0), policy: RetryAlways},
			expectFinalErr:    false,
		},
	} {
		t.Run(test.desc, func(s *testing.T) {
			counter := 0
			var initialClientHeader, initialIdempotencyHeader string
			var gotClientHeader, gotIdempotencyHeader string
			call := func(ctx context.Context) error {
				if counter == 0 {
					headers := callctx.HeadersFromContext(ctx)
					initialClientHeader = headers["x-goog-api-client"][0]
					initialIdempotencyHeader = headers["x-goog-gcs-idempotency-token"][0]
				}
				counter++
				headers := callctx.HeadersFromContext(ctx)
				gotClientHeader = headers["x-goog-api-client"][0]
				gotIdempotencyHeader = headers["x-goog-gcs-idempotency-token"][0]
				if counter <= test.count {
					return test.initialErr
				}
				return test.finalErr
			}
			// Use a short backoff to speed up the test.
			if test.retry == nil {
				test.retry = defaultRetry.clone()
			}
			test.retry.backoff = &gax.Backoff{Initial: time.Millisecond}
			got := run(ctx, call, test.retry, test.isIdempotentValue)
			if test.expectFinalErr && !errors.Is(got, test.finalErr) {
				s.Errorf("got %v, want %v", got, test.finalErr)
			} else if !test.expectFinalErr && !errors.Is(got, test.initialErr) {
				s.Errorf("got %v, want %v", got, test.initialErr)
			}
			wantAttempts := 1 + test.count
			if !test.expectFinalErr {
				wantAttempts = 1
			}
			if test.retry != nil && test.retry.maxAttempts != nil && *test.retry.maxAttempts != 0 && test.retry.policy != RetryNever {
				wantAttempts = *test.retry.maxAttempts
			}

			wantClientHeader := strings.ReplaceAll(initialClientHeader, "gccl-attempt-count/1", fmt.Sprintf("gccl-attempt-count/%v", wantAttempts))
			if gotClientHeader != wantClientHeader {
				t.Errorf("case %q, retry header:\ngot %v\nwant %v", test.desc, gotClientHeader, wantClientHeader)
			}
			wantClientHeaderFormat := "gccl-invocation-id/.{36} gccl-attempt-count/[0-9]+ gl-go/.* gccl/"
			match, err := regexp.MatchString(wantClientHeaderFormat, gotClientHeader)
			if err != nil {
				s.Fatalf("compiling regexp: %v", err)
			}
			if !match {
				s.Errorf("X-Goog-Api-Client header has wrong format\ngot %v\nwant regex matching %v", gotClientHeader, wantClientHeaderFormat)
			}
			if gotIdempotencyHeader != initialIdempotencyHeader {
				t.Errorf("case %q, idempotency header:\ngot %v\nwant %v", test.desc, gotIdempotencyHeader, initialIdempotencyHeader)
			}
		})
	}
}

type fakeApiaryRequest struct {
	header http.Header
}

func (f *fakeApiaryRequest) Header() http.Header {
	return f.header
}

// TestInvokeHeaderMerge tests that values for x-goog-api-client are merged into
// a single space-separated value. This test should be removed with the code once
// both transport package dependencies do the merge.
func TestInvokeHeaderMerge(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	xGoogKey := "x-goog-api-client"

	for _, test := range []struct {
		desc             string
		headerValueOnCtx string
		count            int
	}{
		{
			desc:             "non-retried run",
			headerValueOnCtx: "somekey/value_1",
			count:            0,
		},
		{
			desc:             "retried run",
			headerValueOnCtx: "somekey/value_1 another/value_11",
			count:            2,
		},
	} {
		t.Run(test.desc, func(s *testing.T) {
			counter := 0
			var gotClientHeaders []string

			ctx := callctx.SetHeaders(ctx, xGoogKey, test.headerValueOnCtx)

			call := func(ctx context.Context) error {
				headers := callctx.HeadersFromContext(ctx)
				gotClientHeaders = headers["x-goog-api-client"]
				counter++

				if counter <= test.count {
					// return a retriable error so test will retry if count > 0
					return &googleapi.Error{Code: 500}
				}
				return nil
			}
			// Use a short backoff to speed up the test.
			retry := defaultRetry.clone()
			retry.backoff = &gax.Backoff{Initial: time.Millisecond}

			run(ctx, call, retry, true)

			if len(gotClientHeaders) != 1 {
				s.Errorf("x-goog-api-client header should be merged into a single value, got: %+v", gotClientHeaders)
			}

			gotClientHeader := gotClientHeaders[0]

			wantClientHeaderFormat := fmt.Sprintf("^gccl-invocation-id/.{36} gccl-attempt-count/[0-9]+ gl-go/.* gccl/[0-9]+.[0-9]+.[0-9]+ %s$", test.headerValueOnCtx)
			match, err := regexp.MatchString(wantClientHeaderFormat, gotClientHeader)
			if err != nil {
				s.Fatalf("compiling regexp: %v", err)
			}
			if !match {
				s.Errorf("X-Goog-Api-Client header has wrong format\ngot %v\nwant regex matching %v", gotClientHeader, wantClientHeaderFormat)
			}
		})
	}
}

func TestShouldRetry(t *testing.T) {
	t.Parallel()

	for _, test := range []struct {
		desc        string
		inputErr    error
		shouldRetry bool
	}{
		{
			desc:        "googleapi.Error{Code: 0}",
			inputErr:    &googleapi.Error{Code: 0},
			shouldRetry: false,
		},
		{
			desc:        "googleapi.Error{Code: 429}",
			inputErr:    &googleapi.Error{Code: 429},
			shouldRetry: true,
		},
		{
			desc:        "errors.New(foo)",
			inputErr:    errors.New("foo"),
			shouldRetry: false,
		},
		{
			desc:        "googleapi.Error{Code: 518}",
			inputErr:    &googleapi.Error{Code: 518},
			shouldRetry: true,
		},
		{
			desc:        "googleapi.Error{Code: 599}",
			inputErr:    &googleapi.Error{Code: 599},
			shouldRetry: true,
		},
		{
			desc:        "googleapi.Error{Code: 428}",
			inputErr:    &googleapi.Error{Code: 428},
			shouldRetry: false,
		},
		{
			desc:        "googleapi.Error{Code: 518}",
			inputErr:    &googleapi.Error{Code: 518},
			shouldRetry: true,
		},
		{
			desc:        "url.Error{Err: errors.New(\"connection refused\")}",
			inputErr:    &url.Error{Op: "blah", URL: "blah", Err: errors.New("connection refused")},
			shouldRetry: true,
		},
		{
			desc:        "net.OpError{Err: errors.New(\"connection reset by peer\")}",
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
			inputErr:    xerrors.Errorf("Test unwrapping of a temporary error: %w", &googleapi.Error{Code: 500}),
			shouldRetry: true,
		},
		{
			desc:        "wrapped non-retryable error",
			inputErr:    xerrors.Errorf("Test unwrapping of a non-retriable error: %w", &googleapi.Error{Code: 400}),
			shouldRetry: false,
		},
		{
			desc:        "googleapi.Error{Code: 400}",
			inputErr:    &googleapi.Error{Code: 400},
			shouldRetry: false,
		},
		{
			desc:        "googleapi.Error{Code: 408}",
			inputErr:    &googleapi.Error{Code: 408},
			shouldRetry: true,
		},
		{
			desc:        "retryable gRPC error",
			inputErr:    status.Error(codes.Unavailable, "retryable gRPC error"),
			shouldRetry: true,
		},
		{
			desc:        "non-retryable gRPC error",
			inputErr:    status.Error(codes.PermissionDenied, "non-retryable gRPC error"),
			shouldRetry: false,
		},
		{
			desc:        "wrapped net.ErrClosed",
			inputErr:    &net.OpError{Err: net.ErrClosed},
			shouldRetry: true,
		},
	} {
		t.Run(test.desc, func(s *testing.T) {
			got := ShouldRetry(test.inputErr)

			if got != test.shouldRetry {
				s.Errorf("got %v, want %v", got, test.shouldRetry)
			}
		})
	}
}

// Copyright 2014 Google LLC
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
	"strings"

	"cloud.google.com/go/internal"
	"cloud.google.com/go/internal/version"
	sinternal "cloud.google.com/go/storage/internal"
	"github.com/google/uuid"
	gax "github.com/googleapis/gax-go/v2"
	"google.golang.org/api/googleapi"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

var defaultRetry *retryConfig = &retryConfig{}
var xGoogDefaultHeader = fmt.Sprintf("gl-go/%s gccl/%s", version.Go(), sinternal.Version)

// run determines whether a retry is necessary based on the config and
// idempotency information. It then calls the function with or without retries
// as appropriate, using the configured settings.
func run(ctx context.Context, call func() error, retry *retryConfig, isIdempotent bool, setHeader func(string, int)) error {
	attempts := 1
	invocationID := uuid.New().String()

	if retry == nil {
		retry = defaultRetry
	}
	if (retry.policy == RetryIdempotent && !isIdempotent) || retry.policy == RetryNever {
		setHeader(invocationID, attempts)
		return call()
	}
	bo := gax.Backoff{}
	if retry.backoff != nil {
		bo.Multiplier = retry.backoff.Multiplier
		bo.Initial = retry.backoff.Initial
		bo.Max = retry.backoff.Max
	}
	var errorFunc func(err error) bool = ShouldRetry
	if retry.shouldRetry != nil {
		errorFunc = retry.shouldRetry
	}

	return internal.Retry(ctx, bo, func() (stop bool, err error) {
		setHeader(invocationID, attempts)
		err = call()
		attempts++
		return !errorFunc(err), err
	})
}

func setRetryHeaderHTTP(req interface{ Header() http.Header }) func(string, int) {
	return func(invocationID string, attempts int) {
		if req == nil {
			return
		}
		header := req.Header()
		invocationHeader := fmt.Sprintf("gccl-invocation-id/%v gccl-attempt-count/%v", invocationID, attempts)
		xGoogHeader := strings.Join([]string{invocationHeader, xGoogDefaultHeader}, " ")
		header.Set("x-goog-api-client", xGoogHeader)
	}
}

// TODO: Implement method setting header via context for gRPC
func setRetryHeaderGRPC(_ context.Context) func(string, int) {
	return func(_ string, _ int) {
		return
	}
}

// ShouldRetry returns true if an error is retryable, based on best practice
// guidance from GCS. See
// https://cloud.google.com/storage/docs/retry-strategy#go for more information
// on what errors are considered retryable.
//
// If you would like to customize retryable errors, use the WithErrorFunc to
// supply a RetryOption to your library calls. For example, to retry additional
// errors, you can write a custom func that wraps ShouldRetry and also specifies
// additional errors that should return true.
func ShouldRetry(err error) bool {
	if err == nil {
		return false
	}
	if errors.Is(err, io.ErrUnexpectedEOF) {
		return true
	}

	switch e := err.(type) {
	case *net.OpError:
		if strings.Contains(e.Error(), "use of closed network connection") {
			// TODO: check against net.ErrClosed (go 1.16+) instead of string
			return true
		}
	case *googleapi.Error:
		// Retry on 408, 429, and 5xx, according to
		// https://cloud.google.com/storage/docs/exponential-backoff.
		return e.Code == 408 || e.Code == 429 || (e.Code >= 500 && e.Code < 600)
	case *url.Error:
		// Retry socket-level errors ECONNREFUSED and ECONNRESET (from syscall).
		// Unfortunately the error type is unexported, so we resort to string
		// matching.
		retriable := []string{"connection refused", "connection reset"}
		for _, s := range retriable {
			if strings.Contains(e.Error(), s) {
				return true
			}
		}
	case interface{ Temporary() bool }:
		if e.Temporary() {
			return true
		}
	}
	// HTTP 429, 502, 503, and 504 all map to gRPC UNAVAILABLE per
	// https://grpc.github.io/grpc/core/md_doc_http-grpc-status-mapping.html.
	//
	// This is only necessary for the experimental gRPC-based media operations.
	if st, ok := status.FromError(err); ok && st.Code() == codes.Unavailable {
		return true
	}
	// Unwrap is only supported in go1.13.x+
	if e, ok := err.(interface{ Unwrap() error }); ok {
		return ShouldRetry(e.Unwrap())
	}
	return false
}

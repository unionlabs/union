// Copyright 2016, Google Inc.
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//     * Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above
// copyright notice, this list of conditions and the following disclaimer
// in the documentation and/or other materials provided with the
// distribution.
//     * Neither the name of Google Inc. nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

package gax

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"github.com/googleapis/gax-go/v2/apierror"
	"google.golang.org/genproto/googleapis/rpc/errdetails"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

var canceledContext context.Context

func init() {
	ctx, cancel := context.WithCancel(context.Background())
	cancel()
	canceledContext = ctx
}

// recordSleeper is a test implementation of sleeper.
type recordSleeper int

func (s *recordSleeper) sleep(ctx context.Context, _ time.Duration) error {
	*s++
	return ctx.Err()
}

type boolRetryer bool

func (r boolRetryer) Retry(err error) (time.Duration, bool) { return 0, bool(r) }

func TestInvokeSuccess(t *testing.T) {
	apiCall := func(context.Context, CallSettings) error { return nil }
	var sp recordSleeper
	err := invoke(context.Background(), apiCall, CallSettings{}, sp.sleep)

	if err != nil {
		t.Errorf("found error %s, want nil", err)
	}
	if sp != 0 {
		t.Errorf("slept %d times, should not have slept since the call succeeded", int(sp))
	}
}

func TestInvokeCertificateError(t *testing.T) {
	stat := status.New(codes.Unavailable, "x509: certificate signed by unknown authority")
	apiErr := stat.Err()
	apiCall := func(context.Context, CallSettings) error { return apiErr }
	var sp recordSleeper
	err := invoke(context.Background(), apiCall, CallSettings{}, sp.sleep)
	if diff := cmp.Diff(err, apiErr, cmpopts.EquateErrors()); diff != "" {
		t.Errorf("got(-), want(+): \n%s", diff)
	}
}

func TestInvokeAPIError(t *testing.T) {
	qf := &errdetails.QuotaFailure{
		Violations: []*errdetails.QuotaFailure_Violation{{Subject: "Foo", Description: "Bar"}},
	}
	stat, _ := status.New(codes.ResourceExhausted, "Per user quota has been exhausted").WithDetails(qf)
	apiErr, _ := apierror.FromError(stat.Err())
	apiCall := func(context.Context, CallSettings) error { return stat.Err() }
	var sp recordSleeper
	err := invoke(context.Background(), apiCall, CallSettings{}, sp.sleep)
	if diff := cmp.Diff(err.Error(), apiErr.Error()); diff != "" {
		t.Errorf("got(-), want(+): \n%s", diff)
	}
	if sp != 0 {
		t.Errorf("slept %d times, should not have slept since the call succeeded", int(sp))
	}
}

func TestInvokeCtxError(t *testing.T) {
	ctxErr := context.DeadlineExceeded
	apiCall := func(context.Context, CallSettings) error { return ctxErr }
	var sp recordSleeper
	err := invoke(context.Background(), apiCall, CallSettings{}, sp.sleep)
	if err != ctxErr {
		t.Errorf("found error %s, want %s", err, ctxErr)
	}
	if sp != 0 {
		t.Errorf("slept %d times, should not have slept since the call succeeded", int(sp))
	}

}

func TestInvokeNoRetry(t *testing.T) {
	apiErr := errors.New("foo error")
	apiCall := func(context.Context, CallSettings) error { return apiErr }
	var sp recordSleeper
	err := invoke(context.Background(), apiCall, CallSettings{}, sp.sleep)

	if err != apiErr {
		t.Errorf("found error %s, want %s", err, apiErr)
	}
	if sp != 0 {
		t.Errorf("slept %d times, should not have slept since retry is not specified", int(sp))
	}
}

func TestInvokeNilRetry(t *testing.T) {
	apiErr := errors.New("foo error")
	apiCall := func(context.Context, CallSettings) error { return apiErr }
	var settings CallSettings
	WithRetry(func() Retryer { return nil }).Resolve(&settings)
	var sp recordSleeper
	err := invoke(context.Background(), apiCall, settings, sp.sleep)

	if err != apiErr {
		t.Errorf("found error %s, want %s", err, apiErr)
	}
	if sp != 0 {
		t.Errorf("slept %d times, should not have slept since retry is not specified", int(sp))
	}
}

func TestInvokeNeverRetry(t *testing.T) {
	apiErr := errors.New("foo error")
	apiCall := func(context.Context, CallSettings) error { return apiErr }
	var settings CallSettings
	WithRetry(func() Retryer { return boolRetryer(false) }).Resolve(&settings)
	var sp recordSleeper
	err := invoke(context.Background(), apiCall, settings, sp.sleep)

	if err != apiErr {
		t.Errorf("found error %s, want %s", err, apiErr)
	}
	if sp != 0 {
		t.Errorf("slept %d times, should not have slept since retry is not specified", int(sp))
	}
}

func TestInvokeRetry(t *testing.T) {
	const target = 3

	retryNum := 0
	apiErr := errors.New("foo error")
	apiCall := func(context.Context, CallSettings) error {
		retryNum++
		if retryNum < target {
			return apiErr
		}
		return nil
	}
	var settings CallSettings
	WithRetry(func() Retryer { return boolRetryer(true) }).Resolve(&settings)
	var sp recordSleeper
	err := invoke(context.Background(), apiCall, settings, sp.sleep)

	if err != nil {
		t.Errorf("found error %s, want nil, call should have succeeded after %d tries", err, target)
	}
	if sp != target-1 {
		t.Errorf("retried %d times, want %d", int(sp), int(target-1))
	}
}

func TestInvokeRetryTimeout(t *testing.T) {
	apiErr := errors.New("foo error")
	apiCall := func(context.Context, CallSettings) error { return apiErr }
	var settings CallSettings
	WithRetry(func() Retryer { return boolRetryer(true) }).Resolve(&settings)
	var sp recordSleeper

	err := invoke(canceledContext, apiCall, settings, sp.sleep)

	if err != context.Canceled {
		t.Errorf("found error %s, want %s", err, context.Canceled)
	}
}

func TestInvokeWithTimeout(t *testing.T) {
	// Dummy APICall that sleeps for the given amount of time. This simulates an
	// APICall executing, allowing us to verify which deadline was respected,
	// that which is already set on the Context, or the one calculated using the
	// WithTimeout option's value.
	sleepingCall := func(sleep time.Duration) APICall {
		return func(ctx context.Context, _ CallSettings) error {
			time.Sleep(sleep)
			return ctx.Err()
		}
	}

	bg := context.Background()
	preset, pcc := context.WithTimeout(bg, 10*time.Millisecond)
	defer pcc()

	for _, tst := range []struct {
		name    string
		timeout time.Duration
		sleep   time.Duration
		ctx     context.Context
		want    error
	}{
		{
			name:    "success",
			timeout: 10 * time.Millisecond,
			sleep:   1 * time.Millisecond,
			ctx:     bg,
			want:    nil,
		},
		{
			name:    "respect_context_deadline",
			timeout: 1 * time.Millisecond,
			sleep:   3 * time.Millisecond,
			ctx:     preset,
			want:    nil,
		},
		{
			name:    "with_timeout_deadline_exceeded",
			timeout: 1 * time.Millisecond,
			sleep:   3 * time.Millisecond,
			ctx:     bg,
			want:    context.DeadlineExceeded,
		},
	} {
		t.Run(tst.name, func(t *testing.T) {
			// Recording sleep isn't really necessary since there is
			// no retry here, but we need a sleeper so might as well.
			var sp recordSleeper
			var settings CallSettings

			WithTimeout(tst.timeout).Resolve(&settings)

			err := invoke(tst.ctx, sleepingCall(tst.sleep), settings, sp.sleep)

			if err != tst.want {
				t.Errorf("found error %v, want %v", err, tst.want)
			}
		})
	}
}

// Copyright 2019, Google Inc.
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

package gax_test

import (
	"context"
	"io"
	"net/http"
	"time"

	gax "github.com/googleapis/gax-go/v2"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/types/known/structpb"
)

// Some result that the client might return.
type fakeResponse struct{}

// Some client that can perform RPCs.
type fakeClient struct{}

// PerformSomeRPC is a fake RPC that a client might perform.
func (c *fakeClient) PerformSomeRPC(ctx context.Context) (*fakeResponse, error) {
	// An actual client would return something meaningful here.
	return nil, nil
}

func ExampleOnErrorFunc() {
	ctx := context.Background()
	c := &fakeClient{}

	shouldRetryUnavailableUnKnown := func(err error) bool {
		st, ok := status.FromError(err)
		if !ok {
			return false
		}

		return st.Code() == codes.Unavailable || st.Code() == codes.Unknown
	}
	retryer := gax.OnErrorFunc(gax.Backoff{
		Initial:    time.Second,
		Max:        32 * time.Second,
		Multiplier: 2,
	}, shouldRetryUnavailableUnKnown)

	performSomeRPCWithRetry := func(ctx context.Context) (*fakeResponse, error) {
		for {
			resp, err := c.PerformSomeRPC(ctx)
			if err != nil {
				if delay, shouldRetry := retryer.Retry(err); shouldRetry {
					if err := gax.Sleep(ctx, delay); err != nil {
						return nil, err
					}
					continue
				}
				return nil, err
			}
			return resp, err
		}
	}

	// It's recommended to set deadlines on RPCs and around retrying. This is
	// also usually preferred over setting some fixed number of retries: one
	// advantage this has is that backoff settings can be changed independently
	// of the deadline, whereas with a fixed number of retries the deadline
	// would be a constantly-shifting goalpost.
	ctxWithTimeout, cancel := context.WithDeadline(ctx, time.Now().Add(5*time.Minute))
	defer cancel()

	resp, err := performSomeRPCWithRetry(ctxWithTimeout)
	if err != nil {
		// TODO: handle err
	}
	_ = resp // TODO: use resp if err is nil
}

func ExampleOnCodes() {
	ctx := context.Background()
	c := &fakeClient{}

	// UNKNOWN and UNAVAILABLE are typically safe to retry for idempotent RPCs.
	retryer := gax.OnCodes([]codes.Code{codes.Unknown, codes.Unavailable}, gax.Backoff{
		Initial:    time.Second,
		Max:        32 * time.Second,
		Multiplier: 2,
	})

	performSomeRPCWithRetry := func(ctx context.Context) (*fakeResponse, error) {
		for {
			resp, err := c.PerformSomeRPC(ctx)
			if err != nil {
				if delay, shouldRetry := retryer.Retry(err); shouldRetry {
					if err := gax.Sleep(ctx, delay); err != nil {
						return nil, err
					}
					continue
				}
				return nil, err
			}
			return resp, err
		}
	}

	// It's recommended to set deadlines on RPCs and around retrying. This is
	// also usually preferred over setting some fixed number of retries: one
	// advantage this has is that backoff settings can be changed independently
	// of the deadline, whereas with a fixed number of retries the deadline
	// would be a constantly-shifting goalpost.
	ctxWithTimeout, cancel := context.WithDeadline(ctx, time.Now().Add(5*time.Minute))
	defer cancel()

	resp, err := performSomeRPCWithRetry(ctxWithTimeout)
	if err != nil {
		// TODO: handle err
	}
	_ = resp // TODO: use resp if err is nil
}

func ExampleOnHTTPCodes() {
	ctx := context.Background()
	c := &fakeClient{}

	retryer := gax.OnHTTPCodes(gax.Backoff{
		Initial:    time.Second,
		Max:        32 * time.Second,
		Multiplier: 2,
	}, http.StatusBadGateway, http.StatusServiceUnavailable)

	performSomeRPCWithRetry := func(ctx context.Context) (*fakeResponse, error) {
		for {
			resp, err := c.PerformSomeRPC(ctx)
			if err != nil {
				if delay, shouldRetry := retryer.Retry(err); shouldRetry {
					if err := gax.Sleep(ctx, delay); err != nil {
						return nil, err
					}
					continue
				}
				return nil, err
			}
			return resp, err
		}
	}

	// It's recommended to set deadlines on RPCs and around retrying. This is
	// also usually preferred over setting some fixed number of retries: one
	// advantage this has is that backoff settings can be changed independently
	// of the deadline, whereas with a fixed number of retries the deadline
	// would be a constantly-shifting goalpost.
	ctxWithTimeout, cancel := context.WithDeadline(ctx, time.Now().Add(5*time.Minute))
	defer cancel()

	resp, err := performSomeRPCWithRetry(ctxWithTimeout)
	if err != nil {
		// TODO: handle err
	}
	_ = resp // TODO: use resp if err is nil
}

func ExampleBackoff() {
	ctx := context.Background()

	bo := gax.Backoff{
		Initial:    time.Second,
		Max:        time.Minute, // Maximum amount of time between retries.
		Multiplier: 2,
	}

	performHTTPCallWithRetry := func(ctx context.Context, doHTTPCall func(ctx context.Context) (*http.Response, error)) (*http.Response, error) {
		for {
			resp, err := doHTTPCall(ctx)
			if err != nil {
				// Retry 503 UNAVAILABLE.
				if resp.StatusCode == http.StatusServiceUnavailable {
					if err := gax.Sleep(ctx, bo.Pause()); err != nil {
						return nil, err
					}
					continue
				}
				return nil, err
			}
			return resp, err
		}
	}

	// It's recommended to set deadlines on HTTP calls and around retrying. This
	// is also usually preferred over setting some fixed number of retries: one
	// advantage this has is that backoff settings can be changed independently
	// of the deadline, whereas with a fixed number of retries the deadline
	// would be a constantly-shifting goalpost.
	ctxWithTimeout, cancel := context.WithDeadline(ctx, time.Now().Add(5*time.Minute))
	defer cancel()

	resp, err := performHTTPCallWithRetry(ctxWithTimeout, func(ctx context.Context) (*http.Response, error) {
		req, err := http.NewRequest("some-method", "example.com", nil)
		if err != nil {
			return nil, err
		}
		req = req.WithContext(ctx)
		return http.DefaultClient.Do(req)
	})
	if err != nil {
		// TODO: handle err
	}
	_ = resp // TODO: use resp if err is nil
}

func ExampleProtoJSONStream() {
	var someHTTPCall func() (http.Response, error)

	res, err := someHTTPCall()
	if err != nil {
		// TODO: handle err
	}

	// The type of message expected in the stream.
	var typ protoreflect.MessageType = (&structpb.Struct{}).ProtoReflect().Type()

	stream := gax.NewProtoJSONStreamReader(res.Body, typ)
	defer stream.Close()

	for {
		m, err := stream.Recv()
		if err != nil {
			break
		}
		// TODO: use resp
		_ = m.(*structpb.Struct)
	}
	if err != io.EOF {
		// TODO: handle err
	}
}

func ExampleInvoke_grpc() {
	ctx := context.Background()
	c := &fakeClient{}
	opt := gax.WithRetry(func() gax.Retryer {
		return gax.OnCodes([]codes.Code{codes.Unknown, codes.Unavailable}, gax.Backoff{
			Initial:    time.Second,
			Max:        32 * time.Second,
			Multiplier: 2,
		})
	})

	var resp *fakeResponse
	err := gax.Invoke(ctx, func(ctx context.Context, settings gax.CallSettings) error {
		var err error
		resp, err = c.PerformSomeRPC(ctx)
		return err
	}, opt)
	if err != nil {
		// TODO: handle err
	}
	_ = resp // TODO: use resp if err is nil
}

func ExampleInvoke_http() {
	ctx := context.Background()
	c := &fakeClient{}
	opt := gax.WithRetry(func() gax.Retryer {
		return gax.OnHTTPCodes(gax.Backoff{
			Initial:    time.Second,
			Max:        32 * time.Second,
			Multiplier: 2,
		}, http.StatusBadGateway, http.StatusServiceUnavailable)
	})

	var resp *fakeResponse
	err := gax.Invoke(ctx, func(ctx context.Context, settings gax.CallSettings) error {
		var err error
		resp, err = c.PerformSomeRPC(ctx)
		return err
	}, opt)
	if err != nil {
		// TODO: handle err
	}
	_ = resp // TODO: use resp if err is nil
}

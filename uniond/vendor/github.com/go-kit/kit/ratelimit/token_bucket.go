package ratelimit

import (
	"context"
	"errors"

	"github.com/go-kit/kit/endpoint"
)

// ErrLimited is returned in the request path when the rate limiter is
// triggered and the request is rejected.
var ErrLimited = errors.New("rate limit exceeded")

// Allower dictates whether or not a request is acceptable to run.
// The Limiter from "golang.org/x/time/rate" already implements this interface,
// one is able to use that in NewErroringLimiter without any modifications.
type Allower interface {
	Allow() bool
}

// NewErroringLimiter returns an endpoint.Middleware that acts as a rate
// limiter. Requests that would exceed the
// maximum request rate are simply rejected with an error.
func NewErroringLimiter(limit Allower) endpoint.Middleware {
	return func(next endpoint.Endpoint) endpoint.Endpoint {
		return func(ctx context.Context, request interface{}) (interface{}, error) {
			if !limit.Allow() {
				return nil, ErrLimited
			}
			return next(ctx, request)
		}
	}
}

// Waiter dictates how long a request must be delayed.
// The Limiter from "golang.org/x/time/rate" already implements this interface,
// one is able to use that in NewDelayingLimiter without any modifications.
type Waiter interface {
	Wait(ctx context.Context) error
}

// NewDelayingLimiter returns an endpoint.Middleware that acts as a
// request throttler. Requests that would
// exceed the maximum request rate are delayed via the Waiter function
func NewDelayingLimiter(limit Waiter) endpoint.Middleware {
	return func(next endpoint.Endpoint) endpoint.Endpoint {
		return func(ctx context.Context, request interface{}) (interface{}, error) {
			if err := limit.Wait(ctx); err != nil {
				return nil, err
			}
			return next(ctx, request)
		}
	}
}

// AllowerFunc is an adapter that lets a function operate as if
// it implements Allower
type AllowerFunc func() bool

// Allow makes the adapter implement Allower
func (f AllowerFunc) Allow() bool {
	return f()
}

// WaiterFunc is an adapter that lets a function operate as if
// it implements Waiter
type WaiterFunc func(ctx context.Context) error

// Wait makes the adapter implement Waiter
func (f WaiterFunc) Wait(ctx context.Context) error {
	return f(ctx)
}

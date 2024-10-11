package lb_test

import (
	"context"
	"errors"
	"testing"
	"time"

	"github.com/go-kit/kit/endpoint"
	"github.com/go-kit/kit/sd"
	"github.com/go-kit/kit/sd/lb"
)

func TestRetryMaxTotalFail(t *testing.T) {
	var (
		endpoints = sd.FixedEndpointer{} // no endpoints
		rr        = lb.NewRoundRobin(endpoints)
		retry     = lb.Retry(999, time.Second, rr) // lots of retries
		ctx       = context.Background()
	)
	if _, err := retry(ctx, struct{}{}); err == nil {
		t.Errorf("expected error, got none") // should fail
	}
}

func TestRetryMaxPartialFail(t *testing.T) {
	var (
		endpoints = []endpoint.Endpoint{
			func(context.Context, interface{}) (interface{}, error) { return nil, errors.New("error one") },
			func(context.Context, interface{}) (interface{}, error) { return nil, errors.New("error two") },
			func(context.Context, interface{}) (interface{}, error) { return struct{}{}, nil /* OK */ },
		}
		endpointer = sd.FixedEndpointer{
			0: endpoints[0],
			1: endpoints[1],
			2: endpoints[2],
		}
		retries = len(endpoints) - 1 // not quite enough retries
		rr      = lb.NewRoundRobin(endpointer)
		ctx     = context.Background()
	)
	if _, err := lb.Retry(retries, time.Second, rr)(ctx, struct{}{}); err == nil {
		t.Errorf("expected error two, got none")
	}
}

func TestRetryMaxSuccess(t *testing.T) {
	var (
		endpoints = []endpoint.Endpoint{
			func(context.Context, interface{}) (interface{}, error) { return nil, errors.New("error one") },
			func(context.Context, interface{}) (interface{}, error) { return nil, errors.New("error two") },
			func(context.Context, interface{}) (interface{}, error) { return struct{}{}, nil /* OK */ },
		}
		endpointer = sd.FixedEndpointer{
			0: endpoints[0],
			1: endpoints[1],
			2: endpoints[2],
		}
		retries = len(endpoints) // exactly enough retries
		rr      = lb.NewRoundRobin(endpointer)
		ctx     = context.Background()
	)
	if _, err := lb.Retry(retries, time.Second, rr)(ctx, struct{}{}); err != nil {
		t.Error(err)
	}
}

func TestRetryTimeout(t *testing.T) {
	var (
		step    = make(chan struct{})
		e       = func(context.Context, interface{}) (interface{}, error) { <-step; return struct{}{}, nil }
		timeout = time.Millisecond
		retry   = lb.Retry(999, timeout, lb.NewRoundRobin(sd.FixedEndpointer{0: e}))
		errs    = make(chan error, 1)
		invoke  = func() { _, err := retry(context.Background(), struct{}{}); errs <- err }
	)

	go func() { step <- struct{}{} }() // queue up a flush of the endpoint
	invoke()                           // invoke the endpoint and trigger the flush
	if err := <-errs; err != nil {     // that should succeed
		t.Error(err)
	}

	go func() { time.Sleep(10 * timeout); step <- struct{}{} }() // a delayed flush
	invoke()                                                     // invoke the endpoint
	if err := <-errs; err != context.DeadlineExceeded {          // that should not succeed
		t.Errorf("wanted %v, got none", context.DeadlineExceeded)
	}
}

func TestAbortEarlyCustomMessage(t *testing.T) {
	var (
		myErr     = errors.New("aborting early")
		cb        = func(int, error) (bool, error) { return false, myErr }
		endpoints = sd.FixedEndpointer{} // no endpoints
		rr        = lb.NewRoundRobin(endpoints)
		retry     = lb.RetryWithCallback(time.Second, rr, cb) // lots of retries
		ctx       = context.Background()
	)
	_, err := retry(ctx, struct{}{})
	if want, have := myErr, err.(lb.RetryError).Final; want != have {
		t.Errorf("want %v, have %v", want, have)
	}
}

func TestErrorPassedUnchangedToCallback(t *testing.T) {
	var (
		myErr = errors.New("my custom error")
		cb    = func(_ int, err error) (bool, error) {
			if want, have := myErr, err; want != have {
				t.Errorf("want %v, have %v", want, have)
			}
			return false, nil
		}
		endpoint = func(ctx context.Context, request interface{}) (interface{}, error) {
			return nil, myErr
		}
		endpoints = sd.FixedEndpointer{endpoint} // no endpoints
		rr        = lb.NewRoundRobin(endpoints)
		retry     = lb.RetryWithCallback(time.Second, rr, cb) // lots of retries
		ctx       = context.Background()
	)
	_, err := retry(ctx, struct{}{})
	if want, have := myErr, err.(lb.RetryError).Final; want != have {
		t.Errorf("want %v, have %v", want, have)
	}
}

func TestHandleNilCallback(t *testing.T) {
	var (
		endpointer = sd.FixedEndpointer{
			func(context.Context, interface{}) (interface{}, error) { return struct{}{}, nil /* OK */ },
		}
		rr  = lb.NewRoundRobin(endpointer)
		ctx = context.Background()
	)
	retry := lb.RetryWithCallback(time.Second, rr, nil)
	if _, err := retry(ctx, struct{}{}); err != nil {
		t.Error(err)
	}
}

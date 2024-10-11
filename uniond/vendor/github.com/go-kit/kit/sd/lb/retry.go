package lb

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/go-kit/kit/endpoint"
)

// RetryError is an error wrapper that is used by the retry mechanism. All
// errors returned by the retry mechanism via its endpoint will be RetryErrors.
type RetryError struct {
	RawErrors []error // all errors encountered from endpoints directly
	Final     error   // the final, terminating error
}

func (e RetryError) Error() string {
	var suffix string
	if len(e.RawErrors) > 1 {
		a := make([]string, len(e.RawErrors)-1)
		for i := 0; i < len(e.RawErrors)-1; i++ { // last one is Final
			a[i] = e.RawErrors[i].Error()
		}
		suffix = fmt.Sprintf(" (previously: %s)", strings.Join(a, "; "))
	}
	return fmt.Sprintf("%v%s", e.Final, suffix)
}

// Callback is a function that is given the current attempt count and the error
// received from the underlying endpoint. It should return whether the Retry
// function should continue trying to get a working endpoint, and a custom error
// if desired. The error message may be nil, but a true/false is always
// expected. In all cases, if the replacement error is supplied, the received
// error will be replaced in the calling context.
type Callback func(n int, received error) (keepTrying bool, replacement error)

// Retry wraps a service load balancer and returns an endpoint oriented load
// balancer for the specified service method. Requests to the endpoint will be
// automatically load balanced via the load balancer. Requests that return
// errors will be retried until they succeed, up to max times, or until the
// timeout is elapsed, whichever comes first.
func Retry(max int, timeout time.Duration, b Balancer) endpoint.Endpoint {
	return RetryWithCallback(timeout, b, maxRetries(max))
}

func maxRetries(max int) Callback {
	return func(n int, err error) (keepTrying bool, replacement error) {
		return n < max, nil
	}
}

func alwaysRetry(int, error) (keepTrying bool, replacement error) {
	return true, nil
}

// RetryWithCallback wraps a service load balancer and returns an endpoint
// oriented load balancer for the specified service method. Requests to the
// endpoint will be automatically load balanced via the load balancer. Requests
// that return errors will be retried until they succeed, up to max times, until
// the callback returns false, or until the timeout is elapsed, whichever comes
// first.
func RetryWithCallback(timeout time.Duration, b Balancer, cb Callback) endpoint.Endpoint {
	if cb == nil {
		cb = alwaysRetry
	}
	if b == nil {
		panic("nil Balancer")
	}

	return func(ctx context.Context, request interface{}) (response interface{}, err error) {
		var (
			newctx, cancel = context.WithTimeout(ctx, timeout)
			responses      = make(chan interface{}, 1)
			errs           = make(chan error, 1)
			final          RetryError
		)
		defer cancel()

		for i := 1; ; i++ {
			go func() {
				e, err := b.Endpoint()
				if err != nil {
					errs <- err
					return
				}
				response, err := e(newctx, request)
				if err != nil {
					errs <- err
					return
				}
				responses <- response
			}()

			select {
			case <-newctx.Done():
				return nil, newctx.Err()

			case response := <-responses:
				return response, nil

			case err := <-errs:
				final.RawErrors = append(final.RawErrors, err)
				keepTrying, replacement := cb(i, err)
				if replacement != nil {
					err = replacement
				}
				if !keepTrying {
					final.Final = err
					return nil, final
				}
				continue
			}
		}
	}
}

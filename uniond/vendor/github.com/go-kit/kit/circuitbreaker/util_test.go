package circuitbreaker_test

import (
	"context"
	"errors"
	"fmt"
	"path/filepath"
	"runtime"
	"testing"
	"time"

	"github.com/go-kit/kit/endpoint"
)

func testFailingEndpoint(
	t *testing.T,
	breaker endpoint.Middleware,
	primeWith int,
	shouldPass func(int) bool,
	requestDelay time.Duration,
	openCircuitError string,
) {
	_, file, line, _ := runtime.Caller(1)
	caller := fmt.Sprintf("%s:%d", filepath.Base(file), line)

	// Create a mock endpoint and wrap it with the breaker.
	m := mock{}
	var e endpoint.Endpoint
	e = m.endpoint
	e = breaker(e)

	// Prime the endpoint with successful requests.
	for i := 0; i < primeWith; i++ {
		if _, err := e(context.Background(), struct{}{}); err != nil {
			t.Fatalf("%s: during priming, got error: %v", caller, err)
		}
		time.Sleep(requestDelay)
	}

	// Switch the endpoint to start throwing errors.
	m.err = errors.New("tragedy+disaster")
	m.through = 0

	// The first several should be allowed through and yield our error.
	for i := 0; shouldPass(i); i++ {
		if _, err := e(context.Background(), struct{}{}); err != m.err {
			t.Fatalf("%s: want %v, have %v", caller, m.err, err)
		}
		time.Sleep(requestDelay)
	}
	through := m.through

	// But the rest should be blocked by an open circuit.
	for i := 0; i < 10; i++ {
		if _, err := e(context.Background(), struct{}{}); err.Error() != openCircuitError {
			t.Fatalf("%s: want %q, have %q", caller, openCircuitError, err.Error())
		}
		time.Sleep(requestDelay)
	}

	// Make sure none of those got through.
	if want, have := through, m.through; want != have {
		t.Errorf("%s: want %d, have %d", caller, want, have)
	}
}

type mock struct {
	through int
	err     error
}

func (m *mock) endpoint(context.Context, interface{}) (interface{}, error) {
	m.through++
	return struct{}{}, m.err
}

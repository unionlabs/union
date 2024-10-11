package circuitbreaker_test

import (
	"testing"

	"github.com/sony/gobreaker"

	"github.com/go-kit/kit/circuitbreaker"
)

func TestGobreaker(t *testing.T) {
	var (
		breaker          = circuitbreaker.Gobreaker(gobreaker.NewCircuitBreaker(gobreaker.Settings{}))
		primeWith        = 100
		shouldPass       = func(n int) bool { return n <= 5 } // https://github.com/sony/gobreaker/blob/bfa846d/gobreaker.go#L76
		circuitOpenError = "circuit breaker is open"
	)
	testFailingEndpoint(t, breaker, primeWith, shouldPass, 0, circuitOpenError)
}

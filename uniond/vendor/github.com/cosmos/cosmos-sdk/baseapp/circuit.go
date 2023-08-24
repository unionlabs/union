package baseapp

import "context"

// CircuitBreaker is an interface that defines the methods for a circuit breaker.
type CircuitBreaker interface {
	IsAllowed(ctx context.Context, typeURL string) (bool, error)
}

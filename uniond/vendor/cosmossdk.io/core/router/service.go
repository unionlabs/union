package router

import (
	"context"

	"cosmossdk.io/core/transaction"
)

// Service is the interface that wraps the basic methods for a router.
// A router can be a query router or a message router.
type Service interface {
	// CanInvoke returns an error if the given request cannot be invoked.
	CanInvoke(ctx context.Context, typeURL string) error
	// Invoke execute a message or query. The response should be type casted by the caller to the expected response.
	Invoke(ctx context.Context, req transaction.Msg) (res transaction.Msg, err error)
}

package awslambda

import (
	"context"
)

// HandlerRequestFunc may take information from the received
// payload and use it to place items in the request scoped context.
// HandlerRequestFuncs are executed prior to invoking the endpoint and
// decoding of the payload.
type HandlerRequestFunc func(ctx context.Context, payload []byte) context.Context

// HandlerResponseFunc may take information from a request context
// and use it to manipulate the response before it's marshaled.
// HandlerResponseFunc are executed after invoking the endpoint
// but prior to returning a response.
type HandlerResponseFunc func(ctx context.Context, response interface{}) context.Context

// HandlerFinalizerFunc is executed at the end of Invoke.
// This can be used for logging purposes.
type HandlerFinalizerFunc func(ctx context.Context, resp []byte, err error)

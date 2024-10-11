package awslambda

import (
	"context"
)

// DecodeRequestFunc extracts a user-domain request object from an
// AWS Lambda payload.
type DecodeRequestFunc func(context.Context, []byte) (interface{}, error)

// EncodeResponseFunc encodes the passed response object into []byte,
// ready to be sent as AWS Lambda response.
type EncodeResponseFunc func(context.Context, interface{}) ([]byte, error)

// ErrorEncoder is responsible for encoding an error.
type ErrorEncoder func(ctx context.Context, err error) ([]byte, error)

package nats

import (
	"context"

	"github.com/nats-io/nats.go"
)

// DecodeRequestFunc extracts a user-domain request object from a publisher
// request object. It's designed to be used in NATS subscribers, for subscriber-side
// endpoints. One straightforward DecodeRequestFunc could be something that
// JSON decodes from the request body to the concrete response type.
type DecodeRequestFunc func(context.Context, *nats.Msg) (request interface{}, err error)

// EncodeRequestFunc encodes the passed request object into the NATS request
// object. It's designed to be used in NATS publishers, for publisher-side
// endpoints. One straightforward EncodeRequestFunc could something that JSON
// encodes the object directly to the request payload.
type EncodeRequestFunc func(context.Context, *nats.Msg, interface{}) error

// EncodeResponseFunc encodes the passed response object to the subscriber reply.
// It's designed to be used in NATS subscribers, for subscriber-side
// endpoints. One straightforward EncodeResponseFunc could be something that
// JSON encodes the object directly to the response body.
type EncodeResponseFunc func(context.Context, string, *nats.Conn, interface{}) error

// DecodeResponseFunc extracts a user-domain response object from an NATS
// response object. It's designed to be used in NATS publisher, for publisher-side
// endpoints. One straightforward DecodeResponseFunc could be something that
// JSON decodes from the response payload to the concrete response type.
type DecodeResponseFunc func(context.Context, *nats.Msg) (response interface{}, err error)


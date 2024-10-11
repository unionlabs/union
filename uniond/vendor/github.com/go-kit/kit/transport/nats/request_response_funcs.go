package nats

import (
	"context"

	"github.com/nats-io/nats.go"
)

// RequestFunc may take information from a publisher request and put it into a
// request context. In Subscribers, RequestFuncs are executed prior to invoking the
// endpoint.
type RequestFunc func(context.Context, *nats.Msg) context.Context

// SubscriberResponseFunc may take information from a request context and use it to
// manipulate a Publisher. SubscriberResponseFuncs are only executed in
// subscribers, after invoking the endpoint but prior to publishing a reply.
type SubscriberResponseFunc func(context.Context, *nats.Conn) context.Context

// PublisherResponseFunc may take information from an NATS request and make the
// response available for consumption. ClientResponseFuncs are only executed in
// clients, after a request has been made, but prior to it being decoded.
type PublisherResponseFunc func(context.Context, *nats.Msg) context.Context

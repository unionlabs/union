package http

import (
	"context"
	"net/http"
)

// DecodeRequestFunc extracts a user-domain request object from an HTTP
// request object. It's designed to be used in HTTP servers, for server-side
// endpoints. One straightforward DecodeRequestFunc could be something that
// JSON decodes from the request body to the concrete request type.
type DecodeRequestFunc func(context.Context, *http.Request) (request interface{}, err error)

// EncodeRequestFunc encodes the passed request object into the HTTP request
// object. It's designed to be used in HTTP clients, for client-side
// endpoints. One straightforward EncodeRequestFunc could be something that JSON
// encodes the object directly to the request body.
type EncodeRequestFunc func(context.Context, *http.Request, interface{}) error

// CreateRequestFunc creates an outgoing HTTP request based on the passed
// request object. It's designed to be used in HTTP clients, for client-side
// endpoints. It's a more powerful version of EncodeRequestFunc, and can be used
// if more fine-grained control of the HTTP request is required.
type CreateRequestFunc func(context.Context, interface{}) (*http.Request, error)

// EncodeResponseFunc encodes the passed response object to the HTTP response
// writer. It's designed to be used in HTTP servers, for server-side
// endpoints. One straightforward EncodeResponseFunc could be something that
// JSON encodes the object directly to the response body.
type EncodeResponseFunc func(context.Context, http.ResponseWriter, interface{}) error

// DecodeResponseFunc extracts a user-domain response object from an HTTP
// response object. It's designed to be used in HTTP clients, for client-side
// endpoints. One straightforward DecodeResponseFunc could be something that
// JSON decodes from the response body to the concrete response type.
type DecodeResponseFunc func(context.Context, *http.Response) (response interface{}, err error)

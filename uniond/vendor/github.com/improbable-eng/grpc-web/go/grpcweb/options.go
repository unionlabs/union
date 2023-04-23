//Copyright 2017 Improbable. All Rights Reserved.
// See LICENSE for licensing terms.

package grpcweb

import (
	"net/http"
	"time"
)

var (
	defaultOptions = &options{
		allowedRequestHeaders:          []string{"*"},
		corsForRegisteredEndpointsOnly: true,
		originFunc:                     func(origin string) bool { return false },
		allowNonRootResources:          false,
	}
)

type options struct {
	allowedRequestHeaders          []string
	corsForRegisteredEndpointsOnly bool
	originFunc                     func(origin string) bool
	enableWebsockets               bool
	websocketPingInterval          time.Duration
	websocketOriginFunc            func(req *http.Request) bool
	websocketReadLimit             int64
	allowNonRootResources          bool
	endpointsFunc                  *func() []string
}

func evaluateOptions(opts []Option) *options {
	optCopy := &options{}
	*optCopy = *defaultOptions
	for _, o := range opts {
		o(optCopy)
	}
	return optCopy
}

type Option func(*options)

// WithOriginFunc allows for customizing what CORS Origin requests are allowed.
//
// This is controlling the CORS pre-flight `Access-Control-Allow-Origin`. This mechanism allows you to limit the
// availability of the APIs based on the domain name of the calling website (Origin). You can provide a function that
// filters the allowed Origin values.
//
// The default behaviour is to deny all requests from remote origins.
//
// The relevant CORS pre-flight docs:
// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Origin
func WithOriginFunc(originFunc func(origin string) bool) Option {
	return func(o *options) {
		o.originFunc = originFunc
	}
}

// WithCorsForRegisteredEndpointsOnly allows for customizing whether OPTIONS requests with the `X-GRPC-WEB` header will
// only be accepted if they match a registered gRPC endpoint.
//
// This should be set to false to allow handling gRPC requests for unknown endpoints (e.g. for proxying).
//
// The default behaviour is `true`, i.e. only allows CORS requests for registered endpoints.
func WithCorsForRegisteredEndpointsOnly(onlyRegistered bool) Option {
	return func(o *options) {
		o.corsForRegisteredEndpointsOnly = onlyRegistered
	}
}

// WithEndpointsFunc allows for providing a custom function that provides all supported endpoints for use when the
// when `WithCorsForRegisteredEndpoints` option` is not set to false (i.e. the default state).
//
// When wrapping a http.Handler with `WrapHttpHandler`, failing to specify the `WithEndpointsFunc` option will cause
// all CORS requests to result in a 403 error for websocket requests (if websockets are enabled) or be passed to the
// handler http.Handler or grpc.Server backend (i.e. as if it wasn't wrapped).
//
// When wrapping grpc.Server with `WrapGrpcServer`, registered endpoints will be automatically detected, however if this
// `WithEndpointsFunc` option is specified, the server will not be queried for its endpoints and this function will
// be called instead.
func WithEndpointsFunc(endpointsFunc func() []string) Option {
	return func(o *options) {
		o.endpointsFunc = &endpointsFunc
	}
}

// WithAllowedRequestHeaders allows for customizing what gRPC request headers a browser can add.
//
// This is controlling the CORS pre-flight `Access-Control-Allow-Headers` method and applies to *all* gRPC handlers.
// However, a special `*` value can be passed in that allows
// the browser client to provide *any* header, by explicitly whitelisting all `Access-Control-Request-Headers` of the
// pre-flight request.
//
// The default behaviour is `[]string{'*'}`, allowing all browser client headers. This option overrides that default,
// while maintaining a whitelist for gRPC-internal headers.
//
// Unfortunately, since the CORS pre-flight happens independently from gRPC handler execution, it is impossible to
// automatically discover it from the gRPC handler itself.
//
// The relevant CORS pre-flight docs:
// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Headers
func WithAllowedRequestHeaders(headers []string) Option {
	return func(o *options) {
		o.allowedRequestHeaders = headers
	}
}

// WithWebsockets allows for handling grpc-web requests of websockets - enabling bidirectional requests.
//
// The default behaviour is false, i.e. to disallow websockets
func WithWebsockets(enableWebsockets bool) Option {
	return func(o *options) {
		o.enableWebsockets = enableWebsockets
	}
}

// WithWebsocketPingInterval enables websocket keepalive pinging with the configured timeout.
//
// The default behaviour is to disable websocket pinging.
func WithWebsocketPingInterval(websocketPingInterval time.Duration) Option {
	return func(o *options) {
		o.websocketPingInterval = websocketPingInterval
	}
}

// WithWebsocketOriginFunc allows for customizing the acceptance of Websocket requests - usually to check that the origin
// is valid.
//
// The default behaviour is to check that the origin of the request matches the host of the request and deny all requests from remote origins.
func WithWebsocketOriginFunc(websocketOriginFunc func(req *http.Request) bool) Option {
	return func(o *options) {
		o.websocketOriginFunc = websocketOriginFunc
	}
}

// WithWebsocketsMessageReadLimit sets the maximum message read limit on the underlying websocket.
//
// The default message read limit is 32769 bytes
func WithWebsocketsMessageReadLimit(websocketReadLimit int64) Option {
	return func(o *options) {
		o.websocketReadLimit = websocketReadLimit
	}
}

// WithAllowNonRootResource enables the gRPC wrapper to serve requests that have a path prefix
// added to the URL, before the service name and method placeholders.
//
// This should be set to false when exposing the endpoint as the root resource, to avoid
// the performance cost of path processing for every request.
//
// The default behaviour is `false`, i.e. always serves requests assuming there is no prefix to the gRPC endpoint.
func WithAllowNonRootResource(allowNonRootResources bool) Option {
	return func(o *options) {
		o.allowNonRootResources = allowNonRootResources
	}
}

# grpcweb
--
    import "github.com/improbable-eng/grpc-web/go/grpcweb"

`grpcweb` implements the gRPC-Web spec as a wrapper around a gRPC-Go Server.

It allows web clients (see companion JS library) to talk to gRPC-Go servers over
the gRPC-Web spec. It supports HTTP/1.1 and HTTP2 encoding of a gRPC stream and
supports unary and server-side streaming RPCs. Bi-di and client streams are
unsupported due to limitations in browser protocol support.

See https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-WEB.md for the
protocol specification.

Here's an example of how to use it inside an existing gRPC Go server on a
separate http.Server that serves over TLS:

    grpcServer := grpc.Server()
    wrappedGrpc := grpcweb.WrapServer(grpcServer)
    tlsHttpServer.Handler = http.HandlerFunc(func(resp http.ResponseWriter, req *http.Request) {
    	if wrappedGrpc.IsGrpcWebRequest(req) {
    		wrappedGrpc.ServeHTTP(resp, req)
    		return
    	}
    	// Fall back to other servers.
    	http.DefaultServeMux.ServeHTTP(resp, req)
    })

If you'd like to have a standalone binary, please take a look at `grpcwebproxy`.

## Usage

#### func  ClientHealthCheck

```go
func ClientHealthCheck(ctx context.Context, backendConn *grpc.ClientConn, service string, setServingStatus func(serving bool)) error
```
Client health check function is also part of the grpc/internal package The
following code is a simplified version of client.go For more details see:
https://pkg.go.dev/google.golang.org/grpc/health

#### func  ListGRPCResources

```go
func ListGRPCResources(server *grpc.Server) []string
```
ListGRPCResources is a helper function that lists all URLs that are registered
on gRPC server.

This makes it easy to register all the relevant routes in your HTTP router of
choice.

#### func  WebsocketRequestOrigin

```go
func WebsocketRequestOrigin(req *http.Request) (string, error)
```
WebsocketRequestOrigin returns the host from which a websocket request made by a
web browser originated.

#### type Option

```go
type Option func(*options)
```


#### func  WithAllowNonRootResource

```go
func WithAllowNonRootResource(allowNonRootResources bool) Option
```
WithAllowNonRootResource enables the gRPC wrapper to serve requests that have a
path prefix added to the URL, before the service name and method placeholders.

This should be set to false when exposing the endpoint as the root resource, to
avoid the performance cost of path processing for every request.

The default behaviour is `false`, i.e. always serves requests assuming there is
no prefix to the gRPC endpoint.

#### func  WithAllowedRequestHeaders

```go
func WithAllowedRequestHeaders(headers []string) Option
```
WithAllowedRequestHeaders allows for customizing what gRPC request headers a
browser can add.

This is controlling the CORS pre-flight `Access-Control-Allow-Headers` method
and applies to *all* gRPC handlers. However, a special `*` value can be passed
in that allows the browser client to provide *any* header, by explicitly
whitelisting all `Access-Control-Request-Headers` of the pre-flight request.

The default behaviour is `[]string{'*'}`, allowing all browser client headers.
This option overrides that default, while maintaining a whitelist for
gRPC-internal headers.

Unfortunately, since the CORS pre-flight happens independently from gRPC handler
execution, it is impossible to automatically discover it from the gRPC handler
itself.

The relevant CORS pre-flight docs:
https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Headers

#### func  WithCorsForRegisteredEndpointsOnly

```go
func WithCorsForRegisteredEndpointsOnly(onlyRegistered bool) Option
```
WithCorsForRegisteredEndpointsOnly allows for customizing whether OPTIONS
requests with the `X-GRPC-WEB` header will only be accepted if they match a
registered gRPC endpoint.

This should be set to false to allow handling gRPC requests for unknown
endpoints (e.g. for proxying).

The default behaviour is `true`, i.e. only allows CORS requests for registered
endpoints.

#### func  WithEndpointsFunc

```go
func WithEndpointsFunc(endpointsFunc func() []string) Option
```
WithEndpointsFunc allows for providing a custom function that provides all
supported endpoints for use when the when `WithCorsForRegisteredEndpoints`
option` is not set to false (i.e. the default state).

When wrapping a http.Handler with `WrapHttpHandler`, failing to specify the
`WithEndpointsFunc` option will cause all CORS requests to result in a 403 error
for websocket requests (if websockets are enabled) or be passed to the handler
http.Handler or grpc.Server backend (i.e. as if it wasn't wrapped).

When wrapping grpc.Server with `WrapGrpcServer`, registered endpoints will be
automatically detected, however if this `WithEndpointsFunc` option is specified,
the server will not be queried for its endpoints and this function will be
called instead.

#### func  WithOriginFunc

```go
func WithOriginFunc(originFunc func(origin string) bool) Option
```
WithOriginFunc allows for customizing what CORS Origin requests are allowed.

This is controlling the CORS pre-flight `Access-Control-Allow-Origin`. This
mechanism allows you to limit the availability of the APIs based on the domain
name of the calling website (Origin). You can provide a function that filters
the allowed Origin values.

The default behaviour is to deny all requests from remote origins.

The relevant CORS pre-flight docs:
https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Origin

#### func  WithWebsocketOriginFunc

```go
func WithWebsocketOriginFunc(websocketOriginFunc func(req *http.Request) bool) Option
```
WithWebsocketOriginFunc allows for customizing the acceptance of Websocket
requests - usually to check that the origin is valid.

The default behaviour is to check that the origin of the request matches the
host of the request and deny all requests from remote origins.

#### func  WithWebsocketPingInterval

```go
func WithWebsocketPingInterval(websocketPingInterval time.Duration) Option
```
WithWebsocketPingInterval enables websocket keepalive pinging with the
configured timeout.

The default behaviour is to disable websocket pinging.

#### func  WithWebsockets

```go
func WithWebsockets(enableWebsockets bool) Option
```
WithWebsockets allows for handling grpc-web requests of websockets - enabling
bidirectional requests.

The default behaviour is false, i.e. to disallow websockets

#### func  WithWebsocketsMessageReadLimit

```go
func WithWebsocketsMessageReadLimit(websocketReadLimit int64) Option
```
WithWebsocketsMessageReadLimit sets the maximum message read limit on the
underlying websocket.

The default message read limit is 32769 bytes

#### type WrappedGrpcServer

```go
type WrappedGrpcServer struct {
}
```


#### func  WrapHandler

```go
func WrapHandler(handler http.Handler, options ...Option) *WrappedGrpcServer
```
WrapHandler takes a http.Handler (such as a http.Mux) and returns a
*WrappedGrpcServer that provides gRPC-Web Compatibility.

This behaves nearly identically to WrapServer except when the
WithCorsForRegisteredEndpointsOnly setting is true. Then a WithEndpointsFunc
option must be provided or all CORS requests will NOT be handled.

#### func  WrapServer

```go
func WrapServer(server *grpc.Server, options ...Option) *WrappedGrpcServer
```
WrapServer takes a gRPC Server in Go and returns a *WrappedGrpcServer that
provides gRPC-Web Compatibility.

The internal implementation fakes out a http.Request that carries standard gRPC,
and performs the remapping inside http.ResponseWriter, i.e. mostly the
re-encoding of Trailers (that carry gRPC status).

You can control the behaviour of the wrapper (e.g. modifying CORS behaviour)
using `With*` options.

#### func (*WrappedGrpcServer) HandleGrpcWebRequest

```go
func (w *WrappedGrpcServer) HandleGrpcWebRequest(resp http.ResponseWriter, req *http.Request)
```
HandleGrpcWebRequest takes a HTTP request that is assumed to be a gRPC-Web
request and wraps it with a compatibility layer to transform it to a standard
gRPC request for the wrapped gRPC server and transforms the response to comply
with the gRPC-Web protocol.

#### func (*WrappedGrpcServer) HandleGrpcWebsocketRequest

```go
func (w *WrappedGrpcServer) HandleGrpcWebsocketRequest(resp http.ResponseWriter, req *http.Request)
```
HandleGrpcWebsocketRequest takes a HTTP request that is assumed to be a
gRPC-Websocket request and wraps it with a compatibility layer to transform it
to a standard gRPC request for the wrapped gRPC server and transforms the
response to comply with the gRPC-Web protocol.

#### func (*WrappedGrpcServer) IsAcceptableGrpcCorsRequest

```go
func (w *WrappedGrpcServer) IsAcceptableGrpcCorsRequest(req *http.Request) bool
```
IsAcceptableGrpcCorsRequest determines if a request is a CORS pre-flight request
for a gRPC-Web request and that this request is acceptable for CORS.

You can control the CORS behaviour using `With*` options in the WrapServer
function.

#### func (*WrappedGrpcServer) IsGrpcWebRequest

```go
func (w *WrappedGrpcServer) IsGrpcWebRequest(req *http.Request) bool
```
IsGrpcWebRequest determines if a request is a gRPC-Web request by checking that
the "content-type" is "application/grpc-web" and that the method is POST.

#### func (*WrappedGrpcServer) IsGrpcWebSocketRequest

```go
func (w *WrappedGrpcServer) IsGrpcWebSocketRequest(req *http.Request) bool
```
IsGrpcWebSocketRequest determines if a request is a gRPC-Web request by checking
that the "Sec-Websocket-Protocol" header value is "grpc-websockets"

#### func (*WrappedGrpcServer) ServeHTTP

```go
func (w *WrappedGrpcServer) ServeHTTP(resp http.ResponseWriter, req *http.Request)
```
ServeHTTP takes a HTTP request and if it is a gRPC-Web request wraps it with a
compatibility layer to transform it to a standard gRPC request for the wrapped
gRPC server and transforms the response to comply with the gRPC-Web protocol.

The gRPC-Web compatibility is only invoked if the request is a gRPC-Web request
as determined by IsGrpcWebRequest or the request is a pre-flight (CORS) request
as determined by IsAcceptableGrpcCorsRequest.

You can control the CORS behaviour using `With*` options in the WrapServer
function.

//Copyright 2017 Improbable. All Rights Reserved.
// See LICENSE for licensing terms.

package grpcweb

import (
	"context"
	"encoding/base64"
	"io"
	"net/http"
	"strings"
	"time"

	"github.com/rs/cors"
	"google.golang.org/grpc"
	"google.golang.org/grpc/grpclog"
	"nhooyr.io/websocket"
)

var (
	internalRequestHeadersWhitelist = []string{
		"U-A", // for gRPC-Web User Agent indicator.
	}
)

// https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-WEB.md#protocol-differences-vs-grpc-over-http2
const grpcContentType = "application/grpc"
const grpcWebContentType = "application/grpc-web"
const grpcWebTextContentType = "application/grpc-web-text"

type WrappedGrpcServer struct {
	handler             http.Handler
	opts                *options
	corsWrapper         *cors.Cors
	originFunc          func(origin string) bool
	enableWebsockets    bool
	websocketOriginFunc func(req *http.Request) bool
	websocketReadLimit  int64
	allowedHeaders      []string
	endpointFunc        func(req *http.Request) string
	endpointsFunc       func() []string
}

// WrapServer takes a gRPC Server in Go and returns a *WrappedGrpcServer that provides gRPC-Web Compatibility.
//
// The internal implementation fakes out a http.Request that carries standard gRPC, and performs the remapping inside
// http.ResponseWriter, i.e. mostly the re-encoding of Trailers (that carry gRPC status).
//
// You can control the behaviour of the wrapper (e.g. modifying CORS behaviour) using `With*` options.
func WrapServer(server *grpc.Server, options ...Option) *WrappedGrpcServer {
	return wrapGrpc(options, server, func() []string {
		return ListGRPCResources(server)
	})
}

// WrapHandler takes a http.Handler (such as a http.Mux) and returns a *WrappedGrpcServer that provides gRPC-Web
// Compatibility.
//
// This behaves nearly identically to WrapServer except when the WithCorsForRegisteredEndpointsOnly setting is true.
// Then a WithEndpointsFunc option must be provided or all CORS requests will NOT be handled.
func WrapHandler(handler http.Handler, options ...Option) *WrappedGrpcServer {
	return wrapGrpc(options, handler, func() []string {
		return []string{}
	})
}

func wrapGrpc(options []Option, handler http.Handler, endpointsFunc func() []string) *WrappedGrpcServer {
	opts := evaluateOptions(options)
	allowedHeaders := append(opts.allowedRequestHeaders, internalRequestHeadersWhitelist...)
	corsWrapper := cors.New(cors.Options{
		AllowOriginFunc:  opts.originFunc,
		AllowedHeaders:   allowedHeaders,
		ExposedHeaders:   nil,                                 // make sure that this is *nil*, otherwise the WebResponse overwrite will not work.
		AllowCredentials: true,                                // always allow credentials, otherwise :authorization headers won't work
		MaxAge:           int(10 * time.Minute / time.Second), // make sure pre-flights don't happen too often (every 5s for Chromium :( )
	})
	websocketOriginFunc := opts.websocketOriginFunc
	if websocketOriginFunc == nil {
		websocketOriginFunc = defaultWebsocketOriginFunc
	}

	endpointFunc := func(req *http.Request) string {
		return req.URL.Path
	}

	if opts.allowNonRootResources {
		endpointFunc = getGRPCEndpoint
	}

	if opts.endpointsFunc != nil {
		endpointsFunc = *opts.endpointsFunc
	}

	return &WrappedGrpcServer{
		handler:             handler,
		opts:                opts,
		corsWrapper:         corsWrapper,
		originFunc:          opts.originFunc,
		enableWebsockets:    opts.enableWebsockets,
		websocketOriginFunc: websocketOriginFunc,
		websocketReadLimit:  opts.websocketReadLimit,
		allowedHeaders:      allowedHeaders,
		endpointFunc:        endpointFunc,
		endpointsFunc:       endpointsFunc,
	}
}

// ServeHTTP takes a HTTP request and if it is a gRPC-Web request wraps it with a compatibility layer to transform it to
// a standard gRPC request for the wrapped gRPC server and transforms the response to comply with the gRPC-Web protocol.
//
// The gRPC-Web compatibility is only invoked if the request is a gRPC-Web request as determined by IsGrpcWebRequest or
// the request is a pre-flight (CORS) request as determined by IsAcceptableGrpcCorsRequest.
//
// You can control the CORS behaviour using `With*` options in the WrapServer function.
func (w *WrappedGrpcServer) ServeHTTP(resp http.ResponseWriter, req *http.Request) {
	if w.enableWebsockets && w.IsGrpcWebSocketRequest(req) {
		if w.websocketOriginFunc(req) {
			if !w.opts.corsForRegisteredEndpointsOnly || w.isRequestForRegisteredEndpoint(req) {
				w.HandleGrpcWebsocketRequest(resp, req)
				return
			}
		}
		resp.WriteHeader(403)
		_, _ = resp.Write(make([]byte, 0))
		return
	}

	if w.IsAcceptableGrpcCorsRequest(req) || w.IsGrpcWebRequest(req) {
		w.corsWrapper.Handler(http.HandlerFunc(w.HandleGrpcWebRequest)).ServeHTTP(resp, req)
		return
	}
	w.handler.ServeHTTP(resp, req)
}

// IsGrpcWebSocketRequest determines if a request is a gRPC-Web request by checking that the "Sec-Websocket-Protocol"
// header value is "grpc-websockets"
func (w *WrappedGrpcServer) IsGrpcWebSocketRequest(req *http.Request) bool {
	return strings.ToLower(req.Header.Get("Upgrade")) == "websocket" && strings.ToLower(req.Header.Get("Sec-Websocket-Protocol")) == "grpc-websockets"
}

// HandleGrpcWebRequest takes a HTTP request that is assumed to be a gRPC-Web request and wraps it with a compatibility
// layer to transform it to a standard gRPC request for the wrapped gRPC server and transforms the response to comply
// with the gRPC-Web protocol.
func (w *WrappedGrpcServer) HandleGrpcWebRequest(resp http.ResponseWriter, req *http.Request) {
	intReq, isTextFormat := hackIntoNormalGrpcRequest(req)
	intResp := newGrpcWebResponse(resp, isTextFormat)
	req.URL.Path = w.endpointFunc(req)
	w.handler.ServeHTTP(intResp, intReq)
	intResp.finishRequest(req)
}

// HandleGrpcWebsocketRequest takes a HTTP request that is assumed to be a gRPC-Websocket request and wraps it with a
// compatibility layer to transform it to a standard gRPC request for the wrapped gRPC server and transforms the
// response to comply with the gRPC-Web protocol.
func (w *WrappedGrpcServer) HandleGrpcWebsocketRequest(resp http.ResponseWriter, req *http.Request) {

	wsConn, err := websocket.Accept(resp, req, &websocket.AcceptOptions{
		InsecureSkipVerify: true, // managed by ServeHTTP
		Subprotocols:       []string{"grpc-websockets"},
	})
	if err != nil {
		grpclog.Errorf("Unable to upgrade websocket request: %v", err)
		return
	}

	if w.websocketReadLimit > 0 {
		wsConn.SetReadLimit(w.websocketReadLimit)
	}

	headers := make(http.Header)
	for _, name := range w.allowedHeaders {
		if values, exist := req.Header[name]; exist {
			headers[name] = values
		}
	}

	ctx, cancelFunc := context.WithCancel(req.Context())
	defer cancelFunc()

	messageType, readBytes, err := wsConn.Read(ctx)
	if err != nil {
		grpclog.Errorf("Unable to read first websocket message: %v %v %v", messageType, readBytes, err)
		return
	}

	if messageType != websocket.MessageBinary {
		grpclog.Errorf("First websocket message is non-binary")
		return
	}

	wsHeaders, err := parseHeaders(string(readBytes))
	if err != nil {
		grpclog.Errorf("Unable to parse websocket headers: %v", err)
		return
	}

	respWriter := newWebSocketResponseWriter(ctx, wsConn)
	if w.opts.websocketPingInterval >= time.Second {
		respWriter.enablePing(w.opts.websocketPingInterval)
	}
	wrappedReader := newWebsocketWrappedReader(ctx, wsConn, respWriter, cancelFunc)

	for name, values := range wsHeaders {
		headers[name] = values
	}
	req.Body = wrappedReader
	req.Method = http.MethodPost
	req.Header = headers

	interceptedRequest, isTextFormat := hackIntoNormalGrpcRequest(req.WithContext(ctx))
	if isTextFormat {
		grpclog.Errorf("web socket text format requests not yet supported")
	}
	req.URL.Path = w.endpointFunc(req)
	w.handler.ServeHTTP(respWriter, interceptedRequest)
}

// IsGrpcWebRequest determines if a request is a gRPC-Web request by checking that the "content-type" is
// "application/grpc-web" and that the method is POST.
func (w *WrappedGrpcServer) IsGrpcWebRequest(req *http.Request) bool {
	return req.Method == http.MethodPost && strings.HasPrefix(req.Header.Get("content-type"), grpcWebContentType)
}

// IsAcceptableGrpcCorsRequest determines if a request is a CORS pre-flight request for a gRPC-Web request and that this
// request is acceptable for CORS.
//
// You can control the CORS behaviour using `With*` options in the WrapServer function.
func (w *WrappedGrpcServer) IsAcceptableGrpcCorsRequest(req *http.Request) bool {
	accessControlHeaders := strings.ToLower(req.Header.Get("Access-Control-Request-Headers"))
	if req.Method == http.MethodOptions && strings.Contains(accessControlHeaders, "x-grpc-web") {
		if w.opts.corsForRegisteredEndpointsOnly {
			return w.isRequestForRegisteredEndpoint(req)
		}
		return true
	}
	return false
}

func (w *WrappedGrpcServer) isRequestForRegisteredEndpoint(req *http.Request) bool {
	registeredEndpoints := w.endpointsFunc()
	requestedEndpoint := w.endpointFunc(req)
	for _, v := range registeredEndpoints {
		if v == requestedEndpoint {
			return true
		}
	}
	return false
}

// readerCloser combines an io.Reader and an io.Closer into an io.ReadCloser.
type readerCloser struct {
	reader io.Reader
	closer io.Closer
}

func (r *readerCloser) Read(dest []byte) (int, error) {
	return r.reader.Read(dest)
}
func (r *readerCloser) Close() error {
	return r.closer.Close()
}

func hackIntoNormalGrpcRequest(req *http.Request) (*http.Request, bool) {
	// Hack, this should be a shallow copy, but let's see if this works
	req.ProtoMajor = 2
	req.ProtoMinor = 0

	contentType := req.Header.Get("content-type")
	incomingContentType := grpcWebContentType
	isTextFormat := strings.HasPrefix(contentType, grpcWebTextContentType)
	if isTextFormat {
		// body is base64-encoded: decode it; Wrap it in readerCloser so Body is still closed
		decoder := base64.NewDecoder(base64.StdEncoding, req.Body)
		req.Body = &readerCloser{reader: decoder, closer: req.Body}
		incomingContentType = grpcWebTextContentType
	}
	req.Header.Set("content-type", strings.Replace(contentType, incomingContentType, grpcContentType, 1))

	// Remove content-length header since it represents http1.1 payload size, not the sum of the h2
	// DATA frame payload lengths. https://http2.github.io/http2-spec/#malformed This effectively
	// switches to chunked encoding which is the default for h2
	req.Header.Del("content-length")

	return req, isTextFormat
}

func defaultWebsocketOriginFunc(req *http.Request) bool {
	origin, err := WebsocketRequestOrigin(req)
	if err != nil {
		grpclog.Warning(err)
		return false
	}
	return origin == req.Host
}

package http

import (
	"context"
	"encoding/json"
	"net/http"

	"github.com/go-kit/kit/endpoint"
	"github.com/go-kit/kit/transport"
	"github.com/go-kit/log"
)

// Server wraps an endpoint and implements http.Handler.
type Server struct {
	e            endpoint.Endpoint
	dec          DecodeRequestFunc
	enc          EncodeResponseFunc
	before       []RequestFunc
	after        []ServerResponseFunc
	errorEncoder ErrorEncoder
	finalizer    []ServerFinalizerFunc
	errorHandler transport.ErrorHandler
}

// NewServer constructs a new server, which implements http.Handler and wraps
// the provided endpoint.
func NewServer(
	e endpoint.Endpoint,
	dec DecodeRequestFunc,
	enc EncodeResponseFunc,
	options ...ServerOption,
) *Server {
	s := &Server{
		e:            e,
		dec:          dec,
		enc:          enc,
		errorEncoder: DefaultErrorEncoder,
		errorHandler: transport.NewLogErrorHandler(log.NewNopLogger()),
	}
	for _, option := range options {
		option(s)
	}
	return s
}

// ServerOption sets an optional parameter for servers.
type ServerOption func(*Server)

// ServerBefore functions are executed on the HTTP request object before the
// request is decoded.
func ServerBefore(before ...RequestFunc) ServerOption {
	return func(s *Server) { s.before = append(s.before, before...) }
}

// ServerAfter functions are executed on the HTTP response writer after the
// endpoint is invoked, but before anything is written to the client.
func ServerAfter(after ...ServerResponseFunc) ServerOption {
	return func(s *Server) { s.after = append(s.after, after...) }
}

// ServerErrorEncoder is used to encode errors to the http.ResponseWriter
// whenever they're encountered in the processing of a request. Clients can
// use this to provide custom error formatting and response codes. By default,
// errors will be written with the DefaultErrorEncoder.
func ServerErrorEncoder(ee ErrorEncoder) ServerOption {
	return func(s *Server) { s.errorEncoder = ee }
}

// ServerErrorLogger is used to log non-terminal errors. By default, no errors
// are logged. This is intended as a diagnostic measure. Finer-grained control
// of error handling, including logging in more detail, should be performed in a
// custom ServerErrorEncoder or ServerFinalizer, both of which have access to
// the context.
// Deprecated: Use ServerErrorHandler instead.
func ServerErrorLogger(logger log.Logger) ServerOption {
	return func(s *Server) { s.errorHandler = transport.NewLogErrorHandler(logger) }
}

// ServerErrorHandler is used to handle non-terminal errors. By default, non-terminal errors
// are ignored. This is intended as a diagnostic measure. Finer-grained control
// of error handling, including logging in more detail, should be performed in a
// custom ServerErrorEncoder or ServerFinalizer, both of which have access to
// the context.
func ServerErrorHandler(errorHandler transport.ErrorHandler) ServerOption {
	return func(s *Server) { s.errorHandler = errorHandler }
}

// ServerFinalizer is executed at the end of every HTTP request.
// By default, no finalizer is registered.
func ServerFinalizer(f ...ServerFinalizerFunc) ServerOption {
	return func(s *Server) { s.finalizer = append(s.finalizer, f...) }
}

// ServeHTTP implements http.Handler.
func (s Server) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()

	if len(s.finalizer) > 0 {
		iw := &interceptingWriter{w, http.StatusOK, 0}
		defer func() {
			ctx = context.WithValue(ctx, ContextKeyResponseHeaders, iw.Header())
			ctx = context.WithValue(ctx, ContextKeyResponseSize, iw.written)
			for _, f := range s.finalizer {
				f(ctx, iw.code, r)
			}
		}()
		w = iw.reimplementInterfaces()
	}

	for _, f := range s.before {
		ctx = f(ctx, r)
	}

	request, err := s.dec(ctx, r)
	if err != nil {
		s.errorHandler.Handle(ctx, err)
		s.errorEncoder(ctx, err, w)
		return
	}

	response, err := s.e(ctx, request)
	if err != nil {
		s.errorHandler.Handle(ctx, err)
		s.errorEncoder(ctx, err, w)
		return
	}

	for _, f := range s.after {
		ctx = f(ctx, w)
	}

	if err := s.enc(ctx, w, response); err != nil {
		s.errorHandler.Handle(ctx, err)
		s.errorEncoder(ctx, err, w)
		return
	}
}

// ErrorEncoder is responsible for encoding an error to the ResponseWriter.
// Users are encouraged to use custom ErrorEncoders to encode HTTP errors to
// their clients, and will likely want to pass and check for their own error
// types. See the example shipping/handling service.
type ErrorEncoder func(ctx context.Context, err error, w http.ResponseWriter)

// ServerFinalizerFunc can be used to perform work at the end of an HTTP
// request, after the response has been written to the client. The principal
// intended use is for request logging. In addition to the response code
// provided in the function signature, additional response parameters are
// provided in the context under keys with the ContextKeyResponse prefix.
type ServerFinalizerFunc func(ctx context.Context, code int, r *http.Request)

// NopRequestDecoder is a DecodeRequestFunc that can be used for requests that do not
// need to be decoded, and simply returns nil, nil.
func NopRequestDecoder(ctx context.Context, r *http.Request) (interface{}, error) {
	return nil, nil
}

// EncodeJSONResponse is a EncodeResponseFunc that serializes the response as a
// JSON object to the ResponseWriter. Many JSON-over-HTTP services can use it as
// a sensible default. If the response implements Headerer, the provided headers
// will be applied to the response. If the response implements StatusCoder, the
// provided StatusCode will be used instead of 200.
func EncodeJSONResponse(_ context.Context, w http.ResponseWriter, response interface{}) error {
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	if headerer, ok := response.(Headerer); ok {
		for k, values := range headerer.Headers() {
			for _, v := range values {
				w.Header().Add(k, v)
			}
		}
	}
	code := http.StatusOK
	if sc, ok := response.(StatusCoder); ok {
		code = sc.StatusCode()
	}
	w.WriteHeader(code)
	if code == http.StatusNoContent {
		return nil
	}
	return json.NewEncoder(w).Encode(response)
}

// DefaultErrorEncoder writes the error to the ResponseWriter, by default a
// content type of text/plain, a body of the plain text of the error, and a
// status code of 500. If the error implements Headerer, the provided headers
// will be applied to the response. If the error implements json.Marshaler, and
// the marshaling succeeds, a content type of application/json and the JSON
// encoded form of the error will be used. If the error implements StatusCoder,
// the provided StatusCode will be used instead of 500.
func DefaultErrorEncoder(_ context.Context, err error, w http.ResponseWriter) {
	contentType, body := "text/plain; charset=utf-8", []byte(err.Error())
	if marshaler, ok := err.(json.Marshaler); ok {
		if jsonBody, marshalErr := marshaler.MarshalJSON(); marshalErr == nil {
			contentType, body = "application/json; charset=utf-8", jsonBody
		}
	}
	w.Header().Set("Content-Type", contentType)
	if headerer, ok := err.(Headerer); ok {
		for k, values := range headerer.Headers() {
			for _, v := range values {
				w.Header().Add(k, v)
			}
		}
	}
	code := http.StatusInternalServerError
	if sc, ok := err.(StatusCoder); ok {
		code = sc.StatusCode()
	}
	w.WriteHeader(code)
	w.Write(body)
}

// StatusCoder is checked by DefaultErrorEncoder. If an error value implements
// StatusCoder, the StatusCode will be used when encoding the error. By default,
// StatusInternalServerError (500) is used.
type StatusCoder interface {
	StatusCode() int
}

// Headerer is checked by DefaultErrorEncoder. If an error value implements
// Headerer, the provided headers will be applied to the response writer, after
// the Content-Type is set.
type Headerer interface {
	Headers() http.Header
}

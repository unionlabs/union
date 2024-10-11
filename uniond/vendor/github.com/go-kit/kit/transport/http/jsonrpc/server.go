package jsonrpc

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"

	httptransport "github.com/go-kit/kit/transport/http"
	"github.com/go-kit/log"
)

type requestIDKeyType struct{}

var requestIDKey requestIDKeyType

// Server wraps an endpoint and implements http.Handler.
type Server struct {
	ecm          EndpointCodecMap
	before       []httptransport.RequestFunc
	beforeCodec  []RequestFunc
	after        []httptransport.ServerResponseFunc
	errorEncoder httptransport.ErrorEncoder
	finalizer    httptransport.ServerFinalizerFunc
	logger       log.Logger
}

// NewServer constructs a new server, which implements http.Server.
func NewServer(
	ecm EndpointCodecMap,
	options ...ServerOption,
) *Server {
	s := &Server{
		ecm:          ecm,
		errorEncoder: DefaultErrorEncoder,
		logger:       log.NewNopLogger(),
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
func ServerBefore(before ...httptransport.RequestFunc) ServerOption {
	return func(s *Server) { s.before = append(s.before, before...) }
}

// ServerBeforeCodec functions are executed after the JSON request body has been
// decoded, but before the method's decoder is called. This provides an opportunity
// for middleware to inspect the contents of the rpc request before being passed
// to the codec.
func ServerBeforeCodec(beforeCodec ...RequestFunc) ServerOption {
	return func(s *Server) { s.beforeCodec = append(s.beforeCodec, beforeCodec...) }
}

// ServerAfter functions are executed on the HTTP response writer after the
// endpoint is invoked, but before anything is written to the client.
func ServerAfter(after ...httptransport.ServerResponseFunc) ServerOption {
	return func(s *Server) { s.after = append(s.after, after...) }
}

// ServerErrorEncoder is used to encode errors to the http.ResponseWriter
// whenever they're encountered in the processing of a request. Clients can
// use this to provide custom error formatting and response codes. By default,
// errors will be written with the DefaultErrorEncoder.
func ServerErrorEncoder(ee httptransport.ErrorEncoder) ServerOption {
	return func(s *Server) { s.errorEncoder = ee }
}

// ServerErrorLogger is used to log non-terminal errors. By default, no errors
// are logged. This is intended as a diagnostic measure. Finer-grained control
// of error handling, including logging in more detail, should be performed in a
// custom ServerErrorEncoder or ServerFinalizer, both of which have access to
// the context.
func ServerErrorLogger(logger log.Logger) ServerOption {
	return func(s *Server) { s.logger = logger }
}

// ServerFinalizer is executed at the end of every HTTP request.
// By default, no finalizer is registered.
func ServerFinalizer(f httptransport.ServerFinalizerFunc) ServerOption {
	return func(s *Server) { s.finalizer = f }
}

// ServeHTTP implements http.Handler.
func (s Server) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		w.Header().Set("Content-Type", "text/plain; charset=utf-8")
		w.WriteHeader(http.StatusMethodNotAllowed)
		_, _ = io.WriteString(w, "405 must POST\n")
		return
	}
	ctx := r.Context()

	if s.finalizer != nil {
		iw := &interceptingWriter{w, http.StatusOK}
		defer func() { s.finalizer(ctx, iw.code, r) }()
		w = iw
	}

	for _, f := range s.before {
		ctx = f(ctx, r)
	}

	// Decode the body into an  object
	var req Request
	err := json.NewDecoder(r.Body).Decode(&req)
	if err != nil {
		rpcerr := parseError("JSON could not be decoded: " + err.Error())
		s.logger.Log("err", rpcerr)
		s.errorEncoder(ctx, rpcerr, w)
		return
	}

	ctx = context.WithValue(ctx, requestIDKey, req.ID)
	ctx = context.WithValue(ctx, ContextKeyRequestMethod, req.Method)

	for _, f := range s.beforeCodec {
		ctx = f(ctx, r, req)
	}

	// Get the endpoint and codecs from the map using the method
	// defined in the JSON  object
	ecm, ok := s.ecm[req.Method]
	if !ok {
		err := methodNotFoundError(fmt.Sprintf("Method %s was not found.", req.Method))
		s.logger.Log("err", err)
		s.errorEncoder(ctx, err, w)
		return
	}

	// Decode the JSON "params"
	reqParams, err := ecm.Decode(ctx, req.Params)
	if err != nil {
		s.logger.Log("err", err)
		s.errorEncoder(ctx, err, w)
		return
	}

	// Call the Endpoint with the params
	response, err := ecm.Endpoint(ctx, reqParams)
	if err != nil {
		s.logger.Log("err", err)
		s.errorEncoder(ctx, err, w)
		return
	}

	for _, f := range s.after {
		ctx = f(ctx, w)
	}

	res := Response{
		ID:      req.ID,
		JSONRPC: Version,
	}

	// Encode the response from the Endpoint
	resParams, err := ecm.Encode(ctx, response)
	if err != nil {
		s.logger.Log("err", err)
		s.errorEncoder(ctx, err, w)
		return
	}

	res.Result = resParams

	w.Header().Set("Content-Type", ContentType)
	_ = json.NewEncoder(w).Encode(res)
}

// DefaultErrorEncoder writes the error to the ResponseWriter,
// as a json-rpc error response, with an InternalError status code.
// The Error() string of the error will be used as the response error message.
// If the error implements ErrorCoder, the provided code will be set on the
// response error.
// If the error implements Headerer, the given headers will be set.
func DefaultErrorEncoder(ctx context.Context, err error, w http.ResponseWriter) {
	w.Header().Set("Content-Type", ContentType)
	if headerer, ok := err.(httptransport.Headerer); ok {
		for k := range headerer.Headers() {
			w.Header().Set(k, headerer.Headers().Get(k))
		}
	}

	e := Error{
		Code:    InternalError,
		Message: err.Error(),
	}
	if sc, ok := err.(ErrorCoder); ok {
		e.Code = sc.ErrorCode()
	}

	w.WriteHeader(http.StatusOK)

	var requestID *RequestID
	if v := ctx.Value(requestIDKey); v != nil {
		requestID = v.(*RequestID)
	}
	_ = json.NewEncoder(w).Encode(Response{
		ID:      requestID,
		JSONRPC: Version,
		Error:   &e,
	})
}

// ErrorCoder is checked by DefaultErrorEncoder. If an error value implements
// ErrorCoder, the integer result of ErrorCode() will be used as the JSONRPC
// error code when encoding the error.
//
// By default, InternalError (-32603) is used.
type ErrorCoder interface {
	ErrorCode() int
}

// interceptingWriter intercepts calls to WriteHeader, so that a finalizer
// can be given the correct status code.
type interceptingWriter struct {
	http.ResponseWriter
	code int
}

// WriteHeader may not be explicitly called, so care must be taken to
// initialize w.code to its default value of http.StatusOK.
func (w *interceptingWriter) WriteHeader(code int) {
	w.code = code
	w.ResponseWriter.WriteHeader(code)
}

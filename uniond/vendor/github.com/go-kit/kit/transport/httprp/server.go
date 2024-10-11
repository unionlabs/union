package httprp

import (
	"context"
	"net/http"
	"net/http/httputil"
	"net/url"
)

// RequestFunc may take information from an HTTP request and put it into a
// request context. BeforeFuncs are executed prior to invoking the
// endpoint.
type RequestFunc func(context.Context, *http.Request) context.Context

// Server is a proxying request handler.
type Server struct {
	proxy        http.Handler
	before       []RequestFunc
	errorEncoder func(w http.ResponseWriter, err error)
}

// NewServer constructs a new server that implements http.Server and will proxy
// requests to the given base URL using its scheme, host, and base path.
// If the target's path is "/base" and the incoming request was for "/dir",
// the target request will be for /base/dir.
func NewServer(
	baseURL *url.URL,
	options ...ServerOption,
) *Server {
	s := &Server{
		proxy: httputil.NewSingleHostReverseProxy(baseURL),
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

// ServeHTTP implements http.Handler.
func (s Server) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()

	for _, f := range s.before {
		ctx = f(ctx, r)
	}

	s.proxy.ServeHTTP(w, r)
}

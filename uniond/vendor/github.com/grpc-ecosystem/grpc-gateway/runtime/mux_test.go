package runtime_test

import (
	"bytes"
	"context"
	"fmt"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/grpc-ecosystem/grpc-gateway/runtime"
	"github.com/grpc-ecosystem/grpc-gateway/utilities"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

func TestMuxServeHTTP(t *testing.T) {
	type stubPattern struct {
		method string
		ops    []int
		pool   []string
		verb   string
	}
	for _, spec := range []struct {
		patterns    []stubPattern
		patternOpts []runtime.PatternOpt

		reqMethod string
		reqPath   string
		headers   map[string]string

		respStatus  int
		respContent string

		disablePathLengthFallback bool
		errHandler                runtime.ProtoErrorHandlerFunc
		muxOpts                   []runtime.ServeMuxOption
	}{
		{
			patterns:   nil,
			reqMethod:  "GET",
			reqPath:    "/",
			respStatus: http.StatusNotFound,
		},
		{
			patterns: []stubPattern{
				{
					method: "GET",
					ops:    []int{int(utilities.OpLitPush), 0},
					pool:   []string{"foo"},
				},
			},
			reqMethod:   "GET",
			reqPath:     "/foo",
			respStatus:  http.StatusOK,
			respContent: "GET /foo",
		},
		{
			patterns: []stubPattern{
				{
					method: "GET",
					ops:    []int{int(utilities.OpLitPush), 0},
					pool:   []string{"foo"},
				},
			},
			reqMethod:  "GET",
			reqPath:    "/bar",
			respStatus: http.StatusNotFound,
		},
		{
			patterns: []stubPattern{
				{
					method: "GET",
					ops:    []int{int(utilities.OpLitPush), 0},
					pool:   []string{"foo"},
				},
				{
					method: "GET",
					ops:    []int{int(utilities.OpPush), 0},
				},
			},
			reqMethod:   "GET",
			reqPath:     "/foo",
			respStatus:  http.StatusOK,
			respContent: "GET /foo",
		},
		{
			patterns: []stubPattern{
				{
					method: "GET",
					ops:    []int{int(utilities.OpLitPush), 0},
					pool:   []string{"foo"},
				},
				{
					method: "POST",
					ops:    []int{int(utilities.OpLitPush), 0},
					pool:   []string{"foo"},
				},
			},
			reqMethod:   "POST",
			reqPath:     "/foo",
			respStatus:  http.StatusOK,
			respContent: "POST /foo",
		},
		{
			patterns: []stubPattern{
				{
					method: "GET",
					ops:    []int{int(utilities.OpLitPush), 0},
					pool:   []string{"foo"},
				},
			},
			reqMethod:  "DELETE",
			reqPath:    "/foo",
			respStatus: http.StatusMethodNotAllowed,
		},
		{
			patterns: []stubPattern{
				{
					method: "GET",
					ops:    []int{int(utilities.OpLitPush), 0},
					pool:   []string{"foo"},
				},
			},
			reqMethod: "POST",
			reqPath:   "/foo",
			headers: map[string]string{
				"Content-Type": "application/x-www-form-urlencoded",
			},
			respStatus:  http.StatusOK,
			respContent: "GET /foo",
		},
		{
			patterns: []stubPattern{
				{
					method: "GET",
					ops:    []int{int(utilities.OpLitPush), 0},
					pool:   []string{"foo"},
				},
			},
			reqMethod: "POST",
			reqPath:   "/foo",
			headers: map[string]string{
				"Content-Type": "application/x-www-form-urlencoded",
			},
			respStatus:                http.StatusMethodNotAllowed,
			respContent:               "Method Not Allowed\n",
			disablePathLengthFallback: true,
		},
		{
			patterns: []stubPattern{
				{
					method: "GET",
					ops:    []int{int(utilities.OpLitPush), 0},
					pool:   []string{"foo"},
				},
				{
					method: "POST",
					ops:    []int{int(utilities.OpLitPush), 0},
					pool:   []string{"foo"},
				},
			},
			reqMethod: "POST",
			reqPath:   "/foo",
			headers: map[string]string{
				"Content-Type": "application/x-www-form-urlencoded",
			},
			respStatus:                http.StatusOK,
			respContent:               "POST /foo",
			disablePathLengthFallback: true,
		},
		{
			patterns: []stubPattern{
				{
					method: "GET",
					ops:    []int{int(utilities.OpLitPush), 0},
					pool:   []string{"foo"},
				},
				{
					method: "POST",
					ops:    []int{int(utilities.OpLitPush), 0},
					pool:   []string{"foo"},
				},
			},
			reqMethod: "POST",
			reqPath:   "/foo",
			headers: map[string]string{
				"Content-Type":           "application/x-www-form-urlencoded",
				"X-HTTP-Method-Override": "GET",
			},
			respStatus:  http.StatusOK,
			respContent: "GET /foo",
		},
		{
			patterns: []stubPattern{
				{
					method: "GET",
					ops:    []int{int(utilities.OpLitPush), 0},
					pool:   []string{"foo"},
				},
			},
			reqMethod: "POST",
			reqPath:   "/foo",
			headers: map[string]string{
				"Content-Type": "application/json",
			},
			respStatus: http.StatusMethodNotAllowed,
		},
		{
			patterns: []stubPattern{
				{
					method: "POST",
					ops:    []int{int(utilities.OpLitPush), 0},
					pool:   []string{"foo"},
					verb:   "bar",
				},
			},
			reqMethod: "POST",
			reqPath:   "/foo:bar",
			headers: map[string]string{
				"Content-Type": "application/json",
			},
			respStatus:  http.StatusOK,
			respContent: "POST /foo:bar",
		},
		{
			patterns: []stubPattern{
				{
					method: "GET",
					ops:    []int{int(utilities.OpLitPush), 0, int(utilities.OpPush), 0, int(utilities.OpConcatN), 1, int(utilities.OpCapture), 1},
					pool:   []string{"foo", "id"},
				},
				{
					method: "GET",
					ops:    []int{int(utilities.OpLitPush), 0, int(utilities.OpPush), 0, int(utilities.OpConcatN), 1, int(utilities.OpCapture), 1},
					pool:   []string{"foo", "id"},
					verb:   "verb",
				},
			},
			reqMethod: "GET",
			reqPath:   "/foo/bar:verb",
			headers: map[string]string{
				"Content-Type": "application/json",
			},
			respStatus:  http.StatusOK,
			respContent: "GET /foo/{id=*}:verb",
		},
		{
			// mux identifying invalid path results in 'Not Found' status
			// (with custom handler looking for ErrUnknownURI)
			patterns: []stubPattern{
				{
					method: "GET",
					ops:    []int{int(utilities.OpLitPush), 0},
					pool:   []string{"unimplemented"},
				},
			},
			reqMethod:   "GET",
			reqPath:     "/foobar",
			respStatus:  http.StatusNotFound,
			respContent: "GET /foobar",
			errHandler:  unknownPathIs404,
		},
		{
			// server returning unimplemented results in 'Not Implemented' code
			// even when using custom error handler
			patterns: []stubPattern{
				{
					method: "GET",
					ops:    []int{int(utilities.OpLitPush), 0},
					pool:   []string{"unimplemented"},
				},
			},
			reqMethod:   "GET",
			reqPath:     "/unimplemented",
			respStatus:  http.StatusNotImplemented,
			respContent: `GET /unimplemented`,
			errHandler:  unknownPathIs404,
		},
		{
			patterns: []stubPattern{
				{
					method: "GET",
					ops:    []int{int(utilities.OpLitPush), 0, int(utilities.OpPush), 0, int(utilities.OpConcatN), 1, int(utilities.OpCapture), 1},
					pool:   []string{"foo", "id"},
				},
			},
			patternOpts: []runtime.PatternOpt{runtime.AssumeColonVerbOpt(false)},
			reqMethod:   "GET",
			reqPath:     "/foo/bar",
			headers: map[string]string{
				"Content-Type": "application/json",
			},
			respStatus:  http.StatusOK,
			respContent: "GET /foo/{id=*}",
		},
		{
			patterns: []stubPattern{
				{
					method: "GET",
					ops:    []int{int(utilities.OpLitPush), 0, int(utilities.OpPush), 0, int(utilities.OpConcatN), 1, int(utilities.OpCapture), 1},
					pool:   []string{"foo", "id"},
				},
			},
			patternOpts: []runtime.PatternOpt{runtime.AssumeColonVerbOpt(false)},
			reqMethod:   "GET",
			reqPath:     "/foo/bar:123",
			headers: map[string]string{
				"Content-Type": "application/json",
			},
			respStatus:  http.StatusOK,
			respContent: "GET /foo/{id=*}",
		},
		{
			patterns: []stubPattern{
				{
					method: "POST",
					ops:    []int{int(utilities.OpLitPush), 0, int(utilities.OpPush), 0, int(utilities.OpConcatN), 1, int(utilities.OpCapture), 1},
					pool:   []string{"foo", "id"},
				},
				{
					method: "POST",
					ops:    []int{int(utilities.OpLitPush), 0, int(utilities.OpPush), 0, int(utilities.OpConcatN), 1, int(utilities.OpCapture), 1},
					pool:   []string{"foo", "id"},
					verb:   "verb",
				},
			},
			patternOpts: []runtime.PatternOpt{runtime.AssumeColonVerbOpt(false)},
			reqMethod:   "POST",
			reqPath:     "/foo/bar:verb",
			headers: map[string]string{
				"Content-Type": "application/json",
			},
			respStatus:  http.StatusOK,
			respContent: "POST /foo/{id=*}:verb",
			muxOpts:     []runtime.ServeMuxOption{runtime.WithLastMatchWins()},
		},
	} {
		opts := spec.muxOpts
		if spec.disablePathLengthFallback {
			opts = append(opts, runtime.WithDisablePathLengthFallback())
		}
		if spec.errHandler != nil {
			opts = append(opts, runtime.WithProtoErrorHandler(spec.errHandler))
		}
		mux := runtime.NewServeMux(opts...)
		for _, p := range spec.patterns {
			func(p stubPattern) {
				pat, err := runtime.NewPattern(1, p.ops, p.pool, p.verb, spec.patternOpts...)
				if err != nil {
					t.Fatalf("runtime.NewPattern(1, %#v, %#v, %q) failed with %v; want success", p.ops, p.pool, p.verb, err)
				}
				mux.Handle(p.method, pat, func(w http.ResponseWriter, r *http.Request, pathParams map[string]string) {
					if r.URL.Path == "/unimplemented" {
						// simulate method returning "unimplemented" error
						_, m := runtime.MarshalerForRequest(mux, r)
						runtime.HTTPError(r.Context(), mux, m, w, r, status.Error(codes.Unimplemented, http.StatusText(http.StatusNotImplemented)))
						w.WriteHeader(http.StatusNotImplemented)
						return
					}
					fmt.Fprintf(w, "%s %s", p.method, pat.String())
				})
			}(p)
		}

		url := fmt.Sprintf("http://host.example%s", spec.reqPath)
		r, err := http.NewRequest(spec.reqMethod, url, bytes.NewReader(nil))
		if err != nil {
			t.Fatalf("http.NewRequest(%q, %q, nil) failed with %v; want success", spec.reqMethod, url, err)
		}
		for name, value := range spec.headers {
			r.Header.Set(name, value)
		}
		w := httptest.NewRecorder()
		mux.ServeHTTP(w, r)

		if got, want := w.Code, spec.respStatus; got != want {
			t.Errorf("w.Code = %d; want %d; patterns=%v; req=%v", got, want, spec.patterns, r)
		}
		if spec.respContent != "" {
			if got, want := w.Body.String(), spec.respContent; got != want {
				t.Errorf("w.Body = %q; want %q; patterns=%v; req=%v", got, want, spec.patterns, r)
			}
		}
	}
}

func unknownPathIs404(ctx context.Context, mux *runtime.ServeMux, m runtime.Marshaler, w http.ResponseWriter, r *http.Request, err error) {
	if err == runtime.ErrUnknownURI {
		w.WriteHeader(http.StatusNotFound)
	} else {
		c := status.Convert(err).Code()
		w.WriteHeader(runtime.HTTPStatusFromCode(c))
	}

	fmt.Fprintf(w, "%s %s", r.Method, r.URL.Path)
}

var defaultHeaderMatcherTests = []struct {
	name     string
	in       string
	outValue string
	outValid bool
}{
	{
		"permanent HTTP header should return prefixed",
		"Accept",
		"grpcgateway-Accept",
		true,
	},
	{
		"key prefixed with MetadataHeaderPrefix should return without the prefix",
		"Grpc-Metadata-Custom-Header",
		"Custom-Header",
		true,
	},
	{
		"non-permanent HTTP header key without prefix should not return",
		"Custom-Header",
		"",
		false,
	},
}

func TestDefaultHeaderMatcher(t *testing.T) {
	for _, tt := range defaultHeaderMatcherTests {
		t.Run(tt.name, func(t *testing.T) {
			out, valid := runtime.DefaultHeaderMatcher(tt.in)
			if out != tt.outValue {
				t.Errorf("got %v, want %v", out, tt.outValue)
			}
			if valid != tt.outValid {
				t.Errorf("got %v, want %v", valid, tt.outValid)
			}
		})
	}
}

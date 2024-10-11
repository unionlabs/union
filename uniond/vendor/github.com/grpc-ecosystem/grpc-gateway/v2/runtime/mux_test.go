package runtime_test

import (
	"bytes"
	"context"
	"fmt"
	"net/http"
	"net/http/httptest"
	"net/url"
	"strconv"
	"strings"
	"testing"

	"github.com/grpc-ecosystem/grpc-gateway/v2/runtime"
	"github.com/grpc-ecosystem/grpc-gateway/v2/utilities"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/health/grpc_health_v1"
	"google.golang.org/grpc/status"
)

func TestMuxServeHTTP(t *testing.T) {
	type stubPattern struct {
		method string
		ops    []int
		pool   []string
		verb   string
	}
	for i, spec := range []struct {
		patterns []stubPattern

		reqMethod string
		reqPath   string
		headers   map[string]string

		respStatus  int
		respContent string

		disablePathLengthFallback bool
		unescapingMode            runtime.UnescapingMode
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
					ops:    []int{int(utilities.OpPush), 0},
				},
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
			respStatus: http.StatusNotImplemented,
		},
		{
			patterns: []stubPattern{
				{
					method: "POST",
					ops:    []int{int(utilities.OpLitPush), 0, int(utilities.OpPush), 0, int(utilities.OpConcatN), 1, int(utilities.OpCapture), 1},
					pool:   []string{"foo", "id"},
					verb:   "archive",
				},
			},
			reqMethod:  "DELETE",
			reqPath:    "/foo/bar:archive",
			respStatus: http.StatusNotImplemented,
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
			respStatus:                http.StatusNotImplemented,
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
			respStatus: http.StatusNotImplemented,
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
			patterns: []stubPattern{
				{
					method: "GET",
					ops:    []int{int(utilities.OpLitPush), 0, int(utilities.OpPush), 0, int(utilities.OpConcatN), 1, int(utilities.OpCapture), 1},
					pool:   []string{"foo", "id"},
				},
			},
			reqMethod: "GET",
			reqPath:   "/foo/bar",
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
			reqMethod: "GET",
			reqPath:   "/foo/bar:123",
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
			reqMethod: "POST",
			reqPath:   "/foo/bar:verb",
			headers: map[string]string{
				"Content-Type": "application/json",
			},
			respStatus:  http.StatusOK,
			respContent: "POST /foo/{id=*}:verb",
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
			reqPath:   "foo",
			headers: map[string]string{
				"Content-Type": "application/json",
			},
			respStatus: http.StatusBadRequest,
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
					verb:   "verb:subverb",
				},
			},
			reqMethod: "POST",
			reqPath:   "/foo/bar:verb:subverb",
			headers: map[string]string{
				"Content-Type": "application/json",
			},
			respStatus:  http.StatusOK,
			respContent: "POST /foo/{id=*}:verb:subverb",
		},
		{
			patterns: []stubPattern{
				{
					method: "GET",
					ops:    []int{int(utilities.OpLitPush), 0, int(utilities.OpPush), 1, int(utilities.OpCapture), 1, int(utilities.OpLitPush), 2},
					pool:   []string{"foo", "id", "bar"},
				},
			},
			reqMethod: "POST",
			reqPath:   "/foo/404%2fwith%2Fspace/bar",
			headers: map[string]string{
				"Content-Type": "application/json",
			},
			respStatus:     http.StatusNotFound,
			unescapingMode: runtime.UnescapingModeLegacy,
		},
		{
			patterns: []stubPattern{
				{
					method: "GET",
					ops: []int{
						int(utilities.OpLitPush), 0,
						int(utilities.OpPush), 0,
						int(utilities.OpConcatN), 1,
						int(utilities.OpCapture), 1,
						int(utilities.OpLitPush), 2},
					pool: []string{"foo", "id", "bar"},
				},
			},
			reqMethod: "GET",
			reqPath:   "/foo/success%2fwith%2Fspace/bar",
			headers: map[string]string{
				"Content-Type": "application/json",
			},
			respStatus:     http.StatusOK,
			unescapingMode: runtime.UnescapingModeAllExceptReserved,
			respContent:    "GET /foo/{id=*}/bar",
		},
		{
			patterns: []stubPattern{
				{
					method: "GET",
					ops: []int{
						int(utilities.OpLitPush), 0,
						int(utilities.OpPush), 0,
						int(utilities.OpConcatN), 1,
						int(utilities.OpCapture), 1,
						int(utilities.OpLitPush), 2},
					pool: []string{"foo", "id", "bar"},
				},
			},
			reqMethod: "GET",
			reqPath:   "/foo/success%2fwith%2Fspace/bar",
			headers: map[string]string{
				"Content-Type": "application/json",
			},
			respStatus:     http.StatusNotFound,
			unescapingMode: runtime.UnescapingModeAllCharacters,
		},
		{
			patterns: []stubPattern{
				{
					method: "GET",
					ops: []int{
						int(utilities.OpLitPush), 0,
						int(utilities.OpPush), 0,
						int(utilities.OpConcatN), 1,
						int(utilities.OpCapture), 1,
						int(utilities.OpLitPush), 2},
					pool: []string{"foo", "id", "bar"},
				},
			},
			reqMethod: "GET",
			reqPath:   "/foo/success%2fwith%2Fspace/bar",
			headers: map[string]string{
				"Content-Type": "application/json",
			},
			respStatus:     http.StatusNotFound,
			unescapingMode: runtime.UnescapingModeLegacy,
		},
		{
			patterns: []stubPattern{
				{
					method: "GET",
					ops: []int{
						int(utilities.OpLitPush), 0,
						int(utilities.OpPushM), 0,
						int(utilities.OpConcatN), 1,
						int(utilities.OpCapture), 1,
					},
					pool: []string{"foo", "id", "bar"},
				},
			},
			reqMethod: "GET",
			reqPath:   "/foo/success%2fwith%2Fspace",
			headers: map[string]string{
				"Content-Type": "application/json",
			},
			respStatus:     http.StatusOK,
			unescapingMode: runtime.UnescapingModeAllExceptReserved,
			respContent:    "GET /foo/{id=**}",
		},
		{
			patterns: []stubPattern{
				{
					method: "POST",
					ops: []int{
						int(utilities.OpLitPush), 0,
						int(utilities.OpLitPush), 1,
						int(utilities.OpLitPush), 2,
						int(utilities.OpPush), 0,
						int(utilities.OpConcatN), 2,
						int(utilities.OpCapture), 3,
					},
					pool: []string{"api", "v1", "organizations", "name"},
					verb: "action",
				},
			},
			reqMethod: "POST",
			reqPath:   "/api/v1/" + url.QueryEscape("organizations/foo") + ":action",
			headers: map[string]string{
				"Content-Type": "application/json",
			},
			respStatus:     http.StatusOK,
			unescapingMode: runtime.UnescapingModeAllCharacters,
			respContent:    "POST /api/v1/{name=organizations/*}:action",
		},
		{
			patterns: []stubPattern{
				{
					method: "POST",
					ops: []int{
						int(utilities.OpLitPush), 0,
						int(utilities.OpLitPush), 1,
						int(utilities.OpLitPush), 2,
					},
					pool: []string{"api", "v1", "organizations"},
					verb: "verb",
				},
				{
					method: "POST",
					ops: []int{
						int(utilities.OpLitPush), 0,
						int(utilities.OpLitPush), 1,
						int(utilities.OpLitPush), 2,
					},
					pool: []string{"api", "v1", "organizations"},
					verb: "",
				},
				{
					method: "POST",
					ops: []int{
						int(utilities.OpLitPush), 0,
						int(utilities.OpLitPush), 1,
						int(utilities.OpLitPush), 2,
					},
					pool: []string{"api", "v1", "dummies"},
					verb: "verb",
				},
			},
			reqMethod: "POST",
			reqPath:   "/api/v1/organizations:verb",
			headers: map[string]string{
				"Content-Type": "application/json",
			},
			respStatus:     http.StatusOK,
			unescapingMode: runtime.UnescapingModeAllCharacters,
			respContent:    "POST /api/v1/organizations:verb",
		},
	} {
		t.Run(strconv.Itoa(i), func(t *testing.T) {
			var opts []runtime.ServeMuxOption
			opts = append(opts, runtime.WithUnescapingMode(spec.unescapingMode))
			if spec.disablePathLengthFallback {
				opts = append(opts,
					runtime.WithDisablePathLengthFallback(),
				)
			}
			mux := runtime.NewServeMux(opts...)
			for _, p := range spec.patterns {
				func(p stubPattern) {
					pat, err := runtime.NewPattern(1, p.ops, p.pool, p.verb)
					if err != nil {
						t.Fatalf("runtime.NewPattern(1, %#v, %#v, %q) failed with %v; want success", p.ops, p.pool, p.verb, err)
					}
					mux.Handle(p.method, pat, func(w http.ResponseWriter, r *http.Request, pathParams map[string]string) {
						_, _ = fmt.Fprintf(w, "%s %s", p.method, pat.String())
					})
				}(p)
			}

			reqUrl := fmt.Sprintf("https://host.example%s", spec.reqPath)
			r, err := http.NewRequest(spec.reqMethod, reqUrl, bytes.NewReader(nil))
			if err != nil {
				t.Fatalf("http.NewRequest(%q, %q, nil) failed with %v; want success", spec.reqMethod, reqUrl, err)
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
		})
	}
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

var defaultRouteMatcherTests = []struct {
	name   string
	method string
	path   string
	valid  bool
}{
	{
		"Test route /",
		"GET",
		"/",
		true,
	},
	{
		"Simple Endpoint",
		"GET",
		"/v1/{bucket}/do:action",
		true,
	},
	{
		"Complex Endpoint",
		"POST",
		"/v1/b/{bucket_name=buckets/*}/o/{name}",
		true,
	},
	{
		"Wildcard Endpoint",
		"GET",
		"/v1/endpoint/*",
		true,
	},
	{
		"Invalid Endpoint",
		"POST",
		"v1/b/:name/do",
		false,
	},
}

func TestServeMux_HandlePath(t *testing.T) {
	mux := runtime.NewServeMux()
	testFn := func(w http.ResponseWriter, r *http.Request, pathParams map[string]string) {
	}
	for _, tt := range defaultRouteMatcherTests {
		t.Run(tt.name, func(t *testing.T) {
			err := mux.HandlePath(tt.method, tt.path, testFn)
			if tt.valid && err != nil {
				t.Errorf("The route %v with method %v and path %v invalid, got %v", tt.name, tt.method, tt.path, err)
			}
			if !tt.valid && err == nil {
				t.Errorf("The route %v with method %v and path %v should be invalid", tt.name, tt.method, tt.path)
			}
		})
	}
}

var healthCheckTests = []struct {
	name           string
	code           codes.Code
	status         grpc_health_v1.HealthCheckResponse_ServingStatus
	httpStatusCode int
}{
	{
		"Test grpc error code",
		codes.NotFound,
		grpc_health_v1.HealthCheckResponse_UNKNOWN,
		http.StatusNotFound,
	},
	{
		"Test HealthCheckResponse_SERVING",
		codes.OK,
		grpc_health_v1.HealthCheckResponse_SERVING,
		http.StatusOK,
	},
	{
		"Test HealthCheckResponse_NOT_SERVING",
		codes.OK,
		grpc_health_v1.HealthCheckResponse_NOT_SERVING,
		http.StatusServiceUnavailable,
	},
	{
		"Test HealthCheckResponse_UNKNOWN",
		codes.OK,
		grpc_health_v1.HealthCheckResponse_UNKNOWN,
		http.StatusServiceUnavailable,
	},
	{
		"Test HealthCheckResponse_SERVICE_UNKNOWN",
		codes.OK,
		grpc_health_v1.HealthCheckResponse_SERVICE_UNKNOWN,
		http.StatusNotFound,
	},
}

func TestWithHealthzEndpoint_codes(t *testing.T) {
	for _, tt := range healthCheckTests {
		t.Run(tt.name, func(t *testing.T) {
			mux := runtime.NewServeMux(runtime.WithHealthzEndpoint(&dummyHealthCheckClient{status: tt.status, code: tt.code}))

			r := httptest.NewRequest(http.MethodGet, "/healthz", nil)
			rr := httptest.NewRecorder()

			mux.ServeHTTP(rr, r)

			if rr.Code != tt.httpStatusCode {
				t.Errorf(
					"result http status code for grpc code %q and status %q should be %d, got %d",
					tt.code, tt.status, tt.httpStatusCode, rr.Code,
				)
			}
		})
	}
}

func TestWithHealthEndpointAt_consistentWithHealthz(t *testing.T) {
	const endpointPath = "/healthz"

	r := httptest.NewRequest(http.MethodGet, endpointPath, nil)

	for _, tt := range healthCheckTests {
		tt := tt

		t.Run(tt.name, func(t *testing.T) {
			client := &dummyHealthCheckClient{
				status: tt.status,
				code:   tt.code,
			}

			w := httptest.NewRecorder()

			runtime.NewServeMux(
				runtime.WithHealthEndpointAt(client, endpointPath),
			).ServeHTTP(w, r)

			refW := httptest.NewRecorder()

			runtime.NewServeMux(
				runtime.WithHealthzEndpoint(client),
			).ServeHTTP(refW, r)

			if w.Code != refW.Code {
				t.Errorf(
					"result http status code for grpc code %q and status %q should be equal to %d, but got %d",
					tt.code, tt.status, refW.Code, w.Code,
				)
			}
		})
	}
}

func TestWithHealthzEndpoint_serviceParam(t *testing.T) {
	service := "test"

	// trigger error to output service in body
	dummyClient := dummyHealthCheckClient{status: grpc_health_v1.HealthCheckResponse_UNKNOWN, code: codes.Unknown}
	mux := runtime.NewServeMux(runtime.WithHealthzEndpoint(&dummyClient))

	r := httptest.NewRequest(http.MethodGet, "/healthz?service="+service, nil)
	rr := httptest.NewRecorder()

	mux.ServeHTTP(rr, r)

	if !strings.Contains(rr.Body.String(), service) {
		t.Errorf(
			"service query parameter should be translated to HealthCheckRequest: expected %s to contain %s",
			rr.Body.String(), service,
		)
	}
}

func TestWithHealthzEndpoint_header(t *testing.T) {
	for _, tt := range healthCheckTests {
		t.Run(tt.name, func(t *testing.T) {
			mux := runtime.NewServeMux(runtime.WithHealthzEndpoint(&dummyHealthCheckClient{status: tt.status, code: tt.code}))

			r := httptest.NewRequest(http.MethodGet, "/healthz", nil)
			rr := httptest.NewRecorder()

			mux.ServeHTTP(rr, r)

			if actualHeader := rr.Header().Get("Content-Type"); actualHeader != "application/json" {
				t.Errorf(
					"result http header Content-Type for grpc code %q and status %q should be application/json, got %s",
					tt.code, tt.status, actualHeader,
				)
			}
		})
	}
}

var _ grpc_health_v1.HealthClient = (*dummyHealthCheckClient)(nil)

type dummyHealthCheckClient struct {
	status grpc_health_v1.HealthCheckResponse_ServingStatus
	code   codes.Code
}

func (g *dummyHealthCheckClient) Check(ctx context.Context, r *grpc_health_v1.HealthCheckRequest, opts ...grpc.CallOption) (*grpc_health_v1.HealthCheckResponse, error) {
	if g.code != codes.OK {
		return nil, status.Error(g.code, r.GetService())
	}

	return &grpc_health_v1.HealthCheckResponse{Status: g.status}, nil
}

func (g *dummyHealthCheckClient) Watch(ctx context.Context, r *grpc_health_v1.HealthCheckRequest, opts ...grpc.CallOption) (grpc_health_v1.Health_WatchClient, error) {
	return nil, status.Error(codes.Unimplemented, "unimplemented")
}

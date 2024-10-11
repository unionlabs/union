package cors

import (
	"bytes"
	"fmt"
	"net/http"
	"net/http/httptest"
	"regexp"
	"strings"
	"testing"
)

var testResponse = []byte("bar")
var testHandler = http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
	_, _ = w.Write(testResponse)
})

// For each key-value pair of this map, the value indicates whether the key
// is a list-based field (i.e. not a singleton field);
// see https://httpwg.org/specs/rfc9110.html#abnf.extension.
var allRespHeaders = map[string]bool{
	// see https://www.rfc-editor.org/rfc/rfc9110#section-12.5.5
	"Vary": true,
	// see https://fetch.spec.whatwg.org/#http-new-header-syntax
	"Access-Control-Allow-Origin":      false,
	"Access-Control-Allow-Credentials": false,
	"Access-Control-Allow-Methods":     true,
	"Access-Control-Allow-Headers":     true,
	"Access-Control-Max-Age":           false,
	"Access-Control-Expose-Headers":    true,
	// see https://wicg.github.io/private-network-access/
	"Access-Control-Allow-Private-Network": false,
}

func assertHeaders(t *testing.T, resHeaders http.Header, expHeaders http.Header) {
	t.Helper()
	for name, listBased := range allRespHeaders {
		got := resHeaders[name]
		want := expHeaders[name]
		if !listBased && !slicesEqual(got, want) {
			t.Errorf("Response header %q = %q, want %q", name, got, want)
			continue
		}
		if listBased && !slicesEqual(normalize(got), normalize(want)) {
			t.Errorf("Response header %q = %q, want %q", name, got, want)
			continue
		}
	}
}

// normalize normalizes a list-based field value,
// preserving both empty elements and the order of elements.
func normalize(s []string) (res []string) {
	for _, v := range s {
		for _, e := range strings.Split(v, ",") {
			e = strings.Trim(e, " \t")
			res = append(res, e)
		}
	}
	return
}

// TODO: when updating go directive to 1.21 or later,
// use slices.Equal instead.
func slicesEqual(s1, s2 []string) bool {
	if len(s1) != len(s2) {
		return false
	}
	for i := range s1 {
		if s1[i] != s2[i] {
			return false
		}
	}
	return true
}

func assertResponse(t *testing.T, res *httptest.ResponseRecorder, responseCode int) {
	t.Helper()
	if responseCode != res.Code {
		t.Errorf("assertResponse: expected response code to be %d but got %d. ", responseCode, res.Code)
	}
}

func TestSpec(t *testing.T) {
	cases := []struct {
		name          string
		options       Options
		method        string
		reqHeaders    http.Header
		resHeaders    http.Header
		originAllowed bool
	}{
		{
			"NoConfig",
			Options{
				// Intentionally left blank.
			},
			"GET",
			http.Header{},
			http.Header{
				"Vary": {"Origin"},
			},
			true,
		},
		{
			"MatchAllOrigin",
			Options{
				AllowedOrigins: []string{"*"},
			},
			"GET",
			http.Header{
				"Origin": {"http://foobar.com"},
			},
			http.Header{
				"Vary":                        {"Origin"},
				"Access-Control-Allow-Origin": {"*"},
			},
			true,
		},
		{
			"MatchAllOriginWithCredentials",
			Options{
				AllowedOrigins:   []string{"*"},
				AllowCredentials: true,
			},
			"GET",
			http.Header{
				"Origin": {"http://foobar.com"},
			},
			http.Header{
				"Vary":                             {"Origin"},
				"Access-Control-Allow-Origin":      {"*"},
				"Access-Control-Allow-Credentials": {"true"},
			},
			true,
		},
		{
			"AllowedOrigin",
			Options{
				AllowedOrigins: []string{"http://foobar.com"},
			},
			"GET",
			http.Header{
				"Origin": {"http://foobar.com"},
			},
			http.Header{
				"Vary":                        {"Origin"},
				"Access-Control-Allow-Origin": {"http://foobar.com"},
			},
			true,
		},
		{
			"WildcardOrigin",
			Options{
				AllowedOrigins: []string{"http://*.bar.com"},
			},
			"GET",
			http.Header{
				"Origin": {"http://foo.bar.com"},
			},
			http.Header{
				"Vary":                        {"Origin"},
				"Access-Control-Allow-Origin": {"http://foo.bar.com"},
			},
			true,
		},
		{
			"DisallowedOrigin",
			Options{
				AllowedOrigins: []string{"http://foobar.com"},
			},
			"GET",
			http.Header{
				"Origin": {"http://barbaz.com"},
			},
			http.Header{
				"Vary": {"Origin"},
			},
			false,
		},
		{
			"DisallowedWildcardOrigin",
			Options{
				AllowedOrigins: []string{"http://*.bar.com"},
			},
			"GET",
			http.Header{
				"Origin": {"http://foo.baz.com"},
			},
			http.Header{
				"Vary": {"Origin"},
			},
			false,
		},
		{
			"AllowedOriginFuncMatch",
			Options{
				AllowOriginFunc: func(o string) bool {
					return regexp.MustCompile("^http://foo").MatchString(o)
				},
			},
			"GET",
			http.Header{
				"Origin": {"http://foobar.com"},
			},
			http.Header{
				"Vary":                        {"Origin"},
				"Access-Control-Allow-Origin": {"http://foobar.com"},
			},
			true,
		},
		{
			"AllowOriginRequestFuncMatch",
			Options{
				AllowOriginRequestFunc: func(r *http.Request, o string) bool {
					return regexp.MustCompile("^http://foo").MatchString(o) && r.Header.Get("Authorization") == "secret"
				},
			},
			"GET",
			http.Header{
				"Origin":        {"http://foobar.com"},
				"Authorization": {"secret"},
			},
			http.Header{
				"Vary":                        {"Origin"},
				"Access-Control-Allow-Origin": {"http://foobar.com"},
			},
			true,
		},
		{
			"AllowOriginVaryRequestFuncMatch",
			Options{
				AllowOriginVaryRequestFunc: func(r *http.Request, o string) (bool, []string) {
					return regexp.MustCompile("^http://foo").MatchString(o) && r.Header.Get("Authorization") == "secret", []string{"Authorization"}
				},
			},
			"GET",
			http.Header{
				"Origin":        {"http://foobar.com"},
				"Authorization": {"secret"},
			},
			http.Header{
				"Vary":                        {"Origin, Authorization"},
				"Access-Control-Allow-Origin": {"http://foobar.com"},
			},
			true,
		},
		{
			"AllowOriginRequestFuncNotMatch",
			Options{
				AllowOriginRequestFunc: func(r *http.Request, o string) bool {
					return regexp.MustCompile("^http://foo").MatchString(o) && r.Header.Get("Authorization") == "secret"
				},
			},
			"GET",
			http.Header{
				"Origin":        {"http://foobar.com"},
				"Authorization": {"not-secret"},
			},
			http.Header{
				"Vary": {"Origin"},
			},
			false,
		},
		{
			"MaxAge",
			Options{
				AllowedOrigins: []string{"http://example.com"},
				AllowedMethods: []string{"GET"},
				MaxAge:         10,
			},
			"OPTIONS",
			http.Header{
				"Origin":                        {"http://example.com"},
				"Access-Control-Request-Method": {"GET"},
			},
			http.Header{
				"Vary":                         {"Origin, Access-Control-Request-Method, Access-Control-Request-Headers"},
				"Access-Control-Allow-Origin":  {"http://example.com"},
				"Access-Control-Allow-Methods": {"GET"},
				"Access-Control-Max-Age":       {"10"},
			},
			true,
		},
		{
			"MaxAgeNegative",
			Options{
				AllowedOrigins: []string{"http://example.com"},
				AllowedMethods: []string{"GET"},
				MaxAge:         -1,
			},
			"OPTIONS",
			http.Header{
				"Origin":                        {"http://example.com"},
				"Access-Control-Request-Method": {"GET"},
			},
			http.Header{
				"Vary":                         {"Origin, Access-Control-Request-Method, Access-Control-Request-Headers"},
				"Access-Control-Allow-Origin":  {"http://example.com"},
				"Access-Control-Allow-Methods": {"GET"},
				"Access-Control-Max-Age":       {"0"},
			},
			true,
		},
		{
			"AllowedMethod",
			Options{
				AllowedOrigins: []string{"http://foobar.com"},
				AllowedMethods: []string{"PUT", "DELETE"},
			},
			"OPTIONS",
			http.Header{
				"Origin":                        {"http://foobar.com"},
				"Access-Control-Request-Method": {"PUT"},
			},
			http.Header{
				"Vary":                         {"Origin, Access-Control-Request-Method, Access-Control-Request-Headers"},
				"Access-Control-Allow-Origin":  {"http://foobar.com"},
				"Access-Control-Allow-Methods": {"PUT"},
			},
			true,
		},
		{
			"DisallowedMethod",
			Options{
				AllowedOrigins: []string{"http://foobar.com"},
				AllowedMethods: []string{"PUT", "DELETE"},
			},
			"OPTIONS",
			http.Header{
				"Origin":                        {"http://foobar.com"},
				"Access-Control-Request-Method": {"PATCH"},
			},
			http.Header{
				"Vary": {"Origin, Access-Control-Request-Method, Access-Control-Request-Headers"},
			},
			true,
		},
		{
			"AllowedHeaders",
			Options{
				AllowedOrigins: []string{"http://foobar.com"},
				AllowedHeaders: []string{"X-Header-1", "x-header-2", "X-HEADER-3"},
			},
			"OPTIONS",
			http.Header{
				"Origin":                         {"http://foobar.com"},
				"Access-Control-Request-Method":  {"GET"},
				"Access-Control-Request-Headers": {"x-header-1,x-header-2"},
			},
			http.Header{
				"Vary":                         {"Origin, Access-Control-Request-Method, Access-Control-Request-Headers"},
				"Access-Control-Allow-Origin":  {"http://foobar.com"},
				"Access-Control-Allow-Methods": {"GET"},
				"Access-Control-Allow-Headers": {"x-header-1,x-header-2"},
			},
			true,
		},
		{
			"DefaultAllowedHeaders",
			Options{
				AllowedOrigins: []string{"http://foobar.com"},
				AllowedHeaders: []string{},
			},
			"OPTIONS",
			http.Header{
				"Origin":                         {"http://foobar.com"},
				"Access-Control-Request-Method":  {"GET"},
				"Access-Control-Request-Headers": {"x-requested-with"},
			},
			http.Header{
				"Vary":                         {"Origin, Access-Control-Request-Method, Access-Control-Request-Headers"},
				"Access-Control-Allow-Origin":  {"http://foobar.com"},
				"Access-Control-Allow-Methods": {"GET"},
				"Access-Control-Allow-Headers": {"x-requested-with"},
			},
			true,
		},
		{
			"AllowedWildcardHeader",
			Options{
				AllowedOrigins: []string{"http://foobar.com"},
				AllowedHeaders: []string{"*"},
			},
			"OPTIONS",
			http.Header{
				"Origin":                         {"http://foobar.com"},
				"Access-Control-Request-Method":  {"GET"},
				"Access-Control-Request-Headers": {"x-header-1,x-header-2"},
			},
			http.Header{
				"Vary":                         {"Origin, Access-Control-Request-Method, Access-Control-Request-Headers"},
				"Access-Control-Allow-Origin":  {"http://foobar.com"},
				"Access-Control-Allow-Methods": {"GET"},
				"Access-Control-Allow-Headers": {"x-header-1,x-header-2"},
			},
			true,
		},
		{
			"DisallowedHeader",
			Options{
				AllowedOrigins: []string{"http://foobar.com"},
				AllowedHeaders: []string{"X-Header-1", "x-header-2"},
			},
			"OPTIONS",
			http.Header{
				"Origin":                         {"http://foobar.com"},
				"Access-Control-Request-Method":  {"GET"},
				"Access-Control-Request-Headers": {"x-header-1,x-header-3"},
			},
			http.Header{
				"Vary": {"Origin, Access-Control-Request-Method, Access-Control-Request-Headers"},
			},
			true,
		},
		{
			"ExposedHeader",
			Options{
				AllowedOrigins: []string{"http://foobar.com"},
				ExposedHeaders: []string{"X-Header-1", "x-header-2"},
			},
			"GET",
			http.Header{
				"Origin": {"http://foobar.com"},
			},
			http.Header{
				"Vary":                          {"Origin"},
				"Access-Control-Allow-Origin":   {"http://foobar.com"},
				"Access-Control-Expose-Headers": {"X-Header-1, X-Header-2"},
			},
			true,
		},
		{
			"AllowedCredentials",
			Options{
				AllowedOrigins:   []string{"http://foobar.com"},
				AllowCredentials: true,
			},
			"OPTIONS",
			http.Header{
				"Origin":                        {"http://foobar.com"},
				"Access-Control-Request-Method": {"GET"},
			},
			http.Header{
				"Vary":                             {"Origin, Access-Control-Request-Method, Access-Control-Request-Headers"},
				"Access-Control-Allow-Origin":      {"http://foobar.com"},
				"Access-Control-Allow-Methods":     {"GET"},
				"Access-Control-Allow-Credentials": {"true"},
			},
			true,
		},
		{
			"AllowedPrivateNetwork",
			Options{
				AllowedOrigins:      []string{"http://foobar.com"},
				AllowPrivateNetwork: true,
			},
			"OPTIONS",
			http.Header{
				"Origin":                                 {"http://foobar.com"},
				"Access-Control-Request-Method":          {"GET"},
				"Access-Control-Request-Private-Network": {"true"},
			},
			http.Header{
				"Vary":                                 {"Origin, Access-Control-Request-Method, Access-Control-Request-Headers, Access-Control-Request-Private-Network"},
				"Access-Control-Allow-Origin":          {"http://foobar.com"},
				"Access-Control-Allow-Methods":         {"GET"},
				"Access-Control-Allow-Private-Network": {"true"},
			},
			true,
		},
		{
			"DisallowedPrivateNetwork",
			Options{
				AllowedOrigins: []string{"http://foobar.com"},
			},
			"OPTIONS",
			http.Header{
				"Origin":                                {"http://foobar.com"},
				"Access-Control-Request-Method":         {"GET"},
				"Access-Control-Request-PrivateNetwork": {"true"},
			},
			http.Header{
				"Vary":                         {"Origin, Access-Control-Request-Method, Access-Control-Request-Headers"},
				"Access-Control-Allow-Origin":  {"http://foobar.com"},
				"Access-Control-Allow-Methods": {"GET"},
			},
			true,
		},
		{
			"OptionPassthrough",
			Options{
				OptionsPassthrough: true,
			},
			"OPTIONS",
			http.Header{
				"Origin":                        {"http://foobar.com"},
				"Access-Control-Request-Method": {"GET"},
			},
			http.Header{
				"Vary":                         {"Origin, Access-Control-Request-Method, Access-Control-Request-Headers"},
				"Access-Control-Allow-Origin":  {"*"},
				"Access-Control-Allow-Methods": {"GET"},
			},
			true,
		},
		{
			"NonPreflightOptions",
			Options{
				AllowedOrigins: []string{"http://foobar.com"},
			},
			"OPTIONS",
			http.Header{
				"Origin": {"http://foobar.com"},
			},
			http.Header{
				"Vary":                        {"Origin"},
				"Access-Control-Allow-Origin": {"http://foobar.com"},
			},
			true,
		}, {
			"AllowedOriginsPlusAllowOriginFunc",
			Options{
				AllowedOrigins: []string{"*"},
				AllowOriginFunc: func(origin string) bool {
					return true
				},
			},
			"GET",
			http.Header{
				"Origin": {"http://foobar.com"},
			},
			http.Header{
				"Vary":                        {"Origin"},
				"Access-Control-Allow-Origin": {"http://foobar.com"},
			},
			true,
		},
		{
			"MultipleACRHHeaders",
			Options{
				AllowedOrigins: []string{"http://foobar.com"},
				AllowedHeaders: []string{"Content-Type", "Authorization"},
			},
			"OPTIONS",
			http.Header{
				"Origin":                         {"http://foobar.com"},
				"Access-Control-Request-Method":  {"GET"},
				"Access-Control-Request-Headers": {"authorization", "content-type"},
			},
			http.Header{
				"Vary":                         {"Origin, Access-Control-Request-Method, Access-Control-Request-Headers"},
				"Access-Control-Allow-Origin":  {"http://foobar.com"},
				"Access-Control-Allow-Methods": {"GET"},
				"Access-Control-Allow-Headers": {"authorization", "content-type"},
			},
			true,
		},
		{
			"MultipleACRHHeadersWithOWSAndEmptyElements",
			Options{
				AllowedOrigins: []string{"http://foobar.com"},
				AllowedHeaders: []string{"Content-Type", "Authorization"},
			},
			"OPTIONS",
			http.Header{
				"Origin":                         {"http://foobar.com"},
				"Access-Control-Request-Method":  {"GET"},
				"Access-Control-Request-Headers": {"authorization\t", " ", " content-type"},
			},
			http.Header{
				"Vary":                         {"Origin, Access-Control-Request-Method, Access-Control-Request-Headers"},
				"Access-Control-Allow-Origin":  {"http://foobar.com"},
				"Access-Control-Allow-Methods": {"GET"},
				"Access-Control-Allow-Headers": {"authorization\t", " ", " content-type"},
			},
			true,
		},
	}
	for i := range cases {
		tc := cases[i]
		t.Run(tc.name, func(t *testing.T) {
			s := New(tc.options)

			req, _ := http.NewRequest(tc.method, "http://example.com/foo", nil)
			for name, values := range tc.reqHeaders {
				for _, value := range values {
					req.Header.Add(name, value)
				}
			}

			t.Run("OriginAllowed", func(t *testing.T) {
				if have, want := s.OriginAllowed(req), tc.originAllowed; have != want {
					t.Errorf("OriginAllowed have: %t want: %t", have, want)
				}
			})

			t.Run("Handler", func(t *testing.T) {
				res := httptest.NewRecorder()
				s.Handler(testHandler).ServeHTTP(res, req)
				assertHeaders(t, res.Header(), tc.resHeaders)
			})
			t.Run("HandlerFunc", func(t *testing.T) {
				res := httptest.NewRecorder()
				s.HandlerFunc(res, req)
				assertHeaders(t, res.Header(), tc.resHeaders)
			})
			t.Run("Negroni", func(t *testing.T) {
				res := httptest.NewRecorder()
				s.ServeHTTP(res, req, testHandler)
				assertHeaders(t, res.Header(), tc.resHeaders)
			})

		})
	}
}

func TestDebug(t *testing.T) {
	s := New(Options{
		Debug: true,
	})

	if s.Log == nil {
		t.Error("Logger not created when debug=true")
	}
}

type testLogger struct {
	buf *bytes.Buffer
}

func (l *testLogger) Printf(format string, v ...interface{}) {
	fmt.Fprintf(l.buf, format, v...)
}

func TestLogger(t *testing.T) {
	logger := &testLogger{buf: &bytes.Buffer{}}
	s := New(Options{
		Logger: logger,
	})

	if s.Log == nil {
		t.Error("Logger not created when Logger is set")
	}
	s.logf("test")
	if logger.buf.String() != "test" {
		t.Error("Logger not used")
	}
}

func TestDefault(t *testing.T) {
	s := Default()
	if s.Log != nil {
		t.Error("c.log should be nil when Default")
	}
	if !s.allowedOriginsAll {
		t.Error("c.allowedOriginsAll should be true when Default")
	}
	if s.allowedHeaders.Size() == 0 {
		t.Error("c.allowedHeaders should be empty when Default")
	}
	if s.allowedMethods == nil {
		t.Error("c.allowedMethods should be nil when Default")
	}
}

func TestHandlePreflightInvalidOriginAbortion(t *testing.T) {
	s := New(Options{
		AllowedOrigins: []string{"http://foo.com"},
	})
	res := httptest.NewRecorder()
	req, _ := http.NewRequest("OPTIONS", "http://example.com/foo", nil)
	req.Header.Add("Origin", "http://example.com")

	s.handlePreflight(res, req)

	assertHeaders(t, res.Header(), http.Header{
		"Vary": {"Origin, Access-Control-Request-Method, Access-Control-Request-Headers"},
	})
}

func TestHandlePreflightNoOptionsAbortion(t *testing.T) {
	s := New(Options{
		// Intentionally left blank.
	})
	res := httptest.NewRecorder()
	req, _ := http.NewRequest("GET", "http://example.com/foo", nil)

	s.handlePreflight(res, req)

	assertHeaders(t, res.Header(), http.Header{})
}

func TestHandleActualRequestInvalidOriginAbortion(t *testing.T) {
	s := New(Options{
		AllowedOrigins: []string{"http://foo.com"},
	})
	res := httptest.NewRecorder()
	req, _ := http.NewRequest("GET", "http://example.com/foo", nil)
	req.Header.Add("Origin", "http://example.com")

	s.handleActualRequest(res, req)

	assertHeaders(t, res.Header(), http.Header{
		"Vary": {"Origin"},
	})
}

func TestHandleActualRequestInvalidMethodAbortion(t *testing.T) {
	s := New(Options{
		AllowedMethods:   []string{"POST"},
		AllowCredentials: true,
	})
	res := httptest.NewRecorder()
	req, _ := http.NewRequest("GET", "http://example.com/foo", nil)
	req.Header.Add("Origin", "http://example.com")

	s.handleActualRequest(res, req)

	assertHeaders(t, res.Header(), http.Header{
		"Vary": {"Origin"},
	})
}

func TestIsMethodAllowedReturnsFalseWithNoMethods(t *testing.T) {
	s := New(Options{
		// Intentionally left blank.
	})
	s.allowedMethods = []string{}
	if s.isMethodAllowed("") {
		t.Error("IsMethodAllowed should return false when c.allowedMethods is nil.")
	}
}

func TestIsMethodAllowedReturnsTrueWithOptions(t *testing.T) {
	s := New(Options{
		// Intentionally left blank.
	})
	if !s.isMethodAllowed("OPTIONS") {
		t.Error("IsMethodAllowed should return true when c.allowedMethods is nil.")
	}
}

func TestOptionsSuccessStatusCodeDefault(t *testing.T) {
	s := New(Options{
		// Intentionally left blank.
	})

	req, _ := http.NewRequest("OPTIONS", "http://example.com/foo", nil)
	req.Header.Add("Access-Control-Request-Method", "GET")

	t.Run("Handler", func(t *testing.T) {
		res := httptest.NewRecorder()
		s.Handler(testHandler).ServeHTTP(res, req)
		assertResponse(t, res, http.StatusNoContent)
	})
	t.Run("HandlerFunc", func(t *testing.T) {
		res := httptest.NewRecorder()
		s.HandlerFunc(res, req)
		assertResponse(t, res, http.StatusNoContent)
	})
	t.Run("Negroni", func(t *testing.T) {
		res := httptest.NewRecorder()
		s.ServeHTTP(res, req, testHandler)
		assertResponse(t, res, http.StatusNoContent)
	})
}

func TestOptionsSuccessStatusCodeOverride(t *testing.T) {
	s := New(Options{
		OptionsSuccessStatus: http.StatusOK,
	})

	req, _ := http.NewRequest("OPTIONS", "http://example.com/foo", nil)
	req.Header.Add("Access-Control-Request-Method", "GET")

	t.Run("Handler", func(t *testing.T) {
		res := httptest.NewRecorder()
		s.Handler(testHandler).ServeHTTP(res, req)
		assertResponse(t, res, http.StatusOK)
	})
	t.Run("HandlerFunc", func(t *testing.T) {
		res := httptest.NewRecorder()
		s.HandlerFunc(res, req)
		assertResponse(t, res, http.StatusOK)
	})
	t.Run("Negroni", func(t *testing.T) {
		res := httptest.NewRecorder()
		s.ServeHTTP(res, req, testHandler)
		assertResponse(t, res, http.StatusOK)
	})
}

func TestAccessControlExposeHeadersPresence(t *testing.T) {
	cases := []struct {
		name    string
		options Options
		want    bool
	}{
		{
			name:    "omit",
			options: Options{},
			want:    false,
		},
		{
			name: "include",
			options: Options{
				ExposedHeaders: []string{"X-Something"},
			},
			want: true,
		},
	}

	for _, tt := range cases {
		t.Run(tt.name, func(t *testing.T) {
			s := New(tt.options)

			req, _ := http.NewRequest("GET", "http://example.com/foo", nil)
			req.Header.Add("Origin", "http://foobar.com")

			assertExposeHeaders := func(t *testing.T, resHeaders http.Header) {
				if _, have := resHeaders["Access-Control-Expose-Headers"]; have != tt.want {
					t.Errorf("Access-Control-Expose-Headers have: %t want: %t", have, tt.want)
				}
			}

			t.Run("Handler", func(t *testing.T) {
				res := httptest.NewRecorder()
				s.Handler(testHandler).ServeHTTP(res, req)
				assertExposeHeaders(t, res.Header())
			})
			t.Run("HandlerFunc", func(t *testing.T) {
				res := httptest.NewRecorder()
				s.HandlerFunc(res, req)
				assertExposeHeaders(t, res.Header())
			})
			t.Run("Negroni", func(t *testing.T) {
				res := httptest.NewRecorder()
				s.ServeHTTP(res, req, testHandler)
				assertExposeHeaders(t, res.Header())
			})
		})
	}

}

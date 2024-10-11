package handlers

import (
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

func TestDefaultCORSHandlerReturnsOk(t *testing.T) {
	r := newRequest(http.MethodGet, "http://www.example.com/")
	rr := httptest.NewRecorder()

	testHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {})

	CORS()(testHandler).ServeHTTP(rr, r)
	resp := rr.Result()
	if got, want := resp.StatusCode, http.StatusOK; got != want {
		t.Fatalf("bad status: got %v want %v", got, want)
	}
}

func TestDefaultCORSHandlerReturnsOkWithOrigin(t *testing.T) {
	r := newRequest(http.MethodGet, "http://www.example.com/")
	r.Header.Set("Origin", r.URL.String())

	rr := httptest.NewRecorder()

	testHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {})

	CORS()(testHandler).ServeHTTP(rr, r)
	resp := rr.Result()
	if got, want := resp.StatusCode, http.StatusOK; got != want {
		t.Fatalf("bad status: got %v want %v", got, want)
	}
}

func TestCORSHandlerIgnoreOptionsFallsThrough(t *testing.T) {
	r := newRequest(http.MethodOptions, "http://www.example.com/")
	r.Header.Set("Origin", r.URL.String())

	rr := httptest.NewRecorder()

	testHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusTeapot)
	})

	CORS(IgnoreOptions())(testHandler).ServeHTTP(rr, r)
	resp := rr.Result()

	if got, want := resp.StatusCode, http.StatusTeapot; got != want {
		t.Fatalf("bad status: got %v want %v", got, want)
	}
}

func TestCORSHandlerSetsExposedHeaders(t *testing.T) {
	// Test default configuration.
	r := newRequest(http.MethodGet, "http://www.example.com/")
	r.Header.Set("Origin", r.URL.String())

	rr := httptest.NewRecorder()
	testHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {})

	CORS(ExposedHeaders([]string{"X-CORS-TEST"}))(testHandler).ServeHTTP(rr, r)
	resp := rr.Result()

	if got, want := resp.StatusCode, http.StatusOK; got != want {
		t.Fatalf("bad status: got %v want %v", got, want)
	}

	header := resp.Header.Get(corsExposeHeadersHeader)
	if got, want := header, "X-Cors-Test"; got != want {
		t.Fatalf("bad header: expected %q header, got empty header for method.", want)
	}
}

func TestCORSHandlerUnsetRequestMethodForPreflightBadRequest(t *testing.T) {
	r := newRequest(http.MethodOptions, "http://www.example.com/")
	r.Header.Set("Origin", r.URL.String())

	rr := httptest.NewRecorder()
	testHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {})

	CORS(AllowedMethods([]string{"DELETE"}))(testHandler).ServeHTTP(rr, r)
	resp := rr.Result()

	if got, want := resp.StatusCode, http.StatusBadRequest; got != want {
		t.Fatalf("bad status: got %v want %v", got, want)
	}
}

func TestCORSHandlerInvalidRequestMethodForPreflightMethodNotAllowed(t *testing.T) {
	r := newRequest(http.MethodOptions, "http://www.example.com/")
	r.Header.Set("Origin", r.URL.String())
	r.Header.Set(corsRequestMethodHeader, http.MethodDelete)

	rr := httptest.NewRecorder()

	testHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {})

	CORS()(testHandler).ServeHTTP(rr, r)
	resp := rr.Result()

	if got, want := resp.StatusCode, http.StatusMethodNotAllowed; got != want {
		t.Fatalf("bad status: got %v want %v", got, want)
	}
}

func TestCORSHandlerOptionsRequestMustNotBePassedToNextHandler(t *testing.T) {
	r := newRequest(http.MethodOptions, "http://www.example.com/")
	r.Header.Set("Origin", r.URL.String())
	r.Header.Set(corsRequestMethodHeader, http.MethodGet)

	rr := httptest.NewRecorder()

	testHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		t.Fatal("Options request must not be passed to next handler")
	})

	CORS()(testHandler).ServeHTTP(rr, r)
	resp := rr.Result()

	if got, want := resp.StatusCode, http.StatusOK; got != want {
		t.Fatalf("bad status: got %v want %v", got, want)
	}
}

func TestCORSHandlerOptionsRequestMustNotBePassedToNextHandlerWithCustomStatusCode(t *testing.T) {
	statusCode := http.StatusNoContent
	r := newRequest(http.MethodOptions, "http://www.example.com/")
	r.Header.Set("Origin", r.URL.String())
	r.Header.Set(corsRequestMethodHeader, http.MethodGet)

	rr := httptest.NewRecorder()

	testHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		t.Fatal("Options request must not be passed to next handler")
	})

	CORS(OptionStatusCode(statusCode))(testHandler).ServeHTTP(rr, r)
	resp := rr.Result()

	if got, want := resp.StatusCode, statusCode; got != want {
		t.Fatalf("bad status: got %v want %v", got, want)
	}
}

func TestCORSHandlerOptionsRequestMustNotBePassedToNextHandlerWhenOriginNotAllowed(t *testing.T) {
	r := newRequest(http.MethodOptions, "http://www.example.com/")
	r.Header.Set("Origin", r.URL.String())
	r.Header.Set(corsRequestMethodHeader, http.MethodGet)

	rr := httptest.NewRecorder()

	testHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		t.Fatal("Options request must not be passed to next handler")
	})

	CORS(AllowedOrigins([]string{}))(testHandler).ServeHTTP(rr, r)
	resp := rr.Result()

	if got, want := resp.StatusCode, http.StatusOK; got != want {
		t.Fatalf("bad status: got %v want %v", got, want)
	}
}

func TestCORSHandlerAllowedMethodForPreflight(t *testing.T) {
	r := newRequest(http.MethodOptions, "http://www.example.com/")
	r.Header.Set("Origin", r.URL.String())
	r.Header.Set(corsRequestMethodHeader, http.MethodDelete)

	rc := httptest.NewRecorder()

	testHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {})

	CORS(AllowedMethods([]string{"DELETE"}))(testHandler).ServeHTTP(rc, r)

	resp := rc.Result()
	if got, want := resp.StatusCode, http.StatusOK; got != want {
		t.Fatalf("bad status: got %v want %v", got, want)
	}

	header := resp.Header.Get(corsAllowMethodsHeader)
	if header != http.MethodDelete {
		t.Fatalf("bad header: expected %q method header, got %q header.", http.MethodDelete, header)
	}
}

func TestCORSHandlerAllowMethodsNotSetForSimpleRequestPreflight(t *testing.T) {
	for _, method := range defaultCorsMethods {
		r := newRequest(http.MethodOptions, "http://www.example.com/")
		r.Header.Set("Origin", r.URL.String())
		r.Header.Set(corsRequestMethodHeader, method)

		rr := httptest.NewRecorder()

		testHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {})

		CORS()(testHandler).ServeHTTP(rr, r)
		resp := rr.Result()

		if got, want := resp.StatusCode, http.StatusOK; got != want {
			t.Fatalf("bad status: got %v want %v", got, want)
		}

		header := resp.Header.Get(corsAllowMethodsHeader)
		if got, want := header, ""; got != want {
			t.Fatalf("bad header: expected %q method header, got %q.", want, got)
		}
	}
}

func TestCORSHandlerAllowedHeaderNotSetForSimpleRequestPreflight(t *testing.T) {
	for _, simpleHeader := range defaultCorsHeaders {
		r := newRequest(http.MethodOptions, "http://www.example.com/")
		r.Header.Set("Origin", r.URL.String())
		r.Header.Set(corsRequestMethodHeader, http.MethodGet)
		r.Header.Set(corsRequestHeadersHeader, simpleHeader)

		rr := httptest.NewRecorder()

		testHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {})

		CORS()(testHandler).ServeHTTP(rr, r)
		resp := rr.Result()

		if got, want := resp.StatusCode, http.StatusOK; got != want {
			t.Fatalf("bad status: got %v want %v", got, want)
		}

		header := resp.Header.Get(corsAllowHeadersHeader)
		if got, want := header, ""; got != want {
			t.Fatalf("bad header: expected %q header, got %q.", want, got)
		}
	}
}

func TestCORSHandlerAllowedHeaderForPreflight(t *testing.T) {
	r := newRequest(http.MethodOptions, "http://www.example.com/")
	r.Header.Set("Origin", r.URL.String())
	r.Header.Set(corsRequestMethodHeader, http.MethodPost)
	r.Header.Set(corsRequestHeadersHeader, "Content-Type")

	rr := httptest.NewRecorder()

	testHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {})

	CORS(AllowedHeaders([]string{"Content-Type"}))(testHandler).ServeHTTP(rr, r)
	resp := rr.Result()

	if got, want := resp.StatusCode, http.StatusOK; got != want {
		t.Fatalf("bad status: got %v want %v", got, want)
	}

	header := resp.Header.Get(corsAllowHeadersHeader)
	if got, want := header, "Content-Type"; got != want {
		t.Fatalf("bad header: expected %q header, got %q header.", want, got)
	}
}

func TestCORSHandlerInvalidHeaderForPreflightForbidden(t *testing.T) {
	r := newRequest(http.MethodOptions, "http://www.example.com/")
	r.Header.Set("Origin", r.URL.String())
	r.Header.Set(corsRequestMethodHeader, http.MethodPost)
	r.Header.Set(corsRequestHeadersHeader, "Content-Type")

	rr := httptest.NewRecorder()

	testHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {})

	CORS()(testHandler).ServeHTTP(rr, r)
	resp := rr.Result()

	if got, want := resp.StatusCode, http.StatusForbidden; got != want {
		t.Fatalf("bad status: got %v want %v", got, want)
	}
}

func TestCORSHandlerMaxAgeForPreflight(t *testing.T) {
	r := newRequest(http.MethodOptions, "http://www.example.com/")
	r.Header.Set("Origin", r.URL.String())
	r.Header.Set(corsRequestMethodHeader, http.MethodPost)

	rr := httptest.NewRecorder()

	testHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {})

	CORS(MaxAge(3500))(testHandler).ServeHTTP(rr, r)
	resp := rr.Result()

	if got, want := resp.StatusCode, http.StatusOK; got != want {
		t.Fatalf("bad status: got %v want %v", got, want)
	}

	header := resp.Header.Get(corsMaxAgeHeader)
	if got, want := header, "600"; got != want {
		t.Fatalf("bad header: expected %q to be %q, got %q.", corsMaxAgeHeader, want, got)
	}
}

func TestCORSHandlerAllowedCredentials(t *testing.T) {
	r := newRequest(http.MethodGet, "http://www.example.com/")
	r.Header.Set("Origin", r.URL.String())

	rr := httptest.NewRecorder()

	testHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {})

	CORS(AllowCredentials())(testHandler).ServeHTTP(rr, r)
	resp := rr.Result()

	if status := resp.StatusCode; status != http.StatusOK {
		t.Fatalf("bad status: got %v want %v", status, http.StatusOK)
	}

	header := resp.Header.Get(corsAllowCredentialsHeader)
	if got, want := header, "true"; got != want {
		t.Fatalf("bad header: expected %q to be %q, got %q.", corsAllowCredentialsHeader, want, got)
	}
}

func TestCORSHandlerMultipleAllowOriginsSetsVaryHeader(t *testing.T) {
	r := newRequest(http.MethodGet, "http://www.example.com/")
	r.Header.Set("Origin", r.URL.String())

	rr := httptest.NewRecorder()

	testHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {})

	CORS(AllowedOrigins([]string{r.URL.String(), "http://google.com"}))(testHandler).ServeHTTP(rr, r)
	resp := rr.Result()

	if status := resp.StatusCode; status != http.StatusOK {
		t.Fatalf("bad status: got %v want %v", status, http.StatusOK)
	}

	header := resp.Header.Get(corsVaryHeader)
	if got, want := header, corsOriginHeader; got != want {
		t.Fatalf("bad header: expected %s to be %q, got %q.", corsVaryHeader, want, got)
	}
}

func TestCORSWithMultipleHandlers(t *testing.T) {
	var lastHandledBy string
	corsMiddleware := CORS()

	testHandler1 := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		lastHandledBy = "testHandler1"
	})
	testHandler2 := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		lastHandledBy = "testHandler2"
	})

	r1 := newRequest(http.MethodGet, "http://www.example.com/")
	rr1 := httptest.NewRecorder()
	handler1 := corsMiddleware(testHandler1)

	corsMiddleware(testHandler2)

	handler1.ServeHTTP(rr1, r1)
	if lastHandledBy != "testHandler1" {
		t.Fatalf("bad CORS() registration: Handler served should be Handler registered")
	}
}

func TestCORSOriginValidatorWithImplicitStar(t *testing.T) {
	r := newRequest(http.MethodGet, "http://a.example.com")
	r.Header.Set("Origin", r.URL.String())
	rr := httptest.NewRecorder()

	testHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {})

	originValidator := func(origin string) bool {
		return strings.HasSuffix(origin, ".example.com")
	}

	CORS(AllowedOriginValidator(originValidator))(testHandler).ServeHTTP(rr, r)
	resp := rr.Result()
	header := resp.Header.Get(corsAllowOriginHeader)
	if got, want := header, r.URL.String(); got != want {
		t.Fatalf("bad header: expected %s to be %q, got %q.", corsAllowOriginHeader, want, got)
	}
}

func TestCORSOriginValidatorWithExplicitStar(t *testing.T) {
	r := newRequest(http.MethodGet, "http://a.example.com")
	r.Header.Set("Origin", r.URL.String())
	rr := httptest.NewRecorder()

	testHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {})

	originValidator := func(origin string) bool {
		return strings.HasSuffix(origin, ".example.com")
	}

	CORS(
		AllowedOriginValidator(originValidator),
		AllowedOrigins([]string{"*"}),
	)(testHandler).ServeHTTP(rr, r)
	resp := rr.Result()
	header := resp.Header.Get(corsAllowOriginHeader)
	if got, want := header, "*"; got != want {
		t.Fatalf("bad header: expected %q to be %q, got %q.", corsAllowOriginHeader, want, got)
	}
}

func TestCORSAllowStar(t *testing.T) {
	r := newRequest(http.MethodGet, "http://a.example.com")
	r.Header.Set("Origin", r.URL.String())
	rr := httptest.NewRecorder()

	testHandler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {})

	CORS()(testHandler).ServeHTTP(rr, r)
	resp := rr.Result()
	header := resp.Header.Get(corsAllowOriginHeader)
	if got, want := header, "*"; got != want {
		t.Fatalf("bad header: expected %q to be %q, got %q.", corsAllowOriginHeader, want, got)
	}
}

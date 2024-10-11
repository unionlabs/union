package mux

import (
	"bytes"
	"net/http"
	"testing"
)

type testMiddleware struct {
	timesCalled uint
}

func (tm *testMiddleware) Middleware(h http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		tm.timesCalled++
		h.ServeHTTP(w, r)
	})
}

func dummyHandler(w http.ResponseWriter, r *http.Request) {}

func TestMiddlewareAdd(t *testing.T) {
	router := NewRouter()
	router.HandleFunc("/", dummyHandler).Methods("GET")

	mw := &testMiddleware{}

	router.useInterface(mw)
	if len(router.middlewares) != 1 || router.middlewares[0] != mw {
		t.Fatal("Middleware interface was not added correctly")
	}

	router.Use(mw.Middleware)
	if len(router.middlewares) != 2 {
		t.Fatal("Middleware method was not added correctly")
	}

	banalMw := func(handler http.Handler) http.Handler {
		return handler
	}
	router.Use(banalMw)
	if len(router.middlewares) != 3 {
		t.Fatal("Middleware function was not added correctly")
	}
}

func TestMiddleware(t *testing.T) {
	router := NewRouter()
	router.HandleFunc("/", dummyHandler).Methods("GET")

	mw := &testMiddleware{}
	router.useInterface(mw)

	rw := NewRecorder()
	req := newRequest("GET", "/")

	t.Run("regular middleware call", func(t *testing.T) {
		router.ServeHTTP(rw, req)
		if mw.timesCalled != 1 {
			t.Fatalf("Expected %d calls, but got only %d", 1, mw.timesCalled)
		}
	})

	t.Run("not called for 404", func(t *testing.T) {
		req = newRequest("GET", "/not/found")
		router.ServeHTTP(rw, req)
		if mw.timesCalled != 1 {
			t.Fatalf("Expected %d calls, but got only %d", 1, mw.timesCalled)
		}
	})

	t.Run("not called for method mismatch", func(t *testing.T) {
		req = newRequest("POST", "/")
		router.ServeHTTP(rw, req)
		if mw.timesCalled != 1 {
			t.Fatalf("Expected %d calls, but got only %d", 1, mw.timesCalled)
		}
	})

	t.Run("regular call using function middleware", func(t *testing.T) {
		router.Use(mw.Middleware)
		req = newRequest("GET", "/")
		router.ServeHTTP(rw, req)
		if mw.timesCalled != 3 {
			t.Fatalf("Expected %d calls, but got only %d", 3, mw.timesCalled)
		}
	})
}

func TestMiddlewareSubrouter(t *testing.T) {
	router := NewRouter()
	router.HandleFunc("/", dummyHandler).Methods("GET")

	subrouter := router.PathPrefix("/sub").Subrouter()
	subrouter.HandleFunc("/x", dummyHandler).Methods("GET")

	mw := &testMiddleware{}
	subrouter.useInterface(mw)

	rw := NewRecorder()
	req := newRequest("GET", "/")

	t.Run("not called for route outside subrouter", func(t *testing.T) {
		router.ServeHTTP(rw, req)
		if mw.timesCalled != 0 {
			t.Fatalf("Expected %d calls, but got only %d", 0, mw.timesCalled)
		}
	})

	t.Run("not called for subrouter root 404", func(t *testing.T) {
		req = newRequest("GET", "/sub/")
		router.ServeHTTP(rw, req)
		if mw.timesCalled != 0 {
			t.Fatalf("Expected %d calls, but got only %d", 0, mw.timesCalled)
		}
	})

	t.Run("called once for route inside subrouter", func(t *testing.T) {
		req = newRequest("GET", "/sub/x")
		router.ServeHTTP(rw, req)
		if mw.timesCalled != 1 {
			t.Fatalf("Expected %d calls, but got only %d", 1, mw.timesCalled)
		}
	})

	t.Run("not called for 404 inside subrouter", func(t *testing.T) {
		req = newRequest("GET", "/sub/not/found")
		router.ServeHTTP(rw, req)
		if mw.timesCalled != 1 {
			t.Fatalf("Expected %d calls, but got only %d", 1, mw.timesCalled)
		}
	})

	t.Run("middleware added to router", func(t *testing.T) {
		router.useInterface(mw)

		t.Run("called once for route outside subrouter", func(t *testing.T) {
			req = newRequest("GET", "/")
			router.ServeHTTP(rw, req)
			if mw.timesCalled != 2 {
				t.Fatalf("Expected %d calls, but got only %d", 2, mw.timesCalled)
			}
		})

		t.Run("called twice for route inside subrouter", func(t *testing.T) {
			req = newRequest("GET", "/sub/x")
			router.ServeHTTP(rw, req)
			if mw.timesCalled != 4 {
				t.Fatalf("Expected %d calls, but got only %d", 4, mw.timesCalled)
			}
		})
	})
}

func TestMiddlewareExecution(t *testing.T) {
	mwStr := []byte("Middleware\n")
	handlerStr := []byte("Logic\n")

	router := NewRouter()
	router.HandleFunc("/", func(w http.ResponseWriter, e *http.Request) {
		_, err := w.Write(handlerStr)
		if err != nil {
			t.Fatalf("Failed writing HTTP response: %v", err)
		}
	})

	t.Run("responds normally without middleware", func(t *testing.T) {
		rw := NewRecorder()
		req := newRequest("GET", "/")

		router.ServeHTTP(rw, req)

		if !bytes.Equal(rw.Body.Bytes(), handlerStr) {
			t.Fatal("Handler response is not what it should be")
		}
	})

	t.Run("responds with handler and middleware response", func(t *testing.T) {
		rw := NewRecorder()
		req := newRequest("GET", "/")

		router.Use(func(h http.Handler) http.Handler {
			return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				_, err := w.Write(mwStr)
				if err != nil {
					t.Fatalf("Failed writing HTTP response: %v", err)
				}
				h.ServeHTTP(w, r)
			})
		})

		router.ServeHTTP(rw, req)
		if !bytes.Equal(rw.Body.Bytes(), append(mwStr, handlerStr...)) {
			t.Fatal("Middleware + handler response is not what it should be")
		}
	})
}

func TestMiddlewareNotFound(t *testing.T) {
	mwStr := []byte("Middleware\n")
	handlerStr := []byte("Logic\n")

	router := NewRouter()
	router.HandleFunc("/", func(w http.ResponseWriter, e *http.Request) {
		_, err := w.Write(handlerStr)
		if err != nil {
			t.Fatalf("Failed writing HTTP response: %v", err)
		}
	})
	router.Use(func(h http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			_, err := w.Write(mwStr)
			if err != nil {
				t.Fatalf("Failed writing HTTP response: %v", err)
			}
			h.ServeHTTP(w, r)
		})
	})

	// Test not found call with default handler
	t.Run("not called", func(t *testing.T) {
		rw := NewRecorder()
		req := newRequest("GET", "/notfound")

		router.ServeHTTP(rw, req)
		if bytes.Contains(rw.Body.Bytes(), mwStr) {
			t.Fatal("Middleware was called for a 404")
		}
	})

	t.Run("not called with custom not found handler", func(t *testing.T) {
		rw := NewRecorder()
		req := newRequest("GET", "/notfound")

		router.NotFoundHandler = http.HandlerFunc(func(rw http.ResponseWriter, r *http.Request) {
			_, err := rw.Write([]byte("Custom 404 handler"))
			if err != nil {
				t.Fatalf("Failed writing HTTP response: %v", err)
			}
		})
		router.ServeHTTP(rw, req)

		if bytes.Contains(rw.Body.Bytes(), mwStr) {
			t.Fatal("Middleware was called for a custom 404")
		}
	})
}

func TestMiddlewareMethodMismatch(t *testing.T) {
	mwStr := []byte("Middleware\n")
	handlerStr := []byte("Logic\n")

	router := NewRouter()
	router.HandleFunc("/", func(w http.ResponseWriter, e *http.Request) {
		_, err := w.Write(handlerStr)
		if err != nil {
			t.Fatalf("Failed writing HTTP response: %v", err)
		}
	}).Methods("GET")

	router.Use(func(h http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			_, err := w.Write(mwStr)
			if err != nil {
				t.Fatalf("Failed writing HTTP response: %v", err)
			}
			h.ServeHTTP(w, r)
		})
	})

	t.Run("not called", func(t *testing.T) {
		rw := NewRecorder()
		req := newRequest("POST", "/")

		router.ServeHTTP(rw, req)
		if bytes.Contains(rw.Body.Bytes(), mwStr) {
			t.Fatal("Middleware was called for a method mismatch")
		}
	})

	t.Run("not called with custom method not allowed handler", func(t *testing.T) {
		rw := NewRecorder()
		req := newRequest("POST", "/")

		router.MethodNotAllowedHandler = http.HandlerFunc(func(rw http.ResponseWriter, r *http.Request) {
			_, err := rw.Write([]byte("Method not allowed"))
			if err != nil {
				t.Fatalf("Failed writing HTTP response: %v", err)
			}
		})
		router.ServeHTTP(rw, req)

		if bytes.Contains(rw.Body.Bytes(), mwStr) {
			t.Fatal("Middleware was called for a method mismatch")
		}
	})
}

func TestMiddlewareNotFoundSubrouter(t *testing.T) {
	mwStr := []byte("Middleware\n")
	handlerStr := []byte("Logic\n")

	router := NewRouter()
	router.HandleFunc("/", func(w http.ResponseWriter, e *http.Request) {
		_, err := w.Write(handlerStr)
		if err != nil {
			t.Fatalf("Failed writing HTTP response: %v", err)
		}
	})

	subrouter := router.PathPrefix("/sub/").Subrouter()
	subrouter.HandleFunc("/", func(w http.ResponseWriter, e *http.Request) {
		_, err := w.Write(handlerStr)
		if err != nil {
			t.Fatalf("Failed writing HTTP response: %v", err)
		}
	})

	router.Use(func(h http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			_, err := w.Write(mwStr)
			if err != nil {
				t.Fatalf("Failed writing HTTP response: %v", err)
			}
			h.ServeHTTP(w, r)
		})
	})

	t.Run("not called", func(t *testing.T) {
		rw := NewRecorder()
		req := newRequest("GET", "/sub/notfound")

		router.ServeHTTP(rw, req)
		if bytes.Contains(rw.Body.Bytes(), mwStr) {
			t.Fatal("Middleware was called for a 404")
		}
	})

	t.Run("not called with custom not found handler", func(t *testing.T) {
		rw := NewRecorder()
		req := newRequest("GET", "/sub/notfound")

		subrouter.NotFoundHandler = http.HandlerFunc(func(rw http.ResponseWriter, r *http.Request) {
			_, err := rw.Write([]byte("Custom 404 handler"))
			if err != nil {
				t.Fatalf("Failed writing HTTP response: %v", err)
			}
		})
		router.ServeHTTP(rw, req)

		if bytes.Contains(rw.Body.Bytes(), mwStr) {
			t.Fatal("Middleware was called for a custom 404")
		}
	})
}

func TestMiddlewareMethodMismatchSubrouter(t *testing.T) {
	mwStr := []byte("Middleware\n")
	handlerStr := []byte("Logic\n")

	router := NewRouter()
	router.HandleFunc("/", func(w http.ResponseWriter, e *http.Request) {
		_, err := w.Write(handlerStr)
		if err != nil {
			t.Fatalf("Failed writing HTTP response: %v", err)
		}
	})

	subrouter := router.PathPrefix("/sub/").Subrouter()
	subrouter.HandleFunc("/", func(w http.ResponseWriter, e *http.Request) {
		_, err := w.Write(handlerStr)
		if err != nil {
			t.Fatalf("Failed writing HTTP response: %v", err)
		}
	}).Methods("GET")

	router.Use(func(h http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			_, err := w.Write(mwStr)
			if err != nil {
				t.Fatalf("Failed writing HTTP response: %v", err)
			}
			h.ServeHTTP(w, r)
		})
	})

	t.Run("not called", func(t *testing.T) {
		rw := NewRecorder()
		req := newRequest("POST", "/sub/")

		router.ServeHTTP(rw, req)
		if bytes.Contains(rw.Body.Bytes(), mwStr) {
			t.Fatal("Middleware was called for a method mismatch")
		}
	})

	t.Run("not called with custom method not allowed handler", func(t *testing.T) {
		rw := NewRecorder()
		req := newRequest("POST", "/sub/")

		router.MethodNotAllowedHandler = http.HandlerFunc(func(rw http.ResponseWriter, r *http.Request) {
			_, err := rw.Write([]byte("Method not allowed"))
			if err != nil {
				t.Fatalf("Failed writing HTTP response: %v", err)
			}
		})
		router.ServeHTTP(rw, req)

		if bytes.Contains(rw.Body.Bytes(), mwStr) {
			t.Fatal("Middleware was called for a method mismatch")
		}
	})
}

func TestCORSMethodMiddleware(t *testing.T) {
	testCases := []struct {
		name                                    string
		registerRoutes                          func(r *Router)
		requestHeader                           http.Header
		requestMethod                           string
		requestPath                             string
		expectedAccessControlAllowMethodsHeader string
		expectedResponse                        string
	}{
		{
			name: "does not set without OPTIONS matcher",
			registerRoutes: func(r *Router) {
				r.HandleFunc("/foo", stringHandler("a")).Methods(http.MethodGet, http.MethodPut, http.MethodPatch)
			},
			requestMethod:                           "GET",
			requestPath:                             "/foo",
			expectedAccessControlAllowMethodsHeader: "",
			expectedResponse:                        "a",
		},
		{
			name: "sets on non OPTIONS",
			registerRoutes: func(r *Router) {
				r.HandleFunc("/foo", stringHandler("a")).Methods(http.MethodGet, http.MethodPut, http.MethodPatch)
				r.HandleFunc("/foo", stringHandler("b")).Methods(http.MethodOptions)
			},
			requestMethod:                           "GET",
			requestPath:                             "/foo",
			expectedAccessControlAllowMethodsHeader: "GET,PUT,PATCH,OPTIONS",
			expectedResponse:                        "a",
		},
		{
			name: "sets without preflight headers",
			registerRoutes: func(r *Router) {
				r.HandleFunc("/foo", stringHandler("a")).Methods(http.MethodGet, http.MethodPut, http.MethodPatch)
				r.HandleFunc("/foo", stringHandler("b")).Methods(http.MethodOptions)
			},
			requestMethod:                           "OPTIONS",
			requestPath:                             "/foo",
			expectedAccessControlAllowMethodsHeader: "GET,PUT,PATCH,OPTIONS",
			expectedResponse:                        "b",
		},
		{
			name: "does not set on error",
			registerRoutes: func(r *Router) {
				r.HandleFunc("/foo", stringHandler("a"))
			},
			requestMethod:                           "OPTIONS",
			requestPath:                             "/foo",
			expectedAccessControlAllowMethodsHeader: "",
			expectedResponse:                        "a",
		},
		{
			name: "sets header on valid preflight",
			registerRoutes: func(r *Router) {
				r.HandleFunc("/foo", stringHandler("a")).Methods(http.MethodGet, http.MethodPut, http.MethodPatch)
				r.HandleFunc("/foo", stringHandler("b")).Methods(http.MethodOptions)
			},
			requestMethod: "OPTIONS",
			requestPath:   "/foo",
			requestHeader: http.Header{
				"Access-Control-Request-Method":  []string{"GET"},
				"Access-Control-Request-Headers": []string{"Authorization"},
				"Origin":                         []string{"http://example.com"},
			},
			expectedAccessControlAllowMethodsHeader: "GET,PUT,PATCH,OPTIONS",
			expectedResponse:                        "b",
		},
		{
			name: "does not set methods from unmatching routes",
			registerRoutes: func(r *Router) {
				r.HandleFunc("/foo", stringHandler("c")).Methods(http.MethodDelete)
				r.HandleFunc("/foo/bar", stringHandler("a")).Methods(http.MethodGet, http.MethodPut, http.MethodPatch)
				r.HandleFunc("/foo/bar", stringHandler("b")).Methods(http.MethodOptions)
			},
			requestMethod: "OPTIONS",
			requestPath:   "/foo/bar",
			requestHeader: http.Header{
				"Access-Control-Request-Method":  []string{"GET"},
				"Access-Control-Request-Headers": []string{"Authorization"},
				"Origin":                         []string{"http://example.com"},
			},
			expectedAccessControlAllowMethodsHeader: "GET,PUT,PATCH,OPTIONS",
			expectedResponse:                        "b",
		},
	}

	for _, tt := range testCases {
		t.Run(tt.name, func(t *testing.T) {
			router := NewRouter()

			tt.registerRoutes(router)

			router.Use(CORSMethodMiddleware(router))

			rw := NewRecorder()
			req := newRequest(tt.requestMethod, tt.requestPath)
			req.Header = tt.requestHeader

			router.ServeHTTP(rw, req)

			actualMethodsHeader := rw.Header().Get("Access-Control-Allow-Methods")
			if actualMethodsHeader != tt.expectedAccessControlAllowMethodsHeader {
				t.Fatalf("Expected Access-Control-Allow-Methods to equal %s but got %s", tt.expectedAccessControlAllowMethodsHeader, actualMethodsHeader)
			}

			actualResponse := rw.Body.String()
			if actualResponse != tt.expectedResponse {
				t.Fatalf("Expected response to equal %s but got %s", tt.expectedResponse, actualResponse)
			}
		})
	}
}

func TestCORSMethodMiddlewareSubrouter(t *testing.T) {
	router := NewRouter().StrictSlash(true)

	subrouter := router.PathPrefix("/test").Subrouter()
	subrouter.HandleFunc("/hello", stringHandler("a")).Methods(http.MethodGet, http.MethodOptions, http.MethodPost)
	subrouter.HandleFunc("/hello/{name}", stringHandler("b")).Methods(http.MethodGet, http.MethodOptions)

	subrouter.Use(CORSMethodMiddleware(subrouter))

	rw := NewRecorder()
	req := newRequest("GET", "/test/hello/asdf")
	router.ServeHTTP(rw, req)

	actualMethods := rw.Header().Get("Access-Control-Allow-Methods")
	expectedMethods := "GET,OPTIONS"
	if actualMethods != expectedMethods {
		t.Fatalf("expected methods %q but got: %q", expectedMethods, actualMethods)
	}
}

func TestMiddlewareOnMultiSubrouter(t *testing.T) {
	first := "first"
	second := "second"
	notFound := "404 not found"

	router := NewRouter()
	firstSubRouter := router.PathPrefix("/").Subrouter()
	secondSubRouter := router.PathPrefix("/").Subrouter()

	router.NotFoundHandler = http.HandlerFunc(func(rw http.ResponseWriter, r *http.Request) {
		_, err := rw.Write([]byte(notFound))
		if err != nil {
			t.Fatalf("Failed writing HTTP response: %v", err)
		}
	})

	firstSubRouter.HandleFunc("/first", func(w http.ResponseWriter, r *http.Request) {

	})

	secondSubRouter.HandleFunc("/second", func(w http.ResponseWriter, r *http.Request) {

	})

	firstSubRouter.Use(func(h http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			_, err := w.Write([]byte(first))
			if err != nil {
				t.Fatalf("Failed writing HTTP response: %v", err)
			}
			h.ServeHTTP(w, r)
		})
	})

	secondSubRouter.Use(func(h http.Handler) http.Handler {
		return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			_, err := w.Write([]byte(second))
			if err != nil {
				t.Fatalf("Failed writing HTTP response: %v", err)
			}
			h.ServeHTTP(w, r)
		})
	})

	t.Run("/first uses first middleware", func(t *testing.T) {
		rw := NewRecorder()
		req := newRequest("GET", "/first")

		router.ServeHTTP(rw, req)
		if rw.Body.String() != first {
			t.Fatalf("Middleware did not run: expected %s middleware to write a response (got %s)", first, rw.Body.String())
		}
	})

	t.Run("/second uses second middleware", func(t *testing.T) {
		rw := NewRecorder()
		req := newRequest("GET", "/second")

		router.ServeHTTP(rw, req)
		if rw.Body.String() != second {
			t.Fatalf("Middleware did not run: expected %s middleware to write a response (got %s)", second, rw.Body.String())
		}
	})

	t.Run("uses not found handler", func(t *testing.T) {
		rw := NewRecorder()
		req := newRequest("GET", "/second/not-exist")

		router.ServeHTTP(rw, req)
		if rw.Body.String() != notFound {
			t.Fatalf("Notfound handler did not run: expected %s for not-exist, (got %s)", notFound, rw.Body.String())
		}
	})
}

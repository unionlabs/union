package mux_test

import (
	"fmt"
	"net/http"
	"net/http/httptest"

	"github.com/gorilla/mux"
)

func ExampleCORSMethodMiddleware() {
	r := mux.NewRouter()

	r.HandleFunc("/foo", func(w http.ResponseWriter, r *http.Request) {
		// Handle the request
	}).Methods(http.MethodGet, http.MethodPut, http.MethodPatch)
	r.HandleFunc("/foo", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Access-Control-Allow-Origin", "http://example.com")
		w.Header().Set("Access-Control-Max-Age", "86400")
	}).Methods(http.MethodOptions)

	r.Use(mux.CORSMethodMiddleware(r))

	rw := httptest.NewRecorder()
	req, _ := http.NewRequest("OPTIONS", "/foo", nil)                 // needs to be OPTIONS
	req.Header.Set("Access-Control-Request-Method", "POST")           // needs to be non-empty
	req.Header.Set("Access-Control-Request-Headers", "Authorization") // needs to be non-empty
	req.Header.Set("Origin", "http://example.com")                    // needs to be non-empty

	r.ServeHTTP(rw, req)

	fmt.Println(rw.Header().Get("Access-Control-Allow-Methods"))
	fmt.Println(rw.Header().Get("Access-Control-Allow-Origin"))
	// Output:
	// GET,PUT,PATCH,OPTIONS
	// http://example.com
}

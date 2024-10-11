// Copyright 2012 The Gorilla Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package mux

import (
	"net/http"
	"net/http/httptest"
	"testing"
)

func BenchmarkMux(b *testing.B) {
	router := new(Router)
	handler := func(w http.ResponseWriter, r *http.Request) {}
	router.HandleFunc("/v1/{v1}", handler)

	request, _ := http.NewRequest("GET", "/v1/anything", nil)
	for i := 0; i < b.N; i++ {
		router.ServeHTTP(nil, request)
	}
}

func BenchmarkMuxAlternativeInRegexp(b *testing.B) {
	router := new(Router)
	handler := func(w http.ResponseWriter, r *http.Request) {}
	router.HandleFunc("/v1/{v1:(?:a|b)}", handler)

	requestA, _ := http.NewRequest("GET", "/v1/a", nil)
	requestB, _ := http.NewRequest("GET", "/v1/b", nil)
	for i := 0; i < b.N; i++ {
		router.ServeHTTP(nil, requestA)
		router.ServeHTTP(nil, requestB)
	}
}

func BenchmarkManyPathVariables(b *testing.B) {
	router := new(Router)
	handler := func(w http.ResponseWriter, r *http.Request) {}
	router.HandleFunc("/v1/{v1}/{v2}/{v3}/{v4}/{v5}", handler)

	matchingRequest, _ := http.NewRequest("GET", "/v1/1/2/3/4/5", nil)
	notMatchingRequest, _ := http.NewRequest("GET", "/v1/1/2/3/4", nil)
	recorder := httptest.NewRecorder()
	for i := 0; i < b.N; i++ {
		router.ServeHTTP(nil, matchingRequest)
		router.ServeHTTP(recorder, notMatchingRequest)
	}
}

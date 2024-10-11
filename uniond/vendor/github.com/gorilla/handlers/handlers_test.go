// Copyright 2013 The Gorilla Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package handlers

import (
	"io"
	"log"
	"net/http"
	"net/http/httptest"
	"net/url"
	"strings"
	"testing"
)

const (
	ok         = "ok\n"
	notAllowed = "Method not allowed\n"
)

var okHandler = http.HandlerFunc(func(w http.ResponseWriter, req *http.Request) {
	_, err := w.Write([]byte(ok))
	if err != nil {
		log.Fatalf("error on writing to http.ResponseWriter: %v", err)
	}
})

func newRequest(method, url string) *http.Request {
	req, err := http.NewRequest(method, url, nil)
	if err != nil {
		panic(err)
	}
	return req
}

func TestMethodHandler(t *testing.T) {
	tests := []struct {
		req     *http.Request
		handler http.Handler
		code    int
		allow   string // Contents of the Allow header
		body    string
	}{
		// No handlers
		{newRequest(http.MethodGet, "/foo"), MethodHandler{}, http.StatusMethodNotAllowed, "", notAllowed},
		{newRequest(http.MethodOptions, "/foo"), MethodHandler{}, http.StatusOK, "", ""},

		// A single handler
		{newRequest(http.MethodGet, "/foo"), MethodHandler{http.MethodGet: okHandler}, http.StatusOK, "", ok},
		{newRequest(http.MethodPost, "/foo"), MethodHandler{http.MethodGet: okHandler}, http.StatusMethodNotAllowed, http.MethodGet, notAllowed},

		// Multiple handlers
		{newRequest(http.MethodGet, "/foo"), MethodHandler{http.MethodGet: okHandler, http.MethodPost: okHandler}, http.StatusOK, "", ok},
		{newRequest(http.MethodPost, "/foo"), MethodHandler{http.MethodGet: okHandler, http.MethodPost: okHandler}, http.StatusOK, "", ok},
		{newRequest(http.MethodDelete, "/foo"), MethodHandler{http.MethodGet: okHandler, http.MethodPost: okHandler}, http.StatusMethodNotAllowed, "GET, POST", notAllowed},
		{newRequest(http.MethodOptions, "/foo"), MethodHandler{http.MethodGet: okHandler, http.MethodPost: okHandler}, http.StatusOK, "GET, POST", ""},

		// Override OPTIONS
		{newRequest(http.MethodOptions, "/foo"), MethodHandler{http.MethodOptions: okHandler}, http.StatusOK, "", ok},
	}

	for i, test := range tests {
		rec := httptest.NewRecorder()
		test.handler.ServeHTTP(rec, test.req)
		resp := rec.Result()
		if resp.StatusCode != test.code {
			t.Fatalf("%d: wrong code, got %d want %d", i, resp.StatusCode, test.code)
		}
		if allow := resp.Header.Get("Allow"); allow != test.allow {
			t.Fatalf("%d: wrong Allow, got %s want %s", i, allow, test.allow)
		}

		respBodyBytes, err := io.ReadAll(resp.Body)
		if err != nil {
			t.Errorf("io error while reading response body %v", err)
		}
		if body := string(respBodyBytes); body != test.body {
			t.Fatalf("%d: wrong body, got %q want %q", i, body, test.body)
		}
	}
}

func TestContentTypeHandler(t *testing.T) {
	tests := []struct {
		Method            string
		AllowContentTypes []string
		ContentType       string
		Code              int
	}{
		{http.MethodPost, []string{"application/json"}, "application/json", http.StatusOK},
		{http.MethodPost, []string{"application/json", "application/xml"}, "application/json", http.StatusOK},
		{http.MethodPost, []string{"application/json"}, "application/json; charset=utf-8", http.StatusOK},
		{http.MethodPost, []string{"application/json"}, "application/json+xxx", http.StatusUnsupportedMediaType},
		{http.MethodPost, []string{"application/json"}, "text/plain", http.StatusUnsupportedMediaType},
		{http.MethodGet, []string{"application/json"}, "", http.StatusOK},
		{http.MethodGet, []string{}, "", http.StatusOK},
	}
	for _, test := range tests {
		r, err := http.NewRequest(test.Method, "/", nil)
		if err != nil {
			t.Error(err)
			continue
		}

		h := ContentTypeHandler(okHandler, test.AllowContentTypes...)
		r.Header.Set("Content-Type", test.ContentType)
		w := httptest.NewRecorder()
		h.ServeHTTP(w, r)
		if w.Code != test.Code {
			t.Errorf("expected %d, got %d", test.Code, w.Code)
		}
	}
}

func TestHTTPMethodOverride(t *testing.T) {
	tests := []struct {
		Method         string
		OverrideMethod string
		ExpectedMethod string
	}{
		{http.MethodPost, http.MethodPut, http.MethodPut},
		{http.MethodPost, http.MethodPatch, http.MethodPatch},
		{http.MethodPost, http.MethodDelete, http.MethodDelete},
		{http.MethodPut, http.MethodDelete, http.MethodPut},
		{http.MethodGet, http.MethodGet, http.MethodGet},
		{http.MethodHead, http.MethodHead, http.MethodHead},
		{http.MethodGet, http.MethodPut, http.MethodGet},
		{http.MethodHead, http.MethodDelete, http.MethodHead},
	}

	for _, test := range tests {
		h := HTTPMethodOverrideHandler(okHandler)
		reqs := make([]*http.Request, 0, 2)

		rHeader, err := http.NewRequest(test.Method, "/", nil)
		if err != nil {
			t.Error(err)
		}
		rHeader.Header.Set(HTTPMethodOverrideHeader, test.OverrideMethod)
		reqs = append(reqs, rHeader)

		f := url.Values{HTTPMethodOverrideFormKey: []string{test.OverrideMethod}}
		rForm, err := http.NewRequest(test.Method, "/", strings.NewReader(f.Encode()))
		if err != nil {
			t.Error(err)
		}
		rForm.Header.Set("Content-Type", "application/x-www-form-urlencoded")
		reqs = append(reqs, rForm)

		for _, r := range reqs {
			w := httptest.NewRecorder()
			h.ServeHTTP(w, r)
			if r.Method != test.ExpectedMethod {
				t.Errorf("Expected %s, got %s", test.ExpectedMethod, r.Method)
			}
		}
	}
}

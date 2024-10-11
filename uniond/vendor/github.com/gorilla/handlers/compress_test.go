// Copyright 2013 The Gorilla Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package handlers

import (
	"bufio"
	"bytes"
	"compress/gzip"
	"io"
	"log"
	"net"
	"net/http"
	"net/http/httptest"
	"net/url"
	"os"
	"path/filepath"
	"strconv"
	"testing"
)

var contentType = "text/plain; charset=utf-8"

func compressedRequest(w *httptest.ResponseRecorder, compression string) {
	CompressHandler(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Length", strconv.Itoa(9*1024))
		w.Header().Set("Content-Type", contentType)
		for i := 0; i < 1024; i++ {
			_, err := io.WriteString(w, "Gorilla!\n")
			if err != nil {
				log.Printf("error on writting to http.ResponseWriter: %v\n", err)
			}
		}
	})).ServeHTTP(w, &http.Request{
		Method: http.MethodGet,
		Header: http.Header{
			acceptEncoding: []string{compression},
		},
	})
}

func TestCompressHandlerNoCompression(t *testing.T) {
	w := httptest.NewRecorder()
	compressedRequest(w, "")
	resp := w.Result()
	if enc := resp.Header.Get("Content-Encoding"); enc != "" {
		t.Errorf("wrong content encoding, got %q want %q", enc, "")
	}
	if ct := resp.Header.Get("Content-Type"); ct != contentType {
		t.Errorf("wrong content type, got %q want %q", ct, contentType)
	}
	if w.Body.Len() != 1024*9 {
		t.Errorf("wrong len, got %d want %d", w.Body.Len(), 1024*9)
	}
	if l := resp.Header.Get("Content-Length"); l != "9216" {
		t.Errorf("wrong content-length. got %q expected %d", l, 1024*9)
	}
	if v := resp.Header.Get("Vary"); v != acceptEncoding {
		t.Errorf("wrong vary. got %s expected %s", v, acceptEncoding)
	}
}

func TestAcceptEncodingIsDropped(t *testing.T) {
	tCases := []struct {
		name,
		compression,
		expect string
		isPresent bool
	}{
		{
			"accept-encoding-gzip",
			"gzip",
			"",
			false,
		},
		{
			"accept-encoding-deflate",
			"deflate",
			"",
			false,
		},
		{
			"accept-encoding-gzip,deflate",
			"gzip,deflate",
			"",
			false,
		},
		{
			"accept-encoding-gzip,deflate,something",
			"gzip,deflate,something",
			"",
			false,
		},
		{
			"accept-encoding-unknown",
			"unknown",
			"unknown",
			true,
		},
	}

	for _, tCase := range tCases {
		ch := CompressHandler(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			acceptEnc := r.Header.Get(acceptEncoding)
			if acceptEnc == "" && tCase.isPresent {
				t.Fatalf("%s: expected 'Accept-Encoding' header to be present but was not", tCase.name)
			}
			if acceptEnc != "" {
				if !tCase.isPresent {
					t.Fatalf("%s: expected 'Accept-Encoding' header to be dropped but was still present having value %q", tCase.name, acceptEnc)
				}
				if acceptEnc != tCase.expect {
					t.Fatalf("%s: expected 'Accept-Encoding' to be %q but was %q", tCase.name, tCase.expect, acceptEnc)
				}
			}
		}))

		w := httptest.NewRecorder()
		ch.ServeHTTP(w, &http.Request{
			Method: http.MethodGet,
			Header: http.Header{
				acceptEncoding: []string{tCase.compression},
			},
		})
	}
}

func TestCompressHandlerGzip(t *testing.T) {
	w := httptest.NewRecorder()
	compressedRequest(w, "gzip")
	resp := w.Result()
	if resp.Header.Get("Content-Encoding") != "gzip" {
		t.Errorf("wrong content encoding, got %q want %q", resp.Header.Get("Content-Encoding"), "gzip")
	}
	if resp.Header.Get("Content-Type") != "text/plain; charset=utf-8" {
		t.Errorf("wrong content type, got %s want %s", resp.Header.Get("Content-Type"), "text/plain; charset=utf-8")
	}
	if w.Body.Len() != 72 {
		t.Errorf("wrong len, got %d want %d", w.Body.Len(), 72)
	}
	if l := resp.Header.Get("Content-Length"); l != "" {
		t.Errorf("wrong content-length. got %q expected %q", l, "")
	}
}

func TestCompressHandlerDeflate(t *testing.T) {
	w := httptest.NewRecorder()
	compressedRequest(w, "deflate")
	resp := w.Result()
	if resp.Header.Get("Content-Encoding") != "deflate" {
		t.Fatalf("wrong content encoding, got %q want %q", resp.Header.Get("Content-Encoding"), "deflate")
	}
	if resp.Header.Get("Content-Type") != "text/plain; charset=utf-8" {
		t.Fatalf("wrong content type, got %s want %s", resp.Header.Get("Content-Type"), "text/plain; charset=utf-8")
	}
	if w.Body.Len() != 54 {
		t.Fatalf("wrong len, got %d want %d", w.Body.Len(), 54)
	}
}

func TestCompressHandlerGzipDeflate(t *testing.T) {
	w := httptest.NewRecorder()
	compressedRequest(w, "gzip, deflate ")
	resp := w.Result()
	if resp.Header.Get("Content-Encoding") != "gzip" {
		t.Fatalf("wrong content encoding, got %q want %q", resp.Header.Get("Content-Encoding"), "gzip")
	}
	if resp.Header.Get("Content-Type") != "text/plain; charset=utf-8" {
		t.Fatalf("wrong content type, got %s want %s", resp.Header.Get("Content-Type"), "text/plain; charset=utf-8")
	}
}

// Make sure we can compress and serve an *os.File properly. We need
// to use a real http server to trigger the net/http sendfile special
// case.
func TestCompressFile(t *testing.T) {
	dir, err := os.MkdirTemp("", "gorilla_compress")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(dir)

	err = os.WriteFile(filepath.Join(dir, "hello.txt"), []byte("hello"), 0o644)
	if err != nil {
		t.Fatal(err)
	}

	s := httptest.NewServer(CompressHandler(http.FileServer(http.Dir(dir))))
	defer s.Close()

	url := &url.URL{Scheme: "http", Host: s.Listener.Addr().String(), Path: "/hello.txt"}
	req, err := http.NewRequest(http.MethodGet, url.String(), nil)
	if err != nil {
		t.Fatal(err)
	}
	req.Header.Set(acceptEncoding, "gzip")
	res, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatal(err)
	}

	if res.StatusCode != http.StatusOK {
		t.Fatalf("expected OK, got %q", res.Status)
	}

	var got bytes.Buffer
	gr, err := gzip.NewReader(res.Body)
	if err != nil {
		t.Fatal(err)
	}
	_, err = io.Copy(&got, gr)
	if err != nil {
		t.Fatal(err)
	}

	if got.String() != "hello" {
		t.Errorf("expected hello, got %q", got.String())
	}
}

type fullyFeaturedResponseWriter struct{}

// Header/Write/WriteHeader implement the http.ResponseWriter interface.
func (fullyFeaturedResponseWriter) Header() http.Header {
	return http.Header{}
}

func (fullyFeaturedResponseWriter) Write([]byte) (int, error) {
	return 0, nil
}
func (fullyFeaturedResponseWriter) WriteHeader(int) {}

// Flush implements the http.Flusher interface.
func (fullyFeaturedResponseWriter) Flush() {}

// Hijack implements the http.Hijacker interface.
func (fullyFeaturedResponseWriter) Hijack() (net.Conn, *bufio.ReadWriter, error) {
	return nil, nil, nil
}

func TestCompressHandlerPreserveInterfaces(t *testing.T) {
	// Compile time validation fullyFeaturedResponseWriter implements all the
	// interfaces we're asserting in the test case below.
	var (
		_ http.Flusher  = fullyFeaturedResponseWriter{}
		_ http.Hijacker = fullyFeaturedResponseWriter{}
	)
	var h http.Handler = http.HandlerFunc(func(rw http.ResponseWriter, r *http.Request) {
		comp := r.Header.Get(acceptEncoding)
		if _, ok := rw.(http.Flusher); !ok {
			t.Errorf("ResponseWriter lost http.Flusher interface for %q", comp)
		}
		if _, ok := rw.(http.Hijacker); !ok {
			t.Errorf("ResponseWriter lost http.Hijacker interface for %q", comp)
		}
	})
	h = CompressHandler(h)
	var rw fullyFeaturedResponseWriter
	r, err := http.NewRequest(http.MethodGet, "/", nil)
	if err != nil {
		t.Fatalf("Failed to create test request: %v", err)
	}
	r.Header.Set(acceptEncoding, "gzip")
	h.ServeHTTP(rw, r)

	r.Header.Set(acceptEncoding, "deflate")
	h.ServeHTTP(rw, r)
}

type paltryResponseWriter struct{}

func (paltryResponseWriter) Header() http.Header {
	return http.Header{}
}

func (paltryResponseWriter) Write([]byte) (int, error) {
	return 0, nil
}
func (paltryResponseWriter) WriteHeader(int) {}

func TestCompressHandlerDoesntInventInterfaces(t *testing.T) {
	var h http.Handler = http.HandlerFunc(func(rw http.ResponseWriter, r *http.Request) {
		if _, ok := rw.(http.Hijacker); ok {
			t.Error("ResponseWriter shouldn't implement http.Hijacker")
		}
	})

	h = CompressHandler(h)

	var rw paltryResponseWriter
	r, err := http.NewRequest(http.MethodGet, "/", nil)
	if err != nil {
		t.Fatalf("Failed to create test request: %v", err)
	}
	r.Header.Set(acceptEncoding, "gzip")
	h.ServeHTTP(rw, r)
}

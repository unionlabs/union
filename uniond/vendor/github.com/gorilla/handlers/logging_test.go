// Copyright 2013 The Gorilla Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package handlers

import (
	"bytes"
	"crypto/rand"
	"encoding/base64"
	"errors"
	"fmt"
	"io/fs"
	"mime/multipart"
	"net/http"
	"net/http/httptest"
	"net/url"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"
)

func TestMakeLogger(t *testing.T) {
	rec := httptest.NewRecorder()
	logger, w := makeLogger(rec)
	// initial status
	if logger.Status() != http.StatusOK {
		t.Fatalf("wrong status, got %d want %d", logger.Status(), http.StatusOK)
	}
	// WriteHeader
	w.WriteHeader(http.StatusInternalServerError)
	if logger.Status() != http.StatusInternalServerError {
		t.Fatalf("wrong status, got %d want %d", logger.Status(), http.StatusInternalServerError)
	}
	// Write
	_, err := w.Write([]byte(ok))
	if err != nil {
		t.Fatalf("error while writing to http.ResponseWriter %v", err)
		return
	}
	if logger.Size() != len(ok) {
		t.Fatalf("wrong size, got %d want %d", logger.Size(), len(ok))
	}
	// Header
	w.Header().Set("key", "value")
	if val := w.Header().Get("key"); val != "value" {
		t.Fatalf("wrong header, got %s want %s", val, "value")
	}
}

func TestLoggerCleanup(t *testing.T) {
	rbuf := make([]byte, 128)
	if _, err := rand.Read(rbuf); err != nil {
		t.Fatalf("Failed to generate random content: %v", err)
	}
	contents := base64.StdEncoding.EncodeToString(rbuf)

	var body bytes.Buffer
	body.WriteString(fmt.Sprintf(`
--boundary
Content-Disposition: form-data; name="buzz"; filename="example.txt"

%s
--boundary--
`, contents))
	r := multipart.NewReader(&body, "boundary")
	form, err := r.ReadForm(0) // small max memory to force flush to disk
	if err != nil {
		t.Fatalf("Failed to read multipart form: %v", err)
	}

	tmpFiles, err := os.ReadDir(os.TempDir())
	if err != nil {
		t.Fatalf("Failed to list %s: %v", os.TempDir(), err)
	}

	var tmpFile string
	for _, f := range tmpFiles {
		if !strings.HasPrefix(f.Name(), "multipart-") {
			continue
		}

		path := filepath.Join(os.TempDir(), f.Name())
		switch b, fileError := os.ReadFile(path); {
		case fileError != nil:
			t.Fatalf("Failed to read %s: %v", path, err)
		case string(b) != contents:
			continue
		default:
			tmpFile = path
		}
	}

	if tmpFile == "" {
		t.Fatal("Could not find multipart form tmp file")
	}

	req := newRequest(http.MethodGet, "/subdir/asdf")
	req.MultipartForm = form

	var buf bytes.Buffer
	handler := http.HandlerFunc(func(w http.ResponseWriter, req *http.Request) {
		req.URL.Path = "/" // simulate http.StripPrefix and friends
		w.WriteHeader(http.StatusOK)
	})
	logger := LoggingHandler(&buf, handler)
	logger.ServeHTTP(httptest.NewRecorder(), req)

	if _, osStatErr := os.Stat(tmpFile); osStatErr == nil || !errors.Is(osStatErr, fs.ErrNotExist) {
		t.Fatalf("Expected %s to not exist, got %v", tmpFile, osStatErr)
	}
}

func TestLogPathRewrites(t *testing.T) {
	var buf bytes.Buffer

	handler := http.HandlerFunc(func(w http.ResponseWriter, req *http.Request) {
		req.URL.Path = "/" // simulate http.StripPrefix and friends
		w.WriteHeader(http.StatusOK)
	})
	logger := LoggingHandler(&buf, handler)

	logger.ServeHTTP(httptest.NewRecorder(), newRequest(http.MethodGet, "/subdir/asdf"))

	if !strings.Contains(buf.String(), "GET /subdir/asdf HTTP") {
		t.Fatalf("Got log %#v, wanted substring %#v", buf.String(), "GET /subdir/asdf HTTP")
	}
}

func BenchmarkWriteLog(b *testing.B) {
	loc, err := time.LoadLocation("Europe/Warsaw")
	if err != nil {
		b.Fatalf(err.Error())
	}
	ts := time.Date(1983, 0o5, 26, 3, 30, 45, 0, loc)

	req := newRequest(http.MethodGet, "http://example.com")
	req.RemoteAddr = "192.168.100.5"

	b.ResetTimer()

	params := LogFormatterParams{
		Request:    req,
		URL:        *req.URL,
		TimeStamp:  ts,
		StatusCode: http.StatusUnauthorized,
		Size:       500,
	}

	buf := &bytes.Buffer{}

	for i := 0; i < b.N; i++ {
		buf.Reset()
		writeLog(buf, params)
	}
}

func TestLogFormatterWriteLog_Scenario1(t *testing.T) {
	formatter := writeLog
	expected := "192.168.100.5 - - [26/May/1983:03:30:45 +0200] \"GET / HTTP/1.1\" 200 100\n"
	LoggingScenario1(t, formatter, expected)
}

func TestLogFormatterCombinedLog_Scenario1(t *testing.T) {
	formatter := writeCombinedLog
	expected := "192.168.100.5 - - [26/May/1983:03:30:45 +0200] \"GET / HTTP/1.1\" 200 100 \"http://example.com\" " +
		"\"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_8_2) " +
		"AppleWebKit/537.33 (KHTML, like Gecko) Chrome/27.0.1430.0 Safari/537.33\"\n"
	LoggingScenario1(t, formatter, expected)
}

func TestLogFormatterWriteLog_Scenario2(t *testing.T) {
	formatter := writeLog
	expected := "192.168.100.5 - - [26/May/1983:03:30:45 +0200] \"CONNECT www.example.com:443 HTTP/2.0\" 200 100\n"
	LoggingScenario2(t, formatter, expected)
}

func TestLogFormatterCombinedLog_Scenario2(t *testing.T) {
	formatter := writeCombinedLog
	expected := "192.168.100.5 - - [26/May/1983:03:30:45 +0200] \"CONNECT www.example.com:443 HTTP/2.0\" 200 100 \"http://example.com\" " +
		"\"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_8_2) " +
		"AppleWebKit/537.33 (KHTML, like Gecko) Chrome/27.0.1430.0 Safari/537.33\"\n"
	LoggingScenario2(t, formatter, expected)
}

func TestLogFormatterWriteLog_Scenario3(t *testing.T) {
	formatter := writeLog
	expected := "192.168.100.5 - kamil [26/May/1983:03:30:45 +0200] \"GET / HTTP/1.1\" 401 500\n"
	LoggingScenario3(t, formatter, expected)
}

func TestLogFormatterCombinedLog_Scenario3(t *testing.T) {
	formatter := writeCombinedLog
	expected := "192.168.100.5 - kamil [26/May/1983:03:30:45 +0200] \"GET / HTTP/1.1\" 401 500 \"http://example.com\" " +
		"\"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_8_2) " +
		"AppleWebKit/537.33 (KHTML, like Gecko) Chrome/27.0.1430.0 Safari/537.33\"\n"
	LoggingScenario3(t, formatter, expected)
}

func TestLogFormatterWriteLog_Scenario4(t *testing.T) {
	formatter := writeLog
	expected := "192.168.100.5 - - [26/May/1983:03:30:45 +0200] \"GET /test?abc=hello%20world&a=b%3F HTTP/1.1\" 200 100\n"
	LoggingScenario4(t, formatter, expected)
}

func TestLogFormatterCombinedLog_Scenario5(t *testing.T) {
	formatter := writeCombinedLog
	expected := "::1 - kamil [26/May/1983:03:30:45 +0200] \"GET / HTTP/1.1\" 200 100 \"http://example.com\" " +
		"\"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_8_2) " +
		"AppleWebKit/537.33 (KHTML, like Gecko) Chrome/27.0.1430.0 Safari/537.33\"\n"
	LoggingScenario5(t, formatter, expected)
}

func LoggingScenario1(t *testing.T, formatter LogFormatter, expected string) {
	loc, err := time.LoadLocation("Europe/Warsaw")
	if err != nil {
		panic(err)
	}
	ts := time.Date(1983, 0o5, 26, 3, 30, 45, 0, loc)

	// A typical request with an OK response
	req := constructTypicalRequestOk()

	buf := new(bytes.Buffer)
	params := LogFormatterParams{
		Request:    req,
		URL:        *req.URL,
		TimeStamp:  ts,
		StatusCode: http.StatusOK,
		Size:       100,
	}

	formatter(buf, params)
	log := buf.String()

	if log != expected {
		t.Fatalf("wrong log, got %q want %q", log, expected)
	}
}

func LoggingScenario2(t *testing.T, formatter LogFormatter, expected string) {
	loc, err := time.LoadLocation("Europe/Warsaw")
	if err != nil {
		panic(err)
	}
	ts := time.Date(1983, 0o5, 26, 3, 30, 45, 0, loc)

	// CONNECT request over http/2.0
	req := constructConnectRequest()

	buf := new(bytes.Buffer)
	params := LogFormatterParams{
		Request:    req,
		URL:        *req.URL,
		TimeStamp:  ts,
		StatusCode: http.StatusOK,
		Size:       100,
	}
	formatter(buf, params)
	log := buf.String()

	if log != expected {
		t.Fatalf("wrong log, got %q want %q", log, expected)
	}
}

func LoggingScenario3(t *testing.T, formatter LogFormatter, expected string) {
	loc, err := time.LoadLocation("Europe/Warsaw")
	if err != nil {
		panic(err)
	}
	ts := time.Date(1983, 0o5, 26, 3, 30, 45, 0, loc)

	// Request with an unauthorized user
	req := constructTypicalRequestOk()
	req.URL.User = url.User("kamil")

	buf := new(bytes.Buffer)
	params := LogFormatterParams{
		Request:    req,
		URL:        *req.URL,
		TimeStamp:  ts,
		StatusCode: http.StatusUnauthorized,
		Size:       500,
	}
	formatter(buf, params)
	log := buf.String()

	if log != expected {
		t.Fatalf("wrong log, got %q want %q", log, expected)
	}
}

func LoggingScenario4(t *testing.T, formatter LogFormatter, expected string) {
	loc, err := time.LoadLocation("Europe/Warsaw")
	if err != nil {
		panic(err)
	}
	ts := time.Date(1983, 0o5, 26, 3, 30, 45, 0, loc)

	// Request with url encoded parameters
	req := constructEncodedRequest()

	buf := new(bytes.Buffer)
	params := LogFormatterParams{
		Request:    req,
		URL:        *req.URL,
		TimeStamp:  ts,
		StatusCode: http.StatusOK,
		Size:       100,
	}
	formatter(buf, params)
	log := buf.String()

	if log != expected {
		t.Fatalf("wrong log, got %q want %q", log, expected)
	}
}

func LoggingScenario5(t *testing.T, formatter LogFormatter, expected string) {
	loc, err := time.LoadLocation("Europe/Warsaw")
	if err != nil {
		panic(err)
	}
	ts := time.Date(1983, 0o5, 26, 3, 30, 45, 0, loc)

	req := constructTypicalRequestOk()
	req.URL.User = url.User("kamil")
	req.RemoteAddr = "::1"

	buf := new(bytes.Buffer)
	params := LogFormatterParams{
		Request:    req,
		URL:        *req.URL,
		TimeStamp:  ts,
		StatusCode: http.StatusOK,
		Size:       100,
	}
	formatter(buf, params)
	log := buf.String()

	if log != expected {
		t.Fatalf("wrong log, got %q want %q", log, expected)
	}
}

// A typical request with an OK response.
func constructTypicalRequestOk() *http.Request {
	req := newRequest(http.MethodGet, "http://example.com")
	req.RemoteAddr = "192.168.100.5"
	req.Header.Set("Referer", "http://example.com")
	req.Header.Set(
		"User-Agent",
		"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_8_2) AppleWebKit/537.33 "+
			"(KHTML, like Gecko) Chrome/27.0.1430.0 Safari/537.33",
	)
	return req
}

// CONNECT request over http/2.0.
func constructConnectRequest() *http.Request {
	req := &http.Request{
		Method:     http.MethodConnect,
		Host:       "www.example.com:443",
		Proto:      "HTTP/2.0",
		ProtoMajor: 2,
		ProtoMinor: 0,
		RemoteAddr: "192.168.100.5",
		Header:     http.Header{},
		URL:        &url.URL{Host: "www.example.com:443"},
	}
	req.Header.Set("Referer", "http://example.com")
	req.Header.Set(
		"User-Agent",
		"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_8_2) AppleWebKit/537.33 "+
			"(KHTML, like Gecko) Chrome/27.0.1430.0 Safari/537.33",
	)
	return req
}

func constructEncodedRequest() *http.Request {
	req := constructTypicalRequestOk()
	req.URL, _ = url.Parse("http://example.com/test?abc=hello%20world&a=b%3F")
	return req
}

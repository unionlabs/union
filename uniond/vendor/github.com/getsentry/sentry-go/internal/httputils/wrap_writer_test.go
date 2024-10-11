package httputils

import (
	"bufio"
	"bytes"
	"errors"
	"io"
	"net"
	"net/http"
	"net/http/httptest"
	"testing"
)

// CustomResponseWriter for testing http.Hijacker and http.Pusher.
type CustomResponseWriter struct {
	*httptest.ResponseRecorder
}

func (c *CustomResponseWriter) Hijack() (net.Conn, *bufio.ReadWriter, error) {
	return nil, nil, errors.New("hijack not supported in tests")
}

func (c *CustomResponseWriter) Push(string, *http.PushOptions) error {
	return nil
}

func (c *CustomResponseWriter) Flush() {
	c.ResponseRecorder.Flush()
}

func (c *CustomResponseWriter) ReadFrom(r io.Reader) (n int64, err error) {
	buf := new(bytes.Buffer)
	n, err = buf.ReadFrom(r)
	if err == nil {
		_, err = buf.WriteTo(c.ResponseRecorder)
	}
	return
}

func TestHttpFancyWriterRemembersWroteHeaderWhenFlushed(t *testing.T) {
	f := &httpFancyWriter{basicWriter: basicWriter{ResponseWriter: httptest.NewRecorder()}}
	f.Flush()

	if !f.wroteHeader {
		t.Fatal("want Flush to have set wroteHeader=true")
	}
}

func TestHttp2FancyWriterRemembersWroteHeaderWhenFlushed(t *testing.T) {
	f := &http2FancyWriter{basicWriter{ResponseWriter: httptest.NewRecorder()}}
	f.Flush()

	if !f.wroteHeader {
		t.Fatal("want Flush to have set wroteHeader=true")
	}
}

func TestBytesWritten(t *testing.T) {
	rec := httptest.NewRecorder()
	bw := &basicWriter{ResponseWriter: rec}

	body := []byte("Hello, BytesWritten!")
	_, err := bw.Write(body)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if bw.BytesWritten() != len(body) {
		t.Fatalf("expected %v bytes written, got %v", len(body), bw.BytesWritten())
	}
}

func TestUnwrap(t *testing.T) {
	rec := httptest.NewRecorder()
	bw := &basicWriter{ResponseWriter: rec}

	if bw.Unwrap() != rec {
		t.Fatal("expected Unwrap to return the original ResponseWriter")
	}
}

func TestNewWrapResponseWriter(t *testing.T) {
	rec := httptest.NewRecorder()

	// HTTP/1.1 request
	w1 := NewWrapResponseWriter(rec, 1)
	if _, ok := w1.(*flushWriter); !ok {
		t.Fatalf("expected flushWriter, got %T", w1)
	}

	// HTTP/2 request
	customRec := &CustomResponseWriter{httptest.NewRecorder()}
	w2 := NewWrapResponseWriter(customRec, 2)
	if _, ok := w2.(*http2FancyWriter); !ok {
		t.Fatalf("expected http2FancyWriter, got %T", w2)
	}
}

func TestBasicWriterWriteHeader(t *testing.T) {
	rec := httptest.NewRecorder()
	bw := &basicWriter{ResponseWriter: rec}

	bw.WriteHeader(http.StatusCreated)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected status code %v, got %v", http.StatusCreated, rec.Code)
	}
}

func TestBasicWriterWrite(t *testing.T) {
	rec := httptest.NewRecorder()
	bw := &basicWriter{ResponseWriter: rec}

	body := []byte("Hello, World!")
	n, err := bw.Write(body)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if n != len(body) {
		t.Fatalf("expected %v bytes written, got %v", len(body), n)
	}
	if rec.Body.String() != string(body) {
		t.Fatalf("expected body %v, got %v", string(body), rec.Body.String())
	}
	if bw.bytes != len(body) {
		t.Fatalf("expected %v bytes written in struct, got %v", len(body), bw.bytes)
	}
}

func TestBasicWriterTee(t *testing.T) {
	rec := httptest.NewRecorder()
	var buf bytes.Buffer
	bw := &basicWriter{ResponseWriter: rec}

	bw.Tee(&buf)
	body := []byte("Hello, Tee!")
	_, err := bw.Write(body)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if buf.String() != string(body) {
		t.Fatalf("expected tee body %v, got %v", string(body), buf.String())
	}
}

func TestFlushWriterFlush(t *testing.T) {
	rec := httptest.NewRecorder()
	fw := &flushWriter{basicWriter{ResponseWriter: rec}}

	fw.Flush()
	if !fw.wroteHeader {
		t.Fatal("want Flush to have set wroteHeader=true")
	}
}

func TestHttpFancyWriterHijack(t *testing.T) {
	rec := &CustomResponseWriter{httptest.NewRecorder()}

	f := &httpFancyWriter{basicWriter: basicWriter{ResponseWriter: rec}}
	_, _, err := f.Hijack()
	if err == nil {
		t.Fatal("expected error, got nil")
	}
}

func TestHttpFancyWriterReadFrom(t *testing.T) {
	rec := &CustomResponseWriter{httptest.NewRecorder()}
	f := &httpFancyWriter{basicWriter: basicWriter{ResponseWriter: rec}}

	body := []byte("Hello, ReadFrom!")
	r := bytes.NewReader(body)
	n, err := f.ReadFrom(r)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if int(n) != len(body) {
		t.Fatalf("expected %v bytes read, got %v", len(body), n)
	}
	if rec.Body.String() != string(body) {
		t.Fatalf("expected body %v, got %v", string(body), rec.Body.String())
	}
}

func TestHttp2FancyWriterPush(t *testing.T) {
	rec := &CustomResponseWriter{httptest.NewRecorder()}

	f := &http2FancyWriter{basicWriter: basicWriter{ResponseWriter: rec}}
	err := f.Push("/some-path", nil)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
}

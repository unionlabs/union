// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package otelhttp

import (
	"bytes"
	"context"
	"errors"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/propagation"
	"go.opentelemetry.io/otel/trace"
)

func TestTransportFormatter(t *testing.T) {
	httpMethods := []struct {
		name     string
		method   string
		expected string
	}{
		{
			"GET method",
			http.MethodGet,
			"HTTP GET",
		},
		{
			"HEAD method",
			http.MethodHead,
			"HTTP HEAD",
		},
		{
			"POST method",
			http.MethodPost,
			"HTTP POST",
		},
		{
			"PUT method",
			http.MethodPut,
			"HTTP PUT",
		},
		{
			"PATCH method",
			http.MethodPatch,
			"HTTP PATCH",
		},
		{
			"DELETE method",
			http.MethodDelete,
			"HTTP DELETE",
		},
		{
			"CONNECT method",
			http.MethodConnect,
			"HTTP CONNECT",
		},
		{
			"OPTIONS method",
			http.MethodOptions,
			"HTTP OPTIONS",
		},
		{
			"TRACE method",
			http.MethodTrace,
			"HTTP TRACE",
		},
	}

	for _, tc := range httpMethods {
		t.Run(tc.name, func(t *testing.T) {
			r, err := http.NewRequest(tc.method, "http://localhost/", nil)
			if err != nil {
				t.Fatal(err)
			}
			formattedName := "HTTP " + r.Method

			if formattedName != tc.expected {
				t.Fatalf("unexpected name: got %s, expected %s", formattedName, tc.expected)
			}
		})
	}
}

func TestTransportBasics(t *testing.T) {
	prop := propagation.TraceContext{}
	content := []byte("Hello, world!")

	ctx := context.Background()
	sc := trace.NewSpanContext(trace.SpanContextConfig{
		TraceID: trace.TraceID{0x01},
		SpanID:  trace.SpanID{0x01},
	})
	ctx = trace.ContextWithRemoteSpanContext(ctx, sc)

	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		ctx := prop.Extract(r.Context(), propagation.HeaderCarrier(r.Header))
		span := trace.SpanContextFromContext(ctx)
		if span.SpanID() != sc.SpanID() {
			t.Fatalf("testing remote SpanID: got %s, expected %s", span.SpanID(), sc.SpanID())
		}
		if _, err := w.Write(content); err != nil {
			t.Fatal(err)
		}
	}))
	defer ts.Close()

	r, err := http.NewRequestWithContext(ctx, http.MethodGet, ts.URL, nil)
	if err != nil {
		t.Fatal(err)
	}

	tr := NewTransport(http.DefaultTransport, WithPropagators(prop))

	c := http.Client{Transport: tr}
	res, err := c.Do(r)
	if err != nil {
		t.Fatal(err)
	}

	body, err := io.ReadAll(res.Body)
	if err != nil {
		t.Fatal(err)
	}

	if !bytes.Equal(body, content) {
		t.Fatalf("unexpected content: got %s, expected %s", body, content)
	}
}

func TestNilTransport(t *testing.T) {
	prop := propagation.TraceContext{}
	content := []byte("Hello, world!")

	ctx := context.Background()
	sc := trace.NewSpanContext(trace.SpanContextConfig{
		TraceID: trace.TraceID{0x01},
		SpanID:  trace.SpanID{0x01},
	})
	ctx = trace.ContextWithRemoteSpanContext(ctx, sc)

	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		ctx := prop.Extract(r.Context(), propagation.HeaderCarrier(r.Header))
		span := trace.SpanContextFromContext(ctx)
		if span.SpanID() != sc.SpanID() {
			t.Fatalf("testing remote SpanID: got %s, expected %s", span.SpanID(), sc.SpanID())
		}
		if _, err := w.Write(content); err != nil {
			t.Fatal(err)
		}
	}))
	defer ts.Close()

	r, err := http.NewRequestWithContext(ctx, http.MethodGet, ts.URL, nil)
	if err != nil {
		t.Fatal(err)
	}

	tr := NewTransport(nil, WithPropagators(prop))

	c := http.Client{Transport: tr}
	res, err := c.Do(r)
	if err != nil {
		t.Fatal(err)
	}

	body, err := io.ReadAll(res.Body)
	if err != nil {
		t.Fatal(err)
	}

	if !bytes.Equal(body, content) {
		t.Fatalf("unexpected content: got %s, expected %s", body, content)
	}
}

const readSize = 42

type readCloser struct {
	readErr, closeErr error
}

func (rc readCloser) Read(p []byte) (n int, err error) {
	return readSize, rc.readErr
}

func (rc readCloser) Close() error {
	return rc.closeErr
}

type span struct {
	trace.Span

	ended       bool
	recordedErr error

	statusCode codes.Code
	statusDesc string
}

func (s *span) End(...trace.SpanEndOption) {
	s.ended = true
}

func (s *span) RecordError(err error, _ ...trace.EventOption) {
	s.recordedErr = err
}

func (s *span) SetStatus(c codes.Code, d string) {
	s.statusCode, s.statusDesc = c, d
}

func (s *span) assert(t *testing.T, ended bool, err error, c codes.Code, d string) { // nolint: revive  // ended is not a control flag.
	if ended {
		assert.True(t, s.ended, "not ended")
	} else {
		assert.False(t, s.ended, "ended")
	}

	if err == nil {
		assert.NoError(t, s.recordedErr, "recorded an error")
	} else {
		assert.Equal(t, err, s.recordedErr)
	}

	assert.Equal(t, c, s.statusCode, "status codes not equal")
	assert.Equal(t, d, s.statusDesc, "status description not equal")
}

func TestWrappedBodyRead(t *testing.T) {
	s := new(span)
	called := false
	record := func(numBytes int64) { called = true }
	wb := &wrappedBody{span: trace.Span(s), record: record, body: readCloser{}}
	n, err := wb.Read([]byte{})
	assert.Equal(t, readSize, n, "wrappedBody returned wrong bytes")
	assert.NoError(t, err)
	s.assert(t, false, nil, codes.Unset, "")
	assert.False(t, called, "record should not have been called")
}

func TestWrappedBodyReadEOFError(t *testing.T) {
	s := new(span)
	called := false
	numRecorded := int64(0)
	record := func(numBytes int64) {
		called = true
		numRecorded = numBytes
	}
	wb := &wrappedBody{span: trace.Span(s), record: record, body: readCloser{readErr: io.EOF}}
	n, err := wb.Read([]byte{})
	assert.Equal(t, readSize, n, "wrappedBody returned wrong bytes")
	assert.Equal(t, io.EOF, err)
	s.assert(t, true, nil, codes.Unset, "")
	assert.True(t, called, "record should have been called")
	assert.Equal(t, int64(readSize), numRecorded, "record recorded wrong number of bytes")
}

func TestWrappedBodyReadError(t *testing.T) {
	s := new(span)
	called := false
	record := func(int64) { called = true }
	expectedErr := errors.New("test")
	wb := &wrappedBody{span: trace.Span(s), record: record, body: readCloser{readErr: expectedErr}}
	n, err := wb.Read([]byte{})
	assert.Equal(t, readSize, n, "wrappedBody returned wrong bytes")
	assert.Equal(t, expectedErr, err)
	s.assert(t, false, expectedErr, codes.Error, expectedErr.Error())
	assert.False(t, called, "record should not have been called")
}

func TestWrappedBodyClose(t *testing.T) {
	s := new(span)
	called := false
	record := func(int64) { called = true }
	wb := &wrappedBody{span: trace.Span(s), record: record, body: readCloser{}}
	assert.NoError(t, wb.Close())
	s.assert(t, true, nil, codes.Unset, "")
	assert.True(t, called, "record should have been called")
}

func TestWrappedBodyClosePanic(t *testing.T) {
	s := new(span)
	var body io.ReadCloser
	wb := newWrappedBody(s, func(n int64) {}, body)
	assert.NotPanics(t, func() { wb.Close() }, "nil body should not panic on close")
}

func TestWrappedBodyCloseError(t *testing.T) {
	s := new(span)
	called := false
	record := func(int64) { called = true }
	expectedErr := errors.New("test")
	wb := &wrappedBody{span: trace.Span(s), record: record, body: readCloser{closeErr: expectedErr}}
	assert.Equal(t, expectedErr, wb.Close())
	s.assert(t, true, nil, codes.Unset, "")
	assert.True(t, called, "record should have been called")
}

type readWriteCloser struct {
	readCloser

	writeErr error
}

const writeSize = 1

func (rwc readWriteCloser) Write([]byte) (int, error) {
	return writeSize, rwc.writeErr
}

func TestNewWrappedBodyReadWriteCloserImplementation(t *testing.T) {
	wb := newWrappedBody(nil, func(n int64) {}, readWriteCloser{})
	assert.Implements(t, (*io.ReadWriteCloser)(nil), wb)
}

func TestNewWrappedBodyReadCloserImplementation(t *testing.T) {
	wb := newWrappedBody(nil, func(n int64) {}, readCloser{})
	assert.Implements(t, (*io.ReadCloser)(nil), wb)

	_, ok := wb.(io.ReadWriteCloser)
	assert.False(t, ok, "wrappedBody should not implement io.ReadWriteCloser")
}

func TestWrappedBodyWrite(t *testing.T) {
	s := new(span)
	var rwc io.ReadWriteCloser
	assert.NotPanics(t, func() {
		rwc = newWrappedBody(s, func(n int64) {}, readWriteCloser{}).(io.ReadWriteCloser)
	})

	n, err := rwc.Write([]byte{})
	assert.Equal(t, writeSize, n, "wrappedBody returned wrong bytes")
	assert.NoError(t, err)
	s.assert(t, false, nil, codes.Unset, "")
}

func TestWrappedBodyWriteError(t *testing.T) {
	s := new(span)
	expectedErr := errors.New("test")
	var rwc io.ReadWriteCloser
	assert.NotPanics(t, func() {
		rwc = newWrappedBody(s,
			func(n int64) {},
			readWriteCloser{
				writeErr: expectedErr,
			}).(io.ReadWriteCloser)
	})
	n, err := rwc.Write([]byte{})
	assert.Equal(t, writeSize, n, "wrappedBody returned wrong bytes")
	assert.ErrorIs(t, err, expectedErr)
	s.assert(t, false, expectedErr, codes.Error, expectedErr.Error())
}

func TestTransportProtocolSwitch(t *testing.T) {
	// This test validates the fix to #1329.

	// Simulate a "101 Switching Protocols" response from the test server.
	response := []byte(strings.Join([]string{
		"HTTP/1.1 101 Switching Protocols",
		"Upgrade: WebSocket",
		"Connection: Upgrade",
		"", "", // Needed for extra CRLF.
	}, "\r\n"))

	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		conn, buf, err := w.(http.Hijacker).Hijack()
		require.NoError(t, err)

		_, err = buf.Write(response)
		require.NoError(t, err)
		require.NoError(t, buf.Flush())
		require.NoError(t, conn.Close())
	}))
	defer ts.Close()

	ctx := context.Background()
	r, err := http.NewRequestWithContext(ctx, http.MethodGet, ts.URL, http.NoBody)
	require.NoError(t, err)

	c := http.Client{Transport: NewTransport(http.DefaultTransport)}
	res, err := c.Do(r)
	require.NoError(t, err)
	t.Cleanup(func() { require.NoError(t, res.Body.Close()) })

	assert.Implements(t, (*io.ReadWriteCloser)(nil), res.Body, "invalid body returned for protocol switch")
}

func TestTransportOriginRequestNotModify(t *testing.T) {
	prop := propagation.TraceContext{}

	ctx := context.Background()
	sc := trace.NewSpanContext(trace.SpanContextConfig{
		TraceID: trace.TraceID{0x01},
		SpanID:  trace.SpanID{0x01},
	})
	ctx = trace.ContextWithRemoteSpanContext(ctx, sc)

	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))
	defer ts.Close()

	r, err := http.NewRequestWithContext(ctx, http.MethodGet, ts.URL, http.NoBody)
	require.NoError(t, err)

	expectedRequest := r.Clone(r.Context())

	c := http.Client{Transport: NewTransport(http.DefaultTransport, WithPropagators(prop))}
	res, err := c.Do(r)
	require.NoError(t, err)

	t.Cleanup(func() { require.NoError(t, res.Body.Close()) })

	assert.Equal(t, expectedRequest, r)
}

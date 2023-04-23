//Copyright 2017 Improbable. All Rights Reserved.
// See LICENSE for licensing terms.

package grpcweb

import (
	"bytes"
	"encoding/base64"
	"encoding/binary"
	"io"
	"net/http"
	"strings"

	"golang.org/x/net/http2"
	"google.golang.org/grpc/grpclog"
)

// grpcWebResponse implements http.ResponseWriter.
type grpcWebResponse struct {
	wroteHeaders bool
	wroteBody    bool
	headers      http.Header
	// Flush must be called on this writer before returning to ensure encoded buffer is flushed
	wrapped http.ResponseWriter

	// The standard "application/grpc" content-type will be replaced with this.
	contentType string
}

func newGrpcWebResponse(resp http.ResponseWriter, isTextFormat bool) *grpcWebResponse {
	g := &grpcWebResponse{
		headers:     make(http.Header),
		wrapped:     resp,
		contentType: grpcWebContentType,
	}
	if isTextFormat {
		g.wrapped = newBase64ResponseWriter(g.wrapped)
		g.contentType = grpcWebTextContentType
	}
	return g
}

func (w *grpcWebResponse) Header() http.Header {
	return w.headers
}

func (w *grpcWebResponse) Write(b []byte) (int, error) {
	if !w.wroteHeaders {
		w.prepareHeaders()
	}
	w.wroteBody, w.wroteHeaders = true, true
	return w.wrapped.Write(b)
}

func (w *grpcWebResponse) WriteHeader(code int) {
	w.prepareHeaders()
	w.wrapped.WriteHeader(code)
	w.wroteHeaders = true
}

func (w *grpcWebResponse) Flush() {
	if w.wroteHeaders || w.wroteBody {
		// Work around the fact that WriteHeader and a call to Flush would have caused a 200 response.
		// This is the case when there is no payload.
		flushWriter(w.wrapped)
	}
}

// prepareHeaders runs all required header copying and transformations to
// prepare the header of the wrapped response writer.
func (w *grpcWebResponse) prepareHeaders() {
	wh := w.wrapped.Header()
	copyHeader(
		wh, w.headers,
		skipKeys("trailer"),
		replaceInKeys(http2.TrailerPrefix, ""),
		replaceInVals("content-type", grpcContentType, w.contentType),
		keyCase(http.CanonicalHeaderKey),
	)
	responseHeaderKeys := headerKeys(wh)
	responseHeaderKeys = append(responseHeaderKeys, "grpc-status", "grpc-message")
	wh.Set(
		http.CanonicalHeaderKey("access-control-expose-headers"),
		strings.Join(responseHeaderKeys, ", "),
	)
}

func (w *grpcWebResponse) finishRequest(req *http.Request) {
	if w.wroteHeaders || w.wroteBody {
		w.copyTrailersToPayload()
	} else {
		w.WriteHeader(http.StatusOK)
		flushWriter(w.wrapped)
	}
}

func (w *grpcWebResponse) copyTrailersToPayload() {
	trailers := extractTrailingHeaders(w.headers, w.wrapped.Header())
	trailerBuffer := new(bytes.Buffer)
	trailers.Write(trailerBuffer)
	trailerGrpcDataHeader := []byte{1 << 7, 0, 0, 0, 0} // MSB=1 indicates this is a trailer data frame.
	binary.BigEndian.PutUint32(trailerGrpcDataHeader[1:5], uint32(trailerBuffer.Len()))
	w.wrapped.Write(trailerGrpcDataHeader)
	w.wrapped.Write(trailerBuffer.Bytes())
	flushWriter(w.wrapped)
}

func extractTrailingHeaders(src http.Header, flushed http.Header) http.Header {
	th := make(http.Header)
	copyHeader(
		th, src,
		skipKeys(append([]string{"trailer"}, headerKeys(flushed)...)...),
		replaceInKeys(http2.TrailerPrefix, ""),
		// gRPC-Web spec says that must use lower-case header/trailer names. See
		// "HTTP wire protocols" section in
		// https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-WEB.md#protocol-differences-vs-grpc-over-http2
		keyCase(strings.ToLower),
	)
	return th
}

// An http.ResponseWriter wrapper that writes base64-encoded payloads. You must call Flush()
// on this writer to ensure the base64-encoder flushes its last state.
type base64ResponseWriter struct {
	wrapped http.ResponseWriter
	encoder io.WriteCloser
}

func newBase64ResponseWriter(wrapped http.ResponseWriter) http.ResponseWriter {
	w := &base64ResponseWriter{wrapped: wrapped}
	w.newEncoder()
	return w
}

func (w *base64ResponseWriter) newEncoder() {
	w.encoder = base64.NewEncoder(base64.StdEncoding, w.wrapped)
}

func (w *base64ResponseWriter) Header() http.Header {
	return w.wrapped.Header()
}

func (w *base64ResponseWriter) Write(b []byte) (int, error) {
	return w.encoder.Write(b)
}

func (w *base64ResponseWriter) WriteHeader(code int) {
	w.wrapped.WriteHeader(code)
}

func (w *base64ResponseWriter) Flush() {
	// Flush the base64 encoder by closing it. Grpc-web permits multiple padded base64 parts:
	// https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-WEB.md
	err := w.encoder.Close()
	if err != nil {
		// Must ignore this error since Flush() is not defined as returning an error
		grpclog.Errorf("ignoring error Flushing base64 encoder: %v", err)
	}
	w.newEncoder()
	flushWriter(w.wrapped)
}

func flushWriter(w http.ResponseWriter) {
	f, ok := w.(http.Flusher)
	if !ok {
		return
	}

	f.Flush()
}

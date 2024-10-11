// Copyright 2015 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package rpc

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"math"
	"mime"
	"net/http"
	"net/url"
	"strconv"
	"sync"
	"time"
)

const (
	maxRequestContentLength = 1024 * 1024 * 5
	contentType             = "application/json"
)

// https://www.jsonrpc.org/historical/json-rpc-over-http.html#id13
var acceptedContentTypes = []string{contentType, "application/json-rpc", "application/jsonrequest"}

type httpConn struct {
	client    *http.Client
	url       string
	closeOnce sync.Once
	closeCh   chan interface{}
	mu        sync.Mutex // protects headers
	headers   http.Header
	auth      HTTPAuth
}

// httpConn implements ServerCodec, but it is treated specially by Client
// and some methods don't work. The panic() stubs here exist to ensure
// this special treatment is correct.

func (hc *httpConn) writeJSON(context.Context, interface{}, bool) error {
	panic("writeJSON called on httpConn")
}

func (hc *httpConn) peerInfo() PeerInfo {
	panic("peerInfo called on httpConn")
}

func (hc *httpConn) remoteAddr() string {
	return hc.url
}

func (hc *httpConn) readBatch() ([]*jsonrpcMessage, bool, error) {
	<-hc.closeCh
	return nil, false, io.EOF
}

func (hc *httpConn) close() {
	hc.closeOnce.Do(func() { close(hc.closeCh) })
}

func (hc *httpConn) closed() <-chan interface{} {
	return hc.closeCh
}

// HTTPTimeouts represents the configuration params for the HTTP RPC server.
type HTTPTimeouts struct {
	// ReadTimeout is the maximum duration for reading the entire
	// request, including the body.
	//
	// Because ReadTimeout does not let Handlers make per-request
	// decisions on each request body's acceptable deadline or
	// upload rate, most users will prefer to use
	// ReadHeaderTimeout. It is valid to use them both.
	ReadTimeout time.Duration

	// ReadHeaderTimeout is the amount of time allowed to read
	// request headers. The connection's read deadline is reset
	// after reading the headers and the Handler can decide what
	// is considered too slow for the body. If ReadHeaderTimeout
	// is zero, the value of ReadTimeout is used. If both are
	// zero, there is no timeout.
	ReadHeaderTimeout time.Duration

	// WriteTimeout is the maximum duration before timing out
	// writes of the response. It is reset whenever a new
	// request's header is read. Like ReadTimeout, it does not
	// let Handlers make decisions on a per-request basis.
	WriteTimeout time.Duration

	// IdleTimeout is the maximum amount of time to wait for the
	// next request when keep-alives are enabled. If IdleTimeout
	// is zero, the value of ReadTimeout is used. If both are
	// zero, ReadHeaderTimeout is used.
	IdleTimeout time.Duration
}

// DefaultHTTPTimeouts represents the default timeout values used if further
// configuration is not provided.
var DefaultHTTPTimeouts = HTTPTimeouts{
	ReadTimeout:       30 * time.Second,
	ReadHeaderTimeout: 30 * time.Second,
	WriteTimeout:      30 * time.Second,
	IdleTimeout:       120 * time.Second,
}

// DialHTTP creates a new RPC client that connects to an RPC server over HTTP.
func DialHTTP(endpoint string) (*Client, error) {
	return DialHTTPWithClient(endpoint, new(http.Client))
}

// DialHTTPWithClient creates a new RPC client that connects to an RPC server over HTTP
// using the provided HTTP Client.
//
// Deprecated: use DialOptions and the WithHTTPClient option.
func DialHTTPWithClient(endpoint string, client *http.Client) (*Client, error) {
	// Sanity check URL so we don't end up with a client that will fail every request.
	_, err := url.Parse(endpoint)
	if err != nil {
		return nil, err
	}

	var cfg clientConfig
	cfg.httpClient = client
	fn := newClientTransportHTTP(endpoint, &cfg)
	return newClient(context.Background(), &cfg, fn)
}

func newClientTransportHTTP(endpoint string, cfg *clientConfig) reconnectFunc {
	headers := make(http.Header, 2+len(cfg.httpHeaders))
	headers.Set("accept", contentType)
	headers.Set("content-type", contentType)
	for key, values := range cfg.httpHeaders {
		headers[key] = values
	}

	client := cfg.httpClient
	if client == nil {
		client = new(http.Client)
	}

	hc := &httpConn{
		client:  client,
		headers: headers,
		url:     endpoint,
		auth:    cfg.httpAuth,
		closeCh: make(chan interface{}),
	}

	return func(ctx context.Context) (ServerCodec, error) {
		return hc, nil
	}
}

func (c *Client) sendHTTP(ctx context.Context, op *requestOp, msg interface{}) error {
	hc := c.writeConn.(*httpConn)
	respBody, err := hc.doRequest(ctx, msg)
	if err != nil {
		return err
	}
	defer respBody.Close()

	var resp jsonrpcMessage
	batch := [1]*jsonrpcMessage{&resp}
	if err := json.NewDecoder(respBody).Decode(&resp); err != nil {
		return err
	}
	op.resp <- batch[:]
	return nil
}

func (c *Client) sendBatchHTTP(ctx context.Context, op *requestOp, msgs []*jsonrpcMessage) error {
	hc := c.writeConn.(*httpConn)
	respBody, err := hc.doRequest(ctx, msgs)
	if err != nil {
		return err
	}
	defer respBody.Close()

	var respmsgs []*jsonrpcMessage
	if err := json.NewDecoder(respBody).Decode(&respmsgs); err != nil {
		return err
	}
	op.resp <- respmsgs
	return nil
}

func (hc *httpConn) doRequest(ctx context.Context, msg interface{}) (io.ReadCloser, error) {
	body, err := json.Marshal(msg)
	if err != nil {
		return nil, err
	}
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, hc.url, io.NopCloser(bytes.NewReader(body)))
	if err != nil {
		return nil, err
	}
	req.ContentLength = int64(len(body))
	req.GetBody = func() (io.ReadCloser, error) { return io.NopCloser(bytes.NewReader(body)), nil }

	// set headers
	hc.mu.Lock()
	req.Header = hc.headers.Clone()
	hc.mu.Unlock()
	setHeaders(req.Header, headersFromContext(ctx))

	if hc.auth != nil {
		if err := hc.auth(req.Header); err != nil {
			return nil, err
		}
	}

	// do request
	resp, err := hc.client.Do(req)
	if err != nil {
		return nil, err
	}
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		var buf bytes.Buffer
		var body []byte
		if _, err := buf.ReadFrom(resp.Body); err == nil {
			body = buf.Bytes()
		}

		return nil, HTTPError{
			Status:     resp.Status,
			StatusCode: resp.StatusCode,
			Body:       body,
		}
	}
	return resp.Body, nil
}

// httpServerConn turns a HTTP connection into a Conn.
type httpServerConn struct {
	io.Reader
	io.Writer
	r *http.Request
}

func newHTTPServerConn(r *http.Request, w http.ResponseWriter) ServerCodec {
	body := io.LimitReader(r.Body, maxRequestContentLength)
	conn := &httpServerConn{Reader: body, Writer: w, r: r}

	encoder := func(v any, isErrorResponse bool) error {
		if !isErrorResponse {
			return json.NewEncoder(conn).Encode(v)
		}

		// It's an error response and requires special treatment.
		//
		// In case of a timeout error, the response must be written before the HTTP
		// server's write timeout occurs. So we need to flush the response. The
		// Content-Length header also needs to be set to ensure the client knows
		// when it has the full response.
		encdata, err := json.Marshal(v)
		if err != nil {
			return err
		}
		w.Header().Set("content-length", strconv.Itoa(len(encdata)))

		// If this request is wrapped in a handler that might remove Content-Length (such
		// as the automatic gzip we do in package node), we need to ensure the HTTP server
		// doesn't perform chunked encoding. In case WriteTimeout is reached, the chunked
		// encoding might not be finished correctly, and some clients do not like it when
		// the final chunk is missing.
		w.Header().Set("transfer-encoding", "identity")

		_, err = w.Write(encdata)
		if f, ok := w.(http.Flusher); ok {
			f.Flush()
		}
		return err
	}

	dec := json.NewDecoder(conn)
	dec.UseNumber()

	return NewFuncCodec(conn, encoder, dec.Decode)
}

// Close does nothing and always returns nil.
func (t *httpServerConn) Close() error { return nil }

// RemoteAddr returns the peer address of the underlying connection.
func (t *httpServerConn) RemoteAddr() string {
	return t.r.RemoteAddr
}

// SetWriteDeadline does nothing and always returns nil.
func (t *httpServerConn) SetWriteDeadline(time.Time) error { return nil }

// ServeHTTP serves JSON-RPC requests over HTTP.
func (s *Server) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	// Permit dumb empty requests for remote health-checks (AWS)
	if r.Method == http.MethodGet && r.ContentLength == 0 && r.URL.RawQuery == "" {
		w.WriteHeader(http.StatusOK)
		return
	}
	if code, err := validateRequest(r); err != nil {
		http.Error(w, err.Error(), code)
		return
	}

	// Create request-scoped context.
	connInfo := PeerInfo{Transport: "http", RemoteAddr: r.RemoteAddr}
	connInfo.HTTP.Version = r.Proto
	connInfo.HTTP.Host = r.Host
	connInfo.HTTP.Origin = r.Header.Get("Origin")
	connInfo.HTTP.UserAgent = r.Header.Get("User-Agent")
	ctx := r.Context()
	ctx = context.WithValue(ctx, peerInfoContextKey{}, connInfo)

	// All checks passed, create a codec that reads directly from the request body
	// until EOF, writes the response to w, and orders the server to process a
	// single request.
	w.Header().Set("content-type", contentType)
	codec := newHTTPServerConn(r, w)
	defer codec.close()
	s.serveSingleRequest(ctx, codec)
}

// validateRequest returns a non-zero response code and error message if the
// request is invalid.
func validateRequest(r *http.Request) (int, error) {
	if r.Method == http.MethodPut || r.Method == http.MethodDelete {
		return http.StatusMethodNotAllowed, errors.New("method not allowed")
	}
	if r.ContentLength > maxRequestContentLength {
		err := fmt.Errorf("content length too large (%d>%d)", r.ContentLength, maxRequestContentLength)
		return http.StatusRequestEntityTooLarge, err
	}
	// Allow OPTIONS (regardless of content-type)
	if r.Method == http.MethodOptions {
		return 0, nil
	}
	// Check content-type
	if mt, _, err := mime.ParseMediaType(r.Header.Get("content-type")); err == nil {
		for _, accepted := range acceptedContentTypes {
			if accepted == mt {
				return 0, nil
			}
		}
	}
	// Invalid content-type
	err := fmt.Errorf("invalid content type, only %s is supported", contentType)
	return http.StatusUnsupportedMediaType, err
}

// ContextRequestTimeout returns the request timeout derived from the given context.
func ContextRequestTimeout(ctx context.Context) (time.Duration, bool) {
	timeout := time.Duration(math.MaxInt64)
	hasTimeout := false
	setTimeout := func(d time.Duration) {
		if d < timeout {
			timeout = d
			hasTimeout = true
		}
	}

	if deadline, ok := ctx.Deadline(); ok {
		setTimeout(time.Until(deadline))
	}

	// If the context is an HTTP request context, use the server's WriteTimeout.
	httpSrv, ok := ctx.Value(http.ServerContextKey).(*http.Server)
	if ok && httpSrv.WriteTimeout > 0 {
		wt := httpSrv.WriteTimeout
		// When a write timeout is configured, we need to send the response message before
		// the HTTP server cuts connection. So our internal timeout must be earlier than
		// the server's true timeout.
		//
		// Note: Timeouts are sanitized to be a minimum of 1 second.
		// Also see issue: https://github.com/golang/go/issues/47229
		wt -= 100 * time.Millisecond
		setTimeout(wt)
	}

	return timeout, hasTimeout
}

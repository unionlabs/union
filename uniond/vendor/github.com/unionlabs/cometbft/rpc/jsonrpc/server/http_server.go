// Commons for HTTP handling
package server

import (
	"bufio"
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net"
	"net/http"
	"os"
	"runtime/debug"
	"strconv"
	"strings"
	"time"

	"golang.org/x/net/netutil"

	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/rpc/jsonrpc/types"
	cmttime "github.com/cometbft/cometbft/types/time"
)

// Config is a RPC server configuration.
type Config struct {
	// see netutil.LimitListener
	MaxOpenConnections int
	// mirrors http.Server#ReadTimeout
	ReadTimeout time.Duration
	// mirrors http.Server#WriteTimeout
	WriteTimeout time.Duration
	// MaxBodyBytes controls the maximum number of bytes the
	// server will read parsing the request body.
	MaxBodyBytes int64
	// mirrors http.Server#MaxHeaderBytes
	MaxHeaderBytes int
	// maximum number of requests in a batch request
	MaxRequestBatchSize int
}

// DefaultConfig returns a default configuration.
func DefaultConfig() *Config {
	return &Config{
		MaxOpenConnections:  0, // unlimited
		ReadTimeout:         10 * time.Second,
		WriteTimeout:        10 * time.Second,
		MaxBodyBytes:        int64(1000000), // 1MB
		MaxHeaderBytes:      1 << 20,        // same as the net/http default
		MaxRequestBatchSize: 10,             // default to max 10 requests per batch
	}
}

// Serve creates a http.Server and calls Serve with the given listener. It
// wraps handler with RecoverAndLogHandler and a handler, which limits the max
// body size to config.MaxBodyBytes.
//
// NOTE: This function blocks - you may want to call it in a go-routine.
func Serve(listener net.Listener, handler http.Handler, logger log.Logger, config *Config) error {
	logger.Info("serve", "msg", log.NewLazySprintf("Starting RPC HTTP server on %s", listener.Addr()))
	s := &http.Server{
		Handler:           PreChecksHandler(RecoverAndLogHandler(defaultHandler{h: handler}, logger), config),
		ReadTimeout:       config.ReadTimeout,
		ReadHeaderTimeout: config.ReadTimeout,
		WriteTimeout:      config.WriteTimeout,
		MaxHeaderBytes:    config.MaxHeaderBytes,
	}
	err := s.Serve(listener)
	logger.Info("RPC HTTP server stopped", "err", err)
	return err
}

// ServeTLS creates a http.Server and calls ServeTLS with the given listener,
// certFile and keyFile. It wraps handler with RecoverAndLogHandler and a
// handler, which limits the max body size to config.MaxBodyBytes.
//
// NOTE: This function blocks - you may want to call it in a go-routine.
func ServeTLS(
	listener net.Listener,
	handler http.Handler,
	certFile, keyFile string,
	logger log.Logger,
	config *Config,
) error {
	logger.Info("serve tls", "msg", log.NewLazySprintf("Starting RPC HTTPS server on %s (cert: %q, key: %q)",
		listener.Addr(), certFile, keyFile))
	s := &http.Server{
		Handler:           PreChecksHandler(RecoverAndLogHandler(defaultHandler{h: handler}, logger), config),
		ReadTimeout:       config.ReadTimeout,
		ReadHeaderTimeout: config.ReadTimeout,
		WriteTimeout:      config.WriteTimeout,
		MaxHeaderBytes:    config.MaxHeaderBytes,
	}
	err := s.ServeTLS(listener, certFile, keyFile)

	logger.Error("RPC HTTPS server stopped", "err", err)
	return err
}

// WriteRPCResponseHTTPError marshals res as JSON (with indent) and writes it
// to w.
//
// source: https://www.jsonrpc.org/historical/json-rpc-over-http.html
func WriteRPCResponseHTTPError(
	w http.ResponseWriter,
	httpCode int,
	res types.RPCResponse,
) error {
	if res.Error == nil {
		panic("tried to write http error response without RPC error")
	}

	jsonBytes, err := json.Marshal(res)
	if err != nil {
		return fmt.Errorf("json marshal: %w", err)
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(httpCode)
	_, err = w.Write(jsonBytes)
	return err
}

// WriteRPCResponseHTTP marshals res as JSON (with indent) and writes it to w.
func WriteRPCResponseHTTP(w http.ResponseWriter, res ...types.RPCResponse) error {
	return writeRPCResponseHTTP(w, []httpHeader{}, res...)
}

// WriteCacheableRPCResponseHTTP marshals res as JSON (with indent) and writes
// it to w. Adds Cache-Control to the response header and sets the expiry to
// one day.
func WriteCacheableRPCResponseHTTP(w http.ResponseWriter, res ...types.RPCResponse) error {
	return writeRPCResponseHTTP(w, []httpHeader{{"Cache-Control", "public, max-age=86400"}}, res...)
}

type httpHeader struct {
	name  string
	value string
}

func writeRPCResponseHTTP(w http.ResponseWriter, headers []httpHeader, res ...types.RPCResponse) error {
	var v any
	if len(res) == 1 {
		v = res[0]
	} else {
		v = res
	}

	jsonBytes, err := json.Marshal(v)
	if err != nil {
		return fmt.Errorf("json marshal: %w", err)
	}
	w.Header().Set("Content-Type", "application/json")
	for _, header := range headers {
		w.Header().Set(header.name, header.value)
	}
	w.WriteHeader(http.StatusOK)
	_, err = w.Write(jsonBytes)
	return err
}

// -----------------------------------------------------------------------------

// RecoverAndLogHandler wraps an HTTP handler, adding error logging.
// If the inner function panics, the outer function recovers, logs, sends an
// HTTP 500 error response.
func RecoverAndLogHandler(handler http.Handler, logger log.Logger) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// Wrap the ResponseWriter to remember the status
		rww := &responseWriterWrapper{-1, w}
		begin := cmttime.Now()

		rww.Header().Set("X-Server-Time", strconv.FormatInt(begin.Unix(), 10))

		defer func() {
			// Handle any panics in the panic handler below. Does not use the logger, since we want
			// to avoid any further panics. However, we try to return a 500, since it otherwise
			// defaults to 200 and there is no other way to terminate the connection. If that
			// should panic for whatever reason then the Go HTTP server will handle it and
			// terminate the connection - panicking is the de-facto and only way to get the Go HTTP
			// server to terminate the request and close the connection/stream:
			// https://github.com/golang/go/issues/17790#issuecomment-258481416
			if e := recover(); e != nil {
				fmt.Fprintf(os.Stderr, "Panic during RPC panic recovery: %v\n%v\n", e, string(debug.Stack()))
				w.WriteHeader(http.StatusInternalServerError)
			}
		}()

		defer func() {
			// Send a 500 error if a panic happens during a handler.
			// Without this, Chrome & Firefox were retrying aborted ajax requests,
			// at least to my localhost.
			if e := recover(); e != nil {
				// If RPCResponse
				if res, ok := e.(types.RPCResponse); ok {
					if wErr := WriteRPCResponseHTTP(rww, res); wErr != nil {
						logger.Error("failed to write response", "err", wErr)
					}
				} else {
					// Panics can contain anything, attempt to normalize it as an error.
					var err error
					switch e := e.(type) {
					case error:
						err = e
					case string:
						err = errors.New(e)
					case fmt.Stringer:
						err = errors.New(e.String())
					default:
					}

					logger.Error("panic in RPC HTTP handler", "err", e, "stack", string(debug.Stack()))

					res := types.RPCInternalError(types.JSONRPCIntID(-1), err)
					if wErr := WriteRPCResponseHTTPError(rww, http.StatusInternalServerError, res); wErr != nil {
						logger.Error("failed to write response", "err", wErr)
					}
				}
			}

			// Finally, log.
			durationMS := cmttime.Since(begin).Nanoseconds() / 1000000
			if rww.Status == -1 {
				rww.Status = 200
			}
			logger.Debug("served RPC HTTP response",
				"method", r.Method,
				"url", r.URL,
				"status", rww.Status,
				"duration", durationMS,
				"remoteAddr", r.RemoteAddr,
			)
		}()

		handler.ServeHTTP(rww, r)
	})
}

// Remember the status for logging.
type responseWriterWrapper struct {
	Status int
	http.ResponseWriter
}

func (w *responseWriterWrapper) WriteHeader(status int) {
	w.Status = status
	w.ResponseWriter.WriteHeader(status)
}

// implements http.Hijacker.
func (w *responseWriterWrapper) Hijack() (net.Conn, *bufio.ReadWriter, error) {
	return w.ResponseWriter.(http.Hijacker).Hijack()
}

type defaultHandler struct {
	h http.Handler
}

func (h defaultHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	h.h.ServeHTTP(w, r)
}

// Listen starts a new net.Listener on the given address.
// It returns an error if the address is invalid or the call to Listen() fails.
func Listen(addr string, maxOpenConnections int) (listener net.Listener, err error) {
	parts := strings.SplitN(addr, "://", 2)
	if len(parts) != 2 {
		return nil, fmt.Errorf(
			"invalid listening address %s (use fully formed addresses, including the tcp:// or unix:// prefix)",
			addr,
		)
	}
	proto, addr := parts[0], parts[1]
	listener, err = net.Listen(proto, addr)
	if err != nil {
		return nil, fmt.Errorf("failed to listen on %v: %v", addr, err)
	}
	if maxOpenConnections > 0 {
		listener = netutil.LimitListener(listener, maxOpenConnections)
	}

	return listener, nil
}

// Middleware

// PreChecksHandler is a middleware function that checks the size of batch requests and returns an error
// if it exceeds the maximum configured size. It also checks if the request body is not greater than the
// configured maximum request body bytes limit.
func PreChecksHandler(next http.Handler, config *Config) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// ensure that the current request body bytes is not greater than the configured maximum request body bytes
		r.Body = http.MaxBytesReader(w, r.Body, config.MaxBodyBytes)

		// if maxBatchSize is 0 then don't constraint the limit of requests per batch
		// It cannot be negative because the config.toml validation requires it to be
		// greater than or equal to 0
		if config.MaxRequestBatchSize > 0 {
			var requests []types.RPCRequest
			var responses []types.RPCResponse
			var err error

			data, err := io.ReadAll(r.Body)
			if err != nil {
				res := types.RPCInvalidRequestError(nil, fmt.Errorf("error reading request body: %w", err))
				_ = WriteRPCResponseHTTPError(w, http.StatusBadRequest, res)
				return
			}

			err = json.Unmarshal(data, &requests)
			// if no err it means multiple requests, check if the number of request exceeds
			// the maximum batch size configured
			if err == nil {
				// if the number of requests in batch exceed the maximum configured then return an error
				if len(requests) > config.MaxRequestBatchSize {
					res := types.RPCInvalidRequestError(nil, fmt.Errorf("batch request exceeds maximum (%d) allowed number of requests", config.MaxRequestBatchSize))
					responses = append(responses, res)
					_ = WriteRPCResponseHTTP(w, responses...)
					return
				}
			}

			// ensure the request body can be read again by other handlers
			r.Body = io.NopCloser(bytes.NewBuffer(data))
		}

		// next handler
		next.ServeHTTP(w, r)
	})
}

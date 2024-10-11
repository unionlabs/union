// Package sentryhttp provides Sentry integration for servers based on the
// net/http package.
package sentryhttp

import (
	"context"
	"net/http"
	"time"

	"github.com/getsentry/sentry-go"
	"github.com/getsentry/sentry-go/internal/httputils"
	"github.com/getsentry/sentry-go/internal/traceutils"
)

// The identifier of the HTTP SDK.
const sdkIdentifier = "sentry.go.http"

// A Handler is an HTTP middleware factory that provides integration with
// Sentry.
type Handler struct {
	repanic         bool
	waitForDelivery bool
	timeout         time.Duration
}

// Options configure a Handler.
type Options struct {
	// Repanic configures whether to panic again after recovering from a panic.
	// Use this option if you have other panic handlers or want the default
	// behavior from Go's http package, as documented in
	// https://golang.org/pkg/net/http/#Handler.
	Repanic bool
	// WaitForDelivery indicates, in case of a panic, whether to block the
	// current goroutine and wait until the panic event has been reported to
	// Sentry before repanicking or resuming normal execution.
	//
	// This option is normally not needed. Unless you need different behaviors
	// for different HTTP handlers, configure the SDK to use the
	// HTTPSyncTransport instead.
	//
	// Waiting (or using HTTPSyncTransport) is useful when the web server runs
	// in an environment that interrupts execution at the end of a request flow,
	// like modern serverless platforms.
	WaitForDelivery bool
	// Timeout for the delivery of panic events. Defaults to 2s. Only relevant
	// when WaitForDelivery is true.
	//
	// If the timeout is reached, the current goroutine is no longer blocked
	// waiting, but the delivery is not canceled.
	Timeout time.Duration
}

// New returns a new Handler. Use the Handle and HandleFunc methods to wrap
// existing HTTP handlers.
func New(options Options) *Handler {
	timeout := options.Timeout
	if timeout == 0 {
		timeout = 2 * time.Second
	}
	return &Handler{
		repanic:         options.Repanic,
		timeout:         timeout,
		waitForDelivery: options.WaitForDelivery,
	}
}

// Handle works as a middleware that wraps an existing http.Handler. A wrapped
// handler will recover from and report panics to Sentry, and provide access to
// a request-specific hub to report messages and errors.
func (h *Handler) Handle(handler http.Handler) http.Handler {
	return h.handle(handler)
}

// HandleFunc is like Handle, but with a handler function parameter for cases
// where that is convenient. In particular, use it to wrap a handler function
// literal.
//
//	http.Handle(pattern, h.HandleFunc(func (w http.ResponseWriter, r *http.Request) {
//	    // handler code here
//	}))
func (h *Handler) HandleFunc(handler http.HandlerFunc) http.HandlerFunc {
	return h.handle(handler)
}

func (h *Handler) handle(handler http.Handler) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()
		hub := sentry.GetHubFromContext(r.Context())
		if hub == nil {
			hub = sentry.CurrentHub().Clone()
			ctx = sentry.SetHubOnContext(ctx, hub)
		}

		if client := hub.Client(); client != nil {
			client.SetSDKIdentifier(sdkIdentifier)
		}

		options := []sentry.SpanOption{
			sentry.ContinueTrace(hub, r.Header.Get(sentry.SentryTraceHeader), r.Header.Get(sentry.SentryBaggageHeader)),
			sentry.WithOpName("http.server"),
			sentry.WithTransactionSource(sentry.SourceURL),
			sentry.WithSpanOrigin(sentry.SpanOriginStdLib),
		}

		transaction := sentry.StartTransaction(ctx,
			traceutils.GetHTTPSpanName(r),
			options...,
		)
		transaction.SetData("http.request.method", r.Method)

		rw := httputils.NewWrapResponseWriter(w, r.ProtoMajor)

		defer func() {
			status := rw.Status()
			transaction.Status = sentry.HTTPtoSpanStatus(status)
			transaction.SetData("http.response.status_code", status)
			transaction.Finish()
		}()

		hub.Scope().SetRequest(r)
		r = r.WithContext(transaction.Context())
		defer h.recoverWithSentry(hub, r)

		handler.ServeHTTP(rw, r)
	}
}

func (h *Handler) recoverWithSentry(hub *sentry.Hub, r *http.Request) {
	if err := recover(); err != nil {
		eventID := hub.RecoverWithContext(
			context.WithValue(r.Context(), sentry.RequestContextKey, r),
			err,
		)
		if eventID != nil && h.waitForDelivery {
			hub.Flush(h.timeout)
		}
		if h.repanic {
			panic(err)
		}
	}
}

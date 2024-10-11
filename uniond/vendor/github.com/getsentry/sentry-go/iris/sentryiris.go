//go:build go1.13
// +build go1.13

package sentryiris

import (
	"context"
	"fmt"
	"net/http"
	"time"

	"github.com/getsentry/sentry-go"
	"github.com/kataras/iris/v12"
)

// The identifier of the Iris SDK.
const (
	sdkIdentifier  = "sentry.go.iris"
	valuesKey      = "sentry"
	transactionKey = "sentry_transaction"
)

type handler struct {
	repanic         bool
	waitForDelivery bool
	timeout         time.Duration
}

type Options struct {
	// Repanic configures whether Sentry should repanic after recovery, in most cases it should be set to true,
	// as iris.Default includes it's own Recovery middleware what handles http responses.
	Repanic bool
	// WaitForDelivery configures whether you want to block the request before moving forward with the response.
	// Because Iris's default Recovery handler doesn't restart the application,
	// it's safe to either skip this option or set it to false.
	WaitForDelivery bool
	// Timeout for the event delivery requests.
	Timeout time.Duration
}

// New returns a function that satisfies iris.Handler interface
// It can be used with Use() method.
func New(options Options) iris.Handler {
	timeout := options.Timeout
	if timeout == 0 {
		timeout = 2 * time.Second
	}
	return (&handler{
		repanic:         options.Repanic,
		timeout:         timeout,
		waitForDelivery: options.WaitForDelivery,
	}).handle
}

func (h *handler) handle(ctx iris.Context) {
	hub := sentry.GetHubFromContext(ctx.Request().Context())
	if hub == nil {
		hub = sentry.CurrentHub().Clone()
	}

	if client := hub.Client(); client != nil {
		client.SetSDKIdentifier(sdkIdentifier)
	}

	r := ctx.Request()

	options := []sentry.SpanOption{
		sentry.ContinueTrace(hub, r.Header.Get(sentry.SentryTraceHeader), r.Header.Get(sentry.SentryBaggageHeader)),
		sentry.WithOpName("http.server"),
		sentry.WithTransactionSource(sentry.SourceRoute),
		sentry.WithSpanOrigin(sentry.SpanOriginIris),
	}

	currentRoute := ctx.GetCurrentRoute()

	transaction := sentry.StartTransaction(
		sentry.SetHubOnContext(ctx, hub),
		fmt.Sprintf("%s %s", currentRoute.Method(), currentRoute.Path()),
		options...,
	)

	defer func() {
		transaction.SetData("http.response.status_code", ctx.GetStatusCode())
		transaction.Status = sentry.HTTPtoSpanStatus(ctx.GetStatusCode())
		transaction.Finish()
	}()

	transaction.SetData("http.request.method", r.Method)

	hub.Scope().SetRequest(r)
	ctx.Values().Set(valuesKey, hub)
	ctx.Values().Set(transactionKey, transaction)
	defer h.recoverWithSentry(hub, r)
	ctx.Next()
}

func (h *handler) recoverWithSentry(hub *sentry.Hub, r *http.Request) {
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

// GetHubFromContext retrieves attached *sentry.Hub instance from iris.Context.
func GetHubFromContext(ctx iris.Context) *sentry.Hub {
	if hub, ok := ctx.Values().Get(valuesKey).(*sentry.Hub); ok {
		return hub
	}
	return nil
}

// GetSpanFromContext retrieves attached *sentry.Span instance from iris.Context.
// If there is no transaction on iris.Context, it will return nil.
func GetSpanFromContext(ctx iris.Context) *sentry.Span {
	if span, ok := ctx.Values().Get(transactionKey).(*sentry.Span); ok {
		return span
	}

	return nil
}

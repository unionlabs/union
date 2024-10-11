package sentryecho

import (
	"context"
	"fmt"
	"net/http"
	"time"

	"github.com/getsentry/sentry-go"
	"github.com/labstack/echo/v4"
)

// The identifier of the Echo SDK.
const sdkIdentifier = "sentry.go.echo"

const valuesKey = "sentry"
const transactionKey = "sentry_transaction"

type handler struct {
	repanic         bool
	waitForDelivery bool
	timeout         time.Duration
}

type Options struct {
	// Repanic configures whether Sentry should repanic after recovery, in most cases it should be set to true,
	// as echo includes its own Recover middleware what handles http responses.
	Repanic bool
	// WaitForDelivery configures whether you want to block the request before moving forward with the response.
	// Because Echo's Recover handler doesn't restart the application,
	// it's safe to either skip this option or set it to false.
	WaitForDelivery bool
	// Timeout for the event delivery requests.
	Timeout time.Duration
}

// New returns a function that satisfies echo.HandlerFunc interface
// It can be used with Use() methods.
func New(options Options) echo.MiddlewareFunc {
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

func (h *handler) handle(next echo.HandlerFunc) echo.HandlerFunc {
	return func(ctx echo.Context) error {
		hub := GetHubFromContext(ctx)
		if hub == nil {
			hub = sentry.CurrentHub().Clone()
		}

		if client := hub.Client(); client != nil {
			client.SetSDKIdentifier(sdkIdentifier)
		}

		r := ctx.Request()

		transactionName := r.URL.Path
		transactionSource := sentry.SourceURL

		if path := ctx.Path(); path != "" {
			transactionName = path
			transactionSource = sentry.SourceRoute
		}

		options := []sentry.SpanOption{
			sentry.ContinueTrace(hub, r.Header.Get(sentry.SentryTraceHeader), r.Header.Get(sentry.SentryBaggageHeader)),
			sentry.WithOpName("http.server"),
			sentry.WithTransactionSource(transactionSource),
			sentry.WithSpanOrigin(sentry.SpanOriginEcho),
		}

		transaction := sentry.StartTransaction(
			sentry.SetHubOnContext(r.Context(), hub),
			fmt.Sprintf("%s %s", r.Method, transactionName),
			options...,
		)

		transaction.SetData("http.request.method", r.Method)

		defer func() {
			status := ctx.Response().Status
			if err := ctx.Get("error"); err != nil {
				if httpError, ok := err.(*echo.HTTPError); ok {
					status = httpError.Code
				}
			}

			transaction.Status = sentry.HTTPtoSpanStatus(status)
			transaction.SetData("http.response.status_code", status)
			transaction.Finish()
		}()

		hub.Scope().SetRequest(r)
		ctx.Set(valuesKey, hub)
		ctx.Set(transactionKey, transaction)
		defer h.recoverWithSentry(hub, r)

		err := next(ctx)
		if err != nil {
			// Store the error so it can be used in the deferred function
			ctx.Set("error", err)
		}

		return err
	}
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

// GetHubFromContext retrieves attached *sentry.Hub instance from echo.Context.
func GetHubFromContext(ctx echo.Context) *sentry.Hub {
	if hub, ok := ctx.Get(valuesKey).(*sentry.Hub); ok {
		return hub
	}
	return nil
}

// GetSpanFromContext retrieves attached *sentry.Span instance from echo.Context.
// If there is no transaction on echo.Context, it will return nil.
func GetSpanFromContext(ctx echo.Context) *sentry.Span {
	if span, ok := ctx.Get(transactionKey).(*sentry.Span); ok {
		return span
	}
	return nil
}

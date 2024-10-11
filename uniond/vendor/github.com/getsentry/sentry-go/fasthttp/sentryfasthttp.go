package sentryfasthttp

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"time"

	"github.com/getsentry/sentry-go"
	"github.com/valyala/fasthttp"
)

type contextKey int

const (
	ContextKey = contextKey(1)
	// The identifier of the FastHTTP SDK.
	sdkIdentifier  = "sentry.go.fasthttp"
	valuesKey      = "sentry"
	transactionKey = "sentry_transaction"
)

type Handler struct {
	repanic         bool
	waitForDelivery bool
	timeout         time.Duration
}

type Options struct {
	// Repanic configures whether Sentry should repanic after recovery, in most cases it should be set to false,
	// as fasthttp doesn't include it's own Recovery handler.
	Repanic bool
	// WaitForDelivery configures whether you want to block the request before moving forward with the response.
	// Because fasthttp doesn't include it's own Recovery handler, it will restart the application,
	// and event won't be delivered otherwise.
	WaitForDelivery bool
	// Timeout for the event delivery requests.
	Timeout time.Duration
}

// New returns a struct that provides Handle method
// that satisfy fasthttp.RequestHandler interface.
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

// Handle wraps fasthttp.RequestHandler and recovers from caught panics.
func (h *Handler) Handle(handler fasthttp.RequestHandler) fasthttp.RequestHandler {
	return func(ctx *fasthttp.RequestCtx) {
		hub := GetHubFromContext(ctx)
		if hub == nil {
			hub = sentry.CurrentHub().Clone()
		}

		if client := hub.Client(); client != nil {
			client.SetSDKIdentifier(sdkIdentifier)
		}

		r := convert(ctx)

		options := []sentry.SpanOption{
			sentry.ContinueTrace(hub, r.Header.Get(sentry.SentryTraceHeader), r.Header.Get(sentry.SentryBaggageHeader)),
			sentry.WithOpName("http.server"),
			sentry.WithTransactionSource(sentry.SourceRoute),
			sentry.WithSpanOrigin(sentry.SpanOriginFastHTTP),
		}

		method := string(ctx.Method())

		transaction := sentry.StartTransaction(
			sentry.SetHubOnContext(ctx, hub),
			fmt.Sprintf("%s %s", method, string(ctx.Path())),
			options...,
		)
		defer func() {
			status := ctx.Response.StatusCode()
			transaction.Status = sentry.HTTPtoSpanStatus(status)
			transaction.SetData("http.response.status_code", status)
			transaction.Finish()
		}()

		transaction.SetData("http.request.method", method)

		scope := hub.Scope()
		scope.SetRequest(r)
		scope.SetRequestBody(ctx.Request.Body())
		ctx.SetUserValue(valuesKey, hub)
		ctx.SetUserValue(transactionKey, transaction)
		defer h.recoverWithSentry(hub, ctx)

		handler(ctx)
	}
}

func (h *Handler) recoverWithSentry(hub *sentry.Hub, ctx *fasthttp.RequestCtx) {
	if err := recover(); err != nil {
		eventID := hub.RecoverWithContext(
			context.WithValue(context.Background(), sentry.RequestContextKey, ctx),
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

// GetHubFromContext retrieves attached *sentry.Hub instance from fasthttp.RequestCtx.
func GetHubFromContext(ctx *fasthttp.RequestCtx) *sentry.Hub {
	hub := ctx.UserValue(valuesKey)
	if hub, ok := hub.(*sentry.Hub); ok {
		return hub
	}
	return nil
}

// GetSpanFromContext retrieves attached *sentry.Span instance from *fasthttp.RequestCtx.
// If there is no transaction on *fasthttp.RequestCtx, it will return nil.
func GetSpanFromContext(ctx *fasthttp.RequestCtx) *sentry.Span {
	if span, ok := ctx.UserValue(transactionKey).(*sentry.Span); ok {
		return span
	}
	return nil
}

func convert(ctx *fasthttp.RequestCtx) *http.Request {
	defer func() {
		if err := recover(); err != nil {
			sentry.Logger.Printf("%v", err)
		}
	}()

	r := new(http.Request)

	r.Method = string(ctx.Method())
	uri := ctx.URI()
	// Ignore error.
	r.URL, _ = url.Parse(fmt.Sprintf("%s://%s%s", uri.Scheme(), uri.Host(), uri.Path()))

	// Headers
	r.Header = make(http.Header)
	r.Header.Add("Host", string(ctx.Host()))
	ctx.Request.Header.VisitAll(func(key, value []byte) {
		r.Header.Add(string(key), string(value))
	})
	r.Host = string(ctx.Host())

	// Cookies
	ctx.Request.Header.VisitAllCookie(func(key, value []byte) {
		r.AddCookie(&http.Cookie{Name: string(key), Value: string(value)})
	})

	// Env
	r.RemoteAddr = ctx.RemoteAddr().String()

	// QueryString
	r.URL.RawQuery = string(ctx.URI().QueryString())

	// Body
	r.Body = io.NopCloser(bytes.NewReader(ctx.Request.Body()))

	return r
}

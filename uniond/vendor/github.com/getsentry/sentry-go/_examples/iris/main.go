//go:build go1.13
// +build go1.13

package main

import (
	"context"
	"fmt"
	"net/http"

	"github.com/getsentry/sentry-go"
	sentryiris "github.com/getsentry/sentry-go/iris"
	"github.com/kataras/iris/v12"
)

func main() {
	_ = sentry.Init(sentry.ClientOptions{
		Dsn: "",
		BeforeSend: func(event *sentry.Event, hint *sentry.EventHint) *sentry.Event {
			if hint.Context != nil {
				if req, ok := hint.Context.Value(sentry.RequestContextKey).(*http.Request); ok {
					// You have access to the original Request
					fmt.Println(req)
				}
			}
			fmt.Println(event)
			return event
		},
		Debug:            true,
		AttachStacktrace: true,
	})

	app := iris.Default()

	app.Use(sentryiris.New(sentryiris.Options{
		Repanic: true,
	}))

	app.Use(func(ctx iris.Context) {
		if hub := sentryiris.GetHubFromContext(ctx); hub != nil {
			hub.Scope().SetTag("someRandomTag", "maybeYouNeedIt")
		}
		ctx.Next()
	})

	app.Get("/", func(ctx iris.Context) {
		if hub := sentryiris.GetHubFromContext(ctx); hub != nil {
			hub.WithScope(func(scope *sentry.Scope) {
				scope.SetExtra("unwantedQuery", "someQueryDataMaybe")
				hub.CaptureMessage("User provided unwanted query string, but we recovered just fine")
			})
		}

		expensiveThing := func(ctx context.Context) {
			span := sentry.StartSpan(ctx, "expensive_thing")
			defer span.Finish()

			// do resource intensive thing
		}

		// Acquire transaction on current hub that's created by the SDK.
		// Be careful, it might be a nil value if you didn't set up sentryiris middleware.
		sentrySpan := sentryiris.GetSpanFromContext(ctx)
		// Pass in the `.Context()` method from `*sentry.Span` struct.
		// The `context.Context` instance inherits the context from `iris.Context`.
		expensiveThing(sentrySpan.Context())

		ctx.StatusCode(http.StatusOK)
	})

	app.Get("/foo", func(ctx iris.Context) {
		// sentryiris handler will catch it just fine, and because we attached "someRandomTag"
		// in the middleware before, it will be sent through as well
		panic("y tho")
	})

	_ = app.Run(iris.Addr(":3000"))
}

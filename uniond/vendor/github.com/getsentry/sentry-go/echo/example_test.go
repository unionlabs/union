package sentryecho_test

import (
	"context"
	"net/http"

	"github.com/getsentry/sentry-go"
	sentryecho "github.com/getsentry/sentry-go/echo"
	"github.com/labstack/echo/v4"
)

func ExampleGetSpanFromContext() {
	router := echo.New()
	router.Use(sentryecho.New(sentryecho.Options{}))
	router.GET("/", func(c echo.Context) error {
		expensiveThing := func(ctx context.Context) error {
			span := sentry.StartTransaction(ctx, "expensive_thing")
			defer span.Finish()
			// do resource intensive thing
			return nil
		}

		// Acquire transaction on current hub that's created by the SDK.
		// Be careful, it might be a nil value if you didn't set up sentryecho middleware.
		sentrySpan := sentryecho.GetSpanFromContext(c)
		// Pass in the `.Context()` method from `*sentry.Span` struct.
		// The `context.Context` instance inherits the context from `echo.Context`.
		err := expensiveThing(sentrySpan.Context())
		if err != nil {
			return err
		}

		return c.NoContent(http.StatusOK)
	})
}

<p align="center">
  <a href="https://sentry.io" target="_blank" align="center">
    <img src="https://sentry-brand.storage.googleapis.com/sentry-logo-black.png" width="280">
  </a>
  <br />
</p>

# Official Sentry Echo Handler for Sentry-go SDK

**go.dev:** https://pkg.go.dev/github.com/getsentry/sentry-go/echo

**Example:** https://github.com/getsentry/sentry-go/tree/master/_examples/echo

## Installation

```sh
go get github.com/getsentry/sentry-go/echo
```

```go
import (
    "fmt"
    "net/http"

    "github.com/getsentry/sentry-go"
    sentryecho "github.com/getsentry/sentry-go/echo"
    "github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"
)

// To initialize Sentry's handler, you need to initialize Sentry itself beforehand
if err := sentry.Init(sentry.ClientOptions{
    Dsn: "your-public-dsn",
}); err != nil {
    fmt.Printf("Sentry initialization failed: %v\n", err)
}

// Then create your app
app := echo.New()

app.Use(middleware.Logger())
app.Use(middleware.Recover())

// Once it's done, you can attach the handler as one of your middleware
app.Use(sentryecho.New(sentryecho.Options{}))

// Set up routes
app.GET("/", func(ctx echo.Context) error {
    return ctx.String(http.StatusOK, "Hello, World!")
})

// And run it
app.Logger.Fatal(app.Start(":3000"))
```

## Configuration

`sentryecho` accepts a struct of `Options` that allows you to configure how the handler will behave.

Currently it respects 3 options:

```go
// Repanic configures whether Sentry should repanic after recovery, in most cases it should be set to true,
// as echo includes its own Recover middleware that handles http responses.
Repanic bool
// WaitForDelivery configures whether you want to block the request before moving forward with the response.
// Because Echo's `Recover` handler doesn't restart the application,
// it's safe to either skip this option or set it to `false`.
WaitForDelivery bool
// Timeout for the event delivery requests.
Timeout time.Duration
```

## Usage

`sentryecho` attaches an instance of `*sentry.Hub` (https://pkg.go.dev/github.com/getsentry/sentry-go#Hub) to the `echo.Context`, which makes it available throughout the rest of the request's lifetime.
You can access it by using the `sentryecho.GetHubFromContext()` method on the context itself in any of your proceeding middleware and routes.
And it should be used instead of the global `sentry.CaptureMessage`, `sentry.CaptureException`, or any other calls, as it keeps the separation of data between the requests.

**Keep in mind that `*sentry.Hub` won't be available in middleware attached before to `sentryecho`!**

```go
app := echo.New()

app.Use(middleware.Logger())
app.Use(middleware.Recover())

app.Use(sentryecho.New(sentryecho.Options{
	Repanic: true,
}))

app.Use(func(next echo.HandlerFunc) echo.HandlerFunc {
	return func(ctx echo.Context) error {
		if hub := sentryecho.GetHubFromContext(ctx); hub != nil {
			hub.Scope().SetTag("someRandomTag", "maybeYouNeedIt")
		}
		return next(ctx)
	}
})

app.GET("/", func(ctx echo.Context) error {
	if hub := sentryecho.GetHubFromContext(ctx); hub != nil {
		hub.WithScope(func(scope *sentry.Scope) {
			scope.SetExtra("unwantedQuery", "someQueryDataMaybe")
			hub.CaptureMessage("User provided unwanted query string, but we recovered just fine")
		})
	}
	return ctx.String(http.StatusOK, "Hello, World!")
})

app.GET("/foo", func(ctx echo.Context) error {
	// sentryecho handler will catch it just fine. Also, because we attached "someRandomTag"
	// in the middleware before, it will be sent through as well
	panic("y tho")
})

app.Logger.Fatal(app.Start(":3000"))
```

### Accessing Request in `BeforeSend` callback

```go
sentry.Init(sentry.ClientOptions{
    Dsn: "your-public-dsn",
    BeforeSend: func(event *sentry.Event, hint *sentry.EventHint) *sentry.Event {
        if hint.Context != nil {
            if req, ok := hint.Context.Value(sentry.RequestContextKey).(*http.Request); ok {
                // You have access to the original Request here
            }
        }

        return event
    },
})
```

<p align="center">
  <a href="https://sentry.io" target="_blank" align="center">
    <img src="https://sentry-brand.storage.googleapis.com/sentry-logo-black.png" width="280">
  </a>
  <br />
</p>

# Official Sentry fiber Handler for Sentry-go SDK

**Godoc:** https://godoc.org/github.com/getsentry/sentry-go/fiber

**Example:** https://github.com/getsentry/sentry-go/tree/master/example/fiber

## Installation

```sh
go get github.com/getsentry/sentry-go/fiber
```

```go
import (
	"fmt"
	"github.com/gofiber/fiber/v2"
	"github.com/getsentry/sentry-go"
	sentryfiber "github.com/getsentry/sentry-go/fiber"
	"github.com/gofiber/fiber/v2/utils"
)
```

To initialize Sentry's handler, you need to initialize Sentry itself beforehand

```go
if err := sentry.Init(sentry.ClientOptions{
	Dsn: "your-public-dsn",
}); err != nil {
	fmt.Printf("Sentry initialization failed: %v\n", err)
}

// Create an instance of sentryfiber
sentryHandler := sentryfiber.New(sentryfiber.Options{})

// Once it's done, you can attach the handler as one of your middlewares
app := fiber.New()

app.Use(sentryHandler)

// And run it
app.Listen(":3000")
```

## Configuration

`sentryfiber` accepts a struct of `Options` that allows you to configure how the handler will behave.

Currently it respects 3 options:

```go
// Repanic configures whether Sentry should repanic after recovery, in most cases it should be set to false,
// as fasthttp doesn't include it's own Recovery handler.
Repanic bool
// WaitForDelivery configures whether you want to block the request before moving forward with the response.
// Because fasthttp doesn't include it's own `Recovery` handler, it will restart the application,
// and event won't be delivered otherwise.
WaitForDelivery bool
// Timeout for the event delivery requests.
Timeout time.Duration
```

## Usage

`sentryfiber` attaches an instance of `*sentry.Hub` (https://godoc.org/github.com/getsentry/sentry-go#Hub) to the request's context, which makes it available throughout the rest of the request's lifetime.
You can access it by using the `sentryfiber.GetHubFromContext()` method on the context itself in any of your proceeding middleware and routes.
And it should be used instead of the global `sentry.CaptureMessage`, `sentry.CaptureException`, or any other calls, as it keeps the separation of data between the requests.

**Keep in mind that `*sentry.Hub` won't be available in middleware attached before to `sentryfiber`!**

```go
// Later in the code
sentryHandler := sentryfiber.New(sentryfiber.Options{
    Repanic:         true,
    WaitForDelivery: true,
})

enhanceSentryEvent := func(ctx *fiber.Ctx) {
    if hub := sentryfiber.GetHubFromContext(ctx); hub != nil {
        hub.Scope().SetTag("someRandomTag", "maybeYouNeedIt")
    }
    ctx.Next()
}

app := fiber.New()

app.Use(sentryHandler)

app.All("/foo", enhanceSentryEvent, func(ctx *fiber.Ctx) {
    panic("y tho")
})

app.All("/", func(ctx *fiber.Ctx) {
    if hub := sentryfiber.GetHubFromContext(ctx); hub != nil {
        hub.WithScope(func(scope *sentry.Scope) {
            scope.SetExtra("unwantedQuery", "someQueryDataMaybe")
            hub.CaptureMessage("User provided unwanted query string, but we recovered just fine")
        })
    }
    ctx.Status(fiber.StatusOK)
})

app.Listen(":3000")
```

### Accessing Context in `BeforeSend` callback

```go
sentry.Init(sentry.ClientOptions{
	Dsn: "your-public-dsn",
	BeforeSend: func(event *sentry.Event, hint *sentry.EventHint) *sentry.Event {
		if hint.Context != nil {
			if ctx, ok := hint.Context.Value(sentry.RequestContextKey).(*fiber.Ctx); ok {
				// You have access to the original Context if it panicked
				fmt.Println(ctx.Hostname())
			}
		}
		return event
	},
})
```

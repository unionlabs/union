<p align="center">
  <a href="https://sentry.io" target="_blank" align="center">
    <img src="https://sentry-brand.storage.googleapis.com/sentry-logo-black.png" width="280">
  </a>
  <br />
</p>

# Official Sentry fasthttp Handler for Sentry-go SDK

**go.dev:** https://pkg.go.dev/github.com/getsentry/sentry-go/fasthttp

**Example:** https://github.com/getsentry/sentry-go/tree/master/_examples/fasthttp

## Installation

```sh
go get github.com/getsentry/sentry-go/fasthttp
```

```go
import (
	"fmt"
	"net/http"

	"github.com/getsentry/sentry-go"
	sentryfasthttp "github.com/getsentry/sentry-go/fasthttp"
)

// To initialize Sentry's handler, you need to initialize Sentry itself beforehand
if err := sentry.Init(sentry.ClientOptions{
	Dsn: "your-public-dsn",
}); err != nil {
	fmt.Printf("Sentry initialization failed: %v\n", err)
}

// Create an instance of sentryfasthttp
sentryHandler := sentryfasthttp.New(sentryfasthttp.Options{})

// Once it's done, you can attach the handler as one of your middlewares
fastHTTPHandler := sentryHandler.Handle(func(ctx *fasthttp.RequestCtx) {
	panic("y tho")
})

fmt.Println("Listening and serving HTTP on :3000")

// And run it
if err := fasthttp.ListenAndServe(":3000", fastHTTPHandler); err != nil {
	panic(err)
}
```

## Configuration

`sentryfasthttp` accepts a struct of `Options` that allows you to configure how the handler will behave.

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

`sentryfasthttp` attaches an instance of `*sentry.Hub` (https://pkg.go.dev/github.com/getsentry/sentry-go#Hub) to the request's context, which makes it available throughout the rest of the request's lifetime.
You can access it by using the `sentryfasthttp.GetHubFromContext()` method on the context itself in any of your proceeding middleware and routes.
And it should be used instead of the global `sentry.CaptureMessage`, `sentry.CaptureException`, or any other calls, as it keeps the separation of data between the requests.

**Keep in mind that `*sentry.Hub` won't be available in middleware attached before to `sentryfasthttp`!**

```go
func enhanceSentryEvent(handler fasthttp.RequestHandler) fasthttp.RequestHandler {
	return func(ctx *fasthttp.RequestCtx) {
		if hub := sentryfasthttp.GetHubFromContext(ctx); hub != nil {
			hub.Scope().SetTag("someRandomTag", "maybeYouNeedIt")
		}
		handler(ctx)
	}
}

// Later in the code
sentryHandler := sentryfasthttp.New(sentryfasthttp.Options{
	Repanic: true,
	WaitForDelivery: true,
})

defaultHandler := func(ctx *fasthttp.RequestCtx) {
	if hub := sentryfasthttp.GetHubFromContext(ctx); hub != nil {
		hub.WithScope(func(scope *sentry.Scope) {
			scope.SetExtra("unwantedQuery", "someQueryDataMaybe")
			hub.CaptureMessage("User provided unwanted query string, but we recovered just fine")
		})
	}
	ctx.SetStatusCode(fasthttp.StatusOK)
}

fooHandler := enhanceSentryEvent(func(ctx *fasthttp.RequestCtx) {
	panic("y tho")
})

fastHTTPHandler := func(ctx *fasthttp.RequestCtx) {
	switch string(ctx.Path()) {
	case "/foo":
		fooHandler(ctx)
	default:
		defaultHandler(ctx)
	}
}

fmt.Println("Listening and serving HTTP on :3000")

if err := fasthttp.ListenAndServe(":3000", sentryHandler.Handle(fastHTTPHandler)); err != nil {
	panic(err)
}
```

### Accessing Context in `BeforeSend` callback

```go
sentry.Init(sentry.ClientOptions{
	Dsn: "your-public-dsn",
	BeforeSend: func(event *sentry.Event, hint *sentry.EventHint) *sentry.Event {
		if hint.Context != nil {
			if ctx, ok := hint.Context.Value(sentry.RequestContextKey).(*fasthttp.RequestCtx); ok {
				// You have access to the original Context if it panicked
				fmt.Println(string(ctx.Request.Host()))
			}
		}
		return event
	},
})
```

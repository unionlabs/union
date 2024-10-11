<p align="center">
  <a href="https://sentry.io" target="_blank" align="center">
    <img src="https://sentry-brand.storage.googleapis.com/sentry-logo-black.png" width="280">
  </a>
  <br />
</p>

# Official Sentry [Iris](https://github.com/kataras/iris) Handler for Sentry-go SDK

**go.dev:** https://pkg.go.dev/github.com/getsentry/sentry-go/iris

**Example:** https://github.com/getsentry/sentry-go/tree/master/_examples/iris

## Installation

```sh
go get github.com/getsentry/sentry-go/iris
```

```go
import (
    "fmt"

    "github.com/getsentry/sentry-go"
    sentryiris "github.com/getsentry/sentry-go/iris"
    "github.com/kataras/iris/v12"
)

// To initialize Sentry's handler, you need to initialize Sentry itself beforehand
if err := sentry.Init(sentry.ClientOptions{
    Dsn: "your-public-dsn",
}); err != nil {
    fmt.Printf("Sentry initialization failed: %v\n", err)
}

// Then create your app
app := iris.Default()

// Once it's done, you can attach the handler as one of your middleware
app.Use(sentryiris.New(sentryiris.Options{}))

// Set up routes
app.Get("/", func(ctx iris.Context) {
    ctx.Writef("Hello world!")
})

// And run it
app.Run(iris.Addr(":3000"))
```

## Configuration

`sentryiris` accepts a struct of `Options` that allows you to configure how the handler will behave.

Currently it respects 3 options:

```go
// Whether Sentry should repanic after recovery, in most cases it should be set to true,
// as iris.Default includes its own Recovery middleware that handles http responses.
Repanic         bool
// Whether you want to block the request before moving forward with the response.
// Because Iris's default `Recovery` handler doesn't restart the application,
// it's safe to either skip this option or set it to `false`.
WaitForDelivery bool
// Timeout for the event delivery requests.
Timeout         time.Duration
```

## Usage

`sentryiris` attaches an instance of `*sentry.Hub` (https://pkg.go.dev/github.com/getsentry/sentry-go#Hub) to the `iris.Context`, which makes it available throughout the rest of the request's lifetime.
You can access it by using the `sentryiris.GetHubFromContext()` method on the context itself in any of your proceeding middleware and routes.
And it should be used instead of the global `sentry.CaptureMessage`, `sentry.CaptureException`, or any other calls, as it keeps the separation of data between the requests.

**Keep in mind that `*sentry.Hub` won't be available in middleware attached before to `sentryiris`!**

```go
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
})

app.Get("/foo", func(ctx iris.Context) {
    // sentryiris handler will catch it just fine. Also, because we attached "someRandomTag"
    // in the middleware before, it will be sent through as well
    panic("y tho")
})

app.Run(iris.Addr(":3000"))
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

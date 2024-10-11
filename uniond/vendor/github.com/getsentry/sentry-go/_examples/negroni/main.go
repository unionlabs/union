package main

import (
	"fmt"
	"net/http"

	sentrynegroni "github.com/getsentry/sentry-go/negroni"

	"github.com/getsentry/sentry-go"
	"github.com/urfave/negroni"
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

	app := negroni.Classic()

	app.Use(sentrynegroni.New(sentrynegroni.Options{
		Repanic: true,
	}))

	app.Use(negroni.HandlerFunc(func(rw http.ResponseWriter, r *http.Request, next http.HandlerFunc) {
		hub := sentry.GetHubFromContext(r.Context())
		hub.Scope().SetTag("someRandomTag", "maybeYouNeedIt")
		next(rw, r)
	}))

	mux := http.NewServeMux()

	mux.HandleFunc("/", func(rw http.ResponseWriter, r *http.Request) {
		hub := sentry.GetHubFromContext(r.Context())
		hub.WithScope(func(scope *sentry.Scope) {
			scope.SetExtra("unwantedQuery", "someQueryDataMaybe")
			hub.CaptureMessage("User provided unwanted query string, but we recovered just fine")
		})
		rw.WriteHeader(http.StatusOK)
	})

	mux.HandleFunc("/foo", func(rw http.ResponseWriter, r *http.Request) {
		// sentrynagroni handler will catch it just fine, and because we attached "someRandomTag"
		// in the middleware before, it will be sent through as well
		panic("y tho")
	})

	app.UseHandler(mux)

	_ = http.ListenAndServe(":3000", app)
}

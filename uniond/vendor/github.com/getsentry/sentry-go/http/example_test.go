package sentryhttp_test

import (
	"net/http"
	"time"

	"github.com/getsentry/sentry-go"
	sentryhttp "github.com/getsentry/sentry-go/http"
)

// For a longer and executable example, see
// https://github.com/getsentry/sentry-go/tree/master/_examples/http.
func Example() {
	// Initialize the Sentry SDK once in the main function.
	// sentry.Init(...)

	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		// Use GetHubFromContext to get a hub associated with the current
		// request. Hubs provide data isolation, such that tags, breadcrumbs
		// and other attributes are never mixed up across requests.
		hub := sentry.GetHubFromContext(r.Context())
		_, err := http.Get("example.com")
		if err != nil {
			hub.CaptureException(err)
		}
	})

	// Wrap the default mux with Sentry to capture panics and report errors.
	//
	// Alternatively, you can also wrap individual handlers if you need to use
	// different options for different parts of your app.
	handler := sentryhttp.New(sentryhttp.Options{}).Handle(http.DefaultServeMux)

	server := http.Server{
		Addr:              ":0",
		ReadHeaderTimeout: 3 * time.Second,
		Handler:           handler,
	}
	server.ListenAndServe()
}

package sentry_test

import (
	"fmt"
	"net/http"
	"net/http/httputil"
	"os"
	"time"

	"github.com/getsentry/sentry-go"
)

// TransportWithHooks is an http.RoundTripper that wraps an existing
// http.RoundTripper adding hooks that run before and after each round trip.
type TransportWithHooks struct {
	http.RoundTripper
	Before func(*http.Request) error
	After  func(*http.Request, *http.Response, error) (*http.Response, error)
}

func (t *TransportWithHooks) RoundTrip(req *http.Request) (*http.Response, error) {
	if err := t.Before(req); err != nil {
		return nil, err
	}
	resp, err := t.RoundTripper.RoundTrip(req)
	return t.After(req, resp, err)
}

// Initializing the SDK with a custom HTTP transport gives a lot of flexibility
// to inspect requests and responses. This example adds before and after hooks.
func Example_transportWithHooks() {
	err := sentry.Init(sentry.ClientOptions{
		// Either set your DSN here or set the SENTRY_DSN environment variable.
		Dsn:   "",
		Debug: true,
		HTTPTransport: &TransportWithHooks{
			RoundTripper: http.DefaultTransport,
			Before: func(req *http.Request) error {
				if b, err := httputil.DumpRequestOut(req, true); err != nil {
					fmt.Println(err)
				} else {
					fmt.Printf("%s\n", b)
				}
				return nil
			},
			After: func(req *http.Request, resp *http.Response, err error) (*http.Response, error) {
				if b, err := httputil.DumpResponse(resp, true); err != nil {
					fmt.Println(err)
				} else {
					fmt.Printf("%s\n", b)
				}
				return resp, err
			},
		},
	})
	if err != nil {
		fmt.Fprintf(os.Stderr, "sentry.Init: %s\n", err)
		os.Exit(1)
	}
	defer sentry.Flush(2 * time.Second)

	sentry.CaptureMessage("test")
}

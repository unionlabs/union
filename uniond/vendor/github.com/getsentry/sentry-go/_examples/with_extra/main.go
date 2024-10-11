package main

import (
	"encoding/json"
	"fmt"
	"time"

	"github.com/getsentry/sentry-go"
)

func prettyPrint(v interface{}) string {
	pp, _ := json.MarshalIndent(v, "", "  ")
	return string(pp)
}

type devNullTransport struct{}

func (t *devNullTransport) Configure(options sentry.ClientOptions) {
	dsn, _ := sentry.NewDsn(options.Dsn)
	fmt.Println()
	fmt.Println("Store Endpoint:", dsn.StoreAPIURL())
	fmt.Println("Headers:", dsn.RequestHeaders())
	fmt.Println()
}
func (t *devNullTransport) SendEvent(event *sentry.Event) {
	fmt.Println("Faked Transport")
}

func (t *devNullTransport) Flush(timeout time.Duration) bool {
	return true
}

type CustomComplexError struct {
	Message  string
	MoreData map[string]string
}

func (e CustomComplexError) Error() string {
	return "CustomComplexError: " + e.Message
}

func (e CustomComplexError) GimmeMoreData() map[string]string {
	return e.MoreData
}

type ExtractExtra struct{}

func (ee ExtractExtra) Name() string {
	return "ExtractExtra"
}

func (ee ExtractExtra) SetupOnce(client *sentry.Client) {
	client.AddEventProcessor(func(event *sentry.Event, hint *sentry.EventHint) *sentry.Event {
		if ex, ok := hint.OriginalException.(CustomComplexError); ok {
			for key, val := range ex.GimmeMoreData() {
				event.Extra[key] = val
			}
		}

		return event
	})
}

func main() {
	if err := sentry.Init(sentry.ClientOptions{
		Debug: true,
		Dsn:   "https://hello@example.com/1337",
		BeforeSend: func(event *sentry.Event, hint *sentry.EventHint) *sentry.Event {
			// Solution 1 (use beforeSend, which will be applied to
			// error events and is usually application specific):
			if ex, ok := hint.OriginalException.(CustomComplexError); ok {
				for key, val := range ex.GimmeMoreData() {
					event.Extra[key] = val
				}
			}

			fmt.Printf("%s\n\n", prettyPrint(event.Extra))

			return event
		},
		Transport: &devNullTransport{},

		// Solution 2 (use custom integration, which will be
		// applied to all events, can be extracted as a
		// separate utility, and reused across projects):
		Integrations: func(integrations []sentry.Integration) []sentry.Integration {
			return append(integrations, new(ExtractExtra))
		},
	}); err != nil {
		panic(err)
	}

	// Solution 3 and 4 (use scope event processors, which can be either
	// applied to all events - if used with ConfigureScope or per
	// event/block if used with WithScope):
	sentry.ConfigureScope(func(scope *sentry.Scope) {
		scope.AddEventProcessor(func(event *sentry.Event, hint *sentry.EventHint) *sentry.Event {
			if ex, ok := hint.OriginalException.(CustomComplexError); ok {
				for key, val := range ex.GimmeMoreData() {
					event.Extra[key] = val
				}
			}

			return event
		})
	})

	errWithExtra := CustomComplexError{
		Message: "say what again. SAY WHAT again",
		MoreData: map[string]string{
			"say": "wat",
		},
	}

	sentry.CaptureException(errWithExtra)
}

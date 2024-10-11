package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"strconv"
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

func recoverHandler() {
	defer sentry.Recover()
	panic("ups")
}

func beforeSend() {
	sentry.CaptureMessage("Drop me!")
}

func captureMessage() {
	sentry.CaptureMessage("say what again. SAY WHAT again")
}

func configureScope() {
	sentry.ConfigureScope(func(scope *sentry.Scope) {
		scope.SetExtra("oristhis", "justfantasy")
		scope.SetTag("isthis", "reallife")
		scope.SetLevel(sentry.LevelFatal)
		scope.SetUser(sentry.User{
			ID: "1337",
		})
	})
}

func withScope() {
	sentry.WithScope(func(scope *sentry.Scope) {
		scope.SetLevel(sentry.LevelFatal)
		sentry.CaptureException(errors.New("say what again. SAY WHAT again"))
	})
}

func addBreadcrumbs() {
	sentry.AddBreadcrumb(&sentry.Breadcrumb{
		Message: "Random breadcrumb 1",
	})

	sentry.AddBreadcrumb(&sentry.Breadcrumb{
		Message: "Random breadcrumb 2",
	})

	sentry.AddBreadcrumb(&sentry.Breadcrumb{
		Message: "Random breadcrumb 3",
	})
}

func withScopeAndConfigureScope() {
	sentry.WithScope(func(scope *sentry.Scope) {
		sentry.ConfigureScope(func(scope *sentry.Scope) {
			scope.SetExtras(map[string]interface{}{
				"istillcant": 42,
				"believe":    "that",
			})
			scope.SetTags(map[string]string{
				"italready": "works",
				"just":      "likethat",
			})
		})

		event := sentry.NewEvent()
		event.Message = "say what again. SAY WHAT again"
		sentry.CaptureEvent(event)
	})
}

type CustomComplexError struct {
	Message      string
	AnswerToLife int
}

func (e CustomComplexError) Error() string {
	return "CustomComplexError: " + e.Message
}

func (e CustomComplexError) GimmeMoreData() string {
	return strconv.Itoa(e.AnswerToLife)
}

func eventHint() {
	sentry.CaptureException(CustomComplexError{Message: "Captured", AnswerToLife: 42})
}

func main() {
	if err := sentry.Init(sentry.ClientOptions{
		Debug:        true,
		Dsn:          "https://hello@example.com/1337",
		IgnoreErrors: []string{"^(?i)drop me"},
		BeforeSend: func(event *sentry.Event, hint *sentry.EventHint) *sentry.Event {
			if ex, ok := hint.OriginalException.(CustomComplexError); ok {
				event.Message = event.Message + " - " + ex.GimmeMoreData()
			}

			fmt.Printf("%s\n\n", prettyPrint(event))

			return event
		},
		BeforeBreadcrumb: func(breadcrumb *sentry.Breadcrumb, _ *sentry.BreadcrumbHint) *sentry.Breadcrumb {
			if breadcrumb.Message == "Random breadcrumb 3" {
				breadcrumb.Message = "Not so random breadcrumb 3"
			}

			fmt.Printf("%s\n\n", prettyPrint(breadcrumb))

			return breadcrumb
		},
		SampleRate: 1,
		Transport:  &devNullTransport{},
		Integrations: func(integrations []sentry.Integration) []sentry.Integration {
			return append(integrations, integrations[1])
		},
	}); err != nil {
		panic(err)
	}

	beforeSend()
	configureScope()
	withScope()
	captureMessage()
	addBreadcrumbs()
	withScopeAndConfigureScope()
	recoverHandler()
	eventHint()
}

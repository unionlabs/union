package sentry_test

import (
	"fmt"
	"net/http/httptest"
	"sync"
	"testing"

	"github.com/getsentry/sentry-go"
)

func TestConcurrentScopeUsage(_ *testing.T) {
	var wg sync.WaitGroup

	for i := 0; i < 10; i++ {
		wg.Add(1)
		go func(x int) {
			defer wg.Done()
			sentry.WithScope(func(scope *sentry.Scope) {
				touchScope(scope, x)
			})
		}(i)
		wg.Add(1)
		go func(x int) {
			defer wg.Done()
			sentry.ConfigureScope(func(scope *sentry.Scope) {
				touchScope(scope, x)
			})
		}(i)
	}

	for i := 0; i < 10; i++ {
		func(x int) {
			sentry.WithScope(func(scope *sentry.Scope) {
				touchScope(scope, x)
			})
		}(i)

		func(x int) {
			sentry.ConfigureScope(func(scope *sentry.Scope) {
				touchScope(scope, x)
			})
		}(i)
	}

	// wait for goroutines to finish
	wg.Wait()
}

func touchScope(scope *sentry.Scope, x int) {
	scope.SetTag("foo", "bar")
	scope.SetContext("foo", sentry.Context{"foo": "bar"})
	scope.SetExtra("foo", "bar")
	scope.SetLevel(sentry.LevelDebug)
	scope.SetFingerprint([]string{"foo"})
	scope.AddBreadcrumb(&sentry.Breadcrumb{Message: "foo"}, 100)
	scope.AddAttachment(&sentry.Attachment{Filename: "foo.txt"})
	scope.SetUser(sentry.User{ID: "foo"})
	scope.SetRequest(httptest.NewRequest("GET", "/foo", nil))
	scope.SetPropagationContext(sentry.NewPropagationContext())
	scope.SetSpan(&sentry.Span{TraceID: sentry.TraceIDFromHex("d49d9bf66f13450b81f65bc51cf49c03")})

	sentry.CaptureException(fmt.Errorf("error %d", x))

	scope.ClearBreadcrumbs()
	scope.Clone()
}

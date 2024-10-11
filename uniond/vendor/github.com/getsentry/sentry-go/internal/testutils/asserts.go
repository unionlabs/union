package testutils

import (
	"fmt"
	"reflect"
	"testing"

	"github.com/getsentry/sentry-go/internal/otel/baggage"
	"github.com/google/go-cmp/cmp"
)

func AssertEqual(t *testing.T, got, want interface{}, userMessage ...interface{}) {
	t.Helper()

	// Ideally, we would switch to cmp.Diff. However, in a general case, cmp.Diff
	// is not able to compare structs with unexported (private) fields by default. There
	// are ways to override modify that behaviour (e.g. by passing AllowUnexported), but
	// it significantly complicates its usage.
	if !reflect.DeepEqual(got, want) {
		logFailedAssertion(t, formatUnequalValues(got, want), userMessage...)
	}
}

func AssertNotEqual(t *testing.T, got, want interface{}, userMessage ...interface{}) {
	t.Helper()

	if reflect.DeepEqual(got, want) {
		logFailedAssertion(t, formatUnequalValues(got, want), userMessage...)
	}
}

func AssertTrue(t *testing.T, condition bool, userMessage ...interface{}) {
	t.Helper()

	if !condition {
		logFailedAssertion(t, "\nExpected: true", userMessage...)
	}
}

func AssertFalse(t *testing.T, condition bool, userMessage ...interface{}) {
	t.Helper()

	if condition {
		logFailedAssertion(t, "\nExpected: false", userMessage...)
	}
}

func logFailedAssertion(t *testing.T, summary string, userMessage ...interface{}) {
	t.Helper()
	text := summary

	if len(userMessage) > 0 {
		if message, ok := userMessage[0].(string); ok {
			if message != "" && len(userMessage) > 1 {
				text = fmt.Sprintf(message, userMessage[1:]...) + text
			} else if message != "" {
				text = fmt.Sprint(message) + text
			}
		}
	}

	t.Error(text)
}

func formatUnequalValues(got, want interface{}) string {
	var a, b string

	if reflect.TypeOf(got) != reflect.TypeOf(want) {
		a, b = fmt.Sprintf("%T(%#v)", got, got), fmt.Sprintf("%T(%#v)", want, want)
	} else {
		a, b = fmt.Sprintf("%#v", got), fmt.Sprintf("%#v", want)
	}

	return fmt.Sprintf("\ngot: %s\nwant: %s", a, b)
}

func AssertBaggageStringsEqual(t *testing.T, got, want string) {
	t.Helper()

	baggageGot, err := baggage.Parse(got)
	if err != nil {
		t.Error(err)
	}
	baggageWant, err := baggage.Parse(want)
	if err != nil {
		t.Error(err)
	}

	if diff := cmp.Diff(
		baggageWant,
		baggageGot,
		cmp.AllowUnexported(baggage.Member{}, baggage.Baggage{}),
	); diff != "" {
		t.Errorf("Comparing Baggage (-want +got):\n%s", diff)
	}
}

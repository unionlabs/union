// Package assert provides concise functions for common testing assertions.
package assert

import (
	"strings"
	"testing"
)

// NoError fails the test on a non-nil error.
func NoError(t *testing.T, err error) {
	t.Helper()
	if err != nil {
		t.Fatal(err)
	}
}

// Error fails the test on a nil error.
func Error(t *testing.T, err error) {
	t.Helper()
	if err == nil {
		t.Fatal("expected error; got nil")
	}
}

// ErrorContains asserts that err is non-nil and contains substr.
func ErrorContains(t *testing.T, err error, substr string) {
	t.Helper()
	Error(t, err)
	if !strings.Contains(err.Error(), substr) {
		t.Fatalf("unexpected error message: got %q; expected substring %q", err, substr)
	}
}

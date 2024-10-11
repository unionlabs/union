//go:build !go1.21

package sentry

import (
	"testing"
)

func Test_cleanupFunctionNamePrefix(t *testing.T) {
	f := []Frame{
		{Function: "main.main"},
		{Function: "main.main.func1"},
	}
	got := cleanupFunctionNamePrefix(f)
	assertEqual(t, got, f)

}

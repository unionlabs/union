//go:build !go1.21

package sentry

func cleanupFunctionNamePrefix(f []Frame) []Frame {
	return f
}

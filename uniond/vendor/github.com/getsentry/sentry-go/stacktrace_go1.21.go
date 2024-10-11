//go:build go1.21

package sentry

import "strings"

// Walk backwards through the results and for the current function name
// remove it's parent function's prefix, leaving only it's actual name. This
// fixes issues grouping errors with the new fully qualified function names
// introduced from Go 1.21.

func cleanupFunctionNamePrefix(f []Frame) []Frame {
	for i := len(f) - 1; i > 0; i-- {
		name := f[i].Function
		parentName := f[i-1].Function + "."

		if !strings.HasPrefix(name, parentName) {
			continue
		}

		f[i].Function = name[len(parentName):]
	}

	return f
}

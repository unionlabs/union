package env

import "testing"

var t = &testing.T{}

// Patch an environment variable and defer to return to the previous state
func ExamplePatch() {
	defer Patch(t, "PATH", "/custom/path")()
}

// Patch all environment variables
func ExamplePatchAll() {
	defer PatchAll(t, map[string]string{
		"ONE": "FOO",
		"TWO": "BAR",
	})()
}

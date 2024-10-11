/*
Package env provides functions to test code that read environment variables
or the current working directory.
*/
package env // import "gotest.tools/v3/env"

import (
	"os"
	"strings"

	"gotest.tools/v3/assert"
	"gotest.tools/v3/internal/cleanup"
)

type helperT interface {
	Helper()
}

// Patch changes the value of an environment variable, and returns a
// function which will reset the the value of that variable back to the
// previous state.
//
// When used with Go 1.14+ the unpatch function will be called automatically
// when the test ends, unless the TEST_NOCLEANUP env var is set to true.
//
// Deprecated: use t.SetEnv
func Patch(t assert.TestingT, key, value string) func() {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	oldValue, envVarExists := os.LookupEnv(key)
	assert.NilError(t, os.Setenv(key, value))
	clean := func() {
		if ht, ok := t.(helperT); ok {
			ht.Helper()
		}
		if !envVarExists {
			assert.NilError(t, os.Unsetenv(key))
			return
		}
		assert.NilError(t, os.Setenv(key, oldValue))
	}
	cleanup.Cleanup(t, clean)
	return clean
}

// PatchAll sets the environment to env, and returns a function which will
// reset the environment back to the previous state.
//
// When used with Go 1.14+ the unpatch function will be called automatically
// when the test ends, unless the TEST_NOCLEANUP env var is set to true.
func PatchAll(t assert.TestingT, env map[string]string) func() {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	oldEnv := os.Environ()
	os.Clearenv()

	for key, value := range env {
		assert.NilError(t, os.Setenv(key, value), "setenv %s=%s", key, value)
	}
	clean := func() {
		if ht, ok := t.(helperT); ok {
			ht.Helper()
		}
		os.Clearenv()
		for key, oldVal := range ToMap(oldEnv) {
			assert.NilError(t, os.Setenv(key, oldVal), "setenv %s=%s", key, oldVal)
		}
	}
	cleanup.Cleanup(t, clean)
	return clean
}

// ToMap takes a list of strings in the format returned by [os.Environ] and
// returns a mapping of keys to values.
func ToMap(env []string) map[string]string {
	result := map[string]string{}
	for _, raw := range env {
		key, value := getParts(raw)
		result[key] = value
	}
	return result
}

func getParts(raw string) (string, string) {
	if raw == "" {
		return "", ""
	}
	// Environment variables on windows can begin with =
	// http://blogs.msdn.com/b/oldnewthing/archive/2010/05/06/10008132.aspx
	parts := strings.SplitN(raw[1:], "=", 2)
	key := raw[:1] + parts[0]
	if len(parts) == 1 {
		return key, ""
	}
	return key, parts[1]
}

// ChangeWorkingDir to the directory, and return a function which restores the
// previous working directory.
//
// When used with Go 1.14+ the previous working directory will be restored
// automatically when the test ends, unless the TEST_NOCLEANUP env var is set to
// true.
func ChangeWorkingDir(t assert.TestingT, dir string) func() {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	cwd, err := os.Getwd()
	assert.NilError(t, err)
	assert.NilError(t, os.Chdir(dir))
	clean := func() {
		if ht, ok := t.(helperT); ok {
			ht.Helper()
		}
		assert.NilError(t, os.Chdir(cwd))
	}
	cleanup.Cleanup(t, clean)
	return clean
}

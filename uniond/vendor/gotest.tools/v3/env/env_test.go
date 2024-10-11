package env

import (
	"os"
	"runtime"
	"sort"
	"testing"

	"gotest.tools/v3/assert"
	"gotest.tools/v3/fs"
	"gotest.tools/v3/internal/source"
	"gotest.tools/v3/skip"
)

func TestPatchFromUnset(t *testing.T) {
	key, value := "FOO_IS_UNSET", "VALUE"
	revert := Patch(t, key, value)

	assert.Assert(t, value == os.Getenv(key))
	revert()
	_, isSet := os.LookupEnv(key)
	assert.Assert(t, !isSet)
}

func TestPatch(t *testing.T) {
	skip.If(t, os.Getenv("PATH") == "")
	oldVal := os.Getenv("PATH")

	key, value := "PATH", "NEWVALUE"
	revert := Patch(t, key, value)

	assert.Assert(t, value == os.Getenv(key))
	revert()
	assert.Assert(t, oldVal == os.Getenv(key))
}

func TestPatch_IntegrationWithCleanup(t *testing.T) {
	skip.If(t, source.GoVersionLessThan(1, 14))

	key := "totally_unique_env_var_key"
	t.Run("cleanup in subtest", func(t *testing.T) {
		Patch(t, key, "the-new-value")
		assert.Equal(t, os.Getenv(key), "the-new-value")
	})

	t.Run("env var is unset", func(t *testing.T) {
		v, ok := os.LookupEnv(key)
		assert.Assert(t, !ok, "expected env var to be unset, got %v", v)
	})
}

func TestPatchAll(t *testing.T) {
	oldEnv := os.Environ()
	newEnv := map[string]string{
		"FIRST": "STARS",
		"THEN":  "MOON",
	}

	revert := PatchAll(t, newEnv)

	actual := os.Environ()
	sort.Strings(actual)
	assert.DeepEqual(t, []string{"FIRST=STARS", "THEN=MOON"}, actual)

	revert()
	assert.DeepEqual(t, sorted(oldEnv), sorted(os.Environ()))
}

func TestPatchAllWindows(t *testing.T) {
	skip.If(t, runtime.GOOS != "windows")
	oldEnv := os.Environ()
	newEnv := map[string]string{
		"FIRST":  "STARS",
		"THEN":   "MOON",
		"=FINAL": "SUN",
		"=BAR":   "",
	}

	revert := PatchAll(t, newEnv)

	actual := os.Environ()
	sort.Strings(actual)
	assert.DeepEqual(t, []string{"=BAR=", "=FINAL=SUN", "FIRST=STARS", "THEN=MOON"}, actual)

	revert()
	assert.DeepEqual(t, sorted(oldEnv), sorted(os.Environ()))
}

func sorted(source []string) []string {
	sort.Strings(source)
	return source
}

func TestPatchAll_IntegrationWithCleanup(t *testing.T) {
	skip.If(t, source.GoVersionLessThan(1, 14))

	key := "totally_unique_env_var_key"
	t.Run("cleanup in subtest", func(t *testing.T) {
		PatchAll(t, map[string]string{key: "the-new-value"})
		assert.Equal(t, os.Getenv(key), "the-new-value")
	})

	t.Run("env var is unset", func(t *testing.T) {
		v, ok := os.LookupEnv(key)
		assert.Assert(t, !ok, "expected env var to be unset, got %v", v)
	})
}

func TestToMap(t *testing.T) {
	source := []string{
		"key=value",
		"novaluekey",
		"=foo=bar",
		"z=singlecharkey",
		"b",
		"",
	}
	actual := ToMap(source)
	expected := map[string]string{
		"key":        "value",
		"novaluekey": "",
		"=foo":       "bar",
		"z":          "singlecharkey",
		"b":          "",
		"":           "",
	}
	assert.DeepEqual(t, expected, actual)
}

func TestChangeWorkingDir(t *testing.T) {
	tmpDir := fs.NewDir(t, t.Name())
	defer tmpDir.Remove()

	origWorkDir := pwd(t)

	reset := ChangeWorkingDir(t, tmpDir.Path())
	t.Run("changed to dir", func(t *testing.T) {
		assert.Equal(t, pwd(t), tmpDir.Path())
	})

	t.Run("reset dir", func(t *testing.T) {
		reset()
		assert.Equal(t, pwd(t), origWorkDir)
	})
}

func TestChangeWorkingDir_IntegrationWithCleanup(t *testing.T) {
	skip.If(t, source.GoVersionLessThan(1, 14))

	tmpDir := fs.NewDir(t, t.Name())
	defer tmpDir.Remove()

	origWorkDir := pwd(t)

	t.Run("cleanup in subtest", func(t *testing.T) {
		ChangeWorkingDir(t, tmpDir.Path())
		assert.Equal(t, pwd(t), tmpDir.Path())
	})

	t.Run("working dir is reset", func(t *testing.T) {
		assert.Equal(t, pwd(t), origWorkDir)
	})
}

func pwd(t *testing.T) string {
	t.Helper()
	dir, err := os.Getwd()
	assert.NilError(t, err)
	return dir
}

package sentry

import (
	"runtime/debug"
	"testing"
)

func TestUUIDReturnsRandom32CharacterString(t *testing.T) {
	u1 := uuid()
	u2 := uuid()
	u3 := uuid()

	assertEqual(t, len(u1), 32)
	assertEqual(t, len(u2), 32)
	assertEqual(t, len(u3), 32)

	assertNotEqual(t, u1, u2)
	assertNotEqual(t, u1, u3)
	assertNotEqual(t, u2, u3)
}

func TestFileExistsReturnsTrueForExistingFiles(t *testing.T) {
	assertEqual(t, fileExists(("util.go")), true)
	assertEqual(t, fileExists(("util_test.go")), true)
}

func TestFileExistsReturnsFalseForNonExistingFiles(t *testing.T) {
	assertEqual(t, fileExists(("util_nope.go")), false)
	assertEqual(t, fileExists(("util_nope_test.go")), false)
}

func TestDefaultReleaseSentryReleaseEnvvar(t *testing.T) {
	releaseVersion := "1.2.3"
	t.Setenv("SENTRY_RELEASE", releaseVersion)

	assertEqual(t, defaultRelease(), releaseVersion)
}

func TestDefaultReleaseSentryReleaseEnvvarPrecedence(t *testing.T) {
	releaseVersion := "1.2.3"
	t.Setenv("SOURCE_VERSION", "3.2.1")
	t.Setenv("SENTRY_RELEASE", releaseVersion)

	assertEqual(t, defaultRelease(), releaseVersion)
}

func TestRevisionFromBuildInfo(t *testing.T) {
	releaseVersion := "deadbeef"

	info := &debug.BuildInfo{
		Main: debug.Module{
			Path:    "my/module",
			Version: "(devel)",
		},
		Deps: []*debug.Module{
			{
				Path:    "github.com/getsentry/sentry-go",
				Version: "v0.23.1",
				Replace: &debug.Module{
					Path: "pkg/sentry",
				},
			},
		},
		Settings: []debug.BuildSetting{
			{
				Key:   "vcs.revision",
				Value: releaseVersion,
			},
		},
	}

	assertEqual(t, revisionFromBuildInfo(info), releaseVersion)
}

func TestRevisionFromBuildInfoNoVcsInformation(t *testing.T) {
	info := &debug.BuildInfo{
		Main: debug.Module{
			Path:    "my/module",
			Version: "(devel)",
		},
		Deps: []*debug.Module{
			{
				Path:    "github.com/getsentry/sentry-go",
				Version: "v0.23.1",
				Replace: &debug.Module{
					Path: "pkg/sentry",
				},
			},
		},
	}

	assertEqual(t, revisionFromBuildInfo(info), "")
}

func TestPointer(t *testing.T) {
	i := 5
	v := Pointer(i)
	assertEqual(t, *v, i)
}

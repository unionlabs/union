package prereqs

import (
	"context"
	"testing"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

func TestMeetsMinPlatformReqs(t *testing.T) {
	// Linux
	runtimeOS = "linux"
	runtimeArch = "amd64"
	meetsReqs, err := meetsMinPlatformReqs(context.Background())
	require.Equal(t, true, meetsReqs)
	require.NoError(t, err)
	runtimeArch = "arm64"
	meetsReqs, err = meetsMinPlatformReqs(context.Background())
	require.Equal(t, true, meetsReqs)
	require.NoError(t, err)
	// mips64 is not supported
	runtimeArch = "mips64"
	meetsReqs, err = meetsMinPlatformReqs(context.Background())
	require.Equal(t, false, meetsReqs)
	require.NoError(t, err)

	// Mac OS X
	// In this function we'll set the execShellOutput package variable to another function that will 'mock' the shell
	execShellOutput = func(ctx context.Context, command string, args ...string) (string, error) {
		return "", errors.New("error while running command")
	}
	runtimeOS = "darwin"
	runtimeArch = "amd64"
	meetsReqs, err = meetsMinPlatformReqs(context.Background())
	require.Equal(t, false, meetsReqs)
	require.ErrorContains(t, "error obtaining MacOS version", err)

	// Insufficient version
	execShellOutput = func(ctx context.Context, command string, args ...string) (string, error) {
		return "10.4", nil
	}
	meetsReqs, err = meetsMinPlatformReqs(context.Background())
	require.Equal(t, false, meetsReqs)
	require.NoError(t, err)

	// Just-sufficient older version
	execShellOutput = func(ctx context.Context, command string, args ...string) (string, error) {
		return "10.14", nil
	}
	meetsReqs, err = meetsMinPlatformReqs(context.Background())
	require.Equal(t, true, meetsReqs)
	require.NoError(t, err)

	// Sufficient newer version
	execShellOutput = func(ctx context.Context, command string, args ...string) (string, error) {
		return "10.15.7", nil
	}
	meetsReqs, err = meetsMinPlatformReqs(context.Background())
	require.Equal(t, true, meetsReqs)
	require.NoError(t, err)

	// Handling abnormal response
	execShellOutput = func(ctx context.Context, command string, args ...string) (string, error) {
		return "tiger.lion", nil
	}
	meetsReqs, err = meetsMinPlatformReqs(context.Background())
	require.Equal(t, false, meetsReqs)
	require.ErrorContains(t, "error parsing version", err)

	// Windows
	runtimeOS = "windows"
	runtimeArch = "amd64"
	meetsReqs, err = meetsMinPlatformReqs(context.Background())
	require.Equal(t, true, meetsReqs)
	require.NoError(t, err)
	runtimeArch = "arm64"
	meetsReqs, err = meetsMinPlatformReqs(context.Background())
	require.Equal(t, false, meetsReqs)
	require.NoError(t, err)
}

func TestParseVersion(t *testing.T) {
	version, err := parseVersion("1.2.3", 3, ".")
	require.DeepEqual(t, version, []int{1, 2, 3})
	require.NoError(t, err)

	version, err = parseVersion("6 .7 . 8  ", 3, ".")
	require.DeepEqual(t, version, []int{6, 7, 8})
	require.NoError(t, err)

	version, err = parseVersion("10,3,5,6", 4, ",")
	require.DeepEqual(t, version, []int{10, 3, 5, 6})
	require.NoError(t, err)

	version, err = parseVersion("4;6;8;10;11", 3, ";")
	require.DeepEqual(t, version, []int{4, 6, 8})
	require.NoError(t, err)

	_, err = parseVersion("10.11", 3, ".")
	require.ErrorContains(t, "insufficient information about version", err)
}

func TestWarnIfNotSupported(t *testing.T) {
	runtimeOS = "linux"
	runtimeArch = "amd64"
	hook := logTest.NewGlobal()
	WarnIfPlatformNotSupported(context.Background())
	require.LogsDoNotContain(t, hook, "Failed to detect host platform")
	require.LogsDoNotContain(t, hook, "platform is not supported")

	execShellOutput = func(ctx context.Context, command string, args ...string) (string, error) {
		return "tiger.lion", nil
	}
	runtimeOS = "darwin"
	runtimeArch = "amd64"
	hook = logTest.NewGlobal()
	WarnIfPlatformNotSupported(context.Background())
	require.LogsContain(t, hook, "Failed to detect host platform")
	require.LogsContain(t, hook, "error parsing version")

	runtimeOS = "falseOs"
	runtimeArch = "falseArch"
	hook = logTest.NewGlobal()
	WarnIfPlatformNotSupported(context.Background())
	require.LogsContain(t, hook, "platform is not supported")
}

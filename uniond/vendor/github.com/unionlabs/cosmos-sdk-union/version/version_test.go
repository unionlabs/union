package version_test

import (
	"context"
	"encoding/json"
	"fmt"
	"runtime"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/testutil"
	"github.com/cosmos/cosmos-sdk/testutil/cmdtest"
	"github.com/cosmos/cosmos-sdk/version"
)

func TestNewInfo(t *testing.T) {
	info := version.NewInfo()
	want := fmt.Sprintf(`: 
git commit: 
build tags: 
%s`, fmt.Sprintf("go version %s %s/%s", runtime.Version(), runtime.GOOS, runtime.GOARCH))
	require.Equal(t, want, info.String())
}

func TestInfo_String(t *testing.T) {
	info := version.Info{
		Name:             "testapp",
		AppName:          "testappd",
		Version:          "1.0.0",
		GitCommit:        "1b78457135a4104bc3af97f20654d49e2ea87454",
		BuildTags:        "netgo,ledger",
		GoVersion:        "go version go1.14 linux/amd64",
		CosmosSdkVersion: "0.42.5",
	}
	want := `testapp: 1.0.0
git commit: 1b78457135a4104bc3af97f20654d49e2ea87454
build tags: netgo,ledger
go version go1.14 linux/amd64`
	require.Equal(t, want, info.String())
}

func TestCLI(t *testing.T) {
	setVersionPackageVars(t)

	sys := cmdtest.NewSystem()
	sys.AddCommands(version.NewVersionCommand())

	t.Run("no flags", func(t *testing.T) {
		res := sys.MustRun(t, "version")

		// Only prints the version, with a newline, to stdout.
		require.Equal(t, testVersion+"\n", res.Stdout.String())
		require.Empty(t, res.Stderr.String())
	})

	t.Run("--long flag", func(t *testing.T) {
		res := sys.MustRun(t, "version", "--long")

		out := res.Stdout.String()
		lines := strings.Split(out, "\n")
		require.Contains(t, lines, "name: testchain-app")
		require.Contains(t, lines, "server_name: testchaind")
		require.Contains(t, lines, `version: "3.14"`)
		require.Contains(t, lines, "commit: abc123")
		require.Contains(t, lines, "build_tags: mybuildtag")

		require.Empty(t, res.Stderr.String())
	})

	t.Run("--output=json flag", func(t *testing.T) {
		res := sys.MustRun(t, "version", "--output=json")

		var info version.Info
		require.NoError(t, json.Unmarshal(res.Stdout.Bytes(), &info))

		// Assert against a couple fields that are difficult to predict in test
		// without copying and pasting code.
		require.NotEmpty(t, info.GoVersion)

		// The SDK version appears to not be set during this test, so we'll ignore it here.

		// Now clear out the non-empty fields, so we can compare against a fixed value.
		info.GoVersion = ""

		want := version.Info{
			Name:      testName,
			AppName:   testAppName,
			Version:   testVersion,
			GitCommit: testCommit,
			BuildTags: testBuildTags,
		}
		require.Equal(t, want, info)

		require.Empty(t, res.Stderr.String())
	})

	t.Run("positional args rejected", func(t *testing.T) {
		res := sys.Run("version", "foo")
		require.Error(t, res.Err)
	})
}

const (
	testName      = "testchain-app"
	testAppName   = "testchaind"
	testVersion   = "3.14"
	testCommit    = "abc123"
	testBuildTags = "mybuildtag"
)

// setVersionPackageVars temporarily overrides the package variables in the version package
// so that we can assert meaningful output.
func setVersionPackageVars(t *testing.T) {
	t.Helper()

	var (
		origName      = version.Name
		origAppName   = version.AppName
		origVersion   = version.Version
		origCommit    = version.Commit
		origBuildTags = version.BuildTags
	)

	t.Cleanup(func() {
		version.Name = origName
		version.AppName = origAppName
		version.Version = origVersion
		version.Commit = origCommit
		version.BuildTags = origBuildTags
	})

	version.Name = testName
	version.AppName = testAppName
	version.Version = testVersion
	version.Commit = testCommit
	version.BuildTags = testBuildTags
}

func Test_runVersionCmd(t *testing.T) {
	cmd := version.NewVersionCommand()
	_, mockOut := testutil.ApplyMockIO(cmd)

	cmd.SetArgs([]string{
		fmt.Sprintf("--%s=''", flags.FlagOutput),
		"--long=false",
	})

	require.NoError(t, cmd.Execute())
	require.Equal(t, "\n", mockOut.String())
	mockOut.Reset()

	cmd.SetArgs([]string{
		fmt.Sprintf("--%s=json", flags.FlagOutput), "--long=true",
	})

	info := version.NewInfo()
	stringInfo, err := json.Marshal(info)

	extraInfo := &version.ExtraInfo{"key1": "value1"}
	ctx := context.WithValue(context.Background(), version.ContextKey{}, extraInfo)

	require.NoError(t, err)
	require.NoError(t, cmd.Execute())

	extraInfoFromContext := ctx.Value(version.ContextKey{})
	require.NotNil(t, extraInfoFromContext)

	castedExtraInfo, ok := extraInfoFromContext.(*version.ExtraInfo)
	require.True(t, ok)

	key1Value := (*castedExtraInfo)["key1"]
	require.Equal(t, "value1", key1Value)

	require.Equal(t, string(stringInfo)+"\n", mockOut.String())
}

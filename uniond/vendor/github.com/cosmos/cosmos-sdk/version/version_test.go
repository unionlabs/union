package version_test

import (
	"encoding/json"
	"fmt"
	"runtime"
	"testing"

	"github.com/cometbft/cometbft/libs/cli"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/testutil"
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

func Test_runVersionCmd(t *testing.T) {
	cmd := version.NewVersionCommand()
	_, mockOut := testutil.ApplyMockIO(cmd)

	cmd.SetArgs([]string{
		fmt.Sprintf("--%s=''", cli.OutputFlag),
		"--long=false",
	})

	require.NoError(t, cmd.Execute())
	assert.Equal(t, "\n", mockOut.String())
	mockOut.Reset()

	cmd.SetArgs([]string{
		fmt.Sprintf("--%s=json", cli.OutputFlag), "--long=true",
	})

	info := version.NewInfo()
	stringInfo, err := json.Marshal(info)
	require.NoError(t, err)
	require.NoError(t, cmd.Execute())
	assert.Equal(t, string(stringInfo)+"\n", mockOut.String())
}

package cmd

import (
	"flag"
	"os"
	"os/user"
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/cmd/mock"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/urfave/cli/v2"
)

func TestEnterPassword(t *testing.T) {
	type rets struct {
		pw  string
		err error
	}
	var tt = []struct {
		name        string
		rets        []rets
		expectedErr error
		expectedPw  string
	}{
		{
			"first_match",
			[]rets{{"abcd", nil}, {"abcd", nil}},
			nil,
			"abcd",
		},
		{
			"first_match_with_newline",
			[]rets{{"abcd\n", nil}, {"abcd", nil}},
			nil,
			"abcd",
		},
		{
			"first_match_with_newline_confirm",
			[]rets{{"abcd", nil}, {"abcd\n", nil}},
			nil,
			"abcd",
		},
		{
			"first_match_both_newline",
			[]rets{{"abcd\n", nil}, {"abcd\n", nil}},
			nil,
			"abcd",
		},
		{
			"second_match",
			[]rets{{"abcd", nil}, {"aba", nil}, {"abcd", nil}, {"abcd", nil}},
			nil,
			"abcd",
		},
		{
			"cant_read",
			[]rets{{"pw", errors.New("i/o fail")}},
			errors.New("i/o fail"),
			"",
		},
	}

	for _, tc := range tt {
		t.Run(tc.name, func(t *testing.T) {
			ctrl := gomock.NewController(t)
			m := mock.NewPasswordReader(ctrl)
			for _, ret := range tc.rets {
				m.EXPECT().ReadPassword().Return(ret.pw, ret.err)
			}
			pw, err := EnterPassword(true, m)
			assert.Equal(t, tc.expectedPw, pw)
			if tc.expectedErr != nil {
				assert.ErrorContains(t, tc.expectedErr.Error(), err)
			} else {
				assert.NoError(t, err)
			}
		})
	}
}

func TestExpandSingleEndpointIfFile(t *testing.T) {
	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	ExecutionEndpointFlag := &cli.StringFlag{Name: "execution-endpoint", Value: ""}
	set.String(ExecutionEndpointFlag.Name, "", "")
	context := cli.NewContext(&app, set, nil)

	// with nothing set
	require.NoError(t, ExpandSingleEndpointIfFile(context, ExecutionEndpointFlag))
	require.Equal(t, "", context.String(ExecutionEndpointFlag.Name))

	// with url scheme
	require.NoError(t, context.Set(ExecutionEndpointFlag.Name, "http://localhost:8545"))
	require.NoError(t, ExpandSingleEndpointIfFile(context, ExecutionEndpointFlag))
	require.Equal(t, "http://localhost:8545", context.String(ExecutionEndpointFlag.Name))

	// relative user home path
	usr, err := user.Current()
	require.NoError(t, err)
	require.NoError(t, context.Set(ExecutionEndpointFlag.Name, "~/relative/path.ipc"))
	require.NoError(t, ExpandSingleEndpointIfFile(context, ExecutionEndpointFlag))
	require.Equal(t, usr.HomeDir+"/relative/path.ipc", context.String(ExecutionEndpointFlag.Name))

	// current dir path
	curentdir, err := os.Getwd()
	require.NoError(t, err)
	require.NoError(t, context.Set(ExecutionEndpointFlag.Name, "./path.ipc"))
	require.NoError(t, ExpandSingleEndpointIfFile(context, ExecutionEndpointFlag))
	require.Equal(t, curentdir+"/path.ipc", context.String(ExecutionEndpointFlag.Name))
}

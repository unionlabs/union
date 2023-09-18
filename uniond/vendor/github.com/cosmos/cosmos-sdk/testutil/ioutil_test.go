package testutil_test

import (
	"io"
	"os"
	"testing"

	"github.com/spf13/cobra"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/testutil"
)

func TestApplyMockIO(t *testing.T) {
	cmd := &cobra.Command{}
	oldStdin := cmd.InOrStdin()
	oldStdout := cmd.OutOrStdout()
	oldStderr := cmd.ErrOrStderr()

	testutil.ApplyMockIO(cmd)

	require.NotEqual(t, cmd.InOrStdin(), oldStdin)
	require.NotEqual(t, cmd.OutOrStdout(), oldStdout)
	require.NotEqual(t, cmd.ErrOrStderr(), oldStderr)
	require.Equal(t, cmd.ErrOrStderr(), cmd.OutOrStdout())
}

func TestWriteToNewTempFile(t *testing.T) {
	tempfile := testutil.WriteToNewTempFile(t, "test string")
	require.NoError(t, tempfile.Close())

	bs, err := os.ReadFile(tempfile.Name())
	require.NoError(t, err)
	require.Equal(t, "test string", string(bs))
}

func TestApplyMockIODiscardOutErr(t *testing.T) {
	cmd := &cobra.Command{}
	oldStdin := cmd.InOrStdin()

	testutil.ApplyMockIODiscardOutErr(cmd)
	require.NotEqual(t, cmd.InOrStdin(), oldStdin)
	require.Equal(t, cmd.OutOrStdout(), io.Discard)
	require.Equal(t, cmd.ErrOrStderr(), io.Discard)
}

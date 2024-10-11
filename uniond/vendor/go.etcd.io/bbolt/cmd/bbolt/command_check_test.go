package main_test

import (
	"bytes"
	"io"
	"testing"

	"github.com/stretchr/testify/require"

	main "go.etcd.io/bbolt/cmd/bbolt"
	"go.etcd.io/bbolt/internal/btesting"
	"go.etcd.io/bbolt/internal/guts_cli"
)

func TestCheckCommand_Run(t *testing.T) {
	testCases := []struct {
		name      string
		args      []string
		expErr    error
		expOutput string
	}{
		{
			name:      "check whole db",
			args:      []string{"check", "path"},
			expErr:    nil,
			expOutput: "OK\n",
		},
		{
			name:      "check valid pageId",
			args:      []string{"check", "path", "--from-page", "3"},
			expErr:    nil,
			expOutput: "OK\n",
		},
		{
			name:      "check invalid pageId",
			args:      []string{"check", "path", "--from-page", "1"},
			expErr:    guts_cli.ErrCorrupt,
			expOutput: "page ID (1) out of range [2, 4)",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {

			t.Log("Creating sample DB")
			db := btesting.MustCreateDB(t)
			db.Close()
			defer requireDBNoChange(t, dbData(t, db.Path()), db.Path())

			t.Log("Running check cmd")
			rootCmd := main.NewRootCommand()
			outputBuf := bytes.NewBufferString("") // capture output for assertion
			rootCmd.SetOut(outputBuf)

			tc.args[1] = db.Path() // path to be replaced with db.Path()
			rootCmd.SetArgs(tc.args)
			err := rootCmd.Execute()
			require.Equal(t, tc.expErr, err)

			t.Log("Checking output")
			output, err := io.ReadAll(outputBuf)
			require.NoError(t, err)
			require.Containsf(t, string(output), tc.expOutput, "unexpected stdout:\n\n%s", string(output))
		})
	}
}

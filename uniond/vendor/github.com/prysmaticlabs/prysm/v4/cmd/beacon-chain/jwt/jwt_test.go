package jwt

import (
	"flag"
	"os"
	"path/filepath"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/urfave/cli/v2"
)

func Test_generateJWTSecret(t *testing.T) {
	t.Run("command should be available", func(t *testing.T) {
		generateJwtCommand := Commands
		require.Equal(t, true, generateJwtCommand.Name == "generate-auth-secret")
	})
	t.Run("should create proper file in current directory", func(t *testing.T) {
		require.NoError(t, os.RemoveAll(secretFileName))
		t.Cleanup(func() {
			require.NoError(t, os.RemoveAll(secretFileName))
		})
		app := cli.App{}
		set := flag.NewFlagSet("test", 0)
		cliCtx := cli.NewContext(&app, set, nil)
		err := generateAuthSecretInFile(cliCtx)
		require.NoError(t, err)

		// We check the file has the contents we expect.
		checkAuthFileIntegrity(t, secretFileName)
	})
	t.Run("should create proper file in specified folder", func(t *testing.T) {
		customOutput := filepath.Join("data", "item.txt")
		require.NoError(t, os.RemoveAll(filepath.Dir(customOutput)))
		t.Cleanup(func() {
			require.NoError(t, os.RemoveAll(filepath.Dir(customOutput)))
		})
		app := cli.App{}
		set := flag.NewFlagSet("test", 0)
		set.String(cmd.JwtOutputFileFlag.Name, customOutput, "")
		require.NoError(t, set.Set(cmd.JwtOutputFileFlag.Name, customOutput))

		cliCtx := cli.NewContext(&app, set, nil)
		err := generateAuthSecretInFile(cliCtx)
		require.NoError(t, err)

		// We check the file has the contents we expect.
		checkAuthFileIntegrity(t, customOutput)
	})
	t.Run("creates proper file in nested specified folder", func(t *testing.T) {
		rootDirectory := "data"
		customOutputPath := filepath.Join(rootDirectory, "nest", "nested", "item.txt")
		require.NoError(t, os.RemoveAll(filepath.Dir(customOutputPath)))
		t.Cleanup(func() {
			require.NoError(t, os.RemoveAll(rootDirectory))
			_, err := os.Stat(customOutputPath)
			require.Equal(t, true, err != nil)
		})
		app := cli.App{}
		set := flag.NewFlagSet("test", 0)
		set.String(cmd.JwtOutputFileFlag.Name, customOutputPath, "")
		require.NoError(t, set.Set(cmd.JwtOutputFileFlag.Name, customOutputPath))

		cliCtx := cli.NewContext(&app, set, nil)
		err := generateAuthSecretInFile(cliCtx)
		require.NoError(t, err)

		// We check the file has the contents we expect.
		checkAuthFileIntegrity(t, customOutputPath)
	})
}

func checkAuthFileIntegrity(t testing.TB, fPath string) {
	fileInfo, err := os.Stat(fPath)
	require.NoError(t, err)
	require.Equal(t, true, fileInfo != nil)

	enc, err := file.ReadFileAsBytes(fPath)
	require.NoError(t, err)
	decoded, err := hexutil.Decode(string(enc))
	require.NoError(t, err)
	require.Equal(t, 32, len(decoded))
}

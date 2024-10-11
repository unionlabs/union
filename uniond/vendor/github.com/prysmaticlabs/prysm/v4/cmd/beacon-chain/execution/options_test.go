package execution

import (
	"flag"
	"fmt"
	"path/filepath"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/flags"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/urfave/cli/v2"
)

func TestExecutionchainCmd(t *testing.T) {
	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String(flags.ExecutionEngineEndpoint.Name, "primary", "")
	ctx := cli.NewContext(&app, set, nil)

	endpoints, err := parseExecutionChainEndpoint(ctx)
	require.NoError(t, err)
	assert.Equal(t, "primary", endpoints)
}

func Test_parseJWTSecretFromFile(t *testing.T) {
	t.Run("no flag value specified leads to nil secret", func(t *testing.T) {
		app := cli.App{}
		set := flag.NewFlagSet("test", 0)
		set.String(flags.ExecutionJWTSecretFlag.Name, "", "")
		ctx := cli.NewContext(&app, set, nil)
		secret, err := parseJWTSecretFromFile(ctx)
		require.NoError(t, err)
		require.Equal(t, true, secret == nil)
	})
	t.Run("flag specified but no file found", func(t *testing.T) {
		app := cli.App{}
		set := flag.NewFlagSet("test", 0)
		set.String(flags.ExecutionJWTSecretFlag.Name, "/tmp/askdjkajsd", "")
		ctx := cli.NewContext(&app, set, nil)
		_, err := parseJWTSecretFromFile(ctx)
		require.ErrorContains(t, "no such file", err)
	})
	t.Run("empty string in file", func(t *testing.T) {
		app := cli.App{}
		set := flag.NewFlagSet("test", 0)
		fullPath := filepath.Join(t.TempDir(), "foohex")
		require.NoError(t, file.WriteFile(fullPath, []byte{}))
		set.String(flags.ExecutionJWTSecretFlag.Name, fullPath, "")
		ctx := cli.NewContext(&app, set, nil)
		_, err := parseJWTSecretFromFile(ctx)
		require.ErrorContains(t, "cannot be empty", err)
	})
	t.Run("less than 32 bytes", func(t *testing.T) {
		app := cli.App{}
		set := flag.NewFlagSet("test", 0)
		fullPath := filepath.Join(t.TempDir(), "foohex")
		secret := bytesutil.PadTo([]byte("foo"), 31)
		hexData := fmt.Sprintf("%#x", secret)
		require.NoError(t, file.WriteFile(fullPath, []byte(hexData)))
		set.String(flags.ExecutionJWTSecretFlag.Name, fullPath, "")
		ctx := cli.NewContext(&app, set, nil)
		_, err := parseJWTSecretFromFile(ctx)
		require.ErrorContains(t, "should be a hex string of at least 32 bytes", err)
	})
	t.Run("bad data", func(t *testing.T) {
		app := cli.App{}
		set := flag.NewFlagSet("test", 0)
		fullPath := filepath.Join(t.TempDir(), "foohex")
		secret := []byte("foo")
		require.NoError(t, file.WriteFile(fullPath, secret))
		set.String(flags.ExecutionJWTSecretFlag.Name, fullPath, "")
		ctx := cli.NewContext(&app, set, nil)
		_, err := parseJWTSecretFromFile(ctx)
		require.ErrorContains(t, "invalid byte", err)
	})
	t.Run("correct format", func(t *testing.T) {
		app := cli.App{}
		set := flag.NewFlagSet("test", 0)
		fullPath := filepath.Join(t.TempDir(), "foohex")
		secret := bytesutil.ToBytes32([]byte("foo"))
		secretHex := fmt.Sprintf("%#x", secret)
		require.NoError(t, file.WriteFile(fullPath, []byte(secretHex)))
		set.String(flags.ExecutionJWTSecretFlag.Name, fullPath, "")
		ctx := cli.NewContext(&app, set, nil)
		got, err := parseJWTSecretFromFile(ctx)
		require.NoError(t, err)
		require.DeepEqual(t, secret[:], got)
	})
}

func TestPowchainPreregistration_EmptyWeb3Provider(t *testing.T) {
	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	set.String(flags.ExecutionEngineEndpoint.Name, "", "")
	ctx := cli.NewContext(&app, set, nil)
	_, err := parseExecutionChainEndpoint(ctx)
	assert.ErrorContains(t, "you need to specify", err)
}

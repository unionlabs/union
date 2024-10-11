package flags

import (
	"flag"
	"fmt"
	"os"
	"strings"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/urfave/cli/v2"
)

func TestLoadFlagsFromConfig_PreProcessing_Web3signer(t *testing.T) {
	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	context := cli.NewContext(&app, set, nil)

	pubkey1 := "0xbd36226746676565cd40141a7f0fe1445b9a3fbeb222288b226392c4b230ed0b"
	pubkey2 := "0xbd36226746676565cd40141a7f0fe1445b9a3fbeb222288b226392c4b230ed0a"

	require.NoError(t, os.WriteFile("flags_test.yaml", []byte(fmt.Sprintf("%s:\n - %s\n - %s\n", Web3SignerPublicValidatorKeysFlag.Name,
		pubkey1,
		pubkey2)), 0666))

	require.NoError(t, set.Parse([]string{"test-command", "--" + cmd.ConfigFileFlag.Name, "flags_test.yaml"}))
	comFlags := cmd.WrapFlags([]cli.Flag{
		&cli.StringFlag{
			Name: cmd.ConfigFileFlag.Name,
		},
		&cli.StringSliceFlag{
			Name: Web3SignerPublicValidatorKeysFlag.Name,
		},
	})
	command := &cli.Command{
		Name:  "test-command",
		Flags: comFlags,
		Before: func(cliCtx *cli.Context) error {
			return cmd.LoadFlagsFromConfig(cliCtx, comFlags)
		},
		Action: func(cliCtx *cli.Context) error {
			require.Equal(t, true, cliCtx.IsSet(Web3SignerPublicValidatorKeysFlag.Name))

			require.Equal(t, strings.Join([]string{pubkey1, pubkey2}, ","),
				strings.Join(cliCtx.StringSlice(Web3SignerPublicValidatorKeysFlag.Name), ","))
			return nil
		},
	}
	require.NoError(t, command.Run(context, context.Args().Slice()...))
	require.NoError(t, os.Remove("flags_test.yaml"))
}

func TestLoadFlagsFromConfig_EnableBuilderHasDefaultValue(t *testing.T) {
	app := cli.App{}
	set := flag.NewFlagSet("test", 0)
	context := cli.NewContext(&app, set, nil)

	require.NoError(t, os.WriteFile("flags_test.yaml", []byte("---\nenable-builder: true"), 0666))

	require.NoError(t, set.Parse([]string{"test-command", "--" + cmd.ConfigFileFlag.Name, "flags_test.yaml"}))
	comFlags := cmd.WrapFlags([]cli.Flag{
		&cli.StringFlag{
			Name: cmd.ConfigFileFlag.Name,
		},
		&cli.BoolFlag{
			Name:  EnableBuilderFlag.Name,
			Value: false,
		},
	})
	command := &cli.Command{
		Name:  "test-command",
		Flags: comFlags,
		Before: func(cliCtx *cli.Context) error {
			return cmd.LoadFlagsFromConfig(cliCtx, comFlags)
		},
		Action: func(cliCtx *cli.Context) error {

			require.Equal(t, true,
				cliCtx.Bool(EnableBuilderFlag.Name))
			return nil
		},
	}
	require.NoError(t, command.Run(context, context.Args().Slice()...))
	require.NoError(t, os.Remove("flags_test.yaml"))
}

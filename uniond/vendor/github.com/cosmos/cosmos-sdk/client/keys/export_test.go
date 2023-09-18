package keys

import (
	"bufio"
	"context"
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
	clienttestutil "github.com/cosmos/cosmos-sdk/client/testutil"
	"github.com/cosmos/cosmos-sdk/crypto/hd"
	"github.com/cosmos/cosmos-sdk/testutil"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"

	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func Test_runExportCmd(t *testing.T) {
	cdc := clienttestutil.MakeTestCodec(t)
	testCases := []struct {
		name           string
		keyringBackend string
		extraArgs      []string
		userInput      string
		mustFail       bool
		expectedOutput string
	}{
		{
			name:           "--unsafe only must fail",
			keyringBackend: keyring.BackendTest,
			extraArgs:      []string{"--unsafe"},
			mustFail:       true,
		},
		{
			name:           "--unarmored-hex must fail",
			keyringBackend: keyring.BackendTest,
			extraArgs:      []string{"--unarmored-hex"},
			mustFail:       true,
		},
		{
			name:           "--unsafe --unarmored-hex fail with no user confirmation",
			keyringBackend: keyring.BackendTest,
			extraArgs:      []string{"--unsafe", "--unarmored-hex"},
			userInput:      "",
			mustFail:       true,
			expectedOutput: "",
		},
		{
			name:           "--unsafe --unarmored-hex succeed",
			keyringBackend: keyring.BackendTest,
			extraArgs:      []string{"--unsafe", "--unarmored-hex"},
			userInput:      "y\n",
			mustFail:       false,
			expectedOutput: "2485e33678db4175dc0ecef2d6e1fc493d4a0d7f7ce83324b6ed70afe77f3485\n",
		},
		{
			name:           "file keyring backend properly read password and user confirmation",
			keyringBackend: keyring.BackendFile,
			extraArgs:      []string{"--unsafe", "--unarmored-hex"},
			// first 2 pass for creating the key, then unsafe export confirmation, then unlock keyring pass
			userInput:      "12345678\n12345678\ny\n12345678\n",
			mustFail:       false,
			expectedOutput: "2485e33678db4175dc0ecef2d6e1fc493d4a0d7f7ce83324b6ed70afe77f3485\n",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			kbHome := t.TempDir()
			defaultArgs := []string{
				"keyname1",
				fmt.Sprintf("--%s=%s", flags.FlagHome, kbHome),
				fmt.Sprintf("--%s=%s", flags.FlagKeyringBackend, tc.keyringBackend),
			}

			cmd := ExportKeyCommand()
			cmd.Flags().AddFlagSet(Commands("home").PersistentFlags())

			cmd.SetArgs(append(defaultArgs, tc.extraArgs...))
			mockIn, mockOut := testutil.ApplyMockIO(cmd)

			mockIn.Reset(tc.userInput)
			mockInBuf := bufio.NewReader(mockIn)

			// create a key
			kb, err := keyring.New(sdk.KeyringServiceName(), tc.keyringBackend, kbHome, bufio.NewReader(mockInBuf), cdc)
			require.NoError(t, err)
			t.Cleanup(cleanupKeys(t, kb, "keyname1"))

			path := sdk.GetConfig().GetFullBIP44Path()
			_, err = kb.NewAccount("keyname1", testdata.TestMnemonic, "", path, hd.Secp256k1)
			require.NoError(t, err)

			clientCtx := client.Context{}.
				WithKeyringDir(kbHome).
				WithKeyring(kb).
				WithInput(mockInBuf).
				WithCodec(cdc)
			ctx := context.WithValue(context.Background(), client.ClientContextKey, &clientCtx)

			err = cmd.ExecuteContext(ctx)
			if tc.mustFail {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
				require.Equal(t, tc.expectedOutput, mockOut.String())
			}
		})
	}
}

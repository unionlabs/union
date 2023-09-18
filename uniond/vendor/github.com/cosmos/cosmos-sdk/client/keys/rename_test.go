package keys

import (
	"context"
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	clienttestutil "github.com/cosmos/cosmos-sdk/client/testutil"
	"github.com/cosmos/cosmos-sdk/crypto/hd"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	"github.com/cosmos/cosmos-sdk/testutil"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func Test_runRenameCmd(t *testing.T) {
	// temp keybase
	kbHome := t.TempDir()
	cmd := RenameKeyCommand()
	cmd.Flags().AddFlagSet(Commands(kbHome).PersistentFlags())
	mockIn := testutil.ApplyMockIODiscardOutErr(cmd)

	yesF, _ := cmd.Flags().GetBool(flagYes)
	require.False(t, yesF)

	fakeKeyName1 := "runRenameCmd_Key1"
	fakeKeyName2 := "runRenameCmd_Key2"

	path := sdk.GetConfig().GetFullBIP44Path()

	cdc := clienttestutil.MakeTestCodec(t)
	kb, err := keyring.New(sdk.KeyringServiceName(), keyring.BackendTest, kbHome, mockIn, cdc)
	require.NoError(t, err)

	// put fakeKeyName1 in keyring
	_, err = kb.NewAccount(fakeKeyName1, testdata.TestMnemonic, "", path, hd.Secp256k1)
	require.NoError(t, err)

	clientCtx := client.Context{}.
		WithKeyringDir(kbHome).
		WithCodec(cdc)

	ctx := context.WithValue(context.Background(), client.ClientContextKey, &clientCtx)

	// rename a key 'blah' which doesnt exist
	cmd.SetArgs([]string{"blah", "blaah", fmt.Sprintf("--%s=%s", flags.FlagHome, kbHome)})
	err = cmd.ExecuteContext(ctx)
	require.Error(t, err)
	require.EqualError(t, err, "blah.info: key not found")

	// User confirmation missing
	cmd.SetArgs([]string{
		fakeKeyName1,
		"nokey",
		fmt.Sprintf("--%s=%s", flags.FlagHome, kbHome),
		fmt.Sprintf("--%s=%s", flags.FlagKeyringBackend, keyring.BackendTest),
	})
	err = cmd.Execute()
	require.Error(t, err)
	require.Equal(t, "EOF", err.Error())

	oldKey, err := kb.Key(fakeKeyName1)
	require.NoError(t, err)

	// add a confirmation
	cmd.SetArgs([]string{
		fakeKeyName1,
		fakeKeyName2,
		fmt.Sprintf("--%s=%s", flags.FlagHome, kbHome),
		fmt.Sprintf("--%s=true", flagYes),
		fmt.Sprintf("--%s=%s", flags.FlagKeyringBackend, keyring.BackendTest),
	})
	require.NoError(t, cmd.Execute())

	// key1 is gone
	_, err = kb.Key(fakeKeyName1)
	require.Error(t, err)

	// key2 exists now
	renamedKey, err := kb.Key(fakeKeyName2)
	require.NoError(t, err)
	oldPk, err := oldKey.GetPubKey()
	require.NoError(t, err)
	renamedPk, err := renamedKey.GetPubKey()
	require.NoError(t, err)
	require.Equal(t, oldPk, renamedPk)
	require.Equal(t, oldKey.GetType(), renamedKey.GetType())

	oldAddr, err := oldKey.GetAddress()
	require.NoError(t, err)
	renamedAddr, err := renamedKey.GetAddress()
	require.NoError(t, err)
	require.Equal(t, oldAddr, renamedAddr)

	// try to rename key1 but it doesnt exist anymore so error
	cmd.SetArgs([]string{
		fakeKeyName1,
		fakeKeyName2,
		fmt.Sprintf("--%s=%s", flags.FlagHome, kbHome),
		fmt.Sprintf("--%s=true", flagYes),
		fmt.Sprintf("--%s=%s", flags.FlagKeyringBackend, keyring.BackendTest),
	})
	require.Error(t, cmd.Execute())
}

package keys_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client/keys"
)

func TestConstructors(t *testing.T) {
	require.Equal(t, keys.AddNewKey{
		Name:     "name",
		Password: "password",
		Mnemonic: "mnemonic",
		Account:  1,
		Index:    1,
	}, keys.NewAddNewKey("name", "password", "mnemonic", 1, 1))

	require.Equal(t, keys.RecoverKey{
		Password: "password",
		Mnemonic: "mnemonic",
		Account:  1,
		Index:    1,
	}, keys.NewRecoverKey("password", "mnemonic", 1, 1))

	require.Equal(t, keys.UpdateKeyReq{OldPassword: "old", NewPassword: "new"}, keys.NewUpdateKeyReq("old", "new"))
	require.Equal(t, keys.DeleteKeyReq{Password: "password"}, keys.NewDeleteKeyReq("password"))
}

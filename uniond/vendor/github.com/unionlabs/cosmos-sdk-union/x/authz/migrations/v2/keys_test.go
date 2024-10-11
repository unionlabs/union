package v2

import (
	"testing"

	"github.com/stretchr/testify/require"

	bank "cosmossdk.io/x/bank/types"

	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
)

var (
	granter = sdk.AccAddress(ed25519.GenPrivKey().PubKey().Address())
	grantee = sdk.AccAddress(ed25519.GenPrivKey().PubKey().Address())
	msgType = bank.SendAuthorization{}.MsgTypeURL()
)

func TestGrantkey(t *testing.T) {
	require := require.New(t)
	key := GrantStoreKey(grantee, granter, msgType)
	require.Len(key, len(GrantPrefix)+len(address.MustLengthPrefix(grantee))+len(address.MustLengthPrefix(granter))+len([]byte(msgType)))

	granter1, grantee1, msgType1 := ParseGrantKey(key[1:])
	require.Equal(granter, granter1)
	require.Equal(grantee, grantee1)
	require.Equal(msgType1, msgType)
}

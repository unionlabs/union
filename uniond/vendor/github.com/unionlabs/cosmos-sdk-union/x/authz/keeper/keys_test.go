package keeper

import (
	"testing"
	"time"

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
	key := grantStoreKey(grantee, granter, msgType)
	require.Len(key, len(GrantKey)+len(address.MustLengthPrefix(grantee))+len(address.MustLengthPrefix(granter))+len([]byte(msgType)))

	granter1, grantee1, msgType1 := parseGrantStoreKey(grantStoreKey(grantee, granter, msgType))
	require.Equal(granter, granter1)
	require.Equal(grantee, grantee1)
	require.Equal(msgType1, msgType)
}

func TestGrantQueueKey(t *testing.T) {
	blockTime := time.Now().UTC()
	queueKey := GrantQueueKey(blockTime, granter, grantee)

	expiration, granter1, grantee1, err := parseGrantQueueKey(queueKey)
	require.NoError(t, err)
	require.Equal(t, blockTime, expiration)
	require.Equal(t, granter, granter1)
	require.Equal(t, grantee, grantee1)
}

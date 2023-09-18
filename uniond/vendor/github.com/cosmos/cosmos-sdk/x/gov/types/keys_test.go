package types

import (
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var addr = sdk.AccAddress(ed25519.GenPrivKey().PubKey().Address())

func TestProposalKeys(t *testing.T) {
	// key proposal
	key := ProposalKey(1)
	proposalID := SplitProposalKey(key)
	require.Equal(t, int(proposalID), 1)

	// key active proposal queue
	now := time.Now()
	key = ActiveProposalQueueKey(3, now)
	proposalID, expTime := SplitActiveProposalQueueKey(key)
	require.Equal(t, int(proposalID), 3)
	require.True(t, now.Equal(expTime))

	// key inactive proposal queue
	key = InactiveProposalQueueKey(3, now)
	proposalID, expTime = SplitInactiveProposalQueueKey(key)
	require.Equal(t, int(proposalID), 3)
	require.True(t, now.Equal(expTime))

	// invalid key
	require.Panics(t, func() { SplitProposalKey([]byte("test")) })
	require.Panics(t, func() { SplitInactiveProposalQueueKey([]byte("test")) })
}

func TestDepositKeys(t *testing.T) {
	key := DepositsKey(2)
	proposalID := SplitProposalKey(key)
	require.Equal(t, int(proposalID), 2)

	key = DepositKey(2, addr)
	proposalID, depositorAddr := SplitKeyDeposit(key)
	require.Equal(t, int(proposalID), 2)
	require.Equal(t, addr, depositorAddr)
}

func TestVoteKeys(t *testing.T) {
	key := VotesKey(2)
	proposalID := SplitProposalKey(key)
	require.Equal(t, int(proposalID), 2)

	key = VoteKey(2, addr)
	proposalID, voterAddr := SplitKeyDeposit(key)
	require.Equal(t, int(proposalID), 2)
	require.Equal(t, addr, voterAddr)
}

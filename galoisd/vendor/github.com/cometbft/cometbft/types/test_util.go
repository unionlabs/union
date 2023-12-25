package types

import (
	"fmt"
	"testing"
	"time"

	cmtproto "github.com/cometbft/cometbft/proto/tendermint/types"
	cmtversion "github.com/cometbft/cometbft/proto/tendermint/version"
	"github.com/cometbft/cometbft/version"
	"github.com/stretchr/testify/require"
)

func MakeExtCommit(blockID BlockID, height int64, round int32,
	voteSet *VoteSet, validators []PrivValidator, now time.Time, extEnabled bool) (*ExtendedCommit, error) {

	// all sign
	for i := 0; i < len(validators); i++ {
		pubKey, err := validators[i].GetPubKey()
		if err != nil {
			return nil, fmt.Errorf("can't get pubkey: %w", err)
		}
		vote := &Vote{
			ValidatorAddress: pubKey.Address(),
			ValidatorIndex:   int32(i),
			Height:           height,
			Round:            round,
			Type:             cmtproto.PrecommitType,
			BlockID:          blockID,
			Timestamp:        now,
		}

		_, err = signAddVote(validators[i], vote, voteSet)
		if err != nil {
			return nil, err
		}
	}

	var enableHeight int64
	if extEnabled {
		enableHeight = height
	}

	return voteSet.MakeExtendedCommit(ABCIParams{VoteExtensionsEnableHeight: enableHeight}), nil
}

func signAddVote(privVal PrivValidator, vote *Vote, voteSet *VoteSet) (bool, error) {
	if vote.Type != voteSet.signedMsgType {
		return false, fmt.Errorf("vote and voteset are of different types; %d != %d", vote.Type, voteSet.signedMsgType)
	}
	if _, err := SignAndCheckVote(vote, privVal, voteSet.ChainID(), voteSet.extensionsEnabled); err != nil {
		return false, err
	}
	return voteSet.AddVote(vote)
}

func MakeVote(
	val PrivValidator,
	chainID string,
	valIndex int32,
	height int64,
	round int32,
	step cmtproto.SignedMsgType,
	blockID BlockID,
	time time.Time,
) (*Vote, error) {
	pubKey, err := val.GetPubKey()
	if err != nil {
		return nil, err
	}

	vote := &Vote{
		ValidatorAddress: pubKey.Address(),
		ValidatorIndex:   valIndex,
		Height:           height,
		Round:            round,
		Type:             step,
		BlockID:          blockID,
		Timestamp:        time,
	}

	extensionsEnabled := step == cmtproto.PrecommitType
	if _, err := SignAndCheckVote(vote, val, chainID, extensionsEnabled); err != nil {
		return nil, err
	}

	return vote, nil
}

func MakeVoteNoError(
	t *testing.T,
	val PrivValidator,
	chainID string,
	valIndex int32,
	height int64,
	round int32,
	step cmtproto.SignedMsgType,
	blockID BlockID,
	time time.Time,
) *Vote {
	vote, err := MakeVote(val, chainID, valIndex, height, round, step, blockID, time)
	require.NoError(t, err)
	return vote
}

// MakeBlock returns a new block with an empty header, except what can be
// computed from itself.
// It populates the same set of fields validated by ValidateBasic.
func MakeBlock(height int64, txs []Tx, lastCommit *Commit, evidence []Evidence) *Block {
	block := &Block{
		Header: Header{
			Version: cmtversion.Consensus{Block: version.BlockProtocol, App: 0},
			Height:  height,
		},
		Data: Data{
			Txs: txs,
		},
		Evidence:   EvidenceData{Evidence: evidence},
		LastCommit: lastCommit,
	}
	block.fillHeader()
	return block
}

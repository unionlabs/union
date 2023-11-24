package test

import (
	"fmt"
	"time"

	cmtproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/cometbft/cometbft/types"
)

func MakeCommitFromVoteSet(blockID types.BlockID, voteSet *types.VoteSet, validators []types.PrivValidator, now time.Time) (*types.Commit, error) {
	// all sign
	for i := 0; i < len(validators); i++ {
		pubKey, err := validators[i].GetPubKey()
		if err != nil {
			return nil, err
		}
		vote := &types.Vote{
			ValidatorAddress: pubKey.Address(),
			ValidatorIndex:   int32(i),
			Height:           voteSet.GetHeight(),
			Round:            voteSet.GetRound(),
			Type:             cmtproto.PrecommitType,
			BlockID:          blockID,
			Timestamp:        now,
		}

		v := vote.ToProto()

		if err := validators[i].SignVote(voteSet.ChainID(), v); err != nil {
			return nil, err
		}
		vote.Signature = v.Signature
		if _, err := voteSet.AddVote(vote); err != nil {
			return nil, err
		}
	}

	return voteSet.MakeExtendedCommit(types.ABCIParams{VoteExtensionsEnableHeight: 0}).ToCommit(), nil
}

func MakeCommit(blockID types.BlockID, height int64, round int32, valSet *types.ValidatorSet, privVals []types.PrivValidator, chainID string, now time.Time) (*types.Commit, error) {
	sigs := make([]types.CommitSig, len(valSet.Validators))
	for i := 0; i < len(valSet.Validators); i++ {
		sigs[i] = types.NewCommitSigAbsent()
	}

	for _, privVal := range privVals {
		pk, err := privVal.GetPubKey()
		if err != nil {
			return nil, err
		}
		addr := pk.Address()

		idx, _ := valSet.GetByAddress(addr)
		if idx < 0 {
			return nil, fmt.Errorf("validator with address %s not in validator set", addr)
		}

		vote := &types.Vote{
			ValidatorAddress: addr,
			ValidatorIndex:   idx,
			Height:           height,
			Round:            round,
			Type:             cmtproto.PrecommitType,
			BlockID:          blockID,
			Timestamp:        now,
		}

		v := vote.ToProto()

		if err := privVal.SignVote(chainID, v); err != nil {
			return nil, err
		}

		sigs[idx] = types.CommitSig{
			BlockIDFlag:      types.BlockIDFlagCommit,
			ValidatorAddress: addr,
			Timestamp:        now,
			Signature:        v.Signature,
		}
	}

	return &types.Commit{Height: height, Round: round, BlockID: blockID, Signatures: sigs}, nil
}

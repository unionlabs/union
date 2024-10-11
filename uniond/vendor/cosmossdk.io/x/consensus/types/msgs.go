package types

import (
	"errors"

	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	cmttypes "github.com/cometbft/cometbft/types"
	"github.com/cosmos/gogoproto/types"
)

// ToProtoConsensusParams converts MsgUpdateParams to cmtproto.ConsensusParams.
// It returns an error if any required parameters are missing or if there's a conflict
// between ABCI and Feature parameters.
func (msg MsgUpdateParams) ToProtoConsensusParams() (cmtproto.ConsensusParams, error) {
	if msg.Evidence == nil || msg.Block == nil || msg.Validator == nil {
		return cmtproto.ConsensusParams{}, errors.New("all parameters must be present")
	}

	if msg.Abci != nil && msg.Feature != nil && msg.Feature.VoteExtensionsEnableHeight != nil {
		return cmtproto.ConsensusParams{}, errors.New("abci in sections Feature and (deprecated) ABCI cannot be used simultaneously")
	}

	cp := cmtproto.ConsensusParams{
		Block: &cmtproto.BlockParams{
			MaxBytes: msg.Block.MaxBytes,
			MaxGas:   msg.Block.MaxGas,
		},
		Evidence: &cmtproto.EvidenceParams{
			MaxAgeNumBlocks: msg.Evidence.MaxAgeNumBlocks,
			MaxAgeDuration:  msg.Evidence.MaxAgeDuration,
			MaxBytes:        msg.Evidence.MaxBytes,
		},
		Validator: &cmtproto.ValidatorParams{
			PubKeyTypes: msg.Validator.PubKeyTypes,
		},
		Version:   cmttypes.DefaultConsensusParams().ToProto().Version, // Version is stored in x/upgrade
		Feature:   msg.Feature,
		Synchrony: msg.Synchrony,
	}

	if msg.Abci != nil {
		if cp.Feature == nil {
			cp.Feature = &cmtproto.FeatureParams{}
		}

		cp.Feature.VoteExtensionsEnableHeight = &types.Int64Value{
			Value: msg.Abci.VoteExtensionsEnableHeight,
		}
	}

	return cp, nil
}

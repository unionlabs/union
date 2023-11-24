package types

import (
	"errors"

	cmtproto "github.com/cometbft/cometbft/proto/tendermint/types"
	cmttypes "github.com/cometbft/cometbft/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

var _ sdk.Msg = &MsgUpdateParams{}

func (msg MsgUpdateParams) ToProtoConsensusParams() (cmtproto.ConsensusParams, error) {
	if msg.Evidence == nil || msg.Block == nil || msg.Validator == nil {
		return cmtproto.ConsensusParams{}, errors.New("all parameters must be present")
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
		Version: cmttypes.DefaultConsensusParams().ToProto().Version, // Version is stored in x/upgrade
	}

	if msg.Abci != nil {
		cp.Abci = &cmtproto.ABCIParams{
			VoteExtensionsEnableHeight: msg.Abci.VoteExtensionsEnableHeight,
		}
	}

	return cp, nil
}

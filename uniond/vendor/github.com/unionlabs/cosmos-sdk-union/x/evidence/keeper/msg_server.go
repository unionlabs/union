package keeper

import (
	"context"

	"cosmossdk.io/errors"
	"cosmossdk.io/x/evidence/types"

	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

type msgServer struct {
	Keeper
}

// NewMsgServerImpl returns an implementation of the bank MsgServer interface
// for the provided Keeper.
func NewMsgServerImpl(keeper Keeper) types.MsgServer {
	return &msgServer{Keeper: keeper}
}

var _ types.MsgServer = msgServer{}

// SubmitEvidence implements the MsgServer.SubmitEvidence method.
func (ms msgServer) SubmitEvidence(ctx context.Context, msg *types.MsgSubmitEvidence) (*types.MsgSubmitEvidenceResponse, error) {
	if _, err := ms.addressCodec.StringToBytes(msg.Submitter); err != nil {
		return nil, sdkerrors.ErrInvalidAddress.Wrapf("invalid submitter address: %s", err)
	}

	evidence := msg.GetEvidence()
	if evidence == nil {
		return nil, errors.Wrap(types.ErrInvalidEvidence, "missing evidence")
	}

	if err := evidence.ValidateBasic(); err != nil {
		return nil, errors.Wrapf(types.ErrInvalidEvidence, "failed basic validation: %s", err)
	}

	if err := ms.Keeper.SubmitEvidence(ctx, evidence); err != nil {
		return nil, err
	}

	return &types.MsgSubmitEvidenceResponse{
		Hash: evidence.Hash(),
	}, nil
}

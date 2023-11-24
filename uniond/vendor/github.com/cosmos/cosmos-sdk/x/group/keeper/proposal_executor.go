package keeper

import (
	"bytes"
	"fmt"

	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/x/group"
	"github.com/cosmos/cosmos-sdk/x/group/errors"
)

// doExecuteMsgs routes the messages to the registered handlers. Messages are limited to those that require no authZ or
// by the account of group policy only. Otherwise this gives access to other peoples accounts as the sdk middlewares are bypassed
func (s Keeper) doExecuteMsgs(ctx sdk.Context, router baseapp.MessageRouter, proposal group.Proposal, groupPolicyAcc sdk.AccAddress, decisionPolicy group.DecisionPolicy) ([]sdk.Result, error) {
	// Ensure it's not too early to execute the messages.
	minExecutionDate := proposal.SubmitTime.Add(decisionPolicy.GetMinExecutionPeriod())
	if ctx.BlockTime().Before(minExecutionDate) {
		return nil, errors.ErrInvalid.Wrapf("must wait until %s to execute proposal %d", minExecutionDate, proposal.Id)
	}

	// Ensure it's not too late to execute the messages.
	// After https://github.com/cosmos/cosmos-sdk/issues/11245, proposals should
	// be pruned automatically, so this function should not even be called, as
	// the proposal doesn't exist in state. For sanity check, we can still keep
	// this simple and cheap check.
	expiryDate := proposal.VotingPeriodEnd.Add(s.config.MaxExecutionPeriod)
	if expiryDate.Before(ctx.BlockTime()) {
		return nil, errors.ErrExpired.Wrapf("proposal expired on %s", expiryDate)
	}

	msgs, err := proposal.GetMsgs()
	if err != nil {
		return nil, err
	}

	results := make([]sdk.Result, len(msgs))
	if err := ensureMsgAuthZ(msgs, groupPolicyAcc, s.cdc); err != nil {
		return nil, err
	}
	for i, msg := range msgs {
		handler := s.router.Handler(msg)
		if handler == nil {
			return nil, errorsmod.Wrapf(errors.ErrInvalid, "no message handler found for %q", sdk.MsgTypeURL(msg))
		}
		r, err := handler(ctx, msg)
		if err != nil {
			return nil, errorsmod.Wrapf(err, "message %s at position %d", sdk.MsgTypeURL(msg), i)
		}
		// Handler should always return non-nil sdk.Result.
		if r == nil {
			return nil, fmt.Errorf("got nil sdk.Result for message %q at position %d", msg, i)
		}

		results[i] = *r
	}
	return results, nil
}

// ensureMsgAuthZ checks that if a message requires signers that all of them
// are equal to the given account address of group policy.
func ensureMsgAuthZ(msgs []sdk.Msg, groupPolicyAcc sdk.AccAddress, cdc codec.Codec) error {
	for i := range msgs {
		// In practice, GetSigners() should return a non-empty array without
		// duplicates, so the code below is equivalent to:
		// `msgs[i].GetSigners()[0] == groupPolicyAcc`
		// but we prefer to loop through all GetSigners just to be sure.
		signers, _, err := cdc.GetMsgV1Signers(msgs[i])
		if err != nil {
			return err
		}

		for _, acct := range signers {
			if !bytes.Equal(groupPolicyAcc, acct) {
				return errorsmod.Wrapf(sdkerrors.ErrUnauthorized, "msg does not have group policy authorization; expected %s, got %s", groupPolicyAcc.String(), acct)
			}
		}
	}
	return nil
}

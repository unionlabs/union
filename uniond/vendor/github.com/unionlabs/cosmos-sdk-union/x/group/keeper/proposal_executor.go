package keeper

import (
	"bytes"
	"context"

	"cosmossdk.io/core/address"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/x/group"
	"cosmossdk.io/x/group/errors"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// doExecuteMsgs routes the messages to the registered handlers. Messages are limited to those that require no authZ or
// by the account of group policy only. Otherwise this gives access to other peoples accounts as the sdk middlewares are bypassed
func (k Keeper) doExecuteMsgs(ctx context.Context, proposal group.Proposal, groupPolicyAcc sdk.AccAddress, decisionPolicy group.DecisionPolicy) error {
	currentTime := k.HeaderService.HeaderInfo(ctx).Time

	// Ensure it's not too early to execute the messages.
	minExecutionDate := proposal.SubmitTime.Add(decisionPolicy.GetMinExecutionPeriod())
	if currentTime.Before(minExecutionDate) {
		return errors.ErrInvalid.Wrapf("must wait until %s to execute proposal %d", minExecutionDate, proposal.Id)
	}

	// Ensure it's not too late to execute the messages.
	// After https://github.com/cosmos/cosmos-sdk/issues/11245, proposals should
	// be pruned automatically, so this function should not even be called, as
	// the proposal doesn't exist in state. For sanity check, we can still keep
	// this simple and cheap check.
	expiryDate := proposal.VotingPeriodEnd.Add(k.config.MaxExecutionPeriod)
	if expiryDate.Before(currentTime) {
		return errors.ErrExpired.Wrapf("proposal expired on %s", expiryDate)
	}

	msgs, err := proposal.GetMsgs()
	if err != nil {
		return err
	}

	if err := ensureMsgAuthZ(msgs, groupPolicyAcc, k.cdc, k.accKeeper.AddressCodec()); err != nil {
		return err
	}

	for i, msg := range msgs {
		if _, err := k.MsgRouterService.Invoke(ctx, msg); err != nil {
			return errorsmod.Wrapf(err, "message %s at position %d", sdk.MsgTypeURL(msg), i)
		}
	}
	return nil
}

// ensureMsgAuthZ checks that if a message requires signers that all of them
// are equal to the given account address of group policy.
func ensureMsgAuthZ(msgs []sdk.Msg, groupPolicyAcc sdk.AccAddress, cdc codec.Codec, addressCodec address.Codec) error {
	for i := range msgs {
		// In practice, GetMsgV1Signers should return a non-empty array without duplicates.
		signers, _, err := cdc.GetMsgSigners(msgs[i])
		if err != nil {
			return err
		}

		// The code below should be equivalent to: `signers[0] == groupPolicyAcc`
		// But here, we loop through all the signers just to be sure.
		for _, acct := range signers {
			if !bytes.Equal(groupPolicyAcc, acct) {
				groupPolicyAddr, err := addressCodec.BytesToString(groupPolicyAcc)
				if err != nil {
					return errorsmod.Wrapf(sdkerrors.ErrUnauthorized, "msg does not have group policy authorization; error retrieving group policy address")
				}
				return errorsmod.Wrapf(sdkerrors.ErrUnauthorized, "msg does not have group policy authorization; expected %s, got %s", groupPolicyAddr, acct)
			}
		}
	}
	return nil
}

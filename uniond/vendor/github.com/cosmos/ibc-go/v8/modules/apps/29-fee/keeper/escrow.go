package keeper

import (
	"bytes"
	"fmt"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/apps/29-fee/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
)

// escrowPacketFee sends the packet fee to the 29-fee module account to hold in escrow
func (k Keeper) escrowPacketFee(ctx sdk.Context, packetID channeltypes.PacketId, packetFee types.PacketFee) error {
	// check if the refund address is valid
	refundAddr, err := sdk.AccAddressFromBech32(packetFee.RefundAddress)
	if err != nil {
		return err
	}

	refundAcc := k.authKeeper.GetAccount(ctx, refundAddr)
	if refundAcc == nil {
		return errorsmod.Wrapf(types.ErrRefundAccNotFound, "account with address: %s not found", packetFee.RefundAddress)
	}

	coins := packetFee.Fee.Total()
	if err := k.bankKeeper.SendCoinsFromAccountToModule(ctx, refundAddr, types.ModuleName, coins); err != nil {
		return err
	}

	// multiple fees may be escrowed for a single packet, firstly create a slice containing the new fee
	// retrieve any previous fees stored in escrow for the packet and append them to the list
	fees := []types.PacketFee{packetFee}
	if feesInEscrow, found := k.GetFeesInEscrow(ctx, packetID); found {
		fees = append(fees, feesInEscrow.PacketFees...)
	}

	packetFees := types.NewPacketFees(fees)
	k.SetFeesInEscrow(ctx, packetID, packetFees)

	emitIncentivizedPacketEvent(ctx, packetID, packetFees)

	return nil
}

// DistributePacketFeesOnAcknowledgement pays all the acknowledgement & receive fees for a given packetID while refunding the timeout fees to the refund account.
func (k Keeper) DistributePacketFeesOnAcknowledgement(ctx sdk.Context, forwardRelayer string, reverseRelayer sdk.AccAddress, packetFees []types.PacketFee, packetID channeltypes.PacketId) {
	// cache context before trying to distribute fees
	// if the escrow account has insufficient balance then we want to avoid partially distributing fees
	cacheCtx, writeFn := ctx.CacheContext()

	// forward relayer address will be empty if conversion fails
	forwardAddr, _ := sdk.AccAddressFromBech32(forwardRelayer)

	for _, packetFee := range packetFees {
		if !k.EscrowAccountHasBalance(cacheCtx, packetFee.Fee.Total()) {
			// if the escrow account does not have sufficient funds then there must exist a severe bug
			// the fee module should be locked until manual intervention fixes the issue
			// a locked fee module will simply skip fee logic, all channels will temporarily function as
			// fee disabled channels
			// NOTE: we use the uncached context to lock the fee module so that the state changes from
			// locking the fee module are persisted
			k.lockFeeModule(ctx)
			return
		}

		// check if refundAcc address works
		refundAddr, err := sdk.AccAddressFromBech32(packetFee.RefundAddress)
		if err != nil {
			panic(fmt.Errorf("could not parse refundAcc %s to sdk.AccAddress", packetFee.RefundAddress))
		}

		k.distributePacketFeeOnAcknowledgement(cacheCtx, refundAddr, forwardAddr, reverseRelayer, packetFee)
	}

	// write the cache
	writeFn()

	// removes the fees from the store as fees are now paid
	k.DeleteFeesInEscrow(ctx, packetID)
}

// distributePacketFeeOnAcknowledgement pays the receive fee for a given packetID while refunding the timeout fee to the refund account associated with the Fee.
// If there was no forward relayer or the associated forward relayer address is blocked, the receive fee is refunded.
func (k Keeper) distributePacketFeeOnAcknowledgement(ctx sdk.Context, refundAddr, forwardRelayer, reverseRelayer sdk.AccAddress, packetFee types.PacketFee) {
	// distribute fee to valid forward relayer address otherwise refund the fee
	if !forwardRelayer.Empty() && !k.bankKeeper.BlockedAddr(forwardRelayer) {
		// distribute fee for forward relaying
		k.distributeFee(ctx, forwardRelayer, refundAddr, packetFee.Fee.RecvFee)
	} else {
		// refund onRecv fee as forward relayer is not valid address
		k.distributeFee(ctx, refundAddr, refundAddr, packetFee.Fee.RecvFee)
	}

	// distribute fee for reverse relaying
	k.distributeFee(ctx, reverseRelayer, refundAddr, packetFee.Fee.AckFee)

	// refund unused amount from the escrowed fee
	refundCoins := packetFee.Fee.Total().Sub(packetFee.Fee.RecvFee...).Sub(packetFee.Fee.AckFee...)
	k.distributeFee(ctx, refundAddr, refundAddr, refundCoins)
}

// DistributePacketsFeesOnTimeout pays all the timeout fees for a given packetID while refunding the acknowledgement & receive fees to the refund account.
func (k Keeper) DistributePacketFeesOnTimeout(ctx sdk.Context, timeoutRelayer sdk.AccAddress, packetFees []types.PacketFee, packetID channeltypes.PacketId) {
	// cache context before trying to distribute fees
	// if the escrow account has insufficient balance then we want to avoid partially distributing fees
	cacheCtx, writeFn := ctx.CacheContext()

	for _, packetFee := range packetFees {
		if !k.EscrowAccountHasBalance(cacheCtx, packetFee.Fee.Total()) {
			// if the escrow account does not have sufficient funds then there must exist a severe bug
			// the fee module should be locked until manual intervention fixes the issue
			// a locked fee module will simply skip fee logic, all channels will temporarily function as
			// fee disabled channels
			// NOTE: we use the uncached context to lock the fee module so that the state changes from
			// locking the fee module are persisted
			k.lockFeeModule(ctx)
			return
		}

		// check if refundAcc address works
		refundAddr, err := sdk.AccAddressFromBech32(packetFee.RefundAddress)
		if err != nil {
			panic(fmt.Errorf("could not parse refundAcc %s to sdk.AccAddress", packetFee.RefundAddress))
		}

		k.distributePacketFeeOnTimeout(cacheCtx, refundAddr, timeoutRelayer, packetFee)
	}

	// write the cache
	writeFn()

	// removing the fee from the store as the fee is now paid
	k.DeleteFeesInEscrow(ctx, packetID)
}

// distributePacketFeeOnTimeout pays the timeout fee to the timeout relayer and refunds the acknowledgement & receive fee.
func (k Keeper) distributePacketFeeOnTimeout(ctx sdk.Context, refundAddr, timeoutRelayer sdk.AccAddress, packetFee types.PacketFee) {
	// distribute fee for timeout relaying
	k.distributeFee(ctx, timeoutRelayer, refundAddr, packetFee.Fee.TimeoutFee)

	// refund unused amount from the escrowed fee
	refundCoins := packetFee.Fee.Total().Sub(packetFee.Fee.TimeoutFee...)
	k.distributeFee(ctx, refundAddr, refundAddr, refundCoins)
}

// distributeFee will attempt to distribute the escrowed fee to the receiver address.
// If the distribution fails for any reason (such as the receiving address being blocked),
// the state changes will be discarded.
func (k Keeper) distributeFee(ctx sdk.Context, receiver, refundAccAddress sdk.AccAddress, fee sdk.Coins) {
	// cache context before trying to distribute fees
	cacheCtx, writeFn := ctx.CacheContext()

	err := k.bankKeeper.SendCoinsFromModuleToAccount(cacheCtx, types.ModuleName, receiver, fee)
	if err != nil {
		if bytes.Equal(receiver, refundAccAddress) {
			k.Logger(ctx).Error("error distributing fee", "receiver address", receiver, "fee", fee)
			return // if sending to the refund address already failed, then return (no-op)
		}

		// if an error is returned from x/bank and the receiver is not the refundAccAddress
		// then attempt to refund the fee to the original sender
		err := k.bankKeeper.SendCoinsFromModuleToAccount(cacheCtx, types.ModuleName, refundAccAddress, fee)
		if err != nil {
			k.Logger(ctx).Error("error refunding fee to the original sender", "refund address", refundAccAddress, "fee", fee)
			return // if sending to the refund address fails, no-op
		}

		emitDistributeFeeEvent(ctx, refundAccAddress.String(), fee)
	} else {
		emitDistributeFeeEvent(ctx, receiver.String(), fee)
	}

	// write the cache
	writeFn()
}

// RefundFeesOnChannelClosure will refund all fees associated with the given port and channel identifiers.
// If the escrow account runs out of balance then fee module will become locked as this implies the presence
// of a severe bug. When the fee module is locked, no fee distributions will be performed.
// Please see ADR 004 for more information.
func (k Keeper) RefundFeesOnChannelClosure(ctx sdk.Context, portID, channelID string) error {
	identifiedPacketFees := k.GetIdentifiedPacketFeesForChannel(ctx, portID, channelID)

	// cache context before trying to distribute fees
	// if the escrow account has insufficient balance then we want to avoid partially distributing fees
	cacheCtx, writeFn := ctx.CacheContext()

	for _, identifiedPacketFee := range identifiedPacketFees {
		var failedToSendCoins bool
		for _, packetFee := range identifiedPacketFee.PacketFees {

			if !k.EscrowAccountHasBalance(cacheCtx, packetFee.Fee.Total()) {
				// if the escrow account does not have sufficient funds then there must exist a severe bug
				// the fee module should be locked until manual intervention fixes the issue
				// a locked fee module will simply skip fee logic, all channels will temporarily function as
				// fee disabled channels
				// NOTE: we use the uncached context to lock the fee module so that the state changes from
				// locking the fee module are persisted
				k.lockFeeModule(ctx)

				// return a nil error so state changes are committed but distribution stops
				return nil
			}

			refundAddr, err := sdk.AccAddressFromBech32(packetFee.RefundAddress)
			if err != nil {
				failedToSendCoins = true
				continue
			}

			// refund all fees to refund address
			if err = k.bankKeeper.SendCoinsFromModuleToAccount(cacheCtx, types.ModuleName, refundAddr, packetFee.Fee.Total()); err != nil {
				failedToSendCoins = true
				continue
			}
		}

		if !failedToSendCoins {
			k.DeleteFeesInEscrow(cacheCtx, identifiedPacketFee.PacketId)
		}
	}

	// write the cache
	writeFn()

	return nil
}

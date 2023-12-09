package keeper

import (
	"fmt"
	"strings"

	metrics "github.com/hashicorp/go-metrics"

	errorsmod "cosmossdk.io/errors"
	sdkmath "cosmossdk.io/math"

	"github.com/cosmos/cosmos-sdk/telemetry"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
	coretypes "github.com/cosmos/ibc-go/v8/modules/core/types"
)

// sendTransfer handles transfer sending logic. There are 2 possible cases:
//
// 1. Sender chain is acting as the source zone. The coins are transferred
// to an escrow address (i.e locked) on the sender chain and then transferred
// to the receiving chain through IBC TAO logic. It is expected that the
// receiving chain will mint vouchers to the receiving address.
//
// 2. Sender chain is acting as the sink zone. The coins (vouchers) are burned
// on the sender chain and then transferred to the receiving chain though IBC
// TAO logic. It is expected that the receiving chain, which had previously
// sent the original denomination, will unescrow the fungible token and send
// it to the receiving address.
//
// Another way of thinking of source and sink zones is through the token's
// timeline. Each send to any chain other than the one it was previously
// received from is a movement forwards in the token's timeline. This causes
// trace to be added to the token's history and the destination port and
// destination channel to be prefixed to the denomination. In these instances
// the sender chain is acting as the source zone. When the token is sent back
// to the chain it previously received from, the prefix is removed. This is
// a backwards movement in the token's timeline and the sender chain
// is acting as the sink zone.
//
// Example:
// These steps of transfer occur: A -> B -> C -> A -> C -> B -> A
//
// 1. A -> B : sender chain is source zone. Denom upon receiving: 'B/denom'
// 2. B -> C : sender chain is source zone. Denom upon receiving: 'C/B/denom'
// 3. C -> A : sender chain is source zone. Denom upon receiving: 'A/C/B/denom'
// 4. A -> C : sender chain is sink zone. Denom upon receiving: 'C/B/denom'
// 5. C -> B : sender chain is sink zone. Denom upon receiving: 'B/denom'
// 6. B -> A : sender chain is sink zone. Denom upon receiving: 'denom'
func (k Keeper) sendTransfer(
	ctx sdk.Context,
	sourcePort,
	sourceChannel string,
	token sdk.Coin,
	sender sdk.AccAddress,
	receiver string,
	timeoutHeight clienttypes.Height,
	timeoutTimestamp uint64,
	memo string,
) (uint64, error) {
	channel, found := k.channelKeeper.GetChannel(ctx, sourcePort, sourceChannel)
	if !found {
		return 0, errorsmod.Wrapf(channeltypes.ErrChannelNotFound, "port ID (%s) channel ID (%s)", sourcePort, sourceChannel)
	}

	destinationPort := channel.GetCounterparty().GetPortID()
	destinationChannel := channel.GetCounterparty().GetChannelID()

	// begin createOutgoingPacket logic
	// See spec for this logic: https://github.com/cosmos/ibc/tree/master/spec/app/ics-020-fungible-token-transfer#packet-relay
	channelCap, ok := k.scopedKeeper.GetCapability(ctx, host.ChannelCapabilityPath(sourcePort, sourceChannel))
	if !ok {
		return 0, errorsmod.Wrap(channeltypes.ErrChannelCapabilityNotFound, "module does not own channel capability")
	}

	// NOTE: denomination and hex hash correctness checked during msg.ValidateBasic
	fullDenomPath := token.Denom

	var err error

	// deconstruct the token denomination into the denomination trace info
	// to determine if the sender is the source chain
	if strings.HasPrefix(token.Denom, "ibc/") {
		fullDenomPath, err = k.DenomPathFromHash(ctx, token.Denom)
		if err != nil {
			return 0, err
		}
	}

	labels := []metrics.Label{
		telemetry.NewLabel(coretypes.LabelDestinationPort, destinationPort),
		telemetry.NewLabel(coretypes.LabelDestinationChannel, destinationChannel),
	}

	// NOTE: SendTransfer simply sends the denomination as it exists on its own
	// chain inside the packet data. The receiving chain will perform denom
	// prefixing as necessary.

	if types.SenderChainIsSource(sourcePort, sourceChannel, fullDenomPath) {
		labels = append(labels, telemetry.NewLabel(coretypes.LabelSource, "true"))

		// obtain the escrow address for the source channel end
		escrowAddress := types.GetEscrowAddress(sourcePort, sourceChannel)
		if err := k.escrowToken(ctx, sender, escrowAddress, token); err != nil {
			return 0, err
		}

	} else {
		labels = append(labels, telemetry.NewLabel(coretypes.LabelSource, "false"))

		// transfer the coins to the module account and burn them
		if err := k.bankKeeper.SendCoinsFromAccountToModule(
			ctx, sender, types.ModuleName, sdk.NewCoins(token),
		); err != nil {
			return 0, err
		}

		if err := k.bankKeeper.BurnCoins(
			ctx, types.ModuleName, sdk.NewCoins(token),
		); err != nil {
			// NOTE: should not happen as the module account was
			// retrieved on the step above and it has enough balace
			// to burn.
			panic(fmt.Errorf("cannot burn coins after a successful send to a module account: %v", err))
		}
	}

	packetData := types.NewFungibleTokenPacketData(
		fullDenomPath, token.Amount.String(), sender.String(), receiver, memo,
	)

	sequence, err := k.ics4Wrapper.SendPacket(ctx, channelCap, sourcePort, sourceChannel, timeoutHeight, timeoutTimestamp, packetData.GetBytes())
	if err != nil {
		return 0, err
	}

	defer func() {
		if token.Amount.IsInt64() {
			telemetry.SetGaugeWithLabels(
				[]string{"tx", "msg", "ibc", "transfer"},
				float32(token.Amount.Int64()),
				[]metrics.Label{telemetry.NewLabel(coretypes.LabelDenom, fullDenomPath)},
			)
		}

		telemetry.IncrCounterWithLabels(
			[]string{"ibc", types.ModuleName, "send"},
			1,
			labels,
		)
	}()

	return sequence, nil
}

// OnRecvPacket processes a cross chain fungible token transfer. If the
// sender chain is the source of minted tokens then vouchers will be minted
// and sent to the receiving address. Otherwise if the sender chain is sending
// back tokens this chain originally transferred to it, the tokens are
// unescrowed and sent to the receiving address.
func (k Keeper) OnRecvPacket(ctx sdk.Context, packet channeltypes.Packet, data types.FungibleTokenPacketData) error {
	// validate packet data upon receiving
	if err := data.ValidateBasic(); err != nil {
		return errorsmod.Wrapf(err, "error validating ICS-20 transfer packet data")
	}

	if !k.GetParams(ctx).ReceiveEnabled {
		return types.ErrReceiveDisabled
	}

	// decode the receiver address
	receiver, err := sdk.AccAddressFromBech32(data.Receiver)
	if err != nil {
		return errorsmod.Wrapf(err, "failed to decode receiver address: %s", data.Receiver)
	}

	// parse the transfer amount
	transferAmount, ok := sdkmath.NewIntFromString(data.Amount)
	if !ok {
		return errorsmod.Wrapf(types.ErrInvalidAmount, "unable to parse transfer amount: %s", data.Amount)
	}

	labels := []metrics.Label{
		telemetry.NewLabel(coretypes.LabelSourcePort, packet.GetSourcePort()),
		telemetry.NewLabel(coretypes.LabelSourceChannel, packet.GetSourceChannel()),
	}

	// This is the prefix that would have been prefixed to the denomination
	// on sender chain IF and only if the token originally came from the
	// receiving chain.
	//
	// NOTE: We use SourcePort and SourceChannel here, because the counterparty
	// chain would have prefixed with DestPort and DestChannel when originally
	// receiving this coin as seen in the "sender chain is the source" condition.
	if types.ReceiverChainIsSource(packet.GetSourcePort(), packet.GetSourceChannel(), data.Denom) {
		// sender chain is not the source, unescrow tokens

		// remove prefix added by sender chain
		voucherPrefix := types.GetDenomPrefix(packet.GetSourcePort(), packet.GetSourceChannel())
		unprefixedDenom := data.Denom[len(voucherPrefix):]

		// coin denomination used in sending from the escrow address
		denom := unprefixedDenom

		// The denomination used to send the coins is either the native denom or the hash of the path
		// if the denomination is not native.
		denomTrace := types.ParseDenomTrace(unprefixedDenom)
		if !denomTrace.IsNativeDenom() {
			denom = denomTrace.IBCDenom()
		}
		token := sdk.NewCoin(denom, transferAmount)

		if k.bankKeeper.BlockedAddr(receiver) {
			return errorsmod.Wrapf(ibcerrors.ErrUnauthorized, "%s is not allowed to receive funds", receiver)
		}

		escrowAddress := types.GetEscrowAddress(packet.GetDestPort(), packet.GetDestChannel())
		if err := k.unescrowToken(ctx, escrowAddress, receiver, token); err != nil {
			return err
		}

		defer func() {
			if transferAmount.IsInt64() {
				telemetry.SetGaugeWithLabels(
					[]string{"ibc", types.ModuleName, "packet", "receive"},
					float32(transferAmount.Int64()),
					[]metrics.Label{telemetry.NewLabel(coretypes.LabelDenom, unprefixedDenom)},
				)
			}

			telemetry.IncrCounterWithLabels(
				[]string{"ibc", types.ModuleName, "receive"},
				1,
				append(
					labels, telemetry.NewLabel(coretypes.LabelSource, "true"),
				),
			)
		}()

		return nil
	}

	// sender chain is the source, mint vouchers

	// since SendPacket did not prefix the denomination, we must prefix denomination here
	sourcePrefix := types.GetDenomPrefix(packet.GetDestPort(), packet.GetDestChannel())
	// NOTE: sourcePrefix contains the trailing "/"
	prefixedDenom := sourcePrefix + data.Denom

	// construct the denomination trace from the full raw denomination
	denomTrace := types.ParseDenomTrace(prefixedDenom)

	traceHash := denomTrace.Hash()
	if !k.HasDenomTrace(ctx, traceHash) {
		k.SetDenomTrace(ctx, denomTrace)
	}

	voucherDenom := denomTrace.IBCDenom()
	if !k.bankKeeper.HasDenomMetaData(ctx, voucherDenom) {
		k.setDenomMetadata(ctx, denomTrace)
	}

	ctx.EventManager().EmitEvent(
		sdk.NewEvent(
			types.EventTypeDenomTrace,
			sdk.NewAttribute(types.AttributeKeyTraceHash, traceHash.String()),
			sdk.NewAttribute(types.AttributeKeyDenom, voucherDenom),
		),
	)
	voucher := sdk.NewCoin(voucherDenom, transferAmount)

	// mint new tokens if the source of the transfer is the same chain
	if err := k.bankKeeper.MintCoins(
		ctx, types.ModuleName, sdk.NewCoins(voucher),
	); err != nil {
		return errorsmod.Wrap(err, "failed to mint IBC tokens")
	}

	// send to receiver
	if err := k.bankKeeper.SendCoinsFromModuleToAccount(
		ctx, types.ModuleName, receiver, sdk.NewCoins(voucher),
	); err != nil {
		return errorsmod.Wrapf(err, "failed to send coins to receiver %s", receiver.String())
	}

	defer func() {
		if transferAmount.IsInt64() {
			telemetry.SetGaugeWithLabels(
				[]string{"ibc", types.ModuleName, "packet", "receive"},
				float32(transferAmount.Int64()),
				[]metrics.Label{telemetry.NewLabel(coretypes.LabelDenom, data.Denom)},
			)
		}

		telemetry.IncrCounterWithLabels(
			[]string{"ibc", types.ModuleName, "receive"},
			1,
			append(
				labels, telemetry.NewLabel(coretypes.LabelSource, "false"),
			),
		)
	}()

	return nil
}

// OnAcknowledgementPacket responds to the the success or failure of a packet
// acknowledgement written on the receiving chain. If the acknowledgement
// was a success then nothing occurs. If the acknowledgement failed, then
// the sender is refunded their tokens using the refundPacketToken function.
func (k Keeper) OnAcknowledgementPacket(ctx sdk.Context, packet channeltypes.Packet, data types.FungibleTokenPacketData, ack channeltypes.Acknowledgement) error {
	switch ack.Response.(type) {
	case *channeltypes.Acknowledgement_Error:
		return k.refundPacketToken(ctx, packet, data)
	default:
		// the acknowledgement succeeded on the receiving chain so nothing
		// needs to be executed and no error needs to be returned
		return nil
	}
}

// OnTimeoutPacket refunds the sender since the original packet sent was
// never received and has been timed out.
func (k Keeper) OnTimeoutPacket(ctx sdk.Context, packet channeltypes.Packet, data types.FungibleTokenPacketData) error {
	return k.refundPacketToken(ctx, packet, data)
}

// refundPacketToken will unescrow and send back the tokens back to sender
// if the sending chain was the source chain. Otherwise, the sent tokens
// were burnt in the original send so new tokens are minted and sent to
// the sending address.
func (k Keeper) refundPacketToken(ctx sdk.Context, packet channeltypes.Packet, data types.FungibleTokenPacketData) error {
	// NOTE: packet data type already checked in handler.go

	// parse the denomination from the full denom path
	trace := types.ParseDenomTrace(data.Denom)

	// parse the transfer amount
	transferAmount, ok := sdkmath.NewIntFromString(data.Amount)
	if !ok {
		return errorsmod.Wrapf(types.ErrInvalidAmount, "unable to parse transfer amount (%s) into math.Int", data.Amount)
	}
	token := sdk.NewCoin(trace.IBCDenom(), transferAmount)

	// decode the sender address
	sender, err := sdk.AccAddressFromBech32(data.Sender)
	if err != nil {
		return err
	}

	if types.SenderChainIsSource(packet.GetSourcePort(), packet.GetSourceChannel(), data.Denom) {
		// unescrow tokens back to sender
		escrowAddress := types.GetEscrowAddress(packet.GetSourcePort(), packet.GetSourceChannel())
		return k.unescrowToken(ctx, escrowAddress, sender, token)
	}

	// mint vouchers back to sender
	if err := k.bankKeeper.MintCoins(
		ctx, types.ModuleName, sdk.NewCoins(token),
	); err != nil {
		return err
	}

	if err := k.bankKeeper.SendCoinsFromModuleToAccount(ctx, types.ModuleName, sender, sdk.NewCoins(token)); err != nil {
		panic(fmt.Errorf("unable to send coins from module to account despite previously minting coins to module account: %v", err))
	}

	return nil
}

// escrowToken will send the given token from the provided sender to the escrow address. It will also
// update the total escrowed amount by adding the escrowed token to the current total escrow.
func (k Keeper) escrowToken(ctx sdk.Context, sender, escrowAddress sdk.AccAddress, token sdk.Coin) error {
	if err := k.bankKeeper.SendCoins(ctx, sender, escrowAddress, sdk.NewCoins(token)); err != nil {
		// failure is expected for insufficient balances
		return err
	}

	// track the total amount in escrow keyed by denomination to allow for efficient iteration
	currentTotalEscrow := k.GetTotalEscrowForDenom(ctx, token.GetDenom())
	newTotalEscrow := currentTotalEscrow.Add(token)
	k.SetTotalEscrowForDenom(ctx, newTotalEscrow)

	return nil
}

// unescrowToken will send the given token from the escrow address to the provided receiver. It will also
// update the total escrow by deducting the unescrowed token from the current total escrow.
func (k Keeper) unescrowToken(ctx sdk.Context, escrowAddress, receiver sdk.AccAddress, token sdk.Coin) error {
	if err := k.bankKeeper.SendCoins(ctx, escrowAddress, receiver, sdk.NewCoins(token)); err != nil {
		// NOTE: this error is only expected to occur given an unexpected bug or a malicious
		// counterparty module. The bug may occur in bank or any part of the code that allows
		// the escrow address to be drained. A malicious counterparty module could drain the
		// escrow address by allowing more tokens to be sent back then were escrowed.
		return errorsmod.Wrap(err, "unable to unescrow tokens, this may be caused by a malicious counterparty module or a bug: please open an issue on counterparty module")
	}

	// track the total amount in escrow keyed by denomination to allow for efficient iteration
	currentTotalEscrow := k.GetTotalEscrowForDenom(ctx, token.GetDenom())
	newTotalEscrow := currentTotalEscrow.Sub(token)
	k.SetTotalEscrowForDenom(ctx, newTotalEscrow)

	return nil
}

// DenomPathFromHash returns the full denomination path prefix from an ibc denom with a hash
// component.
func (k Keeper) DenomPathFromHash(ctx sdk.Context, denom string) (string, error) {
	// trim the denomination prefix, by default "ibc/"
	hexHash := denom[len(types.DenomPrefix+"/"):]

	hash, err := types.ParseHexHash(hexHash)
	if err != nil {
		return "", errorsmod.Wrap(types.ErrInvalidDenomForTransfer, err.Error())
	}

	denomTrace, found := k.GetDenomTrace(ctx, hash)
	if !found {
		return "", errorsmod.Wrap(types.ErrTraceNotFound, hexHash)
	}

	fullDenomPath := denomTrace.GetFullDenomPath()
	return fullDenomPath, nil
}

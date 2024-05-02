package types

import (
	"strings"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"

	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
)

// NewPacketFee creates and returns a new PacketFee struct including the incentivization fees, refund address and relayers
func NewPacketFee(fee Fee, refundAddr string, relayers []string) PacketFee {
	return PacketFee{
		Fee:           fee,
		RefundAddress: refundAddr,
		Relayers:      relayers,
	}
}

// Validate performs basic stateless validation of the associated PacketFee
func (p PacketFee) Validate() error {
	_, err := sdk.AccAddressFromBech32(p.RefundAddress)
	if err != nil {
		return errorsmod.Wrap(err, "failed to convert RefundAddress into sdk.AccAddress")
	}

	// enforce relayers are not set
	if len(p.Relayers) != 0 {
		return ErrRelayersNotEmpty
	}

	return p.Fee.Validate()
}

// NewPacketFees creates and returns a new PacketFees struct including a list of type PacketFee
func NewPacketFees(packetFees []PacketFee) PacketFees {
	return PacketFees{
		PacketFees: packetFees,
	}
}

// NewIdentifiedPacketFees creates and returns a new IdentifiedPacketFees struct containing a packet ID and packet fees
func NewIdentifiedPacketFees(packetID channeltypes.PacketId, packetFees []PacketFee) IdentifiedPacketFees {
	return IdentifiedPacketFees{
		PacketId:   packetID,
		PacketFees: packetFees,
	}
}

// NewFee creates and returns a new Fee struct encapsulating the receive, acknowledgement and timeout fees as sdk.Coins
func NewFee(recvFee, ackFee, timeoutFee sdk.Coins) Fee {
	return Fee{
		RecvFee:    recvFee,
		AckFee:     ackFee,
		TimeoutFee: timeoutFee,
	}
}

// Total returns the total amount for a given Fee.
// The total amount is the Max(RecvFee + AckFee, TimeoutFee),
// This is because either the packet is received and acknowledged or it timeouts
func (f Fee) Total() sdk.Coins {
	// maximum returns the denomwise maximum of two sets of coins
	return f.RecvFee.Add(f.AckFee...).Max(f.TimeoutFee)
}

// Validate asserts that each Fee is valid and all three Fees are not empty or zero
func (f Fee) Validate() error {
	var errFees []string
	if !f.AckFee.IsValid() {
		errFees = append(errFees, "ack fee invalid")
	}
	if !f.RecvFee.IsValid() {
		errFees = append(errFees, "recv fee invalid")
	}
	if !f.TimeoutFee.IsValid() {
		errFees = append(errFees, "timeout fee invalid")
	}

	if len(errFees) > 0 {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidCoins, "contains invalid fees: %s", strings.Join(errFees, " , "))
	}

	// if all three fee's are zero or empty return an error
	if f.AckFee.IsZero() && f.RecvFee.IsZero() && f.TimeoutFee.IsZero() {
		return errorsmod.Wrap(ibcerrors.ErrInvalidCoins, "all fees are zero")
	}

	return nil
}

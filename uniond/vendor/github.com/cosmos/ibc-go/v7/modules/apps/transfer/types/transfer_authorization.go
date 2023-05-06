package types

import (
	errorsmod "cosmossdk.io/errors"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/authz"

	ibcerrors "github.com/cosmos/ibc-go/v7/internal/errors"
	channeltypes "github.com/cosmos/ibc-go/v7/modules/core/04-channel/types"
	host "github.com/cosmos/ibc-go/v7/modules/core/24-host"
)

var _ authz.Authorization = (*TransferAuthorization)(nil)

// NewTransferAuthorization creates a new TransferAuthorization object.
func NewTransferAuthorization(allocations ...Allocation) *TransferAuthorization {
	return &TransferAuthorization{
		Allocations: allocations,
	}
}

// MsgTypeURL implements Authorization.MsgTypeURL.
func (a TransferAuthorization) MsgTypeURL() string {
	return sdk.MsgTypeURL(&MsgTransfer{})
}

// Accept implements Authorization.Accept.
func (a TransferAuthorization) Accept(ctx sdk.Context, msg sdk.Msg) (authz.AcceptResponse, error) {
	msgTransfer, ok := msg.(*MsgTransfer)
	if !ok {
		return authz.AcceptResponse{}, errorsmod.Wrap(ibcerrors.ErrInvalidType, "type mismatch")
	}

	for index, allocation := range a.Allocations {
		if !(allocation.SourceChannel == msgTransfer.SourceChannel && allocation.SourcePort == msgTransfer.SourcePort) {
			continue
		}

		limitLeft, isNegative := allocation.SpendLimit.SafeSub(msgTransfer.Token)
		if isNegative {
			return authz.AcceptResponse{}, errorsmod.Wrapf(ibcerrors.ErrInsufficientFunds, "requested amount is more than spend limit")
		}

		if !isAllowedAddress(ctx, msgTransfer.Receiver, allocation.AllowList) {
			return authz.AcceptResponse{}, errorsmod.Wrap(ibcerrors.ErrInvalidAddress, "not allowed address for transfer")
		}

		if limitLeft.IsZero() {
			a.Allocations = append(a.Allocations[:index], a.Allocations[index+1:]...)
			if len(a.Allocations) == 0 {
				return authz.AcceptResponse{Accept: true, Delete: true}, nil
			}
			return authz.AcceptResponse{Accept: true, Delete: false, Updated: &TransferAuthorization{
				Allocations: a.Allocations,
			}}, nil
		}
		a.Allocations[index] = Allocation{
			SourcePort:    allocation.SourcePort,
			SourceChannel: allocation.SourceChannel,
			SpendLimit:    limitLeft,
			AllowList:     allocation.AllowList,
		}

		return authz.AcceptResponse{Accept: true, Delete: false, Updated: &TransferAuthorization{
			Allocations: a.Allocations,
		}}, nil
	}
	return authz.AcceptResponse{}, errorsmod.Wrapf(ibcerrors.ErrNotFound, "requested port and channel allocation does not exist")
}

// ValidateBasic implements Authorization.ValidateBasic.
func (a TransferAuthorization) ValidateBasic() error {
	if len(a.Allocations) == 0 {
		return errorsmod.Wrap(ErrInvalidAuthorization, "allocations cannot be empty")
	}

	foundChannels := make(map[string]bool, 0)

	for _, allocation := range a.Allocations {
		if _, found := foundChannels[allocation.SourceChannel]; found {
			return errorsmod.Wrapf(channeltypes.ErrInvalidChannel, "duplicate source channel ID: %s", allocation.SourceChannel)
		}

		foundChannels[allocation.SourceChannel] = true

		if allocation.SpendLimit == nil {
			return errorsmod.Wrap(ibcerrors.ErrInvalidCoins, "spend limit cannot be nil")
		}

		if err := allocation.SpendLimit.Validate(); err != nil {
			return errorsmod.Wrapf(ibcerrors.ErrInvalidCoins, err.Error())
		}

		if err := host.PortIdentifierValidator(allocation.SourcePort); err != nil {
			return errorsmod.Wrap(err, "invalid source port ID")
		}

		if err := host.ChannelIdentifierValidator(allocation.SourceChannel); err != nil {
			return errorsmod.Wrap(err, "invalid source channel ID")
		}

		found := make(map[string]bool, 0)
		for i := 0; i < len(allocation.AllowList); i++ {
			if found[allocation.AllowList[i]] {
				return errorsmod.Wrapf(ErrInvalidAuthorization, "duplicate entry in allow list %s", allocation.AllowList[i])
			}
			found[allocation.AllowList[i]] = true
		}
	}

	return nil
}

// isAllowedAddress returns a boolean indicating if the receiver address is valid for transfer.
// gasCostPerIteration gas is consumed for each iteration.
func isAllowedAddress(ctx sdk.Context, receiver string, allowedAddrs []string) bool {
	if len(allowedAddrs) == 0 {
		return true
	}

	gasCostPerIteration := ctx.KVGasConfig().IterNextCostFlat

	for _, addr := range allowedAddrs {
		ctx.GasMeter().ConsumeGas(gasCostPerIteration, "transfer authorization")
		if addr == receiver {
			return true
		}
	}
	return false
}

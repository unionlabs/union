package types

import (
	"strings"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"

	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
)

// NewGenesisState creates a 29-fee GenesisState instance.
func NewGenesisState(
	identifiedFees []IdentifiedPacketFees,
	feeEnabledChannels []FeeEnabledChannel,
	registeredPayees []RegisteredPayee,
	registeredCounterpartyPayees []RegisteredCounterpartyPayee,
	forwardRelayers []ForwardRelayerAddress,
) *GenesisState {
	return &GenesisState{
		IdentifiedFees:               identifiedFees,
		FeeEnabledChannels:           feeEnabledChannels,
		RegisteredPayees:             registeredPayees,
		RegisteredCounterpartyPayees: registeredCounterpartyPayees,
		ForwardRelayers:              forwardRelayers,
	}
}

// DefaultGenesisState returns a default instance of the 29-fee GenesisState.
func DefaultGenesisState() *GenesisState {
	return &GenesisState{
		IdentifiedFees:               []IdentifiedPacketFees{},
		ForwardRelayers:              []ForwardRelayerAddress{},
		FeeEnabledChannels:           []FeeEnabledChannel{},
		RegisteredPayees:             []RegisteredPayee{},
		RegisteredCounterpartyPayees: []RegisteredCounterpartyPayee{},
	}
}

// Validate performs basic genesis state validation returning an error upon any
// failure.
func (gs GenesisState) Validate() error {
	// Validate IdentifiedPacketFees
	for _, identifiedFees := range gs.IdentifiedFees {
		if err := identifiedFees.PacketId.Validate(); err != nil {
			return err
		}

		for _, packetFee := range identifiedFees.PacketFees {
			if err := packetFee.Validate(); err != nil {
				return err
			}
		}
	}

	// Validate FeeEnabledChannels
	for _, feeCh := range gs.FeeEnabledChannels {
		if err := host.PortIdentifierValidator(feeCh.PortId); err != nil {
			return errorsmod.Wrap(err, "invalid source port ID")
		}
		if err := host.ChannelIdentifierValidator(feeCh.ChannelId); err != nil {
			return errorsmod.Wrap(err, "invalid source channel ID")
		}
	}

	// Validate RegisteredPayees
	for _, registeredPayee := range gs.RegisteredPayees {
		if registeredPayee.Relayer == registeredPayee.Payee {
			return errorsmod.Wrap(ibcerrors.ErrInvalidAddress, "relayer address and payee address must not be equal")
		}

		if _, err := sdk.AccAddressFromBech32(registeredPayee.Relayer); err != nil {
			return errorsmod.Wrap(err, "failed to convert relayer address into sdk.AccAddress")
		}

		if _, err := sdk.AccAddressFromBech32(registeredPayee.Payee); err != nil {
			return errorsmod.Wrap(err, "failed to convert payee address into sdk.AccAddress")
		}

		if err := host.ChannelIdentifierValidator(registeredPayee.ChannelId); err != nil {
			return errorsmod.Wrapf(err, "invalid channel identifier: %s", registeredPayee.ChannelId)
		}
	}

	// Validate RegisteredCounterpartyPayees
	for _, registeredCounterpartyPayee := range gs.RegisteredCounterpartyPayees {
		if _, err := sdk.AccAddressFromBech32(registeredCounterpartyPayee.Relayer); err != nil {
			return errorsmod.Wrap(err, "failed to convert relayer address into sdk.AccAddress")
		}

		if strings.TrimSpace(registeredCounterpartyPayee.CounterpartyPayee) == "" {
			return ErrCounterpartyPayeeEmpty
		}

		if err := host.ChannelIdentifierValidator(registeredCounterpartyPayee.ChannelId); err != nil {
			return errorsmod.Wrapf(err, "invalid channel identifier: %s", registeredCounterpartyPayee.ChannelId)
		}
	}

	// Validate ForwardRelayers
	for _, rel := range gs.ForwardRelayers {
		if _, err := sdk.AccAddressFromBech32(rel.Address); err != nil {
			return errorsmod.Wrap(err, "failed to convert forward relayer address into sdk.AccAddress")
		}

		if err := rel.PacketId.Validate(); err != nil {
			return err
		}
	}

	return nil
}

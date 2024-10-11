package types

import (
	errorsmod "cosmossdk.io/errors"

	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

var (
	_ exported.ChannelI             = (*Channel)(nil)
	_ exported.CounterpartyChannelI = (*Counterparty)(nil)
)

// NewChannel creates a new Channel instance
func NewChannel(
	state State, ordering Order, counterparty Counterparty,
	hops []string, version string,
) Channel {
	return Channel{
		State:          state,
		Ordering:       ordering,
		Counterparty:   counterparty,
		ConnectionHops: hops,
		Version:        version,
		// UpgradeSequence is intentionally left empty as a new channel has not performed an upgrade.
		UpgradeSequence: 0,
	}
}

// GetState implements Channel interface.
func (ch Channel) GetState() int32 {
	return int32(ch.State)
}

// GetOrdering implements Channel interface.
func (ch Channel) GetOrdering() int32 {
	return int32(ch.Ordering)
}

// GetCounterparty implements Channel interface.
func (ch Channel) GetCounterparty() exported.CounterpartyChannelI {
	return ch.Counterparty
}

// GetConnectionHops implements Channel interface.
func (ch Channel) GetConnectionHops() []string {
	return ch.ConnectionHops
}

// GetVersion implements Channel interface.
func (ch Channel) GetVersion() string {
	return ch.Version
}

// IsOpen returns true if the channel state is OPEN
func (ch Channel) IsOpen() bool {
	return ch.State == OPEN
}

// IsClosed returns true if the channel state is CLOSED
func (ch Channel) IsClosed() bool {
	return ch.State == CLOSED
}

// ValidateBasic performs a basic validation of the channel fields
func (ch Channel) ValidateBasic() error {
	if ch.State == UNINITIALIZED {
		return ErrInvalidChannelState
	}
	if !(ch.Ordering == ORDERED || ch.Ordering == UNORDERED) {
		return errorsmod.Wrap(ErrInvalidChannelOrdering, ch.Ordering.String())
	}
	if len(ch.ConnectionHops) != 1 {
		return errorsmod.Wrap(
			ErrTooManyConnectionHops,
			"current IBC version only supports one connection hop",
		)
	}
	if err := host.ConnectionIdentifierValidator(ch.ConnectionHops[0]); err != nil {
		return errorsmod.Wrap(err, "invalid connection hop ID")
	}
	return ch.Counterparty.ValidateBasic()
}

// NewCounterparty returns a new Counterparty instance
func NewCounterparty(portID, channelID string) Counterparty {
	return Counterparty{
		PortId:    portID,
		ChannelId: channelID,
	}
}

// GetPortID implements CounterpartyChannelI interface
func (c Counterparty) GetPortID() string {
	return c.PortId
}

// GetChannelID implements CounterpartyChannelI interface
func (c Counterparty) GetChannelID() string {
	return c.ChannelId
}

// ValidateBasic performs a basic validation check of the identifiers
func (c Counterparty) ValidateBasic() error {
	if err := host.PortIdentifierValidator(c.PortId); err != nil {
		return errorsmod.Wrap(err, "invalid counterparty port ID")
	}
	if c.ChannelId != "" {
		if err := host.ChannelIdentifierValidator(c.ChannelId); err != nil {
			return errorsmod.Wrap(err, "invalid counterparty channel ID")
		}
	}
	return nil
}

// NewIdentifiedChannel creates a new IdentifiedChannel instance
func NewIdentifiedChannel(portID, channelID string, ch Channel) IdentifiedChannel {
	return IdentifiedChannel{
		State:           ch.State,
		Ordering:        ch.Ordering,
		Counterparty:    ch.Counterparty,
		ConnectionHops:  ch.ConnectionHops,
		Version:         ch.Version,
		UpgradeSequence: ch.UpgradeSequence,
		PortId:          portID,
		ChannelId:       channelID,
	}
}

// ValidateBasic performs a basic validation of the identifiers and channel fields.
func (ic IdentifiedChannel) ValidateBasic() error {
	if err := host.ChannelIdentifierValidator(ic.ChannelId); err != nil {
		return errorsmod.Wrap(err, "invalid channel ID")
	}
	if err := host.PortIdentifierValidator(ic.PortId); err != nil {
		return errorsmod.Wrap(err, "invalid port ID")
	}
	channel := NewChannel(ic.State, ic.Ordering, ic.Counterparty, ic.ConnectionHops, ic.Version)
	return channel.ValidateBasic()
}

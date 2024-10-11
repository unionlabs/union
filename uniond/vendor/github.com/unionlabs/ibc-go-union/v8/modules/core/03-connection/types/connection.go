package types

import (
	errorsmod "cosmossdk.io/errors"

	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

var _ exported.ConnectionI = (*ConnectionEnd)(nil)

// NewConnectionEnd creates a new ConnectionEnd instance.
func NewConnectionEnd(state State, clientID string, counterparty Counterparty, versions []*Version, delayPeriod uint64) ConnectionEnd {
	return ConnectionEnd{
		ClientId:     clientID,
		Versions:     versions,
		State:        state,
		Counterparty: counterparty,
		DelayPeriod:  delayPeriod,
	}
}

// GetState implements the Connection interface
func (c ConnectionEnd) GetState() int32 {
	return int32(c.State)
}

// GetClientID implements the Connection interface
func (c ConnectionEnd) GetClientID() string {
	return c.ClientId
}

// GetCounterparty implements the Connection interface
func (c ConnectionEnd) GetCounterparty() exported.CounterpartyConnectionI {
	return c.Counterparty
}

// GetVersions implements the Connection interface
func (c ConnectionEnd) GetVersions() []*Version {
	return c.Versions
}

// GetDelayPeriod implements the Connection interface
func (c ConnectionEnd) GetDelayPeriod() uint64 {
	return c.DelayPeriod
}

// ValidateBasic implements the Connection interface.
// NOTE: the protocol supports that the connection and client IDs match the
// counterparty's.
func (c ConnectionEnd) ValidateBasic() error {
	if err := host.ClientIdentifierValidator(c.ClientId); err != nil {
		return errorsmod.Wrap(err, "invalid client ID")
	}
	if len(c.Versions) == 0 {
		return errorsmod.Wrap(ibcerrors.ErrInvalidVersion, "empty connection versions")
	}
	for _, version := range c.Versions {
		if err := ValidateVersion(version); err != nil {
			return err
		}
	}
	return c.Counterparty.ValidateBasic()
}

var _ exported.CounterpartyConnectionI = (*Counterparty)(nil)

// NewCounterparty creates a new Counterparty instance.
func NewCounterparty(clientID, connectionID string, prefix commitmenttypes.MerklePrefix) Counterparty {
	return Counterparty{
		ClientId:     clientID,
		ConnectionId: connectionID,
		Prefix:       prefix,
	}
}

// GetClientID implements the CounterpartyConnectionI interface
func (c Counterparty) GetClientID() string {
	return c.ClientId
}

// GetConnectionID implements the CounterpartyConnectionI interface
func (c Counterparty) GetConnectionID() string {
	return c.ConnectionId
}

// GetPrefix implements the CounterpartyConnectionI interface
func (c Counterparty) GetPrefix() exported.Prefix {
	return &c.Prefix
}

// ValidateBasic performs a basic validation check of the identifiers and prefix
func (c Counterparty) ValidateBasic() error {
	if c.ConnectionId != "" {
		if err := host.ConnectionIdentifierValidator(c.ConnectionId); err != nil {
			return errorsmod.Wrap(err, "invalid counterparty connection ID")
		}
	}
	if err := host.ClientIdentifierValidator(c.ClientId); err != nil {
		return errorsmod.Wrap(err, "invalid counterparty client ID")
	}
	if c.Prefix.Empty() {
		return errorsmod.Wrap(ErrInvalidCounterparty, "counterparty prefix cannot be empty")
	}
	return nil
}

// NewIdentifiedConnection creates a new IdentifiedConnection instance
func NewIdentifiedConnection(connectionID string, conn ConnectionEnd) IdentifiedConnection {
	return IdentifiedConnection{
		Id:           connectionID,
		ClientId:     conn.ClientId,
		Versions:     conn.Versions,
		State:        conn.State,
		Counterparty: conn.Counterparty,
		DelayPeriod:  conn.DelayPeriod,
	}
}

// ValidateBasic performs a basic validation of the connection identifier and connection fields.
func (ic IdentifiedConnection) ValidateBasic() error {
	if err := host.ConnectionIdentifierValidator(ic.Id); err != nil {
		return errorsmod.Wrap(err, "invalid connection ID")
	}
	connection := NewConnectionEnd(ic.State, ic.ClientId, ic.Counterparty, ic.Versions, ic.DelayPeriod)
	return connection.ValidateBasic()
}

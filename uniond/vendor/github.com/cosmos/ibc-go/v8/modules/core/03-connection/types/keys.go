package types

import (
	"fmt"
	"regexp"

	errorsmod "cosmossdk.io/errors"

	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
)

const (
	// SubModuleName defines the IBC connection name
	SubModuleName = "connection"

	// StoreKey is the store key string for IBC connections
	StoreKey = SubModuleName

	// RouterKey is the message route for IBC connections
	RouterKey = SubModuleName

	// QuerierRoute is the querier route for IBC connections
	QuerierRoute = SubModuleName

	// KeyNextConnectionSequence is the key used to store the next connection sequence in
	// the keeper.
	KeyNextConnectionSequence = "nextConnectionSequence"

	// ConnectionPrefix is the prefix used when creating a connection identifier
	ConnectionPrefix = "connection-"

	// ParamsKey is the store key for the IBC connection parameters
	ParamsKey = "connectionParams"
)

// FormatConnectionIdentifier returns the connection identifier with the sequence appended.
// This is a SDK specific format not enforced by IBC protocol.
func FormatConnectionIdentifier(sequence uint64) string {
	return fmt.Sprintf("%s%d", ConnectionPrefix, sequence)
}

// IsConnectionIDFormat checks if a connectionID is in the format required on the SDK for
// parsing connection identifiers. The connection identifier must be in the form: `connection-{N}
var IsConnectionIDFormat = regexp.MustCompile(`^connection-[0-9]{1,20}$`).MatchString

// IsValidConnectionID checks if the connection identifier is valid and can be parsed to
// the connection identifier format.
func IsValidConnectionID(connectionID string) bool {
	_, err := ParseConnectionSequence(connectionID)
	return err == nil
}

// ParseConnectionSequence parses the connection sequence from the connection identifier.
func ParseConnectionSequence(connectionID string) (uint64, error) {
	if !IsConnectionIDFormat(connectionID) {
		return 0, errorsmod.Wrap(host.ErrInvalidID, "connection identifier is not in the format: `connection-{N}`")
	}

	sequence, err := host.ParseIdentifier(connectionID, ConnectionPrefix)
	if err != nil {
		return 0, errorsmod.Wrap(err, "invalid connection identifier")
	}

	return sequence, nil
}

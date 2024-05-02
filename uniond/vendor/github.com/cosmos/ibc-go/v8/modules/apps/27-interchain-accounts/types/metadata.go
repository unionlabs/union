package types

import (
	"slices"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"

	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
)

const (
	// EncodingProtobuf defines the protocol buffers proto3 encoding format
	EncodingProtobuf = "proto3"
	// EncodingProto3JSON defines the proto3 JSON encoding format
	EncodingProto3JSON = "proto3json"

	// TxTypeSDKMultiMsg defines the multi message transaction type supported by the Cosmos SDK
	TxTypeSDKMultiMsg = "sdk_multi_msg"
)

// NewMetadata creates and returns a new ICS27 Metadata instance
func NewMetadata(version, controllerConnectionID, hostConnectionID, accAddress, encoding, txType string) Metadata {
	return Metadata{
		Version:                version,
		ControllerConnectionId: controllerConnectionID,
		HostConnectionId:       hostConnectionID,
		Address:                accAddress,
		Encoding:               encoding,
		TxType:                 txType,
	}
}

// NewDefaultMetadata creates and returns a new ICS27 Metadata instance containing the default ICS27 Metadata values
// with the provided controller and host connection identifiers
func NewDefaultMetadata(controllerConnectionID, hostConnectionID string) Metadata {
	metadata := Metadata{
		ControllerConnectionId: controllerConnectionID,
		HostConnectionId:       hostConnectionID,
		Encoding:               EncodingProtobuf,
		TxType:                 TxTypeSDKMultiMsg,
		Version:                Version,
	}

	return metadata
}

// NewDefaultMetadataString creates and returns a new JSON encoded version string containing the default ICS27 Metadata values
// with the provided controller and host connection identifiers
func NewDefaultMetadataString(controllerConnectionID, hostConnectionID string) string {
	metadata := NewDefaultMetadata(controllerConnectionID, hostConnectionID)

	return string(ModuleCdc.MustMarshalJSON(&metadata))
}

// MetadataFromVersion parses Metadata from a json encoded version string.
func MetadataFromVersion(versionString string) (Metadata, error) {
	var metadata Metadata
	if err := ModuleCdc.UnmarshalJSON([]byte(versionString), &metadata); err != nil {
		return Metadata{}, errorsmod.Wrapf(ErrUnknownDataType, "cannot unmarshal ICS-27 interchain accounts metadata")
	}
	return metadata, nil
}

// IsPreviousMetadataEqual compares a metadata to a previous version string set in a channel struct.
// It ensures all fields are equal except the Address string
func IsPreviousMetadataEqual(previousVersion string, metadata Metadata) bool {
	previousMetadata, err := MetadataFromVersion(previousVersion)
	if err != nil {
		return false
	}

	return (previousMetadata.Version == metadata.Version &&
		previousMetadata.ControllerConnectionId == metadata.ControllerConnectionId &&
		previousMetadata.HostConnectionId == metadata.HostConnectionId &&
		previousMetadata.Encoding == metadata.Encoding &&
		previousMetadata.TxType == metadata.TxType)
}

// ValidateControllerMetadata performs validation of the provided ICS27 controller metadata parameters as well
// as the connection params against the provided metadata
func ValidateControllerMetadata(ctx sdk.Context, channelKeeper ChannelKeeper, connectionHops []string, metadata Metadata) error {
	if !isSupportedEncoding(metadata.Encoding) {
		return errorsmod.Wrapf(ErrInvalidCodec, "unsupported encoding format %s", metadata.Encoding)
	}

	if !isSupportedTxType(metadata.TxType) {
		return errorsmod.Wrapf(ErrUnknownDataType, "unsupported transaction type %s", metadata.TxType)
	}

	connection, err := channelKeeper.GetConnection(ctx, connectionHops[0])
	if err != nil {
		return err
	}

	if err := validateConnectionParams(metadata, connectionHops[0], connection.GetCounterparty().GetConnectionID()); err != nil {
		return err
	}

	if metadata.Address != "" {
		if err := ValidateAccountAddress(metadata.Address); err != nil {
			return err
		}
	}

	if metadata.Version != Version {
		return errorsmod.Wrapf(ErrInvalidVersion, "expected %s, got %s", Version, metadata.Version)
	}

	return nil
}

// ValidateHostMetadata performs validation of the provided ICS27 host metadata parameters
func ValidateHostMetadata(ctx sdk.Context, channelKeeper ChannelKeeper, connectionHops []string, metadata Metadata) error {
	if !isSupportedEncoding(metadata.Encoding) {
		return errorsmod.Wrapf(ErrInvalidCodec, "unsupported encoding format %s", metadata.Encoding)
	}

	if !isSupportedTxType(metadata.TxType) {
		return errorsmod.Wrapf(ErrUnknownDataType, "unsupported transaction type %s", metadata.TxType)
	}

	connection, err := channelKeeper.GetConnection(ctx, connectionHops[0])
	if err != nil {
		return err
	}

	if err := validateConnectionParams(metadata, connection.GetCounterparty().GetConnectionID(), connectionHops[0]); err != nil {
		return err
	}

	if metadata.Address != "" {
		if err := ValidateAccountAddress(metadata.Address); err != nil {
			return err
		}
	}

	if metadata.Version != Version {
		return errorsmod.Wrapf(ErrInvalidVersion, "expected %s, got %s", Version, metadata.Version)
	}

	return nil
}

// isSupportedEncoding returns true if the provided encoding is supported, otherwise false
func isSupportedEncoding(encoding string) bool {
	return slices.Contains(getSupportedEncoding(), encoding)
}

// getSupportedEncoding returns a string slice of supported encoding formats
func getSupportedEncoding() []string {
	return []string{EncodingProtobuf, EncodingProto3JSON}
}

// isSupportedTxType returns true if the provided transaction type is supported, otherwise false
func isSupportedTxType(txType string) bool {
	return slices.Contains(getSupportedTxTypes(), txType)
}

// getSupportedTxTypes returns a string slice of supported transaction types
func getSupportedTxTypes() []string {
	return []string{TxTypeSDKMultiMsg}
}

// validateConnectionParams compares the given the controller and host connection IDs to those set in the provided ICS27 Metadata
func validateConnectionParams(metadata Metadata, controllerConnectionID, hostConnectionID string) error {
	if metadata.ControllerConnectionId != controllerConnectionID {
		return errorsmod.Wrapf(connectiontypes.ErrInvalidConnection, "expected %s, got %s", controllerConnectionID, metadata.ControllerConnectionId)
	}

	if metadata.HostConnectionId != hostConnectionID {
		return errorsmod.Wrapf(connectiontypes.ErrInvalidConnection, "expected %s, got %s", hostConnectionID, metadata.HostConnectionId)
	}

	return nil
}

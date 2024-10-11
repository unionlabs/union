package types

import errorsmod "cosmossdk.io/errors"

// MetadataFromVersion attempts to parse the given string into a fee version Metadata,
// an error is returned if it fails to do so.
func MetadataFromVersion(version string) (Metadata, error) {
	var metadata Metadata
	err := ModuleCdc.UnmarshalJSON([]byte(version), &metadata)
	if err != nil {
		return Metadata{}, errorsmod.Wrapf(ErrInvalidVersion, "failed to unmarshal metadata from version: %s", version)
	}

	return metadata, nil
}

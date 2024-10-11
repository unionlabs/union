package types

import (
	"slices"
	"strings"

	errorsmod "cosmossdk.io/errors"
)

var (
	// DefaultIBCVersion represents the latest supported version of IBC used
	// in connection version negotiation. The current version supports the list
	// of orderings defined in SupportedOrderings and requires at least one channel type
	// to be agreed upon.
	DefaultIBCVersion = NewVersion(DefaultIBCVersionIdentifier, SupportedOrderings)

	// DefaultIBCVersionIdentifier is the IBC v1.0.0 protocol version identifier
	DefaultIBCVersionIdentifier = "1"

	// SupportedOrderings is the list of orderings supported by IBC. The current
	// version supports only ORDERED and UNORDERED channels.
	SupportedOrderings = []string{"ORDER_ORDERED", "ORDER_UNORDERED"}

	// AllowNilFeatureSet is a helper map to indicate if a specified version
	// identifier is allowed to have a nil feature set. Any versions supported,
	// but not included in the map default to not supporting nil feature sets.
	allowNilFeatureSet = map[string]bool{
		DefaultIBCVersionIdentifier: false,
	}
)

// NewVersion returns a new instance of Version.
func NewVersion(identifier string, features []string) *Version {
	return &Version{
		Identifier: identifier,
		Features:   features,
	}
}

// GetIdentifier implements the VersionI interface
func (version Version) GetIdentifier() string {
	return version.Identifier
}

// GetFeatures implements the VersionI interface
func (version Version) GetFeatures() []string {
	return version.Features
}

// ValidateVersion does basic validation of the version identifier and
// features. It unmarshals the version string into a Version object.
func ValidateVersion(version *Version) error {
	if version == nil {
		return errorsmod.Wrap(ErrInvalidVersion, "version cannot be nil")
	}
	if strings.TrimSpace(version.Identifier) == "" {
		return errorsmod.Wrap(ErrInvalidVersion, "version identifier cannot be blank")
	}
	for i, feature := range version.Features {
		if strings.TrimSpace(feature) == "" {
			return errorsmod.Wrapf(ErrInvalidVersion, "feature cannot be blank, index %d", i)
		}
	}

	return nil
}

// VerifyProposedVersion verifies that the entire feature set in the
// proposed version is supported by this chain. If the feature set is
// empty it verifies that this is allowed for the specified version
// identifier.
func (version Version) VerifyProposedVersion(proposedVersion *Version) error {
	if proposedVersion.GetIdentifier() != version.GetIdentifier() {
		return errorsmod.Wrapf(
			ErrVersionNegotiationFailed,
			"proposed version identifier does not equal supported version identifier (%s != %s)", proposedVersion.GetIdentifier(), version.GetIdentifier(),
		)
	}

	if len(proposedVersion.GetFeatures()) == 0 && !allowNilFeatureSet[proposedVersion.GetIdentifier()] {
		return errorsmod.Wrapf(
			ErrVersionNegotiationFailed,
			"nil feature sets are not supported for version identifier (%s)", proposedVersion.GetIdentifier(),
		)
	}

	for _, proposedFeature := range proposedVersion.GetFeatures() {
		if !slices.Contains(version.GetFeatures(), proposedFeature) {
			return errorsmod.Wrapf(
				ErrVersionNegotiationFailed,
				"proposed feature (%s) is not a supported feature set (%s)", proposedFeature, version.GetFeatures(),
			)
		}
	}

	return nil
}

// VerifySupportedFeature takes in a version and feature string and returns
// true if the feature is supported by the version and false otherwise.
func VerifySupportedFeature(version *Version, feature string) bool {
	return slices.Contains(version.GetFeatures(), feature)
}

// GetCompatibleVersions returns a descending ordered set of compatible IBC
// versions for the caller chain's connection end. The latest supported
// version should be first element and the set should descend to the oldest
// supported version.
func GetCompatibleVersions() []*Version {
	return []*Version{DefaultIBCVersion}
}

// IsSupportedVersion returns true if the proposed version has a matching version
// identifier and its entire feature set is supported or the version identifier
// supports an empty feature set.
func IsSupportedVersion(supportedVersions []*Version, proposedVersion *Version) bool {
	supportedVersion, found := FindSupportedVersion(proposedVersion, supportedVersions)
	if !found {
		return false
	}

	if err := supportedVersion.VerifyProposedVersion(proposedVersion); err != nil {
		return false
	}

	return true
}

// FindSupportedVersion returns the version with a matching version identifier
// if it exists. The returned boolean is true if the version is found and
// false otherwise.
func FindSupportedVersion(version *Version, supportedVersions []*Version) (*Version, bool) {
	for _, supportedVersion := range supportedVersions {
		if version.GetIdentifier() == supportedVersion.GetIdentifier() {
			return supportedVersion, true
		}
	}

	return nil, false
}

// PickVersion iterates over the descending ordered set of compatible IBC
// versions and selects the first version with a version identifier that is
// supported by the counterparty. The returned version contains a feature
// set with the intersection of the features supported by the source and
// counterparty chains. If the feature set intersection is nil and this is
// not allowed for the chosen version identifier then the search for a
// compatible version continues. This function is called in the ConnOpenTry
// handshake procedure.
//
// CONTRACT: PickVersion must only provide a version that is in the
// intersection of the supported versions and the counterparty versions.
func PickVersion(supportedVersions, counterpartyVersions []*Version) (*Version, error) {
	for _, supportedVersion := range supportedVersions {
		// check if the source version is supported by the counterparty
		if counterpartyVersion, found := FindSupportedVersion(supportedVersion, counterpartyVersions); found {
			featureSet := GetFeatureSetIntersection(supportedVersion.GetFeatures(), counterpartyVersion.GetFeatures())
			if len(featureSet) == 0 && !allowNilFeatureSet[supportedVersion.GetIdentifier()] {
				continue
			}

			return NewVersion(supportedVersion.GetIdentifier(), featureSet), nil
		}
	}

	return nil, errorsmod.Wrapf(
		ErrVersionNegotiationFailed,
		"failed to find a matching counterparty version (%v) from the supported version list (%v)", counterpartyVersions, supportedVersions,
	)
}

// GetFeatureSetIntersection returns the intersections of source feature set
// and the counterparty feature set. This is done by iterating over all the
// features in the source version and seeing if they exist in the feature
// set for the counterparty version.
func GetFeatureSetIntersection(sourceFeatureSet, counterpartyFeatureSet []string) (featureSet []string) {
	for _, feature := range sourceFeatureSet {
		if slices.Contains(counterpartyFeatureSet, feature) {
			featureSet = append(featureSet, feature)
		}
	}

	return featureSet
}

package state_native

import (
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
)

func (b *BeaconState) ProportionalSlashingMultiplier() (uint64, error) {
	switch b.version {
	case version.Bellatrix, version.Capella, version.Deneb:
		return params.BeaconConfig().ProportionalSlashingMultiplierBellatrix, nil
	case version.Altair:
		return params.BeaconConfig().ProportionalSlashingMultiplierAltair, nil
	case version.Phase0:
		return params.BeaconConfig().ProportionalSlashingMultiplier, nil
	}
	return 0, errNotSupported("ProportionalSlashingMultiplier()", b.version)
}

func (b *BeaconState) InactivityPenaltyQuotient() (uint64, error) {
	switch b.version {
	case version.Bellatrix, version.Capella, version.Deneb:
		return params.BeaconConfig().InactivityPenaltyQuotientBellatrix, nil
	case version.Altair:
		return params.BeaconConfig().InactivityPenaltyQuotientAltair, nil
	case version.Phase0:
		return params.BeaconConfig().InactivityPenaltyQuotient, nil
	}
	return 0, errNotSupported("InactivityPenaltyQuotient()", b.version)
}

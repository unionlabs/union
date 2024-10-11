package helpers

import (
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/validator"
)

// ValidatorStatus returns a validator's status at the given epoch.
func ValidatorStatus(val state.ReadOnlyValidator, epoch primitives.Epoch) (validator.Status, error) {
	valStatus, err := ValidatorSubStatus(val, epoch)
	if err != nil {
		return 0, errors.Wrap(err, "could not get validator sub status")
	}
	switch valStatus {
	case validator.PendingInitialized, validator.PendingQueued:
		return validator.Pending, nil
	case validator.ActiveOngoing, validator.ActiveSlashed, validator.ActiveExiting:
		return validator.Active, nil
	case validator.ExitedUnslashed, validator.ExitedSlashed:
		return validator.Exited, nil
	case validator.WithdrawalPossible, validator.WithdrawalDone:
		return validator.Withdrawal, nil
	}
	return 0, errors.New("invalid validator state")
}

// ValidatorSubStatus returns a validator's sub-status at the given epoch.
func ValidatorSubStatus(val state.ReadOnlyValidator, epoch primitives.Epoch) (validator.Status, error) {
	farFutureEpoch := params.BeaconConfig().FarFutureEpoch

	// Pending.
	if val.ActivationEpoch() > epoch {
		if val.ActivationEligibilityEpoch() == farFutureEpoch {
			return validator.PendingInitialized, nil
		} else if val.ActivationEligibilityEpoch() < farFutureEpoch {
			return validator.PendingQueued, nil
		}
	}

	// Active.
	if val.ActivationEpoch() <= epoch && epoch < val.ExitEpoch() {
		if val.ExitEpoch() == farFutureEpoch {
			return validator.ActiveOngoing, nil
		} else if val.ExitEpoch() < farFutureEpoch {
			if val.Slashed() {
				return validator.ActiveSlashed, nil
			}
			return validator.ActiveExiting, nil
		}
	}

	// Exited.
	if val.ExitEpoch() <= epoch && epoch < val.WithdrawableEpoch() {
		if val.Slashed() {
			return validator.ExitedSlashed, nil
		}
		return validator.ExitedUnslashed, nil
	}

	if val.WithdrawableEpoch() <= epoch {
		if val.EffectiveBalance() != 0 {
			return validator.WithdrawalPossible, nil
		} else {
			return validator.WithdrawalDone, nil
		}
	}

	return 0, errors.New("invalid validator state")
}

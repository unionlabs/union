// Package validators contains libraries to shuffle validators
// and retrieve active validator indices from a given slot
// or an attestation. It also provides helper functions to locate
// validator based on pubic key.
package validators

import (
	"context"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

// ErrValidatorAlreadyExited is an error raised when trying to process an exit of
// an already exited validator
var ErrValidatorAlreadyExited = errors.New("validator already exited")

// MaxExitEpochAndChurn returns the maximum non-FAR_FUTURE_EPOCH exit
// epoch and the number of them
func MaxExitEpochAndChurn(s state.BeaconState) (maxExitEpoch primitives.Epoch, churn uint64) {
	farFutureEpoch := params.BeaconConfig().FarFutureEpoch
	err := s.ReadFromEveryValidator(func(idx int, val state.ReadOnlyValidator) error {
		e := val.ExitEpoch()
		if e != farFutureEpoch {
			if e > maxExitEpoch {
				maxExitEpoch = e
				churn = 1
			} else if e == maxExitEpoch {
				churn++
			}
		}
		return nil
	})
	_ = err
	return
}

// InitiateValidatorExit takes in validator index and updates
// validator with correct voluntary exit parameters.
//
// Spec pseudocode definition:
//
//	def initiate_validator_exit(state: BeaconState, index: ValidatorIndex) -> None:
//	  """
//	  Initiate the exit of the validator with index ``index``.
//	  """
//	  # Return if validator already initiated exit
//	  validator = state.validators[index]
//	  if validator.exit_epoch != FAR_FUTURE_EPOCH:
//	      return
//
//	  # Compute exit queue epoch
//	  exit_epochs = [v.exit_epoch for v in state.validators if v.exit_epoch != FAR_FUTURE_EPOCH]
//	  exit_queue_epoch = max(exit_epochs + [compute_activation_exit_epoch(get_current_epoch(state))])
//	  exit_queue_churn = len([v for v in state.validators if v.exit_epoch == exit_queue_epoch])
//	  if exit_queue_churn >= get_validator_churn_limit(state):
//	      exit_queue_epoch += Epoch(1)
//
//	  # Set validator exit epoch and withdrawable epoch
//	  validator.exit_epoch = exit_queue_epoch
//	  validator.withdrawable_epoch = Epoch(validator.exit_epoch + MIN_VALIDATOR_WITHDRAWABILITY_DELAY)
func InitiateValidatorExit(ctx context.Context, s state.BeaconState, idx primitives.ValidatorIndex, exitQueueEpoch primitives.Epoch, churn uint64) (state.BeaconState, primitives.Epoch, error) {
	exitableEpoch := helpers.ActivationExitEpoch(time.CurrentEpoch(s))
	if exitableEpoch > exitQueueEpoch {
		exitQueueEpoch = exitableEpoch
		churn = 0
	}
	validator, err := s.ValidatorAtIndex(idx)
	if err != nil {
		return nil, 0, err
	}
	if validator.ExitEpoch != params.BeaconConfig().FarFutureEpoch {
		return s, validator.ExitEpoch, ErrValidatorAlreadyExited
	}
	activeValidatorCount, err := helpers.ActiveValidatorCount(ctx, s, time.CurrentEpoch(s))
	if err != nil {
		return nil, 0, errors.Wrap(err, "could not get active validator count")
	}
	currentChurn := helpers.ValidatorExitChurnLimit(activeValidatorCount)

	if churn >= currentChurn {
		exitQueueEpoch, err = exitQueueEpoch.SafeAdd(1)
		if err != nil {
			return nil, 0, err
		}
	}
	validator.ExitEpoch = exitQueueEpoch
	validator.WithdrawableEpoch, err = exitQueueEpoch.SafeAddEpoch(params.BeaconConfig().MinValidatorWithdrawabilityDelay)
	if err != nil {
		return nil, 0, err
	}
	if err := s.UpdateValidatorAtIndex(idx, validator); err != nil {
		return nil, 0, err
	}
	return s, exitQueueEpoch, nil
}

// SlashValidator slashes the malicious validator's balance and awards
// the whistleblower's balance.
//
// Spec pseudocode definition:
//
//	def slash_validator(state: BeaconState,
//	                  slashed_index: ValidatorIndex,
//	                  whistleblower_index: ValidatorIndex=None) -> None:
//	  """
//	  Slash the validator with index ``slashed_index``.
//	  """
//	  epoch = get_current_epoch(state)
//	  initiate_validator_exit(state, slashed_index)
//	  validator = state.validators[slashed_index]
//	  validator.slashed = True
//	  validator.withdrawable_epoch = max(validator.withdrawable_epoch, Epoch(epoch + EPOCHS_PER_SLASHINGS_VECTOR))
//	  state.slashings[epoch % EPOCHS_PER_SLASHINGS_VECTOR] += validator.effective_balance
//	  decrease_balance(state, slashed_index, validator.effective_balance // MIN_SLASHING_PENALTY_QUOTIENT)
//
//	  # Apply proposer and whistleblower rewards
//	  proposer_index = get_beacon_proposer_index(state)
//	  if whistleblower_index is None:
//	      whistleblower_index = proposer_index
//	  whistleblower_reward = Gwei(validator.effective_balance // WHISTLEBLOWER_REWARD_QUOTIENT)
//	  proposer_reward = Gwei(whistleblower_reward // PROPOSER_REWARD_QUOTIENT)
//	  increase_balance(state, proposer_index, proposer_reward)
//	  increase_balance(state, whistleblower_index, Gwei(whistleblower_reward - proposer_reward))
func SlashValidator(
	ctx context.Context,
	s state.BeaconState,
	slashedIdx primitives.ValidatorIndex,
	penaltyQuotient uint64,
	proposerRewardQuotient uint64) (state.BeaconState, error) {
	maxExitEpoch, churn := MaxExitEpochAndChurn(s)
	s, _, err := InitiateValidatorExit(ctx, s, slashedIdx, maxExitEpoch, churn)
	if err != nil && !errors.Is(err, ErrValidatorAlreadyExited) {
		return nil, errors.Wrapf(err, "could not initiate validator %d exit", slashedIdx)
	}
	currentEpoch := slots.ToEpoch(s.Slot())
	validator, err := s.ValidatorAtIndex(slashedIdx)
	if err != nil {
		return nil, err
	}
	validator.Slashed = true
	maxWithdrawableEpoch := primitives.MaxEpoch(validator.WithdrawableEpoch, currentEpoch+params.BeaconConfig().EpochsPerSlashingsVector)
	validator.WithdrawableEpoch = maxWithdrawableEpoch

	if err := s.UpdateValidatorAtIndex(slashedIdx, validator); err != nil {
		return nil, err
	}

	// The slashing amount is represented by epochs per slashing vector. The validator's effective balance is then applied to that amount.
	slashings := s.Slashings()
	currentSlashing := slashings[currentEpoch%params.BeaconConfig().EpochsPerSlashingsVector]
	if err := s.UpdateSlashingsAtIndex(
		uint64(currentEpoch%params.BeaconConfig().EpochsPerSlashingsVector),
		currentSlashing+validator.EffectiveBalance,
	); err != nil {
		return nil, err
	}
	if err := helpers.DecreaseBalance(s, slashedIdx, validator.EffectiveBalance/penaltyQuotient); err != nil {
		return nil, err
	}

	proposerIdx, err := helpers.BeaconProposerIndex(ctx, s)
	if err != nil {
		return nil, errors.Wrap(err, "could not get proposer idx")
	}
	whistleBlowerIdx := proposerIdx
	whistleblowerReward := validator.EffectiveBalance / params.BeaconConfig().WhistleBlowerRewardQuotient
	proposerReward := whistleblowerReward / proposerRewardQuotient
	err = helpers.IncreaseBalance(s, proposerIdx, proposerReward)
	if err != nil {
		return nil, err
	}
	err = helpers.IncreaseBalance(s, whistleBlowerIdx, whistleblowerReward-proposerReward)
	if err != nil {
		return nil, err
	}
	return s, nil
}

// ActivatedValidatorIndices determines the indices activated during the given epoch.
func ActivatedValidatorIndices(epoch primitives.Epoch, validators []*ethpb.Validator) []primitives.ValidatorIndex {
	activations := make([]primitives.ValidatorIndex, 0)
	for i := 0; i < len(validators); i++ {
		val := validators[i]
		if val.ActivationEpoch <= epoch && epoch < val.ExitEpoch {
			activations = append(activations, primitives.ValidatorIndex(i))
		}
	}
	return activations
}

// SlashedValidatorIndices determines the indices slashed during the given epoch.
func SlashedValidatorIndices(epoch primitives.Epoch, validators []*ethpb.Validator) []primitives.ValidatorIndex {
	slashed := make([]primitives.ValidatorIndex, 0)
	for i := 0; i < len(validators); i++ {
		val := validators[i]
		maxWithdrawableEpoch := primitives.MaxEpoch(val.WithdrawableEpoch, epoch+params.BeaconConfig().EpochsPerSlashingsVector)
		if val.WithdrawableEpoch == maxWithdrawableEpoch && val.Slashed {
			slashed = append(slashed, primitives.ValidatorIndex(i))
		}
	}
	return slashed
}

// ExitedValidatorIndices determines the indices exited during the current epoch.
func ExitedValidatorIndices(epoch primitives.Epoch, validators []*ethpb.Validator, activeValidatorCount uint64) ([]primitives.ValidatorIndex, error) {
	exited := make([]primitives.ValidatorIndex, 0)
	exitEpochs := make([]primitives.Epoch, 0)
	for i := 0; i < len(validators); i++ {
		val := validators[i]
		if val.ExitEpoch != params.BeaconConfig().FarFutureEpoch {
			exitEpochs = append(exitEpochs, val.ExitEpoch)
		}
	}
	exitQueueEpoch := primitives.Epoch(0)
	for _, i := range exitEpochs {
		if exitQueueEpoch < i {
			exitQueueEpoch = i
		}
	}

	// We use the exit queue churn to determine if we have passed a churn limit.
	exitQueueChurn := uint64(0)
	for _, val := range validators {
		if val.ExitEpoch == exitQueueEpoch {
			exitQueueChurn++
		}
	}
	churn := helpers.ValidatorExitChurnLimit(activeValidatorCount)
	if churn < exitQueueChurn {
		exitQueueEpoch++
	}
	withdrawableEpoch := exitQueueEpoch + params.BeaconConfig().MinValidatorWithdrawabilityDelay
	for i, val := range validators {
		if val.ExitEpoch == epoch && val.WithdrawableEpoch == withdrawableEpoch &&
			val.EffectiveBalance > params.BeaconConfig().EjectionBalance {
			exited = append(exited, primitives.ValidatorIndex(i))
		}
	}
	return exited, nil
}

// EjectedValidatorIndices determines the indices ejected during the given epoch.
func EjectedValidatorIndices(epoch primitives.Epoch, validators []*ethpb.Validator, activeValidatorCount uint64) ([]primitives.ValidatorIndex, error) {
	ejected := make([]primitives.ValidatorIndex, 0)
	exitEpochs := make([]primitives.Epoch, 0)
	for i := 0; i < len(validators); i++ {
		val := validators[i]
		if val.ExitEpoch != params.BeaconConfig().FarFutureEpoch {
			exitEpochs = append(exitEpochs, val.ExitEpoch)
		}
	}
	exitQueueEpoch := primitives.Epoch(0)
	for _, i := range exitEpochs {
		if exitQueueEpoch < i {
			exitQueueEpoch = i
		}
	}

	// We use the exit queue churn to determine if we have passed a churn limit.
	exitQueueChurn := uint64(0)
	for _, val := range validators {
		if val.ExitEpoch == exitQueueEpoch {
			exitQueueChurn++
		}
	}
	churn := helpers.ValidatorExitChurnLimit(activeValidatorCount)
	if churn < exitQueueChurn {
		exitQueueEpoch++
	}
	withdrawableEpoch := exitQueueEpoch + params.BeaconConfig().MinValidatorWithdrawabilityDelay
	for i, val := range validators {
		if val.ExitEpoch == epoch && val.WithdrawableEpoch == withdrawableEpoch &&
			val.EffectiveBalance <= params.BeaconConfig().EjectionBalance {
			ejected = append(ejected, primitives.ValidatorIndex(i))
		}
	}
	return ejected, nil
}

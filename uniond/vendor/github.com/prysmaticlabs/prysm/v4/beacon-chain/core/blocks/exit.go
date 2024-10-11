package blocks

import (
	"context"
	"fmt"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	v "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/validators"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

// ValidatorAlreadyExitedMsg defines a message saying that a validator has already exited.
var ValidatorAlreadyExitedMsg = "has already submitted an exit, which will take place at epoch"

// ValidatorCannotExitYetMsg defines a message saying that a validator cannot exit
// because it has not been active long enough.
var ValidatorCannotExitYetMsg = "validator has not been active long enough to exit"

// ProcessVoluntaryExits is one of the operations performed
// on each processed beacon block to determine which validators
// should exit the state's validator registry.
//
// Spec pseudocode definition:
//
//	def process_voluntary_exit(state: BeaconState, signed_voluntary_exit: SignedVoluntaryExit) -> None:
//	 voluntary_exit = signed_voluntary_exit.message
//	 validator = state.validators[voluntary_exit.validator_index]
//	 # Verify the validator is active
//	 assert is_active_validator(validator, get_current_epoch(state))
//	 # Verify exit has not been initiated
//	 assert validator.exit_epoch == FAR_FUTURE_EPOCH
//	 # Exits must specify an epoch when they become valid; they are not valid before then
//	 assert get_current_epoch(state) >= voluntary_exit.epoch
//	 # Verify the validator has been active long enough
//	 assert get_current_epoch(state) >= validator.activation_epoch + SHARD_COMMITTEE_PERIOD
//	 # Verify signature
//	 domain = get_domain(state, DOMAIN_VOLUNTARY_EXIT, voluntary_exit.epoch)
//	 signing_root = compute_signing_root(voluntary_exit, domain)
//	 assert bls.Verify(validator.pubkey, signing_root, signed_voluntary_exit.signature)
//	 # Initiate exit
//	 initiate_validator_exit(state, voluntary_exit.validator_index)
func ProcessVoluntaryExits(
	ctx context.Context,
	beaconState state.BeaconState,
	exits []*ethpb.SignedVoluntaryExit,
) (state.BeaconState, error) {
	// Avoid calculating the epoch churn if no exits exist.
	if len(exits) == 0 {
		return beaconState, nil
	}
	maxExitEpoch, churn := v.MaxExitEpochAndChurn(beaconState)
	var exitEpoch primitives.Epoch
	for idx, exit := range exits {
		if exit == nil || exit.Exit == nil {
			return nil, errors.New("nil voluntary exit in block body")
		}
		val, err := beaconState.ValidatorAtIndexReadOnly(exit.Exit.ValidatorIndex)
		if err != nil {
			return nil, err
		}
		if err := VerifyExitAndSignature(val, beaconState, exit); err != nil {
			return nil, errors.Wrapf(err, "could not verify exit %d", idx)
		}
		beaconState, exitEpoch, err = v.InitiateValidatorExit(ctx, beaconState, exit.Exit.ValidatorIndex, maxExitEpoch, churn)
		if err == nil {
			if exitEpoch > maxExitEpoch {
				maxExitEpoch = exitEpoch
				churn = 1
			} else if exitEpoch == maxExitEpoch {
				churn++
			}
		} else if !errors.Is(err, v.ErrValidatorAlreadyExited) {
			return nil, err
		}
	}
	return beaconState, nil
}

// VerifyExitAndSignature implements the spec defined validation for voluntary exits.
//
// Spec pseudocode definition:
//
//	def process_voluntary_exit(state: BeaconState, signed_voluntary_exit: SignedVoluntaryExit) -> None:
//	 voluntary_exit = signed_voluntary_exit.message
//	 validator = state.validators[voluntary_exit.validator_index]
//	 # Verify the validator is active
//	 assert is_active_validator(validator, get_current_epoch(state))
//	 # Verify exit has not been initiated
//	 assert validator.exit_epoch == FAR_FUTURE_EPOCH
//	 # Exits must specify an epoch when they become valid; they are not valid before then
//	 assert get_current_epoch(state) >= voluntary_exit.epoch
//	 # Verify the validator has been active long enough
//	 assert get_current_epoch(state) >= validator.activation_epoch + SHARD_COMMITTEE_PERIOD
//	 # Verify signature
//	 domain = get_domain(state, DOMAIN_VOLUNTARY_EXIT, voluntary_exit.epoch)
//	 signing_root = compute_signing_root(voluntary_exit, domain)
//	 assert bls.Verify(validator.pubkey, signing_root, signed_voluntary_exit.signature)
//	 # Initiate exit
//	 initiate_validator_exit(state, voluntary_exit.validator_index)
func VerifyExitAndSignature(
	validator state.ReadOnlyValidator,
	state state.ReadOnlyBeaconState,
	signed *ethpb.SignedVoluntaryExit,
) error {
	if signed == nil || signed.Exit == nil {
		return errors.New("nil exit")
	}

	currentSlot := state.Slot()
	fork := state.Fork()
	genesisRoot := state.GenesisValidatorsRoot()

	// EIP-7044: Beginning in Deneb, fix the fork version to Capella.
	// This allows for signed validator exits to be valid forever.
	if state.Version() >= version.Deneb {
		fork = &ethpb.Fork{
			PreviousVersion: params.BeaconConfig().CapellaForkVersion,
			CurrentVersion:  params.BeaconConfig().CapellaForkVersion,
			Epoch:           params.BeaconConfig().CapellaForkEpoch,
		}
	}

	exit := signed.Exit
	if err := verifyExitConditions(validator, currentSlot, exit); err != nil {
		return err
	}
	domain, err := signing.Domain(fork, exit.Epoch, params.BeaconConfig().DomainVoluntaryExit, genesisRoot)
	if err != nil {
		return err
	}
	valPubKey := validator.PublicKey()
	if err := signing.VerifySigningRoot(exit, valPubKey[:], signed.Signature, domain); err != nil {
		return signing.ErrSigFailedToVerify
	}
	return nil
}

// verifyExitConditions implements the spec defined validation for voluntary exits (excluding signatures).
//
// Spec pseudocode definition:
//
//	def process_voluntary_exit(state: BeaconState, signed_voluntary_exit: SignedVoluntaryExit) -> None:
//	 voluntary_exit = signed_voluntary_exit.message
//	 validator = state.validators[voluntary_exit.validator_index]
//	 # Verify the validator is active
//	 assert is_active_validator(validator, get_current_epoch(state))
//	 # Verify exit has not been initiated
//	 assert validator.exit_epoch == FAR_FUTURE_EPOCH
//	 # Exits must specify an epoch when they become valid; they are not valid before then
//	 assert get_current_epoch(state) >= voluntary_exit.epoch
//	 # Verify the validator has been active long enough
//	 assert get_current_epoch(state) >= validator.activation_epoch + SHARD_COMMITTEE_PERIOD
//	 # Verify signature
//	 domain = get_domain(state, DOMAIN_VOLUNTARY_EXIT, voluntary_exit.epoch)
//	 signing_root = compute_signing_root(voluntary_exit, domain)
//	 assert bls.Verify(validator.pubkey, signing_root, signed_voluntary_exit.signature)
//	 # Initiate exit
//	 initiate_validator_exit(state, voluntary_exit.validator_index)
func verifyExitConditions(validator state.ReadOnlyValidator, currentSlot primitives.Slot, exit *ethpb.VoluntaryExit) error {
	currentEpoch := slots.ToEpoch(currentSlot)
	// Verify the validator is active.
	if !helpers.IsActiveValidatorUsingTrie(validator, currentEpoch) {
		return errors.New("non-active validator cannot exit")
	}
	// Verify the validator has not yet submitted an exit.
	if validator.ExitEpoch() != params.BeaconConfig().FarFutureEpoch {
		return fmt.Errorf("validator with index %d %s: %v", exit.ValidatorIndex, ValidatorAlreadyExitedMsg, validator.ExitEpoch())
	}
	// Exits must specify an epoch when they become valid; they are not valid before then.
	if currentEpoch < exit.Epoch {
		return fmt.Errorf("expected current epoch >= exit epoch, received %d < %d", currentEpoch, exit.Epoch)
	}
	// Verify the validator has been active long enough.
	if currentEpoch < validator.ActivationEpoch()+params.BeaconConfig().ShardCommitteePeriod {
		return fmt.Errorf(
			"%s: %d of %d epochs. Validator will be eligible for exit at epoch %d",
			ValidatorCannotExitYetMsg,
			currentEpoch-validator.ActivationEpoch(),
			params.BeaconConfig().ShardCommitteePeriod,
			validator.ActivationEpoch()+params.BeaconConfig().ShardCommitteePeriod,
		)
	}
	return nil
}

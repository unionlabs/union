package state_native

import (
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	consensus_types "github.com/prysmaticlabs/prysm/v4/consensus-types"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
)

// Validators participating in consensus on the beacon chain.
func (b *BeaconState) Validators() []*ethpb.Validator {
	b.lock.RLock()
	defer b.lock.RUnlock()

	return b.validatorsVal()
}

func (b *BeaconState) validatorsVal() []*ethpb.Validator {
	var v []*ethpb.Validator
	if features.Get().EnableExperimentalState {
		if b.validatorsMultiValue == nil {
			return nil
		}
		v = b.validatorsMultiValue.Value(b)
	} else {
		if b.validators == nil {
			return nil
		}
		v = b.validators
	}

	res := make([]*ethpb.Validator, len(v))
	for i := 0; i < len(res); i++ {
		val := v[i]
		if val == nil {
			continue
		}
		res[i] = ethpb.CopyValidator(val)
	}
	return res
}

// references of validators participating in consensus on the beacon chain.
// This assumes that a lock is already held on BeaconState. This does not
// copy fully and instead just copies the reference.
func (b *BeaconState) validatorsReferences() []*ethpb.Validator {
	if b.validators == nil {
		return nil
	}

	res := make([]*ethpb.Validator, len(b.validators), len(b.validators)+int(params.BeaconConfig().MaxDeposits))
	for i := 0; i < len(res); i++ {
		validator := b.validators[i]
		if validator == nil {
			continue
		}
		// copy validator reference instead.
		res[i] = validator
	}
	return res
}

func (b *BeaconState) validatorsLen() int {
	if features.Get().EnableExperimentalState {
		if b.validatorsMultiValue == nil {
			return 0
		}
		return b.validatorsMultiValue.Len(b)
	}
	return len(b.validators)
}

// ValidatorAtIndex is the validator at the provided index.
func (b *BeaconState) ValidatorAtIndex(idx primitives.ValidatorIndex) (*ethpb.Validator, error) {
	b.lock.RLock()
	defer b.lock.RUnlock()

	return b.validatorAtIndex(idx)
}

func (b *BeaconState) validatorAtIndex(idx primitives.ValidatorIndex) (*ethpb.Validator, error) {
	if features.Get().EnableExperimentalState {
		if b.validatorsMultiValue == nil {
			return &ethpb.Validator{}, nil
		}
		v, err := b.validatorsMultiValue.At(b, uint64(idx))
		if err != nil {
			return nil, err
		}
		return ethpb.CopyValidator(v), nil
	}

	if b.validators == nil {
		return &ethpb.Validator{}, nil
	}
	if uint64(len(b.validators)) <= uint64(idx) {
		return nil, errors.Wrapf(consensus_types.ErrOutOfBounds, "validator index %d does not exist", idx)
	}
	val := b.validators[idx]
	return ethpb.CopyValidator(val), nil
}

// ValidatorAtIndexReadOnly is the validator at the provided index. This method
// doesn't clone the validator.
func (b *BeaconState) ValidatorAtIndexReadOnly(idx primitives.ValidatorIndex) (state.ReadOnlyValidator, error) {
	b.lock.RLock()
	defer b.lock.RUnlock()

	if features.Get().EnableExperimentalState {
		if b.validatorsMultiValue == nil {
			return nil, state.ErrNilValidatorsInState
		}
		v, err := b.validatorsMultiValue.At(b, uint64(idx))
		if err != nil {
			return nil, err
		}
		return NewValidator(v)
	}

	if b.validators == nil {
		return nil, state.ErrNilValidatorsInState
	}
	if uint64(len(b.validators)) <= uint64(idx) {
		return nil, errors.Wrapf(consensus_types.ErrOutOfBounds, "validator index %d does not exist", idx)
	}
	val := b.validators[idx]
	return NewValidator(val)
}

// ValidatorIndexByPubkey returns a given validator by its 48-byte public key.
func (b *BeaconState) ValidatorIndexByPubkey(key [fieldparams.BLSPubkeyLength]byte) (primitives.ValidatorIndex, bool) {
	if b == nil || b.valMapHandler == nil || b.valMapHandler.IsNil() {
		return 0, false
	}
	b.lock.RLock()
	defer b.lock.RUnlock()

	var numOfVals int
	if features.Get().EnableExperimentalState {
		numOfVals = b.validatorsMultiValue.Len(b)
	} else {
		numOfVals = len(b.validators)
	}

	idx, ok := b.valMapHandler.Get(key)
	if ok && primitives.ValidatorIndex(numOfVals) <= idx {
		return primitives.ValidatorIndex(0), false
	}
	return idx, ok
}

// PubkeyAtIndex returns the pubkey at the given
// validator index.
func (b *BeaconState) PubkeyAtIndex(idx primitives.ValidatorIndex) [fieldparams.BLSPubkeyLength]byte {
	b.lock.RLock()
	defer b.lock.RUnlock()

	var v *ethpb.Validator
	if features.Get().EnableExperimentalState {
		var err error
		v, err = b.validatorsMultiValue.At(b, uint64(idx))
		if err != nil {
			return [fieldparams.BLSPubkeyLength]byte{}
		}
	} else {
		if uint64(idx) >= uint64(len(b.validators)) {
			return [fieldparams.BLSPubkeyLength]byte{}
		}
		v = b.validators[idx]
	}

	if v == nil {
		return [fieldparams.BLSPubkeyLength]byte{}
	}
	return bytesutil.ToBytes48(v.PublicKey)
}

// PublicKeys builds a list of all validator public keys, with each key's index aligned to its validator index.
func (b *BeaconState) PublicKeys() ([][fieldparams.BLSPubkeyLength]byte, error) {
	b.lock.RLock()
	defer b.lock.RUnlock()

	l := b.validatorsLen()
	res := make([][fieldparams.BLSPubkeyLength]byte, l)
	for i := 0; i < l; i++ {
		if features.Get().EnableExperimentalState {
			val, err := b.validatorsMultiValue.At(b, uint64(i))
			if err != nil {
				return nil, err
			}
			copy(res[i][:], val.PublicKey)
		} else {
			copy(res[i][:], b.validators[i].PublicKey)
		}
	}
	return res, nil
}

// NumValidators returns the size of the validator registry.
func (b *BeaconState) NumValidators() int {
	b.lock.RLock()
	defer b.lock.RUnlock()

	return b.validatorsLen()
}

// ReadFromEveryValidator reads values from every validator and applies it to the provided function.
//
// WARNING: This method is potentially unsafe, as it exposes the actual validator registry.
func (b *BeaconState) ReadFromEveryValidator(f func(idx int, val state.ReadOnlyValidator) error) error {
	b.lock.RLock()
	defer b.lock.RUnlock()

	if features.Get().EnableExperimentalState {
		return b.readFromEveryValidatorMVSlice(f)
	}

	if b.validators == nil {
		return state.ErrNilValidatorsInState
	}

	validators := b.validators

	for i, v := range validators {
		v, err := NewValidator(v)
		if err != nil {
			return err
		}
		if err = f(i, v); err != nil {
			return err
		}
	}
	return nil
}

// WARNING: This function works only for the multi-value slice feature.
func (b *BeaconState) readFromEveryValidatorMVSlice(f func(idx int, val state.ReadOnlyValidator) error) error {
	if b.validatorsMultiValue == nil {
		return state.ErrNilValidatorsInState
	}
	l := b.validatorsMultiValue.Len(b)
	for i := 0; i < l; i++ {
		v, err := b.validatorsMultiValue.At(b, uint64(i))
		if err != nil {
			return err
		}
		rov, err := NewValidator(v)
		if err != nil {
			return err
		}
		if err = f(i, rov); err != nil {
			return err
		}
	}
	return nil
}

// Balances of validators participating in consensus on the beacon chain.
func (b *BeaconState) Balances() []uint64 {
	b.lock.RLock()
	defer b.lock.RUnlock()

	return b.balancesVal()
}

func (b *BeaconState) balancesVal() []uint64 {
	if features.Get().EnableExperimentalState {
		if b.balancesMultiValue == nil {
			return nil
		}
		return b.balancesMultiValue.Value(b)
	}
	if b.balances == nil {
		return nil
	}
	res := make([]uint64, len(b.balances))
	copy(res, b.balances)
	return res
}

// BalanceAtIndex of validator with the provided index.
func (b *BeaconState) BalanceAtIndex(idx primitives.ValidatorIndex) (uint64, error) {
	b.lock.RLock()
	defer b.lock.RUnlock()

	return b.balanceAtIndex(idx)
}

func (b *BeaconState) balanceAtIndex(idx primitives.ValidatorIndex) (uint64, error) {
	if features.Get().EnableExperimentalState {
		if b.balancesMultiValue == nil {
			return 0, nil
		}
		return b.balancesMultiValue.At(b, uint64(idx))
	}
	if b.balances == nil {
		return 0, nil
	}
	if uint64(len(b.balances)) <= uint64(idx) {
		return 0, errors.Wrapf(consensus_types.ErrOutOfBounds, "balance index %d does not exist", idx)
	}
	return b.balances[idx], nil
}

// BalancesLength returns the length of the balances slice.
func (b *BeaconState) BalancesLength() int {
	b.lock.RLock()
	defer b.lock.RUnlock()

	if features.Get().EnableExperimentalState {
		if b.balancesMultiValue == nil {
			return 0
		}
		return b.balancesMultiValue.Len(b)
	}
	return len(b.balances)
}

// Slashings of validators on the beacon chain.
func (b *BeaconState) Slashings() []uint64 {
	if b.slashings == nil {
		return nil
	}

	b.lock.RLock()
	defer b.lock.RUnlock()

	return b.slashingsVal()
}

// slashingsVal of validators on the beacon chain.
// This assumes that a lock is already held on BeaconState.
func (b *BeaconState) slashingsVal() []uint64 {
	if b.slashings == nil {
		return nil
	}

	res := make([]uint64, len(b.slashings))
	copy(res, b.slashings)
	return res
}

// InactivityScores of validators participating in consensus on the beacon chain.
func (b *BeaconState) InactivityScores() ([]uint64, error) {
	if b.version == version.Phase0 {
		return nil, errNotSupported("InactivityScores", b.version)
	}

	b.lock.RLock()
	defer b.lock.RUnlock()

	return b.inactivityScoresVal(), nil
}

func (b *BeaconState) inactivityScoresVal() []uint64 {
	if features.Get().EnableExperimentalState {
		if b.inactivityScoresMultiValue == nil {
			return nil
		}
		return b.inactivityScoresMultiValue.Value(b)
	}
	if b.inactivityScores == nil {
		return nil
	}
	res := make([]uint64, len(b.inactivityScores))
	copy(res, b.inactivityScores)
	return res
}

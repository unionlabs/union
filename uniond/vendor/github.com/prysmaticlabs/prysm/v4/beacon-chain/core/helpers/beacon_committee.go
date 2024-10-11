// Package helpers contains helper functions outlined in the Ethereum Beacon Chain spec, such as
// computing committees, randao, rewards/penalties, and more.
package helpers

import (
	"context"
	"fmt"
	"sort"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	forkchoicetypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/container/slice"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/math"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

var (
	committeeCache       = cache.NewCommitteesCache()
	proposerIndicesCache = cache.NewProposerIndicesCache()
)

// SlotCommitteeCount returns the number of beacon committees of a slot. The
// active validator count is provided as an argument rather than an imported implementation
// from the spec definition. Having the active validator count as an argument allows for
// cheaper computation, instead of retrieving head state, one can retrieve the validator
// count.
//
// Spec pseudocode definition:
//
//	def get_committee_count_per_slot(state: BeaconState, epoch: Epoch) -> uint64:
//	 """
//	 Return the number of committees in each slot for the given ``epoch``.
//	 """
//	 return max(uint64(1), min(
//	     MAX_COMMITTEES_PER_SLOT,
//	     uint64(len(get_active_validator_indices(state, epoch))) // SLOTS_PER_EPOCH // TARGET_COMMITTEE_SIZE,
//	 ))
func SlotCommitteeCount(activeValidatorCount uint64) uint64 {
	var committeesPerSlot = activeValidatorCount / uint64(params.BeaconConfig().SlotsPerEpoch) / params.BeaconConfig().TargetCommitteeSize

	if committeesPerSlot > params.BeaconConfig().MaxCommitteesPerSlot {
		return params.BeaconConfig().MaxCommitteesPerSlot
	}
	if committeesPerSlot == 0 {
		return 1
	}

	return committeesPerSlot
}

// BeaconCommitteeFromState returns the crosslink committee of a given slot and committee index. This
// is a spec implementation where state is used as an argument. In case of state retrieval
// becomes expensive, consider using BeaconCommittee below.
//
// Spec pseudocode definition:
//
//	def get_beacon_committee(state: BeaconState, slot: Slot, index: CommitteeIndex) -> Sequence[ValidatorIndex]:
//	 """
//	 Return the beacon committee at ``slot`` for ``index``.
//	 """
//	 epoch = compute_epoch_at_slot(slot)
//	 committees_per_slot = get_committee_count_per_slot(state, epoch)
//	 return compute_committee(
//	     indices=get_active_validator_indices(state, epoch),
//	     seed=get_seed(state, epoch, DOMAIN_BEACON_ATTESTER),
//	     index=(slot % SLOTS_PER_EPOCH) * committees_per_slot + index,
//	     count=committees_per_slot * SLOTS_PER_EPOCH,
//	 )
func BeaconCommitteeFromState(ctx context.Context, state state.ReadOnlyBeaconState, slot primitives.Slot, committeeIndex primitives.CommitteeIndex) ([]primitives.ValidatorIndex, error) {
	epoch := slots.ToEpoch(slot)
	seed, err := Seed(state, epoch, params.BeaconConfig().DomainBeaconAttester)
	if err != nil {
		return nil, errors.Wrap(err, "could not get seed")
	}

	committee, err := committeeCache.Committee(ctx, slot, seed, committeeIndex)
	if err != nil {
		return nil, errors.Wrap(err, "could not interface with committee cache")
	}
	if committee != nil {
		return committee, nil
	}

	activeIndices, err := ActiveValidatorIndices(ctx, state, epoch)
	if err != nil {
		return nil, errors.Wrap(err, "could not get active indices")
	}

	return BeaconCommittee(ctx, activeIndices, seed, slot, committeeIndex)
}

// BeaconCommittee returns the beacon committee of a given slot and committee index. The
// validator indices and seed are provided as an argument rather than an imported implementation
// from the spec definition. Having them as an argument allows for cheaper computation run time.
//
// Spec pseudocode definition:
//
//	def get_beacon_committee(state: BeaconState, slot: Slot, index: CommitteeIndex) -> Sequence[ValidatorIndex]:
//	 """
//	 Return the beacon committee at ``slot`` for ``index``.
//	 """
//	 epoch = compute_epoch_at_slot(slot)
//	 committees_per_slot = get_committee_count_per_slot(state, epoch)
//	 return compute_committee(
//	     indices=get_active_validator_indices(state, epoch),
//	     seed=get_seed(state, epoch, DOMAIN_BEACON_ATTESTER),
//	     index=(slot % SLOTS_PER_EPOCH) * committees_per_slot + index,
//	     count=committees_per_slot * SLOTS_PER_EPOCH,
//	 )
func BeaconCommittee(
	ctx context.Context,
	validatorIndices []primitives.ValidatorIndex,
	seed [32]byte,
	slot primitives.Slot,
	committeeIndex primitives.CommitteeIndex,
) ([]primitives.ValidatorIndex, error) {
	committee, err := committeeCache.Committee(ctx, slot, seed, committeeIndex)
	if err != nil {
		return nil, errors.Wrap(err, "could not interface with committee cache")
	}
	if committee != nil {
		return committee, nil
	}

	committeesPerSlot := SlotCommitteeCount(uint64(len(validatorIndices)))

	indexOffset, err := math.Add64(uint64(committeeIndex), uint64(slot.ModSlot(params.BeaconConfig().SlotsPerEpoch).Mul(committeesPerSlot)))
	if err != nil {
		return nil, errors.Wrap(err, "could not add calculate index offset")
	}
	count := committeesPerSlot * uint64(params.BeaconConfig().SlotsPerEpoch)

	return computeCommittee(validatorIndices, seed, indexOffset, count)
}

// CommitteeAssignmentContainer represents a committee list, committee index, and to be attested slot for a given epoch.
type CommitteeAssignmentContainer struct {
	Committee      []primitives.ValidatorIndex
	AttesterSlot   primitives.Slot
	CommitteeIndex primitives.CommitteeIndex
}

// CommitteeAssignments is a map of validator indices pointing to the appropriate committee
// assignment for the given epoch.
//
// 1. Determine the proposer validator index for each slot.
// 2. Compute all committees.
// 3. Determine the attesting slot for each committee.
// 4. Construct a map of validator indices pointing to the respective committees.
func CommitteeAssignments(
	ctx context.Context,
	state state.BeaconState,
	epoch primitives.Epoch,
) (map[primitives.ValidatorIndex]*CommitteeAssignmentContainer, map[primitives.ValidatorIndex][]primitives.Slot, error) {
	nextEpoch := time.NextEpoch(state)
	if epoch > nextEpoch {
		return nil, nil, fmt.Errorf(
			"epoch %d can't be greater than next epoch %d",
			epoch,
			nextEpoch,
		)
	}

	// We determine the slots in which proposers are supposed to act.
	// Some validators may need to propose multiple times per epoch, so
	// we use a map of proposer idx -> []slot to keep track of this possibility.
	startSlot, err := slots.EpochStart(epoch)
	if err != nil {
		return nil, nil, err
	}
	minValidStartSlot := primitives.Slot(0)
	if state.Slot() >= params.BeaconConfig().SlotsPerHistoricalRoot {
		minValidStartSlot = state.Slot() - params.BeaconConfig().SlotsPerHistoricalRoot
	}
	if startSlot < minValidStartSlot {
		return nil, nil, fmt.Errorf("start slot %d is smaller than the minimum valid start slot %d", startSlot, minValidStartSlot)
	}

	proposerIndexToSlots := make(map[primitives.ValidatorIndex][]primitives.Slot, params.BeaconConfig().SlotsPerEpoch)
	for slot := startSlot; slot < startSlot+params.BeaconConfig().SlotsPerEpoch; slot++ {
		// Skip proposer assignment for genesis slot.
		if slot == 0 {
			continue
		}
		if err := state.SetSlot(slot); err != nil {
			return nil, nil, err
		}
		i, err := BeaconProposerIndex(ctx, state)
		if err != nil {
			return nil, nil, errors.Wrapf(err, "could not check proposer at slot %d", state.Slot())
		}
		proposerIndexToSlots[i] = append(proposerIndexToSlots[i], slot)
	}

	// If previous proposer indices computation is outside if current proposal epoch range,
	// we need to reset state slot back to start slot so that we can compute the correct committees.
	currentProposalEpoch := epoch < nextEpoch
	if !currentProposalEpoch {
		if err := state.SetSlot(state.Slot() - params.BeaconConfig().SlotsPerEpoch); err != nil {
			return nil, nil, err
		}
	}

	activeValidatorIndices, err := ActiveValidatorIndices(ctx, state, epoch)
	if err != nil {
		return nil, nil, err
	}
	// Each slot in an epoch has a different set of committees. This value is derived from the
	// active validator set, which does not change.
	numCommitteesPerSlot := SlotCommitteeCount(uint64(len(activeValidatorIndices)))
	validatorIndexToCommittee := make(map[primitives.ValidatorIndex]*CommitteeAssignmentContainer, len(activeValidatorIndices))

	// Compute all committees for all slots.
	for i := primitives.Slot(0); i < params.BeaconConfig().SlotsPerEpoch; i++ {
		// Compute committees.
		for j := uint64(0); j < numCommitteesPerSlot; j++ {
			slot := startSlot + i
			committee, err := BeaconCommitteeFromState(ctx, state, slot, primitives.CommitteeIndex(j) /*committee index*/)
			if err != nil {
				return nil, nil, err
			}

			cac := &CommitteeAssignmentContainer{
				Committee:      committee,
				CommitteeIndex: primitives.CommitteeIndex(j),
				AttesterSlot:   slot,
			}
			for _, vIndex := range committee {
				validatorIndexToCommittee[vIndex] = cac
			}
		}
	}

	return validatorIndexToCommittee, proposerIndexToSlots, nil
}

// VerifyBitfieldLength verifies that a bitfield length matches the given committee size.
func VerifyBitfieldLength(bf bitfield.Bitfield, committeeSize uint64) error {
	if bf.Len() != committeeSize {
		return fmt.Errorf(
			"wanted participants bitfield length %d, got: %d",
			committeeSize,
			bf.Len())
	}
	return nil
}

// VerifyAttestationBitfieldLengths verifies that an attestations aggregation bitfields is
// a valid length matching the size of the committee.
func VerifyAttestationBitfieldLengths(ctx context.Context, state state.ReadOnlyBeaconState, att *ethpb.Attestation) error {
	committee, err := BeaconCommitteeFromState(ctx, state, att.Data.Slot, att.Data.CommitteeIndex)
	if err != nil {
		return errors.Wrap(err, "could not retrieve beacon committees")
	}

	if committee == nil {
		return errors.New("no committee exist for this attestation")
	}

	if err := VerifyBitfieldLength(att.AggregationBits, uint64(len(committee))); err != nil {
		return errors.Wrap(err, "failed to verify aggregation bitfield")
	}
	return nil
}

// ShuffledIndices uses input beacon state and returns the shuffled indices of the input epoch,
// the shuffled indices then can be used to break up into committees.
func ShuffledIndices(s state.ReadOnlyBeaconState, epoch primitives.Epoch) ([]primitives.ValidatorIndex, error) {
	seed, err := Seed(s, epoch, params.BeaconConfig().DomainBeaconAttester)
	if err != nil {
		return nil, errors.Wrapf(err, "could not get seed for epoch %d", epoch)
	}

	indices := make([]primitives.ValidatorIndex, 0, s.NumValidators())
	if err := s.ReadFromEveryValidator(func(idx int, val state.ReadOnlyValidator) error {
		if IsActiveValidatorUsingTrie(val, epoch) {
			indices = append(indices, primitives.ValidatorIndex(idx))
		}
		return nil
	}); err != nil {
		return nil, err
	}

	// UnshuffleList is used as an optimized implementation for raw speed.
	return UnshuffleList(indices, seed)
}

// UpdateCommitteeCache gets called at the beginning of every epoch to cache the committee shuffled indices
// list with committee index and epoch number. It caches the shuffled indices for the input epoch.
func UpdateCommitteeCache(ctx context.Context, state state.ReadOnlyBeaconState, e primitives.Epoch) error {
	seed, err := Seed(state, e, params.BeaconConfig().DomainBeaconAttester)
	if err != nil {
		return err
	}
	if committeeCache.HasEntry(string(seed[:])) {
		return nil
	}
	shuffledIndices, err := ShuffledIndices(state, e)
	if err != nil {
		return err
	}

	count := SlotCommitteeCount(uint64(len(shuffledIndices)))

	// Store the sorted indices as well as shuffled indices. In current spec,
	// sorted indices is required to retrieve proposer index. This is also
	// used for failing verify signature fallback.
	sortedIndices := make([]primitives.ValidatorIndex, len(shuffledIndices))
	copy(sortedIndices, shuffledIndices)
	sort.Slice(sortedIndices, func(i, j int) bool {
		return sortedIndices[i] < sortedIndices[j]
	})

	if err := committeeCache.AddCommitteeShuffledList(ctx, &cache.Committees{
		ShuffledIndices: shuffledIndices,
		CommitteeCount:  uint64(params.BeaconConfig().SlotsPerEpoch.Mul(count)),
		Seed:            seed,
		SortedIndices:   sortedIndices,
	}); err != nil {
		return err
	}
	return nil
}

// UpdateProposerIndicesInCache updates proposer indices entry of the committee cache.
// Input state is used to retrieve active validator indices.
// Input root is to use as key in the cache.
// Input epoch is the epoch to retrieve proposer indices for.
func UpdateProposerIndicesInCache(ctx context.Context, state state.ReadOnlyBeaconState, epoch primitives.Epoch) error {
	// The cache uses the state root at the end of (current epoch - 1) as key.
	// (e.g. for epoch 2, the key is root at slot 63)
	if epoch <= params.BeaconConfig().GenesisEpoch+params.BeaconConfig().MinSeedLookahead {
		return nil
	}
	slot, err := slots.EpochEnd(epoch - 1)
	if err != nil {
		return err
	}
	root, err := StateRootAtSlot(state, slot)
	if err != nil {
		return err
	}
	// Skip cache update if the key already exists
	_, ok := proposerIndicesCache.ProposerIndices(epoch, [32]byte(root))
	if ok {
		return nil
	}
	indices, err := ActiveValidatorIndices(ctx, state, epoch)
	if err != nil {
		return err
	}
	proposerIndices, err := precomputeProposerIndices(state, indices, epoch)
	if err != nil {
		return err
	}
	if len(proposerIndices) != int(params.BeaconConfig().SlotsPerEpoch) {
		return errors.New("invalid proposer length returned from state")
	}
	// This is here to deal with tests only
	var indicesArray [fieldparams.SlotsPerEpoch]primitives.ValidatorIndex
	copy(indicesArray[:], proposerIndices)
	proposerIndicesCache.Prune(epoch - 2)
	proposerIndicesCache.Set(epoch, [32]byte(root), indicesArray)
	return nil
}

// UpdateCachedCheckpointToStateRoot updates the map from checkpoints to state root in the proposer indices cache
func UpdateCachedCheckpointToStateRoot(state state.ReadOnlyBeaconState, cp *forkchoicetypes.Checkpoint) error {
	if cp.Epoch <= params.BeaconConfig().GenesisEpoch+params.BeaconConfig().MinSeedLookahead {
		return nil
	}
	slot, err := slots.EpochEnd(cp.Epoch)
	if err != nil {
		return err
	}
	root, err := state.StateRootAtIndex(uint64(slot % params.BeaconConfig().SlotsPerHistoricalRoot))
	if err != nil {
		return err
	}
	proposerIndicesCache.SetCheckpoint(*cp, [32]byte(root))
	return nil
}

// ClearCache clears the beacon committee cache and sync committee cache.
func ClearCache() {
	committeeCache.Clear()
	proposerIndicesCache.Prune(0)
	syncCommitteeCache.Clear()
	balanceCache.Clear()
}

// computeCommittee returns the requested shuffled committee out of the total committees using
// validator indices and seed.
//
// Spec pseudocode definition:
//
//	def compute_committee(indices: Sequence[ValidatorIndex],
//	                    seed: Bytes32,
//	                    index: uint64,
//	                    count: uint64) -> Sequence[ValidatorIndex]:
//	  """
//	  Return the committee corresponding to ``indices``, ``seed``, ``index``, and committee ``count``.
//	  """
//	  start = (len(indices) * index) // count
//	  end = (len(indices) * uint64(index + 1)) // count
//	  return [indices[compute_shuffled_index(uint64(i), uint64(len(indices)), seed)] for i in range(start, end)]
func computeCommittee(
	indices []primitives.ValidatorIndex,
	seed [32]byte,
	index, count uint64,
) ([]primitives.ValidatorIndex, error) {
	validatorCount := uint64(len(indices))
	start := slice.SplitOffset(validatorCount, count, index)
	end := slice.SplitOffset(validatorCount, count, index+1)

	if start > validatorCount || end > validatorCount {
		return nil, errors.New("index out of range")
	}

	// Save the shuffled indices in cache, this is only needed once per epoch or once per new committee index.
	shuffledIndices := make([]primitives.ValidatorIndex, len(indices))
	copy(shuffledIndices, indices)
	// UnshuffleList is used here as it is an optimized implementation created
	// for fast computation of committees.
	// Reference implementation: https://github.com/protolambda/eth2-shuffle
	shuffledList, err := UnshuffleList(shuffledIndices, seed)
	if err != nil {
		return nil, err
	}

	return shuffledList[start:end], nil
}

// This computes proposer indices of the current epoch and returns a list of proposer indices,
// the index of the list represents the slot number.
func precomputeProposerIndices(state state.ReadOnlyBeaconState, activeIndices []primitives.ValidatorIndex, e primitives.Epoch) ([]primitives.ValidatorIndex, error) {
	hashFunc := hash.CustomSHA256Hasher()
	proposerIndices := make([]primitives.ValidatorIndex, params.BeaconConfig().SlotsPerEpoch)

	seed, err := Seed(state, e, params.BeaconConfig().DomainBeaconProposer)
	if err != nil {
		return nil, errors.Wrap(err, "could not generate seed")
	}
	slot, err := slots.EpochStart(e)
	if err != nil {
		return nil, err
	}
	for i := uint64(0); i < uint64(params.BeaconConfig().SlotsPerEpoch); i++ {
		seedWithSlot := append(seed[:], bytesutil.Bytes8(uint64(slot)+i)...)
		seedWithSlotHash := hashFunc(seedWithSlot)
		index, err := ComputeProposerIndex(state, activeIndices, seedWithSlotHash)
		if err != nil {
			return nil, err
		}
		proposerIndices[i] = index
	}

	return proposerIndices, nil
}

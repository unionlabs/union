package altair

import (
	"context"
	goErrors "errors"
	"fmt"
	"time"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	coreTime "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/math"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

const maxRandomByte = uint64(1<<8 - 1)

var (
	ErrTooLate = errors.New("sync message is too late")
)

// ValidateNilSyncContribution validates the following fields are not nil:
// -the contribution and proof itself
// -the message within contribution and proof
// -the contribution within contribution and proof
// -the aggregation bits within contribution
func ValidateNilSyncContribution(s *ethpb.SignedContributionAndProof) error {
	if s == nil {
		return errors.New("signed message can't be nil")
	}
	if s.Message == nil {
		return errors.New("signed contribution's message can't be nil")
	}
	if s.Message.Contribution == nil {
		return errors.New("inner contribution can't be nil")
	}
	if s.Message.Contribution.AggregationBits == nil {
		return errors.New("contribution's bitfield can't be nil")
	}
	return nil
}

// NextSyncCommittee returns the next sync committee for a given state.
//
// Spec code:
// def get_next_sync_committee(state: BeaconState) -> SyncCommittee:
//
//	"""
//	Return the next sync committee, with possible pubkey duplicates.
//	"""
//	indices = get_next_sync_committee_indices(state)
//	pubkeys = [state.validators[index].pubkey for index in indices]
//	aggregate_pubkey = bls.AggregatePKs(pubkeys)
//	return SyncCommittee(pubkeys=pubkeys, aggregate_pubkey=aggregate_pubkey)
func NextSyncCommittee(ctx context.Context, s state.BeaconState) (*ethpb.SyncCommittee, error) {
	indices, err := NextSyncCommitteeIndices(ctx, s)
	if err != nil {
		return nil, err
	}
	pubkeys := make([][]byte, len(indices))
	for i, index := range indices {
		p := s.PubkeyAtIndex(index)
		pubkeys[i] = p[:]
	}
	aggregated, err := bls.AggregatePublicKeys(pubkeys)
	if err != nil {
		return nil, err
	}
	return &ethpb.SyncCommittee{
		Pubkeys:         pubkeys,
		AggregatePubkey: aggregated.Marshal(),
	}, nil
}

// NextSyncCommitteeIndices returns the next sync committee indices for a given state.
//
// Spec code:
// def get_next_sync_committee_indices(state: BeaconState) -> Sequence[ValidatorIndex]:
//
//	"""
//	Return the sync committee indices, with possible duplicates, for the next sync committee.
//	"""
//	epoch = Epoch(get_current_epoch(state) + 1)
//
//	MAX_RANDOM_BYTE = 2**8 - 1
//	active_validator_indices = get_active_validator_indices(state, epoch)
//	active_validator_count = uint64(len(active_validator_indices))
//	seed = get_seed(state, epoch, DOMAIN_SYNC_COMMITTEE)
//	i = 0
//	sync_committee_indices: List[ValidatorIndex] = []
//	while len(sync_committee_indices) < SYNC_COMMITTEE_SIZE:
//	    shuffled_index = compute_shuffled_index(uint64(i % active_validator_count), active_validator_count, seed)
//	    candidate_index = active_validator_indices[shuffled_index]
//	    random_byte = hash(seed + uint_to_bytes(uint64(i // 32)))[i % 32]
//	    effective_balance = state.validators[candidate_index].effective_balance
//	    if effective_balance * MAX_RANDOM_BYTE >= MAX_EFFECTIVE_BALANCE * random_byte:
//	        sync_committee_indices.append(candidate_index)
//	    i += 1
//	return sync_committee_indices
func NextSyncCommitteeIndices(ctx context.Context, s state.BeaconState) ([]primitives.ValidatorIndex, error) {
	epoch := coreTime.NextEpoch(s)
	indices, err := helpers.ActiveValidatorIndices(ctx, s, epoch)
	if err != nil {
		return nil, err
	}
	seed, err := helpers.Seed(s, epoch, params.BeaconConfig().DomainSyncCommittee)
	if err != nil {
		return nil, err
	}
	count := uint64(len(indices))
	cfg := params.BeaconConfig()
	syncCommitteeSize := cfg.SyncCommitteeSize
	cIndices := make([]primitives.ValidatorIndex, 0, syncCommitteeSize)
	hashFunc := hash.CustomSHA256Hasher()

	for i := primitives.ValidatorIndex(0); uint64(len(cIndices)) < params.BeaconConfig().SyncCommitteeSize; i++ {
		if ctx.Err() != nil {
			return nil, ctx.Err()
		}

		sIndex, err := helpers.ComputeShuffledIndex(i.Mod(count), count, seed, true)
		if err != nil {
			return nil, err
		}

		b := append(seed[:], bytesutil.Bytes8(uint64(i.Div(32)))...)
		randomByte := hashFunc(b)[i%32]
		cIndex := indices[sIndex]
		v, err := s.ValidatorAtIndexReadOnly(cIndex)
		if err != nil {
			return nil, err
		}

		effectiveBal := v.EffectiveBalance()
		if effectiveBal*maxRandomByte >= cfg.MaxEffectiveBalance*uint64(randomByte) {
			cIndices = append(cIndices, cIndex)
		}
	}

	return cIndices, nil
}

// SyncSubCommitteePubkeys returns the pubkeys participating in a sync subcommittee.
//
// def get_sync_subcommittee_pubkeys(state: BeaconState, subcommittee_index: uint64) -> Sequence[BLSPubkey]:
//
//	# Committees assigned to `slot` sign for `slot - 1`
//	# This creates the exceptional logic below when transitioning between sync committee periods
//	next_slot_epoch = compute_epoch_at_slot(Slot(state.slot + 1))
//	if compute_sync_committee_period(get_current_epoch(state)) == compute_sync_committee_period(next_slot_epoch):
//	    sync_committee = state.current_sync_committee
//	else:
//	    sync_committee = state.next_sync_committee
//
//	# Return pubkeys for the subcommittee index
//	sync_subcommittee_size = SYNC_COMMITTEE_SIZE // SYNC_COMMITTEE_SUBNET_COUNT
//	i = subcommittee_index * sync_subcommittee_size
//	return sync_committee.pubkeys[i:i + sync_subcommittee_size]
func SyncSubCommitteePubkeys(syncCommittee *ethpb.SyncCommittee, subComIdx primitives.CommitteeIndex) ([][]byte, error) {
	cfg := params.BeaconConfig()
	subCommSize := cfg.SyncCommitteeSize / cfg.SyncCommitteeSubnetCount
	i := uint64(subComIdx) * subCommSize
	endOfSubCom := i + subCommSize
	pubkeyLen := uint64(len(syncCommittee.Pubkeys))
	if endOfSubCom > pubkeyLen {
		return nil, errors.Errorf("end index is larger than array length: %d > %d", endOfSubCom, pubkeyLen)
	}
	return syncCommittee.Pubkeys[i:endOfSubCom], nil
}

// IsSyncCommitteeAggregator checks whether the provided signature is for a valid
// aggregator.
//
// def is_sync_committee_aggregator(signature: BLSSignature) -> bool:
//
//	modulo = max(1, SYNC_COMMITTEE_SIZE // SYNC_COMMITTEE_SUBNET_COUNT // TARGET_AGGREGATORS_PER_SYNC_SUBCOMMITTEE)
//	return bytes_to_uint64(hash(signature)[0:8]) % modulo == 0
func IsSyncCommitteeAggregator(sig []byte) (bool, error) {
	if len(sig) != fieldparams.BLSSignatureLength {
		return false, errors.New("incorrect sig length")
	}

	cfg := params.BeaconConfig()
	modulo := math.Max(1, cfg.SyncCommitteeSize/cfg.SyncCommitteeSubnetCount/cfg.TargetAggregatorsPerSyncSubcommittee)
	hashedSig := hash.Hash(sig)
	return bytesutil.FromBytes8(hashedSig[:8])%modulo == 0, nil
}

// ValidateSyncMessageTime validates sync message to ensure that the provided slot is valid.
// Spec: [IGNORE] The message's slot is for the current slot (with a MAXIMUM_GOSSIP_CLOCK_DISPARITY allowance), i.e. sync_committee_message.slot == current_slot
func ValidateSyncMessageTime(slot primitives.Slot, genesisTime time.Time, clockDisparity time.Duration) error {
	if err := slots.ValidateClock(slot, uint64(genesisTime.Unix())); err != nil {
		return err
	}
	messageTime, err := slots.ToTime(uint64(genesisTime.Unix()), slot)
	if err != nil {
		return err
	}
	currentSlot := slots.Since(genesisTime)
	slotStartTime, err := slots.ToTime(uint64(genesisTime.Unix()), currentSlot)
	if err != nil {
		return err
	}

	lowestSlotBound := slotStartTime.Add(-clockDisparity)
	currentLowerBound := time.Now().Add(-clockDisparity)
	// In the event the Slot's start time, is before the
	// current allowable bound, we set the slot's start
	// time as the bound.
	if slotStartTime.Before(currentLowerBound) {
		lowestSlotBound = slotStartTime
	}

	lowerBound := lowestSlotBound
	upperBound := time.Now().Add(clockDisparity)
	// Verify sync message slot is within the time range.
	if messageTime.Before(lowerBound) || messageTime.After(upperBound) {
		syncErr := fmt.Errorf(
			"sync message time %v (message slot %d) not within allowable range of %v to %v (current slot %d)",
			messageTime,
			slot,
			lowerBound,
			upperBound,
			currentSlot,
		)
		// Wrap error message if sync message is too late.
		if messageTime.Before(lowerBound) {
			syncErr = goErrors.Join(ErrTooLate, syncErr)
		}
		return syncErr
	}
	return nil
}

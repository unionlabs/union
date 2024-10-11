package slasher

import (
	"context"
	"fmt"
	"math"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	slashertypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/slasher/types"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// A struct encapsulating input arguments to
// functions used for attester slashing detection and
// loading, saving, and updating min/max span chunks.
type chunkUpdateArgs struct {
	kind                slashertypes.ChunkKind
	chunkIndex          uint64
	validatorChunkIndex uint64
	currentEpoch        primitives.Epoch
}

// Chunker defines a struct which represents a slice containing a chunk for K different validator's
// min spans used for surround vote detection in slasher. The interface defines methods used to check
// if an attestation is slashable for a validator index based on the contents of
// the chunk as well as the ability to update the data in the chunk with incoming information.
type Chunker interface {
	NeutralElement() uint16
	Chunk() []uint16
	CheckSlashable(
		ctx context.Context,
		slasherDB db.SlasherDatabase,
		validatorIdx primitives.ValidatorIndex,
		attestation *slashertypes.IndexedAttestationWrapper,
	) (*ethpb.AttesterSlashing, error)
	Update(
		args *chunkUpdateArgs,
		validatorIndex primitives.ValidatorIndex,
		startEpoch,
		newTargetEpoch primitives.Epoch,
	) (keepGoing bool, err error)
	StartEpoch(sourceEpoch, currentEpoch primitives.Epoch) (epoch primitives.Epoch, exists bool)
	NextChunkStartEpoch(startEpoch primitives.Epoch) primitives.Epoch
}

// MinSpanChunksSlice represents a slice containing a chunk for K different validator's min spans.
//
// For a given epoch, e, and attestations a validator index has produced, atts,
// min_spans[e] is defined as min((att.target.epoch - e) for att in attestations)
// where att.source.epoch > e. That is, it is the minimum distance between the
// specified epoch and all attestation target epochs a validator has created
// where att.source.epoch > e.
//
// nolint:dupword
//
// Under ideal network conditions, where every target epoch immediately follows its source,
// min spans for a validator will look as follows:
//
//	min_spans = [2, 2, 2, ..., 2]
//
// Next, we can chunk this list of min spans into chunks of length C. For C = 2, for example:
//
//	                     chunk0  chunk1       chunkN
//	                      {  }   {   }         {  }
//	chunked_min_spans = [[2, 2], [2, 2], ..., [2, 2]]
//
// Finally, we can store each chunk index for K validators into a single flat slice. For K = 3:
//
//	                                  val0    val1    val2
//	                                  {  }    {  }    {  }
//	chunk_0_for_validators_0_to_2 = [[2, 2], [2, 2], [2, 2]]
//
//	                                  val0    val1    val2
//	                                  {  }    {  }    {  }
//	chunk_1_for_validators_0_to_2 = [[2, 2], [2, 2], [2, 2]]
//
//	                         ...
//
//	                                  val0    val1    val2
//	                                  {  }    {  }    {  }
//	chunk_N_for_validators_0_to_2 = [[2, 2], [2, 2], [2, 2]]
//
// MinSpanChunksSlice represents the data structure above for a single chunk index.
type MinSpanChunksSlice struct {
	params *Parameters
	data   []uint16
}

// MaxSpanChunksSlice represents the same data structure as MinSpanChunksSlice however
// keeps track of validator max spans for slashing detection instead.
type MaxSpanChunksSlice struct {
	params *Parameters
	data   []uint16
}

// EmptyMinSpanChunksSlice initializes a min span chunk of length C*K for
// C = chunkSize and K = validatorChunkSize filled with neutral elements.
// For min spans, the neutral element is `undefined`, represented by MaxUint16.
func EmptyMinSpanChunksSlice(params *Parameters) *MinSpanChunksSlice {
	m := &MinSpanChunksSlice{
		params: params,
	}
	data := make([]uint16, params.chunkSize*params.validatorChunkSize)
	for i := 0; i < len(data); i++ {
		data[i] = m.NeutralElement()
	}
	m.data = data
	return m
}

// EmptyMaxSpanChunksSlice initializes a max span chunk of length C*K for
// C = chunkSize and K = validatorChunkSize filled with neutral elements.
// For max spans, the neutral element is 0.
func EmptyMaxSpanChunksSlice(params *Parameters) *MaxSpanChunksSlice {
	m := &MaxSpanChunksSlice{
		params: params,
	}
	data := make([]uint16, params.chunkSize*params.validatorChunkSize)
	for i := 0; i < len(data); i++ {
		data[i] = m.NeutralElement()
	}
	m.data = data
	return m
}

// MinChunkSpansSliceFrom initializes a min span chunks slice from a slice of uint16 values.
// Returns an error if the slice is not of length C*K for C = chunkSize and K = validatorChunkSize.
func MinChunkSpansSliceFrom(params *Parameters, chunk []uint16) (*MinSpanChunksSlice, error) {
	requiredLen := params.chunkSize * params.validatorChunkSize
	if uint64(len(chunk)) != requiredLen {
		return nil, fmt.Errorf("chunk has wrong length, %d, expected %d", len(chunk), requiredLen)
	}
	return &MinSpanChunksSlice{
		params: params,
		data:   chunk,
	}, nil
}

// MaxChunkSpansSliceFrom initializes a max span chunks slice from a slice of uint16 values.
// Returns an error if the slice is not of length C*K for C = chunkSize and K = validatorChunkSize.
func MaxChunkSpansSliceFrom(params *Parameters, chunk []uint16) (*MaxSpanChunksSlice, error) {
	requiredLen := params.chunkSize * params.validatorChunkSize
	if uint64(len(chunk)) != requiredLen {
		return nil, fmt.Errorf("chunk has wrong length, %d, expected %d", len(chunk), requiredLen)
	}
	return &MaxSpanChunksSlice{
		params: params,
		data:   chunk,
	}, nil
}

// NeutralElement for a min span chunks slice is undefined, in this case
// using MaxUint16 as a sane value given it is impossible we reach it.
func (_ *MinSpanChunksSlice) NeutralElement() uint16 {
	return math.MaxUint16
}

// NeutralElement for a max span chunks slice is 0.
func (_ *MaxSpanChunksSlice) NeutralElement() uint16 {
	return 0
}

// Chunk returns the underlying slice of uint16's for the min chunks slice.
func (m *MinSpanChunksSlice) Chunk() []uint16 {
	return m.data
}

// Chunk returns the underlying slice of uint16's for the max chunks slice.
func (m *MaxSpanChunksSlice) Chunk() []uint16 {
	return m.data
}

// CheckSlashable takes in a validator index and an incoming attestation
// and checks if the validator is slashable depending on the data
// within the min span chunks slice. Recall that for an incoming attestation, B, and an
// existing attestation, A:
//
//	B surrounds A if and only if B.target > min_spans[B.source]
//
// That is, this condition is sufficient to check if an incoming attestation
// is surrounding a previous one. We also check if we indeed have an existing
// attestation record in the database if the condition holds true in order
// to be confident of a slashable offense.
func (m *MinSpanChunksSlice) CheckSlashable(
	ctx context.Context,
	slasherDB db.SlasherDatabase,
	validatorIdx primitives.ValidatorIndex,
	attestation *slashertypes.IndexedAttestationWrapper,
) (*ethpb.AttesterSlashing, error) {
	sourceEpoch := attestation.IndexedAttestation.Data.Source.Epoch
	targetEpoch := attestation.IndexedAttestation.Data.Target.Epoch
	minTarget, err := chunkDataAtEpoch(m.params, m.data, validatorIdx, sourceEpoch)
	if err != nil {
		return nil, errors.Wrapf(
			err, "could not get min target for validator %d at epoch %d", validatorIdx, sourceEpoch,
		)
	}
	if targetEpoch > minTarget {
		existingAttRecord, err := slasherDB.AttestationRecordForValidator(
			ctx, validatorIdx, minTarget,
		)
		if err != nil {
			return nil, errors.Wrapf(
				err, "could not get existing attestation record at target %d", minTarget,
			)
		}
		if existingAttRecord != nil {
			if sourceEpoch < existingAttRecord.IndexedAttestation.Data.Source.Epoch {
				surroundingVotesTotal.Inc()
				return &ethpb.AttesterSlashing{
					Attestation_1: attestation.IndexedAttestation,
					Attestation_2: existingAttRecord.IndexedAttestation,
				}, nil
			}
		}
	}
	return nil, nil
}

// CheckSlashable takes in a validator index and an incoming attestation
// and checks if the validator is slashable depending on the data
// within the max span chunks slice. Recall that for an incoming attestation, B, and an
// existing attestation, A:
//
//	B surrounds A if and only if B.target < max_spans[B.source]
//
// That is, this condition is sufficient to check if an incoming attestation
// is surrounded by a previous one. We also check if we indeed have an existing
// attestation record in the database if the condition holds true in order
// to be confident of a slashable offense.
func (m *MaxSpanChunksSlice) CheckSlashable(
	ctx context.Context,
	slasherDB db.SlasherDatabase,
	validatorIdx primitives.ValidatorIndex,
	attestation *slashertypes.IndexedAttestationWrapper,
) (*ethpb.AttesterSlashing, error) {
	sourceEpoch := attestation.IndexedAttestation.Data.Source.Epoch
	targetEpoch := attestation.IndexedAttestation.Data.Target.Epoch
	maxTarget, err := chunkDataAtEpoch(m.params, m.data, validatorIdx, sourceEpoch)
	if err != nil {
		return nil, errors.Wrapf(
			err, "could not get max target for validator %d at epoch %d", validatorIdx, sourceEpoch,
		)
	}
	if targetEpoch < maxTarget {
		existingAttRecord, err := slasherDB.AttestationRecordForValidator(
			ctx, validatorIdx, maxTarget,
		)
		if err != nil {
			return nil, errors.Wrapf(
				err, "could not get existing attestation record at target %d", maxTarget,
			)
		}
		if existingAttRecord != nil {
			if existingAttRecord.IndexedAttestation.Data.Source.Epoch < sourceEpoch {
				surroundedVotesTotal.Inc()
				return &ethpb.AttesterSlashing{
					Attestation_1: existingAttRecord.IndexedAttestation,
					Attestation_2: attestation.IndexedAttestation,
				}, nil
			}
		}
	}
	return nil, nil
}

// Update a min span chunk for a validator index starting at the current epoch, e_c, then updating
// down to e_c - H where H is the historyLength we keep for each span. This historyLength
// corresponds to the weak subjectivity period of Ethereum consensus.
// This means our updates are done in a sliding window manner. For example, if the current epoch
// is 20 and the historyLength is 12, then we will update every value for the validator's min span
// from epoch 20 down to epoch 9.
//
// Recall that for an epoch, e, min((att.target - e) for att in attestations where att.source > e)
// That is, it is the minimum distance between the specified epoch and all attestation
// target epochs a validator has created where att.source.epoch > e.
//
// Recall that a MinSpanChunksSlice struct represents a single slice for a chunk index
// from the collection below:
//
//	                                  val0    val1    val2
//	                                  {  }    {  }    {  }
//	chunk_0_for_validators_0_to_2 = [[2, 2], [2, 2], [2, 2]]
//
//	                                  val0    val1    val2
//	                                  {  }    {  }    {  }
//	chunk_1_for_validators_0_to_2 = [[2, 2], [2, 2], [2, 2]]
//
//	                         ...
//
//	                                  val0    val1    val2
//	                                  {  }    {  }    {  }
//	chunk_N_for_validators_0_to_2 = [[2, 2], [2, 2], [2, 2]]
//
// Let's take a look at how this update will look for a real set of min span chunk:
// For the purposes of a simple example, let's set H = 2, meaning a min span
// will hold 2 epochs worth of attesting history. Then we set C = 2 meaning we will
// chunk the min span into arrays each of length 2.
//
// So assume we get an epoch 4 and validator 0, then, we need to update every epoch in the span from
// 4 down to 3. First, we find out which chunk epoch 4 falls into, which is calculated as:
// chunk_idx = (epoch % H) / C = (4 % 2) / 2 = 0
//
//	                                  val0    val1    val2
//	                                  {  }    {  }    {  }
//	chunk_0_for_validators_0_to_3 = [[2, 2], [2, 2], [2, 2]]
//	                                  |
//	                                  |-> epoch 4 for validator 0
//
// Next up, we proceed with the update process for validator index 0, starting at epoch 4
// all the way down to epoch 2. We will need to go down the array as far as we can get. If the
// lowest epoch we need to update is < the lowest epoch of a chunk, we need to proceed to
// a different chunk index.
//
// Once we finish updating a chunk, we need to move on to the next chunk. This function
// returns a boolean named keepGoing which allows the caller to determine if we should
// continue and update another chunk index. We stop whenever we reach the min epoch we need
// to update. In our example, we stop at 2, which is still part of chunk 0, so no need
// to jump to another min span chunks slice to perform updates.
func (m *MinSpanChunksSlice) Update(
	args *chunkUpdateArgs,
	validatorIndex primitives.ValidatorIndex,
	startEpoch,
	newTargetEpoch primitives.Epoch,
) (keepGoing bool, err error) {
	// The lowest epoch we need to update.
	minEpoch := primitives.Epoch(0)
	if args.currentEpoch > (m.params.historyLength - 1) {
		minEpoch = args.currentEpoch - (m.params.historyLength - 1)
	}
	epochInChunk := startEpoch
	// We go down the chunk for the validator, updating every value starting at start_epoch down to min_epoch.
	// As long as the epoch, e, in the same chunk index and e >= min_epoch, we proceed with
	// a for loop.
	for m.params.chunkIndex(epochInChunk) == args.chunkIndex && epochInChunk >= minEpoch {
		var chunkTarget primitives.Epoch
		chunkTarget, err = chunkDataAtEpoch(m.params, m.data, validatorIndex, epochInChunk)
		if err != nil {
			err = errors.Wrapf(err, "could not get chunk data at epoch %d", epochInChunk)
			return
		}
		// If the newly incoming value is < the existing value, we update
		// the data in the min span to meet with its definition.
		if newTargetEpoch < chunkTarget {
			if err = setChunkDataAtEpoch(m.params, m.data, validatorIndex, epochInChunk, newTargetEpoch); err != nil {
				err = errors.Wrapf(err, "could not set chunk data at epoch %d", epochInChunk)
				return
			}
		} else {
			// We can stop because spans are guaranteed to be minimums and
			// if we did not meet the minimum condition, there is nothing to update.
			return
		}
		if epochInChunk > 0 {
			epochInChunk -= 1
		}
	}
	// We should keep going and update the previous chunk if we are yet to reach
	// the minimum epoch required for the update procedure.
	keepGoing = epochInChunk >= minEpoch
	return
}

// Update a max span chunk for a validator index starting at a given start epoch, e_c, then updating
// up to the current epoch according to the definition of max spans. If we need to continue updating
// a next chunk, this function returns a boolean letting the caller know it should keep going. To understand
// more about how update exactly works, refer to the detailed documentation for the Update function for
// MinSpanChunksSlice.
func (m *MaxSpanChunksSlice) Update(
	args *chunkUpdateArgs,
	validatorIndex primitives.ValidatorIndex,
	startEpoch,
	newTargetEpoch primitives.Epoch,
) (keepGoing bool, err error) {
	epochInChunk := startEpoch
	// We go down the chunk for the validator, updating every value starting at start_epoch up to
	// and including the current epoch. As long as the epoch, e, is in the same chunk index and e <= currentEpoch,
	// we proceed with a for loop.
	for m.params.chunkIndex(epochInChunk) == args.chunkIndex && epochInChunk <= args.currentEpoch {
		var chunkTarget primitives.Epoch
		chunkTarget, err = chunkDataAtEpoch(m.params, m.data, validatorIndex, epochInChunk)
		if err != nil {
			err = errors.Wrapf(err, "could not get chunk data at epoch %d", epochInChunk)
			return
		}
		// If the newly incoming value is > the existing value, we update
		// the data in the max span to meet with its definition.
		if newTargetEpoch > chunkTarget {
			if err = setChunkDataAtEpoch(m.params, m.data, validatorIndex, epochInChunk, newTargetEpoch); err != nil {
				err = errors.Wrapf(err, "could not set chunk data at epoch %d", epochInChunk)
				return
			}
		} else {
			// We can stop because spans are guaranteed to be maxima and
			// if we did not meet the condition, there is nothing to update.
			return
		}
		epochInChunk++
	}
	// If the epoch to update now lies beyond the current chunk, then
	// continue to the next chunk to update it.
	keepGoing = epochInChunk <= args.currentEpoch
	return
}

// StartEpoch given a source epoch and current epoch, determines the start epoch of
// a min span chunk for use in chunk updates. To compute this value, we look at the difference between
// H = historyLength and the current epoch. Then, we check if the source epoch > difference. If so,
// then the start epoch is source epoch - 1. Otherwise, we return to the caller a boolean signifying
// the input arguments are invalid for the chunk and the start epoch does not exist.
func (m *MinSpanChunksSlice) StartEpoch(
	sourceEpoch, currentEpoch primitives.Epoch,
) (epoch primitives.Epoch, exists bool) {
	// Given min span chunks are used for detecting surrounding votes, we have no need
	// for a start epoch of the chunk if the source epoch is 0 in the input arguments.
	// To further clarify, min span chunks are updated in reverse order [a, b, c, d] where
	// if the start epoch is d, then we go down the chunk updating everything from d, c, b, to
	// a. If the source epoch is 0, this would correspond to a, which means there is nothing
	// more to update.
	if sourceEpoch == 0 {
		return
	}
	var difference primitives.Epoch
	if currentEpoch > m.params.historyLength {
		difference = currentEpoch - m.params.historyLength
	}
	if sourceEpoch <= difference {
		return
	}
	epoch = sourceEpoch.Sub(1)
	exists = true
	return
}

// StartEpoch given a source epoch and current epoch, determines the start epoch of
// a max span chunk for use in chunk updates. The source epoch cannot be >= the current epoch.
func (_ *MaxSpanChunksSlice) StartEpoch(
	sourceEpoch, currentEpoch primitives.Epoch,
) (epoch primitives.Epoch, exists bool) {
	if sourceEpoch >= currentEpoch {
		return
	}
	// Given max spans is a list of max targets for source epochs, the precondition is that
	// every attestation's source epoch must be < than its target epoch. So the start epoch
	// for updates is given as source epoch + 1.
	epoch = sourceEpoch.Add(1)
	exists = true
	return
}

// NextChunkStartEpoch given an epoch, determines the start epoch of the next chunk. For min
// span chunks, this will be the last epoch of chunk index = (current chunk - 1). For example:
//
//	                     chunk0     chunk1     chunk2
//	                       |          |          |
//	max_spans_val_i = [[-, -, -], [-, -, -], [-, -, -]]
//
// If C = chunkSize is 3 epochs per chunk, and we input start epoch of chunk 1 which is 3 then the next start
// epoch is the last epoch of chunk 0, which is epoch 2. This is computed as:
//
//	last_epoch(chunkIndex(startEpoch)-1)
//	last_epoch(chunkIndex(3) - 1)
//	last_epoch(1 - 1)
//	last_epoch(0)
//	2
func (m *MinSpanChunksSlice) NextChunkStartEpoch(startEpoch primitives.Epoch) primitives.Epoch {
	prevChunkIdx := m.params.chunkIndex(startEpoch)
	if prevChunkIdx > 0 {
		prevChunkIdx--
	}
	return m.params.lastEpoch(prevChunkIdx)
}

// NextChunkStartEpoch given an epoch, determines the start epoch of the next chunk. For max
// span chunks, this will be the start epoch of chunk index = (current chunk + 1). For example:
//
//	                     chunk0     chunk1     chunk2
//	                       |          |          |
//	max_spans_val_i = [[-, -, -], [-, -, -], [-, -, -]]
//
// If C = chunkSize is 3 epochs per chunk, and we input start epoch of chunk 1 which is 3. The next start
// epoch is the start epoch of chunk 2, which is epoch 4. This is computed as:
//
//	first_epoch(chunkIndex(startEpoch)+1)
//	first_epoch(chunkIndex(3)+1)
//	first_epoch(1 + 1)
//	first_epoch(2)
//	4
func (m *MaxSpanChunksSlice) NextChunkStartEpoch(startEpoch primitives.Epoch) primitives.Epoch {
	return m.params.firstEpoch(m.params.chunkIndex(startEpoch) + 1)
}

// Given a validator index and epoch, retrieves the target epoch at its specific
// index for the validator index and epoch in a min/max span chunk.
func chunkDataAtEpoch(
	params *Parameters, chunk []uint16, validatorIdx primitives.ValidatorIndex, epoch primitives.Epoch,
) (primitives.Epoch, error) {
	requiredLen := params.chunkSize * params.validatorChunkSize
	if uint64(len(chunk)) != requiredLen {
		return 0, fmt.Errorf("chunk has wrong length, %d, expected %d", len(chunk), requiredLen)
	}
	cellIdx := params.cellIndex(validatorIdx, epoch)
	if cellIdx >= uint64(len(chunk)) {
		return 0, fmt.Errorf("cell index %d out of bounds (len(chunk) = %d)", cellIdx, len(chunk))
	}
	distance := chunk[cellIdx]
	return epoch.Add(uint64(distance)), nil
}

// Updates the value at a specific index in a chunk for a validator index + epoch
// pair given a target epoch. Recall that for min spans, each element in a chunk
// is the minimum distance between the a given epoch, e, and all attestation target epochs
// a validator has created where att.source.epoch > e.
func setChunkDataAtEpoch(
	params *Parameters,
	chunk []uint16,
	validatorIdx primitives.ValidatorIndex,
	epochInChunk,
	targetEpoch primitives.Epoch,
) error {
	distance, err := epochDistance(targetEpoch, epochInChunk)
	if err != nil {
		return err
	}
	return setChunkRawDistance(params, chunk, validatorIdx, epochInChunk, distance)
}

// Updates the value at a specific index in a chunk for a validator index and epoch
// to a specified, raw distance value.
func setChunkRawDistance(
	params *Parameters,
	chunk []uint16,
	validatorIdx primitives.ValidatorIndex,
	epochInChunk primitives.Epoch,
	distance uint16,
) error {
	cellIdx := params.cellIndex(validatorIdx, epochInChunk)
	if cellIdx >= uint64(len(chunk)) {
		return fmt.Errorf("cell index %d out of bounds (len(chunk) = %d)", cellIdx, len(chunk))
	}
	chunk[cellIdx] = distance
	return nil
}

// Computes a distance between two epochs. Given the result stored in
// min/max spans is at maximum WEAK_SUBJECTIVITY_PERIOD, we are guaranteed the
// distance can be represented as a uint16 safely.
func epochDistance(epoch, baseEpoch primitives.Epoch) (uint16, error) {
	if baseEpoch > epoch {
		return 0, fmt.Errorf("base epoch %d cannot be less than epoch %d", baseEpoch, epoch)
	}
	return uint16(epoch.Sub(uint64(baseEpoch))), nil
}

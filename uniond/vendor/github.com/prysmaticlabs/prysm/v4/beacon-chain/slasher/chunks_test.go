package slasher

import (
	"context"
	"math"
	"testing"

	dbtest "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	slashertypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/slasher/types"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

var (
	_ = Chunker(&MinSpanChunksSlice{})
	_ = Chunker(&MaxSpanChunksSlice{})
)

func TestMinSpanChunksSlice_Chunk(t *testing.T) {
	chunk := EmptyMinSpanChunksSlice(&Parameters{
		chunkSize:          2,
		validatorChunkSize: 2,
	})
	wanted := []uint16{math.MaxUint16, math.MaxUint16, math.MaxUint16, math.MaxUint16}
	require.DeepEqual(t, wanted, chunk.Chunk())
}

func TestMaxSpanChunksSlice_Chunk(t *testing.T) {
	chunk := EmptyMaxSpanChunksSlice(&Parameters{
		chunkSize:          2,
		validatorChunkSize: 2,
	})
	wanted := []uint16{0, 0, 0, 0}
	require.DeepEqual(t, wanted, chunk.Chunk())
}

func TestMinSpanChunksSlice_NeutralElement(t *testing.T) {
	chunk := EmptyMinSpanChunksSlice(&Parameters{})
	require.Equal(t, uint16(math.MaxUint16), chunk.NeutralElement())
}

func TestMaxSpanChunksSlice_NeutralElement(t *testing.T) {
	chunk := EmptyMaxSpanChunksSlice(&Parameters{})
	require.Equal(t, uint16(0), chunk.NeutralElement())
}

func TestMinSpanChunksSlice_MinChunkSpanFrom(t *testing.T) {
	params := &Parameters{
		chunkSize:          3,
		validatorChunkSize: 2,
	}
	_, err := MinChunkSpansSliceFrom(params, []uint16{})
	require.ErrorContains(t, "chunk has wrong length", err)

	data := []uint16{2, 2, 2, 2, 2, 2}
	chunk, err := MinChunkSpansSliceFrom(&Parameters{
		chunkSize:          3,
		validatorChunkSize: 2,
	}, data)
	require.NoError(t, err)
	require.DeepEqual(t, data, chunk.Chunk())
}

func TestMaxSpanChunksSlice_MaxChunkSpanFrom(t *testing.T) {
	params := &Parameters{
		chunkSize:          3,
		validatorChunkSize: 2,
	}
	_, err := MaxChunkSpansSliceFrom(params, []uint16{})
	require.ErrorContains(t, "chunk has wrong length", err)

	data := []uint16{2, 2, 2, 2, 2, 2}
	chunk, err := MaxChunkSpansSliceFrom(&Parameters{
		chunkSize:          3,
		validatorChunkSize: 2,
	}, data)
	require.NoError(t, err)
	require.DeepEqual(t, data, chunk.Chunk())
}

func TestMinSpanChunksSlice_CheckSlashable(t *testing.T) {
	ctx := context.Background()
	slasherDB := dbtest.SetupSlasherDB(t)
	params := &Parameters{
		chunkSize:          3,
		validatorChunkSize: 2,
		historyLength:      3,
	}
	validatorIdx := primitives.ValidatorIndex(1)
	source := primitives.Epoch(1)
	target := primitives.Epoch(2)
	att := createAttestationWrapper(t, source, target, nil, nil)

	// A faulty chunk should lead to error.
	chunk := &MinSpanChunksSlice{
		params: params,
		data:   []uint16{},
	}
	_, err := chunk.CheckSlashable(ctx, nil, validatorIdx, att)
	require.ErrorContains(t, "could not get min target for validator", err)

	// We initialize a proper slice with 2 chunks with chunk size 3, 2 validators, and
	// a history length of 3 representing a perfect attesting history.
	//
	//     val0     val1
	//   {     }  {     }
	//  [2, 2, 2, 2, 2, 2]
	data := []uint16{2, 2, 2, 2, 2, 2}
	chunk, err = MinChunkSpansSliceFrom(params, data)
	require.NoError(t, err)

	// An attestation with source 1 and target 2 should not be slashable
	// based on our min chunk for either validator.
	slashing, err := chunk.CheckSlashable(ctx, slasherDB, validatorIdx, att)
	require.NoError(t, err)
	require.Equal(t, (*ethpb.AttesterSlashing)(nil), slashing)

	slashing, err = chunk.CheckSlashable(ctx, slasherDB, validatorIdx.Sub(1), att)
	require.NoError(t, err)
	require.Equal(t, (*ethpb.AttesterSlashing)(nil), slashing)

	// Next up we initialize an empty chunks slice and mark an attestation
	// with (source 1, target 2) as attested.
	chunk = EmptyMinSpanChunksSlice(params)
	source = primitives.Epoch(1)
	target = primitives.Epoch(2)
	att = createAttestationWrapper(t, source, target, nil, nil)
	chunkIdx := uint64(0)
	startEpoch := target
	currentEpoch := target
	args := &chunkUpdateArgs{
		chunkIndex:   chunkIdx,
		currentEpoch: currentEpoch,
	}
	_, err = chunk.Update(args, validatorIdx, startEpoch, target)
	require.NoError(t, err)

	// Next up, we create a surrounding vote, but it should NOT be slashable
	// because we DO NOT have an existing attestation record in our database at the min target epoch.
	source = primitives.Epoch(0)
	target = primitives.Epoch(3)
	surroundingVote := createAttestationWrapper(t, source, target, nil, nil)

	slashing, err = chunk.CheckSlashable(ctx, slasherDB, validatorIdx, surroundingVote)
	require.NoError(t, err)
	require.Equal(t, (*ethpb.AttesterSlashing)(nil), slashing)

	// Next up, we save the old attestation record, then check if the
	// surrounding vote is indeed slashable.
	attData := att.IndexedAttestation.Data
	attRecord := createAttestationWrapper(t, attData.Source.Epoch, attData.Target.Epoch, []uint64{uint64(validatorIdx)}, []byte{1})
	err = slasherDB.SaveAttestationRecordsForValidators(
		ctx,
		[]*slashertypes.IndexedAttestationWrapper{attRecord},
	)
	require.NoError(t, err)

	slashing, err = chunk.CheckSlashable(ctx, slasherDB, validatorIdx, surroundingVote)
	require.NoError(t, err)
	require.NotEqual(t, (*ethpb.AttesterSlashing)(nil), slashing)
}

func TestMaxSpanChunksSlice_CheckSlashable(t *testing.T) {
	ctx := context.Background()
	slasherDB := dbtest.SetupSlasherDB(t)
	params := &Parameters{
		chunkSize:          4,
		validatorChunkSize: 2,
		historyLength:      4,
	}
	validatorIdx := primitives.ValidatorIndex(1)
	source := primitives.Epoch(1)
	target := primitives.Epoch(2)
	att := createAttestationWrapper(t, source, target, nil, nil)

	// A faulty chunk should lead to error.
	chunk := &MaxSpanChunksSlice{
		params: params,
		data:   []uint16{},
	}
	_, err := chunk.CheckSlashable(ctx, nil, validatorIdx, att)
	require.ErrorContains(t, "could not get max target for validator", err)

	// We initialize a proper slice with 2 chunks with chunk size 4, 2 validators, and
	// a history length of 4 representing a perfect attesting history.
	//
	//      val0        val1
	//   {        }  {        }
	//  [0, 0, 0, 0, 0, 0, 0, 0]
	data := []uint16{0, 0, 0, 0, 0, 0, 0, 0}
	chunk, err = MaxChunkSpansSliceFrom(params, data)
	require.NoError(t, err)

	// An attestation with source 1 and target 2 should not be slashable
	// based on our max chunk for either validator.
	slashing, err := chunk.CheckSlashable(ctx, slasherDB, validatorIdx, att)
	require.NoError(t, err)
	require.Equal(t, (*ethpb.AttesterSlashing)(nil), slashing)

	slashing, err = chunk.CheckSlashable(ctx, slasherDB, validatorIdx.Sub(1), att)
	require.NoError(t, err)
	require.Equal(t, (*ethpb.AttesterSlashing)(nil), slashing)

	// Next up we initialize an empty chunks slice and mark an attestation
	// with (source 0, target 3) as attested.
	chunk = EmptyMaxSpanChunksSlice(params)
	source = primitives.Epoch(0)
	target = primitives.Epoch(3)
	att = createAttestationWrapper(t, source, target, nil, nil)
	chunkIdx := uint64(0)
	startEpoch := source
	currentEpoch := target
	args := &chunkUpdateArgs{
		chunkIndex:   chunkIdx,
		currentEpoch: currentEpoch,
	}
	_, err = chunk.Update(args, validatorIdx, startEpoch, target)
	require.NoError(t, err)

	// Next up, we create a surrounded vote, but it should NOT be slashable
	// because we DO NOT have an existing attestation record in our database at the max target epoch.
	source = primitives.Epoch(1)
	target = primitives.Epoch(2)
	surroundedVote := createAttestationWrapper(t, source, target, nil, nil)

	slashing, err = chunk.CheckSlashable(ctx, slasherDB, validatorIdx, surroundedVote)
	require.NoError(t, err)
	require.Equal(t, (*ethpb.AttesterSlashing)(nil), slashing)

	// Next up, we save the old attestation record, then check if the
	// surroundedVote vote is indeed slashable.
	attData := att.IndexedAttestation.Data
	signingRoot := [32]byte{1}
	attRecord := createAttestationWrapper(
		t, attData.Source.Epoch, attData.Target.Epoch, []uint64{uint64(validatorIdx)}, signingRoot[:],
	)
	err = slasherDB.SaveAttestationRecordsForValidators(
		ctx,
		[]*slashertypes.IndexedAttestationWrapper{attRecord},
	)
	require.NoError(t, err)

	slashing, err = chunk.CheckSlashable(ctx, slasherDB, validatorIdx, surroundedVote)
	require.NoError(t, err)
	require.NotEqual(t, (*ethpb.AttesterSlashing)(nil), slashing)
}

func TestMinSpanChunksSlice_Update_MultipleChunks(t *testing.T) {
	// Let's set H = historyLength = 2, meaning a min span
	// will hold 2 epochs worth of attesting history. Then we set C = 2 meaning we will
	// chunk the min span into arrays each of length 2 and K = 3 meaning we store each chunk index
	// for 3 validators at a time.
	//
	// So assume we get a target 3 for source 0 and validator 0, then, we need to update every epoch in the span from
	// 3 to 0 inclusive. First, we find out which chunk epoch 3 falls into, which is calculated as:
	// chunk_idx = (epoch % H) / C = (3 % 4) / 2 = 1
	//
	//                                       val0        val1        val2
	//                                     {     }     {      }    {      }
	//   chunk_1_for_validators_0_to_3 = [[nil, nil], [nil, nil], [nil, nil]]
	//                                      |    |
	//                                      |    |-> epoch 3 for validator 0
	//                                      |
	//                                      |-> epoch 2 for validator 0
	//
	//                                       val0        val1        val2
	//                                     {     }     {      }    {      }
	//   chunk_0_for_validators_0_to_3 = [[nil, nil], [nil, nil], [nil, nil]]
	//                                      |    |
	//                                      |    |-> epoch 1 for validator 0
	//                                      |
	//                                      |-> epoch 0 for validator 0
	//
	// Next up, we proceed with the update process for validator index 0, starting epoch 3
	// updating every value along the way according to the update rules for min spans.
	//
	// Once we finish updating a chunk, we need to move on to the next chunk. This function
	// returns a boolean named keepGoing which allows the caller to determine if we should
	// continue and update another chunk index. We stop whenever we reach the min epoch we need
	// to update, in our example, we stop at 0, which is a part chunk 0, so we need to perform updates
	// across 2 different min span chunk slices as shown above.
	params := &Parameters{
		chunkSize:          2,
		validatorChunkSize: 3,
		historyLength:      4,
	}
	chunk := EmptyMinSpanChunksSlice(params)
	target := primitives.Epoch(3)
	chunkIdx := uint64(1)
	validatorIdx := primitives.ValidatorIndex(0)
	startEpoch := target
	currentEpoch := target
	args := &chunkUpdateArgs{
		chunkIndex:   chunkIdx,
		currentEpoch: currentEpoch,
	}
	keepGoing, err := chunk.Update(args, validatorIdx, startEpoch, target)
	require.NoError(t, err)

	// We should keep going! We still have to update the data for chunk index 0.
	require.Equal(t, true, keepGoing)
	want := []uint16{1, 0, math.MaxUint16, math.MaxUint16, math.MaxUint16, math.MaxUint16}
	require.DeepEqual(t, want, chunk.Chunk())

	// Now we update for chunk index 0.
	chunk = EmptyMinSpanChunksSlice(params)
	chunkIdx = uint64(0)
	validatorIdx = primitives.ValidatorIndex(0)
	startEpoch = primitives.Epoch(1)
	currentEpoch = target
	args = &chunkUpdateArgs{
		chunkIndex:   chunkIdx,
		currentEpoch: currentEpoch,
	}
	keepGoing, err = chunk.Update(args, validatorIdx, startEpoch, target)
	require.NoError(t, err)
	require.Equal(t, false, keepGoing)
	want = []uint16{3, 2, math.MaxUint16, math.MaxUint16, math.MaxUint16, math.MaxUint16}
	require.DeepEqual(t, want, chunk.Chunk())
}

func TestMaxSpanChunksSlice_Update_MultipleChunks(t *testing.T) {
	params := &Parameters{
		chunkSize:          2,
		validatorChunkSize: 3,
		historyLength:      4,
	}
	chunk := EmptyMaxSpanChunksSlice(params)
	target := primitives.Epoch(3)
	chunkIdx := uint64(0)
	validatorIdx := primitives.ValidatorIndex(0)
	startEpoch := primitives.Epoch(0)
	currentEpoch := target
	args := &chunkUpdateArgs{
		chunkIndex:   chunkIdx,
		currentEpoch: currentEpoch,
	}
	keepGoing, err := chunk.Update(args, validatorIdx, startEpoch, target)
	require.NoError(t, err)

	// We should keep going! We still have to update the data for chunk index 1.
	require.Equal(t, true, keepGoing)
	want := []uint16{3, 2, 0, 0, 0, 0}
	require.DeepEqual(t, want, chunk.Chunk())

	// Now we update for chunk index 1.
	chunk = EmptyMaxSpanChunksSlice(params)
	chunkIdx = uint64(1)
	validatorIdx = primitives.ValidatorIndex(0)
	startEpoch = primitives.Epoch(2)
	currentEpoch = target
	args = &chunkUpdateArgs{
		chunkIndex:   chunkIdx,
		currentEpoch: currentEpoch,
	}
	keepGoing, err = chunk.Update(args, validatorIdx, startEpoch, target)
	require.NoError(t, err)
	require.Equal(t, false, keepGoing)
	want = []uint16{1, 0, 0, 0, 0, 0}
	require.DeepEqual(t, want, chunk.Chunk())
}

func TestMinSpanChunksSlice_Update_SingleChunk(t *testing.T) {
	// Let's set H = historyLength = 2, meaning a min span
	// will hold 2 epochs worth of attesting history. Then we set C = 2 meaning we will
	// chunk the min span into arrays each of length 2 and K = 3 meaning we store each chunk index
	// for 3 validators at a time.
	//
	// So assume we get a target 1 for source 0 and validator 0, then, we need to update every epoch in the span from
	// 1 to 0 inclusive. First, we find out which chunk epoch 4 falls into, which is calculated as:
	// chunk_idx = (epoch % H) / C = (1 % 2) / 2 = 0
	//
	//                                       val0        val1        val2
	//                                     {     }     {      }    {      }
	//   chunk_0_for_validators_0_to_3 = [[nil, nil], [nil, nil], [nil, nil]]
	//                                           |
	//                                           |-> epoch 1 for validator 0
	//
	// Next up, we proceed with the update process for validator index 0, starting epoch 1
	// updating every value along the way according to the update rules for min spans.
	//
	// Once we finish updating a chunk, we need to move on to the next chunk. This function
	// returns a boolean named keepGoing which allows the caller to determine if we should
	// continue and update another chunk index. We stop whenever we reach the min epoch we need
	// to update, in our example, we stop at 0, which is still part of chunk 0, so there is no
	// need to keep going.
	params := &Parameters{
		chunkSize:          2,
		validatorChunkSize: 3,
		historyLength:      2,
	}
	chunk := EmptyMinSpanChunksSlice(params)
	target := primitives.Epoch(1)
	chunkIdx := uint64(0)
	validatorIdx := primitives.ValidatorIndex(0)
	startEpoch := target
	currentEpoch := target
	args := &chunkUpdateArgs{
		chunkIndex:   chunkIdx,
		currentEpoch: currentEpoch,
	}
	keepGoing, err := chunk.Update(args, validatorIdx, startEpoch, target)
	require.NoError(t, err)
	require.Equal(t, false, keepGoing)
	want := []uint16{1, 0, math.MaxUint16, math.MaxUint16, math.MaxUint16, math.MaxUint16}
	require.DeepEqual(t, want, chunk.Chunk())
}

func TestMaxSpanChunksSlice_Update_SingleChunk(t *testing.T) {
	params := &Parameters{
		chunkSize:          4,
		validatorChunkSize: 2,
		historyLength:      4,
	}
	chunk := EmptyMaxSpanChunksSlice(params)
	target := primitives.Epoch(3)
	chunkIdx := uint64(0)
	validatorIdx := primitives.ValidatorIndex(0)
	startEpoch := primitives.Epoch(0)
	currentEpoch := target
	args := &chunkUpdateArgs{
		chunkIndex:   chunkIdx,
		currentEpoch: currentEpoch,
	}
	keepGoing, err := chunk.Update(args, validatorIdx, startEpoch, target)
	require.NoError(t, err)
	require.Equal(t, false, keepGoing)
	want := []uint16{3, 2, 1, 0, 0, 0, 0, 0}
	require.DeepEqual(t, want, chunk.Chunk())
}

func TestMinSpanChunksSlice_StartEpoch(t *testing.T) {
	type args struct {
		sourceEpoch  primitives.Epoch
		currentEpoch primitives.Epoch
	}
	tests := []struct {
		name           string
		params         *Parameters
		args           args
		wantEpoch      primitives.Epoch
		shouldNotExist bool
	}{
		{
			name:   "source_epoch == 0 returns false",
			params: DefaultParams(),
			args: args{
				sourceEpoch: 0,
			},
			shouldNotExist: true,
		},
		{
			name: "source_epoch == (current_epoch - HISTORY_LENGTH) returns false",
			params: &Parameters{
				historyLength: 3,
			},
			args: args{
				sourceEpoch:  1,
				currentEpoch: 4,
			},
			shouldNotExist: true,
		},
		{
			name: "source_epoch < (current_epoch - HISTORY_LENGTH) returns false",
			params: &Parameters{
				historyLength: 3,
			},
			args: args{
				sourceEpoch:  1,
				currentEpoch: 5,
			},
			shouldNotExist: true,
		},
		{
			name: "source_epoch > (current_epoch - HISTORY_LENGTH) returns true",
			params: &Parameters{
				historyLength: 3,
			},
			args: args{
				sourceEpoch:  1,
				currentEpoch: 3,
			},
			wantEpoch: 0,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			m := &MinSpanChunksSlice{
				params: tt.params,
			}
			gotEpoch, gotExists := m.StartEpoch(tt.args.sourceEpoch, tt.args.currentEpoch)
			assert.Equal(t, false, tt.shouldNotExist && gotExists)
			assert.Equal(t, false, !tt.shouldNotExist && gotEpoch != tt.wantEpoch)
		})
	}
}

func TestMaxSpanChunksSlice_StartEpoch(t *testing.T) {
	type args struct {
		sourceEpoch  primitives.Epoch
		currentEpoch primitives.Epoch
	}
	tests := []struct {
		name           string
		params         *Parameters
		args           args
		wantEpoch      primitives.Epoch
		shouldNotExist bool
	}{
		{
			name:   "source_epoch == current_epoch returns false",
			params: DefaultParams(),
			args: args{
				sourceEpoch:  1,
				currentEpoch: 1,
			},
			shouldNotExist: true,
		},
		{
			name:   "source_epoch > current_epoch returns false",
			params: DefaultParams(),
			args: args{
				sourceEpoch:  2,
				currentEpoch: 1,
			},
			shouldNotExist: true,
		},
		{
			name:   "source_epoch < current_epoch returns true",
			params: DefaultParams(),
			args: args{
				sourceEpoch:  1,
				currentEpoch: 2,
			},
			wantEpoch: 2,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			m := &MaxSpanChunksSlice{
				params: tt.params,
			}
			gotEpoch, gotExists := m.StartEpoch(tt.args.sourceEpoch, tt.args.currentEpoch)
			assert.Equal(t, false, tt.shouldNotExist && gotExists)
			assert.Equal(t, false, !tt.shouldNotExist && gotEpoch != tt.wantEpoch)
		})
	}
}

func TestMinSpanChunksSlice_NextChunkStartEpoch(t *testing.T) {
	tests := []struct {
		name       string
		params     *Parameters
		startEpoch primitives.Epoch
		want       primitives.Epoch
	}{
		{
			name: "Start epoch 0",
			params: &Parameters{
				chunkSize:     3,
				historyLength: 4096,
			},
			startEpoch: 0,
			want:       2,
		},
		{
			name: "Start epoch of chunk 1 returns last epoch of chunk 0",
			params: &Parameters{
				chunkSize:     3,
				historyLength: 4096,
			},
			startEpoch: 3,
			want:       2,
		},
		{
			name: "Start epoch inside of chunk 2 returns last epoch of chunk 1",
			params: &Parameters{
				chunkSize:     3,
				historyLength: 4096,
			},
			startEpoch: 8,
			want:       5,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			m := &MinSpanChunksSlice{
				params: tt.params,
			}
			got := m.NextChunkStartEpoch(tt.startEpoch)
			assert.Equal(t, true, got == tt.want)
		})
	}
}

func TestMaxSpanChunksSlice_NextChunkStartEpoch(t *testing.T) {
	tests := []struct {
		name       string
		params     *Parameters
		startEpoch primitives.Epoch
		want       primitives.Epoch
	}{
		{
			name: "Start epoch 0",
			params: &Parameters{
				chunkSize:     3,
				historyLength: 4,
			},
			startEpoch: 0,
			want:       3,
		},
		{
			name: "Start epoch of chunk 1 returns start epoch of chunk 2",
			params: &Parameters{
				chunkSize:     3,
				historyLength: 4,
			},
			startEpoch: 3,
			want:       6,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			m := &MaxSpanChunksSlice{
				params: tt.params,
			}
			got := m.NextChunkStartEpoch(tt.startEpoch)
			assert.Equal(t, true, got == tt.want)
		})
	}
}

func Test_chunkDataAtEpoch_SetRetrieve(t *testing.T) {
	// We initialize a chunks slice for 2 validators and with chunk size 3,
	// which will look as follows:
	//
	//     val0     val1
	//   {     }  {     }
	//  [2, 2, 2, 2, 2, 2]
	//
	// To give an example, epoch 1 for validator 1 will be at the following position:
	//
	//  [2, 2, 2, 2, 2, 2]
	//               |-> epoch 1, validator 1.
	params := &Parameters{
		chunkSize:          3,
		validatorChunkSize: 2,
	}
	chunk := []uint16{2, 2, 2, 2, 2, 2}
	validatorIdx := primitives.ValidatorIndex(1)
	epochInChunk := primitives.Epoch(1)

	// We expect a chunk with the wrong length to throw an error.
	_, err := chunkDataAtEpoch(params, []uint16{}, validatorIdx, epochInChunk)
	require.ErrorContains(t, "chunk has wrong length", err)

	// We update the value for epoch 1 using target epoch 6.
	targetEpoch := primitives.Epoch(6)
	err = setChunkDataAtEpoch(params, chunk, validatorIdx, epochInChunk, targetEpoch)
	require.NoError(t, err)

	// We expect the retrieved value at epoch 1 is the target epoch 6.
	received, err := chunkDataAtEpoch(params, chunk, validatorIdx, epochInChunk)
	require.NoError(t, err)
	assert.Equal(t, targetEpoch, received)
}

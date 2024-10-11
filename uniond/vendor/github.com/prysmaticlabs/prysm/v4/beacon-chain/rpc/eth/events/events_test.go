package events

import (
	"fmt"
	"io"
	"math"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	mockChain "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/operation"
	statefeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/state"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/eth/v1"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

type flushableResponseRecorder struct {
	*httptest.ResponseRecorder
	flushed bool
}

func (f *flushableResponseRecorder) Flush() {
	f.flushed = true
}

func TestStreamEvents_OperationsEvents(t *testing.T) {
	t.Run("operations", func(t *testing.T) {
		s := &Server{
			StateNotifier:     &mockChain.MockStateNotifier{},
			OperationNotifier: &mockChain.MockOperationNotifier{},
		}

		topics := []string{
			AttestationTopic,
			VoluntaryExitTopic,
			SyncCommitteeContributionTopic,
			BLSToExecutionChangeTopic,
			BlobSidecarTopic,
			AttesterSlashingTopic,
			ProposerSlashingTopic,
		}
		for i, topic := range topics {
			topics[i] = "topics=" + topic
		}
		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://example.com/eth/v1/events?%s", strings.Join(topics, "&")), nil)
		w := &flushableResponseRecorder{
			ResponseRecorder: httptest.NewRecorder(),
		}

		go func() {
			s.StreamEvents(w, request)
		}()
		// wait for initiation of StreamEvents
		time.Sleep(100 * time.Millisecond)
		s.OperationNotifier.OperationFeed().Send(&feed.Event{
			Type: operation.UnaggregatedAttReceived,
			Data: &operation.UnAggregatedAttReceivedData{
				Attestation: util.HydrateAttestation(&eth.Attestation{}),
			},
		})
		s.OperationNotifier.OperationFeed().Send(&feed.Event{
			Type: operation.AggregatedAttReceived,
			Data: &operation.AggregatedAttReceivedData{
				Attestation: &eth.AggregateAttestationAndProof{
					AggregatorIndex: 0,
					Aggregate:       util.HydrateAttestation(&eth.Attestation{}),
					SelectionProof:  make([]byte, 96),
				},
			},
		})
		s.OperationNotifier.OperationFeed().Send(&feed.Event{
			Type: operation.ExitReceived,
			Data: &operation.ExitReceivedData{
				Exit: &eth.SignedVoluntaryExit{
					Exit: &eth.VoluntaryExit{
						Epoch:          0,
						ValidatorIndex: 0,
					},
					Signature: make([]byte, 96),
				},
			},
		})
		s.OperationNotifier.OperationFeed().Send(&feed.Event{
			Type: operation.SyncCommitteeContributionReceived,
			Data: &operation.SyncCommitteeContributionReceivedData{
				Contribution: &eth.SignedContributionAndProof{
					Message: &eth.ContributionAndProof{
						AggregatorIndex: 0,
						Contribution: &eth.SyncCommitteeContribution{
							Slot:              0,
							BlockRoot:         make([]byte, 32),
							SubcommitteeIndex: 0,
							AggregationBits:   make([]byte, 16),
							Signature:         make([]byte, 96),
						},
						SelectionProof: make([]byte, 96),
					},
					Signature: make([]byte, 96),
				},
			},
		})
		s.OperationNotifier.OperationFeed().Send(&feed.Event{
			Type: operation.BLSToExecutionChangeReceived,
			Data: &operation.BLSToExecutionChangeReceivedData{
				Change: &eth.SignedBLSToExecutionChange{
					Message: &eth.BLSToExecutionChange{
						ValidatorIndex:     0,
						FromBlsPubkey:      make([]byte, 48),
						ToExecutionAddress: make([]byte, 20),
					},
					Signature: make([]byte, 96),
				},
			},
		})
		ro, err := blocks.NewROBlob(util.HydrateBlobSidecar(&eth.BlobSidecar{}))
		require.NoError(t, err)
		vblob := blocks.NewVerifiedROBlob(ro)
		s.OperationNotifier.OperationFeed().Send(&feed.Event{
			Type: operation.BlobSidecarReceived,
			Data: &operation.BlobSidecarReceivedData{
				Blob: &vblob,
			},
		})

		s.OperationNotifier.OperationFeed().Send(&feed.Event{
			Type: operation.AttesterSlashingReceived,
			Data: &operation.AttesterSlashingReceivedData{
				AttesterSlashing: &eth.AttesterSlashing{
					Attestation_1: &eth.IndexedAttestation{
						AttestingIndices: []uint64{0, 1},
						Data: &eth.AttestationData{
							BeaconBlockRoot: make([]byte, fieldparams.RootLength),
							Source: &eth.Checkpoint{
								Root: make([]byte, fieldparams.RootLength),
							},
							Target: &eth.Checkpoint{
								Root: make([]byte, fieldparams.RootLength),
							},
						},
						Signature: make([]byte, fieldparams.BLSSignatureLength),
					},
					Attestation_2: &eth.IndexedAttestation{
						AttestingIndices: []uint64{0, 1},
						Data: &eth.AttestationData{
							BeaconBlockRoot: make([]byte, fieldparams.RootLength),
							Source: &eth.Checkpoint{
								Root: make([]byte, fieldparams.RootLength),
							},
							Target: &eth.Checkpoint{
								Root: make([]byte, fieldparams.RootLength),
							},
						},
						Signature: make([]byte, fieldparams.BLSSignatureLength),
					},
				},
			},
		})

		s.OperationNotifier.OperationFeed().Send(&feed.Event{
			Type: operation.ProposerSlashingReceived,
			Data: &operation.ProposerSlashingReceivedData{
				ProposerSlashing: &eth.ProposerSlashing{
					Header_1: &eth.SignedBeaconBlockHeader{
						Header: &eth.BeaconBlockHeader{
							ParentRoot: make([]byte, fieldparams.RootLength),
							StateRoot:  make([]byte, fieldparams.RootLength),
							BodyRoot:   make([]byte, fieldparams.RootLength),
						},
						Signature: make([]byte, fieldparams.BLSSignatureLength),
					},
					Header_2: &eth.SignedBeaconBlockHeader{
						Header: &eth.BeaconBlockHeader{
							ParentRoot: make([]byte, fieldparams.RootLength),
							StateRoot:  make([]byte, fieldparams.RootLength),
							BodyRoot:   make([]byte, fieldparams.RootLength),
						},
						Signature: make([]byte, fieldparams.BLSSignatureLength),
					},
				},
			},
		})

		time.Sleep(1 * time.Second)
		request.Context().Done()

		resp := w.Result()
		body, err := io.ReadAll(resp.Body)
		require.NoError(t, err)
		require.NotNil(t, body)
		assert.Equal(t, operationsResult, string(body))
	})
	t.Run("state", func(t *testing.T) {
		s := &Server{
			StateNotifier:     &mockChain.MockStateNotifier{},
			OperationNotifier: &mockChain.MockOperationNotifier{},
		}

		topics := []string{HeadTopic, FinalizedCheckpointTopic, ChainReorgTopic, BlockTopic}
		for i, topic := range topics {
			topics[i] = "topics=" + topic
		}
		request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://example.com/eth/v1/events?%s", strings.Join(topics, "&")), nil)
		w := &flushableResponseRecorder{
			ResponseRecorder: httptest.NewRecorder(),
		}

		go func() {
			s.StreamEvents(w, request)
		}()
		// wait for initiation of StreamEvents
		time.Sleep(100 * time.Millisecond)
		s.StateNotifier.StateFeed().Send(&feed.Event{
			Type: statefeed.NewHead,
			Data: &ethpb.EventHead{
				Slot:                      0,
				Block:                     make([]byte, 32),
				State:                     make([]byte, 32),
				EpochTransition:           true,
				PreviousDutyDependentRoot: make([]byte, 32),
				CurrentDutyDependentRoot:  make([]byte, 32),
				ExecutionOptimistic:       false,
			},
		})
		s.StateNotifier.StateFeed().Send(&feed.Event{
			Type: statefeed.FinalizedCheckpoint,
			Data: &ethpb.EventFinalizedCheckpoint{
				Block:               make([]byte, 32),
				State:               make([]byte, 32),
				Epoch:               0,
				ExecutionOptimistic: false,
			},
		})
		s.StateNotifier.StateFeed().Send(&feed.Event{
			Type: statefeed.Reorg,
			Data: &ethpb.EventChainReorg{
				Slot:                0,
				Depth:               0,
				OldHeadBlock:        make([]byte, 32),
				NewHeadBlock:        make([]byte, 32),
				OldHeadState:        make([]byte, 32),
				NewHeadState:        make([]byte, 32),
				Epoch:               0,
				ExecutionOptimistic: false,
			},
		})
		b, err := blocks.NewSignedBeaconBlock(util.HydrateSignedBeaconBlock(&eth.SignedBeaconBlock{}))
		require.NoError(t, err)
		s.StateNotifier.StateFeed().Send(&feed.Event{
			Type: statefeed.BlockProcessed,
			Data: &statefeed.BlockProcessedData{
				Slot:        0,
				BlockRoot:   [32]byte{},
				SignedBlock: b,
				Verified:    true,
				Optimistic:  false,
			},
		})

		// wait for feed
		time.Sleep(1 * time.Second)
		request.Context().Done()

		resp := w.Result()
		body, err := io.ReadAll(resp.Body)
		require.NoError(t, err)
		require.NotNil(t, body)
		assert.Equal(t, stateResult, string(body))
	})
	t.Run("payload attributes", func(t *testing.T) {
		type testCase struct {
			name     string
			getState func() state.BeaconState
			getBlock func() interfaces.SignedBeaconBlock
			expected string
		}
		testCases := []testCase{
			{
				name: "bellatrix",
				getState: func() state.BeaconState {
					st, err := util.NewBeaconStateBellatrix()
					require.NoError(t, err)
					return st
				},
				getBlock: func() interfaces.SignedBeaconBlock {
					b, err := blocks.NewSignedBeaconBlock(util.HydrateSignedBeaconBlockBellatrix(&eth.SignedBeaconBlockBellatrix{}))
					require.NoError(t, err)
					return b
				},
				expected: payloadAttributesBellatrixResult,
			},
			{
				name: "capella",
				getState: func() state.BeaconState {
					st, err := util.NewBeaconStateCapella()
					require.NoError(t, err)
					return st
				},
				getBlock: func() interfaces.SignedBeaconBlock {
					b, err := blocks.NewSignedBeaconBlock(util.HydrateSignedBeaconBlockCapella(&eth.SignedBeaconBlockCapella{}))
					require.NoError(t, err)
					return b
				},
				expected: payloadAttributesCapellaResult,
			},
			{
				name: "deneb",
				getState: func() state.BeaconState {
					st, err := util.NewBeaconStateDeneb()
					require.NoError(t, err)
					return st
				},
				getBlock: func() interfaces.SignedBeaconBlock {
					b, err := blocks.NewSignedBeaconBlock(util.HydrateSignedBeaconBlockDeneb(&eth.SignedBeaconBlockDeneb{}))
					require.NoError(t, err)
					return b
				},
				expected: payloadAttributesDenebResult,
			},
		}
		for _, tc := range testCases {
			st := tc.getState()
			v := &eth.Validator{ExitEpoch: math.MaxUint64}
			require.NoError(t, st.SetValidators([]*eth.Validator{v}))
			currentSlot := primitives.Slot(0)
			// to avoid slot processing
			require.NoError(t, st.SetSlot(currentSlot+1))
			b := tc.getBlock()
			mockChainService := &mockChain.ChainService{
				Root:  make([]byte, 32),
				State: st,
				Block: b,
				Slot:  &currentSlot,
			}
			s := &Server{
				StateNotifier:     &mockChain.MockStateNotifier{},
				OperationNotifier: &mockChain.MockOperationNotifier{},
				HeadFetcher:       mockChainService,
				ChainInfoFetcher:  mockChainService,
			}

			request := httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://example.com/eth/v1/events?topics=%s", PayloadAttributesTopic), nil)
			w := &flushableResponseRecorder{
				ResponseRecorder: httptest.NewRecorder(),
			}

			go func() {
				s.StreamEvents(w, request)
			}()
			// wait for initiation of StreamEvents
			time.Sleep(100 * time.Millisecond)
			s.StateNotifier.StateFeed().Send(&feed.Event{Type: statefeed.MissedSlot})

			// wait for feed
			time.Sleep(1 * time.Second)
			request.Context().Done()

			resp := w.Result()
			body, err := io.ReadAll(resp.Body)
			require.NoError(t, err)
			require.NotNil(t, body)
			assert.Equal(t, tc.expected, string(body), "wrong result for "+tc.name)
		}
	})
}

const operationsResult = `:

event: attestation
data: {"aggregation_bits":"0x00","data":{"slot":"0","index":"0","beacon_block_root":"0x0000000000000000000000000000000000000000000000000000000000000000","source":{"epoch":"0","root":"0x0000000000000000000000000000000000000000000000000000000000000000"},"target":{"epoch":"0","root":"0x0000000000000000000000000000000000000000000000000000000000000000"}},"signature":"0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"}

event: attestation
data: {"aggregation_bits":"0x00","data":{"slot":"0","index":"0","beacon_block_root":"0x0000000000000000000000000000000000000000000000000000000000000000","source":{"epoch":"0","root":"0x0000000000000000000000000000000000000000000000000000000000000000"},"target":{"epoch":"0","root":"0x0000000000000000000000000000000000000000000000000000000000000000"}},"signature":"0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"}

event: voluntary_exit
data: {"message":{"epoch":"0","validator_index":"0"},"signature":"0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"}

event: contribution_and_proof
data: {"message":{"aggregator_index":"0","contribution":{"slot":"0","beacon_block_root":"0x0000000000000000000000000000000000000000000000000000000000000000","subcommittee_index":"0","aggregation_bits":"0x00000000000000000000000000000000","signature":"0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"},"selection_proof":"0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"},"signature":"0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"}

event: bls_to_execution_change
data: {"message":{"validator_index":"0","from_bls_pubkey":"0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","to_execution_address":"0x0000000000000000000000000000000000000000"},"signature":"0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"}

event: blob_sidecar
data: {"block_root":"0xc78009fdf07fc56a11f122370658a353aaa542ed63e44c4bc15ff4cd105ab33c","index":"0","slot":"0","kzg_commitment":"0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000","versioned_hash":"0x01b0761f87b081d5cf10757ccc89f12be355c70e2e29df288b65b30710dcbcd1"}

event: attester_slashing
data: {"attestation_1":{"attesting_indices":["0","1"],"data":{"slot":"0","index":"0","beacon_block_root":"0x0000000000000000000000000000000000000000000000000000000000000000","source":{"epoch":"0","root":"0x0000000000000000000000000000000000000000000000000000000000000000"},"target":{"epoch":"0","root":"0x0000000000000000000000000000000000000000000000000000000000000000"}},"signature":"0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"},"attestation_2":{"attesting_indices":["0","1"],"data":{"slot":"0","index":"0","beacon_block_root":"0x0000000000000000000000000000000000000000000000000000000000000000","source":{"epoch":"0","root":"0x0000000000000000000000000000000000000000000000000000000000000000"},"target":{"epoch":"0","root":"0x0000000000000000000000000000000000000000000000000000000000000000"}},"signature":"0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"}}

event: proposer_slashing
data: {"signed_header_1":{"message":{"slot":"0","proposer_index":"0","parent_root":"0x0000000000000000000000000000000000000000000000000000000000000000","state_root":"0x0000000000000000000000000000000000000000000000000000000000000000","body_root":"0x0000000000000000000000000000000000000000000000000000000000000000"},"signature":"0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"},"signed_header_2":{"message":{"slot":"0","proposer_index":"0","parent_root":"0x0000000000000000000000000000000000000000000000000000000000000000","state_root":"0x0000000000000000000000000000000000000000000000000000000000000000","body_root":"0x0000000000000000000000000000000000000000000000000000000000000000"},"signature":"0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"}}

`

const stateResult = `:

event: head
data: {"slot":"0","block":"0x0000000000000000000000000000000000000000000000000000000000000000","state":"0x0000000000000000000000000000000000000000000000000000000000000000","epoch_transition":true,"execution_optimistic":false,"previous_duty_dependent_root":"0x0000000000000000000000000000000000000000000000000000000000000000","current_duty_dependent_root":"0x0000000000000000000000000000000000000000000000000000000000000000"}

event: finalized_checkpoint
data: {"block":"0x0000000000000000000000000000000000000000000000000000000000000000","state":"0x0000000000000000000000000000000000000000000000000000000000000000","epoch":"0","execution_optimistic":false}

event: chain_reorg
data: {"slot":"0","depth":"0","old_head_block":"0x0000000000000000000000000000000000000000000000000000000000000000","old_head_state":"0x0000000000000000000000000000000000000000000000000000000000000000","new_head_block":"0x0000000000000000000000000000000000000000000000000000000000000000","new_head_state":"0x0000000000000000000000000000000000000000000000000000000000000000","epoch":"0","execution_optimistic":false}

event: block
data: {"slot":"0","block":"0xeade62f0457b2fdf48e7d3fc4b60736688286be7c7a3ac4c9a16a5e0600bd9e4","execution_optimistic":false}

`

const payloadAttributesBellatrixResult = `:

event: payload_attributes
data: {"version":"bellatrix","data":{"proposer_index":"0","proposal_slot":"1","parent_block_number":"0","parent_block_root":"0x0000000000000000000000000000000000000000000000000000000000000000","parent_block_hash":"0x0000000000000000000000000000000000000000000000000000000000000000","payload_attributes":{"timestamp":"12","prev_randao":"0x0000000000000000000000000000000000000000000000000000000000000000","suggested_fee_recipient":"0x0000000000000000000000000000000000000000"}}}

`

const payloadAttributesCapellaResult = `:

event: payload_attributes
data: {"version":"capella","data":{"proposer_index":"0","proposal_slot":"1","parent_block_number":"0","parent_block_root":"0x0000000000000000000000000000000000000000000000000000000000000000","parent_block_hash":"0x0000000000000000000000000000000000000000000000000000000000000000","payload_attributes":{"timestamp":"12","prev_randao":"0x0000000000000000000000000000000000000000000000000000000000000000","suggested_fee_recipient":"0x0000000000000000000000000000000000000000","withdrawals":[]}}}

`

const payloadAttributesDenebResult = `:

event: payload_attributes
data: {"version":"deneb","data":{"proposer_index":"0","proposal_slot":"1","parent_block_number":"0","parent_block_root":"0x0000000000000000000000000000000000000000000000000000000000000000","parent_block_hash":"0x0000000000000000000000000000000000000000000000000000000000000000","payload_attributes":{"timestamp":"12","prev_randao":"0x0000000000000000000000000000000000000000000000000000000000000000","suggested_fee_recipient":"0x0000000000000000000000000000000000000000","withdrawals":[],"parent_beacon_block_root":"0xbef96cb938fd48b2403d3e662664325abb0102ed12737cbb80d717520e50cf4a"}}}

`

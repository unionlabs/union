package rewards

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"strconv"
	"strings"
	"testing"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/go-bitfield"
	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/altair"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/testutil"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	mockstategen "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen/mock"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls/blst"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func BlockRewardTestSetup(t *testing.T, forkName string) (state.BeaconState, interfaces.SignedBeaconBlock, error) {
	helpers.ClearCache()
	var sbb interfaces.SignedBeaconBlock
	var st state.BeaconState
	var err error
	switch forkName {
	case "phase0":
		return nil, nil, errors.New("phase0 not supported")
	case "altair":
		st, err = util.NewBeaconStateAltair()
		require.NoError(t, err)
		b := util.HydrateSignedBeaconBlockAltair(util.NewBeaconBlockAltair())
		sbb, err = blocks.NewSignedBeaconBlock(b)
		require.NoError(t, err)
	case "bellatrix":
		st, err = util.NewBeaconStateBellatrix()
		require.NoError(t, err)
		b := util.HydrateSignedBeaconBlockBellatrix(util.NewBeaconBlockBellatrix())
		sbb, err = blocks.NewSignedBeaconBlock(b)
		require.NoError(t, err)
	case "capella":
		st, err = util.NewBeaconStateCapella()
		require.NoError(t, err)
		b := util.HydrateSignedBeaconBlockCapella(util.NewBeaconBlockCapella())
		sbb, err = blocks.NewSignedBeaconBlock(b)
		require.NoError(t, err)
	case "deneb":
		st, err = util.NewBeaconStateDeneb()
		require.NoError(t, err)
		b := util.HydrateSignedBeaconBlockDeneb(util.NewBeaconBlockDeneb())
		sbb, err = blocks.NewSignedBeaconBlock(b)
		require.NoError(t, err)
	default:
		return nil, nil, errors.New("fork is not supported")
	}
	valCount := 64
	require.NoError(t, st.SetSlot(1))
	require.NoError(t, err)
	validators := make([]*eth.Validator, 0, valCount)
	balances := make([]uint64, 0, valCount)
	secretKeys := make([]bls.SecretKey, 0, valCount)
	for i := 0; i < valCount; i++ {
		blsKey, err := bls.RandKey()
		require.NoError(t, err)
		secretKeys = append(secretKeys, blsKey)
		validators = append(validators, &eth.Validator{
			PublicKey:         blsKey.PublicKey().Marshal(),
			ExitEpoch:         params.BeaconConfig().FarFutureEpoch,
			WithdrawableEpoch: params.BeaconConfig().FarFutureEpoch,
			EffectiveBalance:  params.BeaconConfig().MaxEffectiveBalance,
		})
		balances = append(balances, params.BeaconConfig().MaxEffectiveBalance)
	}
	require.NoError(t, st.SetValidators(validators))
	require.NoError(t, st.SetBalances(balances))
	require.NoError(t, st.SetCurrentParticipationBits(make([]byte, valCount)))
	syncCommittee, err := altair.NextSyncCommittee(context.Background(), st)
	require.NoError(t, err)
	require.NoError(t, st.SetCurrentSyncCommittee(syncCommittee))
	slot0bRoot := bytesutil.PadTo([]byte("slot0root"), 32)
	bRoots := make([][]byte, fieldparams.BlockRootsLength)
	bRoots[0] = slot0bRoot
	require.NoError(t, st.SetBlockRoots(bRoots))

	sbb.SetSlot(2)
	// we have to set the proposer index to the value that will be randomly chosen (fortunately it's deterministic)
	sbb.SetProposerIndex(12)
	sbb.SetAttestations([]*eth.Attestation{
		{
			AggregationBits: bitfield.Bitlist{0b00000111},
			Data:            util.HydrateAttestationData(&eth.AttestationData{}),
			Signature:       make([]byte, fieldparams.BLSSignatureLength),
		},
		{
			AggregationBits: bitfield.Bitlist{0b00000111},
			Data:            util.HydrateAttestationData(&eth.AttestationData{}),
			Signature:       make([]byte, fieldparams.BLSSignatureLength),
		},
	})

	attData1 := util.HydrateAttestationData(&eth.AttestationData{BeaconBlockRoot: bytesutil.PadTo([]byte("root1"), 32)})
	attData2 := util.HydrateAttestationData(&eth.AttestationData{BeaconBlockRoot: bytesutil.PadTo([]byte("root2"), 32)})
	domain, err := signing.Domain(st.Fork(), 0, params.BeaconConfig().DomainBeaconAttester, st.GenesisValidatorsRoot())
	require.NoError(t, err)
	sigRoot1, err := signing.ComputeSigningRoot(attData1, domain)
	require.NoError(t, err)
	sigRoot2, err := signing.ComputeSigningRoot(attData2, domain)
	require.NoError(t, err)
	sbb.SetAttesterSlashings([]*eth.AttesterSlashing{
		{
			Attestation_1: &eth.IndexedAttestation{
				AttestingIndices: []uint64{0},
				Data:             attData1,
				Signature:        secretKeys[0].Sign(sigRoot1[:]).Marshal(),
			},
			Attestation_2: &eth.IndexedAttestation{
				AttestingIndices: []uint64{0},
				Data:             attData2,
				Signature:        secretKeys[0].Sign(sigRoot2[:]).Marshal(),
			},
		},
	})
	header1 := &eth.BeaconBlockHeader{
		Slot:          0,
		ProposerIndex: 1,
		ParentRoot:    bytesutil.PadTo([]byte("root1"), 32),
		StateRoot:     bytesutil.PadTo([]byte("root1"), 32),
		BodyRoot:      bytesutil.PadTo([]byte("root1"), 32),
	}
	header2 := &eth.BeaconBlockHeader{
		Slot:          0,
		ProposerIndex: 1,
		ParentRoot:    bytesutil.PadTo([]byte("root2"), 32),
		StateRoot:     bytesutil.PadTo([]byte("root2"), 32),
		BodyRoot:      bytesutil.PadTo([]byte("root2"), 32),
	}
	domain, err = signing.Domain(st.Fork(), 0, params.BeaconConfig().DomainBeaconProposer, st.GenesisValidatorsRoot())
	require.NoError(t, err)
	sigRoot1, err = signing.ComputeSigningRoot(header1, domain)
	require.NoError(t, err)
	sigRoot2, err = signing.ComputeSigningRoot(header2, domain)
	require.NoError(t, err)
	sbb.SetProposerSlashings([]*eth.ProposerSlashing{
		{
			Header_1: &eth.SignedBeaconBlockHeader{
				Header:    header1,
				Signature: secretKeys[1].Sign(sigRoot1[:]).Marshal(),
			},
			Header_2: &eth.SignedBeaconBlockHeader{
				Header:    header2,
				Signature: secretKeys[1].Sign(sigRoot2[:]).Marshal(),
			},
		},
	})
	scBits := bitfield.NewBitvector512()
	scBits.SetBitAt(10, true)
	scBits.SetBitAt(100, true)
	domain, err = signing.Domain(st.Fork(), 0, params.BeaconConfig().DomainSyncCommittee, st.GenesisValidatorsRoot())
	require.NoError(t, err)
	sszBytes := primitives.SSZBytes(slot0bRoot)
	r, err := signing.ComputeSigningRoot(&sszBytes, domain)
	require.NoError(t, err)
	// Bits set in sync committee bits determine which validators will be treated as participating in sync committee.
	// These validators have to sign the message.
	sig1, err := blst.SignatureFromBytes(secretKeys[47].Sign(r[:]).Marshal())
	require.NoError(t, err)
	sig2, err := blst.SignatureFromBytes(secretKeys[19].Sign(r[:]).Marshal())
	require.NoError(t, err)
	aggSig := bls.AggregateSignatures([]bls.Signature{sig1, sig2}).Marshal()
	err = sbb.SetSyncAggregate(&eth.SyncAggregate{SyncCommitteeBits: scBits, SyncCommitteeSignature: aggSig})
	require.NoError(t, err)

	return st, sbb, nil
}

func TestBlockRewards(t *testing.T) {
	phase0block, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlock())
	require.NoError(t, err)
	t.Run("phase 0", func(t *testing.T) {
		mockChainService := &mock.ChainService{Optimistic: true}
		s := &Server{
			Blocker: &testutil.MockBlocker{SlotBlockMap: map[primitives.Slot]interfaces.ReadOnlySignedBeaconBlock{
				0: phase0block,
			}},
			OptimisticModeFetcher: mockChainService,
			FinalizationFetcher:   mockChainService,
		}
		url := "http://only.the.slot.number.at.the.end.is.important/0"
		request := httptest.NewRequest("GET", url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.BlockRewards(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, "Block rewards are not supported for Phase 0 blocks", e.Message)
	})
	t.Run("altair", func(t *testing.T) {
		st, sbb, err := BlockRewardTestSetup(t, "altair")
		require.NoError(t, err)

		mockChainService := &mock.ChainService{Optimistic: true}
		s := &Server{
			Blocker: &testutil.MockBlocker{SlotBlockMap: map[primitives.Slot]interfaces.ReadOnlySignedBeaconBlock{
				0: phase0block,
				2: sbb,
			}},
			OptimisticModeFetcher: mockChainService,
			FinalizationFetcher:   mockChainService,
			BlockRewardFetcher:    &BlockRewardService{Replayer: mockstategen.NewReplayerBuilder(mockstategen.WithMockState(st))},
		}

		url := "http://only.the.slot.number.at.the.end.is.important/2"
		request := httptest.NewRequest("GET", url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.BlockRewards(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &BlockRewardsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, "12", resp.Data.ProposerIndex)
		assert.Equal(t, "125089490", resp.Data.Total)
		assert.Equal(t, "89442", resp.Data.Attestations)
		assert.Equal(t, "48", resp.Data.SyncAggregate)
		assert.Equal(t, "62500000", resp.Data.AttesterSlashings)
		assert.Equal(t, "62500000", resp.Data.ProposerSlashings)
		assert.Equal(t, true, resp.ExecutionOptimistic)
		assert.Equal(t, false, resp.Finalized)
	})
	t.Run("bellatrix", func(t *testing.T) {
		st, sbb, err := BlockRewardTestSetup(t, "bellatrix")
		require.NoError(t, err)

		mockChainService := &mock.ChainService{Optimistic: true}
		s := &Server{
			Blocker: &testutil.MockBlocker{SlotBlockMap: map[primitives.Slot]interfaces.ReadOnlySignedBeaconBlock{
				0: phase0block,
				2: sbb,
			}},
			OptimisticModeFetcher: mockChainService,
			FinalizationFetcher:   mockChainService,
			BlockRewardFetcher:    &BlockRewardService{Replayer: mockstategen.NewReplayerBuilder(mockstategen.WithMockState(st))},
		}

		url := "http://only.the.slot.number.at.the.end.is.important/2"
		request := httptest.NewRequest("GET", url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.BlockRewards(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &BlockRewardsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, "12", resp.Data.ProposerIndex)
		assert.Equal(t, "125089490", resp.Data.Total)
		assert.Equal(t, "89442", resp.Data.Attestations)
		assert.Equal(t, "48", resp.Data.SyncAggregate)
		assert.Equal(t, "62500000", resp.Data.AttesterSlashings)
		assert.Equal(t, "62500000", resp.Data.ProposerSlashings)
		assert.Equal(t, true, resp.ExecutionOptimistic)
		assert.Equal(t, false, resp.Finalized)
	})
	t.Run("capella", func(t *testing.T) {
		st, sbb, err := BlockRewardTestSetup(t, "capella")
		require.NoError(t, err)

		mockChainService := &mock.ChainService{Optimistic: true}
		s := &Server{
			Blocker: &testutil.MockBlocker{SlotBlockMap: map[primitives.Slot]interfaces.ReadOnlySignedBeaconBlock{
				0: phase0block,
				2: sbb,
			}},
			OptimisticModeFetcher: mockChainService,
			FinalizationFetcher:   mockChainService,
			BlockRewardFetcher:    &BlockRewardService{Replayer: mockstategen.NewReplayerBuilder(mockstategen.WithMockState(st))},
		}

		url := "http://only.the.slot.number.at.the.end.is.important/2"
		request := httptest.NewRequest("GET", url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.BlockRewards(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &BlockRewardsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, "12", resp.Data.ProposerIndex)
		assert.Equal(t, "125089490", resp.Data.Total)
		assert.Equal(t, "89442", resp.Data.Attestations)
		assert.Equal(t, "48", resp.Data.SyncAggregate)
		assert.Equal(t, "62500000", resp.Data.AttesterSlashings)
		assert.Equal(t, "62500000", resp.Data.ProposerSlashings)
		assert.Equal(t, true, resp.ExecutionOptimistic)
		assert.Equal(t, false, resp.Finalized)
	})
	t.Run("deneb", func(t *testing.T) {
		st, sbb, err := BlockRewardTestSetup(t, "deneb")
		require.NoError(t, err)

		mockChainService := &mock.ChainService{Optimistic: true}
		s := &Server{
			Blocker: &testutil.MockBlocker{SlotBlockMap: map[primitives.Slot]interfaces.ReadOnlySignedBeaconBlock{
				0: phase0block,
				2: sbb,
			}},
			OptimisticModeFetcher: mockChainService,
			FinalizationFetcher:   mockChainService,
			BlockRewardFetcher:    &BlockRewardService{Replayer: mockstategen.NewReplayerBuilder(mockstategen.WithMockState(st))},
		}

		url := "http://only.the.slot.number.at.the.end.is.important/2"
		request := httptest.NewRequest("GET", url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.BlockRewards(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &BlockRewardsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, "12", resp.Data.ProposerIndex)
		assert.Equal(t, "125089490", resp.Data.Total)
		assert.Equal(t, "89442", resp.Data.Attestations)
		assert.Equal(t, "48", resp.Data.SyncAggregate)
		assert.Equal(t, "62500000", resp.Data.AttesterSlashings)
		assert.Equal(t, "62500000", resp.Data.ProposerSlashings)
		assert.Equal(t, true, resp.ExecutionOptimistic)
		assert.Equal(t, false, resp.Finalized)
	})
}

func TestAttestationRewards(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	cfg := params.BeaconConfig()
	cfg.AltairForkEpoch = 1
	params.OverrideBeaconConfig(cfg)
	helpers.ClearCache()

	valCount := 64

	st, err := util.NewBeaconStateCapella()
	require.NoError(t, err)
	require.NoError(t, st.SetSlot(params.BeaconConfig().SlotsPerEpoch*3-1))
	validators := make([]*eth.Validator, 0, valCount)
	balances := make([]uint64, 0, valCount)
	secretKeys := make([]bls.SecretKey, 0, valCount)
	for i := 0; i < valCount; i++ {
		blsKey, err := bls.RandKey()
		require.NoError(t, err)
		secretKeys = append(secretKeys, blsKey)
		validators = append(validators, &eth.Validator{
			PublicKey:         blsKey.PublicKey().Marshal(),
			ExitEpoch:         params.BeaconConfig().FarFutureEpoch,
			WithdrawableEpoch: params.BeaconConfig().FarFutureEpoch,
			EffectiveBalance:  params.BeaconConfig().MaxEffectiveBalance / 64 * uint64(i+1),
		})
		balances = append(balances, params.BeaconConfig().MaxEffectiveBalance/64*uint64(i+1))
	}
	require.NoError(t, st.SetValidators(validators))
	require.NoError(t, st.SetBalances(balances))
	require.NoError(t, st.SetInactivityScores(make([]uint64, len(validators))))
	participation := make([]byte, len(validators))
	for i := range participation {
		participation[i] = 0b111
	}
	require.NoError(t, st.SetCurrentParticipationBits(participation))
	require.NoError(t, st.SetPreviousParticipationBits(participation))

	currentSlot := params.BeaconConfig().SlotsPerEpoch * 3
	mockChainService := &mock.ChainService{Optimistic: true, Slot: &currentSlot}
	s := &Server{
		Stater: &testutil.MockStater{StatesBySlot: map[primitives.Slot]state.BeaconState{
			params.BeaconConfig().SlotsPerEpoch*3 - 1: st,
		}},
		TimeFetcher:           mockChainService,
		OptimisticModeFetcher: mockChainService,
		FinalizationFetcher:   mockChainService,
	}

	t.Run("ideal rewards", func(t *testing.T) {
		url := "http://only.the.epoch.number.at.the.end.is.important/1"
		request := httptest.NewRequest("POST", url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.AttestationRewards(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &AttestationRewardsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 16, len(resp.Data.IdealRewards))
		sum := uint64(0)
		for _, r := range resp.Data.IdealRewards {
			hr, err := strconv.ParseUint(r.Head, 10, 64)
			require.NoError(t, err)
			sr, err := strconv.ParseUint(r.Source, 10, 64)
			require.NoError(t, err)
			tr, err := strconv.ParseUint(r.Target, 10, 64)
			require.NoError(t, err)
			sum += hr + sr + tr
		}
		assert.Equal(t, uint64(20756849), sum)
	})
	t.Run("filtered vals", func(t *testing.T) {
		url := "http://only.the.epoch.number.at.the.end.is.important/1"
		var body bytes.Buffer
		pubkey := fmt.Sprintf("%#x", secretKeys[10].PublicKey().Marshal())
		valIds, err := json.Marshal([]string{"20", pubkey})
		require.NoError(t, err)
		_, err = body.Write(valIds)
		require.NoError(t, err)
		request := httptest.NewRequest("POST", url, &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.AttestationRewards(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &AttestationRewardsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 2, len(resp.Data.TotalRewards))
		sum := uint64(0)
		for _, r := range resp.Data.TotalRewards {
			hr, err := strconv.ParseUint(r.Head, 10, 64)
			require.NoError(t, err)
			sr, err := strconv.ParseUint(r.Source, 10, 64)
			require.NoError(t, err)
			tr, err := strconv.ParseUint(r.Target, 10, 64)
			require.NoError(t, err)
			sum += hr + sr + tr
		}
		assert.Equal(t, uint64(794265), sum)
	})
	t.Run("all vals", func(t *testing.T) {
		url := "http://only.the.epoch.number.at.the.end.is.important/1"
		request := httptest.NewRequest("POST", url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.AttestationRewards(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &AttestationRewardsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 64, len(resp.Data.TotalRewards))
		sum := uint64(0)
		for _, r := range resp.Data.TotalRewards {
			hr, err := strconv.ParseUint(r.Head, 10, 64)
			require.NoError(t, err)
			sr, err := strconv.ParseUint(r.Source, 10, 64)
			require.NoError(t, err)
			tr, err := strconv.ParseUint(r.Target, 10, 64)
			require.NoError(t, err)
			sum += hr + sr + tr
		}
		assert.Equal(t, uint64(54221955), sum)
	})
	t.Run("penalty", func(t *testing.T) {
		st := st.Copy()
		validators := st.Validators()
		validators[63].Slashed = true
		require.NoError(t, st.SetValidators(validators))

		s := &Server{
			Stater: &testutil.MockStater{StatesBySlot: map[primitives.Slot]state.BeaconState{
				params.BeaconConfig().SlotsPerEpoch*3 - 1: st,
			}},
			TimeFetcher:           mockChainService,
			OptimisticModeFetcher: mockChainService,
			FinalizationFetcher:   mockChainService,
		}

		url := "http://only.the.epoch.number.at.the.end.is.important/1"
		var body bytes.Buffer
		valIds, err := json.Marshal([]string{"63"})
		require.NoError(t, err)
		_, err = body.Write(valIds)
		require.NoError(t, err)
		request := httptest.NewRequest("POST", url, &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.AttestationRewards(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &AttestationRewardsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, "0", resp.Data.TotalRewards[0].Head)
		assert.Equal(t, "-439299", resp.Data.TotalRewards[0].Source)
		assert.Equal(t, "-815841", resp.Data.TotalRewards[0].Target)
		assert.Equal(t, "0", resp.Data.TotalRewards[0].Inactivity)
	})
	t.Run("inactivity", func(t *testing.T) {
		st := st.Copy()
		validators := st.Validators()
		validators[63].Slashed = true
		require.NoError(t, st.SetValidators(validators))
		inactivityScores, err := st.InactivityScores()
		require.NoError(t, err)
		inactivityScores[63] = 10
		require.NoError(t, st.SetInactivityScores(inactivityScores))

		s := &Server{
			Stater: &testutil.MockStater{StatesBySlot: map[primitives.Slot]state.BeaconState{
				params.BeaconConfig().SlotsPerEpoch*3 - 1: st,
			}},
			TimeFetcher:           mockChainService,
			OptimisticModeFetcher: mockChainService,
			FinalizationFetcher:   mockChainService,
		}

		url := "http://only.the.epoch.number.at.the.end.is.important/1"
		var body bytes.Buffer
		valIds, err := json.Marshal([]string{"63"})
		require.NoError(t, err)
		_, err = body.Write(valIds)
		require.NoError(t, err)
		request := httptest.NewRequest("POST", url, &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.AttestationRewards(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &AttestationRewardsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, "-4768", resp.Data.TotalRewards[0].Inactivity)
	})
	t.Run("invalid validator index/pubkey", func(t *testing.T) {
		url := "http://only.the.epoch.number.at.the.end.is.important/1"
		var body bytes.Buffer
		valIds, err := json.Marshal([]string{"10", "foo"})
		require.NoError(t, err)
		_, err = body.Write(valIds)
		require.NoError(t, err)
		request := httptest.NewRequest("POST", url, &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.AttestationRewards(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, "foo is not a validator index or pubkey", e.Message)
	})
	t.Run("unknown validator pubkey", func(t *testing.T) {
		url := "http://only.the.epoch.number.at.the.end.is.important/1"
		var body bytes.Buffer
		privkey, err := bls.RandKey()
		require.NoError(t, err)
		pubkey := fmt.Sprintf("%#x", privkey.PublicKey().Marshal())
		valIds, err := json.Marshal([]string{"10", pubkey})
		require.NoError(t, err)
		_, err = body.Write(valIds)
		require.NoError(t, err)
		request := httptest.NewRequest("POST", url, &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.AttestationRewards(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, "No validator index found for pubkey "+pubkey, e.Message)
	})
	t.Run("validator index too large", func(t *testing.T) {
		url := "http://only.the.epoch.number.at.the.end.is.important/1"
		var body bytes.Buffer
		valIds, err := json.Marshal([]string{"10", "999"})
		require.NoError(t, err)
		_, err = body.Write(valIds)
		require.NoError(t, err)
		request := httptest.NewRequest("POST", url, &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.AttestationRewards(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, "Validator index 999 is too large. Maximum allowed index is 63", e.Message)
	})
	t.Run("phase 0", func(t *testing.T) {
		url := "http://only.the.epoch.number.at.the.end.is.important/0"
		request := httptest.NewRequest("POST", url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.AttestationRewards(writer, request)
		assert.Equal(t, http.StatusNotFound, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusNotFound, e.Code)
		assert.Equal(t, "Attestation rewards are not supported for Phase 0", e.Message)
	})
	t.Run("invalid epoch", func(t *testing.T) {
		url := "http://only.the.epoch.number.at.the.end.is.important/foo"
		request := httptest.NewRequest("POST", url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.AttestationRewards(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "Could not decode epoch"))
	})
	t.Run("previous epoch", func(t *testing.T) {
		url := "http://only.the.epoch.number.at.the.end.is.important/2"
		request := httptest.NewRequest("POST", url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.AttestationRewards(writer, request)
		assert.Equal(t, http.StatusNotFound, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusNotFound, e.Code)
		assert.Equal(t, "Attestation rewards are available after two epoch transitions to ensure all attestations have a chance of inclusion", e.Message)
	})
}

func TestSyncCommiteeRewards(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	cfg := params.BeaconConfig()
	cfg.AltairForkEpoch = 1
	params.OverrideBeaconConfig(cfg)
	helpers.ClearCache()

	const valCount = 1024
	// we have to set the proposer index to the value that will be randomly chosen (fortunately it's deterministic)
	const proposerIndex = 84

	st, err := util.NewBeaconStateCapella()
	require.NoError(t, err)
	require.NoError(t, st.SetSlot(params.BeaconConfig().SlotsPerEpoch-1))
	validators := make([]*eth.Validator, 0, valCount)
	secretKeys := make([]bls.SecretKey, 0, valCount)
	for i := 0; i < valCount; i++ {
		blsKey, err := bls.RandKey()
		require.NoError(t, err)
		secretKeys = append(secretKeys, blsKey)
		validators = append(validators, &eth.Validator{
			PublicKey:         blsKey.PublicKey().Marshal(),
			ExitEpoch:         params.BeaconConfig().FarFutureEpoch,
			WithdrawableEpoch: params.BeaconConfig().FarFutureEpoch,
			EffectiveBalance:  params.BeaconConfig().MaxEffectiveBalance,
		})
	}
	require.NoError(t, st.SetValidators(validators))
	require.NoError(t, st.SetInactivityScores(make([]uint64, len(validators))))
	syncCommitteePubkeys := make([][]byte, fieldparams.SyncCommitteeLength)
	for i := 0; i < fieldparams.SyncCommitteeLength; i++ {
		syncCommitteePubkeys[i] = secretKeys[i].PublicKey().Marshal()
	}
	aggPubkey, err := bls.AggregatePublicKeys(syncCommitteePubkeys)
	require.NoError(t, err)
	require.NoError(t, st.SetCurrentSyncCommittee(&eth.SyncCommittee{
		Pubkeys:         syncCommitteePubkeys,
		AggregatePubkey: aggPubkey.Marshal(),
	}))

	b := util.HydrateSignedBeaconBlockCapella(util.NewBeaconBlockCapella())
	b.Block.Slot = 32
	b.Block.ProposerIndex = proposerIndex
	scBits := bitfield.NewBitvector512()
	// last 10 sync committee members didn't perform their duty
	for i := uint64(0); i < fieldparams.SyncCommitteeLength-10; i++ {
		scBits.SetBitAt(i, true)
	}
	domain, err := signing.Domain(st.Fork(), 0, params.BeaconConfig().DomainSyncCommittee, st.GenesisValidatorsRoot())
	require.NoError(t, err)
	sszBytes := primitives.SSZBytes("")
	r, err := signing.ComputeSigningRoot(&sszBytes, domain)
	require.NoError(t, err)
	// Bits set in sync committee bits determine which validators will be treated as participating in sync committee.
	// These validators have to sign the message.
	sigs := make([]bls.Signature, fieldparams.SyncCommitteeLength-10)
	for i := range sigs {
		sigs[i], err = blst.SignatureFromBytes(secretKeys[i].Sign(r[:]).Marshal())
		require.NoError(t, err)
	}
	aggSig := bls.AggregateSignatures(sigs).Marshal()
	b.Block.Body.SyncAggregate = &eth.SyncAggregate{SyncCommitteeBits: scBits, SyncCommitteeSignature: aggSig}
	sbb, err := blocks.NewSignedBeaconBlock(b)
	require.NoError(t, err)
	phase0block, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlock())
	require.NoError(t, err)

	currentSlot := params.BeaconConfig().SlotsPerEpoch
	mockChainService := &mock.ChainService{Optimistic: true, Slot: &currentSlot}
	s := &Server{
		Blocker: &testutil.MockBlocker{SlotBlockMap: map[primitives.Slot]interfaces.ReadOnlySignedBeaconBlock{
			0:  phase0block,
			32: sbb,
		}},
		OptimisticModeFetcher: mockChainService,
		FinalizationFetcher:   mockChainService,
		BlockRewardFetcher:    &BlockRewardService{Replayer: mockstategen.NewReplayerBuilder(mockstategen.WithMockState(st))},
	}

	t.Run("ok - filtered vals", func(t *testing.T) {
		balances := make([]uint64, 0, valCount)
		for i := 0; i < valCount; i++ {
			balances = append(balances, params.BeaconConfig().MaxEffectiveBalance)
		}
		require.NoError(t, st.SetBalances(balances))

		url := "http://only.the.slot.number.at.the.end.is.important/32"
		var body bytes.Buffer
		pubkey := fmt.Sprintf("%#x", secretKeys[10].PublicKey().Marshal())
		valIds, err := json.Marshal([]string{"20", pubkey})
		require.NoError(t, err)
		_, err = body.Write(valIds)
		require.NoError(t, err)
		request := httptest.NewRequest("POST", url, &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SyncCommitteeRewards(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &SyncCommitteeRewardsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 2, len(resp.Data))
		sum := uint64(0)
		for _, scReward := range resp.Data {
			r, err := strconv.ParseUint(scReward.Reward, 10, 64)
			require.NoError(t, err)
			sum += r
		}
		assert.Equal(t, uint64(1396), sum)
		assert.Equal(t, true, resp.ExecutionOptimistic)
		assert.Equal(t, false, resp.Finalized)
	})
	t.Run("ok - all vals", func(t *testing.T) {
		balances := make([]uint64, 0, valCount)
		for i := 0; i < valCount; i++ {
			balances = append(balances, params.BeaconConfig().MaxEffectiveBalance)
		}
		require.NoError(t, st.SetBalances(balances))

		url := "http://only.the.slot.number.at.the.end.is.important/32"
		request := httptest.NewRequest("POST", url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SyncCommitteeRewards(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &SyncCommitteeRewardsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 512, len(resp.Data))
		sum := 0
		for _, scReward := range resp.Data {
			r, err := strconv.Atoi(scReward.Reward)
			require.NoError(t, err)
			sum += r
		}
		assert.Equal(t, 343416, sum)
	})
	t.Run("ok - validator outside sync committee is ignored", func(t *testing.T) {
		balances := make([]uint64, 0, valCount)
		for i := 0; i < valCount; i++ {
			balances = append(balances, params.BeaconConfig().MaxEffectiveBalance)
		}
		require.NoError(t, st.SetBalances(balances))

		url := "http://only.the.slot.number.at.the.end.is.important/32"
		var body bytes.Buffer
		pubkey := fmt.Sprintf("%#x", secretKeys[10].PublicKey().Marshal())
		valIds, err := json.Marshal([]string{"20", "999", pubkey})
		require.NoError(t, err)
		_, err = body.Write(valIds)
		require.NoError(t, err)
		request := httptest.NewRequest("POST", url, &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SyncCommitteeRewards(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &SyncCommitteeRewardsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 2, len(resp.Data))
		sum := 0
		for _, scReward := range resp.Data {
			r, err := strconv.Atoi(scReward.Reward)
			require.NoError(t, err)
			sum += r
		}
		assert.Equal(t, 1396, sum)
	})
	t.Run("ok - proposer reward is deducted", func(t *testing.T) {
		balances := make([]uint64, 0, valCount)
		for i := 0; i < valCount; i++ {
			balances = append(balances, params.BeaconConfig().MaxEffectiveBalance)
		}
		require.NoError(t, st.SetBalances(balances))

		url := "http://only.the.slot.number.at.the.end.is.important/32"
		var body bytes.Buffer
		pubkey := fmt.Sprintf("%#x", secretKeys[10].PublicKey().Marshal())
		valIds, err := json.Marshal([]string{"20", "84", pubkey})
		require.NoError(t, err)
		_, err = body.Write(valIds)
		require.NoError(t, err)
		request := httptest.NewRequest("POST", url, &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SyncCommitteeRewards(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &SyncCommitteeRewardsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 3, len(resp.Data))
		sum := 0
		for _, scReward := range resp.Data {
			r, err := strconv.Atoi(scReward.Reward)
			require.NoError(t, err)
			sum += r
		}
		assert.Equal(t, 2094, sum)
	})
	t.Run("invalid validator index/pubkey", func(t *testing.T) {
		balances := make([]uint64, 0, valCount)
		for i := 0; i < valCount; i++ {
			balances = append(balances, params.BeaconConfig().MaxEffectiveBalance)
		}
		require.NoError(t, st.SetBalances(balances))

		url := "http://only.the.slot.number.at.the.end.is.important/32"
		var body bytes.Buffer
		valIds, err := json.Marshal([]string{"10", "foo"})
		require.NoError(t, err)
		_, err = body.Write(valIds)
		require.NoError(t, err)
		request := httptest.NewRequest("POST", url, &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SyncCommitteeRewards(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, "foo is not a validator index or pubkey", e.Message)
	})
	t.Run("unknown validator pubkey", func(t *testing.T) {
		balances := make([]uint64, 0, valCount)
		for i := 0; i < valCount; i++ {
			balances = append(balances, params.BeaconConfig().MaxEffectiveBalance)
		}
		require.NoError(t, st.SetBalances(balances))

		url := "http://only.the.slot.number.at.the.end.is.important/32"
		var body bytes.Buffer
		privkey, err := bls.RandKey()
		require.NoError(t, err)
		pubkey := fmt.Sprintf("%#x", privkey.PublicKey().Marshal())
		valIds, err := json.Marshal([]string{"10", pubkey})
		require.NoError(t, err)
		_, err = body.Write(valIds)
		require.NoError(t, err)
		request := httptest.NewRequest("POST", url, &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SyncCommitteeRewards(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, "No validator index found for pubkey "+pubkey, e.Message)
	})
	t.Run("validator index too large", func(t *testing.T) {
		balances := make([]uint64, 0, valCount)
		for i := 0; i < valCount; i++ {
			balances = append(balances, params.BeaconConfig().MaxEffectiveBalance)
		}
		require.NoError(t, st.SetBalances(balances))

		url := "http://only.the.slot.number.at.the.end.is.important/32"
		var body bytes.Buffer
		valIds, err := json.Marshal([]string{"10", "9999"})
		require.NoError(t, err)
		_, err = body.Write(valIds)
		require.NoError(t, err)
		request := httptest.NewRequest("POST", url, &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SyncCommitteeRewards(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, "Validator index 9999 is too large. Maximum allowed index is 1023", e.Message)
	})
	t.Run("phase 0", func(t *testing.T) {
		balances := make([]uint64, 0, valCount)
		for i := 0; i < valCount; i++ {
			balances = append(balances, params.BeaconConfig().MaxEffectiveBalance)
		}
		require.NoError(t, st.SetBalances(balances))

		url := "http://only.the.slot.number.at.the.end.is.important/0"
		request := httptest.NewRequest("POST", url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SyncCommitteeRewards(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, "Sync committee rewards are not supported for Phase 0", e.Message)
	})
}

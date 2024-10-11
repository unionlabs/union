package beacon

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/api/server"
	blockchainmock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	prysmtime "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/attestations"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/blstoexec"
	blstoexecmock "github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/blstoexec/mock"
	slashingsmock "github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/slashings/mock"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/synccommittee"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/voluntaryexits/mock"
	p2pMock "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/core"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls/common"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/encoding/ssz"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	ethpbv1alpha1 "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

func TestListAttestations(t *testing.T) {
	att1 := &ethpbv1alpha1.Attestation{
		AggregationBits: []byte{1, 10},
		Data: &ethpbv1alpha1.AttestationData{
			Slot:            1,
			CommitteeIndex:  1,
			BeaconBlockRoot: bytesutil.PadTo([]byte("blockroot1"), 32),
			Source: &ethpbv1alpha1.Checkpoint{
				Epoch: 1,
				Root:  bytesutil.PadTo([]byte("sourceroot1"), 32),
			},
			Target: &ethpbv1alpha1.Checkpoint{
				Epoch: 10,
				Root:  bytesutil.PadTo([]byte("targetroot1"), 32),
			},
		},
		Signature: bytesutil.PadTo([]byte("signature1"), 96),
	}
	att2 := &ethpbv1alpha1.Attestation{
		AggregationBits: []byte{1, 10},
		Data: &ethpbv1alpha1.AttestationData{
			Slot:            1,
			CommitteeIndex:  4,
			BeaconBlockRoot: bytesutil.PadTo([]byte("blockroot2"), 32),
			Source: &ethpbv1alpha1.Checkpoint{
				Epoch: 1,
				Root:  bytesutil.PadTo([]byte("sourceroot2"), 32),
			},
			Target: &ethpbv1alpha1.Checkpoint{
				Epoch: 10,
				Root:  bytesutil.PadTo([]byte("targetroot2"), 32),
			},
		},
		Signature: bytesutil.PadTo([]byte("signature2"), 96),
	}
	att3 := &ethpbv1alpha1.Attestation{
		AggregationBits: bitfield.NewBitlist(8),
		Data: &ethpbv1alpha1.AttestationData{
			Slot:            2,
			CommitteeIndex:  2,
			BeaconBlockRoot: bytesutil.PadTo([]byte("blockroot3"), 32),
			Source: &ethpbv1alpha1.Checkpoint{
				Epoch: 1,
				Root:  bytesutil.PadTo([]byte("sourceroot3"), 32),
			},
			Target: &ethpbv1alpha1.Checkpoint{
				Epoch: 10,
				Root:  bytesutil.PadTo([]byte("targetroot3"), 32),
			},
		},
		Signature: bytesutil.PadTo([]byte("signature3"), 96),
	}
	att4 := &ethpbv1alpha1.Attestation{
		AggregationBits: bitfield.NewBitlist(8),
		Data: &ethpbv1alpha1.AttestationData{
			Slot:            2,
			CommitteeIndex:  4,
			BeaconBlockRoot: bytesutil.PadTo([]byte("blockroot4"), 32),
			Source: &ethpbv1alpha1.Checkpoint{
				Epoch: 1,
				Root:  bytesutil.PadTo([]byte("sourceroot4"), 32),
			},
			Target: &ethpbv1alpha1.Checkpoint{
				Epoch: 10,
				Root:  bytesutil.PadTo([]byte("targetroot4"), 32),
			},
		},
		Signature: bytesutil.PadTo([]byte("signature4"), 96),
	}
	s := &Server{
		AttestationsPool: attestations.NewPool(),
	}
	require.NoError(t, s.AttestationsPool.SaveAggregatedAttestations([]*ethpbv1alpha1.Attestation{att1, att2}))
	require.NoError(t, s.AttestationsPool.SaveUnaggregatedAttestations([]*ethpbv1alpha1.Attestation{att3, att4}))

	t.Run("empty request", func(t *testing.T) {
		url := "http://example.com"
		request := httptest.NewRequest(http.MethodGet, url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.ListAttestations(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &ListAttestationsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.NotNil(t, resp)
		require.NotNil(t, resp.Data)
		assert.Equal(t, 4, len(resp.Data))
	})
	t.Run("slot request", func(t *testing.T) {
		url := "http://example.com?slot=2"
		request := httptest.NewRequest(http.MethodGet, url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.ListAttestations(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &ListAttestationsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.NotNil(t, resp)
		require.NotNil(t, resp.Data)
		assert.Equal(t, 2, len(resp.Data))
		for _, a := range resp.Data {
			assert.Equal(t, "2", a.Data.Slot)
		}
	})
	t.Run("index request", func(t *testing.T) {
		url := "http://example.com?committee_index=4"
		request := httptest.NewRequest(http.MethodGet, url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.ListAttestations(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &ListAttestationsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.NotNil(t, resp)
		require.NotNil(t, resp.Data)
		assert.Equal(t, 2, len(resp.Data))
		for _, a := range resp.Data {
			assert.Equal(t, "4", a.Data.CommitteeIndex)
		}
	})
	t.Run("both slot + index request", func(t *testing.T) {
		url := "http://example.com?slot=2&committee_index=4"
		request := httptest.NewRequest(http.MethodGet, url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.ListAttestations(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &ListAttestationsResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.NotNil(t, resp)
		require.NotNil(t, resp.Data)
		assert.Equal(t, 1, len(resp.Data))
		for _, a := range resp.Data {
			assert.Equal(t, "2", a.Data.Slot)
			assert.Equal(t, "4", a.Data.CommitteeIndex)
		}
	})
}

func TestSubmitAttestations(t *testing.T) {
	transition.SkipSlotCache.Disable()
	defer transition.SkipSlotCache.Enable()

	params.SetupTestConfigCleanup(t)
	c := params.BeaconConfig().Copy()
	// Required for correct committee size calculation.
	c.SlotsPerEpoch = 1
	params.OverrideBeaconConfig(c)

	_, keys, err := util.DeterministicDepositsAndKeys(1)
	require.NoError(t, err)
	validators := []*ethpbv1alpha1.Validator{
		{
			PublicKey: keys[0].PublicKey().Marshal(),
			ExitEpoch: params.BeaconConfig().FarFutureEpoch,
		},
	}
	bs, err := util.NewBeaconState(func(state *ethpbv1alpha1.BeaconState) error {
		state.Validators = validators
		state.Slot = 1
		state.PreviousJustifiedCheckpoint = &ethpbv1alpha1.Checkpoint{
			Epoch: 0,
			Root:  bytesutil.PadTo([]byte("sourceroot1"), 32),
		}
		return nil
	})
	require.NoError(t, err)
	b := bitfield.NewBitlist(1)
	b.SetBitAt(0, true)

	chainService := &blockchainmock.ChainService{State: bs}
	s := &Server{
		HeadFetcher:       chainService,
		ChainInfoFetcher:  chainService,
		OperationNotifier: &blockchainmock.MockOperationNotifier{},
	}

	t.Run("single", func(t *testing.T) {
		broadcaster := &p2pMock.MockBroadcaster{}
		s.Broadcaster = broadcaster
		s.AttestationsPool = attestations.NewPool()

		var body bytes.Buffer
		_, err := body.WriteString(singleAtt)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitAttestations(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		assert.Equal(t, true, broadcaster.BroadcastCalled.Load())
		assert.Equal(t, 1, broadcaster.NumAttestations())
		assert.Equal(t, "0x03", hexutil.Encode(broadcaster.BroadcastAttestations[0].AggregationBits))
		assert.Equal(t, "0x8146f4397bfd8fd057ebbcd6a67327bdc7ed5fb650533edcb6377b650dea0b6da64c14ecd60846d5c0a0cd43893d6972092500f82c9d8a955e2b58c5ed3cbe885d84008ace6bd86ba9e23652f58e2ec207cec494c916063257abf285b9b15b15", hexutil.Encode(broadcaster.BroadcastAttestations[0].Signature))
		assert.Equal(t, primitives.Slot(0), broadcaster.BroadcastAttestations[0].Data.Slot)
		assert.Equal(t, primitives.CommitteeIndex(0), broadcaster.BroadcastAttestations[0].Data.CommitteeIndex)
		assert.Equal(t, "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2", hexutil.Encode(broadcaster.BroadcastAttestations[0].Data.BeaconBlockRoot))
		assert.Equal(t, "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2", hexutil.Encode(broadcaster.BroadcastAttestations[0].Data.Source.Root))
		assert.Equal(t, primitives.Epoch(0), broadcaster.BroadcastAttestations[0].Data.Source.Epoch)
		assert.Equal(t, "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2", hexutil.Encode(broadcaster.BroadcastAttestations[0].Data.Target.Root))
		assert.Equal(t, primitives.Epoch(0), broadcaster.BroadcastAttestations[0].Data.Target.Epoch)
		assert.Equal(t, 1, s.AttestationsPool.UnaggregatedAttestationCount())
	})
	t.Run("multiple", func(t *testing.T) {
		broadcaster := &p2pMock.MockBroadcaster{}
		s.Broadcaster = broadcaster
		s.AttestationsPool = attestations.NewPool()

		var body bytes.Buffer
		_, err := body.WriteString(multipleAtts)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitAttestations(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		assert.Equal(t, true, broadcaster.BroadcastCalled.Load())
		assert.Equal(t, 2, broadcaster.NumAttestations())
		assert.Equal(t, 2, s.AttestationsPool.UnaggregatedAttestationCount())
	})
	t.Run("no body", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodPost, "http://example.com", nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitAttestations(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "No data submitted"))
	})
	t.Run("empty", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString("[]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitAttestations(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "No data submitted"))
	})
	t.Run("invalid", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString(invalidAtt)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitAttestations(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &server.IndexedVerificationFailureError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		require.Equal(t, 1, len(e.Failures))
		assert.Equal(t, true, strings.Contains(e.Failures[0].Message, "Incorrect attestation signature"))
	})
}

func TestListVoluntaryExits(t *testing.T) {
	exit1 := &ethpbv1alpha1.SignedVoluntaryExit{
		Exit: &ethpbv1alpha1.VoluntaryExit{
			Epoch:          1,
			ValidatorIndex: 1,
		},
		Signature: bytesutil.PadTo([]byte("signature1"), 96),
	}
	exit2 := &ethpbv1alpha1.SignedVoluntaryExit{
		Exit: &ethpbv1alpha1.VoluntaryExit{
			Epoch:          2,
			ValidatorIndex: 2,
		},
		Signature: bytesutil.PadTo([]byte("signature2"), 96),
	}

	s := &Server{
		VoluntaryExitsPool: &mock.PoolMock{Exits: []*ethpbv1alpha1.SignedVoluntaryExit{exit1, exit2}},
	}

	request := httptest.NewRequest(http.MethodGet, "http://example.com", nil)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.ListVoluntaryExits(writer, request)
	assert.Equal(t, http.StatusOK, writer.Code)
	resp := &ListVoluntaryExitsResponse{}
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
	require.NotNil(t, resp)
	require.NotNil(t, resp.Data)
	require.Equal(t, 2, len(resp.Data))
	assert.Equal(t, "0x7369676e6174757265310000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000", resp.Data[0].Signature)
	assert.Equal(t, "1", resp.Data[0].Message.Epoch)
	assert.Equal(t, "1", resp.Data[0].Message.ValidatorIndex)
	assert.Equal(t, "0x7369676e6174757265320000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000", resp.Data[1].Signature)
	assert.Equal(t, "2", resp.Data[1].Message.Epoch)
	assert.Equal(t, "2", resp.Data[1].Message.ValidatorIndex)
}

func TestSubmitVoluntaryExit(t *testing.T) {
	transition.SkipSlotCache.Disable()
	defer transition.SkipSlotCache.Enable()

	t.Run("ok", func(t *testing.T) {
		_, keys, err := util.DeterministicDepositsAndKeys(1)
		require.NoError(t, err)
		validator := &ethpbv1alpha1.Validator{
			ExitEpoch: params.BeaconConfig().FarFutureEpoch,
			PublicKey: keys[0].PublicKey().Marshal(),
		}
		bs, err := util.NewBeaconState(func(state *ethpbv1alpha1.BeaconState) error {
			state.Validators = []*ethpbv1alpha1.Validator{validator}
			// Satisfy activity time required before exiting.
			state.Slot = params.BeaconConfig().SlotsPerEpoch.Mul(uint64(params.BeaconConfig().ShardCommitteePeriod))
			return nil
		})
		require.NoError(t, err)

		broadcaster := &p2pMock.MockBroadcaster{}
		s := &Server{
			ChainInfoFetcher:   &blockchainmock.ChainService{State: bs},
			VoluntaryExitsPool: &mock.PoolMock{},
			Broadcaster:        broadcaster,
		}

		var body bytes.Buffer
		_, err = body.WriteString(exit1)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitVoluntaryExit(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		require.NoError(t, err)
		pendingExits, err := s.VoluntaryExitsPool.PendingExits()
		require.NoError(t, err)
		require.Equal(t, 1, len(pendingExits))
		assert.Equal(t, true, broadcaster.BroadcastCalled.Load())
	})
	t.Run("across fork", func(t *testing.T) {
		params.SetupTestConfigCleanup(t)
		config := params.BeaconConfig()
		config.AltairForkEpoch = params.BeaconConfig().ShardCommitteePeriod + 1
		params.OverrideBeaconConfig(config)

		bs, _ := util.DeterministicGenesisState(t, 1)
		// Satisfy activity time required before exiting.
		require.NoError(t, bs.SetSlot(params.BeaconConfig().SlotsPerEpoch.Mul(uint64(params.BeaconConfig().ShardCommitteePeriod))))

		broadcaster := &p2pMock.MockBroadcaster{}
		s := &Server{
			ChainInfoFetcher:   &blockchainmock.ChainService{State: bs},
			VoluntaryExitsPool: &mock.PoolMock{},
			Broadcaster:        broadcaster,
		}

		var body bytes.Buffer
		_, err := body.WriteString(exit2)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitVoluntaryExit(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		require.NoError(t, err)
		pendingExits, err := s.VoluntaryExitsPool.PendingExits()
		require.NoError(t, err)
		require.Equal(t, 1, len(pendingExits))
		assert.Equal(t, true, broadcaster.BroadcastCalled.Load())
	})
	t.Run("no body", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodPost, "http://example.com", nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s := &Server{}
		s.SubmitVoluntaryExit(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "No data submitted"))
	})
	t.Run("invalid", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString(invalidExit1)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s := &Server{}
		s.SubmitVoluntaryExit(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
	})
	t.Run("wrong signature", func(t *testing.T) {
		bs, _ := util.DeterministicGenesisState(t, 1)
		s := &Server{ChainInfoFetcher: &blockchainmock.ChainService{State: bs}}

		var body bytes.Buffer
		_, err := body.WriteString(invalidExit2)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitVoluntaryExit(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "Invalid exit"))
	})
	t.Run("invalid validator index", func(t *testing.T) {
		_, keys, err := util.DeterministicDepositsAndKeys(1)
		require.NoError(t, err)
		validator := &ethpbv1alpha1.Validator{
			ExitEpoch: params.BeaconConfig().FarFutureEpoch,
			PublicKey: keys[0].PublicKey().Marshal(),
		}
		bs, err := util.NewBeaconState(func(state *ethpbv1alpha1.BeaconState) error {
			state.Validators = []*ethpbv1alpha1.Validator{validator}
			return nil
		})
		require.NoError(t, err)

		s := &Server{ChainInfoFetcher: &blockchainmock.ChainService{State: bs}}

		var body bytes.Buffer
		_, err = body.WriteString(invalidExit3)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitVoluntaryExit(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "Could not get validator"))
	})
}

func TestSubmitSyncCommitteeSignatures(t *testing.T) {
	st, _ := util.DeterministicGenesisStateAltair(t, 10)

	t.Run("single", func(t *testing.T) {
		broadcaster := &p2pMock.MockBroadcaster{}
		s := &Server{
			CoreService: &core.Service{
				SyncCommitteePool: synccommittee.NewStore(),
				P2P:               broadcaster,
				HeadFetcher: &blockchainmock.ChainService{
					State:                st,
					SyncCommitteeIndices: []primitives.CommitteeIndex{0},
				},
			},
		}

		var body bytes.Buffer
		_, err := body.WriteString(singleSyncCommitteeMsg)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitSyncCommitteeSignatures(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		require.NoError(t, err)
		msgsInPool, err := s.CoreService.SyncCommitteePool.SyncCommitteeMessages(1)
		require.NoError(t, err)
		require.Equal(t, 1, len(msgsInPool))
		assert.Equal(t, primitives.Slot(1), msgsInPool[0].Slot)
		assert.Equal(t, "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2", hexutil.Encode(msgsInPool[0].BlockRoot))
		assert.Equal(t, primitives.ValidatorIndex(1), msgsInPool[0].ValidatorIndex)
		assert.Equal(t, "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505", hexutil.Encode(msgsInPool[0].Signature))
		assert.Equal(t, true, broadcaster.BroadcastCalled.Load())
	})
	t.Run("multiple", func(t *testing.T) {
		broadcaster := &p2pMock.MockBroadcaster{}
		s := &Server{
			CoreService: &core.Service{
				SyncCommitteePool: synccommittee.NewStore(),
				P2P:               broadcaster,
				HeadFetcher: &blockchainmock.ChainService{
					State:                st,
					SyncCommitteeIndices: []primitives.CommitteeIndex{0},
				},
			},
		}

		var body bytes.Buffer
		_, err := body.WriteString(multipleSyncCommitteeMsg)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitSyncCommitteeSignatures(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		require.NoError(t, err)
		msgsInPool, err := s.CoreService.SyncCommitteePool.SyncCommitteeMessages(1)
		require.NoError(t, err)
		require.Equal(t, 1, len(msgsInPool))
		msgsInPool, err = s.CoreService.SyncCommitteePool.SyncCommitteeMessages(2)
		require.NoError(t, err)
		require.Equal(t, 1, len(msgsInPool))
		assert.Equal(t, true, broadcaster.BroadcastCalled.Load())
	})
	t.Run("invalid", func(t *testing.T) {
		broadcaster := &p2pMock.MockBroadcaster{}
		s := &Server{
			CoreService: &core.Service{
				SyncCommitteePool: synccommittee.NewStore(),
				P2P:               broadcaster,
				HeadFetcher: &blockchainmock.ChainService{
					State: st,
				},
			},
		}

		var body bytes.Buffer
		_, err := body.WriteString(invalidSyncCommitteeMsg)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitSyncCommitteeSignatures(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		require.NoError(t, err)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		msgsInPool, err := s.CoreService.SyncCommitteePool.SyncCommitteeMessages(1)
		require.NoError(t, err)
		assert.Equal(t, 0, len(msgsInPool))
		assert.Equal(t, false, broadcaster.BroadcastCalled.Load())
	})
	t.Run("empty", func(t *testing.T) {
		s := &Server{}

		var body bytes.Buffer
		_, err := body.WriteString("[]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitSyncCommitteeSignatures(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "No data submitted"))
	})
	t.Run("no body", func(t *testing.T) {
		s := &Server{}

		request := httptest.NewRequest(http.MethodPost, "http://example.com", nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitSyncCommitteeSignatures(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "No data submitted"))
	})
}

func TestListBLSToExecutionChanges(t *testing.T) {
	change1 := &ethpbv1alpha1.SignedBLSToExecutionChange{
		Message: &ethpbv1alpha1.BLSToExecutionChange{
			ValidatorIndex:     1,
			FromBlsPubkey:      bytesutil.PadTo([]byte("pubkey1"), 48),
			ToExecutionAddress: bytesutil.PadTo([]byte("address1"), 20),
		},
		Signature: bytesutil.PadTo([]byte("signature1"), 96),
	}
	change2 := &ethpbv1alpha1.SignedBLSToExecutionChange{
		Message: &ethpbv1alpha1.BLSToExecutionChange{
			ValidatorIndex:     2,
			FromBlsPubkey:      bytesutil.PadTo([]byte("pubkey2"), 48),
			ToExecutionAddress: bytesutil.PadTo([]byte("address2"), 20),
		},
		Signature: bytesutil.PadTo([]byte("signature2"), 96),
	}

	s := &Server{
		BLSChangesPool: &blstoexecmock.PoolMock{Changes: []*ethpbv1alpha1.SignedBLSToExecutionChange{change1, change2}},
	}
	request := httptest.NewRequest(http.MethodGet, "http://foo.example/eth/v1/beacon/pool/bls_to_execution_changes", nil)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.ListBLSToExecutionChanges(writer, request)
	assert.Equal(t, http.StatusOK, writer.Code)

	resp := &BLSToExecutionChangesPoolResponse{}
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
	require.Equal(t, 2, len(resp.Data))
	assert.DeepEqual(t, shared.SignedBLSChangeFromConsensus(change1), resp.Data[0])
	assert.DeepEqual(t, shared.SignedBLSChangeFromConsensus(change2), resp.Data[1])
}

func TestSubmitSignedBLSToExecutionChanges_Ok(t *testing.T) {
	transition.SkipSlotCache.Disable()
	defer transition.SkipSlotCache.Enable()

	params.SetupTestConfigCleanup(t)
	c := params.BeaconConfig().Copy()
	// Required for correct committee size calculation.
	c.CapellaForkEpoch = c.BellatrixForkEpoch.Add(2)
	params.OverrideBeaconConfig(c)

	spb := &ethpbv1alpha1.BeaconStateCapella{
		Fork: &ethpbv1alpha1.Fork{
			CurrentVersion:  params.BeaconConfig().GenesisForkVersion,
			PreviousVersion: params.BeaconConfig().GenesisForkVersion,
			Epoch:           params.BeaconConfig().CapellaForkEpoch,
		},
	}
	numValidators := 10
	validators := make([]*ethpbv1alpha1.Validator, numValidators)
	blsChanges := make([]*ethpbv1alpha1.BLSToExecutionChange, numValidators)
	spb.Balances = make([]uint64, numValidators)
	privKeys := make([]common.SecretKey, numValidators)
	maxEffectiveBalance := params.BeaconConfig().MaxEffectiveBalance
	executionAddress := []byte{0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13}

	for i := range validators {
		v := &ethpbv1alpha1.Validator{}
		v.EffectiveBalance = maxEffectiveBalance
		v.WithdrawableEpoch = params.BeaconConfig().FarFutureEpoch
		v.WithdrawalCredentials = make([]byte, 32)
		priv, err := bls.RandKey()
		require.NoError(t, err)
		privKeys[i] = priv
		pubkey := priv.PublicKey().Marshal()

		message := &ethpbv1alpha1.BLSToExecutionChange{
			ToExecutionAddress: executionAddress,
			ValidatorIndex:     primitives.ValidatorIndex(i),
			FromBlsPubkey:      pubkey,
		}

		hashFn := ssz.NewHasherFunc(hash.CustomSHA256Hasher())
		digest := hashFn.Hash(pubkey)
		digest[0] = params.BeaconConfig().BLSWithdrawalPrefixByte
		copy(v.WithdrawalCredentials, digest[:])
		validators[i] = v
		blsChanges[i] = message
	}
	spb.Validators = validators
	slot, err := slots.EpochStart(params.BeaconConfig().CapellaForkEpoch)
	require.NoError(t, err)
	spb.Slot = slot
	st, err := state_native.InitializeFromProtoCapella(spb)
	require.NoError(t, err)

	signedChanges := make([]*shared.SignedBLSToExecutionChange, numValidators)
	for i, message := range blsChanges {
		signature, err := signing.ComputeDomainAndSign(st, prysmtime.CurrentEpoch(st), message, params.BeaconConfig().DomainBLSToExecutionChange, privKeys[i])
		require.NoError(t, err)
		signed := &shared.SignedBLSToExecutionChange{
			Message:   shared.BLSChangeFromConsensus(message),
			Signature: hexutil.Encode(signature),
		}
		signedChanges[i] = signed
	}

	broadcaster := &p2pMock.MockBroadcaster{}
	chainService := &blockchainmock.ChainService{State: st}
	s := &Server{
		HeadFetcher:       chainService,
		ChainInfoFetcher:  chainService,
		AttestationsPool:  attestations.NewPool(),
		Broadcaster:       broadcaster,
		OperationNotifier: &blockchainmock.MockOperationNotifier{},
		BLSChangesPool:    blstoexec.NewPool(),
	}
	jsonBytes, err := json.Marshal(signedChanges)
	require.NoError(t, err)

	request := httptest.NewRequest(http.MethodPost, "http://foo.example/eth/v1/beacon/pool/bls_to_execution_changes", bytes.NewReader(jsonBytes))
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}
	s.SubmitBLSToExecutionChanges(writer, request)
	assert.Equal(t, http.StatusOK, writer.Code)
	time.Sleep(100 * time.Millisecond) // Delay to let the routine start
	assert.Equal(t, true, broadcaster.BroadcastCalled.Load())
	assert.Equal(t, numValidators, len(broadcaster.BroadcastMessages))

	poolChanges, err := s.BLSChangesPool.PendingBLSToExecChanges()
	require.Equal(t, len(poolChanges), len(signedChanges))
	require.NoError(t, err)
	for i, v1alphaChange := range poolChanges {
		sc, err := signedChanges[i].ToConsensus()
		require.NoError(t, err)
		require.DeepEqual(t, v1alphaChange, sc)
	}
}

func TestSubmitSignedBLSToExecutionChanges_Bellatrix(t *testing.T) {
	transition.SkipSlotCache.Disable()
	defer transition.SkipSlotCache.Enable()

	params.SetupTestConfigCleanup(t)
	c := params.BeaconConfig().Copy()
	// Required for correct committee size calculation.
	c.CapellaForkEpoch = c.BellatrixForkEpoch.Add(2)
	params.OverrideBeaconConfig(c)

	spb := &ethpbv1alpha1.BeaconStateBellatrix{
		Fork: &ethpbv1alpha1.Fork{
			CurrentVersion:  params.BeaconConfig().BellatrixForkVersion,
			PreviousVersion: params.BeaconConfig().AltairForkVersion,
			Epoch:           params.BeaconConfig().BellatrixForkEpoch,
		},
	}
	numValidators := 10
	validators := make([]*ethpbv1alpha1.Validator, numValidators)
	blsChanges := make([]*ethpbv1alpha1.BLSToExecutionChange, numValidators)
	spb.Balances = make([]uint64, numValidators)
	privKeys := make([]common.SecretKey, numValidators)
	maxEffectiveBalance := params.BeaconConfig().MaxEffectiveBalance
	executionAddress := []byte{0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13}

	for i := range validators {
		v := &ethpbv1alpha1.Validator{}
		v.EffectiveBalance = maxEffectiveBalance
		v.WithdrawableEpoch = params.BeaconConfig().FarFutureEpoch
		v.WithdrawalCredentials = make([]byte, 32)
		priv, err := bls.RandKey()
		require.NoError(t, err)
		privKeys[i] = priv
		pubkey := priv.PublicKey().Marshal()

		message := &ethpbv1alpha1.BLSToExecutionChange{
			ToExecutionAddress: executionAddress,
			ValidatorIndex:     primitives.ValidatorIndex(i),
			FromBlsPubkey:      pubkey,
		}

		hashFn := ssz.NewHasherFunc(hash.CustomSHA256Hasher())
		digest := hashFn.Hash(pubkey)
		digest[0] = params.BeaconConfig().BLSWithdrawalPrefixByte
		copy(v.WithdrawalCredentials, digest[:])
		validators[i] = v
		blsChanges[i] = message
	}
	spb.Validators = validators
	slot, err := slots.EpochStart(params.BeaconConfig().BellatrixForkEpoch)
	require.NoError(t, err)
	spb.Slot = slot
	st, err := state_native.InitializeFromProtoBellatrix(spb)
	require.NoError(t, err)

	spc := &ethpbv1alpha1.BeaconStateCapella{
		Fork: &ethpbv1alpha1.Fork{
			CurrentVersion:  params.BeaconConfig().GenesisForkVersion,
			PreviousVersion: params.BeaconConfig().GenesisForkVersion,
			Epoch:           params.BeaconConfig().CapellaForkEpoch,
		},
	}
	slot, err = slots.EpochStart(params.BeaconConfig().CapellaForkEpoch)
	require.NoError(t, err)
	spc.Slot = slot

	stc, err := state_native.InitializeFromProtoCapella(spc)
	require.NoError(t, err)

	signedChanges := make([]*shared.SignedBLSToExecutionChange, numValidators)
	for i, message := range blsChanges {
		signature, err := signing.ComputeDomainAndSign(stc, prysmtime.CurrentEpoch(stc), message, params.BeaconConfig().DomainBLSToExecutionChange, privKeys[i])
		require.NoError(t, err)

		signedChanges[i] = &shared.SignedBLSToExecutionChange{
			Message:   shared.BLSChangeFromConsensus(message),
			Signature: hexutil.Encode(signature),
		}
	}

	broadcaster := &p2pMock.MockBroadcaster{}
	chainService := &blockchainmock.ChainService{State: st}
	s := &Server{
		HeadFetcher:       chainService,
		ChainInfoFetcher:  chainService,
		AttestationsPool:  attestations.NewPool(),
		Broadcaster:       broadcaster,
		OperationNotifier: &blockchainmock.MockOperationNotifier{},
		BLSChangesPool:    blstoexec.NewPool(),
	}

	jsonBytes, err := json.Marshal(signedChanges)
	require.NoError(t, err)

	request := httptest.NewRequest(http.MethodPost, "http://foo.example/eth/v1/beacon/pool/bls_to_execution_changes", bytes.NewReader(jsonBytes))
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.SubmitBLSToExecutionChanges(writer, request)
	assert.Equal(t, http.StatusOK, writer.Code)
	// Check that we didn't broadcast the messages but did in fact fill in
	// the pool
	assert.Equal(t, false, broadcaster.BroadcastCalled.Load())

	poolChanges, err := s.BLSChangesPool.PendingBLSToExecChanges()
	require.Equal(t, len(poolChanges), len(signedChanges))
	require.NoError(t, err)
	for i, v1alphaChange := range poolChanges {
		sc, err := signedChanges[i].ToConsensus()
		require.NoError(t, err)
		require.DeepEqual(t, v1alphaChange, sc)
	}
}

func TestSubmitSignedBLSToExecutionChanges_Failures(t *testing.T) {
	transition.SkipSlotCache.Disable()
	defer transition.SkipSlotCache.Enable()

	params.SetupTestConfigCleanup(t)
	c := params.BeaconConfig().Copy()
	// Required for correct committee size calculation.
	c.CapellaForkEpoch = c.BellatrixForkEpoch.Add(2)
	params.OverrideBeaconConfig(c)

	spb := &ethpbv1alpha1.BeaconStateCapella{
		Fork: &ethpbv1alpha1.Fork{
			CurrentVersion:  params.BeaconConfig().GenesisForkVersion,
			PreviousVersion: params.BeaconConfig().GenesisForkVersion,
			Epoch:           params.BeaconConfig().CapellaForkEpoch,
		},
	}
	numValidators := 10
	validators := make([]*ethpbv1alpha1.Validator, numValidators)
	blsChanges := make([]*ethpbv1alpha1.BLSToExecutionChange, numValidators)
	spb.Balances = make([]uint64, numValidators)
	privKeys := make([]common.SecretKey, numValidators)
	maxEffectiveBalance := params.BeaconConfig().MaxEffectiveBalance
	executionAddress := []byte{0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13}

	for i := range validators {
		v := &ethpbv1alpha1.Validator{}
		v.EffectiveBalance = maxEffectiveBalance
		v.WithdrawableEpoch = params.BeaconConfig().FarFutureEpoch
		v.WithdrawalCredentials = make([]byte, 32)
		priv, err := bls.RandKey()
		require.NoError(t, err)
		privKeys[i] = priv
		pubkey := priv.PublicKey().Marshal()

		message := &ethpbv1alpha1.BLSToExecutionChange{
			ToExecutionAddress: executionAddress,
			ValidatorIndex:     primitives.ValidatorIndex(i),
			FromBlsPubkey:      pubkey,
		}

		hashFn := ssz.NewHasherFunc(hash.CustomSHA256Hasher())
		digest := hashFn.Hash(pubkey)
		digest[0] = params.BeaconConfig().BLSWithdrawalPrefixByte
		copy(v.WithdrawalCredentials, digest[:])
		validators[i] = v
		blsChanges[i] = message
	}
	spb.Validators = validators
	slot, err := slots.EpochStart(params.BeaconConfig().CapellaForkEpoch)
	require.NoError(t, err)
	spb.Slot = slot
	st, err := state_native.InitializeFromProtoCapella(spb)
	require.NoError(t, err)

	signedChanges := make([]*shared.SignedBLSToExecutionChange, numValidators)
	for i, message := range blsChanges {
		signature, err := signing.ComputeDomainAndSign(st, prysmtime.CurrentEpoch(st), message, params.BeaconConfig().DomainBLSToExecutionChange, privKeys[i])
		require.NoError(t, err)
		if i == 1 {
			signature[0] = 0x00
		}
		signedChanges[i] = &shared.SignedBLSToExecutionChange{
			Message:   shared.BLSChangeFromConsensus(message),
			Signature: hexutil.Encode(signature),
		}
	}

	broadcaster := &p2pMock.MockBroadcaster{}
	chainService := &blockchainmock.ChainService{State: st}
	s := &Server{
		HeadFetcher:       chainService,
		ChainInfoFetcher:  chainService,
		AttestationsPool:  attestations.NewPool(),
		Broadcaster:       broadcaster,
		OperationNotifier: &blockchainmock.MockOperationNotifier{},
		BLSChangesPool:    blstoexec.NewPool(),
	}

	jsonBytes, err := json.Marshal(signedChanges)
	require.NoError(t, err)

	request := httptest.NewRequest(http.MethodPost, "http://foo.example/eth/v1/beacon/pool/bls_to_execution_changes", bytes.NewReader(jsonBytes))
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.SubmitBLSToExecutionChanges(writer, request)
	assert.Equal(t, http.StatusBadRequest, writer.Code)
	time.Sleep(10 * time.Millisecond) // Delay to allow the routine to start
	require.StringContains(t, "One or more BLSToExecutionChange failed validation", writer.Body.String())
	assert.Equal(t, true, broadcaster.BroadcastCalled.Load())
	assert.Equal(t, numValidators, len(broadcaster.BroadcastMessages)+1)

	poolChanges, err := s.BLSChangesPool.PendingBLSToExecChanges()
	require.Equal(t, len(poolChanges)+1, len(signedChanges))
	require.NoError(t, err)
	require.DeepEqual(t, shared.SignedBLSChangeFromConsensus(poolChanges[0]), signedChanges[0])

	for i := 2; i < numValidators; i++ {
		require.DeepEqual(t, shared.SignedBLSChangeFromConsensus(poolChanges[i-1]), signedChanges[i])
	}
}

func TestGetAttesterSlashings(t *testing.T) {
	bs, err := util.NewBeaconState()
	require.NoError(t, err)
	slashing1 := &ethpbv1alpha1.AttesterSlashing{
		Attestation_1: &ethpbv1alpha1.IndexedAttestation{
			AttestingIndices: []uint64{1, 10},
			Data: &ethpbv1alpha1.AttestationData{
				Slot:            1,
				CommitteeIndex:  1,
				BeaconBlockRoot: bytesutil.PadTo([]byte("blockroot1"), 32),
				Source: &ethpbv1alpha1.Checkpoint{
					Epoch: 1,
					Root:  bytesutil.PadTo([]byte("sourceroot1"), 32),
				},
				Target: &ethpbv1alpha1.Checkpoint{
					Epoch: 10,
					Root:  bytesutil.PadTo([]byte("targetroot1"), 32),
				},
			},
			Signature: bytesutil.PadTo([]byte("signature1"), 96),
		},
		Attestation_2: &ethpbv1alpha1.IndexedAttestation{
			AttestingIndices: []uint64{2, 20},
			Data: &ethpbv1alpha1.AttestationData{
				Slot:            2,
				CommitteeIndex:  2,
				BeaconBlockRoot: bytesutil.PadTo([]byte("blockroot2"), 32),
				Source: &ethpbv1alpha1.Checkpoint{
					Epoch: 2,
					Root:  bytesutil.PadTo([]byte("sourceroot2"), 32),
				},
				Target: &ethpbv1alpha1.Checkpoint{
					Epoch: 20,
					Root:  bytesutil.PadTo([]byte("targetroot2"), 32),
				},
			},
			Signature: bytesutil.PadTo([]byte("signature2"), 96),
		},
	}
	slashing2 := &ethpbv1alpha1.AttesterSlashing{
		Attestation_1: &ethpbv1alpha1.IndexedAttestation{
			AttestingIndices: []uint64{3, 30},
			Data: &ethpbv1alpha1.AttestationData{
				Slot:            3,
				CommitteeIndex:  3,
				BeaconBlockRoot: bytesutil.PadTo([]byte("blockroot3"), 32),
				Source: &ethpbv1alpha1.Checkpoint{
					Epoch: 3,
					Root:  bytesutil.PadTo([]byte("sourceroot3"), 32),
				},
				Target: &ethpbv1alpha1.Checkpoint{
					Epoch: 30,
					Root:  bytesutil.PadTo([]byte("targetroot3"), 32),
				},
			},
			Signature: bytesutil.PadTo([]byte("signature3"), 96),
		},
		Attestation_2: &ethpbv1alpha1.IndexedAttestation{
			AttestingIndices: []uint64{4, 40},
			Data: &ethpbv1alpha1.AttestationData{
				Slot:            4,
				CommitteeIndex:  4,
				BeaconBlockRoot: bytesutil.PadTo([]byte("blockroot4"), 32),
				Source: &ethpbv1alpha1.Checkpoint{
					Epoch: 4,
					Root:  bytesutil.PadTo([]byte("sourceroot4"), 32),
				},
				Target: &ethpbv1alpha1.Checkpoint{
					Epoch: 40,
					Root:  bytesutil.PadTo([]byte("targetroot4"), 32),
				},
			},
			Signature: bytesutil.PadTo([]byte("signature4"), 96),
		},
	}

	s := &Server{
		ChainInfoFetcher: &blockchainmock.ChainService{State: bs},
		SlashingsPool:    &slashingsmock.PoolMock{PendingAttSlashings: []*ethpbv1alpha1.AttesterSlashing{slashing1, slashing2}},
	}

	request := httptest.NewRequest(http.MethodGet, "http://example.com/beacon/pool/attester_slashings", nil)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.GetAttesterSlashings(writer, request)
	require.Equal(t, http.StatusOK, writer.Code)
	resp := &GetAttesterSlashingsResponse{}
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
	require.NotNil(t, resp)
	require.NotNil(t, resp.Data)
	assert.Equal(t, 2, len(resp.Data))
}

func TestGetProposerSlashings(t *testing.T) {
	bs, err := util.NewBeaconState()
	require.NoError(t, err)
	slashing1 := &ethpbv1alpha1.ProposerSlashing{
		Header_1: &ethpbv1alpha1.SignedBeaconBlockHeader{
			Header: &ethpbv1alpha1.BeaconBlockHeader{
				Slot:          1,
				ProposerIndex: 1,
				ParentRoot:    bytesutil.PadTo([]byte("parentroot1"), 32),
				StateRoot:     bytesutil.PadTo([]byte("stateroot1"), 32),
				BodyRoot:      bytesutil.PadTo([]byte("bodyroot1"), 32),
			},
			Signature: bytesutil.PadTo([]byte("signature1"), 96),
		},
		Header_2: &ethpbv1alpha1.SignedBeaconBlockHeader{
			Header: &ethpbv1alpha1.BeaconBlockHeader{
				Slot:          2,
				ProposerIndex: 2,
				ParentRoot:    bytesutil.PadTo([]byte("parentroot2"), 32),
				StateRoot:     bytesutil.PadTo([]byte("stateroot2"), 32),
				BodyRoot:      bytesutil.PadTo([]byte("bodyroot2"), 32),
			},
			Signature: bytesutil.PadTo([]byte("signature2"), 96),
		},
	}
	slashing2 := &ethpbv1alpha1.ProposerSlashing{
		Header_1: &ethpbv1alpha1.SignedBeaconBlockHeader{
			Header: &ethpbv1alpha1.BeaconBlockHeader{
				Slot:          3,
				ProposerIndex: 3,
				ParentRoot:    bytesutil.PadTo([]byte("parentroot3"), 32),
				StateRoot:     bytesutil.PadTo([]byte("stateroot3"), 32),
				BodyRoot:      bytesutil.PadTo([]byte("bodyroot3"), 32),
			},
			Signature: bytesutil.PadTo([]byte("signature3"), 96),
		},
		Header_2: &ethpbv1alpha1.SignedBeaconBlockHeader{
			Header: &ethpbv1alpha1.BeaconBlockHeader{
				Slot:          4,
				ProposerIndex: 4,
				ParentRoot:    bytesutil.PadTo([]byte("parentroot4"), 32),
				StateRoot:     bytesutil.PadTo([]byte("stateroot4"), 32),
				BodyRoot:      bytesutil.PadTo([]byte("bodyroot4"), 32),
			},
			Signature: bytesutil.PadTo([]byte("signature4"), 96),
		},
	}

	s := &Server{
		ChainInfoFetcher: &blockchainmock.ChainService{State: bs},
		SlashingsPool:    &slashingsmock.PoolMock{PendingPropSlashings: []*ethpbv1alpha1.ProposerSlashing{slashing1, slashing2}},
	}

	request := httptest.NewRequest(http.MethodGet, "http://example.com/beacon/pool/attester_slashings", nil)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.GetProposerSlashings(writer, request)
	require.Equal(t, http.StatusOK, writer.Code)
	resp := &GetProposerSlashingsResponse{}
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
	require.NotNil(t, resp)
	require.NotNil(t, resp.Data)
	assert.Equal(t, 2, len(resp.Data))
}

func TestSubmitAttesterSlashing_Ok(t *testing.T) {
	ctx := context.Background()

	transition.SkipSlotCache.Disable()
	defer transition.SkipSlotCache.Enable()

	_, keys, err := util.DeterministicDepositsAndKeys(1)
	require.NoError(t, err)
	validator := &ethpbv1alpha1.Validator{
		PublicKey: keys[0].PublicKey().Marshal(),
	}
	bs, err := util.NewBeaconState(func(state *ethpbv1alpha1.BeaconState) error {
		state.Validators = []*ethpbv1alpha1.Validator{validator}
		return nil
	})
	require.NoError(t, err)

	slashing := &ethpbv1alpha1.AttesterSlashing{
		Attestation_1: &ethpbv1alpha1.IndexedAttestation{
			AttestingIndices: []uint64{0},
			Data: &ethpbv1alpha1.AttestationData{
				Slot:            1,
				CommitteeIndex:  1,
				BeaconBlockRoot: bytesutil.PadTo([]byte("blockroot1"), 32),
				Source: &ethpbv1alpha1.Checkpoint{
					Epoch: 1,
					Root:  bytesutil.PadTo([]byte("sourceroot1"), 32),
				},
				Target: &ethpbv1alpha1.Checkpoint{
					Epoch: 10,
					Root:  bytesutil.PadTo([]byte("targetroot1"), 32),
				},
			},
			Signature: make([]byte, 96),
		},
		Attestation_2: &ethpbv1alpha1.IndexedAttestation{
			AttestingIndices: []uint64{0},
			Data: &ethpbv1alpha1.AttestationData{
				Slot:            1,
				CommitteeIndex:  1,
				BeaconBlockRoot: bytesutil.PadTo([]byte("blockroot2"), 32),
				Source: &ethpbv1alpha1.Checkpoint{
					Epoch: 1,
					Root:  bytesutil.PadTo([]byte("sourceroot2"), 32),
				},
				Target: &ethpbv1alpha1.Checkpoint{
					Epoch: 10,
					Root:  bytesutil.PadTo([]byte("targetroot2"), 32),
				},
			},
			Signature: make([]byte, 96),
		},
	}

	for _, att := range []*ethpbv1alpha1.IndexedAttestation{slashing.Attestation_1, slashing.Attestation_2} {
		sb, err := signing.ComputeDomainAndSign(bs, att.Data.Target.Epoch, att.Data, params.BeaconConfig().DomainBeaconAttester, keys[0])
		require.NoError(t, err)
		sig, err := bls.SignatureFromBytes(sb)
		require.NoError(t, err)
		att.Signature = sig.Marshal()
	}

	broadcaster := &p2pMock.MockBroadcaster{}
	chainmock := &blockchainmock.ChainService{State: bs}
	s := &Server{
		ChainInfoFetcher:  chainmock,
		SlashingsPool:     &slashingsmock.PoolMock{},
		Broadcaster:       broadcaster,
		OperationNotifier: chainmock.OperationNotifier(),
	}

	toSubmit := shared.AttesterSlashingsFromConsensus([]*ethpbv1alpha1.AttesterSlashing{slashing})
	b, err := json.Marshal(toSubmit[0])
	require.NoError(t, err)
	var body bytes.Buffer
	_, err = body.Write(b)
	require.NoError(t, err)
	request := httptest.NewRequest(http.MethodPost, "http://example.com/beacon/pool/attester_slashings", &body)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.SubmitAttesterSlashing(writer, request)
	require.Equal(t, http.StatusOK, writer.Code)
	pendingSlashings := s.SlashingsPool.PendingAttesterSlashings(ctx, bs, true)
	require.Equal(t, 1, len(pendingSlashings))
	assert.DeepEqual(t, slashing, pendingSlashings[0])
	assert.Equal(t, true, broadcaster.BroadcastCalled.Load())
	require.Equal(t, 1, broadcaster.NumMessages())
	_, ok := broadcaster.BroadcastMessages[0].(*ethpbv1alpha1.AttesterSlashing)
	assert.Equal(t, true, ok)
}

func TestSubmitAttesterSlashing_AcrossFork(t *testing.T) {
	ctx := context.Background()

	transition.SkipSlotCache.Disable()
	defer transition.SkipSlotCache.Enable()

	params.SetupTestConfigCleanup(t)
	config := params.BeaconConfig()
	config.AltairForkEpoch = 1
	params.OverrideBeaconConfig(config)

	bs, keys := util.DeterministicGenesisState(t, 1)

	slashing := &ethpbv1alpha1.AttesterSlashing{
		Attestation_1: &ethpbv1alpha1.IndexedAttestation{
			AttestingIndices: []uint64{0},
			Data: &ethpbv1alpha1.AttestationData{
				Slot:            params.BeaconConfig().SlotsPerEpoch,
				CommitteeIndex:  1,
				BeaconBlockRoot: bytesutil.PadTo([]byte("blockroot1"), 32),
				Source: &ethpbv1alpha1.Checkpoint{
					Epoch: 1,
					Root:  bytesutil.PadTo([]byte("sourceroot1"), 32),
				},
				Target: &ethpbv1alpha1.Checkpoint{
					Epoch: 10,
					Root:  bytesutil.PadTo([]byte("targetroot1"), 32),
				},
			},
			Signature: make([]byte, 96),
		},
		Attestation_2: &ethpbv1alpha1.IndexedAttestation{
			AttestingIndices: []uint64{0},
			Data: &ethpbv1alpha1.AttestationData{
				Slot:            params.BeaconConfig().SlotsPerEpoch,
				CommitteeIndex:  1,
				BeaconBlockRoot: bytesutil.PadTo([]byte("blockroot2"), 32),
				Source: &ethpbv1alpha1.Checkpoint{
					Epoch: 1,
					Root:  bytesutil.PadTo([]byte("sourceroot2"), 32),
				},
				Target: &ethpbv1alpha1.Checkpoint{
					Epoch: 10,
					Root:  bytesutil.PadTo([]byte("targetroot2"), 32),
				},
			},
			Signature: make([]byte, 96),
		},
	}

	newBs := bs.Copy()
	newBs, err := transition.ProcessSlots(ctx, newBs, params.BeaconConfig().SlotsPerEpoch)
	require.NoError(t, err)

	for _, att := range []*ethpbv1alpha1.IndexedAttestation{slashing.Attestation_1, slashing.Attestation_2} {
		sb, err := signing.ComputeDomainAndSign(newBs, att.Data.Target.Epoch, att.Data, params.BeaconConfig().DomainBeaconAttester, keys[0])
		require.NoError(t, err)
		sig, err := bls.SignatureFromBytes(sb)
		require.NoError(t, err)
		att.Signature = sig.Marshal()
	}

	broadcaster := &p2pMock.MockBroadcaster{}
	chainmock := &blockchainmock.ChainService{State: bs}
	s := &Server{
		ChainInfoFetcher:  chainmock,
		SlashingsPool:     &slashingsmock.PoolMock{},
		Broadcaster:       broadcaster,
		OperationNotifier: chainmock.OperationNotifier(),
	}

	toSubmit := shared.AttesterSlashingsFromConsensus([]*ethpbv1alpha1.AttesterSlashing{slashing})
	b, err := json.Marshal(toSubmit[0])
	require.NoError(t, err)
	var body bytes.Buffer
	_, err = body.Write(b)
	require.NoError(t, err)
	request := httptest.NewRequest(http.MethodPost, "http://example.com/beacon/pool/attester_slashings", &body)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.SubmitAttesterSlashing(writer, request)
	require.Equal(t, http.StatusOK, writer.Code)
	pendingSlashings := s.SlashingsPool.PendingAttesterSlashings(ctx, bs, true)
	require.Equal(t, 1, len(pendingSlashings))
	assert.DeepEqual(t, slashing, pendingSlashings[0])
	assert.Equal(t, true, broadcaster.BroadcastCalled.Load())
	require.Equal(t, 1, broadcaster.NumMessages())
	_, ok := broadcaster.BroadcastMessages[0].(*ethpbv1alpha1.AttesterSlashing)
	assert.Equal(t, true, ok)
}

func TestSubmitAttesterSlashing_InvalidSlashing(t *testing.T) {
	bs, err := util.NewBeaconState()
	require.NoError(t, err)

	broadcaster := &p2pMock.MockBroadcaster{}
	s := &Server{
		ChainInfoFetcher: &blockchainmock.ChainService{State: bs},
		SlashingsPool:    &slashingsmock.PoolMock{},
		Broadcaster:      broadcaster,
	}

	var body bytes.Buffer
	_, err = body.WriteString(invalidAttesterSlashing)
	require.NoError(t, err)
	request := httptest.NewRequest(http.MethodPost, "http://example.com/beacon/pool/attester_slashings", &body)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.SubmitAttesterSlashing(writer, request)
	require.Equal(t, http.StatusBadRequest, writer.Code)
	e := &httputil.DefaultJsonError{}
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
	assert.Equal(t, http.StatusBadRequest, e.Code)
	assert.StringContains(t, "Invalid attester slashing", e.Message)
}

func TestSubmitProposerSlashing_Ok(t *testing.T) {
	ctx := context.Background()

	transition.SkipSlotCache.Disable()
	defer transition.SkipSlotCache.Enable()

	_, keys, err := util.DeterministicDepositsAndKeys(1)
	require.NoError(t, err)
	validator := &ethpbv1alpha1.Validator{
		PublicKey:         keys[0].PublicKey().Marshal(),
		WithdrawableEpoch: primitives.Epoch(1),
	}
	bs, err := util.NewBeaconState(func(state *ethpbv1alpha1.BeaconState) error {
		state.Validators = []*ethpbv1alpha1.Validator{validator}
		return nil
	})
	require.NoError(t, err)

	slashing := &ethpbv1alpha1.ProposerSlashing{
		Header_1: &ethpbv1alpha1.SignedBeaconBlockHeader{
			Header: &ethpbv1alpha1.BeaconBlockHeader{
				Slot:          1,
				ProposerIndex: 0,
				ParentRoot:    bytesutil.PadTo([]byte("parentroot1"), 32),
				StateRoot:     bytesutil.PadTo([]byte("stateroot1"), 32),
				BodyRoot:      bytesutil.PadTo([]byte("bodyroot1"), 32),
			},
			Signature: make([]byte, 96),
		},
		Header_2: &ethpbv1alpha1.SignedBeaconBlockHeader{
			Header: &ethpbv1alpha1.BeaconBlockHeader{
				Slot:          1,
				ProposerIndex: 0,
				ParentRoot:    bytesutil.PadTo([]byte("parentroot2"), 32),
				StateRoot:     bytesutil.PadTo([]byte("stateroot2"), 32),
				BodyRoot:      bytesutil.PadTo([]byte("bodyroot2"), 32),
			},
			Signature: make([]byte, 96),
		},
	}

	for _, h := range []*ethpbv1alpha1.SignedBeaconBlockHeader{slashing.Header_1, slashing.Header_2} {
		sb, err := signing.ComputeDomainAndSign(
			bs,
			slots.ToEpoch(h.Header.Slot),
			h.Header,
			params.BeaconConfig().DomainBeaconProposer,
			keys[0],
		)
		require.NoError(t, err)
		sig, err := bls.SignatureFromBytes(sb)
		require.NoError(t, err)
		h.Signature = sig.Marshal()
	}

	broadcaster := &p2pMock.MockBroadcaster{}
	chainmock := &blockchainmock.ChainService{State: bs}
	s := &Server{
		ChainInfoFetcher:  chainmock,
		SlashingsPool:     &slashingsmock.PoolMock{},
		Broadcaster:       broadcaster,
		OperationNotifier: chainmock.OperationNotifier(),
	}

	toSubmit := shared.ProposerSlashingsFromConsensus([]*ethpbv1alpha1.ProposerSlashing{slashing})
	b, err := json.Marshal(toSubmit[0])
	require.NoError(t, err)
	var body bytes.Buffer
	_, err = body.Write(b)
	require.NoError(t, err)
	request := httptest.NewRequest(http.MethodPost, "http://example.com/beacon/pool/proposer_slashings", &body)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.SubmitProposerSlashing(writer, request)
	require.Equal(t, http.StatusOK, writer.Code)
	pendingSlashings := s.SlashingsPool.PendingProposerSlashings(ctx, bs, true)
	require.Equal(t, 1, len(pendingSlashings))
	assert.DeepEqual(t, slashing, pendingSlashings[0])
	assert.Equal(t, true, broadcaster.BroadcastCalled.Load())
	require.Equal(t, 1, broadcaster.NumMessages())
	_, ok := broadcaster.BroadcastMessages[0].(*ethpbv1alpha1.ProposerSlashing)
	assert.Equal(t, true, ok)
}

func TestSubmitProposerSlashing_AcrossFork(t *testing.T) {
	ctx := context.Background()

	transition.SkipSlotCache.Disable()
	defer transition.SkipSlotCache.Enable()

	params.SetupTestConfigCleanup(t)
	config := params.BeaconConfig()
	config.AltairForkEpoch = 1
	params.OverrideBeaconConfig(config)

	bs, keys := util.DeterministicGenesisState(t, 1)

	slashing := &ethpbv1alpha1.ProposerSlashing{
		Header_1: &ethpbv1alpha1.SignedBeaconBlockHeader{
			Header: &ethpbv1alpha1.BeaconBlockHeader{
				Slot:          params.BeaconConfig().SlotsPerEpoch,
				ProposerIndex: 0,
				ParentRoot:    bytesutil.PadTo([]byte("parentroot1"), 32),
				StateRoot:     bytesutil.PadTo([]byte("stateroot1"), 32),
				BodyRoot:      bytesutil.PadTo([]byte("bodyroot1"), 32),
			},
			Signature: make([]byte, 96),
		},
		Header_2: &ethpbv1alpha1.SignedBeaconBlockHeader{
			Header: &ethpbv1alpha1.BeaconBlockHeader{
				Slot:          params.BeaconConfig().SlotsPerEpoch,
				ProposerIndex: 0,
				ParentRoot:    bytesutil.PadTo([]byte("parentroot2"), 32),
				StateRoot:     bytesutil.PadTo([]byte("stateroot2"), 32),
				BodyRoot:      bytesutil.PadTo([]byte("bodyroot2"), 32),
			},
			Signature: make([]byte, 96),
		},
	}

	newBs := bs.Copy()
	newBs, err := transition.ProcessSlots(ctx, newBs, params.BeaconConfig().SlotsPerEpoch)
	require.NoError(t, err)

	for _, h := range []*ethpbv1alpha1.SignedBeaconBlockHeader{slashing.Header_1, slashing.Header_2} {
		sb, err := signing.ComputeDomainAndSign(
			newBs,
			slots.ToEpoch(h.Header.Slot),
			h.Header,
			params.BeaconConfig().DomainBeaconProposer,
			keys[0],
		)
		require.NoError(t, err)
		sig, err := bls.SignatureFromBytes(sb)
		require.NoError(t, err)
		h.Signature = sig.Marshal()
	}

	broadcaster := &p2pMock.MockBroadcaster{}
	chainmock := &blockchainmock.ChainService{State: bs}
	s := &Server{
		ChainInfoFetcher:  chainmock,
		SlashingsPool:     &slashingsmock.PoolMock{},
		Broadcaster:       broadcaster,
		OperationNotifier: chainmock.OperationNotifier(),
	}

	toSubmit := shared.ProposerSlashingsFromConsensus([]*ethpbv1alpha1.ProposerSlashing{slashing})
	b, err := json.Marshal(toSubmit[0])
	require.NoError(t, err)
	var body bytes.Buffer
	_, err = body.Write(b)
	require.NoError(t, err)
	request := httptest.NewRequest(http.MethodPost, "http://example.com/beacon/pool/proposer_slashings", &body)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.SubmitProposerSlashing(writer, request)
	require.Equal(t, http.StatusOK, writer.Code)
	pendingSlashings := s.SlashingsPool.PendingProposerSlashings(ctx, bs, true)
	require.Equal(t, 1, len(pendingSlashings))
	assert.DeepEqual(t, slashing, pendingSlashings[0])
	assert.Equal(t, true, broadcaster.BroadcastCalled.Load())
	require.Equal(t, 1, broadcaster.NumMessages())
	_, ok := broadcaster.BroadcastMessages[0].(*ethpbv1alpha1.ProposerSlashing)
	assert.Equal(t, true, ok)
}

func TestSubmitProposerSlashing_InvalidSlashing(t *testing.T) {
	bs, err := util.NewBeaconState()
	require.NoError(t, err)

	broadcaster := &p2pMock.MockBroadcaster{}
	s := &Server{
		ChainInfoFetcher: &blockchainmock.ChainService{State: bs},
		SlashingsPool:    &slashingsmock.PoolMock{},
		Broadcaster:      broadcaster,
	}

	var body bytes.Buffer
	_, err = body.WriteString(invalidProposerSlashing)
	require.NoError(t, err)
	request := httptest.NewRequest(http.MethodPost, "http://example.com/beacon/pool/proposer_slashings", &body)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.SubmitProposerSlashing(writer, request)
	require.Equal(t, http.StatusBadRequest, writer.Code)
	e := &httputil.DefaultJsonError{}
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
	assert.Equal(t, http.StatusBadRequest, e.Code)
	assert.StringContains(t, "Invalid proposer slashing", e.Message)
}

var (
	singleAtt = `[
  {
    "aggregation_bits": "0x03",
    "signature": "0x8146f4397bfd8fd057ebbcd6a67327bdc7ed5fb650533edcb6377b650dea0b6da64c14ecd60846d5c0a0cd43893d6972092500f82c9d8a955e2b58c5ed3cbe885d84008ace6bd86ba9e23652f58e2ec207cec494c916063257abf285b9b15b15",
    "data": {
      "slot": "0",
      "index": "0",
      "beacon_block_root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2",
      "source": {
        "epoch": "0",
        "root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2"
      },
      "target": {
        "epoch": "0",
        "root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2"
      }
    }
  }
]`
	multipleAtts = `[
  {
    "aggregation_bits": "0x03",
    "signature": "0x8146f4397bfd8fd057ebbcd6a67327bdc7ed5fb650533edcb6377b650dea0b6da64c14ecd60846d5c0a0cd43893d6972092500f82c9d8a955e2b58c5ed3cbe885d84008ace6bd86ba9e23652f58e2ec207cec494c916063257abf285b9b15b15",
    "data": {
      "slot": "0",
      "index": "0",
      "beacon_block_root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2",
      "source": {
        "epoch": "0",
        "root": "0x736f75726365726f6f7431000000000000000000000000000000000000000000"
      },
      "target": {
        "epoch": "0",
        "root": "0x746172676574726f6f7431000000000000000000000000000000000000000000"
      }
    }
  },
  {
    "aggregation_bits": "0x03",
    "signature": "0x8146f4397bfd8fd057ebbcd6a67327bdc7ed5fb650533edcb6377b650dea0b6da64c14ecd60846d5c0a0cd43893d6972092500f82c9d8a955e2b58c5ed3cbe885d84008ace6bd86ba9e23652f58e2ec207cec494c916063257abf285b9b15b15",
    "data": {
      "slot": "0",
      "index": "0",
      "beacon_block_root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2",
      "source": {
        "epoch": "0",
        "root": "0x736f75726365726f6f7431000000000000000000000000000000000000000000"
      },
      "target": {
        "epoch": "0",
        "root": "0x746172676574726f6f7432000000000000000000000000000000000000000000"
      }
    }
  }
]`
	// signature is invalid
	invalidAtt = `[
  {
    "aggregation_bits": "0x03",
    "signature": "0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    "data": {
      "slot": "0",
      "index": "0",
      "beacon_block_root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2",
      "source": {
        "epoch": "0",
        "root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2"
      },
      "target": {
        "epoch": "0",
        "root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2"
      }
    }
  }
]`
	exit1 = `{
  "message": {
    "epoch": "0",
    "validator_index": "0"
  },
  "signature": "0xaf20377dabe56887f72273806ea7f3bab3df464fe0178b2ec9bb83d891bf038671c222e2fa7fc0b3e83a0a86ecf235f6104f8130d9e3177cdf5391953fcebb9676f906f4e366b95cb4d734f48f7fc0f116c643519a58a3bb1f7501a1f64b87d2"
}`
	exit2 = fmt.Sprintf(`{
  "message": {
    "epoch": "%d",
    "validator_index": "0"
  },
  "signature": "0xa430330829331089c4381427217231c32c26ac551de410961002491257b1ef50c3d49a89fc920ac2f12f0a27a95ab9b811e49f04cb08020ff7dbe03bdb479f85614608c4e5d0108052497f4ae0148c0c2ef79c05adeaf74e6c003455f2cc5716"
}`, params.BeaconConfig().ShardCommitteePeriod+1)
	// epoch is invalid
	invalidExit1 = `{
  "message": {
    "epoch": "foo",
    "validator_index": "0"
  },
  "signature": "0xaf20377dabe56887f72273806ea7f3bab3df464fe0178b2ec9bb83d891bf038671c222e2fa7fc0b3e83a0a86ecf235f6104f8130d9e3177cdf5391953fcebb9676f906f4e366b95cb4d734f48f7fc0f116c643519a58a3bb1f7501a1f64b87d2"
}`
	// signature is wrong
	invalidExit2 = `{
  "message": {
    "epoch": "0",
    "validator_index": "0"
  },
  "signature": "0xa430330829331089c4381427217231c32c26ac551de410961002491257b1ef50c3d49a89fc920ac2f12f0a27a95ab9b811e49f04cb08020ff7dbe03bdb479f85614608c4e5d0108052497f4ae0148c0c2ef79c05adeaf74e6c003455f2cc5716"
}`
	// non-existing validator index
	invalidExit3 = `{
  "message": {
    "epoch": "0",
    "validator_index": "99"
  },
  "signature": "0xa430330829331089c4381427217231c32c26ac551de410961002491257b1ef50c3d49a89fc920ac2f12f0a27a95ab9b811e49f04cb08020ff7dbe03bdb479f85614608c4e5d0108052497f4ae0148c0c2ef79c05adeaf74e6c003455f2cc5716"
}`
	singleSyncCommitteeMsg = `[
  {
    "slot": "1",
    "beacon_block_root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2",
    "validator_index": "1",
    "signature": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
  }
]`
	multipleSyncCommitteeMsg = `[
  {
    "slot": "1",
    "beacon_block_root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2",
    "validator_index": "1",
    "signature": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
  },
  {
    "slot": "2",
    "beacon_block_root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2",
    "validator_index": "1",
    "signature": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
  }
]`
	// signature is invalid
	invalidSyncCommitteeMsg = `[
  {
    "slot": "1",
    "beacon_block_root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2",
    "validator_index": "1",
    "signature": "foo"
  }
]`
	// signatures are invalid
	invalidAttesterSlashing = `{
  "attestation_1": {
    "attesting_indices": [
      "1"
    ],
    "signature": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505",
    "data": {
      "slot": "1",
      "index": "1",
      "beacon_block_root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2",
      "source": {
        "epoch": "1",
        "root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2"
      },
      "target": {
        "epoch": "1",
        "root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2"
      }
    }
  },
  "attestation_2": {
    "attesting_indices": [
      "1"
    ],
    "signature": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505",
    "data": {
      "slot": "1",
      "index": "1",
      "beacon_block_root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2",
      "source": {
        "epoch": "1",
        "root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2"
      },
      "target": {
        "epoch": "1",
        "root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2"
      }
    }
  }
}`
	// signatures are invalid
	invalidProposerSlashing = `{
  "signed_header_1": {
    "message": {
      "slot": "1",
      "proposer_index": "1",
      "parent_root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2",
      "state_root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2",
      "body_root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2"
    },
    "signature": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
  },
  "signed_header_2": {
    "message": {
      "slot": "1",
      "proposer_index": "1",
      "parent_root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2",
      "state_root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2",
      "body_root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2"
    },
    "signature": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
  }
}`
)

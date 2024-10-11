package validator

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
	"time"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/gorilla/mux"
	"github.com/pkg/errors"
	mockChain "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	builderTest "github.com/prysmaticlabs/prysm/v4/beacon-chain/builder/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	dbutil "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	doublylinkedtree "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/doubly-linked-tree"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/attestations"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/synccommittee"
	p2pmock "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/core"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/testutil"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen"
	mockSync "github.com/prysmaticlabs/prysm/v4/beacon-chain/sync/initial-sync/testing"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	ethpbalpha "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

func TestGetAggregateAttestation(t *testing.T) {
	root1 := bytesutil.PadTo([]byte("root1"), 32)
	sig1 := bytesutil.PadTo([]byte("sig1"), fieldparams.BLSSignatureLength)
	attSlot1 := &ethpbalpha.Attestation{
		AggregationBits: []byte{0, 1},
		Data: &ethpbalpha.AttestationData{
			Slot:            1,
			CommitteeIndex:  1,
			BeaconBlockRoot: root1,
			Source: &ethpbalpha.Checkpoint{
				Epoch: 1,
				Root:  root1,
			},
			Target: &ethpbalpha.Checkpoint{
				Epoch: 1,
				Root:  root1,
			},
		},
		Signature: sig1,
	}
	root21 := bytesutil.PadTo([]byte("root2_1"), 32)
	sig21 := bytesutil.PadTo([]byte("sig2_1"), fieldparams.BLSSignatureLength)
	attslot21 := &ethpbalpha.Attestation{
		AggregationBits: []byte{0, 1, 1},
		Data: &ethpbalpha.AttestationData{
			Slot:            2,
			CommitteeIndex:  2,
			BeaconBlockRoot: root21,
			Source: &ethpbalpha.Checkpoint{
				Epoch: 1,
				Root:  root21,
			},
			Target: &ethpbalpha.Checkpoint{
				Epoch: 1,
				Root:  root21,
			},
		},
		Signature: sig21,
	}
	root22 := bytesutil.PadTo([]byte("root2_2"), 32)
	sig22 := bytesutil.PadTo([]byte("sig2_2"), fieldparams.BLSSignatureLength)
	attslot22 := &ethpbalpha.Attestation{
		AggregationBits: []byte{0, 1, 1, 1},
		Data: &ethpbalpha.AttestationData{
			Slot:            2,
			CommitteeIndex:  3,
			BeaconBlockRoot: root22,
			Source: &ethpbalpha.Checkpoint{
				Epoch: 1,
				Root:  root22,
			},
			Target: &ethpbalpha.Checkpoint{
				Epoch: 1,
				Root:  root22,
			},
		},
		Signature: sig22,
	}
	root33 := bytesutil.PadTo([]byte("root3_3"), 32)
	sig33 := bls.NewAggregateSignature().Marshal()
	attslot33 := &ethpbalpha.Attestation{
		AggregationBits: []byte{1, 0, 0, 1},
		Data: &ethpbalpha.AttestationData{
			Slot:            2,
			CommitteeIndex:  3,
			BeaconBlockRoot: root33,
			Source: &ethpbalpha.Checkpoint{
				Epoch: 1,
				Root:  root33,
			},
			Target: &ethpbalpha.Checkpoint{
				Epoch: 1,
				Root:  root33,
			},
		},
		Signature: sig33,
	}
	pool := attestations.NewPool()
	err := pool.SaveAggregatedAttestations([]*ethpbalpha.Attestation{attSlot1, attslot21, attslot22})
	assert.NoError(t, err)
	s := &Server{
		AttestationsPool: pool,
	}

	t.Run("ok", func(t *testing.T) {
		reqRoot, err := attslot22.Data.HashTreeRoot()
		require.NoError(t, err)
		attDataRoot := hexutil.Encode(reqRoot[:])
		url := "http://example.com?attestation_data_root=" + attDataRoot + "&slot=2"
		request := httptest.NewRequest(http.MethodGet, url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAggregateAttestation(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &AggregateAttestationResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.NotNil(t, resp)
		require.NotNil(t, resp.Data)
		assert.DeepEqual(t, "0x00010101", resp.Data.AggregationBits)
		assert.DeepEqual(t, hexutil.Encode(sig22), resp.Data.Signature)
		assert.Equal(t, "2", resp.Data.Data.Slot)
		assert.Equal(t, "3", resp.Data.Data.CommitteeIndex)
		assert.DeepEqual(t, hexutil.Encode(root22), resp.Data.Data.BeaconBlockRoot)
		require.NotNil(t, resp.Data.Data.Source)
		assert.Equal(t, "1", resp.Data.Data.Source.Epoch)
		assert.DeepEqual(t, hexutil.Encode(root22), resp.Data.Data.Source.Root)
		require.NotNil(t, resp.Data.Data.Target)
		assert.Equal(t, "1", resp.Data.Data.Target.Epoch)
		assert.DeepEqual(t, hexutil.Encode(root22), resp.Data.Data.Target.Root)
	})

	t.Run("aggregate beforehand", func(t *testing.T) {
		err = s.AttestationsPool.SaveUnaggregatedAttestation(attslot33)
		require.NoError(t, err)
		newAtt := ethpbalpha.CopyAttestation(attslot33)
		newAtt.AggregationBits = []byte{0, 1, 0, 1}
		err = s.AttestationsPool.SaveUnaggregatedAttestation(newAtt)
		require.NoError(t, err)

		reqRoot, err := attslot33.Data.HashTreeRoot()
		require.NoError(t, err)
		attDataRoot := hexutil.Encode(reqRoot[:])
		url := "http://example.com?attestation_data_root=" + attDataRoot + "&slot=2"
		request := httptest.NewRequest(http.MethodGet, url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAggregateAttestation(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &AggregateAttestationResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.NotNil(t, resp)
		assert.DeepEqual(t, "0x01010001", resp.Data.AggregationBits)
	})
	t.Run("no matching attestation", func(t *testing.T) {
		attDataRoot := hexutil.Encode(bytesutil.PadTo([]byte("foo"), 32))
		url := "http://example.com?attestation_data_root=" + attDataRoot + "&slot=2"
		request := httptest.NewRequest(http.MethodGet, url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAggregateAttestation(writer, request)
		assert.Equal(t, http.StatusNotFound, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusNotFound, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "No matching attestation found"))
	})
	t.Run("no attestation_data_root provided", func(t *testing.T) {
		url := "http://example.com?slot=2"
		request := httptest.NewRequest(http.MethodGet, url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAggregateAttestation(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "attestation_data_root is required"))
	})
	t.Run("invalid attestation_data_root provided", func(t *testing.T) {
		url := "http://example.com?attestation_data_root=foo&slot=2"
		request := httptest.NewRequest(http.MethodGet, url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAggregateAttestation(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "attestation_data_root is invalid"))
	})
	t.Run("no slot provided", func(t *testing.T) {
		attDataRoot := hexutil.Encode(bytesutil.PadTo([]byte("foo"), 32))
		url := "http://example.com?attestation_data_root=" + attDataRoot
		request := httptest.NewRequest(http.MethodGet, url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAggregateAttestation(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "slot is required"))
	})
	t.Run("invalid slot provided", func(t *testing.T) {
		attDataRoot := hexutil.Encode(bytesutil.PadTo([]byte("foo"), 32))
		url := "http://example.com?attestation_data_root=" + attDataRoot + "&slot=foo"
		request := httptest.NewRequest(http.MethodGet, url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAggregateAttestation(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "slot is invalid"))
	})
}

func TestGetAggregateAttestation_SameSlotAndRoot_ReturnMostAggregationBits(t *testing.T) {
	root := bytesutil.PadTo([]byte("root"), 32)
	sig := bytesutil.PadTo([]byte("sig"), fieldparams.BLSSignatureLength)
	att1 := &ethpbalpha.Attestation{
		AggregationBits: []byte{3, 0, 0, 1},
		Data: &ethpbalpha.AttestationData{
			Slot:            1,
			CommitteeIndex:  1,
			BeaconBlockRoot: root,
			Source: &ethpbalpha.Checkpoint{
				Epoch: 1,
				Root:  root,
			},
			Target: &ethpbalpha.Checkpoint{
				Epoch: 1,
				Root:  root,
			},
		},
		Signature: sig,
	}
	att2 := &ethpbalpha.Attestation{
		AggregationBits: []byte{0, 3, 0, 1},
		Data: &ethpbalpha.AttestationData{
			Slot:            1,
			CommitteeIndex:  1,
			BeaconBlockRoot: root,
			Source: &ethpbalpha.Checkpoint{
				Epoch: 1,
				Root:  root,
			},
			Target: &ethpbalpha.Checkpoint{
				Epoch: 1,
				Root:  root,
			},
		},
		Signature: sig,
	}
	pool := attestations.NewPool()
	err := pool.SaveAggregatedAttestations([]*ethpbalpha.Attestation{att1, att2})
	assert.NoError(t, err)
	s := &Server{
		AttestationsPool: pool,
	}
	reqRoot, err := att1.Data.HashTreeRoot()
	require.NoError(t, err)
	attDataRoot := hexutil.Encode(reqRoot[:])
	url := "http://example.com?attestation_data_root=" + attDataRoot + "&slot=1"
	request := httptest.NewRequest(http.MethodGet, url, nil)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.GetAggregateAttestation(writer, request)
	assert.Equal(t, http.StatusOK, writer.Code)
	resp := &AggregateAttestationResponse{}
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
	require.NotNil(t, resp)
	assert.DeepEqual(t, "0x03000001", resp.Data.AggregationBits)
}

func TestSubmitContributionAndProofs(t *testing.T) {
	c := &core.Service{
		OperationNotifier: (&mockChain.ChainService{}).OperationNotifier(),
	}

	s := &Server{CoreService: c}

	t.Run("single", func(t *testing.T) {
		broadcaster := &p2pmock.MockBroadcaster{}
		c.Broadcaster = broadcaster
		c.SyncCommitteePool = synccommittee.NewStore()

		var body bytes.Buffer
		_, err := body.WriteString(singleContribution)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitContributionAndProofs(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		assert.Equal(t, 1, len(broadcaster.BroadcastMessages))
		contributions, err := c.SyncCommitteePool.SyncCommitteeContributions(1)
		require.NoError(t, err)
		assert.Equal(t, 1, len(contributions))
	})
	t.Run("multiple", func(t *testing.T) {
		broadcaster := &p2pmock.MockBroadcaster{}
		c.Broadcaster = broadcaster
		c.SyncCommitteePool = synccommittee.NewStore()

		var body bytes.Buffer
		_, err := body.WriteString(multipleContributions)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitContributionAndProofs(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		assert.Equal(t, 2, len(broadcaster.BroadcastMessages))
		contributions, err := c.SyncCommitteePool.SyncCommitteeContributions(1)
		require.NoError(t, err)
		assert.Equal(t, 2, len(contributions))
	})
	t.Run("no body", func(t *testing.T) {
		s.SyncCommitteePool = synccommittee.NewStore()

		request := httptest.NewRequest(http.MethodPost, "http://example.com", nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitContributionAndProofs(writer, request)
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

		s.SubmitContributionAndProofs(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "No data submitted"))
	})
	t.Run("invalid", func(t *testing.T) {
		c.SyncCommitteePool = synccommittee.NewStore()

		var body bytes.Buffer
		_, err := body.WriteString(invalidContribution)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitContributionAndProofs(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
	})
}

func TestSubmitAggregateAndProofs(t *testing.T) {
	c := &core.Service{
		GenesisTimeFetcher: &mockChain.ChainService{},
	}

	s := &Server{
		CoreService: c,
	}

	t.Run("single", func(t *testing.T) {
		broadcaster := &p2pmock.MockBroadcaster{}
		c.Broadcaster = broadcaster

		var body bytes.Buffer
		_, err := body.WriteString(singleAggregate)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitAggregateAndProofs(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		assert.Equal(t, 1, len(broadcaster.BroadcastMessages))
	})
	t.Run("multiple", func(t *testing.T) {
		broadcaster := &p2pmock.MockBroadcaster{}
		c.Broadcaster = broadcaster
		c.SyncCommitteePool = synccommittee.NewStore()

		var body bytes.Buffer
		_, err := body.WriteString(multipleAggregates)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitAggregateAndProofs(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		assert.Equal(t, 2, len(broadcaster.BroadcastMessages))
	})
	t.Run("no body", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodPost, "http://example.com", nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitAggregateAndProofs(writer, request)
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

		s.SubmitAggregateAndProofs(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "No data submitted"))
	})
	t.Run("invalid", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString(invalidAggregate)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitAggregateAndProofs(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
	})
}

func TestSubmitSyncCommitteeSubscription(t *testing.T) {
	genesis := util.NewBeaconBlock()
	deposits, _, err := util.DeterministicDepositsAndKeys(64)
	require.NoError(t, err)
	eth1Data, err := util.DeterministicEth1Data(len(deposits))
	require.NoError(t, err)
	bs, err := util.GenesisBeaconState(context.Background(), deposits, 0, eth1Data)
	require.NoError(t, err, "Could not set up genesis state")
	genesisRoot, err := genesis.Block.HashTreeRoot()
	require.NoError(t, err, "Could not get signing root")
	roots := make([][]byte, fieldparams.BlockRootsLength)
	roots[0] = genesisRoot[:]
	require.NoError(t, bs.SetBlockRoots(roots))

	pubkeys := make([][]byte, len(deposits))
	for i := 0; i < len(deposits); i++ {
		pubkeys[i] = deposits[i].Data.PublicKey
	}

	chainSlot := primitives.Slot(0)
	chain := &mockChain.ChainService{
		State: bs, Root: genesisRoot[:], Slot: &chainSlot,
	}
	s := &Server{
		HeadFetcher: chain,
		SyncChecker: &mockSync.Sync{IsSyncing: false},
	}

	t.Run("single", func(t *testing.T) {
		cache.SyncSubnetIDs.EmptyAllCaches()

		var body bytes.Buffer
		_, err := body.WriteString(singleSyncCommitteeSubscription)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitSyncCommitteeSubscription(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		subnets, _, _, _ := cache.SyncSubnetIDs.GetSyncCommitteeSubnets(pubkeys[1], 0)
		require.Equal(t, 2, len(subnets))
		assert.Equal(t, uint64(0), subnets[0])
		assert.Equal(t, uint64(2), subnets[1])
	})
	t.Run("multiple", func(t *testing.T) {
		cache.SyncSubnetIDs.EmptyAllCaches()

		var body bytes.Buffer
		_, err := body.WriteString(multipleSyncCommitteeSubscription)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitSyncCommitteeSubscription(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		subnets, _, _, _ := cache.SyncSubnetIDs.GetSyncCommitteeSubnets(pubkeys[0], 0)
		require.Equal(t, 1, len(subnets))
		assert.Equal(t, uint64(0), subnets[0])
		subnets, _, _, _ = cache.SyncSubnetIDs.GetSyncCommitteeSubnets(pubkeys[1], 0)
		require.Equal(t, 1, len(subnets))
		assert.Equal(t, uint64(2), subnets[0])
	})
	t.Run("no body", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodPost, "http://example.com", nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitSyncCommitteeSubscription(writer, request)
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

		s.SubmitSyncCommitteeSubscription(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "No data submitted"))
	})
	t.Run("invalid", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString(invalidSyncCommitteeSubscription)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitSyncCommitteeSubscription(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
	})
	t.Run("epoch in the past", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString(singleSyncCommitteeSubscription2)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitSyncCommitteeSubscription(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "Epoch for subscription at index 0 is in the past"))
	})
	t.Run("first epoch after the next sync committee is valid", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString(singleSyncCommitteeSubscription3)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitSyncCommitteeSubscription(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
	})
	t.Run("epoch too far in the future", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString(singleSyncCommitteeSubscription4)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitSyncCommitteeSubscription(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "Epoch for subscription at index 0 is too far in the future"))
	})
	t.Run("sync not ready", func(t *testing.T) {
		st, err := util.NewBeaconState()
		require.NoError(t, err)
		chainService := &mockChain.ChainService{State: st}
		s := &Server{
			SyncChecker:           &mockSync.Sync{IsSyncing: true},
			HeadFetcher:           chainService,
			TimeFetcher:           chainService,
			OptimisticModeFetcher: chainService,
		}

		request := httptest.NewRequest(http.MethodPost, "http://example.com", nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitSyncCommitteeSubscription(writer, request)
		assert.Equal(t, http.StatusServiceUnavailable, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusServiceUnavailable, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "Beacon node is currently syncing"))
	})
}

func TestSubmitBeaconCommitteeSubscription(t *testing.T) {
	genesis := util.NewBeaconBlock()
	depChainStart := params.BeaconConfig().MinGenesisActiveValidatorCount
	deposits, _, err := util.DeterministicDepositsAndKeys(depChainStart)
	require.NoError(t, err)
	eth1Data, err := util.DeterministicEth1Data(len(deposits))
	require.NoError(t, err)
	bs, err := transition.GenesisBeaconState(context.Background(), deposits, 0, eth1Data)
	require.NoError(t, err, "Could not set up genesis state")
	// Set state to non-epoch start slot.
	require.NoError(t, bs.SetSlot(5))
	genesisRoot, err := genesis.Block.HashTreeRoot()
	require.NoError(t, err, "Could not get signing root")
	roots := make([][]byte, fieldparams.BlockRootsLength)
	roots[0] = genesisRoot[:]
	require.NoError(t, bs.SetBlockRoots(roots))

	pubkeys := make([][]byte, len(deposits))
	for i := 0; i < len(deposits); i++ {
		pubkeys[i] = deposits[i].Data.PublicKey
	}

	chainSlot := primitives.Slot(0)
	chain := &mockChain.ChainService{
		State: bs, Root: genesisRoot[:], Slot: &chainSlot,
	}
	s := &Server{
		HeadFetcher: chain,
		SyncChecker: &mockSync.Sync{IsSyncing: false},
	}

	t.Run("single", func(t *testing.T) {
		cache.SubnetIDs.EmptyAllCaches()

		var body bytes.Buffer
		_, err := body.WriteString(singleBeaconCommitteeContribution)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitBeaconCommitteeSubscription(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		subnets := cache.SubnetIDs.GetAttesterSubnetIDs(1)
		require.Equal(t, 1, len(subnets))
		assert.Equal(t, uint64(5), subnets[0])
	})
	t.Run("multiple", func(t *testing.T) {
		cache.SubnetIDs.EmptyAllCaches()

		var body bytes.Buffer
		_, err := body.WriteString(multipleBeaconCommitteeContribution)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitBeaconCommitteeSubscription(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		subnets := cache.SubnetIDs.GetAttesterSubnetIDs(1)
		require.Equal(t, 2, len(subnets))
		assert.Equal(t, uint64(5), subnets[0])
		assert.Equal(t, uint64(4), subnets[1])
	})
	t.Run("is aggregator", func(t *testing.T) {
		cache.SubnetIDs.EmptyAllCaches()

		var body bytes.Buffer
		_, err := body.WriteString(singleBeaconCommitteeContribution2)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitBeaconCommitteeSubscription(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		subnets := cache.SubnetIDs.GetAggregatorSubnetIDs(1)
		require.Equal(t, 1, len(subnets))
		assert.Equal(t, uint64(5), subnets[0])
	})
	t.Run("no body", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodPost, "http://example.com", nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitBeaconCommitteeSubscription(writer, request)
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

		s.SubmitBeaconCommitteeSubscription(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "No data submitted"))
	})
	t.Run("invalid", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString(invalidBeaconCommitteeContribution)
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitBeaconCommitteeSubscription(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
	})
	t.Run("sync not ready", func(t *testing.T) {
		st, err := util.NewBeaconState()
		require.NoError(t, err)
		chainService := &mockChain.ChainService{State: st}
		s := &Server{
			SyncChecker:           &mockSync.Sync{IsSyncing: true},
			HeadFetcher:           chainService,
			TimeFetcher:           chainService,
			OptimisticModeFetcher: chainService,
		}

		request := httptest.NewRequest(http.MethodPost, "http://example.com", nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.SubmitBeaconCommitteeSubscription(writer, request)
		assert.Equal(t, http.StatusServiceUnavailable, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusServiceUnavailable, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "Beacon node is currently syncing"))
	})
}

func TestGetAttestationData(t *testing.T) {
	t.Run("ok", func(t *testing.T) {
		block := util.NewBeaconBlock()
		block.Block.Slot = 3*params.BeaconConfig().SlotsPerEpoch + 1
		targetBlock := util.NewBeaconBlock()
		targetBlock.Block.Slot = 1 * params.BeaconConfig().SlotsPerEpoch
		justifiedBlock := util.NewBeaconBlock()
		justifiedBlock.Block.Slot = 2 * params.BeaconConfig().SlotsPerEpoch
		blockRoot, err := block.Block.HashTreeRoot()
		require.NoError(t, err, "Could not hash beacon block")
		justifiedRoot, err := justifiedBlock.Block.HashTreeRoot()
		require.NoError(t, err, "Could not get signing root for justified block")
		slot := 3*params.BeaconConfig().SlotsPerEpoch + 1
		justifiedCheckpoint := &ethpbalpha.Checkpoint{
			Epoch: 2,
			Root:  justifiedRoot[:],
		}
		offset := int64(slot.Mul(params.BeaconConfig().SecondsPerSlot))
		chain := &mockChain.ChainService{
			Optimistic:                 false,
			Genesis:                    time.Now().Add(time.Duration(-1*offset) * time.Second),
			Root:                       blockRoot[:],
			CurrentJustifiedCheckPoint: justifiedCheckpoint,
			TargetRoot:                 blockRoot,
		}

		s := &Server{
			SyncChecker:           &mockSync.Sync{IsSyncing: false},
			HeadFetcher:           chain,
			TimeFetcher:           chain,
			OptimisticModeFetcher: chain,
			CoreService: &core.Service{
				HeadFetcher:           chain,
				GenesisTimeFetcher:    chain,
				FinalizedFetcher:      chain,
				AttestationCache:      cache.NewAttestationCache(),
				OptimisticModeFetcher: chain,
			},
		}

		url := fmt.Sprintf("http://example.com?slot=%d&committee_index=%d", slot, 0)
		request := httptest.NewRequest(http.MethodGet, url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAttestationData(writer, request)

		expectedResponse := &GetAttestationDataResponse{
			Data: &shared.AttestationData{
				Slot:            strconv.FormatUint(uint64(slot), 10),
				BeaconBlockRoot: hexutil.Encode(blockRoot[:]),
				CommitteeIndex:  strconv.FormatUint(0, 10),
				Source: &shared.Checkpoint{
					Epoch: strconv.FormatUint(2, 10),
					Root:  hexutil.Encode(justifiedRoot[:]),
				},
				Target: &shared.Checkpoint{
					Epoch: strconv.FormatUint(3, 10),
					Root:  hexutil.Encode(blockRoot[:]),
				},
			},
		}

		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetAttestationDataResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.NotNil(t, resp)
		assert.DeepEqual(t, expectedResponse, resp)
	})

	t.Run("syncing", func(t *testing.T) {
		beaconState, err := util.NewBeaconState()
		require.NoError(t, err)
		chain := &mockChain.ChainService{
			Optimistic: false,
			State:      beaconState,
			Genesis:    time.Now(),
		}

		s := &Server{
			SyncChecker:           &mockSync.Sync{IsSyncing: true},
			HeadFetcher:           chain,
			TimeFetcher:           chain,
			OptimisticModeFetcher: chain,
		}

		url := fmt.Sprintf("http://example.com?slot=%d&committee_index=%d", 1, 2)
		request := httptest.NewRequest(http.MethodGet, url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAttestationData(writer, request)

		assert.Equal(t, http.StatusServiceUnavailable, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusServiceUnavailable, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "syncing"))
	})

	t.Run("optimistic", func(t *testing.T) {
		beaconState, err := util.NewBeaconState()
		require.NoError(t, err)
		chain := &mockChain.ChainService{
			Optimistic:                 true,
			State:                      beaconState,
			Genesis:                    time.Now(),
			CurrentJustifiedCheckPoint: &ethpbalpha.Checkpoint{},
		}

		s := &Server{
			SyncChecker:           &mockSync.Sync{IsSyncing: false},
			HeadFetcher:           chain,
			TimeFetcher:           chain,
			OptimisticModeFetcher: chain,
			CoreService: &core.Service{
				AttestationCache:      cache.NewAttestationCache(),
				GenesisTimeFetcher:    chain,
				HeadFetcher:           chain,
				FinalizedFetcher:      chain,
				OptimisticModeFetcher: chain,
			},
		}

		url := fmt.Sprintf("http://example.com?slot=%d&committee_index=%d", 0, 0)
		request := httptest.NewRequest(http.MethodGet, url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAttestationData(writer, request)

		assert.Equal(t, http.StatusServiceUnavailable, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusServiceUnavailable, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "optimistic"))

		chain.Optimistic = false

		writer = httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAttestationData(writer, request)

		assert.Equal(t, http.StatusOK, writer.Code)
	})

	t.Run("invalid slot", func(t *testing.T) {
		slot := 3*params.BeaconConfig().SlotsPerEpoch + 1
		offset := int64(slot.Mul(params.BeaconConfig().SecondsPerSlot))
		chain := &mockChain.ChainService{
			Optimistic:                 false,
			Genesis:                    time.Now().Add(time.Duration(-1*offset) * time.Second),
			CurrentJustifiedCheckPoint: &ethpbalpha.Checkpoint{},
		}

		s := &Server{
			SyncChecker:           &mockSync.Sync{IsSyncing: false},
			HeadFetcher:           chain,
			TimeFetcher:           chain,
			OptimisticModeFetcher: chain,
			CoreService: &core.Service{
				GenesisTimeFetcher:    chain,
				OptimisticModeFetcher: chain,
				FinalizedFetcher:      chain,
			},
		}

		url := fmt.Sprintf("http://example.com?slot=%d&committee_index=%d", 1000000000000, 2)
		request := httptest.NewRequest(http.MethodGet, url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAttestationData(writer, request)

		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "invalid request"))
	})

	t.Run("request slot is not current slot", func(t *testing.T) {
		ctx := context.Background()
		db := dbutil.SetupDB(t)

		slot := 3*params.BeaconConfig().SlotsPerEpoch + 1
		block := util.NewBeaconBlock()
		block.Block.Slot = slot
		block2 := util.NewBeaconBlock()
		block2.Block.Slot = slot - 1
		targetBlock := util.NewBeaconBlock()
		targetBlock.Block.Slot = 1 * params.BeaconConfig().SlotsPerEpoch
		justifiedBlock := util.NewBeaconBlock()
		justifiedBlock.Block.Slot = 2 * params.BeaconConfig().SlotsPerEpoch
		blockRoot, err := block.Block.HashTreeRoot()
		require.NoError(t, err, "Could not hash beacon block")
		blockRoot2, err := block2.HashTreeRoot()
		require.NoError(t, err)
		util.SaveBlock(t, ctx, db, block2)
		justifiedRoot, err := justifiedBlock.Block.HashTreeRoot()
		require.NoError(t, err, "Could not get signing root for justified block")
		justifiedCheckpoint := &ethpbalpha.Checkpoint{
			Epoch: 2,
			Root:  justifiedRoot[:],
		}

		offset := int64(slot.Mul(params.BeaconConfig().SecondsPerSlot))
		chain := &mockChain.ChainService{
			Root:                       blockRoot[:],
			Genesis:                    time.Now().Add(time.Duration(-1*offset) * time.Second),
			CurrentJustifiedCheckPoint: justifiedCheckpoint,
			TargetRoot:                 blockRoot2,
		}

		s := &Server{
			SyncChecker:           &mockSync.Sync{IsSyncing: false},
			HeadFetcher:           chain,
			TimeFetcher:           chain,
			OptimisticModeFetcher: chain,
			CoreService: &core.Service{
				HeadFetcher:           chain,
				GenesisTimeFetcher:    chain,
				OptimisticModeFetcher: chain,
				StateGen:              stategen.New(db, doublylinkedtree.New()),
				FinalizedFetcher:      chain,
			},
		}

		url := fmt.Sprintf("http://example.com?slot=%d&committee_index=%d", slot-1, 0)
		request := httptest.NewRequest(http.MethodGet, url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAttestationData(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
	})

	t.Run("succeeds in first epoch", func(t *testing.T) {
		slot := primitives.Slot(5)
		block := util.NewBeaconBlock()
		block.Block.Slot = slot
		targetBlock := util.NewBeaconBlock()
		targetBlock.Block.Slot = 0
		justifiedBlock := util.NewBeaconBlock()
		justifiedBlock.Block.Slot = 0
		blockRoot, err := block.Block.HashTreeRoot()
		require.NoError(t, err, "Could not hash beacon block")
		justifiedRoot, err := justifiedBlock.Block.HashTreeRoot()
		require.NoError(t, err, "Could not get signing root for justified block")

		justifiedCheckpt := &ethpbalpha.Checkpoint{
			Epoch: 0,
			Root:  justifiedRoot[:],
		}
		require.NoError(t, err)
		offset := int64(slot.Mul(params.BeaconConfig().SecondsPerSlot))
		chain := &mockChain.ChainService{
			Root:                       blockRoot[:],
			Genesis:                    time.Now().Add(time.Duration(-1*offset) * time.Second),
			CurrentJustifiedCheckPoint: justifiedCheckpt,
			TargetRoot:                 blockRoot,
		}

		s := &Server{
			SyncChecker:           &mockSync.Sync{IsSyncing: false},
			HeadFetcher:           chain,
			TimeFetcher:           chain,
			OptimisticModeFetcher: chain,
			CoreService: &core.Service{
				AttestationCache:      cache.NewAttestationCache(),
				OptimisticModeFetcher: chain,
				HeadFetcher:           chain,
				GenesisTimeFetcher:    chain,
				FinalizedFetcher:      chain,
			},
		}

		url := fmt.Sprintf("http://example.com?slot=%d&committee_index=%d", slot, 0)
		request := httptest.NewRequest(http.MethodGet, url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAttestationData(writer, request)

		expectedResponse := &GetAttestationDataResponse{
			Data: &shared.AttestationData{
				Slot:            strconv.FormatUint(uint64(slot), 10),
				BeaconBlockRoot: hexutil.Encode(blockRoot[:]),
				CommitteeIndex:  strconv.FormatUint(0, 10),
				Source: &shared.Checkpoint{
					Epoch: strconv.FormatUint(0, 10),
					Root:  hexutil.Encode(justifiedRoot[:]),
				},
				Target: &shared.Checkpoint{
					Epoch: strconv.FormatUint(0, 10),
					Root:  hexutil.Encode(blockRoot[:]),
				},
			},
		}

		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetAttestationDataResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.NotNil(t, resp)
		assert.DeepEqual(t, expectedResponse, resp)
	})

	t.Run("handles far away justified epoch", func(t *testing.T) {
		// Scenario:
		//
		// State slot = 10000
		// Last justified slot = epoch start of 1500
		// HistoricalRootsLimit = 8192
		//
		// More background: https://github.com/prysmaticlabs/prysm/issues/2153
		// This test breaks if it doesn't use mainnet config

		// Ensure HistoricalRootsLimit matches scenario
		params.SetupTestConfigCleanup(t)
		cfg := params.MainnetConfig().Copy()
		cfg.HistoricalRootsLimit = 8192
		params.OverrideBeaconConfig(cfg)

		block := util.NewBeaconBlock()
		block.Block.Slot = 10000
		epochBoundaryBlock := util.NewBeaconBlock()
		var err error
		epochBoundaryBlock.Block.Slot, err = slots.EpochStart(slots.ToEpoch(10000))
		require.NoError(t, err)
		justifiedBlock := util.NewBeaconBlock()
		justifiedBlock.Block.Slot, err = slots.EpochStart(slots.ToEpoch(1500))
		require.NoError(t, err)
		justifiedBlock.Block.Slot -= 2 // Imagine two skip block
		blockRoot, err := block.Block.HashTreeRoot()
		require.NoError(t, err, "Could not hash beacon block")
		justifiedBlockRoot, err := justifiedBlock.Block.HashTreeRoot()
		require.NoError(t, err, "Could not hash justified block")
		justifiedCheckpt := &ethpbalpha.Checkpoint{
			Epoch: slots.ToEpoch(1500),
			Root:  justifiedBlockRoot[:],
		}
		slot := primitives.Slot(10000)
		offset := int64(slot.Mul(params.BeaconConfig().SecondsPerSlot))
		chain := &mockChain.ChainService{
			Root:                       blockRoot[:],
			Genesis:                    time.Now().Add(time.Duration(-1*offset) * time.Second),
			CurrentJustifiedCheckPoint: justifiedCheckpt,
			TargetRoot:                 blockRoot,
		}

		s := &Server{
			SyncChecker:           &mockSync.Sync{IsSyncing: false},
			HeadFetcher:           chain,
			TimeFetcher:           chain,
			OptimisticModeFetcher: chain,
			CoreService: &core.Service{
				AttestationCache:      cache.NewAttestationCache(),
				OptimisticModeFetcher: chain,
				HeadFetcher:           chain,
				GenesisTimeFetcher:    chain,
				FinalizedFetcher:      chain,
			},
		}

		url := fmt.Sprintf("http://example.com?slot=%d&committee_index=%d", slot, 0)
		request := httptest.NewRequest(http.MethodGet, url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAttestationData(writer, request)

		expectedResponse := &GetAttestationDataResponse{
			Data: &shared.AttestationData{
				Slot:            strconv.FormatUint(uint64(slot), 10),
				BeaconBlockRoot: hexutil.Encode(blockRoot[:]),
				CommitteeIndex:  strconv.FormatUint(0, 10),
				Source: &shared.Checkpoint{
					Epoch: strconv.FormatUint(uint64(slots.ToEpoch(1500)), 10),
					Root:  hexutil.Encode(justifiedBlockRoot[:]),
				},
				Target: &shared.Checkpoint{
					Epoch: strconv.FormatUint(312, 10),
					Root:  hexutil.Encode(blockRoot[:]),
				},
			},
		}

		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetAttestationDataResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.NotNil(t, resp)
		assert.DeepEqual(t, expectedResponse, resp)
	})
}

func TestProduceSyncCommitteeContribution(t *testing.T) {
	root := bytesutil.PadTo([]byte("0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2"), 32)
	sig := bls.NewAggregateSignature().Marshal()
	message := &ethpbalpha.SyncCommitteeMessage{
		Slot:           1,
		BlockRoot:      root,
		ValidatorIndex: 0,
		Signature:      sig,
	}
	syncCommitteePool := synccommittee.NewStore()
	require.NoError(t, syncCommitteePool.SaveSyncCommitteeMessage(message))
	server := Server{
		CoreService: &core.Service{
			HeadFetcher: &mockChain.ChainService{
				SyncCommitteeIndices: []primitives.CommitteeIndex{0},
			},
		},
		SyncCommitteePool: syncCommitteePool,
	}
	t.Run("ok", func(t *testing.T) {
		url := "http://example.com?slot=1&subcommittee_index=1&beacon_block_root=0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2"
		request := httptest.NewRequest(http.MethodGet, url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		server.ProduceSyncCommitteeContribution(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &ProduceSyncCommitteeContributionResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.NotNil(t, resp.Data)
		require.Equal(t, resp.Data.Slot, "1")
		require.Equal(t, resp.Data.SubcommitteeIndex, "1")
		require.Equal(t, resp.Data.BeaconBlockRoot, "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2")
	})
	t.Run("no slot provided", func(t *testing.T) {
		url := "http://example.com?subcommittee_index=1&beacon_block_root=0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2"
		request := httptest.NewRequest(http.MethodGet, url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		server.ProduceSyncCommitteeContribution(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		resp := &ProduceSyncCommitteeContributionResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.ErrorContains(t, "slot is required", errors.New(writer.Body.String()))
	})
	t.Run("no subcommittee_index provided", func(t *testing.T) {
		url := "http://example.com?slot=1&beacon_block_root=0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2"
		request := httptest.NewRequest(http.MethodGet, url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		server.ProduceSyncCommitteeContribution(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		resp := &ProduceSyncCommitteeContributionResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.ErrorContains(t, "subcommittee_index is required", errors.New(writer.Body.String()))
	})
	t.Run("no beacon_block_root provided", func(t *testing.T) {
		url := "http://example.com?slot=1&subcommittee_index=1"
		request := httptest.NewRequest(http.MethodGet, url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		server.ProduceSyncCommitteeContribution(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		resp := &ProduceSyncCommitteeContributionResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.ErrorContains(t, "Invalid Beacon Block Root: empty hex string", errors.New(writer.Body.String()))
	})
	t.Run("invalid block root", func(t *testing.T) {
		url := "http://example.com?slot=1&subcommittee_index=1&beacon_block_root=0"
		request := httptest.NewRequest(http.MethodGet, url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		server.ProduceSyncCommitteeContribution(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		resp := &ProduceSyncCommitteeContributionResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.ErrorContains(t, "Invalid Beacon Block Root: hex string without 0x prefix", errors.New(writer.Body.String()))
	})
	t.Run("no committee messages", func(t *testing.T) {
		url := "http://example.com?slot=1&subcommittee_index=1&beacon_block_root=0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2"
		request := httptest.NewRequest(http.MethodGet, url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		server.ProduceSyncCommitteeContribution(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &ProduceSyncCommitteeContributionResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.NotNil(t, resp)
		require.NotNil(t, resp.Data)

		request = httptest.NewRequest(http.MethodGet, url, nil)
		writer = httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		syncCommitteePool = synccommittee.NewStore()
		server = Server{
			CoreService: &core.Service{
				HeadFetcher: &mockChain.ChainService{
					SyncCommitteeIndices: []primitives.CommitteeIndex{0},
				},
			},
			SyncCommitteePool: syncCommitteePool,
		}
		server.ProduceSyncCommitteeContribution(writer, request)
		assert.Equal(t, http.StatusNotFound, writer.Code)
		resp2 := &ProduceSyncCommitteeContributionResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp2))
		require.ErrorContains(t, "No subcommittee messages found", errors.New(writer.Body.String()))
	})
}

func TestServer_RegisterValidator(t *testing.T) {

	tests := []struct {
		name    string
		request string
		code    int
		wantErr string
	}{
		{
			name:    "Happy Path",
			request: registrations,
			code:    http.StatusOK,
			wantErr: "",
		},
		{
			name:    "Empty Request",
			request: "",
			code:    http.StatusBadRequest,
			wantErr: "No data submitted",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var body bytes.Buffer
			_, err := body.WriteString(tt.request)
			require.NoError(t, err)
			url := "http://example.com/eth/v1/validator/register_validator"
			request := httptest.NewRequest(http.MethodPost, url, &body)
			writer := httptest.NewRecorder()
			db := dbutil.SetupDB(t)

			server := Server{
				CoreService: &core.Service{
					HeadFetcher: &mockChain.ChainService{
						SyncCommitteeIndices: []primitives.CommitteeIndex{0},
					},
				},
				BlockBuilder: &builderTest.MockBuilderService{
					HasConfigured: true,
				},
				BeaconDB: db,
			}

			server.RegisterValidator(writer, request)
			require.Equal(t, tt.code, writer.Code)
			if tt.wantErr != "" {
				require.Equal(t, strings.Contains(writer.Body.String(), tt.wantErr), true)
			}
		})
	}
}

func TestGetAttesterDuties(t *testing.T) {
	helpers.ClearCache()

	genesis := util.NewBeaconBlock()
	depChainStart := params.BeaconConfig().MinGenesisActiveValidatorCount
	deposits, _, err := util.DeterministicDepositsAndKeys(depChainStart)
	require.NoError(t, err)
	eth1Data, err := util.DeterministicEth1Data(len(deposits))
	require.NoError(t, err)
	bs, err := transition.GenesisBeaconState(context.Background(), deposits, 0, eth1Data)
	require.NoError(t, err, "Could not set up genesis state")
	// Set state to non-epoch start slot.
	require.NoError(t, bs.SetSlot(5))
	genesisRoot, err := genesis.Block.HashTreeRoot()
	require.NoError(t, err, "Could not get signing root")
	roots := make([][]byte, fieldparams.BlockRootsLength)
	roots[0] = genesisRoot[:]
	require.NoError(t, bs.SetBlockRoots(roots))

	// Deactivate last validator.
	vals := bs.Validators()
	vals[len(vals)-1].ExitEpoch = 0
	require.NoError(t, bs.SetValidators(vals))

	pubKeys := make([][]byte, len(deposits))
	for i := 0; i < len(deposits); i++ {
		pubKeys[i] = deposits[i].Data.PublicKey
	}

	// nextEpochState must not be used for committee calculations when requesting next epoch
	nextEpochState := bs.Copy()
	require.NoError(t, nextEpochState.SetSlot(params.BeaconConfig().SlotsPerEpoch))
	require.NoError(t, nextEpochState.SetValidators(vals[:512]))

	chainSlot := primitives.Slot(0)
	chain := &mockChain.ChainService{
		State: bs, Root: genesisRoot[:], Slot: &chainSlot,
	}
	s := &Server{
		Stater: &testutil.MockStater{
			StatesBySlot: map[primitives.Slot]state.BeaconState{
				0:                                   bs,
				params.BeaconConfig().SlotsPerEpoch: nextEpochState,
			},
		},
		TimeFetcher:           chain,
		SyncChecker:           &mockSync.Sync{IsSyncing: false},
		OptimisticModeFetcher: chain,
	}

	t.Run("single validator", func(t *testing.T) {
		var body bytes.Buffer
		_, err = body.WriteString("[\"0\"]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodGet, "http://www.example.com/eth/v1/validator/duties/attester/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": "0"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAttesterDuties(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetAttesterDutiesResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, hexutil.Encode(genesisRoot[:]), resp.DependentRoot)
		require.Equal(t, 1, len(resp.Data))
		duty := resp.Data[0]
		assert.Equal(t, "1", duty.CommitteeIndex)
		assert.Equal(t, "0", duty.Slot)
		assert.Equal(t, "0", duty.ValidatorIndex)
		assert.Equal(t, hexutil.Encode(pubKeys[0]), duty.Pubkey)
		assert.Equal(t, "171", duty.CommitteeLength)
		assert.Equal(t, "3", duty.CommitteesAtSlot)
		assert.Equal(t, "80", duty.ValidatorCommitteeIndex)
	})
	t.Run("multiple validators", func(t *testing.T) {
		var body bytes.Buffer
		_, err = body.WriteString("[\"0\",\"1\"]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodGet, "http://www.example.com/eth/v1/validator/duties/attester/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": "0"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAttesterDuties(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetAttesterDutiesResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 2, len(resp.Data))
	})
	t.Run("no body", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodPost, "http://www.example.com/eth/v1/validator/duties/attester/{epoch}", nil)
		request = mux.SetURLVars(request, map[string]string{"epoch": "0"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAttesterDuties(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.StringContains(t, "No data submitted", e.Message)
	})
	t.Run("empty", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString("[]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://www.example.com/eth/v1/validator/duties/attester/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": "0"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAttesterDuties(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.StringContains(t, "No data submitted", e.Message)
	})
	t.Run("invalid", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString("[\"foo\"]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://www.example.com/eth/v1/validator/duties/attester/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": "0"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAttesterDuties(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
	})
	t.Run("next epoch", func(t *testing.T) {
		var body bytes.Buffer
		_, err = body.WriteString("[\"0\"]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodGet, "http://www.example.com/eth/v1/validator/duties/attester/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": strconv.FormatUint(uint64(slots.ToEpoch(bs.Slot())+1), 10)})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAttesterDuties(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetAttesterDutiesResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, hexutil.Encode(genesisRoot[:]), resp.DependentRoot)
		require.Equal(t, 1, len(resp.Data))
		duty := resp.Data[0]
		assert.Equal(t, "0", duty.CommitteeIndex)
		assert.Equal(t, "62", duty.Slot)
		assert.Equal(t, "0", duty.ValidatorIndex)
		assert.Equal(t, hexutil.Encode(pubKeys[0]), duty.Pubkey)
		assert.Equal(t, "170", duty.CommitteeLength)
		assert.Equal(t, "3", duty.CommitteesAtSlot)
		assert.Equal(t, "110", duty.ValidatorCommitteeIndex)
	})
	t.Run("epoch out of bounds", func(t *testing.T) {
		var body bytes.Buffer
		_, err = body.WriteString("[\"0\"]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodGet, "http://www.example.com/eth/v1/validator/duties/attester/{epoch}", &body)
		currentEpoch := slots.ToEpoch(bs.Slot())
		request = mux.SetURLVars(request, map[string]string{"epoch": strconv.FormatUint(uint64(currentEpoch+2), 10)})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAttesterDuties(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, fmt.Sprintf("Request epoch %d can not be greater than next epoch %d", currentEpoch+2, currentEpoch+1)))
	})
	t.Run("validator index out of bounds", func(t *testing.T) {
		var body bytes.Buffer
		_, err = body.WriteString(fmt.Sprintf("[\"%d\"]", len(pubKeys)))
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodGet, "http://www.example.com/eth/v1/validator/duties/attester/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": "0"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAttesterDuties(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, fmt.Sprintf("Invalid validator index %d", len(pubKeys))))
	})
	t.Run("inactive validator - no duties", func(t *testing.T) {
		var body bytes.Buffer
		_, err = body.WriteString(fmt.Sprintf("[\"%d\"]", len(pubKeys)-1))
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodGet, "http://www.example.com/eth/v1/validator/duties/attester/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": "0"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAttesterDuties(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetAttesterDutiesResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 0, len(resp.Data))
	})
	t.Run("execution optimistic", func(t *testing.T) {
		ctx := context.Background()

		parentRoot := [32]byte{'a'}
		blk := util.NewBeaconBlock()
		blk.Block.ParentRoot = parentRoot[:]
		blk.Block.Slot = 31
		root, err := blk.Block.HashTreeRoot()
		require.NoError(t, err)
		db := dbutil.SetupDB(t)
		util.SaveBlock(t, ctx, db, blk)
		require.NoError(t, db.SaveGenesisBlockRoot(ctx, root))

		chainSlot := primitives.Slot(0)
		chain := &mockChain.ChainService{
			State: bs, Root: genesisRoot[:], Slot: &chainSlot, Optimistic: true,
		}
		s := &Server{
			Stater:                &testutil.MockStater{StatesBySlot: map[primitives.Slot]state.BeaconState{0: bs}},
			TimeFetcher:           chain,
			OptimisticModeFetcher: chain,
			SyncChecker:           &mockSync.Sync{IsSyncing: false},
		}

		var body bytes.Buffer
		_, err = body.WriteString("[\"0\"]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodGet, "http://www.example.com/eth/v1/validator/duties/attester/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": "0"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAttesterDuties(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		resp := &GetAttesterDutiesResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, true, resp.ExecutionOptimistic)
	})
	t.Run("sync not ready", func(t *testing.T) {
		st, err := util.NewBeaconState()
		require.NoError(t, err)
		chainService := &mockChain.ChainService{State: st}
		s := &Server{
			SyncChecker:           &mockSync.Sync{IsSyncing: true},
			HeadFetcher:           chainService,
			TimeFetcher:           chainService,
			OptimisticModeFetcher: chainService,
		}

		request := httptest.NewRequest(http.MethodGet, "http://www.example.com/eth/v1/validator/duties/attester/{epoch}", nil)
		request = mux.SetURLVars(request, map[string]string{"epoch": "0"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetAttesterDuties(writer, request)
		require.Equal(t, http.StatusServiceUnavailable, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusServiceUnavailable, e.Code)
	})
}

func TestGetProposerDuties(t *testing.T) {
	helpers.ClearCache()

	genesis := util.NewBeaconBlock()
	depChainStart := params.BeaconConfig().MinGenesisActiveValidatorCount
	deposits, _, err := util.DeterministicDepositsAndKeys(depChainStart)
	require.NoError(t, err)
	eth1Data, err := util.DeterministicEth1Data(len(deposits))
	require.NoError(t, err)
	genesisRoot, err := genesis.Block.HashTreeRoot()
	require.NoError(t, err)
	roots := make([][]byte, fieldparams.BlockRootsLength)
	roots[0] = genesisRoot[:]
	// We DON'T WANT this root to be returned when testing the next epoch
	roots[31] = []byte("next_epoch_dependent_root")

	pubKeys := make([][]byte, len(deposits))
	for i := 0; i < len(deposits); i++ {
		pubKeys[i] = deposits[i].Data.PublicKey
	}

	t.Run("ok", func(t *testing.T) {
		bs, err := transition.GenesisBeaconState(context.Background(), deposits, 0, eth1Data)
		require.NoError(t, err, "Could not set up genesis state")
		require.NoError(t, bs.SetSlot(params.BeaconConfig().SlotsPerEpoch))
		require.NoError(t, bs.SetBlockRoots(roots))
		chainSlot := primitives.Slot(0)
		chain := &mockChain.ChainService{
			State: bs, Root: genesisRoot[:], Slot: &chainSlot,
		}
		s := &Server{
			Stater:                 &testutil.MockStater{StatesBySlot: map[primitives.Slot]state.BeaconState{0: bs}},
			HeadFetcher:            chain,
			TimeFetcher:            chain,
			OptimisticModeFetcher:  chain,
			SyncChecker:            &mockSync.Sync{IsSyncing: false},
			PayloadIDCache:         cache.NewPayloadIDCache(),
			TrackedValidatorsCache: cache.NewTrackedValidatorsCache(),
		}

		request := httptest.NewRequest(http.MethodGet, "http://www.example.com/eth/v1/validator/duties/proposer/{epoch}", nil)
		request = mux.SetURLVars(request, map[string]string{"epoch": "0"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetProposerDuties(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetProposerDutiesResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, hexutil.Encode(genesisRoot[:]), resp.DependentRoot)
		assert.Equal(t, 31, len(resp.Data))
		// We expect a proposer duty for slot 11.
		var expectedDuty *ProposerDuty
		for _, duty := range resp.Data {
			if duty.Slot == "11" {
				expectedDuty = duty
			}
		}
		require.NotNil(t, expectedDuty, "Expected duty for slot 11 not found")
		assert.Equal(t, "12289", expectedDuty.ValidatorIndex)
		assert.Equal(t, hexutil.Encode(pubKeys[12289]), expectedDuty.Pubkey)
	})
	t.Run("next epoch", func(t *testing.T) {
		bs, err := transition.GenesisBeaconState(context.Background(), deposits, 0, eth1Data)
		require.NoError(t, err, "Could not set up genesis state")
		require.NoError(t, bs.SetBlockRoots(roots))
		chainSlot := primitives.Slot(0)
		chain := &mockChain.ChainService{
			State: bs, Root: genesisRoot[:], Slot: &chainSlot,
		}
		s := &Server{
			Stater:                 &testutil.MockStater{StatesBySlot: map[primitives.Slot]state.BeaconState{0: bs}},
			HeadFetcher:            chain,
			TimeFetcher:            chain,
			OptimisticModeFetcher:  chain,
			SyncChecker:            &mockSync.Sync{IsSyncing: false},
			PayloadIDCache:         cache.NewPayloadIDCache(),
			TrackedValidatorsCache: cache.NewTrackedValidatorsCache(),
		}

		request := httptest.NewRequest(http.MethodGet, "http://www.example.com/eth/v1/validator/duties/proposer/{epoch}", nil)
		request = mux.SetURLVars(request, map[string]string{"epoch": "1"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetProposerDuties(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetProposerDutiesResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, hexutil.Encode(genesisRoot[:]), resp.DependentRoot)
		assert.Equal(t, 32, len(resp.Data))
		// We expect a proposer duty for slot 43.
		var expectedDuty *ProposerDuty
		for _, duty := range resp.Data {
			if duty.Slot == "43" {
				expectedDuty = duty
			}
		}
		require.NotNil(t, expectedDuty, "Expected duty for slot 43 not found")
		assert.Equal(t, "1360", expectedDuty.ValidatorIndex)
		assert.Equal(t, hexutil.Encode(pubKeys[1360]), expectedDuty.Pubkey)
	})
	t.Run("epoch out of bounds", func(t *testing.T) {
		bs, err := transition.GenesisBeaconState(context.Background(), deposits, 0, eth1Data)
		require.NoError(t, err, "Could not set up genesis state")
		// Set state to non-epoch start slot.
		require.NoError(t, bs.SetSlot(5))
		require.NoError(t, bs.SetBlockRoots(roots))
		chainSlot := primitives.Slot(0)
		chain := &mockChain.ChainService{
			State: bs, Root: genesisRoot[:], Slot: &chainSlot,
		}
		s := &Server{
			Stater:                 &testutil.MockStater{StatesBySlot: map[primitives.Slot]state.BeaconState{0: bs}},
			HeadFetcher:            chain,
			TimeFetcher:            chain,
			OptimisticModeFetcher:  chain,
			SyncChecker:            &mockSync.Sync{IsSyncing: false},
			PayloadIDCache:         cache.NewPayloadIDCache(),
			TrackedValidatorsCache: cache.NewTrackedValidatorsCache(),
		}

		currentEpoch := slots.ToEpoch(bs.Slot())
		request := httptest.NewRequest(http.MethodGet, "http://www.example.com/eth/v1/validator/duties/proposer/{epoch}", nil)
		request = mux.SetURLVars(request, map[string]string{"epoch": strconv.FormatUint(uint64(currentEpoch+2), 10)})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetProposerDuties(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.StringContains(t, fmt.Sprintf("Request epoch %d can not be greater than next epoch %d", currentEpoch+2, currentEpoch+1), e.Message)
	})
	t.Run("execution optimistic", func(t *testing.T) {
		ctx := context.Background()
		bs, err := transition.GenesisBeaconState(context.Background(), deposits, 0, eth1Data)
		require.NoError(t, err, "Could not set up genesis state")
		// Set state to non-epoch start slot.
		require.NoError(t, bs.SetSlot(5))
		require.NoError(t, bs.SetBlockRoots(roots))
		parentRoot := [32]byte{'a'}
		blk := util.NewBeaconBlock()
		blk.Block.ParentRoot = parentRoot[:]
		blk.Block.Slot = 31
		root, err := blk.Block.HashTreeRoot()
		require.NoError(t, err)
		db := dbutil.SetupDB(t)
		util.SaveBlock(t, ctx, db, blk)
		require.NoError(t, db.SaveGenesisBlockRoot(ctx, root))

		chainSlot := primitives.Slot(0)
		chain := &mockChain.ChainService{
			State: bs, Root: genesisRoot[:], Slot: &chainSlot, Optimistic: true,
		}
		s := &Server{
			Stater:                 &testutil.MockStater{StatesBySlot: map[primitives.Slot]state.BeaconState{0: bs}},
			HeadFetcher:            chain,
			TimeFetcher:            chain,
			OptimisticModeFetcher:  chain,
			SyncChecker:            &mockSync.Sync{IsSyncing: false},
			PayloadIDCache:         cache.NewPayloadIDCache(),
			TrackedValidatorsCache: cache.NewTrackedValidatorsCache(),
		}

		request := httptest.NewRequest(http.MethodGet, "http://www.example.com/eth/v1/validator/duties/proposer/{epoch}", nil)
		request = mux.SetURLVars(request, map[string]string{"epoch": "0"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetProposerDuties(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetProposerDutiesResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, true, resp.ExecutionOptimistic)
	})
	t.Run("sync not ready", func(t *testing.T) {
		st, err := util.NewBeaconState()
		require.NoError(t, err)
		chainService := &mockChain.ChainService{State: st}
		s := &Server{
			SyncChecker:           &mockSync.Sync{IsSyncing: true},
			HeadFetcher:           chainService,
			TimeFetcher:           chainService,
			OptimisticModeFetcher: chainService,
		}

		request := httptest.NewRequest(http.MethodGet, "http://www.example.com/eth/v1/validator/duties/proposer/{epoch}", nil)
		request = mux.SetURLVars(request, map[string]string{"epoch": "0"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetProposerDuties(writer, request)
		assert.Equal(t, http.StatusServiceUnavailable, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusServiceUnavailable, e.Code)
	})
}

func TestGetSyncCommitteeDuties(t *testing.T) {
	helpers.ClearCache()
	params.SetupTestConfigCleanup(t)
	cfg := params.BeaconConfig()
	cfg.AltairForkEpoch = 0
	params.OverrideBeaconConfig(cfg)

	genesisTime := time.Now()
	numVals := uint64(11)
	st, _ := util.DeterministicGenesisStateAltair(t, numVals)
	require.NoError(t, st.SetGenesisTime(uint64(genesisTime.Unix())))
	vals := st.Validators()
	currCommittee := &ethpbalpha.SyncCommittee{}
	for i := 0; i < 5; i++ {
		currCommittee.Pubkeys = append(currCommittee.Pubkeys, vals[i].PublicKey)
		currCommittee.AggregatePubkey = make([]byte, 48)
	}
	// add one public key twice - this is needed for one of the test cases
	currCommittee.Pubkeys = append(currCommittee.Pubkeys, vals[0].PublicKey)
	require.NoError(t, st.SetCurrentSyncCommittee(currCommittee))
	nextCommittee := &ethpbalpha.SyncCommittee{}
	for i := 5; i < 10; i++ {
		nextCommittee.Pubkeys = append(nextCommittee.Pubkeys, vals[i].PublicKey)
		nextCommittee.AggregatePubkey = make([]byte, 48)

	}
	require.NoError(t, st.SetNextSyncCommittee(nextCommittee))

	mockChainService := &mockChain.ChainService{Genesis: genesisTime}
	s := &Server{
		Stater:                &testutil.MockStater{BeaconState: st},
		SyncChecker:           &mockSync.Sync{IsSyncing: false},
		TimeFetcher:           mockChainService,
		HeadFetcher:           mockChainService,
		OptimisticModeFetcher: mockChainService,
	}

	t.Run("single validator", func(t *testing.T) {
		cache.SyncSubnetIDs.EmptyAllCaches()

		var body bytes.Buffer
		_, err := body.WriteString("[\"1\"]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodGet, "http://www.example.com/eth/v1/validator/duties/sync/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": "0"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetSyncCommitteeDuties(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetSyncCommitteeDutiesResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 1, len(resp.Data))
		duty := resp.Data[0]
		assert.Equal(t, hexutil.Encode(vals[1].PublicKey), duty.Pubkey)
		assert.Equal(t, "1", duty.ValidatorIndex)
		require.Equal(t, 1, len(duty.ValidatorSyncCommitteeIndices))
		assert.Equal(t, "1", duty.ValidatorSyncCommitteeIndices[0])
		subnetId, _, ok, _ := cache.SyncSubnetIDs.GetSyncCommitteeSubnets(vals[1].PublicKey, 0)
		require.Equal(t, true, ok)
		assert.Equal(t, 1, len(subnetId))
	})
	t.Run("multiple validators", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString("[\"1\",\"2\"]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodGet, "http://www.example.com/eth/v1/validator/duties/sync/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": "0"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetSyncCommitteeDuties(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetSyncCommitteeDutiesResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 2, len(resp.Data))
	})
	t.Run("no body", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodPost, "http://www.example.com/eth/v1/validator/duties/sync/{epoch}", nil)
		request = mux.SetURLVars(request, map[string]string{"epoch": "0"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetSyncCommitteeDuties(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.StringContains(t, "No data submitted", e.Message)
	})
	t.Run("empty", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString("[]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://www.example.com/eth/v1/validator/duties/sync/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": "0"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetSyncCommitteeDuties(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.StringContains(t, "No data submitted", e.Message)
	})
	t.Run("invalid", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString("[\"foo\"]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://www.example.com/eth/v1/validator/duties/sync/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": "0"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetSyncCommitteeDuties(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
	})
	t.Run("validator without duty not returned", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString("[\"1\",\"10\"]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodGet, "http://www.example.com/eth/v1/validator/duties/sync/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": "0"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetSyncCommitteeDuties(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetSyncCommitteeDutiesResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 1, len(resp.Data))
		assert.Equal(t, "1", resp.Data[0].ValidatorIndex)
	})
	t.Run("multiple indices for validator", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString("[\"0\"]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodGet, "http://www.example.com/eth/v1/validator/duties/sync/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": "0"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetSyncCommitteeDuties(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetSyncCommitteeDutiesResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		duty := resp.Data[0]
		require.Equal(t, 2, len(duty.ValidatorSyncCommitteeIndices))
		assert.DeepEqual(t, []string{"0", "5"}, duty.ValidatorSyncCommitteeIndices)
	})
	t.Run("validator index out of bound", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString(fmt.Sprintf("[\"%d\"]", numVals))
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodGet, "http://www.example.com/eth/v1/validator/duties/sync/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": "0"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetSyncCommitteeDuties(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.StringContains(t, "Invalid validator index", e.Message)
	})
	t.Run("next sync committee period", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString("[\"5\"]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodGet, "http://www.example.com/eth/v1/validator/duties/sync/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": strconv.FormatUint(uint64(params.BeaconConfig().EpochsPerSyncCommitteePeriod), 10)})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetSyncCommitteeDuties(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetSyncCommitteeDutiesResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 1, len(resp.Data))
		duty := resp.Data[0]
		assert.Equal(t, hexutil.Encode(vals[5].PublicKey), duty.Pubkey)
		assert.Equal(t, "5", duty.ValidatorIndex)
		require.Equal(t, 1, len(duty.ValidatorSyncCommitteeIndices))
		assert.Equal(t, "0", duty.ValidatorSyncCommitteeIndices[0])
	})
	t.Run("correct sync committee is fetched", func(t *testing.T) {
		// in this test we swap validators in the current and next sync committee inside the new state

		newSyncPeriodStartSlot := primitives.Slot(uint64(params.BeaconConfig().EpochsPerSyncCommitteePeriod) * uint64(params.BeaconConfig().SlotsPerEpoch))
		newSyncPeriodSt, _ := util.DeterministicGenesisStateAltair(t, numVals)
		require.NoError(t, newSyncPeriodSt.SetSlot(newSyncPeriodStartSlot))
		require.NoError(t, newSyncPeriodSt.SetGenesisTime(uint64(genesisTime.Unix())))
		vals := newSyncPeriodSt.Validators()
		currCommittee := &ethpbalpha.SyncCommittee{}
		for i := 5; i < 10; i++ {
			currCommittee.Pubkeys = append(currCommittee.Pubkeys, vals[i].PublicKey)
			currCommittee.AggregatePubkey = make([]byte, 48)
		}
		require.NoError(t, newSyncPeriodSt.SetCurrentSyncCommittee(currCommittee))
		nextCommittee := &ethpbalpha.SyncCommittee{}
		for i := 0; i < 5; i++ {
			nextCommittee.Pubkeys = append(nextCommittee.Pubkeys, vals[i].PublicKey)
			nextCommittee.AggregatePubkey = make([]byte, 48)

		}
		require.NoError(t, newSyncPeriodSt.SetNextSyncCommittee(nextCommittee))

		stateFetchFn := func(slot primitives.Slot) state.BeaconState {
			if slot < newSyncPeriodStartSlot {
				return st
			} else {
				return newSyncPeriodSt
			}
		}
		mockChainService := &mockChain.ChainService{Genesis: genesisTime, Slot: &newSyncPeriodStartSlot}
		s := &Server{
			Stater:                &testutil.MockStater{BeaconState: stateFetchFn(newSyncPeriodStartSlot)},
			SyncChecker:           &mockSync.Sync{IsSyncing: false},
			TimeFetcher:           mockChainService,
			HeadFetcher:           mockChainService,
			OptimisticModeFetcher: mockChainService,
		}

		var body bytes.Buffer
		_, err := body.WriteString("[\"8\"]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodGet, "http://www.example.com/eth/v1/validator/duties/sync/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": strconv.FormatUint(uint64(params.BeaconConfig().EpochsPerSyncCommitteePeriod), 10)})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetSyncCommitteeDuties(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetSyncCommitteeDutiesResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 1, len(resp.Data))
		duty := resp.Data[0]
		assert.Equal(t, hexutil.Encode(vals[8].PublicKey), duty.Pubkey)
		assert.Equal(t, "8", duty.ValidatorIndex)
		require.Equal(t, 1, len(duty.ValidatorSyncCommitteeIndices))
		assert.Equal(t, "3", duty.ValidatorSyncCommitteeIndices[0])
	})
	t.Run("epoch not at period start", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString("[\"1\"]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodGet, "http://www.example.com/eth/v1/validator/duties/sync/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": "1"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetSyncCommitteeDuties(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetSyncCommitteeDutiesResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 1, len(resp.Data))
		duty := resp.Data[0]
		assert.Equal(t, hexutil.Encode(vals[1].PublicKey), duty.Pubkey)
		assert.Equal(t, "1", duty.ValidatorIndex)
		require.Equal(t, 1, len(duty.ValidatorSyncCommitteeIndices))
		assert.Equal(t, "1", duty.ValidatorSyncCommitteeIndices[0])
	})
	t.Run("epoch too far in the future", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString("[\"5\"]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodGet, "http://www.example.com/eth/v1/validator/duties/sync/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": strconv.FormatUint(uint64(params.BeaconConfig().EpochsPerSyncCommitteePeriod*2), 10)})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetSyncCommitteeDuties(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.StringContains(t, "Epoch is too far in the future", e.Message)
	})
	t.Run("execution optimistic", func(t *testing.T) {
		ctx := context.Background()
		db := dbutil.SetupDB(t)
		require.NoError(t, db.SaveStateSummary(ctx, &ethpbalpha.StateSummary{Slot: 0, Root: []byte("root")}))
		require.NoError(t, db.SaveLastValidatedCheckpoint(ctx, &ethpbalpha.Checkpoint{Epoch: 0, Root: []byte("root")}))

		parentRoot := [32]byte{'a'}
		blk := util.NewBeaconBlock()
		blk.Block.ParentRoot = parentRoot[:]
		root, err := blk.Block.HashTreeRoot()
		require.NoError(t, err)
		util.SaveBlock(t, ctx, db, blk)
		require.NoError(t, db.SaveGenesisBlockRoot(ctx, root))

		slot, err := slots.EpochStart(1)
		require.NoError(t, err)

		st2, err := util.NewBeaconStateBellatrix()
		require.NoError(t, err)
		require.NoError(t, st2.SetSlot(slot))

		mockChainService := &mockChain.ChainService{
			Genesis:    genesisTime,
			Optimistic: true,
			Slot:       &slot,
			FinalizedCheckPoint: &ethpbalpha.Checkpoint{
				Root:  root[:],
				Epoch: 1,
			},
			State: st2,
		}
		s := &Server{
			Stater:                &testutil.MockStater{BeaconState: st},
			SyncChecker:           &mockSync.Sync{IsSyncing: false},
			TimeFetcher:           mockChainService,
			HeadFetcher:           mockChainService,
			OptimisticModeFetcher: mockChainService,
			ChainInfoFetcher:      mockChainService,
			BeaconDB:              db,
		}

		var body bytes.Buffer
		_, err = body.WriteString("[\"1\"]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodGet, "http://www.example.com/eth/v1/validator/duties/sync/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": "1"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetSyncCommitteeDuties(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetSyncCommitteeDutiesResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, true, resp.ExecutionOptimistic)
	})
	t.Run("sync not ready", func(t *testing.T) {
		st, err := util.NewBeaconState()
		require.NoError(t, err)
		chainService := &mockChain.ChainService{State: st}
		s := &Server{
			SyncChecker:           &mockSync.Sync{IsSyncing: true},
			HeadFetcher:           chainService,
			TimeFetcher:           chainService,
			OptimisticModeFetcher: chainService,
		}

		request := httptest.NewRequest(http.MethodGet, "http://www.example.com/eth/v1/validator/duties/sync/{epoch}", nil)
		request = mux.SetURLVars(request, map[string]string{"epoch": "1"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetSyncCommitteeDuties(writer, request)
		assert.Equal(t, http.StatusServiceUnavailable, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusServiceUnavailable, e.Code)
	})
}

func TestPrepareBeaconProposer(t *testing.T) {
	tests := []struct {
		name    string
		request []*shared.FeeRecipient
		code    int
		wantErr string
	}{
		{
			name: "Happy Path",
			request: []*shared.FeeRecipient{{
				FeeRecipient:   "0xb698D697092822185bF0311052215d5B5e1F3934",
				ValidatorIndex: "1",
			},
			},
			code:    http.StatusOK,
			wantErr: "",
		},
		{
			name: "invalid fee recipient length",
			request: []*shared.FeeRecipient{{
				FeeRecipient:   "0xb698D697092822185bF0311052",
				ValidatorIndex: "1",
			},
			},
			code:    http.StatusBadRequest,
			wantErr: "Invalid fee_recipient",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			b, err := json.Marshal(tt.request)
			require.NoError(t, err)
			var body bytes.Buffer
			_, err = body.WriteString(string(b))
			require.NoError(t, err)
			url := "http://example.com/eth/v1/validator/prepare_beacon_proposer"
			request := httptest.NewRequest(http.MethodPost, url, &body)
			writer := httptest.NewRecorder()
			db := dbutil.SetupDB(t)
			server := &Server{
				BeaconDB:               db,
				TrackedValidatorsCache: cache.NewTrackedValidatorsCache(),
				PayloadIDCache:         cache.NewPayloadIDCache(),
			}
			server.PrepareBeaconProposer(writer, request)
			require.Equal(t, tt.code, writer.Code)
			if tt.wantErr != "" {
				require.Equal(t, strings.Contains(writer.Body.String(), tt.wantErr), true)
			} else {
				require.NoError(t, err)
				feebytes, err := hexutil.Decode(tt.request[0].FeeRecipient)
				require.NoError(t, err)
				index, err := strconv.ParseUint(tt.request[0].ValidatorIndex, 10, 64)
				require.NoError(t, err)
				val, tracked := server.TrackedValidatorsCache.Validator(primitives.ValidatorIndex(index))
				require.Equal(t, true, tracked)
				require.Equal(t, primitives.ExecutionAddress(feebytes), val.FeeRecipient)
			}
		})
	}
}

func TestProposer_PrepareBeaconProposerOverlapping(t *testing.T) {
	hook := logTest.NewGlobal()
	db := dbutil.SetupDB(t)

	// New validator
	proposerServer := &Server{
		BeaconDB:               db,
		TrackedValidatorsCache: cache.NewTrackedValidatorsCache(),
		PayloadIDCache:         cache.NewPayloadIDCache(),
	}
	req := []*shared.FeeRecipient{{
		FeeRecipient:   hexutil.Encode(bytesutil.PadTo([]byte{0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF}, fieldparams.FeeRecipientLength)),
		ValidatorIndex: "1",
	}}
	b, err := json.Marshal(req)
	require.NoError(t, err)
	var body bytes.Buffer
	_, err = body.WriteString(string(b))
	require.NoError(t, err)
	url := "http://example.com/eth/v1/validator/prepare_beacon_proposer"
	request := httptest.NewRequest(http.MethodPost, url, &body)
	writer := httptest.NewRecorder()

	proposerServer.PrepareBeaconProposer(writer, request)
	require.Equal(t, http.StatusOK, writer.Code)
	require.LogsContain(t, hook, "Updated fee recipient addresses")

	// Same validator
	hook.Reset()
	_, err = body.WriteString(string(b))
	require.NoError(t, err)
	request = httptest.NewRequest(http.MethodPost, url, &body)
	writer = httptest.NewRecorder()
	proposerServer.PrepareBeaconProposer(writer, request)
	require.Equal(t, http.StatusOK, writer.Code)
	require.LogsContain(t, hook, "Updated fee recipient addresses")

	// Same validator with different fee recipient
	hook.Reset()
	req = []*shared.FeeRecipient{{
		FeeRecipient:   hexutil.Encode(bytesutil.PadTo([]byte{0x01, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF}, fieldparams.FeeRecipientLength)),
		ValidatorIndex: "1",
	}}
	b, err = json.Marshal(req)
	require.NoError(t, err)
	_, err = body.WriteString(string(b))
	require.NoError(t, err)
	request = httptest.NewRequest(http.MethodPost, url, &body)
	writer = httptest.NewRecorder()
	proposerServer.PrepareBeaconProposer(writer, request)
	require.Equal(t, http.StatusOK, writer.Code)
	require.LogsContain(t, hook, "Updated fee recipient addresses")

	// More than one validator
	hook.Reset()
	req = []*shared.FeeRecipient{
		{
			FeeRecipient:   hexutil.Encode(bytesutil.PadTo([]byte{0x01, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF}, fieldparams.FeeRecipientLength)),
			ValidatorIndex: "1",
		},
		{
			FeeRecipient:   hexutil.Encode(bytesutil.PadTo([]byte{0x01, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF}, fieldparams.FeeRecipientLength)),
			ValidatorIndex: "2",
		},
	}
	b, err = json.Marshal(req)
	require.NoError(t, err)
	_, err = body.WriteString(string(b))
	require.NoError(t, err)
	request = httptest.NewRequest(http.MethodPost, url, &body)
	writer = httptest.NewRecorder()
	proposerServer.PrepareBeaconProposer(writer, request)
	require.Equal(t, http.StatusOK, writer.Code)
	require.LogsContain(t, hook, "Updated fee recipient addresses")

	// Same validators
	hook.Reset()
	b, err = json.Marshal(req)
	require.NoError(t, err)
	_, err = body.WriteString(string(b))
	require.NoError(t, err)
	request = httptest.NewRequest(http.MethodPost, url, &body)
	writer = httptest.NewRecorder()
	proposerServer.PrepareBeaconProposer(writer, request)
	require.Equal(t, http.StatusOK, writer.Code)
	require.LogsContain(t, hook, "Updated fee recipient addresses")
}

func BenchmarkServer_PrepareBeaconProposer(b *testing.B) {
	db := dbutil.SetupDB(b)
	proposerServer := &Server{
		BeaconDB:               db,
		TrackedValidatorsCache: cache.NewTrackedValidatorsCache(),
		PayloadIDCache:         cache.NewPayloadIDCache(),
	}
	f := bytesutil.PadTo([]byte{0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF, 0x01, 0xFF}, fieldparams.FeeRecipientLength)
	recipients := make([]*shared.FeeRecipient, 0)
	for i := 0; i < 10000; i++ {
		recipients = append(recipients, &shared.FeeRecipient{FeeRecipient: hexutil.Encode(f), ValidatorIndex: fmt.Sprint(i)})
	}
	byt, err := json.Marshal(recipients)
	require.NoError(b, err)
	var body bytes.Buffer
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, err = body.WriteString(string(byt))
		require.NoError(b, err)
		url := "http://example.com/eth/v1/validator/prepare_beacon_proposer"
		request := httptest.NewRequest(http.MethodPost, url, &body)
		writer := httptest.NewRecorder()
		proposerServer.PrepareBeaconProposer(writer, request)
		if writer.Code != http.StatusOK {
			b.Fatal()
		}
	}
}

func TestGetLiveness(t *testing.T) {
	// Setup:
	// Epoch 0 - both validators not live
	// Epoch 1 - validator with index 1 is live
	// Epoch 2 - validator with index 0 is live
	oldSt, err := util.NewBeaconStateBellatrix()
	require.NoError(t, err)
	require.NoError(t, oldSt.AppendCurrentParticipationBits(0))
	require.NoError(t, oldSt.AppendCurrentParticipationBits(0))
	headSt, err := util.NewBeaconStateBellatrix()
	require.NoError(t, err)
	require.NoError(t, headSt.SetSlot(params.BeaconConfig().SlotsPerEpoch*2))
	require.NoError(t, headSt.AppendPreviousParticipationBits(0))
	require.NoError(t, headSt.AppendPreviousParticipationBits(1))
	require.NoError(t, headSt.AppendCurrentParticipationBits(1))
	require.NoError(t, headSt.AppendCurrentParticipationBits(0))

	s := &Server{
		HeadFetcher: &mockChain.ChainService{State: headSt},
		Stater: &testutil.MockStater{
			// We configure states for last slots of an epoch
			StatesBySlot: map[primitives.Slot]state.BeaconState{
				params.BeaconConfig().SlotsPerEpoch - 1:   oldSt,
				params.BeaconConfig().SlotsPerEpoch*3 - 1: headSt,
			},
		},
	}

	t.Run("old epoch", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString("[\"0\",\"1\"]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com/eth/v1/validator/liveness/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": "0"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetLiveness(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetLivenessResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.NotNil(t, resp.Data)
		data0 := resp.Data[0]
		data1 := resp.Data[1]
		assert.Equal(t, true, (data0.Index == "0" && !data0.IsLive) || (data0.Index == "1" && !data0.IsLive))
		assert.Equal(t, true, (data1.Index == "0" && !data1.IsLive) || (data1.Index == "1" && !data1.IsLive))
	})
	t.Run("previous epoch", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString("[\"0\",\"1\"]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com/eth/v1/validator/liveness/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": "1"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetLiveness(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetLivenessResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.NotNil(t, resp.Data)
		data0 := resp.Data[0]
		data1 := resp.Data[1]
		assert.Equal(t, true, (data0.Index == "0" && !data0.IsLive) || (data0.Index == "1" && data0.IsLive))
		assert.Equal(t, true, (data1.Index == "0" && !data1.IsLive) || (data1.Index == "1" && data1.IsLive))
	})
	t.Run("current epoch", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString("[\"0\",\"1\"]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com/eth/v1/validator/liveness/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": "2"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetLiveness(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &GetLivenessResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.NotNil(t, resp.Data)
		data0 := resp.Data[0]
		data1 := resp.Data[1]
		assert.Equal(t, true, (data0.Index == "0" && data0.IsLive) || (data0.Index == "1" && !data0.IsLive))
		assert.Equal(t, true, (data1.Index == "0" && data1.IsLive) || (data1.Index == "1" && !data1.IsLive))
	})
	t.Run("future epoch", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString("[\"0\",\"1\"]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com/eth/v1/validator/liveness/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": "3"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetLiveness(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		require.StringContains(t, "Requested epoch cannot be in the future", e.Message)
	})
	t.Run("no epoch provided", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString("[\"0\",\"1\"]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com/eth/v1/validator/liveness/{epoch}", &body)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetLiveness(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "epoch is required"))
	})
	t.Run("invalid epoch provided", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString("[\"0\",\"1\"]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com/eth/v1/validator/liveness/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": "foo"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetLiveness(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.Equal(t, true, strings.Contains(e.Message, "epoch is invalid"))
	})
	t.Run("no body", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodPost, "http://example.com/eth/v1/validator/liveness/{epoch}", nil)
		request = mux.SetURLVars(request, map[string]string{"epoch": "3"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetLiveness(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.StringContains(t, "No data submitted", e.Message)
	})
	t.Run("empty", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString("[]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com/eth/v1/validator/liveness/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": "3"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetLiveness(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.StringContains(t, "No data submitted", e.Message)
	})
	t.Run("unknown validator index", func(t *testing.T) {
		var body bytes.Buffer
		_, err := body.WriteString("[\"0\",\"1\",\"2\"]")
		require.NoError(t, err)
		request := httptest.NewRequest(http.MethodPost, "http://example.com/eth/v1/validator/liveness/{epoch}", &body)
		request = mux.SetURLVars(request, map[string]string{"epoch": "0"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetLiveness(writer, request)
		assert.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		require.StringContains(t, "Validator index 2 is invalid", e.Message)
	})
}

var (
	singleContribution = `[
  {
    "message": {
      "aggregator_index": "1",
      "selection_proof": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505",
      "contribution": {
        "slot": "1",
        "beacon_block_root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2",
        "subcommittee_index": "1",
        "aggregation_bits": "0x01",
        "signature": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
      }
    },
    "signature": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
  }
]`
	multipleContributions = `[
  {
    "message": {
      "aggregator_index": "1",
      "selection_proof": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505",
      "contribution": {
        "slot": "1",
        "beacon_block_root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2",
        "subcommittee_index": "1",
        "aggregation_bits": "0x01",
        "signature": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
      }
    },
    "signature": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
  },
  {
    "message": {
      "aggregator_index": "1",
      "selection_proof": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505",
      "contribution": {
        "slot": "1",
        "beacon_block_root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2",
        "subcommittee_index": "1",
        "aggregation_bits": "0x01",
        "signature": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
      }
    },
    "signature": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
  }
]`
	// aggregator_index is invalid
	invalidContribution = `[
  {
    "message": {
      "aggregator_index": "foo",
      "selection_proof": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505",
      "contribution": {
        "slot": "1",
        "beacon_block_root": "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2",
        "subcommittee_index": "1",
        "aggregation_bits": "0x01",
        "signature": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
      }
    },
    "signature": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
  }
]`
	singleAggregate = `[
  {
    "message": {
      "aggregator_index": "1",
      "aggregate": {
        "aggregation_bits": "0x01",
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
      "selection_proof": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
    },
    "signature": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
  }
]`
	multipleAggregates = `[
  {
    "message": {
      "aggregator_index": "1",
      "aggregate": {
        "aggregation_bits": "0x01",
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
      "selection_proof": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
    },
    "signature": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
  },
{
    "message": {
      "aggregator_index": "1",
      "aggregate": {
        "aggregation_bits": "0x01",
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
      "selection_proof": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
    },
    "signature": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
  }
]
`
	// aggregator_index is invalid
	invalidAggregate = `[
  {
    "message": {
      "aggregator_index": "foo",
      "aggregate": {
        "aggregation_bits": "0x01",
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
      "selection_proof": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
    },
    "signature": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
  }
]`
	singleSyncCommitteeSubscription = `[
  {
    "validator_index": "1",
    "sync_committee_indices": [
      "0",
      "2"
    ],
    "until_epoch": "1"
  }
]`
	singleSyncCommitteeSubscription2 = `[
  {
    "validator_index": "0",
    "sync_committee_indices": [
      "0",
      "2"
    ],
    "until_epoch": "0"
  }
]`
	singleSyncCommitteeSubscription3 = fmt.Sprintf(`[
  {
    "validator_index": "0",
    "sync_committee_indices": [
      "0",
      "2"
    ],
    "until_epoch": "%d"
  }
]`, 2*params.BeaconConfig().EpochsPerSyncCommitteePeriod)
	singleSyncCommitteeSubscription4 = fmt.Sprintf(`[
  {
    "validator_index": "0",
    "sync_committee_indices": [
      "0",
      "2"
    ],
    "until_epoch": "%d"
  }
]`, 2*params.BeaconConfig().EpochsPerSyncCommitteePeriod+1)
	multipleSyncCommitteeSubscription = `[
  {
    "validator_index": "0",
    "sync_committee_indices": [
      "0"
    ],
    "until_epoch": "1"
  },
  {
    "validator_index": "1",
    "sync_committee_indices": [
      "2"
    ],
    "until_epoch": "1"
  }
]`
	// validator_index is invalid
	invalidSyncCommitteeSubscription = `[
  {
    "validator_index": "foo",
    "sync_committee_indices": [
      "0",
      "2"
    ],
    "until_epoch": "1"
  }
]`
	singleBeaconCommitteeContribution = `[
  {
    "validator_index": "1",
    "committee_index": "1",
    "committees_at_slot": "2",
    "slot": "1",
    "is_aggregator": false
  }
]`
	singleBeaconCommitteeContribution2 = `[
  {
    "validator_index": "1",
    "committee_index": "1",
    "committees_at_slot": "2",
    "slot": "1",
    "is_aggregator": true
  }
]`
	multipleBeaconCommitteeContribution = `[
  {
    "validator_index": "1",
    "committee_index": "1",
    "committees_at_slot": "2",
    "slot": "1",
    "is_aggregator": false
  },
  {
    "validator_index": "2",
    "committee_index": "0",
    "committees_at_slot": "2",
    "slot": "1",
    "is_aggregator": false
  }
]`
	multipleBeaconCommitteeContribution2 = `[
  {
    "validator_index": "1",
    "committee_index": "1",
    "committees_at_slot": "2",
    "slot": "1",
    "is_aggregator": true
  },
  {
    "validator_index": "2",
    "committee_index": "1",
    "committees_at_slot": "2",
    "slot": "1",
    "is_aggregator": false
  }
]`
	// validator_index is invalid
	invalidBeaconCommitteeContribution = `[
  {
    "validator_index": "foo",
    "committee_index": "1",
    "committees_at_slot": "2",
    "slot": "1",
    "is_aggregator": false
  }
]`
	registrations = `[{
    "message": {
      "fee_recipient": "0xabcf8e0d4e9587369b2301d0790347320302cc09",
      "gas_limit": "1",
      "timestamp": "1",
      "pubkey": "0x93247f2209abcacf57b75a51dafae777f9dd38bc7053d1af526f220a7489a6d3a2753e5f3e8b1cfe39b56f43611df74a"
    },
    "signature": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
  },{
    "message": {
      "fee_recipient": "0xabcf8e0d4e9587369b2301d0790347320302cc09",
      "gas_limit": "1",
      "timestamp": "1",
      "pubkey": "0x93247f2209abcacf57b75a51dafae777f9dd38bc7053d1af526f220a7489a6d3a2753e5f3e8b1cfe39b56f43611df74a"
    },
    "signature": "0x1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505cc411d61252fb6cb3fa0017b679f8bb2305b26a285fa2737f175668d0dff91cc1b66ac1fb663c9bc59509846d6ec05345bd908eda73e670af888da41af171505"
  }]`
)

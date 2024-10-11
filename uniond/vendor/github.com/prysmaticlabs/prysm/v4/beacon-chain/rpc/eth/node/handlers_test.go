package node

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"runtime"
	"testing"

	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/enr"
	"github.com/libp2p/go-libp2p/core/peer"
	ma "github.com/multiformats/go-multiaddr"
	"github.com/prysmaticlabs/go-bitfield"
	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	mockp2p "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/testutil"
	syncmock "github.com/prysmaticlabs/prysm/v4/beacon-chain/sync/initial-sync/testing"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/wrapper"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	pb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

type dummyIdentity enode.ID

func (_ dummyIdentity) Verify(_ *enr.Record, _ []byte) error { return nil }
func (id dummyIdentity) NodeAddr(_ *enr.Record) []byte       { return id[:] }

func TestSyncStatus(t *testing.T) {
	currentSlot := new(primitives.Slot)
	*currentSlot = 110
	state, err := util.NewBeaconState()
	require.NoError(t, err)
	err = state.SetSlot(100)
	require.NoError(t, err)
	chainService := &mock.ChainService{Slot: currentSlot, State: state, Optimistic: true}
	syncChecker := &syncmock.Sync{}
	syncChecker.IsSyncing = true

	s := &Server{
		HeadFetcher:               chainService,
		GenesisTimeFetcher:        chainService,
		OptimisticModeFetcher:     chainService,
		SyncChecker:               syncChecker,
		ExecutionChainInfoFetcher: &testutil.MockExecutionChainInfoFetcher{},
	}

	request := httptest.NewRequest(http.MethodGet, "http://example.com", nil)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.GetSyncStatus(writer, request)
	assert.Equal(t, http.StatusOK, writer.Code)
	resp := &SyncStatusResponse{}
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
	require.NotNil(t, resp)
	assert.Equal(t, "100", resp.Data.HeadSlot)
	assert.Equal(t, "10", resp.Data.SyncDistance)
	assert.Equal(t, true, resp.Data.IsSyncing)
	assert.Equal(t, true, resp.Data.IsOptimistic)
	assert.Equal(t, false, resp.Data.ElOffline)
}

func TestGetVersion(t *testing.T) {
	semVer := version.SemanticVersion()
	os := runtime.GOOS
	arch := runtime.GOARCH

	request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/node/version", nil)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s := &Server{}
	s.GetVersion(writer, request)
	assert.Equal(t, http.StatusOK, writer.Code)
	resp := &GetVersionResponse{}
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
	assert.StringContains(t, semVer, resp.Data.Version)
	assert.StringContains(t, os, resp.Data.Version)
	assert.StringContains(t, arch, resp.Data.Version)
}

func TestGetHealth(t *testing.T) {
	checker := &syncmock.Sync{}
	s := &Server{
		SyncChecker: checker,
	}

	request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/node/health", nil)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}
	s.GetHealth(writer, request)
	assert.Equal(t, http.StatusServiceUnavailable, writer.Code)

	checker.IsInitialized = true
	request = httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/node/health", nil)
	writer = httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}
	s.GetHealth(writer, request)
	assert.Equal(t, http.StatusPartialContent, writer.Code)

	request = httptest.NewRequest(http.MethodGet, fmt.Sprintf("http://example.com/eth/v1/node/health?syncing_status=%d", http.StatusPaymentRequired), nil)
	writer = httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}
	s.GetHealth(writer, request)
	assert.Equal(t, http.StatusPaymentRequired, writer.Code)

	checker.IsSynced = true
	request = httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/node/health", nil)
	writer = httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}
	s.GetHealth(writer, request)
	assert.Equal(t, http.StatusOK, writer.Code)
}

func TestGetIdentity(t *testing.T) {
	p2pAddr, err := ma.NewMultiaddr("/ip4/7.7.7.7/udp/30303")
	require.NoError(t, err)
	discAddr1, err := ma.NewMultiaddr("/ip4/7.7.7.7/udp/30303/p2p/QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N")
	require.NoError(t, err)
	discAddr2, err := ma.NewMultiaddr("/ip6/1:2:3:4:5:6:7:8/udp/20202/p2p/QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N")
	require.NoError(t, err)
	enrRecord := &enr.Record{}
	err = enrRecord.SetSig(dummyIdentity{1}, []byte{42})
	require.NoError(t, err)
	enrRecord.Set(enr.IPv4{7, 7, 7, 7})
	err = enrRecord.SetSig(dummyIdentity{}, []byte{})
	require.NoError(t, err)
	attnets := bitfield.NewBitvector64()
	attnets.SetBitAt(1, true)
	metadataProvider := &mockp2p.MockMetadataProvider{Data: wrapper.WrappedMetadataV0(&pb.MetaDataV0{SeqNumber: 1, Attnets: attnets})}

	t.Run("OK", func(t *testing.T) {
		peerManager := &mockp2p.MockPeerManager{
			Enr:           enrRecord,
			PID:           "foo",
			BHost:         &mockp2p.MockHost{Addresses: []ma.Multiaddr{p2pAddr}},
			DiscoveryAddr: []ma.Multiaddr{discAddr1, discAddr2},
		}
		s := &Server{
			PeerManager:      peerManager,
			MetadataProvider: metadataProvider,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/node/identity", nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetIdentity(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		resp := &GetIdentityResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		expectedID := peer.ID("foo").String()
		assert.Equal(t, expectedID, resp.Data.PeerId)
		expectedEnr, err := p2p.SerializeENR(enrRecord)
		require.NoError(t, err)
		assert.Equal(t, fmt.Sprint("enr:", expectedEnr), resp.Data.Enr)
		require.Equal(t, 1, len(resp.Data.P2PAddresses))
		assert.Equal(t, p2pAddr.String()+"/p2p/"+expectedID, resp.Data.P2PAddresses[0])
		require.Equal(t, 2, len(resp.Data.DiscoveryAddresses))
		ipv4Found, ipv6Found := false, false
		for _, address := range resp.Data.DiscoveryAddresses {
			if address == discAddr1.String() {
				ipv4Found = true
			} else if address == discAddr2.String() {
				ipv6Found = true
			}
		}
		assert.Equal(t, true, ipv4Found, "IPv4 discovery address not found")
		assert.Equal(t, true, ipv6Found, "IPv6 discovery address not found")
		assert.Equal(t, discAddr1.String(), resp.Data.DiscoveryAddresses[0])
		assert.Equal(t, discAddr2.String(), resp.Data.DiscoveryAddresses[1])
	})

	t.Run("ENR failure", func(t *testing.T) {
		peerManager := &mockp2p.MockPeerManager{
			Enr:           &enr.Record{},
			PID:           "foo",
			BHost:         &mockp2p.MockHost{Addresses: []ma.Multiaddr{p2pAddr}},
			DiscoveryAddr: []ma.Multiaddr{discAddr1, discAddr2},
		}
		s := &Server{
			PeerManager:      peerManager,
			MetadataProvider: metadataProvider,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/node/identity", nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetIdentity(writer, request)
		require.Equal(t, http.StatusInternalServerError, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusInternalServerError, e.Code)
		assert.StringContains(t, "Could not obtain enr", e.Message)
	})

	t.Run("Discovery addresses failure", func(t *testing.T) {
		peerManager := &mockp2p.MockPeerManager{
			Enr:               enrRecord,
			PID:               "foo",
			BHost:             &mockp2p.MockHost{Addresses: []ma.Multiaddr{p2pAddr}},
			DiscoveryAddr:     []ma.Multiaddr{discAddr1, discAddr2},
			FailDiscoveryAddr: true,
		}
		s := &Server{
			PeerManager:      peerManager,
			MetadataProvider: metadataProvider,
		}

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/node/identity", nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetIdentity(writer, request)
		require.Equal(t, http.StatusInternalServerError, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusInternalServerError, e.Code)
		assert.StringContains(t, "Could not obtain discovery address", e.Message)
	})
}

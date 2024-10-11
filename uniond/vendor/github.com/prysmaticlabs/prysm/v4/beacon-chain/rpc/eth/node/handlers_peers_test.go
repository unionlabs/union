package node

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strconv"
	"strings"
	"testing"

	"github.com/ethereum/go-ethereum/p2p/enr"
	"github.com/gorilla/mux"
	"github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/peer"
	libp2ptest "github.com/libp2p/go-libp2p/p2p/host/peerstore/test"
	ma "github.com/multiformats/go-multiaddr"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers"
	mockp2p "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestGetPeer(t *testing.T) {
	const rawId = "16Uiu2HAkvyYtoQXZNTsthjgLHjEnv7kvwzEmjvsJjWXpbhtqpSUN"
	decodedId, err := peer.Decode(rawId)
	require.NoError(t, err)
	enrRecord := &enr.Record{}
	err = enrRecord.SetSig(dummyIdentity{1}, []byte{42})
	require.NoError(t, err)
	enrRecord.Set(enr.IPv4{7, 7, 7, 7})
	err = enrRecord.SetSig(dummyIdentity{}, []byte{})
	require.NoError(t, err)
	const p2pAddr = "/ip4/7.7.7.7/udp/30303/p2p/QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N"
	p2pMultiAddr, err := ma.NewMultiaddr(p2pAddr)
	require.NoError(t, err)
	peerFetcher := &mockp2p.MockPeersProvider{}
	s := Server{PeersFetcher: peerFetcher}
	peerFetcher.Peers().Add(enrRecord, decodedId, p2pMultiAddr, network.DirInbound)

	t.Run("OK", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/node/peers/{peer_id}", nil)
		request = mux.SetURLVars(request, map[string]string{"peer_id": rawId})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetPeer(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		resp := &GetPeerResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		assert.Equal(t, rawId, resp.Data.PeerId)
		assert.Equal(t, p2pAddr, resp.Data.LastSeenP2PAddress)
		assert.Equal(t, "enr:yoABgmlwhAcHBwc", resp.Data.Enr)
		assert.Equal(t, "disconnected", resp.Data.State)
		assert.Equal(t, "inbound", resp.Data.Direction)
	})

	t.Run("Invalid ID", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/node/peers/{peer_id}", nil)
		request = mux.SetURLVars(request, map[string]string{"peer_id": "foo"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetPeer(writer, request)
		require.Equal(t, http.StatusBadRequest, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusBadRequest, e.Code)
		assert.StringContains(t, "Invalid peer ID", e.Message)
	})

	t.Run("Peer not found", func(t *testing.T) {
		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/node/peers/{peer_id}", nil)
		request = mux.SetURLVars(request, map[string]string{"peer_id": "16Uiu2HAmQqFdEcHbSmQTQuLoAhnMUrgoWoraKK4cUJT6FuuqHqTU"})
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetPeer(writer, request)
		require.Equal(t, http.StatusNotFound, writer.Code)
		e := &httputil.DefaultJsonError{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
		assert.Equal(t, http.StatusNotFound, e.Code)
		assert.StringContains(t, "Peer not found", e.Message)
	})
}

func TestGetPeers(t *testing.T) {
	ids := libp2ptest.GeneratePeerIDs(9)
	peerFetcher := &mockp2p.MockPeersProvider{}
	peerFetcher.ClearPeers()
	peerStatus := peerFetcher.Peers()

	for i, id := range ids {
		// Make last peer undiscovered
		if i == len(ids)-1 {
			peerStatus.Add(nil, id, nil, network.DirUnknown)
		} else {
			enrRecord := &enr.Record{}
			err := enrRecord.SetSig(dummyIdentity{1}, []byte{42})
			require.NoError(t, err)
			enrRecord.Set(enr.IPv4{127, 0, 0, byte(i)})
			err = enrRecord.SetSig(dummyIdentity{}, []byte{})
			require.NoError(t, err)
			var p2pAddr = "/ip4/127.0.0." + strconv.Itoa(i) + "/udp/30303/p2p/QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N"
			p2pMultiAddr, err := ma.NewMultiaddr(p2pAddr)
			require.NoError(t, err)

			var direction network.Direction
			if i%2 == 0 {
				direction = network.DirInbound
			} else {
				direction = network.DirOutbound
			}
			peerStatus.Add(enrRecord, id, p2pMultiAddr, direction)

			switch i {
			case 0, 1:
				peerStatus.SetConnectionState(id, peers.PeerConnecting)
			case 2, 3:
				peerStatus.SetConnectionState(id, peers.PeerConnected)
			case 4, 5:
				peerStatus.SetConnectionState(id, peers.PeerDisconnecting)
			case 6, 7:
				peerStatus.SetConnectionState(id, peers.PeerDisconnected)
			default:
				t.Fatalf("Failed to set connection state for peer")
			}
		}
	}

	s := Server{PeersFetcher: peerFetcher}

	t.Run("OK", func(t *testing.T) {
		// We will check the first peer from the list.
		expectedId := ids[0]

		request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/node/peers?state=connecting&direction=inbound", nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}

		s.GetPeers(writer, request)
		require.Equal(t, http.StatusOK, writer.Code)
		resp := &GetPeersResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		require.Equal(t, 1, len(resp.Data))
		returnedPeer := resp.Data[0]
		assert.Equal(t, expectedId.String(), returnedPeer.PeerId)
		expectedEnr, err := peerStatus.ENR(expectedId)
		require.NoError(t, err)
		serializedEnr, err := p2p.SerializeENR(expectedEnr)
		require.NoError(t, err)
		assert.Equal(t, "enr:"+serializedEnr, returnedPeer.Enr)
		expectedP2PAddr, err := peerStatus.Address(expectedId)
		require.NoError(t, err)
		assert.Equal(t, expectedP2PAddr.String(), returnedPeer.LastSeenP2PAddress)
		assert.Equal(t, "connecting", returnedPeer.State)
		assert.Equal(t, "inbound", returnedPeer.Direction)
	})

	filterTests := []struct {
		name       string
		states     []string
		directions []string
		wantIds    []peer.ID
	}{
		{
			name:       "No filters - return all peers",
			states:     []string{},
			directions: []string{},
			wantIds:    ids[:len(ids)-1], // Excluding last peer as it is not connected.
		},
		{
			name:       "State filter empty - return peers for all states",
			states:     []string{},
			directions: []string{"inbound"},
			wantIds:    []peer.ID{ids[0], ids[2], ids[4], ids[6]},
		},
		{
			name:       "Direction filter empty - return peers for all directions",
			states:     []string{"connected"},
			directions: []string{},
			wantIds:    []peer.ID{ids[2], ids[3]},
		},
		{
			name:       "One state and direction",
			states:     []string{"disconnected"},
			directions: []string{"inbound"},
			wantIds:    []peer.ID{ids[6]},
		},
		{
			name:       "Multiple states and directions",
			states:     []string{"connecting", "disconnecting"},
			directions: []string{"inbound", "outbound"},
			wantIds:    []peer.ID{ids[0], ids[1], ids[4], ids[5]},
		},
		{
			name:       "Unknown filter is ignored",
			states:     []string{"connected", "foo"},
			directions: []string{"outbound", "foo"},
			wantIds:    []peer.ID{ids[3]},
		},
		{
			name:       "Only unknown filters - return all peers",
			states:     []string{"foo"},
			directions: []string{"foo"},
			wantIds:    ids[:len(ids)-1], // Excluding last peer as it is not connected.
		},
	}
	for _, tt := range filterTests {
		t.Run(tt.name, func(t *testing.T) {
			states := strings.Join(tt.states, "&state=")
			statesQuery := ""
			if states != "" {
				statesQuery = "state=" + states
			}
			directions := strings.Join(tt.directions, "&direction=")
			directionsQuery := ""
			if directions != "" {
				directionsQuery = "direction=" + directions
			}
			request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/node/peers?"+statesQuery+"&"+directionsQuery, nil)
			writer := httptest.NewRecorder()
			writer.Body = &bytes.Buffer{}

			s.GetPeers(writer, request)
			require.Equal(t, http.StatusOK, writer.Code)
			resp := &GetPeersResponse{}
			require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
			assert.Equal(t, len(tt.wantIds), len(resp.Data), "Wrong number of peers returned")
			for _, id := range tt.wantIds {
				expectedId := id.String()
				found := false
				for _, returnedPeer := range resp.Data {
					if returnedPeer.PeerId == expectedId {
						found = true
						break
					}
				}
				if !found {
					t.Errorf("Expected ID '" + expectedId + "' not found")
				}
			}
		})
	}
}

func TestGetPeers_NoPeersReturnsEmptyArray(t *testing.T) {
	peerFetcher := &mockp2p.MockPeersProvider{}
	peerFetcher.ClearPeers()
	s := Server{PeersFetcher: peerFetcher}

	request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/node/peers?state=connecting&state=connected", nil)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}

	s.GetPeers(writer, request)
	require.Equal(t, http.StatusOK, writer.Code)
	resp := &GetPeersResponse{}
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
	assert.Equal(t, 0, len(resp.Data))
}

func TestGetPeerCount(t *testing.T) {
	ids := libp2ptest.GeneratePeerIDs(10)
	peerFetcher := &mockp2p.MockPeersProvider{}
	peerFetcher.ClearPeers()
	peerStatus := peerFetcher.Peers()

	for i, id := range ids {
		enrRecord := &enr.Record{}
		err := enrRecord.SetSig(dummyIdentity{1}, []byte{42})
		require.NoError(t, err)
		enrRecord.Set(enr.IPv4{127, 0, 0, byte(i)})
		err = enrRecord.SetSig(dummyIdentity{}, []byte{})
		require.NoError(t, err)
		var p2pAddr = "/ip4/127.0.0." + strconv.Itoa(i) + "/udp/30303/p2p/QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N"
		p2pMultiAddr, err := ma.NewMultiaddr(p2pAddr)
		require.NoError(t, err)

		var direction network.Direction
		if i%2 == 0 {
			direction = network.DirInbound
		} else {
			direction = network.DirOutbound
		}
		peerStatus.Add(enrRecord, id, p2pMultiAddr, direction)

		switch i {
		case 0:
			peerStatus.SetConnectionState(id, peers.PeerConnecting)
		case 1, 2:
			peerStatus.SetConnectionState(id, peers.PeerConnected)
		case 3, 4, 5:
			peerStatus.SetConnectionState(id, peers.PeerDisconnecting)
		case 6, 7, 8, 9:
			peerStatus.SetConnectionState(id, peers.PeerDisconnected)
		default:
			t.Fatalf("Failed to set connection state for peer")
		}
	}

	s := Server{PeersFetcher: peerFetcher}
	request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/node/peer_count", nil)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}
	s.GetPeerCount(writer, request)
	require.Equal(t, http.StatusOK, writer.Code)
	resp := &GetPeerCountResponse{}
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
	assert.Equal(t, "1", resp.Data.Connecting, "Wrong number of connecting peers")
	assert.Equal(t, "2", resp.Data.Connected, "Wrong number of connected peers")
	assert.Equal(t, "3", resp.Data.Disconnecting, "Wrong number of disconnecting peers")
	assert.Equal(t, "4", resp.Data.Disconnected, "Wrong number of disconnected peers")
}

func BenchmarkGetPeers(b *testing.B) {
	// We simulate having a lot of peers.
	ids := libp2ptest.GeneratePeerIDs(2000)
	peerFetcher := &mockp2p.MockPeersProvider{}

	for _, id := range ids {
		enrRecord := &enr.Record{}
		err := enrRecord.SetSig(dummyIdentity{1}, []byte{42})
		require.NoError(b, err)
		enrRecord.Set(enr.IPv4{7, 7, 7, 7})
		err = enrRecord.SetSig(dummyIdentity{}, []byte{})
		require.NoError(b, err)
		const p2pAddr = "/ip4/7.7.7.7/udp/30303/p2p/QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N"
		p2pMultiAddr, err := ma.NewMultiaddr(p2pAddr)
		require.NoError(b, err)
		peerFetcher.Peers().Add(enrRecord, id, p2pMultiAddr, network.DirInbound)
	}

	s := Server{PeersFetcher: peerFetcher}
	request := httptest.NewRequest(http.MethodGet, "http://example.com/eth/v1/node/peers", nil)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}
	s.GetPeers(writer, request)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		s.GetPeers(writer, request)
	}
}

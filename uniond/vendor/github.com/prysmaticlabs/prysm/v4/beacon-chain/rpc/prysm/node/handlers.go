package node

import (
	"encoding/json"
	"io"
	"net/http"
	"strings"

	corenet "github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers/peerdata"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"go.opencensus.io/trace"
)

// ListTrustedPeer retrieves data about the node's trusted peers.
func (s *Server) ListTrustedPeer(w http.ResponseWriter, r *http.Request) {
	_, span := trace.StartSpan(r.Context(), "node.ListTrustedPeer")
	defer span.End()

	peerStatus := s.PeersFetcher.Peers()
	allIds := s.PeersFetcher.Peers().GetTrustedPeers()
	allPeers := make([]*Peer, 0, len(allIds))
	for _, id := range allIds {
		p, err := httpPeerInfo(peerStatus, id)
		if err != nil {
			errJson := &httputil.DefaultJsonError{
				Message: errors.Wrapf(err, "Could not get peer info").Error(),
				Code:    http.StatusInternalServerError,
			}
			httputil.WriteError(w, errJson)
			return
		}
		// peers added into trusted set but never connected should also be listed
		if p == nil {
			p = &Peer{
				PeerID:             id.String(),
				Enr:                "",
				LastSeenP2PAddress: "",
				State:              eth.ConnectionState(corenet.NotConnected).String(),
				Direction:          eth.PeerDirection(corenet.DirUnknown).String(),
			}
		}
		allPeers = append(allPeers, p)
	}
	response := &PeersResponse{Peers: allPeers}
	httputil.WriteJson(w, response)
}

// AddTrustedPeer adds a new peer into node's trusted peer set by Multiaddr
func (s *Server) AddTrustedPeer(w http.ResponseWriter, r *http.Request) {
	_, span := trace.StartSpan(r.Context(), "node.AddTrustedPeer")
	defer span.End()

	body, err := io.ReadAll(r.Body)
	if err != nil {
		errJson := &httputil.DefaultJsonError{
			Message: errors.Wrapf(err, "Could not read request body").Error(),
			Code:    http.StatusInternalServerError,
		}
		httputil.WriteError(w, errJson)
		return
	}
	var addrRequest *AddrRequest
	err = json.Unmarshal(body, &addrRequest)
	if err != nil {
		errJson := &httputil.DefaultJsonError{
			Message: errors.Wrapf(err, "Could not decode request body into peer address").Error(),
			Code:    http.StatusBadRequest,
		}
		httputil.WriteError(w, errJson)
		return
	}
	info, err := peer.AddrInfoFromString(addrRequest.Addr)
	if err != nil {
		errJson := &httputil.DefaultJsonError{
			Message: errors.Wrapf(err, "Could not derive peer info from multiaddress").Error(),
			Code:    http.StatusBadRequest,
		}
		httputil.WriteError(w, errJson)
		return
	}

	// also add new peerdata to peers
	direction, err := s.PeersFetcher.Peers().Direction(info.ID)
	if err != nil {
		s.PeersFetcher.Peers().Add(nil, info.ID, info.Addrs[0], corenet.DirUnknown)
	} else {
		s.PeersFetcher.Peers().Add(nil, info.ID, info.Addrs[0], direction)
	}

	var ids []peer.ID
	ids = append(ids, info.ID)
	s.PeersFetcher.Peers().SetTrustedPeers(ids)
	w.WriteHeader(http.StatusOK)
}

// RemoveTrustedPeer removes peer from our trusted peer set but does not close connection.
func (s *Server) RemoveTrustedPeer(w http.ResponseWriter, r *http.Request) {
	_, span := trace.StartSpan(r.Context(), "node.RemoveTrustedPeer")
	defer span.End()

	segments := strings.Split(r.URL.Path, "/")
	id := segments[len(segments)-1]
	peerId, err := peer.Decode(id)
	if err != nil {
		errJson := &httputil.DefaultJsonError{
			Message: errors.Wrapf(err, "Could not decode peer id").Error(),
			Code:    http.StatusBadRequest,
		}
		httputil.WriteError(w, errJson)
		return
	}

	// if the peer is not a trusted peer, do nothing but return 200
	if !s.PeersFetcher.Peers().IsTrustedPeers(peerId) {
		w.WriteHeader(http.StatusOK)
		return
	}

	var ids []peer.ID
	ids = append(ids, peerId)
	s.PeersFetcher.Peers().DeleteTrustedPeers(ids)
	w.WriteHeader(http.StatusOK)
}

// httpPeerInfo does the same thing as peerInfo function in node.go but returns the
// http peer response.
func httpPeerInfo(peerStatus *peers.Status, id peer.ID) (*Peer, error) {
	enr, err := peerStatus.ENR(id)
	if err != nil {
		if errors.Is(err, peerdata.ErrPeerUnknown) {
			return nil, nil
		}
		return nil, errors.Wrap(err, "could not obtain ENR")
	}
	var serializedEnr string
	if enr != nil {
		serializedEnr, err = p2p.SerializeENR(enr)
		if err != nil {
			return nil, errors.Wrap(err, "could not serialize ENR")
		}
	}
	address, err := peerStatus.Address(id)
	if err != nil {
		if errors.Is(err, peerdata.ErrPeerUnknown) {
			return nil, nil
		}
		return nil, errors.Wrap(err, "could not obtain address")
	}
	connectionState, err := peerStatus.ConnectionState(id)
	if err != nil {
		if errors.Is(err, peerdata.ErrPeerUnknown) {
			return nil, nil
		}
		return nil, errors.Wrap(err, "could not obtain connection state")
	}
	direction, err := peerStatus.Direction(id)
	if err != nil {
		if errors.Is(err, peerdata.ErrPeerUnknown) {
			return nil, nil
		}
		return nil, errors.Wrap(err, "could not obtain direction")
	}
	if eth.PeerDirection(direction) == eth.PeerDirection_UNKNOWN {
		return nil, nil
	}
	v1ConnState := eth.ConnectionState(connectionState).String()
	v1PeerDirection := eth.PeerDirection(direction).String()
	p := Peer{
		PeerID:    id.String(),
		State:     v1ConnState,
		Direction: v1PeerDirection,
	}
	if address != nil {
		p.LastSeenP2PAddress = address.String()
	}
	if serializedEnr != "" {
		p.Enr = "enr:" + serializedEnr
	}

	return &p, nil
}

package debug

import (
	"context"

	"github.com/golang/protobuf/ptypes/empty"
	"github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/libp2p/go-libp2p/core/protocol"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// GetPeer returns the data known about the peer defined by the provided peer id.
func (ds *Server) GetPeer(_ context.Context, peerReq *ethpb.PeerRequest) (*ethpb.DebugPeerResponse, error) {
	pid, err := peer.Decode(peerReq.PeerId)
	if err != nil {
		return nil, status.Errorf(codes.InvalidArgument, "Unable to parse provided peer id: %v", err)
	}
	return ds.getPeer(pid)
}

// ListPeers returns all peers known to the host node, regardless of if they are connected/
// disconnected.
func (ds *Server) ListPeers(_ context.Context, _ *empty.Empty) (*ethpb.DebugPeerResponses, error) {
	var responses []*ethpb.DebugPeerResponse
	for _, pid := range ds.PeersFetcher.Peers().All() {
		resp, err := ds.getPeer(pid)
		if err != nil {
			return nil, err
		}
		responses = append(responses, resp)
	}
	return &ethpb.DebugPeerResponses{Responses: responses}, nil
}

func (ds *Server) getPeer(pid peer.ID) (*ethpb.DebugPeerResponse, error) {
	peers := ds.PeersFetcher.Peers()
	peerStore := ds.PeerManager.Host().Peerstore()
	addr, err := peers.Address(pid)
	if err != nil {
		return nil, status.Errorf(codes.NotFound, "Requested peer does not exist: %v", err)
	}
	dir, err := peers.Direction(pid)
	if err != nil {
		return nil, status.Errorf(codes.NotFound, "Requested peer does not exist: %v", err)
	}
	pbDirection := ethpb.PeerDirection_UNKNOWN
	switch dir {
	case network.DirInbound:
		pbDirection = ethpb.PeerDirection_INBOUND
	case network.DirOutbound:
		pbDirection = ethpb.PeerDirection_OUTBOUND
	}
	connState, err := peers.ConnectionState(pid)
	if err != nil {
		return nil, status.Errorf(codes.NotFound, "Requested peer does not exist: %v", err)
	}
	record, err := peers.ENR(pid)
	if err != nil {
		return nil, status.Errorf(codes.NotFound, "Requested peer does not exist: %v", err)
	}
	enr := ""
	if record != nil {
		enr, err = p2p.SerializeENR(record)
		if err != nil {
			return nil, status.Errorf(codes.Internal, "Unable to serialize enr: %v", err)
		}
	}
	metadata, err := peers.Metadata(pid)
	if err != nil {
		return nil, status.Errorf(codes.NotFound, "Requested peer does not exist: %v", err)
	}
	protocols, err := peerStore.GetProtocols(pid)
	if err != nil {
		return nil, status.Errorf(codes.NotFound, "Requested peer does not exist: %v", err)
	}
	resp, err := peers.Scorers().BadResponsesScorer().Count(pid)
	if err != nil {
		return nil, status.Errorf(codes.NotFound, "Requested peer does not exist: %v", err)
	}

	rawPversion, err := peerStore.Get(pid, "ProtocolVersion")
	pVersion, ok := rawPversion.(string)
	if err != nil || !ok {
		pVersion = ""
	}
	rawAversion, err := peerStore.Get(pid, "AgentVersion")
	aVersion, ok := rawAversion.(string)
	if err != nil || !ok {
		aVersion = ""
	}
	peerInfo := &ethpb.DebugPeerResponse_PeerInfo{
		Protocols:       protocol.ConvertToStrings(protocols),
		FaultCount:      uint64(resp),
		ProtocolVersion: pVersion,
		AgentVersion:    aVersion,
		PeerLatency:     uint64(peerStore.LatencyEWMA(pid).Milliseconds()),
	}
	if metadata != nil && !metadata.IsNil() {
		switch {
		case metadata.MetadataObjV0() != nil:
			peerInfo.MetadataV0 = metadata.MetadataObjV0()
		case metadata.MetadataObjV1() != nil:
			peerInfo.MetadataV1 = metadata.MetadataObjV1()
		}
	}
	addresses := peerStore.Addrs(pid)
	var stringAddrs []string
	if addr != nil {
		stringAddrs = append(stringAddrs, addr.String())
	}
	for _, a := range addresses {
		// Do not double count address
		if addr != nil && addr.String() == a.String() {
			continue
		}
		stringAddrs = append(stringAddrs, a.String())
	}
	pStatus, err := peers.ChainState(pid)
	if err != nil {
		// In the event chain state is non existent, we
		// initialize with the zero value.
		pStatus = new(ethpb.Status)
	}
	lastUpdated, err := peers.ChainStateLastUpdated(pid)
	if err != nil {
		return nil, status.Errorf(codes.NotFound, "Requested peer does not exist: %v", err)
	}
	unixTime := uint64(0)
	if !lastUpdated.IsZero() {
		unixTime = uint64(lastUpdated.Unix())
	}
	gScore, bPenalty, topicMaps, err := peers.Scorers().GossipScorer().GossipData(pid)
	if err != nil {
		return nil, status.Errorf(codes.NotFound, "Requested peer does not exist: %v", err)
	}
	scoreInfo := &ethpb.ScoreInfo{
		OverallScore:       float32(peers.Scorers().Score(pid)),
		ProcessedBlocks:    peers.Scorers().BlockProviderScorer().ProcessedBlocks(pid),
		BlockProviderScore: float32(peers.Scorers().BlockProviderScorer().Score(pid)),
		TopicScores:        topicMaps,
		GossipScore:        float32(gScore),
		BehaviourPenalty:   float32(bPenalty),
		ValidationError:    errorToString(peers.Scorers().ValidationError(pid)),
	}
	return &ethpb.DebugPeerResponse{
		ListeningAddresses: stringAddrs,
		Direction:          pbDirection,
		ConnectionState:    ethpb.ConnectionState(connState),
		PeerId:             pid.String(),
		Enr:                enr,
		PeerInfo:           peerInfo,
		PeerStatus:         pStatus,
		LastUpdated:        unixTime,
		ScoreInfo:          scoreInfo,
	}, nil
}

func errorToString(err error) string {
	if err == nil {
		return ""
	}
	return err.Error()
}

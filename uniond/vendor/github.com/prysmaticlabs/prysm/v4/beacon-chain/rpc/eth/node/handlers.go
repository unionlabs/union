package node

import (
	"fmt"
	"net/http"
	"runtime"
	"strconv"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/eth/v1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"go.opencensus.io/trace"
)

var (
	stateConnecting    = ethpb.ConnectionState_CONNECTING.String()
	stateConnected     = ethpb.ConnectionState_CONNECTED.String()
	stateDisconnecting = ethpb.ConnectionState_DISCONNECTING.String()
	stateDisconnected  = ethpb.ConnectionState_DISCONNECTED.String()
	directionInbound   = ethpb.PeerDirection_INBOUND.String()
	directionOutbound  = ethpb.PeerDirection_OUTBOUND.String()
)

// GetSyncStatus requests the beacon node to describe if it's currently syncing or not, and
// if it is, what block it is up to.
func (s *Server) GetSyncStatus(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "node.GetSyncStatus")
	defer span.End()

	isOptimistic, err := s.OptimisticModeFetcher.IsOptimistic(ctx)
	if err != nil {
		httputil.HandleError(w, "Could not check optimistic status: "+err.Error(), http.StatusInternalServerError)
		return
	}

	headSlot := s.HeadFetcher.HeadSlot()
	response := &SyncStatusResponse{
		Data: &SyncStatusResponseData{
			HeadSlot:     strconv.FormatUint(uint64(headSlot), 10),
			SyncDistance: strconv.FormatUint(uint64(s.GenesisTimeFetcher.CurrentSlot()-headSlot), 10),
			IsSyncing:    s.SyncChecker.Syncing(),
			IsOptimistic: isOptimistic,
			ElOffline:    !s.ExecutionChainInfoFetcher.ExecutionClientConnected(),
		},
	}
	httputil.WriteJson(w, response)
}

// GetIdentity retrieves data about the node's network presence.
func (s *Server) GetIdentity(w http.ResponseWriter, r *http.Request) {
	_, span := trace.StartSpan(r.Context(), "node.GetIdentity")
	defer span.End()

	peerId := s.PeerManager.PeerID().String()
	sourcep2p := s.PeerManager.Host().Addrs()
	p2pAddresses := make([]string, len(sourcep2p))
	for i := range sourcep2p {
		p2pAddresses[i] = sourcep2p[i].String() + "/p2p/" + peerId
	}
	sourceDisc, err := s.PeerManager.DiscoveryAddresses()
	if err != nil {
		httputil.HandleError(w, "Could not obtain discovery address: "+err.Error(), http.StatusInternalServerError)
		return
	}
	discoveryAddresses := make([]string, len(sourceDisc))
	for i := range sourceDisc {
		discoveryAddresses[i] = sourceDisc[i].String()
	}
	serializedEnr, err := p2p.SerializeENR(s.PeerManager.ENR())
	if err != nil {
		httputil.HandleError(w, "Could not obtain enr: "+err.Error(), http.StatusInternalServerError)
		return
	}

	resp := &GetIdentityResponse{
		Data: &Identity{
			PeerId:             peerId,
			Enr:                "enr:" + serializedEnr,
			P2PAddresses:       p2pAddresses,
			DiscoveryAddresses: discoveryAddresses,
			Metadata: &Metadata{
				SeqNumber: strconv.FormatUint(s.MetadataProvider.MetadataSeq(), 10),
				Attnets:   hexutil.Encode(s.MetadataProvider.Metadata().AttnetsBitfield()),
			},
		},
	}
	httputil.WriteJson(w, resp)
}

// GetVersion requests that the beacon node identify information about its implementation in a
// format similar to a HTTP User-Agent field.
func (*Server) GetVersion(w http.ResponseWriter, r *http.Request) {
	_, span := trace.StartSpan(r.Context(), "node.GetVersion")
	defer span.End()

	v := fmt.Sprintf("Prysm/%s (%s %s)", version.SemanticVersion(), runtime.GOOS, runtime.GOARCH)
	resp := &GetVersionResponse{
		Data: &Version{
			Version: v,
		},
	}
	httputil.WriteJson(w, resp)
}

// GetHealth returns node health status in http status codes. Useful for load balancers.
func (s *Server) GetHealth(w http.ResponseWriter, r *http.Request) {
	_, span := trace.StartSpan(r.Context(), "node.GetHealth")
	defer span.End()

	rawSyncingStatus, syncingStatus, ok := shared.UintFromQuery(w, r, "syncing_status", false)
	// lint:ignore uintcast -- custom syncing status being outside of range is harmless
	intSyncingStatus := int(syncingStatus)
	if !ok || (rawSyncingStatus != "" && http.StatusText(intSyncingStatus) == "") {
		httputil.HandleError(w, "syncing_status is not a valid HTTP status code", http.StatusBadRequest)
		return
	}

	if s.SyncChecker.Synced() {
		return
	}
	if s.SyncChecker.Syncing() || s.SyncChecker.Initialized() {
		if rawSyncingStatus != "" {
			w.WriteHeader(intSyncingStatus)
		} else {
			w.WriteHeader(http.StatusPartialContent)
		}
		return
	}
	w.WriteHeader(http.StatusServiceUnavailable)
}

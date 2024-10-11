package beacon

import (
	"fmt"
	"log"
	"net/http"
	"strconv"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"go.opencensus.io/trace"
)

// GetWeakSubjectivity computes the starting epoch of the current weak subjectivity period, and then also
// determines the best block root and state root to use for a Checkpoint Sync starting from that point.
func (s *Server) GetWeakSubjectivity(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "beacon.GetWeakSubjectivity")
	defer span.End()

	if shared.IsSyncing(ctx, w, s.SyncChecker, s.HeadFetcher, s.TimeFetcher, s.OptimisticModeFetcher) {
		return
	}

	hs, err := s.HeadFetcher.HeadStateReadOnly(ctx)
	if err != nil {
		httputil.HandleError(w, "Could not get head state: "+err.Error(), http.StatusInternalServerError)
		return
	}
	wsEpoch, err := helpers.LatestWeakSubjectivityEpoch(ctx, hs, params.BeaconConfig())
	if err != nil {
		httputil.HandleError(w, "Could not get weak subjectivity epoch: "+err.Error(), http.StatusInternalServerError)
		return
	}
	wsSlot, err := slots.EpochStart(wsEpoch)
	if err != nil {
		httputil.HandleError(w, "Could not get weak subjectivity slot: "+err.Error(), http.StatusInternalServerError)
		return
	}
	cbr, err := s.CanonicalHistory.BlockRootForSlot(ctx, wsSlot)
	if err != nil {
		httputil.HandleError(w, fmt.Sprintf("Could not find highest block below slot %d: %s", wsSlot, err.Error()), http.StatusInternalServerError)
		return
	}
	cb, err := s.BeaconDB.Block(ctx, cbr)
	if err != nil {
		httputil.HandleError(
			w,
			fmt.Sprintf("Block with root %#x from slot index %d not found in db: %s", cbr, wsSlot, err.Error()),
			http.StatusInternalServerError,
		)
		return
	}
	stateRoot := cb.Block().StateRoot()
	log.Printf("Weak subjectivity checkpoint reported as epoch=%d, block root=%#x, state root=%#x", wsEpoch, cbr, stateRoot)

	resp := &GetWeakSubjectivityResponse{
		Data: &WeakSubjectivityData{
			WsCheckpoint: &shared.Checkpoint{
				Epoch: strconv.FormatUint(uint64(wsEpoch), 10),
				Root:  hexutil.Encode(cbr[:]),
			},
			StateRoot: hexutil.Encode(stateRoot[:]),
		},
	}
	httputil.WriteJson(w, resp)
}

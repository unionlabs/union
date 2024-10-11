package blob

import (
	"net/http"
	"net/url"
	"strconv"
	"strings"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/core"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	field_params "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"go.opencensus.io/trace"
)

// Blobs is an HTTP handler for Beacon API getBlobs.
func (s *Server) Blobs(w http.ResponseWriter, r *http.Request) {
	ctx, span := trace.StartSpan(r.Context(), "beacon.Blobs")
	defer span.End()
	var sidecars []*eth.BlobSidecar

	indices := parseIndices(r.URL)
	segments := strings.Split(r.URL.Path, "/")
	blockId := segments[len(segments)-1]

	verifiedBlobs, rpcErr := s.Blocker.Blobs(ctx, blockId, indices)
	if rpcErr != nil {
		code := core.ErrorReasonToHTTP(rpcErr.Reason)
		switch code {
		case http.StatusBadRequest:
			httputil.HandleError(w, "Invalid block ID: "+rpcErr.Err.Error(), code)
			return
		case http.StatusNotFound:
			httputil.HandleError(w, "Block not found: "+rpcErr.Err.Error(), code)
			return
		case http.StatusInternalServerError:
			httputil.HandleError(w, "Internal server error: "+rpcErr.Err.Error(), code)
			return
		default:
			httputil.HandleError(w, rpcErr.Err.Error(), code)
			return
		}
	}
	for i := range verifiedBlobs {
		sidecars = append(sidecars, verifiedBlobs[i].BlobSidecar)
	}
	if httputil.RespondWithSsz(r) {
		sidecarResp := &eth.BlobSidecars{
			Sidecars: sidecars,
		}
		sszResp, err := sidecarResp.MarshalSSZ()
		if err != nil {
			httputil.HandleError(w, err.Error(), http.StatusInternalServerError)
			return
		}
		httputil.WriteSsz(w, sszResp, "blob_sidecars.ssz")
		return
	}

	httputil.WriteJson(w, buildSidecarsResponse(sidecars))
}

// parseIndices filters out invalid and duplicate blob indices
func parseIndices(url *url.URL) []uint64 {
	rawIndices := url.Query()["indices"]
	indices := make([]uint64, 0, field_params.MaxBlobsPerBlock)
loop:
	for _, raw := range rawIndices {
		ix, err := strconv.ParseUint(raw, 10, 64)
		if err != nil {
			continue
		}
		if ix >= field_params.MaxBlobsPerBlock {
			continue
		}
		for i := range indices {
			if ix == indices[i] {
				continue loop
			}
		}
		indices = append(indices, ix)
	}
	return indices
}

func buildSidecarsResponse(sidecars []*eth.BlobSidecar) *SidecarsResponse {
	resp := &SidecarsResponse{Data: make([]*Sidecar, len(sidecars))}
	for i, sc := range sidecars {
		proofs := make([]string, len(sc.CommitmentInclusionProof))
		for j := range sc.CommitmentInclusionProof {
			proofs[j] = hexutil.Encode(sc.CommitmentInclusionProof[j])
		}
		resp.Data[i] = &Sidecar{
			Index:                    strconv.FormatUint(sc.Index, 10),
			Blob:                     hexutil.Encode(sc.Blob),
			KzgCommitment:            hexutil.Encode(sc.KzgCommitment),
			SignedBeaconBlockHeader:  shared.SignedBeaconBlockHeaderFromConsensus(sc.SignedBlockHeader),
			KzgProof:                 hexutil.Encode(sc.KzgProof),
			CommitmentInclusionProof: proofs,
		}
	}
	return resp
}

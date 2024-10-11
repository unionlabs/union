package sync

import (
	"context"
	"sort"
	"time"

	libp2pcore "github.com/libp2p/go-libp2p/core"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/types"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/flags"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"go.opencensus.io/trace"
)

func blobMinReqEpoch(finalized, current primitives.Epoch) primitives.Epoch {
	// max(finalized_epoch, current_epoch - MIN_EPOCHS_FOR_BLOB_SIDECARS_REQUESTS, DENEB_FORK_EPOCH)
	denebFork := params.BeaconConfig().DenebForkEpoch
	var reqWindow primitives.Epoch
	if current > params.BeaconConfig().MinEpochsForBlobsSidecarsRequest {
		reqWindow = current - params.BeaconConfig().MinEpochsForBlobsSidecarsRequest
	}
	if finalized >= reqWindow && finalized > denebFork {
		return finalized
	}
	if reqWindow >= finalized && reqWindow > denebFork {
		return reqWindow
	}
	return denebFork
}

// blobSidecarByRootRPCHandler handles the /eth2/beacon_chain/req/blob_sidecars_by_root/1/ RPC request.
// spec: https://github.com/ethereum/consensus-specs/blob/a7e45db9ac2b60a33e144444969ad3ac0aae3d4c/specs/deneb/p2p-interface.md#blobsidecarsbyroot-v1
func (s *Service) blobSidecarByRootRPCHandler(ctx context.Context, msg interface{}, stream libp2pcore.Stream) error {
	ctx, span := trace.StartSpan(ctx, "sync.blobSidecarByRootRPCHandler")
	defer span.End()
	ctx, cancel := context.WithTimeout(ctx, ttfbTimeout)
	defer cancel()
	SetRPCStreamDeadlines(stream)
	log := log.WithField("handler", p2p.BlobSidecarsByRootName[1:]) // slice the leading slash off the name var
	ref, ok := msg.(*types.BlobSidecarsByRootReq)
	if !ok {
		return errors.New("message is not type BlobSidecarsByRootReq")
	}

	blobIdents := *ref
	if err := validateBlobByRootRequest(blobIdents); err != nil {
		s.cfg.p2p.Peers().Scorers().BadResponsesScorer().Increment(stream.Conn().RemotePeer())
		s.writeErrorResponseToStream(responseCodeInvalidRequest, err.Error(), stream)
		return err
	}
	// Sort the identifiers so that requests for the same blob root will be adjacent, minimizing db lookups.
	sort.Sort(blobIdents)

	batchSize := flags.Get().BlobBatchLimit
	var ticker *time.Ticker
	if len(blobIdents) > batchSize {
		ticker = time.NewTicker(time.Second)
	}
	minReqEpoch := blobMinReqEpoch(s.cfg.chain.FinalizedCheckpt().Epoch, slots.ToEpoch(s.cfg.clock.CurrentSlot()))

	for i := range blobIdents {
		if err := ctx.Err(); err != nil {
			closeStream(stream, log)
			return err
		}

		// Throttle request processing to no more than batchSize/sec.
		if i != 0 && i%batchSize == 0 && ticker != nil {
			<-ticker.C
		}
		s.rateLimiter.add(stream, 1)
		root, idx := bytesutil.ToBytes32(blobIdents[i].BlockRoot), blobIdents[i].Index
		sc, err := s.cfg.blobStorage.Get(root, idx)
		if err != nil {
			if db.IsNotFound(err) {
				log.WithError(err).Debugf("BlobSidecar not found in db, root=%x, index=%d", root, idx)
				continue
			}
			log.WithError(err).Errorf("unexpected db error retrieving BlobSidecar, root=%x, index=%d", root, idx)
			s.writeErrorResponseToStream(responseCodeServerError, types.ErrGeneric.Error(), stream)
			return err
		}

		// If any root in the request content references a block earlier than minimum_request_epoch,
		// peers MAY respond with error code 3: ResourceUnavailable or not include the blob in the response.
		if slots.ToEpoch(sc.Slot()) < minReqEpoch {
			s.writeErrorResponseToStream(responseCodeResourceUnavailable, types.ErrBlobLTMinRequest.Error(), stream)
			log.WithError(types.ErrBlobLTMinRequest).
				Debugf("requested blob for block %#x before minimum_request_epoch", blobIdents[i].BlockRoot)
			return types.ErrBlobLTMinRequest
		}
		SetStreamWriteDeadline(stream, defaultWriteDuration)
		if chunkErr := WriteBlobSidecarChunk(stream, s.cfg.chain, s.cfg.p2p.Encoding(), sc); chunkErr != nil {
			log.WithError(chunkErr).Debug("Could not send a chunked response")
			s.writeErrorResponseToStream(responseCodeServerError, types.ErrGeneric.Error(), stream)
			tracing.AnnotateError(span, chunkErr)
			return chunkErr
		}
	}
	closeStream(stream, log)
	return nil
}

func validateBlobByRootRequest(blobIdents types.BlobSidecarsByRootReq) error {
	if uint64(len(blobIdents)) > params.BeaconConfig().MaxRequestBlobSidecars {
		return types.ErrMaxBlobReqExceeded
	}
	return nil
}

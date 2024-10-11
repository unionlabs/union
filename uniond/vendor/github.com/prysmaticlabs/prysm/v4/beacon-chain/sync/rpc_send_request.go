package sync

import (
	"context"
	"fmt"
	"io"

	"github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/encoder"
	p2ptypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/types"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	pb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/sirupsen/logrus"
)

var errBlobChunkedReadFailure = errors.New("failed to read stream of chunk-encoded blobs")
var errBlobUnmarshal = errors.New("Could not unmarshal chunk-encoded blob")

// Any error from the following declaration block should result in peer downscoring.
var (
	// ErrInvalidFetchedData is used to signal that an error occurred which should result in peer downscoring.
	ErrInvalidFetchedData             = errors.New("invalid data returned from peer")
	errBlobIndexOutOfBounds           = errors.Wrap(ErrInvalidFetchedData, "blob index out of range")
	errMaxRequestBlobSidecarsExceeded = errors.Wrap(ErrInvalidFetchedData, "peer exceeded req blob chunk tx limit")
	errChunkResponseSlotNotAsc        = errors.Wrap(ErrInvalidFetchedData, "blob slot not higher than previous block root")
	errChunkResponseIndexNotAsc       = errors.Wrap(ErrInvalidFetchedData, "blob indices for a block must start at 0 and increase by 1")
	errUnrequested                    = errors.Wrap(ErrInvalidFetchedData, "received BlobSidecar in response that was not requested")
	errBlobResponseOutOfBounds        = errors.Wrap(ErrInvalidFetchedData, "received BlobSidecar with slot outside BlobSidecarsByRangeRequest bounds")
	errChunkResponseBlockMismatch     = errors.Wrap(ErrInvalidFetchedData, "blob block details do not match")
	errChunkResponseParentMismatch    = errors.Wrap(ErrInvalidFetchedData, "parent root for response element doesn't match previous element root")
)

// BeaconBlockProcessor defines a block processing function, which allows to start utilizing
// blocks even before all blocks are ready.
type BeaconBlockProcessor func(block interfaces.ReadOnlySignedBeaconBlock) error

// SendBeaconBlocksByRangeRequest sends BeaconBlocksByRange and returns fetched blocks, if any.
func SendBeaconBlocksByRangeRequest(
	ctx context.Context, tor blockchain.TemporalOracle, p2pProvider p2p.SenderEncoder, pid peer.ID,
	req *pb.BeaconBlocksByRangeRequest, blockProcessor BeaconBlockProcessor,
) ([]interfaces.ReadOnlySignedBeaconBlock, error) {
	topic, err := p2p.TopicFromMessage(p2p.BeaconBlocksByRangeMessageName, slots.ToEpoch(tor.CurrentSlot()))
	if err != nil {
		return nil, err
	}
	stream, err := p2pProvider.Send(ctx, req, topic, pid)
	if err != nil {
		return nil, err
	}
	defer closeStream(stream, log)

	// Augment block processing function, if non-nil block processor is provided.
	blocks := make([]interfaces.ReadOnlySignedBeaconBlock, 0, req.Count)
	process := func(blk interfaces.ReadOnlySignedBeaconBlock) error {
		blocks = append(blocks, blk)
		if blockProcessor != nil {
			return blockProcessor(blk)
		}
		return nil
	}
	var prevSlot primitives.Slot
	for i := uint64(0); ; i++ {
		isFirstChunk := i == 0
		blk, err := ReadChunkedBlock(stream, tor, p2pProvider, isFirstChunk)
		if errors.Is(err, io.EOF) {
			break
		}
		if err != nil {
			return nil, err
		}
		// The response MUST contain no more than `count` blocks, and no more than
		// MAX_REQUEST_BLOCKS blocks.
		currentEpoch := slots.ToEpoch(tor.CurrentSlot())
		if i >= req.Count || i >= params.MaxRequestBlock(currentEpoch) {
			return nil, ErrInvalidFetchedData
		}
		// Returned blocks MUST be in the slot range [start_slot, start_slot + count * step).
		if blk.Block().Slot() < req.StartSlot || blk.Block().Slot() >= req.StartSlot.Add(req.Count*req.Step) {
			return nil, ErrInvalidFetchedData
		}
		// Returned blocks, where they exist, MUST be sent in a consecutive order.
		// Consecutive blocks MUST have values in `step` increments (slots may be skipped in between).
		isSlotOutOfOrder := false
		if prevSlot >= blk.Block().Slot() {
			isSlotOutOfOrder = true
		} else if req.Step != 0 && blk.Block().Slot().SubSlot(prevSlot).Mod(req.Step) != 0 {
			isSlotOutOfOrder = true
		}
		if !isFirstChunk && isSlotOutOfOrder {
			return nil, ErrInvalidFetchedData
		}
		prevSlot = blk.Block().Slot()
		if err := process(blk); err != nil {
			return nil, err
		}
	}
	return blocks, nil
}

// SendBeaconBlocksByRootRequest sends BeaconBlocksByRoot and returns fetched blocks, if any.
func SendBeaconBlocksByRootRequest(
	ctx context.Context, clock blockchain.TemporalOracle, p2pProvider p2p.P2P, pid peer.ID,
	req *p2ptypes.BeaconBlockByRootsReq, blockProcessor BeaconBlockProcessor,
) ([]interfaces.ReadOnlySignedBeaconBlock, error) {
	topic, err := p2p.TopicFromMessage(p2p.BeaconBlocksByRootsMessageName, slots.ToEpoch(clock.CurrentSlot()))
	if err != nil {
		return nil, err
	}
	stream, err := p2pProvider.Send(ctx, req, topic, pid)
	if err != nil {
		return nil, err
	}
	defer closeStream(stream, log)

	// Augment block processing function, if non-nil block processor is provided.
	blocks := make([]interfaces.ReadOnlySignedBeaconBlock, 0, len(*req))
	process := func(block interfaces.ReadOnlySignedBeaconBlock) error {
		blocks = append(blocks, block)
		if blockProcessor != nil {
			return blockProcessor(block)
		}
		return nil
	}
	currentEpoch := slots.ToEpoch(clock.CurrentSlot())
	for i := 0; i < len(*req); i++ {
		// Exit if peer sends more than max request blocks.
		if uint64(i) >= params.MaxRequestBlock(currentEpoch) {
			break
		}
		isFirstChunk := i == 0
		blk, err := ReadChunkedBlock(stream, clock, p2pProvider, isFirstChunk)
		if errors.Is(err, io.EOF) {
			break
		}
		if err != nil {
			return nil, err
		}

		if err := process(blk); err != nil {
			return nil, err
		}
	}
	return blocks, nil
}

func SendBlobsByRangeRequest(ctx context.Context, tor blockchain.TemporalOracle, p2pApi p2p.SenderEncoder, pid peer.ID, ctxMap ContextByteVersions, req *pb.BlobSidecarsByRangeRequest) ([]blocks.ROBlob, error) {
	topic, err := p2p.TopicFromMessage(p2p.BlobSidecarsByRangeName, slots.ToEpoch(tor.CurrentSlot()))
	if err != nil {
		return nil, err
	}
	log.WithFields(logrus.Fields{
		"topic":     topic,
		"startSlot": req.StartSlot,
		"count":     req.Count,
	}).Debug("Sending blob by range request")
	stream, err := p2pApi.Send(ctx, req, topic, pid)
	if err != nil {
		return nil, err
	}
	defer closeStream(stream, log)

	max := params.BeaconConfig().MaxRequestBlobSidecars
	if max > req.Count*fieldparams.MaxBlobsPerBlock {
		max = req.Count * fieldparams.MaxBlobsPerBlock
	}
	blobVal := composeBlobValidations(blobValidatorFromRangeReq(req), newSequentialBlobValidator())
	return readChunkEncodedBlobs(stream, p2pApi.Encoding(), ctxMap, blobVal, max)
}

func SendBlobSidecarByRoot(
	ctx context.Context, tor blockchain.TemporalOracle, p2pApi p2p.P2P, pid peer.ID,
	ctxMap ContextByteVersions, req *p2ptypes.BlobSidecarsByRootReq,
) ([]blocks.ROBlob, error) {
	if uint64(len(*req)) > params.BeaconConfig().MaxRequestBlobSidecars {
		return nil, errors.Wrapf(p2ptypes.ErrMaxBlobReqExceeded, "length=%d", len(*req))
	}

	topic, err := p2p.TopicFromMessage(p2p.BlobSidecarsByRootName, slots.ToEpoch(tor.CurrentSlot()))
	if err != nil {
		return nil, err
	}
	log.WithField("topic", topic).Debug("Sending blob sidecar request")
	stream, err := p2pApi.Send(ctx, req, topic, pid)
	if err != nil {
		return nil, err
	}
	defer closeStream(stream, log)

	max := params.BeaconConfig().MaxRequestBlobSidecars
	if max > uint64(len(*req))*fieldparams.MaxBlobsPerBlock {
		max = uint64(len(*req)) * fieldparams.MaxBlobsPerBlock
	}
	return readChunkEncodedBlobs(stream, p2pApi.Encoding(), ctxMap, blobValidatorFromRootReq(req), max)
}

type blobResponseValidation func(blocks.ROBlob) error

func composeBlobValidations(vf ...blobResponseValidation) blobResponseValidation {
	return func(blob blocks.ROBlob) error {
		for i := range vf {
			if err := vf[i](blob); err != nil {
				return err
			}
		}
		return nil
	}
}

type seqBlobValid struct {
	prev *blocks.ROBlob
}

func (sbv *seqBlobValid) nextValid(blob blocks.ROBlob) error {
	if blob.Index >= fieldparams.MaxBlobsPerBlock {
		return errBlobIndexOutOfBounds
	}
	if sbv.prev == nil {
		// The first blob we see for a block must have index 0.
		if blob.Index != 0 {
			return errChunkResponseIndexNotAsc
		}
		sbv.prev = &blob
		return nil
	}
	if sbv.prev.Slot() == blob.Slot() {
		if sbv.prev.BlockRoot() != blob.BlockRoot() {
			return errors.Wrap(errChunkResponseBlockMismatch, "block roots do not match")
		}
		if sbv.prev.ParentRoot() != blob.ParentRoot() {
			return errors.Wrap(errChunkResponseBlockMismatch, "block parent roots do not match")
		}
		// Blob indices in responses should be strictly monotonically incrementing.
		if blob.Index != sbv.prev.Index+1 {
			return errChunkResponseIndexNotAsc
		}
	} else {
		// If the slot is adjacent we know there are no intervening blocks with missing blobs, so we can
		// check that the new blob descends from the last seen.
		if blob.Slot() == sbv.prev.Slot()+1 && blob.ParentRoot() != sbv.prev.BlockRoot() {
			return errChunkResponseParentMismatch
		}
		// The first blob we see for a block must have index 0.
		if blob.Index != 0 {
			return errChunkResponseIndexNotAsc
		}
		// Blocks must be in ascending slot order.
		if sbv.prev.Slot() >= blob.Slot() {
			return errChunkResponseSlotNotAsc
		}
	}
	sbv.prev = &blob
	return nil
}

func newSequentialBlobValidator() blobResponseValidation {
	sbv := &seqBlobValid{}
	return func(blob blocks.ROBlob) error {
		return sbv.nextValid(blob)
	}
}

func blobValidatorFromRootReq(req *p2ptypes.BlobSidecarsByRootReq) blobResponseValidation {
	blobIds := make(map[[32]byte]map[uint64]bool)
	for _, sc := range *req {
		blockRoot := bytesutil.ToBytes32(sc.BlockRoot)
		if blobIds[blockRoot] == nil {
			blobIds[blockRoot] = make(map[uint64]bool)
		}
		blobIds[blockRoot][sc.Index] = true
	}
	return func(sc blocks.ROBlob) error {
		blobIndices := blobIds[sc.BlockRoot()]
		if blobIndices == nil {
			return errors.Wrapf(errUnrequested, "root=%#x", sc.BlockRoot())
		}
		requested := blobIndices[sc.Index]
		if !requested {
			return errors.Wrapf(errUnrequested, "root=%#x index=%d", sc.BlockRoot(), sc.Index)
		}
		return nil
	}
}

func blobValidatorFromRangeReq(req *pb.BlobSidecarsByRangeRequest) blobResponseValidation {
	end := req.StartSlot + primitives.Slot(req.Count)
	return func(sc blocks.ROBlob) error {
		if sc.Slot() < req.StartSlot || sc.Slot() >= end {
			return errors.Wrapf(errBlobResponseOutOfBounds, "req start,end:%d,%d, resp:%d", req.StartSlot, end, sc.Slot())
		}
		return nil
	}
}

func readChunkEncodedBlobs(stream network.Stream, encoding encoder.NetworkEncoding, ctxMap ContextByteVersions, vf blobResponseValidation, max uint64) ([]blocks.ROBlob, error) {
	sidecars := make([]blocks.ROBlob, 0)
	// Attempt an extra read beyond max to check if the peer is violating the spec by
	// sending more than MAX_REQUEST_BLOB_SIDECARS, or more blobs than requested.
	for i := uint64(0); i < max+1; i++ {
		sc, err := readChunkedBlobSidecar(stream, encoding, ctxMap, vf)
		if err != nil {
			if errors.Is(err, io.EOF) {
				break
			}
			return nil, err
		}
		if i == max {
			// We have read an extra sidecar beyond what the spec allows. Since this is a spec violation, we return
			// an error that wraps ErrInvalidFetchedData. The part of the state machine that handles rpc peer scoring
			// will downscore the peer if the request ends in an error that wraps that one.
			return nil, errMaxRequestBlobSidecarsExceeded
		}
		sidecars = append(sidecars, sc)
	}

	return sidecars, nil
}

func readChunkedBlobSidecar(stream network.Stream, encoding encoder.NetworkEncoding, ctxMap ContextByteVersions, vf blobResponseValidation) (blocks.ROBlob, error) {
	var b blocks.ROBlob
	pb := &ethpb.BlobSidecar{}
	decode := encoding.DecodeWithMaxLength
	var (
		code uint8
		msg  string
	)
	code, msg, err := ReadStatusCode(stream, encoding)
	if err != nil {
		return b, err
	}
	if code != 0 {
		return b, errors.Wrap(errBlobChunkedReadFailure, msg)
	}
	ctxb, err := readContextFromStream(stream)
	if err != nil {
		return b, errors.Wrap(err, "error reading chunk context bytes from stream")
	}

	v, found := ctxMap[bytesutil.ToBytes4(ctxb)]
	if !found {
		return b, errors.Wrapf(errBlobUnmarshal, fmt.Sprintf("unrecognized fork digest %#x", ctxb))
	}
	// Only deneb is supported at this time, because we lack a fork-spanning interface/union type for blobs.
	if v != version.Deneb {
		return b, fmt.Errorf("unexpected context bytes for deneb BlobSidecar, ctx=%#x, v=%s", ctxb, version.String(v))
	}
	if err := decode(stream, pb); err != nil {
		return b, errors.Wrap(err, "failed to decode the protobuf-encoded BlobSidecar message from RPC chunk stream")
	}

	rob, err := blocks.NewROBlob(pb)
	if err != nil {
		return b, errors.Wrap(err, "unexpected error initializing ROBlob")
	}
	if err := vf(rob); err != nil {
		return b, errors.Wrap(err, "validation failure decoding blob RPC response")
	}

	return rob, nil
}

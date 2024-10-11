package validator

import (
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/async/event"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/blocks"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed"
	blockfeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/block"
	statefeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/state"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// StreamBlocksAltair to clients every single time a block is received by the beacon node.
func (vs *Server) StreamBlocksAltair(req *ethpb.StreamBlocksRequest, stream ethpb.BeaconNodeValidator_StreamBlocksAltairServer) error {
	blocksChannel := make(chan *feed.Event, 1)
	var blockSub event.Subscription
	if req.VerifiedOnly {
		blockSub = vs.StateNotifier.StateFeed().Subscribe(blocksChannel)
	} else {
		blockSub = vs.BlockNotifier.BlockFeed().Subscribe(blocksChannel)
	}
	defer blockSub.Unsubscribe()

	for {
		select {
		case blockEvent := <-blocksChannel:
			if req.VerifiedOnly {
				if err := sendVerifiedBlocks(stream, blockEvent); err != nil {
					return err
				}
			} else {
				if err := vs.sendBlocks(stream, blockEvent); err != nil {
					return err
				}
			}
		case <-blockSub.Err():
			return status.Error(codes.Aborted, "Subscriber closed, exiting goroutine")
		case <-vs.Ctx.Done():
			return status.Error(codes.Canceled, "Context canceled")
		case <-stream.Context().Done():
			return status.Error(codes.Canceled, "Context canceled")
		}
	}
}

// StreamSlots sends a block's slot to clients every single time a block is received by the beacon node.
func (vs *Server) StreamSlots(req *ethpb.StreamSlotsRequest, stream ethpb.BeaconNodeValidator_StreamSlotsServer) error {
	ch := make(chan *feed.Event, 1)
	var sub event.Subscription
	if req.VerifiedOnly {
		sub = vs.StateNotifier.StateFeed().Subscribe(ch)
	} else {
		sub = vs.BlockNotifier.BlockFeed().Subscribe(ch)
	}
	defer sub.Unsubscribe()

	for {
		select {
		case ev := <-ch:
			var s primitives.Slot
			if req.VerifiedOnly {
				if ev.Type != statefeed.BlockProcessed {
					continue
				}
				data, ok := ev.Data.(*statefeed.BlockProcessedData)
				if !ok || data == nil {
					continue
				}
				s = data.Slot
			} else {
				if ev.Type != blockfeed.ReceivedBlock {
					continue
				}
				data, ok := ev.Data.(*blockfeed.ReceivedBlockData)
				if !ok || data == nil {
					continue
				}
				s = data.SignedBlock.Block().Slot()
			}
			if err := stream.Send(&ethpb.StreamSlotsResponse{Slot: s}); err != nil {
				return status.Errorf(codes.Unavailable, "Could not send over stream: %v", err)
			}
		case <-sub.Err():
			return status.Error(codes.Aborted, "Subscriber closed, exiting goroutine")
		case <-vs.Ctx.Done():
			return status.Error(codes.Canceled, "Context canceled")
		case <-stream.Context().Done():
			return status.Error(codes.Canceled, "Context canceled")
		}
	}
}

func sendVerifiedBlocks(stream ethpb.BeaconNodeValidator_StreamBlocksAltairServer, blockEvent *feed.Event) error {
	if blockEvent.Type != statefeed.BlockProcessed {
		return nil
	}
	data, ok := blockEvent.Data.(*statefeed.BlockProcessedData)
	if !ok || data == nil {
		return nil
	}
	b := &ethpb.StreamBlocksResponse{}
	switch data.SignedBlock.Version() {
	case version.Phase0:
		pb, err := data.SignedBlock.Proto()
		if err != nil {
			return errors.Wrap(err, "could not get protobuf block")
		}
		phBlk, ok := pb.(*ethpb.SignedBeaconBlock)
		if !ok {
			log.Warn("Mismatch between version and block type, was expecting SignedBeaconBlock")
			return nil
		}
		b.Block = &ethpb.StreamBlocksResponse_Phase0Block{Phase0Block: phBlk}
	case version.Altair:
		pb, err := data.SignedBlock.Proto()
		if err != nil {
			return errors.Wrap(err, "could not get protobuf block")
		}
		phBlk, ok := pb.(*ethpb.SignedBeaconBlockAltair)
		if !ok {
			log.Warn("Mismatch between version and block type, was expecting SignedBeaconBlockAltair")
			return nil
		}
		b.Block = &ethpb.StreamBlocksResponse_AltairBlock{AltairBlock: phBlk}
	case version.Bellatrix:
		pb, err := data.SignedBlock.Proto()
		if err != nil {
			return errors.Wrap(err, "could not get protobuf block")
		}
		phBlk, ok := pb.(*ethpb.SignedBeaconBlockBellatrix)
		if !ok {
			log.Warn("Mismatch between version and block type, was expecting SignedBeaconBlockBellatrix")
			return nil
		}
		b.Block = &ethpb.StreamBlocksResponse_BellatrixBlock{BellatrixBlock: phBlk}
	case version.Capella:
		pb, err := data.SignedBlock.Proto()
		if err != nil {
			return errors.Wrap(err, "could not get protobuf block")
		}
		phBlk, ok := pb.(*ethpb.SignedBeaconBlockCapella)
		if !ok {
			log.Warn("Mismatch between version and block type, was expecting SignedBeaconBlockCapella")
			return nil
		}
		b.Block = &ethpb.StreamBlocksResponse_CapellaBlock{CapellaBlock: phBlk}
	case version.Deneb:
		pb, err := data.SignedBlock.Proto()
		if err != nil {
			return errors.Wrap(err, "could not get protobuf block")
		}
		phBlk, ok := pb.(*ethpb.SignedBeaconBlockDeneb)
		if !ok {
			log.Warn("Mismatch between version and block type, was expecting SignedBeaconBlockDeneb")
			return nil
		}
		b.Block = &ethpb.StreamBlocksResponse_DenebBlock{DenebBlock: phBlk}
	}

	if err := stream.Send(b); err != nil {
		return status.Errorf(codes.Unavailable, "Could not send over stream: %v", err)
	}

	return nil
}

func (vs *Server) sendBlocks(stream ethpb.BeaconNodeValidator_StreamBlocksAltairServer, blockEvent *feed.Event) error {
	if blockEvent.Type != blockfeed.ReceivedBlock {
		return nil
	}

	data, ok := blockEvent.Data.(*blockfeed.ReceivedBlockData)
	if !ok || data == nil {
		// Got bad data over the stream.
		return nil
	}
	if data.SignedBlock == nil {
		// One nil block shouldn't stop the stream.
		return nil
	}
	log := log.WithField("blockSlot", data.SignedBlock.Block().Slot())
	headState, err := vs.HeadFetcher.HeadStateReadOnly(vs.Ctx)
	if err != nil {
		log.WithError(err).Error("Could not get head state")
		return nil
	}
	signed := data.SignedBlock
	sig := signed.Signature()
	if err := blocks.VerifyBlockSignature(headState, signed.Block().ProposerIndex(), sig[:], signed.Block().HashTreeRoot); err != nil {
		log.WithError(err).Error("Could not verify block signature")
		return nil
	}
	b := &ethpb.StreamBlocksResponse{}
	pb, err := data.SignedBlock.Proto()
	if err != nil {
		return errors.Wrap(err, "could not get protobuf block")
	}
	switch p := pb.(type) {
	case *ethpb.SignedBeaconBlock:
		b.Block = &ethpb.StreamBlocksResponse_Phase0Block{Phase0Block: p}
	case *ethpb.SignedBeaconBlockAltair:
		b.Block = &ethpb.StreamBlocksResponse_AltairBlock{AltairBlock: p}
	case *ethpb.SignedBeaconBlockBellatrix:
		b.Block = &ethpb.StreamBlocksResponse_BellatrixBlock{BellatrixBlock: p}
	case *ethpb.SignedBeaconBlockCapella:
		b.Block = &ethpb.StreamBlocksResponse_CapellaBlock{CapellaBlock: p}
	case *ethpb.SignedBeaconBlockDeneb:
		b.Block = &ethpb.StreamBlocksResponse_DenebBlock{DenebBlock: p}
	default:
		log.Errorf("Unknown block type %T", p)
	}
	if err := stream.Send(b); err != nil {
		return status.Errorf(codes.Unavailable, "Could not send over stream: %v", err)
	}

	return nil
}

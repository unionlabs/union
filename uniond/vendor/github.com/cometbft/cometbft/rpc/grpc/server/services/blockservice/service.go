package blockservice

import (
	"context"
	"fmt"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	blocksvc "github.com/cometbft/cometbft/api/cometbft/services/block/v1"
	ptypes "github.com/cometbft/cometbft/api/cometbft/types/v1"
	"github.com/cometbft/cometbft/internal/rpctrace"
	"github.com/cometbft/cometbft/libs/log"
	cmtpubsub "github.com/cometbft/cometbft/libs/pubsub"
	"github.com/cometbft/cometbft/store"
	"github.com/cometbft/cometbft/types"
)

type blockServiceServer struct {
	store    *store.BlockStore
	eventBus *types.EventBus
	logger   log.Logger
}

// New creates a new CometBFT version service server.
func New(store *store.BlockStore, eventBus *types.EventBus, logger log.Logger) blocksvc.BlockServiceServer {
	return &blockServiceServer{
		store:    store,
		eventBus: eventBus,
		logger:   logger.With("service", "BlockService"),
	}
}

// GetByHeight implements v1.BlockServiceServer GetByHeight method.
func (s *blockServiceServer) GetByHeight(_ context.Context, req *blocksvc.GetByHeightRequest) (*blocksvc.GetByHeightResponse, error) {
	logger := s.logger.With("endpoint", "GetByHeight")
	if err := validateBlockHeight(req.Height, s.store.Base(), s.store.Height()); err != nil {
		return nil, err
	}

	blockID, block, err := s.getBlock(req.Height, logger)
	if err != nil {
		return nil, err
	}

	return &blocksvc.GetByHeightResponse{
		BlockId: blockID,
		Block:   block,
	}, nil
}

func (s *blockServiceServer) getBlock(height int64, logger log.Logger) (*ptypes.BlockID, *ptypes.Block, error) {
	traceID, err := rpctrace.New()
	if err != nil {
		logger.Error("Error generating RPC trace ID", "err", err)
		return nil, nil, status.Error(codes.Internal, "Internal server error - see logs for details")
	}

	block, blockMeta := s.store.LoadBlock(height)
	if block == nil {
		return nil, nil, status.Errorf(codes.NotFound, "Block not found for height %d", height)
	}
	bp, err := block.ToProto()
	if err != nil {
		logger.Error("Error attempting to convert block to its Protobuf representation", "err", err, "traceID", traceID)
		return nil, nil, status.Errorf(codes.Internal, "Failed to load block from store (see logs for trace ID: %s)", traceID)
	}

	if blockMeta == nil {
		logger.Error("Failed to load block meta when block was successfully loaded", "height", height)
		return nil, nil, status.Error(codes.Internal, "Internal server error - see logs for details")
	}

	blockIDProto := blockMeta.BlockID.ToProto()
	return &blockIDProto, bp, nil
}

// GetLatestHeight implements v1.BlockServiceServer GetLatestHeight method.
func (s *blockServiceServer) GetLatestHeight(_ *blocksvc.GetLatestHeightRequest, stream blocksvc.BlockService_GetLatestHeightServer) error {
	logger := s.logger.With("endpoint", "GetLatestHeight")

	traceID, err := rpctrace.New()
	if err != nil {
		logger.Error("Error generating RPC trace ID", "err", err)
		return status.Error(codes.Internal, "Internal server error")
	}

	// The trace ID is reused as a unique subscriber ID
	sub, err := s.eventBus.Subscribe(context.Background(), traceID, types.QueryForEvent(types.EventNewBlock), 1)
	if err != nil {
		logger.Error("Cannot subscribe to new block events", "err", err, "traceID", traceID)
		return status.Errorf(codes.Internal, "Cannot subscribe to new block events (see logs for trace ID: %s)", traceID)
	}

	for {
		select {
		case msg := <-sub.Out():
			height, err := getHeightFromMsg(msg)
			if err != nil {
				logger.Error("Failed to extract height from subscription message", "err", err, "traceID", traceID)
				return status.Errorf(codes.Internal, "Internal server error (see logs for trace ID: %s)", traceID)
			}
			if err := stream.Send(&blocksvc.GetLatestHeightResponse{Height: height}); err != nil {
				logger.Error("Failed to stream new block", "err", err, "height", height, "traceID", traceID)
				return status.Errorf(codes.Unavailable, "Cannot send stream response (see logs for trace ID: %s)", traceID)
			}
		case <-sub.Canceled():
			switch sub.Err() {
			case cmtpubsub.ErrUnsubscribed:
				return status.Error(codes.Canceled, "Subscription terminated")
			case nil:
				return status.Error(codes.Canceled, "Subscription canceled without errors")
			default:
				logger.Info("Subscription canceled with errors", "err", sub.Err(), "traceID", traceID)
				return status.Errorf(codes.Canceled, "Subscription canceled with errors (see logs for trace ID: %s)", traceID)
			}
		default:
			continue
		}
		if sub.Err() != nil {
			logger.Error("New block subscription error", "err", sub.Err(), "traceID", traceID)
			return status.Errorf(codes.Internal, "New block subscription error (see logs for trace ID: %s)", traceID)
		}
	}
}

func validateBlockHeight(height, baseHeight, latestHeight int64) error {
	switch {
	case height <= 0:
		return status.Error(codes.InvalidArgument, "Height cannot be zero or negative")
	case height < baseHeight:
		return status.Errorf(codes.InvalidArgument, "Requested height %d is below base height %d", height, baseHeight)
	case height > latestHeight:
		return status.Errorf(codes.InvalidArgument, "Requested height %d is higher than latest height %d", height, latestHeight)
	}
	return nil
}

func getHeightFromMsg(msg cmtpubsub.Message) (int64, error) {
	switch eventType := msg.Data().(type) {
	case types.EventDataNewBlock:
		return eventType.Block.Height, nil
	default:
		return -1, fmt.Errorf("unexpected event type: %v", eventType)
	}
}

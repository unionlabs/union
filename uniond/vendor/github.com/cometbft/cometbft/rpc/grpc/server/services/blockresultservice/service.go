package blockresultservice

import (
	"context"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	brs "github.com/cometbft/cometbft/api/cometbft/services/block_results/v1"
	"github.com/cometbft/cometbft/libs/log"
	sm "github.com/cometbft/cometbft/state"
	"github.com/cometbft/cometbft/store"
)

type blockResultsService struct {
	stateStore sm.Store
	blockStore *store.BlockStore
	logger     log.Logger
}

// New creates a new CometBFT block results service server.
func New(bs *store.BlockStore, ss sm.Store, logger log.Logger) brs.BlockResultsServiceServer {
	return &blockResultsService{
		stateStore: ss,
		blockStore: bs,
		logger:     logger.With("service", "BlockResultsService"),
	}
}

// GetBlockResults returns the block results of the requested height.
func (s *blockResultsService) GetBlockResults(_ context.Context, req *brs.GetBlockResultsRequest) (*brs.GetBlockResultsResponse, error) {
	logger := s.logger.With("endpoint", "GetBlockResults")
	ss, err := s.stateStore.Load()
	if err != nil {
		logger.Error("Error loading store", "err", err)
		return nil, status.Error(codes.Internal, "Internal server error")
	}
	if req.Height > ss.LastBlockHeight || req.Height < 0 {
		return nil, status.Errorf(codes.InvalidArgument, "Height must be between 0 and the last effective height (%d)", ss.LastBlockHeight)
	}

	res, err := s.stateStore.LoadFinalizeBlockResponse(req.Height)
	if err != nil {
		logger.Error("Error fetching BlockResults", "height", req.Height, "err", err)
		return nil, status.Error(codes.Internal, "Internal server error")
	}

	return &brs.GetBlockResultsResponse{
		Height:              req.Height,
		TxResults:           res.TxResults,
		FinalizeBlockEvents: formatProtoToRef(res.Events),
		ValidatorUpdates:    formatProtoToRef(res.ValidatorUpdates),
		AppHash:             res.AppHash,
	}, nil
}

func formatProtoToRef[T any](collection []T) []*T {
	res := []*T{}
	for i := range collection {
		res = append(res, &collection[i])
	}
	return res
}

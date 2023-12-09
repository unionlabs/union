package keeper

import (
	"context"
	"encoding/hex"
	"fmt"

	proto "github.com/cosmos/gogoproto/proto"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	"cosmossdk.io/x/evidence/exported"
	"cosmossdk.io/x/evidence/types"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/query"
)

var _ types.QueryServer = Querier{}

type Querier struct {
	k *Keeper
}

func NewQuerier(keeper *Keeper) Querier {
	return Querier{k: keeper}
}

// Evidence implements the Query/Evidence gRPC method
func (k Querier) Evidence(c context.Context, req *types.QueryEvidenceRequest) (*types.QueryEvidenceResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}

	if req.Hash == "" {
		return nil, status.Errorf(codes.InvalidArgument, "invalid request; hash is empty")
	}

	ctx := sdk.UnwrapSDKContext(c)

	decodedHash, err := hex.DecodeString(req.Hash)
	if err != nil {
		return nil, fmt.Errorf("invalid evidence hash: %w", err)
	}

	evidence, _ := k.k.Evidences.Get(ctx, decodedHash)
	if evidence == nil {
		return nil, status.Errorf(codes.NotFound, "evidence %s not found", req.Hash)
	}

	msg, ok := evidence.(proto.Message)
	if !ok {
		return nil, status.Errorf(codes.Internal, "can't protomarshal %T", msg)
	}

	evidenceAny, err := codectypes.NewAnyWithValue(msg)
	if err != nil {
		return nil, status.Errorf(codes.Internal, err.Error())
	}

	return &types.QueryEvidenceResponse{Evidence: evidenceAny}, nil
}

// AllEvidence implements the Query/AllEvidence gRPC method
func (k Querier) AllEvidence(ctx context.Context, req *types.QueryAllEvidenceRequest) (*types.QueryAllEvidenceResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}

	evidences, pageRes, err := query.CollectionPaginate(ctx, k.k.Evidences, req.Pagination, func(_ []byte, value exported.Evidence) (*codectypes.Any, error) {
		return codectypes.NewAnyWithValue(value)
	})
	if err != nil {
		return nil, err
	}

	return &types.QueryAllEvidenceResponse{Evidence: evidences, Pagination: pageRes}, nil
}

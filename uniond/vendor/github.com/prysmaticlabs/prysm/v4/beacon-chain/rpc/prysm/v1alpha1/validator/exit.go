package validator

import (
	"context"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/blocks"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed"
	opfeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/operation"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// ProposeExit proposes an exit for a validator.
func (vs *Server) ProposeExit(ctx context.Context, req *ethpb.SignedVoluntaryExit) (*ethpb.ProposeExitResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "nil request")
	}
	s, err := vs.HeadFetcher.HeadStateReadOnly(ctx)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "Could not get head state: %v", err)
	}
	if req.Exit == nil {
		return nil, status.Error(codes.InvalidArgument, "voluntary exit does not exist")
	}
	if req.Signature == nil || len(req.Signature) != fieldparams.BLSSignatureLength {
		return nil, status.Error(codes.InvalidArgument, "invalid signature provided")
	}

	// Confirm the validator is eligible to exit with the parameters provided.
	val, err := s.ValidatorAtIndexReadOnly(req.Exit.ValidatorIndex)
	if err != nil {
		return nil, status.Error(codes.InvalidArgument, "validator index exceeds validator set length")
	}

	if err := blocks.VerifyExitAndSignature(val, s, req); err != nil {
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}

	vs.OperationNotifier.OperationFeed().Send(&feed.Event{
		Type: opfeed.ExitReceived,
		Data: &opfeed.ExitReceivedData{
			Exit: req,
		},
	})

	vs.ExitPool.InsertVoluntaryExit(req)

	r, err := req.Exit.HashTreeRoot()
	if err != nil {
		return nil, status.Errorf(codes.Internal, "Could not get tree hash of exit: %v", err)
	}

	return &ethpb.ProposeExitResponse{
		ExitRoot: r[:],
	}, vs.P2P.Broadcast(ctx, req)
}

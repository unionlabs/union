package validator

import (
	"context"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/core"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/emptypb"
)

// GetSyncMessageBlockRoot retrieves the sync committee block root of the beacon chain.
func (vs *Server) GetSyncMessageBlockRoot(
	ctx context.Context, _ *emptypb.Empty,
) (*ethpb.SyncMessageBlockRootResponse, error) {
	// An optimistic validator MUST NOT participate in sync committees
	// (i.e., sign across the DOMAIN_SYNC_COMMITTEE, DOMAIN_SYNC_COMMITTEE_SELECTION_PROOF or DOMAIN_CONTRIBUTION_AND_PROOF domains).
	if err := vs.optimisticStatus(ctx); err != nil {
		return nil, err
	}

	r, err := vs.HeadFetcher.HeadRoot(ctx)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "Could not retrieve head root: %v", err)
	}

	return &ethpb.SyncMessageBlockRootResponse{
		Root: r,
	}, nil
}

// SubmitSyncMessage submits the sync committee message to the network.
// It also saves the sync committee message into the pending pool for block inclusion.
func (vs *Server) SubmitSyncMessage(ctx context.Context, msg *ethpb.SyncCommitteeMessage) (*emptypb.Empty, error) {
	if err := vs.CoreService.SubmitSyncMessage(ctx, msg); err != nil {
		return &emptypb.Empty{}, status.Errorf(core.ErrorReasonToGRPC(err.Reason), err.Err.Error())
	}
	return &emptypb.Empty{}, nil
}

// GetSyncSubcommitteeIndex is called by a sync committee participant to get
// its subcommittee index for sync message aggregation duty.
func (vs *Server) GetSyncSubcommitteeIndex(
	ctx context.Context, req *ethpb.SyncSubcommitteeIndexRequest,
) (*ethpb.SyncSubcommitteeIndexResponse, error) {
	index, exists := vs.HeadFetcher.HeadPublicKeyToValidatorIndex(bytesutil.ToBytes48(req.PublicKey))
	if !exists {
		return nil, errors.New("public key does not exist in state")
	}
	indices, err := vs.HeadFetcher.HeadSyncCommitteeIndices(ctx, index, req.Slot)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "Could not get sync subcommittee index: %v", err)
	}
	return &ethpb.SyncSubcommitteeIndexResponse{Indices: indices}, nil
}

// GetSyncCommitteeContribution is called by a sync committee aggregator
// to retrieve sync committee contribution object.
func (vs *Server) GetSyncCommitteeContribution(
	ctx context.Context, req *ethpb.SyncCommitteeContributionRequest,
) (*ethpb.SyncCommitteeContribution, error) {
	// An optimistic validator MUST NOT participate in sync committees
	// (i.e., sign across the DOMAIN_SYNC_COMMITTEE, DOMAIN_SYNC_COMMITTEE_SELECTION_PROOF or DOMAIN_CONTRIBUTION_AND_PROOF domains).
	if err := vs.optimisticStatus(ctx); err != nil {
		return nil, err
	}

	msgs, err := vs.SyncCommitteePool.SyncCommitteeMessages(req.Slot)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "Could not get sync subcommittee messages: %v", err)
	}
	headRoot, err := vs.HeadFetcher.HeadRoot(ctx)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "Could not get head root: %v", err)
	}
	sig, aggregatedBits, err := vs.CoreService.AggregatedSigAndAggregationBits(
		ctx,
		&ethpb.AggregatedSigAndAggregationBitsRequest{
			Msgs:      msgs,
			Slot:      req.Slot,
			SubnetId:  req.SubnetId,
			BlockRoot: headRoot,
		})
	if err != nil {
		return nil, status.Errorf(codes.Internal, "Could not get contribution data: %v", err)
	}
	contribution := &ethpb.SyncCommitteeContribution{
		Slot:              req.Slot,
		BlockRoot:         headRoot,
		SubcommitteeIndex: req.SubnetId,
		AggregationBits:   aggregatedBits,
		Signature:         sig,
	}

	return contribution, nil
}

// SubmitSignedContributionAndProof is called by a sync committee aggregator
// to submit signed contribution and proof object.
func (vs *Server) SubmitSignedContributionAndProof(
	ctx context.Context, s *ethpb.SignedContributionAndProof,
) (*emptypb.Empty, error) {
	err := vs.CoreService.SubmitSignedContributionAndProof(ctx, s)
	if err != nil {
		return &emptypb.Empty{}, status.Errorf(core.ErrorReasonToGRPC(err.Reason), err.Err.Error())
	}
	return &emptypb.Empty{}, nil
}

// AggregatedSigAndAggregationBits returns the aggregated signature and aggregation bits
// associated with a particular set of sync committee messages.
func (vs *Server) AggregatedSigAndAggregationBits(
	ctx context.Context,
	req *ethpb.AggregatedSigAndAggregationBitsRequest,
) (*ethpb.AggregatedSigAndAggregationBitsResponse, error) {
	sig, aggregatedBits, err := vs.CoreService.AggregatedSigAndAggregationBits(ctx, req)
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}
	return &ethpb.AggregatedSigAndAggregationBitsResponse{
		AggregatedSig: sig,
		Bits:          aggregatedBits,
	}, nil
}

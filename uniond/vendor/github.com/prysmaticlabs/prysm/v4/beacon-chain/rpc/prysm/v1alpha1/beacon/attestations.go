package beacon

import (
	"context"
	"sort"
	"strconv"
	"strings"

	"github.com/prysmaticlabs/prysm/v4/api/pagination"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/operation"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/filters"
	"github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/attestation"
	attaggregation "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/attestation/aggregation/attestations"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/emptypb"
)

// sortableAttestations implements the Sort interface to sort attestations
// by slot as the canonical sorting attribute.
type sortableAttestations []*ethpb.Attestation

// Len is the number of elements in the collection.
func (s sortableAttestations) Len() int { return len(s) }

// Swap swaps the elements with indexes i and j.
func (s sortableAttestations) Swap(i, j int) { s[i], s[j] = s[j], s[i] }

// Less reports whether the element with index i must sort before the element with index j.
func (s sortableAttestations) Less(i, j int) bool {
	return s[i].Data.Slot < s[j].Data.Slot
}

func mapAttestationsByTargetRoot(atts []*ethpb.Attestation) map[[32]byte][]*ethpb.Attestation {
	attsMap := make(map[[32]byte][]*ethpb.Attestation, len(atts))
	if len(atts) == 0 {
		return attsMap
	}
	for _, att := range atts {
		attsMap[bytesutil.ToBytes32(att.Data.Target.Root)] = append(attsMap[bytesutil.ToBytes32(att.Data.Target.Root)], att)
	}
	return attsMap
}

// ListAttestations retrieves attestations by block root, slot, or epoch.
// Attestations are sorted by data slot by default.
//
// The server may return an empty list when no attestations match the given
// filter criteria. This RPC should not return NOT_FOUND. Only one filter
// criteria should be used.
func (bs *Server) ListAttestations(
	ctx context.Context, req *ethpb.ListAttestationsRequest,
) (*ethpb.ListAttestationsResponse, error) {
	if int(req.PageSize) > cmd.Get().MaxRPCPageSize {
		return nil, status.Errorf(codes.InvalidArgument, "Requested page size %d can not be greater than max size %d",
			req.PageSize, cmd.Get().MaxRPCPageSize)
	}
	var blocks []interfaces.ReadOnlySignedBeaconBlock
	var err error
	switch q := req.QueryFilter.(type) {
	case *ethpb.ListAttestationsRequest_GenesisEpoch:
		blocks, _, err = bs.BeaconDB.Blocks(ctx, filters.NewFilter().SetStartEpoch(0).SetEndEpoch(0))
		if err != nil {
			return nil, status.Errorf(codes.Internal, "Could not fetch attestations: %v", err)
		}
	case *ethpb.ListAttestationsRequest_Epoch:
		blocks, _, err = bs.BeaconDB.Blocks(ctx, filters.NewFilter().SetStartEpoch(q.Epoch).SetEndEpoch(q.Epoch))
		if err != nil {
			return nil, status.Errorf(codes.Internal, "Could not fetch attestations: %v", err)
		}
	default:
		return nil, status.Error(codes.InvalidArgument, "Must specify a filter criteria for fetching attestations")
	}
	atts := make([]*ethpb.Attestation, 0, params.BeaconConfig().MaxAttestations*uint64(len(blocks)))
	for _, blk := range blocks {
		atts = append(atts, blk.Block().Body().Attestations()...)
	}
	// We sort attestations according to the Sortable interface.
	sort.Sort(sortableAttestations(atts))
	numAttestations := len(atts)

	// If there are no attestations, we simply return a response specifying this.
	// Otherwise, attempting to paginate 0 attestations below would result in an error.
	if numAttestations == 0 {
		return &ethpb.ListAttestationsResponse{
			Attestations:  make([]*ethpb.Attestation, 0),
			TotalSize:     int32(0),
			NextPageToken: strconv.Itoa(0),
		}, nil
	}

	start, end, nextPageToken, err := pagination.StartAndEndPage(req.PageToken, int(req.PageSize), numAttestations)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "Could not paginate attestations: %v", err)
	}
	return &ethpb.ListAttestationsResponse{
		Attestations:  atts[start:end],
		TotalSize:     int32(numAttestations),
		NextPageToken: nextPageToken,
	}, nil
}

// ListIndexedAttestations retrieves indexed attestations by block root.
// IndexedAttestationsForEpoch are sorted by data slot by default. Start-end epoch
// filter is used to retrieve blocks with.
//
// The server may return an empty list when no attestations match the given
// filter criteria. This RPC should not return NOT_FOUND.
func (bs *Server) ListIndexedAttestations(
	ctx context.Context, req *ethpb.ListIndexedAttestationsRequest,
) (*ethpb.ListIndexedAttestationsResponse, error) {
	var blocks []interfaces.ReadOnlySignedBeaconBlock
	var err error
	switch q := req.QueryFilter.(type) {
	case *ethpb.ListIndexedAttestationsRequest_GenesisEpoch:
		blocks, _, err = bs.BeaconDB.Blocks(ctx, filters.NewFilter().SetStartEpoch(0).SetEndEpoch(0))
		if err != nil {
			return nil, status.Errorf(codes.Internal, "Could not fetch attestations: %v", err)
		}
	case *ethpb.ListIndexedAttestationsRequest_Epoch:
		blocks, _, err = bs.BeaconDB.Blocks(ctx, filters.NewFilter().SetStartEpoch(q.Epoch).SetEndEpoch(q.Epoch))
		if err != nil {
			return nil, status.Errorf(codes.Internal, "Could not fetch attestations: %v", err)
		}
	default:
		return nil, status.Error(codes.InvalidArgument, "Must specify a filter criteria for fetching attestations")
	}

	attsArray := make([]*ethpb.Attestation, 0, params.BeaconConfig().MaxAttestations*uint64(len(blocks)))
	for _, b := range blocks {
		attsArray = append(attsArray, b.Block().Body().Attestations()...)
	}
	// We sort attestations according to the Sortable interface.
	sort.Sort(sortableAttestations(attsArray))
	numAttestations := len(attsArray)

	// If there are no attestations, we simply return a response specifying this.
	// Otherwise, attempting to paginate 0 attestations below would result in an error.
	if numAttestations == 0 {
		return &ethpb.ListIndexedAttestationsResponse{
			IndexedAttestations: make([]*ethpb.IndexedAttestation, 0),
			TotalSize:           int32(0),
			NextPageToken:       strconv.Itoa(0),
		}, nil
	}
	// We use the retrieved committees for the b root to convert all attestations
	// into indexed form effectively.
	mappedAttestations := mapAttestationsByTargetRoot(attsArray)
	indexedAtts := make([]*ethpb.IndexedAttestation, 0, numAttestations)
	for targetRoot, atts := range mappedAttestations {
		attState, err := bs.StateGen.StateByRoot(ctx, targetRoot)
		if err != nil && strings.Contains(err.Error(), "unknown state summary") {
			// We shouldn't stop the request if we encounter an attestation we don't have the state for.
			log.Debugf("Could not get state for attestation target root %#x", targetRoot)
			continue
		} else if err != nil {
			return nil, status.Errorf(
				codes.Internal,
				"Could not retrieve state for attestation target root %#x: %v",
				targetRoot,
				err,
			)
		}
		for i := 0; i < len(atts); i++ {
			att := atts[i]
			committee, err := helpers.BeaconCommitteeFromState(ctx, attState, att.Data.Slot, att.Data.CommitteeIndex)
			if err != nil {
				return nil, status.Errorf(
					codes.Internal,
					"Could not retrieve committee from state %v",
					err,
				)
			}
			idxAtt, err := attestation.ConvertToIndexed(ctx, att, committee)
			if err != nil {
				return nil, err
			}
			indexedAtts = append(indexedAtts, idxAtt)
		}
	}

	start, end, nextPageToken, err := pagination.StartAndEndPage(req.PageToken, int(req.PageSize), len(indexedAtts))
	if err != nil {
		return nil, status.Errorf(codes.Internal, "Could not paginate attestations: %v", err)
	}
	return &ethpb.ListIndexedAttestationsResponse{
		IndexedAttestations: indexedAtts[start:end],
		TotalSize:           int32(len(indexedAtts)),
		NextPageToken:       nextPageToken,
	}, nil
}

// StreamAttestations to clients at the end of every slot. This method retrieves the
// aggregated attestations currently in the pool at the start of a slot and sends
// them over a gRPC stream.
// DEPRECATED: This endpoint is superseded by the /eth/v1/events Beacon API endpoint
func (bs *Server) StreamAttestations(
	_ *emptypb.Empty, stream ethpb.BeaconChain_StreamAttestationsServer,
) error {
	attestationsChannel := make(chan *feed.Event, 1)
	attSub := bs.AttestationNotifier.OperationFeed().Subscribe(attestationsChannel)
	defer attSub.Unsubscribe()
	for {
		select {
		case event := <-attestationsChannel:
			if event.Type == operation.UnaggregatedAttReceived {
				data, ok := event.Data.(*operation.UnAggregatedAttReceivedData)
				if !ok {
					// Got bad data over the stream.
					continue
				}
				if data.Attestation == nil {
					// One nil attestation shouldn't stop the stream.
					continue
				}
				if err := stream.Send(data.Attestation); err != nil {
					return status.Errorf(codes.Unavailable, "Could not send over stream: %v", err)
				}
			}
		case <-bs.Ctx.Done():
			return status.Error(codes.Canceled, "Context canceled")
		case <-stream.Context().Done():
			return status.Error(codes.Canceled, "Context canceled")
		}
	}
}

// StreamIndexedAttestations to clients at the end of every slot. This method retrieves the
// aggregated attestations currently in the pool, converts them into indexed form, and
// sends them over a gRPC stream.
// DEPRECATED: This endpoint is superseded by the /eth/v1/events Beacon API endpoint
func (bs *Server) StreamIndexedAttestations(
	_ *emptypb.Empty, stream ethpb.BeaconChain_StreamIndexedAttestationsServer,
) error {
	attestationsChannel := make(chan *feed.Event, 1)
	attSub := bs.AttestationNotifier.OperationFeed().Subscribe(attestationsChannel)
	defer attSub.Unsubscribe()
	go bs.collectReceivedAttestations(stream.Context())
	for {
		select {
		case event, ok := <-attestationsChannel:
			if !ok {
				log.Error("Indexed attestations stream channel closed")
				continue
			}
			if event.Type == operation.UnaggregatedAttReceived {
				data, ok := event.Data.(*operation.UnAggregatedAttReceivedData)
				if !ok {
					// Got bad data over the stream.
					log.Warningf("Indexed attestations stream got data of wrong type on stream expected *UnAggregatedAttReceivedData, received %T", event.Data)
					continue
				}
				if data.Attestation == nil {
					// One nil attestation shouldn't stop the stream.
					log.Debug("Indexed attestations stream got a nil attestation")
					continue
				}
				bs.ReceivedAttestationsBuffer <- data.Attestation
			} else if event.Type == operation.AggregatedAttReceived {
				data, ok := event.Data.(*operation.AggregatedAttReceivedData)
				if !ok {
					// Got bad data over the stream.
					log.Warningf("Indexed attestations stream got data of wrong type on stream expected *AggregatedAttReceivedData, received %T", event.Data)
					continue
				}
				if data.Attestation == nil || data.Attestation.Aggregate == nil {
					// One nil attestation shouldn't stop the stream.
					log.Debug("Indexed attestations stream got nil attestation or nil attestation aggregate")
					continue
				}
				bs.ReceivedAttestationsBuffer <- data.Attestation.Aggregate
			}
		case aggAtts, ok := <-bs.CollectedAttestationsBuffer:
			if !ok {
				log.Error("Indexed attestations stream collected attestations channel closed")
				continue
			}
			if len(aggAtts) == 0 {
				continue
			}
			// All attestations we receive have the same target epoch given they
			// have the same data root, so we just use the target epoch from
			// the first one to determine committees for converting into indexed
			// form.
			targetRoot := aggAtts[0].Data.Target.Root
			targetEpoch := aggAtts[0].Data.Target.Epoch
			committeesBySlot, _, err := bs.retrieveCommitteesForRoot(stream.Context(), targetRoot)
			if err != nil {
				return status.Errorf(
					codes.Internal,
					"Could not retrieve committees for target root %#x: %v",
					targetRoot,
					err,
				)
			}
			// We use the retrieved committees for the epoch to convert all attestations
			// into indexed form effectively.
			startSlot, err := slots.EpochStart(targetEpoch)
			if err != nil {
				log.Error(err)
				continue
			}
			endSlot := startSlot + params.BeaconConfig().SlotsPerEpoch
			for _, att := range aggAtts {
				// Out of range check, the attestation slot cannot be greater
				// the last slot of the requested epoch or smaller than its start slot
				// given committees are accessed as a map of slot -> committees list, where there are
				// SLOTS_PER_EPOCH keys in the map.
				if att.Data.Slot < startSlot || att.Data.Slot > endSlot {
					continue
				}
				committeesForSlot, ok := committeesBySlot[att.Data.Slot]
				if !ok || committeesForSlot.Committees == nil {
					continue
				}
				committee := committeesForSlot.Committees[att.Data.CommitteeIndex]
				idxAtt, err := attestation.ConvertToIndexed(stream.Context(), att, committee.ValidatorIndices)
				if err != nil {
					continue
				}
				if err := stream.Send(idxAtt); err != nil {
					return status.Errorf(codes.Unavailable, "Could not send over stream: %v", err)
				}
			}
		case <-bs.Ctx.Done():
			return status.Error(codes.Canceled, "Context canceled")
		case <-stream.Context().Done():
			return status.Error(codes.Canceled, "Context canceled")
		}
	}
}

// already being done by the attestation pool in the operations service.
func (bs *Server) collectReceivedAttestations(ctx context.Context) {
	attsByRoot := make(map[[32]byte][]*ethpb.Attestation)
	twoThirdsASlot := 2 * slots.DivideSlotBy(3) /* 2/3 slot duration */
	ticker := slots.NewSlotTickerWithOffset(bs.GenesisTimeFetcher.GenesisTime(), twoThirdsASlot, params.BeaconConfig().SecondsPerSlot)
	for {
		select {
		case <-ticker.C():
			aggregatedAttsByTarget := make(map[[32]byte][]*ethpb.Attestation)
			for root, atts := range attsByRoot {
				// We aggregate the received attestations, we know they all have the same data root.
				aggAtts, err := attaggregation.Aggregate(atts)
				if err != nil {
					log.WithError(err).Error("Could not aggregate attestations")
					continue
				}
				if len(aggAtts) == 0 {
					continue
				}
				targetRoot := bytesutil.ToBytes32(atts[0].Data.Target.Root)
				aggregatedAttsByTarget[targetRoot] = append(aggregatedAttsByTarget[targetRoot], aggAtts...)
				attsByRoot[root] = make([]*ethpb.Attestation, 0)
			}
			for _, atts := range aggregatedAttsByTarget {
				bs.CollectedAttestationsBuffer <- atts
			}
		case att := <-bs.ReceivedAttestationsBuffer:
			attDataRoot, err := att.Data.HashTreeRoot()
			if err != nil {
				log.WithError(err).Error("Could not hash tree root attestation data")
				continue
			}
			attsByRoot[attDataRoot] = append(attsByRoot[attDataRoot], att)
		case <-ctx.Done():
			return
		case <-bs.Ctx.Done():
			return
		}
	}
}

// AttestationPool retrieves pending attestations.
//
// The server returns a list of attestations that have been seen but not
// yet processed. Pool attestations eventually expire as the slot
// advances, so an attestation missing from this request does not imply
// that it was included in a block. The attestation may have expired.
// Refer to the ethereum consensus specification for more details on how
// attestations are processed and when they are no longer valid.
// https://github.com/ethereum/consensus-specs/blob/dev/specs/core/0_beacon-chain.md#attestations
func (bs *Server) AttestationPool(
	_ context.Context, req *ethpb.AttestationPoolRequest,
) (*ethpb.AttestationPoolResponse, error) {
	if int(req.PageSize) > cmd.Get().MaxRPCPageSize {
		return nil, status.Errorf(
			codes.InvalidArgument,
			"Requested page size %d can not be greater than max size %d",
			req.PageSize,
			cmd.Get().MaxRPCPageSize,
		)
	}
	atts := bs.AttestationsPool.AggregatedAttestations()
	numAtts := len(atts)
	if numAtts == 0 {
		return &ethpb.AttestationPoolResponse{
			Attestations:  make([]*ethpb.Attestation, 0),
			TotalSize:     int32(0),
			NextPageToken: strconv.Itoa(0),
		}, nil
	}
	start, end, nextPageToken, err := pagination.StartAndEndPage(req.PageToken, int(req.PageSize), numAtts)
	if err != nil {
		return nil, status.Errorf(codes.Internal, "Could not paginate attestations: %v", err)
	}
	return &ethpb.AttestationPoolResponse{
		Attestations:  atts[start:end],
		TotalSize:     int32(numAtts),
		NextPageToken: nextPageToken,
	}, nil
}

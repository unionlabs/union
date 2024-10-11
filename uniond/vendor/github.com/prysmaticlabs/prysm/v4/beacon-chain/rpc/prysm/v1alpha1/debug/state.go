package debug

import (
	"context"
	"fmt"

	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	pbrpc "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// GetBeaconState retrieves an ssz-encoded beacon state
// from the beacon node by either a slot or block root.
func (ds *Server) GetBeaconState(
	ctx context.Context,
	req *pbrpc.BeaconStateRequest,
) (*pbrpc.SSZResponse, error) {
	switch q := req.QueryFilter.(type) {
	case *pbrpc.BeaconStateRequest_Slot:
		currentSlot := ds.GenesisTimeFetcher.CurrentSlot()
		requestedSlot := q.Slot
		if requestedSlot > currentSlot {
			return nil, status.Errorf(
				codes.InvalidArgument,
				"Cannot retrieve information about a slot in the future, current slot %d, requested slot %d",
				currentSlot,
				requestedSlot,
			)
		}

		st, err := ds.ReplayerBuilder.ReplayerForSlot(q.Slot).ReplayBlocks(ctx)
		if err != nil {
			return nil, status.Error(codes.Internal, fmt.Sprintf("error replaying blocks for state at slot %d: %v", q.Slot, err))
		}

		encoded, err := st.MarshalSSZ()
		if err != nil {
			return nil, status.Errorf(codes.Internal, "Could not ssz encode beacon state: %v", err)
		}
		return &pbrpc.SSZResponse{
			Encoded: encoded,
		}, nil
	case *pbrpc.BeaconStateRequest_BlockRoot:
		st, err := ds.StateGen.StateByRoot(ctx, bytesutil.ToBytes32(q.BlockRoot))
		if err != nil {
			return nil, status.Errorf(codes.Internal, "Could not compute state by block root: %v", err)
		}
		encoded, err := st.MarshalSSZ()
		if err != nil {
			return nil, status.Errorf(codes.Internal, "Could not ssz encode beacon state: %v", err)
		}
		return &pbrpc.SSZResponse{
			Encoded: encoded,
		}, nil
	default:
		return nil, status.Error(codes.InvalidArgument, "Need to specify either a block root or slot to request state")
	}
}

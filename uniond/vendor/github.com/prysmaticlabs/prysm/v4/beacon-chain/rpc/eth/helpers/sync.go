package helpers

import (
	"bytes"
	"context"
	"strconv"
	"strings"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/api/grpc"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/lookup"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/sync"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// ValidateSyncGRPC checks whether the node is currently syncing and returns an error if it is.
// It also appends syncing info to gRPC headers.
func ValidateSyncGRPC(
	ctx context.Context,
	syncChecker sync.Checker,
	headFetcher blockchain.HeadFetcher,
	timeFetcher blockchain.TimeFetcher,
	optimisticModeFetcher blockchain.OptimisticModeFetcher,
) error {
	if !syncChecker.Syncing() {
		return nil
	}
	headSlot := headFetcher.HeadSlot()
	isOptimistic, err := optimisticModeFetcher.IsOptimistic(ctx)
	if err != nil {
		return status.Errorf(codes.Internal, "Could not check optimistic status: %v", err)
	}

	syncDetailsContainer := &shared.SyncDetailsContainer{
		Data: &shared.SyncDetails{
			HeadSlot:     strconv.FormatUint(uint64(headSlot), 10),
			SyncDistance: strconv.FormatUint(uint64(timeFetcher.CurrentSlot()-headSlot), 10),
			IsSyncing:    true,
			IsOptimistic: isOptimistic,
		},
	}

	err = grpc.AppendCustomErrorHeader(ctx, syncDetailsContainer)
	if err != nil {
		return status.Errorf(
			codes.Internal,
			"Syncing to latest head, not ready to respond. Could not prepare sync details: %v",
			err,
		)
	}
	return status.Error(codes.Unavailable, "Syncing to latest head, not ready to respond")
}

// IsOptimistic checks whether the beacon state's block is optimistic.
func IsOptimistic(
	ctx context.Context,
	stateId []byte,
	optimisticModeFetcher blockchain.OptimisticModeFetcher,
	stateFetcher lookup.Stater,
	chainInfo blockchain.ChainInfoFetcher,
	database db.ReadOnlyDatabase,
) (bool, error) {
	stateIdString := strings.ToLower(string(stateId))
	switch stateIdString {
	case "head":
		return optimisticModeFetcher.IsOptimistic(ctx)
	case "genesis":
		return false, nil
	case "finalized":
		fcp := chainInfo.FinalizedCheckpt()
		if fcp == nil {
			return true, errors.New("received nil finalized checkpoint")
		}
		// Special genesis case in the event our checkpoint root is a zerohash.
		if bytes.Equal(fcp.Root, params.BeaconConfig().ZeroHash[:]) {
			return false, nil
		}
		return optimisticModeFetcher.IsOptimisticForRoot(ctx, bytesutil.ToBytes32(fcp.Root))
	case "justified":
		jcp := chainInfo.CurrentJustifiedCheckpt()
		if jcp == nil {
			return true, errors.New("received nil justified checkpoint")
		}
		// Special genesis case in the event our checkpoint root is a zerohash.
		if bytes.Equal(jcp.Root, params.BeaconConfig().ZeroHash[:]) {
			return false, nil
		}
		return optimisticModeFetcher.IsOptimisticForRoot(ctx, bytesutil.ToBytes32(jcp.Root))
	default:
		if len(stateIdString) >= 2 && stateIdString[:2] == "0x" {
			id, err := hexutil.Decode(stateIdString)
			if err != nil {
				return false, err
			}
			return isStateRootOptimistic(ctx, id, optimisticModeFetcher, stateFetcher, chainInfo, database)
		} else if len(stateId) == 32 {
			return isStateRootOptimistic(ctx, stateId, optimisticModeFetcher, stateFetcher, chainInfo, database)
		} else {
			optimistic, err := optimisticModeFetcher.IsOptimistic(ctx)
			if err != nil {
				return true, errors.Wrap(err, "could not check optimistic status")
			}
			if !optimistic {
				return false, nil
			}
			slotNumber, parseErr := strconv.ParseUint(stateIdString, 10, 64)
			if parseErr != nil {
				// ID format does not match any valid options.
				e := lookup.NewStateIdParseError(parseErr)
				return true, &e
			}
			fcp := chainInfo.FinalizedCheckpt()
			if fcp == nil {
				return true, errors.New("received nil finalized checkpoint")
			}
			finalizedSlot, err := slots.EpochStart(fcp.Epoch)
			if err != nil {
				return true, errors.Wrap(err, "could not get head state's finalized slot")
			}
			lastValidatedCheckpoint, err := database.LastValidatedCheckpoint(ctx)
			if err != nil {
				return true, errors.Wrap(err, "could not get last validated checkpoint")
			}
			validatedSlot, err := slots.EpochStart(lastValidatedCheckpoint.Epoch)
			if err != nil {
				return true, errors.Wrap(err, "could not get last validated slot")
			}
			if primitives.Slot(slotNumber) <= validatedSlot {
				return false, nil
			}
			// if the finalized checkpoint is higher than the last
			// validated checkpoint, we are syncing and have synced
			// a finalization being optimistic
			if validatedSlot < finalizedSlot {
				return true, nil
			}
			if primitives.Slot(slotNumber) == chainInfo.HeadSlot() {
				// We know the head is optimistic because we checked it above.
				return true, nil
			}
			headRoot, err := chainInfo.HeadRoot(ctx)
			if err != nil {
				return true, errors.Wrap(err, "could not get head root")
			}
			r, err := chainInfo.Ancestor(ctx, headRoot, primitives.Slot(slotNumber))
			if err != nil {
				return true, errors.Wrap(err, "could not get ancestor root")
			}
			return optimisticModeFetcher.IsOptimisticForRoot(ctx, bytesutil.ToBytes32(r))
		}
	}
}

func isStateRootOptimistic(
	ctx context.Context,
	stateId []byte,
	optimisticModeFetcher blockchain.OptimisticModeFetcher,
	stateFetcher lookup.Stater,
	chainInfo blockchain.ChainInfoFetcher,
	database db.ReadOnlyDatabase,
) (bool, error) {
	st, err := stateFetcher.State(ctx, stateId)
	if err != nil {
		return true, errors.Wrap(err, "could not fetch state")
	}
	if st.Slot() == chainInfo.HeadSlot() {
		return optimisticModeFetcher.IsOptimistic(ctx)
	}
	has, roots, err := database.BlockRootsBySlot(ctx, st.Slot())
	if err != nil {
		return true, errors.Wrapf(err, "could not get block roots for slot %d", st.Slot())
	}
	if !has {
		return true, errors.New("no block roots returned from the database")
	}
	for _, r := range roots {
		b, err := database.Block(ctx, r)
		if err != nil {
			return true, errors.Wrapf(err, "could not obtain block")
		}
		if bytesutil.ToBytes32(stateId) != b.Block().StateRoot() {
			continue
		}
		return optimisticModeFetcher.IsOptimisticForRoot(ctx, r)
	}
	// No block matching requested state root, return true.
	return true, nil
}

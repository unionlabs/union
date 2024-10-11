package beacon

import (
	"context"
	"fmt"
	"path"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/pkg/errors"
	base "github.com/prysmaticlabs/prysm/v4/api/client"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/encoding/ssz/detect"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	log "github.com/sirupsen/logrus"
	"golang.org/x/mod/semver"
)

var errCheckpointBlockMismatch = errors.New("mismatch between checkpoint sync state and block")

// OriginData represents the BeaconState and ReadOnlySignedBeaconBlock necessary to start an empty Beacon Node
// using Checkpoint Sync.
type OriginData struct {
	sb []byte
	bb []byte
	st state.BeaconState
	b  interfaces.ReadOnlySignedBeaconBlock
	vu *detect.VersionedUnmarshaler
	br [32]byte
	sr [32]byte
}

// SaveBlock saves the downloaded block to a unique file in the given path.
// For readability and collision avoidance, the file name includes: type, config name, slot and root
func (o *OriginData) SaveBlock(dir string) (string, error) {
	blockPath := path.Join(dir, fname("block", o.vu, o.b.Block().Slot(), o.br))
	return blockPath, file.WriteFile(blockPath, o.BlockBytes())
}

// SaveState saves the downloaded state to a unique file in the given path.
// For readability and collision avoidance, the file name includes: type, config name, slot and root
func (o *OriginData) SaveState(dir string) (string, error) {
	statePath := path.Join(dir, fname("state", o.vu, o.st.Slot(), o.sr))
	return statePath, file.WriteFile(statePath, o.StateBytes())
}

// StateBytes returns the ssz-encoded bytes of the downloaded BeaconState value.
func (o *OriginData) StateBytes() []byte {
	return o.sb
}

// BlockBytes returns the ssz-encoded bytes of the downloaded ReadOnlySignedBeaconBlock value.
func (o *OriginData) BlockBytes() []byte {
	return o.bb
}

func fname(prefix string, vu *detect.VersionedUnmarshaler, slot primitives.Slot, root [32]byte) string {
	return fmt.Sprintf("%s_%s_%s_%d-%#x.ssz", prefix, vu.Config.ConfigName, version.String(vu.Fork), slot, root)
}

// DownloadFinalizedData downloads the most recently finalized state, and the block most recently applied to that state.
// This pair can be used to initialize a new beacon node via checkpoint sync.
func DownloadFinalizedData(ctx context.Context, client *Client) (*OriginData, error) {
	sb, err := client.GetState(ctx, IdFinalized)
	if err != nil {
		return nil, err
	}
	vu, err := detect.FromState(sb)
	if err != nil {
		return nil, errors.Wrap(err, "error detecting chain config for finalized state")
	}
	log.Printf("detected supported config in remote finalized state, name=%s, fork=%s", vu.Config.ConfigName, version.String(vu.Fork))
	s, err := vu.UnmarshalBeaconState(sb)
	if err != nil {
		return nil, errors.Wrap(err, "error unmarshaling finalized state to correct version")
	}

	slot := s.LatestBlockHeader().Slot
	bb, err := client.GetBlock(ctx, IdFromSlot(slot))
	if err != nil {
		return nil, errors.Wrapf(err, "error requesting block by slot = %d", slot)
	}
	b, err := vu.UnmarshalBeaconBlock(bb)
	if err != nil {
		return nil, errors.Wrap(err, "unable to unmarshal block to a supported type using the detected fork schedule")
	}
	br, err := b.Block().HashTreeRoot()
	if err != nil {
		return nil, errors.Wrap(err, "error computing hash_tree_root of retrieved block")
	}
	bodyRoot, err := b.Block().Body().HashTreeRoot()
	if err != nil {
		return nil, errors.Wrap(err, "error computing hash_tree_root of retrieved block body")
	}

	sbr := bytesutil.ToBytes32(s.LatestBlockHeader().BodyRoot)
	if sbr != bodyRoot {
		return nil, errors.Wrapf(errCheckpointBlockMismatch, "state body root = %#x, block body root = %#x", sbr, bodyRoot)
	}
	sr, err := s.HashTreeRoot(ctx)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to compute htr for finalized state at slot=%d", s.Slot())
	}

	log.
		WithField("block_slot", b.Block().Slot()).
		WithField("state_slot", s.Slot()).
		WithField("state_root", hexutil.Encode(sr[:])).
		WithField("block_root", hexutil.Encode(br[:])).
		Info("Downloaded checkpoint sync state and block.")
	return &OriginData{
		st: s,
		b:  b,
		sb: sb,
		bb: bb,
		vu: vu,
		br: br,
		sr: sr,
	}, nil
}

// WeakSubjectivityData represents the state root, block root and epoch of the BeaconState + ReadOnlySignedBeaconBlock
// that falls at the beginning of the current weak subjectivity period. These values can be used to construct
// a weak subjectivity checkpoint beacon node flag to be used for validation.
type WeakSubjectivityData struct {
	BlockRoot [32]byte
	StateRoot [32]byte
	Epoch     primitives.Epoch
}

// CheckpointString returns the standard string representation of a Checkpoint.
// The format is a hex-encoded block root, followed by the epoch of the block, separated by a colon. For example:
// "0x1c35540cac127315fabb6bf29181f2ae0de1a3fc909d2e76ba771e61312cc49a:74888"
func (wsd *WeakSubjectivityData) CheckpointString() string {
	return fmt.Sprintf("%#x:%d", wsd.BlockRoot, wsd.Epoch)
}

// ComputeWeakSubjectivityCheckpoint attempts to use the prysm weak_subjectivity api
// to obtain the current weak_subjectivity checkpoint.
// For non-prysm nodes, the same computation will be performed with extra steps,
// using the head state downloaded from the beacon node api.
func ComputeWeakSubjectivityCheckpoint(ctx context.Context, client *Client) (*WeakSubjectivityData, error) {
	ws, err := client.GetWeakSubjectivity(ctx)
	if err != nil {
		// a 404/405 is expected if querying an endpoint that doesn't support the weak subjectivity checkpoint api
		if !errors.Is(err, base.ErrNotOK) {
			return nil, errors.Wrap(err, "unexpected API response for prysm-only weak subjectivity checkpoint API")
		}
		// fall back to vanilla Beacon Node API method
		return computeBackwardsCompatible(ctx, client)
	}
	log.Printf("server weak subjectivity checkpoint response - epoch=%d, block_root=%#x, state_root=%#x", ws.Epoch, ws.BlockRoot, ws.StateRoot)
	return ws, nil
}

const (
	prysmMinimumVersion     = "v2.0.7"
	prysmImplementationName = "Prysm"
)

// errUnsupportedPrysmCheckpointVersion indicates remote beacon node can't be used for checkpoint retrieval.
var errUnsupportedPrysmCheckpointVersion = errors.New("node does not meet minimum version requirements for checkpoint retrieval")

// for older endpoints or clients that do not support the weak_subjectivity api method
// we gather the necessary data for a checkpoint sync by:
// - inspecting the remote server's head state and computing the weak subjectivity epoch locally
// - requesting the state at the first slot of the epoch
// - using hash_tree_root(state.latest_block_header) to compute the block the state integrates
// - requesting that block by its root
func computeBackwardsCompatible(ctx context.Context, client *Client) (*WeakSubjectivityData, error) {
	log.Print("falling back to generic checkpoint derivation, weak_subjectivity API not supported by server")
	nv, err := client.GetNodeVersion(ctx)
	if err != nil {
		return nil, errors.Wrap(err, "unable to proceed with fallback method without confirming node version")
	}
	if nv.implementation == prysmImplementationName && semver.Compare(nv.semver, prysmMinimumVersion) < 0 {
		return nil, errors.Wrapf(errUnsupportedPrysmCheckpointVersion, "%s < minimum (%s)", nv.semver, prysmMinimumVersion)
	}
	epoch, err := getWeakSubjectivityEpochFromHead(ctx, client)
	if err != nil {
		return nil, errors.Wrap(err, "error computing weak subjectivity epoch via head state inspection")
	}

	// use first slot of the epoch for the state slot
	slot, err := slots.EpochStart(epoch)
	if err != nil {
		return nil, errors.Wrapf(err, "error computing first slot of epoch=%d", epoch)
	}

	log.Printf("requesting checkpoint state at slot %d", slot)
	// get the state at the first slot of the epoch
	sb, err := client.GetState(ctx, IdFromSlot(slot))
	if err != nil {
		return nil, errors.Wrapf(err, "failed to request state by slot from api, slot=%d", slot)
	}

	// ConfigFork is used to unmarshal the BeaconState so we can read the block root in latest_block_header
	vu, err := detect.FromState(sb)
	if err != nil {
		return nil, errors.Wrap(err, "error detecting chain config for beacon state")
	}
	log.Printf("detected supported config in checkpoint state, name=%s, fork=%s", vu.Config.ConfigName, version.String(vu.Fork))

	s, err := vu.UnmarshalBeaconState(sb)
	if err != nil {
		return nil, errors.Wrap(err, "error using detected config fork to unmarshal state bytes")
	}

	// compute state and block roots
	sr, err := s.HashTreeRoot(ctx)
	if err != nil {
		return nil, errors.Wrap(err, "error computing hash_tree_root of state")
	}

	h := s.LatestBlockHeader()
	h.StateRoot = sr[:]
	br, err := h.HashTreeRoot()
	if err != nil {
		return nil, errors.Wrap(err, "error while computing block root using state data")
	}

	bb, err := client.GetBlock(ctx, IdFromRoot(br))
	if err != nil {
		return nil, errors.Wrapf(err, "error requesting block by root = %d", br)
	}
	b, err := vu.UnmarshalBeaconBlock(bb)
	if err != nil {
		return nil, errors.Wrap(err, "unable to unmarshal block to a supported type using the detected fork schedule")
	}
	br, err = b.Block().HashTreeRoot()
	if err != nil {
		return nil, errors.Wrap(err, "error computing hash_tree_root for block obtained via root")
	}

	return &WeakSubjectivityData{
		Epoch:     epoch,
		BlockRoot: br,
		StateRoot: sr,
	}, nil
}

// this method downloads the head state, which can be used to find the correct chain config
// and use prysm's helper methods to compute the latest weak subjectivity epoch.
func getWeakSubjectivityEpochFromHead(ctx context.Context, client *Client) (primitives.Epoch, error) {
	headBytes, err := client.GetState(ctx, IdHead)
	if err != nil {
		return 0, err
	}
	vu, err := detect.FromState(headBytes)
	if err != nil {
		return 0, errors.Wrap(err, "error detecting chain config for beacon state")
	}
	log.Printf("detected supported config in remote head state, name=%s, fork=%s", vu.Config.ConfigName, version.String(vu.Fork))
	headState, err := vu.UnmarshalBeaconState(headBytes)
	if err != nil {
		return 0, errors.Wrap(err, "error unmarshaling state to correct version")
	}

	epoch, err := helpers.LatestWeakSubjectivityEpoch(ctx, headState, vu.Config)
	if err != nil {
		return 0, errors.Wrap(err, "error computing the weak subjectivity epoch from head state")
	}

	log.Printf("(computed client-side) weak subjectivity epoch = %d", epoch)
	return epoch, nil
}

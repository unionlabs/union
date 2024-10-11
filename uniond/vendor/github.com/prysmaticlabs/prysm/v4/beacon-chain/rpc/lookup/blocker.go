package lookup

import (
	"context"
	"fmt"
	"strconv"
	"strings"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/filesystem"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/core"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	log "github.com/sirupsen/logrus"
)

// BlockIdParseError represents an error scenario where a block ID could not be parsed.
type BlockIdParseError struct {
	message string
}

// NewBlockIdParseError creates a new error instance.
func NewBlockIdParseError(reason error) BlockIdParseError {
	return BlockIdParseError{
		message: errors.Wrapf(reason, "could not parse block ID").Error(),
	}
}

// Error returns the underlying error message.
func (e BlockIdParseError) Error() string {
	return e.message
}

// Blocker is responsible for retrieving blocks.
type Blocker interface {
	Block(ctx context.Context, id []byte) (interfaces.ReadOnlySignedBeaconBlock, error)
	Blobs(ctx context.Context, id string, indices []uint64) ([]*blocks.VerifiedROBlob, *core.RpcError)
}

// BeaconDbBlocker is an implementation of Blocker. It retrieves blocks from the beacon chain database.
type BeaconDbBlocker struct {
	BeaconDB         db.ReadOnlyDatabase
	ChainInfoFetcher blockchain.ChainInfoFetcher
	BlobStorage      *filesystem.BlobStorage
}

// Block returns the beacon block for a given identifier. The identifier can be one of:
//   - "head" (canonical head in node's view)
//   - "genesis"
//   - "finalized"
//   - "justified"
//   - <slot>
//   - <hex encoded block root with '0x' prefix>
//   - <block root>
func (p *BeaconDbBlocker) Block(ctx context.Context, id []byte) (interfaces.ReadOnlySignedBeaconBlock, error) {
	var err error
	var blk interfaces.ReadOnlySignedBeaconBlock
	switch string(id) {
	case "head":
		blk, err = p.ChainInfoFetcher.HeadBlock(ctx)
		if err != nil {
			return nil, errors.Wrap(err, "could not retrieve head block")
		}
	case "finalized":
		finalized := p.ChainInfoFetcher.FinalizedCheckpt()
		finalizedRoot := bytesutil.ToBytes32(finalized.Root)
		blk, err = p.BeaconDB.Block(ctx, finalizedRoot)
		if err != nil {
			return nil, errors.New("could not get finalized block from db")
		}
	case "genesis":
		blk, err = p.BeaconDB.GenesisBlock(ctx)
		if err != nil {
			return nil, errors.Wrap(err, "could not retrieve genesis block")
		}
	default:
		stringId := strings.ToLower(string(id))
		if len(stringId) >= 2 && stringId[:2] == "0x" {
			decoded, err := hexutil.Decode(string(id))
			if err != nil {
				e := NewBlockIdParseError(err)
				return nil, &e
			}
			blk, err = p.BeaconDB.Block(ctx, bytesutil.ToBytes32(decoded))
			if err != nil {
				return nil, errors.Wrap(err, "could not retrieve block")
			}
		} else if len(id) == 32 {
			blk, err = p.BeaconDB.Block(ctx, bytesutil.ToBytes32(id))
			if err != nil {
				return nil, errors.Wrap(err, "could not retrieve block")
			}
		} else {
			slot, err := strconv.ParseUint(string(id), 10, 64)
			if err != nil {
				e := NewBlockIdParseError(err)
				return nil, &e
			}
			blks, err := p.BeaconDB.BlocksBySlot(ctx, primitives.Slot(slot))
			if err != nil {
				return nil, errors.Wrapf(err, "could not retrieve blocks for slot %d", slot)
			}
			_, roots, err := p.BeaconDB.BlockRootsBySlot(ctx, primitives.Slot(slot))
			if err != nil {
				return nil, errors.Wrapf(err, "could not retrieve block roots for slot %d", slot)
			}
			numBlks := len(blks)
			if numBlks == 0 {
				return nil, nil
			}
			for i, b := range blks {
				canonical, err := p.ChainInfoFetcher.IsCanonical(ctx, roots[i])
				if err != nil {
					return nil, errors.Wrapf(err, "could not determine if block root is canonical")
				}
				if canonical {
					blk = b
					break
				}
			}
		}
	}
	return blk, nil
}

// Blobs returns the blobs for a given block id identifier and blob indices. The identifier can be one of:
//   - "head" (canonical head in node's view)
//   - "genesis"
//   - "finalized"
//   - "justified"
//   - <slot>
//   - <hex encoded block root with '0x' prefix>
//   - <block root>
func (p *BeaconDbBlocker) Blobs(ctx context.Context, id string, indices []uint64) ([]*blocks.VerifiedROBlob, *core.RpcError) {
	var root []byte
	switch id {
	case "genesis":
		return nil, &core.RpcError{Err: errors.New("blobs are not supported for Phase 0 fork"), Reason: core.BadRequest}
	case "head":
		var err error
		root, err = p.ChainInfoFetcher.HeadRoot(ctx)
		if err != nil {
			return nil, &core.RpcError{Err: errors.Wrapf(err, "could not retrieve head root"), Reason: core.Internal}
		}
	case "finalized":
		fcp := p.ChainInfoFetcher.FinalizedCheckpt()
		if fcp == nil {
			return nil, &core.RpcError{Err: errors.New("received nil finalized checkpoint"), Reason: core.Internal}
		}
		root = fcp.Root
	case "justified":
		jcp := p.ChainInfoFetcher.CurrentJustifiedCheckpt()
		if jcp == nil {
			return nil, &core.RpcError{Err: errors.New("received nil justified checkpoint"), Reason: core.Internal}
		}
		root = jcp.Root
	default:
		if bytesutil.IsHex([]byte(id)) {
			var err error
			root, err = hexutil.Decode(id)
			if len(root) != fieldparams.RootLength {
				return nil, &core.RpcError{Err: fmt.Errorf("invalid block root of length %d", len(root)), Reason: core.BadRequest}
			}
			if err != nil {
				return nil, &core.RpcError{Err: NewBlockIdParseError(err), Reason: core.BadRequest}
			}
		} else {
			slot, err := strconv.ParseUint(id, 10, 64)
			if err != nil {
				return nil, &core.RpcError{Err: NewBlockIdParseError(err), Reason: core.BadRequest}
			}
			denebStart, err := slots.EpochStart(params.BeaconConfig().DenebForkEpoch)
			if err != nil {
				return nil, &core.RpcError{Err: errors.Wrap(err, "could not calculate Deneb start slot"), Reason: core.Internal}
			}
			if primitives.Slot(slot) < denebStart {
				return nil, &core.RpcError{Err: errors.New("blobs are not supported before Deneb fork"), Reason: core.BadRequest}
			}
			ok, roots, err := p.BeaconDB.BlockRootsBySlot(ctx, primitives.Slot(slot))
			if !ok {
				return nil, &core.RpcError{Err: fmt.Errorf("block not found: no block roots at slot %d", slot), Reason: core.NotFound}
			}
			if err != nil {
				return nil, &core.RpcError{Err: errors.Wrap(err, "failed to get block roots by slot"), Reason: core.Internal}
			}
			root = roots[0][:]
			if len(roots) == 1 {
				break
			}
			for _, blockRoot := range roots {
				canonical, err := p.ChainInfoFetcher.IsCanonical(ctx, blockRoot)
				if err != nil {
					return nil, &core.RpcError{Err: errors.Wrap(err, "could not determine if block root is canonical"), Reason: core.Internal}
				}
				if canonical {
					root = blockRoot[:]
					break
				}
			}
		}
	}

	if len(indices) == 0 {
		m, err := p.BlobStorage.Indices(bytesutil.ToBytes32(root))
		if err != nil {
			log.WithFields(log.Fields{
				"block root": hexutil.Encode(root),
			}).Error(errors.Wrapf(err, "could not retrieve blob indices for root %#x", root))
			return nil, &core.RpcError{Err: fmt.Errorf("could not retrieve blob indices for root %#x", root), Reason: core.Internal}
		}
		for k, v := range m {
			if v {
				indices = append(indices, uint64(k))
			}
		}
	}
	// returns empty slice if there are no indices
	blobs := make([]*blocks.VerifiedROBlob, len(indices))
	for i, index := range indices {
		vblob, err := p.BlobStorage.Get(bytesutil.ToBytes32(root), index)
		if err != nil {
			log.WithFields(log.Fields{
				"block root": hexutil.Encode(root),
				"blob index": index,
			}).Error(errors.Wrapf(err, "could not retrieve blob for block root %#x at index %d", root, index))
			return nil, &core.RpcError{Err: fmt.Errorf("could not retrieve blob for block root %#x at index %d", root, index), Reason: core.Internal}
		}
		blobs[i] = &vblob
	}
	return blobs, nil
}

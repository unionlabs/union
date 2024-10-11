package execution

import (
	"context"
	"fmt"
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/execution/types"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	"go.opencensus.io/trace"
)

// searchThreshold to apply for when searching for blocks of a particular time. If the buffer
// is exceeded we recalibrate the search again.
const searchThreshold = 5

// amount of times we repeat a failed search till is satisfies the conditional.
const repeatedSearches = 2 * searchThreshold

var errBlockTimeTooLate = errors.New("provided time is later than the current eth1 head")

// BlockExists returns true if the block exists, its height and any possible error encountered.
func (s *Service) BlockExists(ctx context.Context, hash common.Hash) (bool, *big.Int, error) {
	ctx, span := trace.StartSpan(ctx, "powchain.BlockExists")
	defer span.End()

	if exists, hdrInfo, err := s.headerCache.HeaderInfoByHash(hash); exists || err != nil {
		if err != nil {
			return false, nil, err
		}
		span.AddAttributes(trace.BoolAttribute("blockCacheHit", true))
		return true, hdrInfo.Number, nil
	}
	span.AddAttributes(trace.BoolAttribute("blockCacheHit", false))
	header, err := s.HeaderByHash(ctx, hash)
	if err != nil {
		return false, big.NewInt(0), errors.Wrap(err, "could not query block with given hash")
	}

	if err := s.headerCache.AddHeader(header); err != nil {
		return false, big.NewInt(0), err
	}

	return true, new(big.Int).Set(header.Number), nil
}

// BlockHashByHeight returns the block hash of the block at the given height.
func (s *Service) BlockHashByHeight(ctx context.Context, height *big.Int) (common.Hash, error) {
	ctx, span := trace.StartSpan(ctx, "powchain.BlockHashByHeight")
	defer span.End()

	if exists, hInfo, err := s.headerCache.HeaderInfoByHeight(height); exists || err != nil {
		if err != nil {
			return [32]byte{}, err
		}
		span.AddAttributes(trace.BoolAttribute("headerCacheHit", true))
		return hInfo.Hash, nil
	}
	span.AddAttributes(trace.BoolAttribute("headerCacheHit", false))

	if s.rpcClient == nil {
		err := errors.New("nil rpc client")
		tracing.AnnotateError(span, err)
		return [32]byte{}, err
	}

	header, err := s.HeaderByNumber(ctx, height)
	if err != nil {
		return [32]byte{}, errors.Wrap(err, fmt.Sprintf("could not query header with height %d", height.Uint64()))
	}
	if err := s.headerCache.AddHeader(header); err != nil {
		return [32]byte{}, err
	}
	return header.Hash, nil
}

// BlockTimeByHeight fetches an eth1 block timestamp by its height.
func (s *Service) BlockTimeByHeight(ctx context.Context, height *big.Int) (uint64, error) {
	ctx, span := trace.StartSpan(ctx, "powchain.BlockTimeByHeight")
	defer span.End()
	if s.rpcClient == nil {
		err := errors.New("nil rpc client")
		tracing.AnnotateError(span, err)
		return 0, err
	}

	header, err := s.HeaderByNumber(ctx, height)
	if err != nil {
		return 0, errors.Wrap(err, fmt.Sprintf("could not query block with height %d", height.Uint64()))
	}
	return header.Time, nil
}

// BlockByTimestamp returns the most recent block number up to a given timestamp.
// This is an optimized version with the worst case being O(2*repeatedSearches) number of calls
// while in best case search for the block is performed in O(1).
func (s *Service) BlockByTimestamp(ctx context.Context, time uint64) (*types.HeaderInfo, error) {
	ctx, span := trace.StartSpan(ctx, "powchain.BlockByTimestamp")
	defer span.End()

	s.latestEth1DataLock.RLock()
	latestBlkHeight := s.latestEth1Data.BlockHeight
	latestBlkTime := s.latestEth1Data.BlockTime
	s.latestEth1DataLock.RUnlock()

	if time > latestBlkTime {
		return nil, errors.Wrap(errBlockTimeTooLate, fmt.Sprintf("(%d > %d)", time, latestBlkTime))
	}
	// Initialize a pointer to eth1 chain's history to start our search from.
	cursorNum := big.NewInt(0).SetUint64(latestBlkHeight)
	cursorTime := latestBlkTime

	var numOfBlocks uint64
	estimatedBlk := cursorNum.Uint64()
	maxTimeBuffer := searchThreshold * params.BeaconConfig().SecondsPerETH1Block
	// Terminate if we can't find an acceptable block after
	// repeated searches.
	for i := 0; i < repeatedSearches; i++ {
		if ctx.Err() != nil {
			return nil, ctx.Err()
		}
		if time > cursorTime+maxTimeBuffer {
			numOfBlocks = (time - cursorTime) / params.BeaconConfig().SecondsPerETH1Block
			// In the event we have an infeasible estimated block, this is a defensive
			// check to ensure it does not exceed rational bounds.
			if cursorNum.Uint64()+numOfBlocks > latestBlkHeight {
				break
			}
			estimatedBlk = cursorNum.Uint64() + numOfBlocks
		} else if time+maxTimeBuffer < cursorTime {
			numOfBlocks = (cursorTime - time) / params.BeaconConfig().SecondsPerETH1Block
			// In the event we have an infeasible number of blocks
			// we exit early.
			if numOfBlocks >= cursorNum.Uint64() {
				break
			}
			estimatedBlk = cursorNum.Uint64() - numOfBlocks
		} else {
			// Exit if we are in the range of
			// time - buffer <= head.time <= time + buffer
			break
		}
		hInfo, err := s.retrieveHeaderInfo(ctx, estimatedBlk)
		if err != nil {
			return nil, err
		}
		cursorNum = hInfo.Number
		cursorTime = hInfo.Time
	}

	// Exit early if we get the desired block.
	if cursorTime == time {
		return s.retrieveHeaderInfo(ctx, cursorNum.Uint64())
	}
	if cursorTime > time {
		return s.findMaxTargetEth1Block(ctx, big.NewInt(0).SetUint64(estimatedBlk), time)
	}
	return s.findMinTargetEth1Block(ctx, big.NewInt(0).SetUint64(estimatedBlk), time)
}

// Performs a search to find a target eth1 block which is earlier than or equal to the
// target time. This method is used when head.time > targetTime
func (s *Service) findMaxTargetEth1Block(ctx context.Context, upperBoundBlk *big.Int, targetTime uint64) (*types.HeaderInfo, error) {
	for bn := upperBoundBlk; ; bn = big.NewInt(0).Sub(bn, big.NewInt(1)) {
		if ctx.Err() != nil {
			return nil, ctx.Err()
		}
		info, err := s.retrieveHeaderInfo(ctx, bn.Uint64())
		if err != nil {
			return nil, err
		}
		if info.Time <= targetTime {
			return info, nil
		}
	}
}

// Performs a search to find a target eth1 block which is just earlier than or equal to the
// target time. This method is used when head.time < targetTime
func (s *Service) findMinTargetEth1Block(ctx context.Context, lowerBoundBlk *big.Int, targetTime uint64) (*types.HeaderInfo, error) {
	for bn := lowerBoundBlk; ; bn = big.NewInt(0).Add(bn, big.NewInt(1)) {
		if ctx.Err() != nil {
			return nil, ctx.Err()
		}
		info, err := s.retrieveHeaderInfo(ctx, bn.Uint64())
		if err != nil {
			return nil, err
		}
		// Return the last block before we hit the threshold time.
		if info.Time > targetTime {
			return s.retrieveHeaderInfo(ctx, info.Number.Uint64()-1)
		}
		// If time is equal, this is our target block.
		if info.Time == targetTime {
			return info, nil
		}
	}
}

func (s *Service) retrieveHeaderInfo(ctx context.Context, bNum uint64) (*types.HeaderInfo, error) {
	bn := big.NewInt(0).SetUint64(bNum)
	exists, info, err := s.headerCache.HeaderInfoByHeight(bn)
	if err != nil {
		return nil, err
	}
	if !exists {
		hdr, err := s.HeaderByNumber(ctx, bn)
		if err != nil {
			return nil, err
		}
		if hdr == nil {
			return nil, errors.Errorf("header with the number %d does not exist", bNum)
		}
		if err := s.headerCache.AddHeader(hdr); err != nil {
			return nil, err
		}
		info = hdr
	}
	return info, nil
}

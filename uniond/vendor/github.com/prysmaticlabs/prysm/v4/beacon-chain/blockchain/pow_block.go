package blockchain

import (
	"bytes"
	"context"
	"fmt"
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/holiman/uint256"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/sirupsen/logrus"
)

// validateMergeBlock validates terminal block hash in the event of manual overrides before checking for total difficulty.
//
// def validate_merge_block(block: ReadOnlyBeaconBlock) -> None:
//
//	if TERMINAL_BLOCK_HASH != Hash32():
//	    # If `TERMINAL_BLOCK_HASH` is used as an override, the activation epoch must be reached.
//	    assert compute_epoch_at_slot(block.slot) >= TERMINAL_BLOCK_HASH_ACTIVATION_EPOCH
//	    assert block.body.execution_payload.parent_hash == TERMINAL_BLOCK_HASH
//	    return
//
//	pow_block = get_pow_block(block.body.execution_payload.parent_hash)
//	# Check if `pow_block` is available
//	assert pow_block is not None
//	pow_parent = get_pow_block(pow_block.parent_hash)
//	# Check if `pow_parent` is available
//	assert pow_parent is not None
//	# Check if `pow_block` is a valid terminal PoW block
//	assert is_valid_terminal_pow_block(pow_block, pow_parent)
func (s *Service) validateMergeBlock(ctx context.Context, b interfaces.ReadOnlySignedBeaconBlock) error {
	if err := blocks.BeaconBlockIsNil(b); err != nil {
		return err
	}
	payload, err := b.Block().Body().Execution()
	if err != nil {
		return err
	}
	if payload.IsNil() {
		return errors.New("nil execution payload")
	}
	ok, err := canUseValidatedTerminalBlockHash(b.Block().Slot(), payload)
	if err != nil {
		return errors.Wrap(err, "could not validate terminal block hash")
	}
	if ok {
		return nil
	}
	mergeBlockParentHash, mergeBlockTD, err := s.getBlkParentHashAndTD(ctx, payload.ParentHash())
	if err != nil {
		return errors.Wrap(err, "could not get merge block parent hash and total difficulty")
	}
	_, mergeBlockParentTD, err := s.getBlkParentHashAndTD(ctx, mergeBlockParentHash)
	if err != nil {
		return errors.Wrap(err, "could not get merge parent block total difficulty")
	}
	valid, err := validateTerminalBlockDifficulties(mergeBlockTD, mergeBlockParentTD)
	if err != nil {
		return err
	}
	if !valid {
		err := fmt.Errorf("invalid TTD, configTTD: %s, currentTTD: %s, parentTTD: %s",
			params.BeaconConfig().TerminalTotalDifficulty, mergeBlockTD, mergeBlockParentTD)
		return invalidBlock{error: err}
	}

	log.WithFields(logrus.Fields{
		"slot":                            b.Block().Slot(),
		"mergeBlockHash":                  common.BytesToHash(payload.ParentHash()).String(),
		"mergeBlockParentHash":            common.BytesToHash(mergeBlockParentHash).String(),
		"terminalTotalDifficulty":         params.BeaconConfig().TerminalTotalDifficulty,
		"mergeBlockTotalDifficulty":       mergeBlockTD,
		"mergeBlockParentTotalDifficulty": mergeBlockParentTD,
	}).Info("Validated terminal block")

	log.Info(mergeAsciiArt)

	return nil
}

// getBlkParentHashAndTD retrieves the parent hash and total difficulty of the given block.
func (s *Service) getBlkParentHashAndTD(ctx context.Context, blkHash []byte) ([]byte, *uint256.Int, error) {
	blk, err := s.cfg.ExecutionEngineCaller.ExecutionBlockByHash(ctx, common.BytesToHash(blkHash), false /* no txs */)
	if err != nil {
		return nil, nil, errors.Wrap(err, "could not get pow block")
	}
	if blk == nil {
		return nil, nil, errors.New("pow block is nil")
	}
	blk.Version = version.Bellatrix
	blkTDBig, err := hexutil.DecodeBig(blk.TotalDifficulty)
	if err != nil {
		return nil, nil, errors.Wrap(err, "could not decode merge block total difficulty")
	}
	blkTDUint256, overflows := uint256.FromBig(blkTDBig)
	if overflows {
		return nil, nil, errors.New("total difficulty overflows")
	}
	return blk.ParentHash[:], blkTDUint256, nil
}

// canUseValidatedTerminalBlockHash validates if the merge block is a valid terminal PoW block.
// spec code:
// if TERMINAL_BLOCK_HASH != Hash32():
//
//	# If `TERMINAL_BLOCK_HASH` is used as an override, the activation epoch must be reached.
//	assert compute_epoch_at_slot(block.slot) >= TERMINAL_BLOCK_HASH_ACTIVATION_EPOCH
//	assert block.body.execution_payload.parent_hash == TERMINAL_BLOCK_HASH
//	return
func canUseValidatedTerminalBlockHash(blkSlot primitives.Slot, payload interfaces.ExecutionData) (bool, error) {
	if bytesutil.ToBytes32(params.BeaconConfig().TerminalBlockHash.Bytes()) == [32]byte{} {
		return false, nil
	}
	if params.BeaconConfig().TerminalBlockHashActivationEpoch > slots.ToEpoch(blkSlot) {
		return false, errors.New("terminal block hash activation epoch not reached")
	}
	if !bytes.Equal(payload.ParentHash(), params.BeaconConfig().TerminalBlockHash.Bytes()) {
		return false, errors.New("parent hash does not match terminal block hash")
	}
	return true, nil
}

// validateTerminalBlockDifficulties validates terminal pow block by comparing own total difficulty with parent's total difficulty.
//
// def is_valid_terminal_pow_block(block: PowBlock, parent: PowBlock) -> bool:
//
//	is_total_difficulty_reached = block.total_difficulty >= TERMINAL_TOTAL_DIFFICULTY
//	is_parent_total_difficulty_valid = parent.total_difficulty < TERMINAL_TOTAL_DIFFICULTY
//	return is_total_difficulty_reached and is_parent_total_difficulty_valid
func validateTerminalBlockDifficulties(currentDifficulty *uint256.Int, parentDifficulty *uint256.Int) (bool, error) {
	b, ok := new(big.Int).SetString(params.BeaconConfig().TerminalTotalDifficulty, 10)
	if !ok {
		return false, errors.New("failed to parse terminal total difficulty")
	}
	ttd, of := uint256.FromBig(b)
	if of {
		return false, errors.New("overflow terminal total difficulty")
	}
	totalDifficultyReached := currentDifficulty.Cmp(ttd) >= 0
	parentTotalDifficultyValid := ttd.Cmp(parentDifficulty) > 0
	return totalDifficultyReached && parentTotalDifficultyValid, nil
}

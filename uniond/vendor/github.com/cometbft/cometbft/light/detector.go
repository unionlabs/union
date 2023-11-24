package light

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/cometbft/cometbft/light/provider"
	"github.com/cometbft/cometbft/types"
)

// The detector component of the light client detects and handles attacks on the light client.
// More info here:
// cometbft/docs/architecture/adr-047-handling-evidence-from-light-client.md

// detectDivergence is a second wall of defense for the light client.
//
// It takes the target verified header and compares it with the headers of a set of
// witness providers that the light client is connected to. If a conflicting header
// is returned it verifies and examines the conflicting header against the verified
// trace that was produced from the primary. If successful, it produces two sets of evidence
// and sends them to the opposite provider before halting.
//
// If there are no conflicting headers, the light client deems the verified target header
// trusted and saves it to the trusted store.
func (c *Client) detectDivergence(ctx context.Context, primaryTrace []*types.LightBlock, now time.Time) error {
	if primaryTrace == nil || len(primaryTrace) < 2 {
		return errors.New("nil or single block primary trace")
	}
	var (
		headerMatched      bool
		lastVerifiedHeader = primaryTrace[len(primaryTrace)-1].SignedHeader
		witnessesToRemove  = make([]int, 0)
	)
	c.logger.Debug("Running detector against trace", "finalizeBlockHeight", lastVerifiedHeader.Height,
		"endBlockHash", lastVerifiedHeader.Hash, "length", len(primaryTrace))

	c.providerMutex.Lock()
	defer c.providerMutex.Unlock()

	if len(c.witnesses) == 0 {
		return ErrNoWitnesses
	}

	// launch one goroutine per witness to retrieve the light block of the target height
	// and compare it with the header from the primary
	errc := make(chan error, len(c.witnesses))
	for i, witness := range c.witnesses {
		go c.compareNewHeaderWithWitness(ctx, errc, lastVerifiedHeader, witness, i)
	}

	// handle errors from the header comparisons as they come in
	for i := 0; i < cap(errc); i++ {
		err := <-errc

		switch e := err.(type) {
		case nil: // at least one header matched
			headerMatched = true
		case errConflictingHeaders:
			// We have conflicting headers. This could possibly imply an attack on the light client.
			// First we need to verify the witness's header using the same skipping verification and then we
			// need to find the point that the headers diverge and examine this for any evidence of an attack.
			//
			// We combine these actions together, verifying the witnesses headers and outputting the trace
			// which captures the bifurcation point and if successful provides the information to create valid evidence.
			err := c.handleConflictingHeaders(ctx, primaryTrace, e.Block, e.WitnessIndex, now)
			if err != nil {
				// return information of the attack
				return err
			}
			// if attempt to generate conflicting headers failed then remove witness
			witnessesToRemove = append(witnessesToRemove, e.WitnessIndex)

		case errBadWitness:
			// these are all malevolent errors and should result in removing the
			// witness
			c.logger.Info("witness returned an error during header comparison, removing...",
				"witness", c.witnesses[e.WitnessIndex], "err", err)
			witnessesToRemove = append(witnessesToRemove, e.WitnessIndex)
		default:
			// Benign errors which can be ignored unless there was a context
			// canceled
			if errors.Is(e, context.Canceled) || errors.Is(e, context.DeadlineExceeded) {
				return e
			}
			c.logger.Info("error in light block request to witness", "err", err)
		}
	}

	// remove witnesses that have misbehaved
	if err := c.removeWitnesses(witnessesToRemove); err != nil {
		return err
	}

	// 1. If we had at least one witness that returned the same header then we
	// conclude that we can trust the header
	if headerMatched {
		return nil
	}

	// 2. Else all witnesses have either not responded, don't have the block or sent invalid blocks.
	return ErrFailedHeaderCrossReferencing
}

// compareNewHeaderWithWitness takes the verified header from the primary and compares it with a
// header from a specified witness. The function can return one of three errors:
//
// 1: errConflictingHeaders -> there may have been an attack on this light client
// 2: errBadWitness -> the witness has either not responded, doesn't have the header or has given us an invalid one
//
//	Note: In the case of an invalid header we remove the witness
//
// 3: nil -> the hashes of the two headers match
func (c *Client) compareNewHeaderWithWitness(ctx context.Context, errc chan error, h *types.SignedHeader,
	witness provider.Provider, witnessIndex int) {

	lightBlock, err := witness.LightBlock(ctx, h.Height)
	switch err {
	// no error means we move on to checking the hash of the two headers
	case nil:
		break

	// the witness hasn't been helpful in comparing headers, we mark the response and continue
	// comparing with the rest of the witnesses
	case provider.ErrNoResponse, provider.ErrLightBlockNotFound, context.DeadlineExceeded, context.Canceled:
		errc <- err
		return

	// the witness' head of the blockchain is lower than the height of the primary. This could be one of
	// two things:
	//    1) The witness is lagging behind
	//    2) The primary may be performing a lunatic attack with a height and time in the future
	case provider.ErrHeightTooHigh:
		// The light client now asks for the latest header that the witness has
		var isTargetHeight bool
		isTargetHeight, lightBlock, err = c.getTargetBlockOrLatest(ctx, h.Height, witness)
		if err != nil {
			errc <- err
			return
		}

		// if the witness caught up and has returned a block of the target height then we can
		// break from this switch case and continue to verify the hashes
		if isTargetHeight {
			break
		}

		// witness' last header is below the primary's header. We check the times to see if the blocks
		// have conflicting times
		if !lightBlock.Time.Before(h.Time) {
			errc <- errConflictingHeaders{Block: lightBlock, WitnessIndex: witnessIndex}
			return
		}

		// the witness is behind. We wait for a period WAITING = 2 * DRIFT + LAG.
		// This should give the witness ample time if it is a participating member
		// of consensus to produce a block that has a time that is after the primary's
		// block time. If not the witness is too far behind and the light client removes it
		time.Sleep(2*c.maxClockDrift + c.maxBlockLag)
		isTargetHeight, lightBlock, err = c.getTargetBlockOrLatest(ctx, h.Height, witness)
		if err != nil {
			if errors.Is(err, context.Canceled) || errors.Is(err, context.DeadlineExceeded) {
				errc <- err
			} else {
				errc <- errBadWitness{Reason: err, WitnessIndex: witnessIndex}
			}
			return
		}
		if isTargetHeight {
			break
		}

		// the witness still doesn't have a block at the height of the primary.
		// Check if there is a conflicting time
		if !lightBlock.Time.Before(h.Time) {
			errc <- errConflictingHeaders{Block: lightBlock, WitnessIndex: witnessIndex}
			return
		}

		// Following this request response procedure, the witness has been unable to produce a block
		// that can somehow conflict with the primary's block. We thus conclude that the witness
		// is too far behind and thus we return a no response error.
		//
		// NOTE: If the clock drift / lag has been miscalibrated it is feasible that the light client has
		// drifted too far ahead for any witness to be able provide a comparable block and thus may allow
		// for a malicious primary to attack it
		errc <- provider.ErrNoResponse
		return

	default:
		// all other errors (i.e. invalid block, closed connection or unreliable provider) we mark the
		// witness as bad and remove it
		errc <- errBadWitness{Reason: err, WitnessIndex: witnessIndex}
		return
	}

	if !bytes.Equal(h.Hash(), lightBlock.Hash()) {
		errc <- errConflictingHeaders{Block: lightBlock, WitnessIndex: witnessIndex}
	}

	c.logger.Debug("Matching header received by witness", "height", h.Height, "witness", witnessIndex)
	errc <- nil
}

// sendEvidence sends evidence to a provider on a best effort basis.
func (c *Client) sendEvidence(ctx context.Context, ev *types.LightClientAttackEvidence, receiver provider.Provider) {
	err := receiver.ReportEvidence(ctx, ev)
	if err != nil {
		c.logger.Error("Failed to report evidence to provider", "ev", ev, "provider", receiver)
	}
}

// handleConflictingHeaders handles the primary style of attack, which is where a primary and witness have
// two headers of the same height but with different hashes
func (c *Client) handleConflictingHeaders(
	ctx context.Context,
	primaryTrace []*types.LightBlock,
	challengingBlock *types.LightBlock,
	witnessIndex int,
	now time.Time,
) error {
	supportingWitness := c.witnesses[witnessIndex]
	witnessTrace, primaryBlock, err := c.examineConflictingHeaderAgainstTrace(
		ctx,
		primaryTrace,
		challengingBlock,
		supportingWitness,
		now,
	)
	if err != nil {
		c.logger.Info("error validating witness's divergent header", "witness", supportingWitness, "err", err)
		return nil
	}

	// We are suspecting that the primary is faulty, hence we hold the witness as the source of truth
	// and generate evidence against the primary that we can send to the witness
	commonBlock, trustedBlock := witnessTrace[0], witnessTrace[len(witnessTrace)-1]
	evidenceAgainstPrimary := newLightClientAttackEvidence(primaryBlock, trustedBlock, commonBlock)
	c.logger.Error("ATTEMPTED ATTACK DETECTED. Sending evidence againt primary by witness", "ev", evidenceAgainstPrimary,
		"primary", c.primary, "witness", supportingWitness)
	c.sendEvidence(ctx, evidenceAgainstPrimary, supportingWitness)

	if primaryBlock.Commit.Round != witnessTrace[len(witnessTrace)-1].Commit.Round {
		c.logger.Info("The light client has detected, and prevented, an attempted amnesia attack." +
			" We think this attack is pretty unlikely, so if you see it, that's interesting to us." +
			" Can you let us know by opening an issue through https://github.com/tendermint/tendermint/issues/new?")
	}

	// This may not be valid because the witness itself is at fault. So now we reverse it, examining the
	// trace provided by the witness and holding the primary as the source of truth. Note: primary may not
	// respond but this is okay as we will halt anyway.
	primaryTrace, witnessBlock, err := c.examineConflictingHeaderAgainstTrace(
		ctx,
		witnessTrace,
		primaryBlock,
		c.primary,
		now,
	)
	if err != nil {
		c.logger.Info("Error validating primary's divergent header", "primary", c.primary, "err", err)
		return ErrLightClientAttack
	}

	// We now use the primary trace to create evidence against the witness and send it to the primary
	commonBlock, trustedBlock = primaryTrace[0], primaryTrace[len(primaryTrace)-1]
	evidenceAgainstWitness := newLightClientAttackEvidence(witnessBlock, trustedBlock, commonBlock)
	c.logger.Error("Sending evidence against witness by primary", "ev", evidenceAgainstWitness,
		"primary", c.primary, "witness", supportingWitness)
	c.sendEvidence(ctx, evidenceAgainstWitness, c.primary)
	// We return the error and don't process anymore witnesses
	return ErrLightClientAttack
}

// examineConflictingHeaderAgainstTrace takes a trace from one provider and a divergent header that
// it has received from another and preforms verifySkipping at the heights of each of the intermediate
// headers in the trace until it reaches the divergentHeader. 1 of 2 things can happen.
//
//  1. The light client verifies a header that is different to the intermediate header in the trace. This
//     is the bifurcation point and the light client can create evidence from it
//  2. The source stops responding, doesn't have the block or sends an invalid header in which case we
//     return the error and remove the witness
//
// CONTRACT:
//  1. Trace can not be empty len(trace) > 0
//  2. The last block in the trace can not be of a lower height than the target block
//     trace[len(trace)-1].Height >= targetBlock.Height
//  3. The
func (c *Client) examineConflictingHeaderAgainstTrace(
	ctx context.Context,
	trace []*types.LightBlock,
	targetBlock *types.LightBlock,
	source provider.Provider, now time.Time,
) ([]*types.LightBlock, *types.LightBlock, error) {

	var (
		previouslyVerifiedBlock, sourceBlock *types.LightBlock
		sourceTrace                          []*types.LightBlock
		err                                  error
	)

	if targetBlock.Height < trace[0].Height {
		return nil, nil, fmt.Errorf("target block has a height lower than the trusted height (%d < %d)",
			targetBlock.Height, trace[0].Height)
	}

	for idx, traceBlock := range trace {
		// this case only happens in a forward lunatic attack. We treat the block with the
		// height directly after the targetBlock as the divergent block
		if traceBlock.Height > targetBlock.Height {
			// sanity check that the time of the traceBlock is indeed less than that of the targetBlock. If the trace
			// was correctly verified we should expect monotonically increasing time. This means that if the block at
			// the end of the trace has a lesser time than the target block then all blocks in the trace should have a
			// lesser time
			if traceBlock.Time.After(targetBlock.Time) {
				return nil, nil,
					errors.New("sanity check failed: expected traceblock to have a lesser time than the target block")
			}

			// before sending back the divergent block and trace we need to ensure we have verified
			// the final gap between the previouslyVerifiedBlock and the targetBlock
			if previouslyVerifiedBlock.Height != targetBlock.Height {
				sourceTrace, err = c.verifySkipping(ctx, source, previouslyVerifiedBlock, targetBlock, now)
				if err != nil {
					return nil, nil, fmt.Errorf("verifySkipping of conflicting header failed: %w", err)
				}
			}
			return sourceTrace, traceBlock, nil
		}

		// get the corresponding block from the source to verify and match up against the traceBlock
		if traceBlock.Height == targetBlock.Height {
			sourceBlock = targetBlock
		} else {
			sourceBlock, err = source.LightBlock(ctx, traceBlock.Height)
			if err != nil {
				return nil, nil, fmt.Errorf("failed to examine trace: %w", err)
			}
		}

		// The first block in the trace MUST be the same to the light block that the source produces
		// else we cannot continue with verification.
		if idx == 0 {
			if shash, thash := sourceBlock.Hash(), traceBlock.Hash(); !bytes.Equal(shash, thash) {
				return nil, nil, fmt.Errorf("trusted block is different to the source's first block (%X = %X)",
					thash, shash)
			}
			previouslyVerifiedBlock = sourceBlock
			continue
		}

		// we check that the source provider can verify a block at the same height of the
		// intermediate height
		sourceTrace, err = c.verifySkipping(ctx, source, previouslyVerifiedBlock, sourceBlock, now)
		if err != nil {
			return nil, nil, fmt.Errorf("verifySkipping of conflicting header failed: %w", err)
		}
		// check if the headers verified by the source has diverged from the trace
		if shash, thash := sourceBlock.Hash(), traceBlock.Hash(); !bytes.Equal(shash, thash) {
			// Bifurcation point found!
			return sourceTrace, traceBlock, nil
		}

		// headers are still the same. update the previouslyVerifiedBlock
		previouslyVerifiedBlock = sourceBlock
	}

	// We have reached the end of the trace. This should never happen. This can only happen if one of the stated
	// prerequisites to this function were not met. Namely that either trace[len(trace)-1].Height < targetBlock.Height
	// or that trace[i].Hash() != targetBlock.Hash()
	return nil, nil, errNoDivergence

}

// getTargetBlockOrLatest gets the latest height, if it is greater than the target height then it queries
// the target height else it returns the latest. returns true if it successfully managed to acquire the target
// height.
func (c *Client) getTargetBlockOrLatest(
	ctx context.Context,
	height int64,
	witness provider.Provider,
) (bool, *types.LightBlock, error) {
	lightBlock, err := witness.LightBlock(ctx, 0)
	if err != nil {
		return false, nil, err
	}

	if lightBlock.Height == height {
		// the witness has caught up to the height of the provider's signed header. We
		// can resume with checking the hashes.
		return true, lightBlock, nil
	}

	if lightBlock.Height > height {
		// the witness has caught up. We recursively call the function again. However in order
		// to avoid a wild goose chase where the witness sends us one header below and one header
		// above the height we set a timeout to the context
		lightBlock, err := witness.LightBlock(ctx, height)
		return true, lightBlock, err
	}

	return false, lightBlock, nil
}

// newLightClientAttackEvidence determines the type of attack and then forms the evidence filling out
// all the fields such that it is ready to be sent to a full node.
func newLightClientAttackEvidence(conflicted, trusted, common *types.LightBlock) *types.LightClientAttackEvidence {
	ev := &types.LightClientAttackEvidence{ConflictingBlock: conflicted}
	// if this is an equivocation or amnesia attack, i.e. the validator sets are the same, then we
	// return the height of the conflicting block else if it is a lunatic attack and the validator sets
	// are not the same then we send the height of the common header.
	if ev.ConflictingHeaderIsInvalid(trusted.Header) {
		ev.CommonHeight = common.Height
		ev.Timestamp = common.Time
		ev.TotalVotingPower = common.ValidatorSet.TotalVotingPower()
	} else {
		ev.CommonHeight = trusted.Height
		ev.Timestamp = trusted.Time
		ev.TotalVotingPower = trusted.ValidatorSet.TotalVotingPower()
	}
	ev.ByzantineValidators = ev.GetByzantineValidators(common.ValidatorSet, trusted.SignedHeader)
	return ev
}

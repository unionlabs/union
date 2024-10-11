package keeper

import (
	"context"
	"errors"

	"cosmossdk.io/collections"
	"cosmossdk.io/math"
	v1 "cosmossdk.io/x/gov/types/v1"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// Tally iterates over the votes and updates the tally of a proposal based on the voting power of the voters
func (k Keeper) Tally(ctx context.Context, proposal v1.Proposal) (passes, burnDeposits bool, tallyResults v1.TallyResult, err error) {
	validators, err := k.getCurrentValidators(ctx)
	if err != nil {
		return false, false, v1.TallyResult{}, err
	}

	if k.config.CalculateVoteResultsAndVotingPowerFn == nil {
		k.config.CalculateVoteResultsAndVotingPowerFn = defaultCalculateVoteResultsAndVotingPower
	}

	totalVoterPower, results, err := k.config.CalculateVoteResultsAndVotingPowerFn(ctx, k, proposal.Id, validators)
	if err != nil {
		return false, false, v1.TallyResult{}, err
	}

	params, err := k.Params.Get(ctx)
	if err != nil {
		return false, false, v1.TallyResult{}, err
	}
	tallyResults = v1.NewTallyResultFromMap(results)

	// If there is no staked coins, the proposal fails
	totalBonded, err := k.sk.TotalBondedTokens(ctx)
	if err != nil {
		return false, false, v1.TallyResult{}, err
	}

	if totalBonded.IsZero() {
		return false, false, tallyResults, nil
	}

	// If there are more spam votes than the sum of all other options, proposal fails
	// A proposal with no votes should not be considered spam
	if !totalVoterPower.Equal(math.LegacyZeroDec()) &&
		results[v1.OptionSpam].GTE(results[v1.OptionOne].Add(results[v1.OptionTwo].Add(results[v1.OptionThree].Add(results[v1.OptionFour])))) {
		return false, true, tallyResults, nil
	}

	switch proposal.ProposalType {
	case v1.ProposalType_PROPOSAL_TYPE_OPTIMISTIC:
		return k.tallyOptimistic(totalVoterPower, totalBonded, results, params)
	case v1.ProposalType_PROPOSAL_TYPE_EXPEDITED:
		return k.tallyExpedited(totalVoterPower, totalBonded, results, params)
	case v1.ProposalType_PROPOSAL_TYPE_MULTIPLE_CHOICE:
		return k.tallyMultipleChoice(totalVoterPower, totalBonded, results, params)
	default:
		return k.tallyStandard(ctx, proposal, totalVoterPower, totalBonded, results, params)
	}
}

// tallyStandard tallies the votes of a standard proposal
// If there is not enough quorum of votes, the proposal fails
// If no one votes (everyone abstains), proposal fails
// If more than 1/3 of voters veto, proposal fails
// If more than 1/2 of non-abstaining voters vote Yes, proposal passes
// If more than 1/2 of non-abstaining voters vote No, proposal fails
// Checking for spam votes is done before calling this function
func (k Keeper) tallyStandard(ctx context.Context, proposal v1.Proposal, totalVoterPower math.LegacyDec, totalBonded math.Int, results map[v1.VoteOption]math.LegacyDec, params v1.Params) (passes, burnDeposits bool, tallyResults v1.TallyResult, err error) {
	tallyResults = v1.NewTallyResultFromMap(results)

	quorumStr := params.Quorum
	yesQuorumStr := params.YesQuorum
	thresholdStr := params.Threshold
	vetoThresholdStr := params.VetoThreshold

	if len(proposal.Messages) > 0 {
		// check if any of the message has message based params
		customMessageParams, err := k.MessageBasedParams.Get(ctx, sdk.MsgTypeURL(proposal.Messages[0]))
		if err != nil && !errors.Is(err, collections.ErrNotFound) {
			return false, false, tallyResults, err
		} else if err == nil {
			quorumStr = customMessageParams.GetQuorum()
			thresholdStr = customMessageParams.GetThreshold()
			vetoThresholdStr = customMessageParams.GetVetoThreshold()
			yesQuorumStr = customMessageParams.GetYesQuorum()
		}
	}

	// If there is not enough quorum of votes, the proposal fails
	percentVoting := totalVoterPower.Quo(math.LegacyNewDecFromInt(totalBonded))
	quorum, _ := math.LegacyNewDecFromStr(quorumStr)
	if percentVoting.LT(quorum) {
		return false, params.BurnVoteQuorum, tallyResults, nil
	}

	// If no one votes (everyone abstains), proposal fails
	if totalVoterPower.Equal(results[v1.OptionAbstain]) {
		return false, false, tallyResults, nil
	}

	// If yes quorum enabled and less than yes_quorum of voters vote Yes, proposal fails
	yesQuorum, _ := math.LegacyNewDecFromStr(yesQuorumStr)
	if yesQuorum.GT(math.LegacyZeroDec()) && results[v1.OptionYes].Quo(totalVoterPower).LT(yesQuorum) {
		return false, false, tallyResults, nil
	}

	// If more than 1/3 of voters veto, proposal fails
	vetoThreshold, _ := math.LegacyNewDecFromStr(vetoThresholdStr)
	if results[v1.OptionNoWithVeto].Quo(totalVoterPower).GT(vetoThreshold) {
		return false, params.BurnVoteVeto, tallyResults, nil
	}

	// If more than 1/2 of non-abstaining voters vote Yes, proposal passes
	threshold, _ := math.LegacyNewDecFromStr(thresholdStr)
	if results[v1.OptionYes].Quo(totalVoterPower.Sub(results[v1.OptionAbstain])).GT(threshold) {
		return true, false, tallyResults, nil
	}

	// If more than 1/2 of non-abstaining voters vote No, proposal fails
	return false, false, tallyResults, nil
}

// tallyExpedited tallies the votes of an expedited proposal
// If there is not enough expedited quorum of votes, the proposal fails
// If no one votes (everyone abstains), proposal fails
// If more than 1/3 of voters veto, proposal fails
// If more than 2/3 of non-abstaining voters vote Yes, proposal passes
// If more than 1/2 of non-abstaining voters vote No, proposal fails
// Checking for spam votes is done before calling this function
func (k Keeper) tallyExpedited(totalVoterPower math.LegacyDec, totalBonded math.Int, results map[v1.VoteOption]math.LegacyDec, params v1.Params) (passes, burnDeposits bool, tallyResults v1.TallyResult, err error) {
	tallyResults = v1.NewTallyResultFromMap(results)

	// If there is not enough quorum of votes, the proposal fails
	percentVoting := totalVoterPower.Quo(math.LegacyNewDecFromInt(totalBonded))
	expeditedQuorum, _ := math.LegacyNewDecFromStr(params.ExpeditedQuorum)
	if percentVoting.LT(expeditedQuorum) {
		return false, params.BurnVoteQuorum, tallyResults, nil
	}

	// If no one votes (everyone abstains), proposal fails
	if totalVoterPower.Equal(results[v1.OptionAbstain]) {
		return false, false, tallyResults, nil
	}

	// If yes quorum enabled and less than yes_quorum of voters vote Yes, proposal fails
	yesQuorum, _ := math.LegacyNewDecFromStr(params.YesQuorum)
	if yesQuorum.GT(math.LegacyZeroDec()) && results[v1.OptionYes].Quo(totalVoterPower).LT(yesQuorum) {
		return false, false, tallyResults, nil
	}

	// If more than 1/3 of voters veto, proposal fails
	vetoThreshold, _ := math.LegacyNewDecFromStr(params.VetoThreshold)
	if results[v1.OptionNoWithVeto].Quo(totalVoterPower).GT(vetoThreshold) {
		return false, params.BurnVoteVeto, tallyResults, nil
	}

	// If more than 2/3 of non-abstaining voters vote Yes, proposal passes
	threshold, _ := math.LegacyNewDecFromStr(params.GetExpeditedThreshold())
	if results[v1.OptionYes].Quo(totalVoterPower.Sub(results[v1.OptionAbstain])).GT(threshold) {
		return true, false, tallyResults, nil
	}

	// If more than 1/2 of non-abstaining voters vote No, proposal fails
	return false, false, tallyResults, nil
}

// tallyOptimistic tallies the votes of an optimistic proposal
// If proposal has no votes, proposal passes
// If the threshold of no is reached, proposal fails
// Any other case, proposal passes
// Checking for spam votes is done before calling this function
func (k Keeper) tallyOptimistic(totalVoterPower math.LegacyDec, totalBonded math.Int, results map[v1.VoteOption]math.LegacyDec, params v1.Params) (passes, burnDeposits bool, tallyResults v1.TallyResult, err error) {
	tallyResults = v1.NewTallyResultFromMap(results)
	optimisticNoThreshold, _ := math.LegacyNewDecFromStr(params.OptimisticRejectedThreshold)

	// If proposal has no votes, proposal passes
	if totalVoterPower.Equal(math.LegacyZeroDec()) {
		return true, false, tallyResults, nil
	}

	// If the threshold of no is reached, proposal fails
	if results[v1.OptionNo].Quo(totalBonded.ToLegacyDec()).GT(optimisticNoThreshold) {
		return false, false, tallyResults, nil
	}

	return true, false, tallyResults, nil
}

// tallyMultipleChoice tallies the votes of a multiple choice proposal
// If there is not enough quorum of votes, the proposal fails
// Any other case, proposal passes
// Checking for spam votes is done before calling this function
func (k Keeper) tallyMultipleChoice(totalVoterPower math.LegacyDec, totalBonded math.Int, results map[v1.VoteOption]math.LegacyDec, params v1.Params) (passes, burnDeposits bool, tallyResults v1.TallyResult, err error) {
	tallyResults = v1.NewTallyResultFromMap(results)

	// If there is not enough quorum of votes, the proposal fails
	percentVoting := totalVoterPower.Quo(math.LegacyNewDecFromInt(totalBonded))
	quorum, _ := math.LegacyNewDecFromStr(params.Quorum)
	if percentVoting.LT(quorum) {
		return false, params.BurnVoteQuorum, tallyResults, nil
	}

	// a multiple choice proposal always passes unless it was spam or quorum was not reached.
	return true, false, tallyResults, nil
}

// getCurrentValidators fetches all the bonded validators, insert them into currValidators
func (k Keeper) getCurrentValidators(ctx context.Context) (map[string]v1.ValidatorGovInfo, error) {
	currValidators := make(map[string]v1.ValidatorGovInfo)
	if err := k.sk.IterateBondedValidatorsByPower(ctx, func(index int64, validator sdk.ValidatorI) (stop bool) {
		valBz, err := k.sk.ValidatorAddressCodec().StringToBytes(validator.GetOperator())
		if err != nil {
			return false
		}
		currValidators[validator.GetOperator()] = v1.NewValidatorGovInfo(
			valBz,
			validator.GetBondedTokens(),
			validator.GetDelegatorShares(),
			math.LegacyZeroDec(),
			v1.WeightedVoteOptions{},
		)

		return false
	}); err != nil {
		return nil, err
	}

	return currValidators, nil
}

// calculateVoteResultsAndVotingPower iterate over all votes, tally up the voting power of each validator
// and returns the votes results from voters
func defaultCalculateVoteResultsAndVotingPower(
	ctx context.Context,
	k Keeper,
	proposalID uint64,
	validators map[string]v1.ValidatorGovInfo,
) (math.LegacyDec, map[v1.VoteOption]math.LegacyDec, error) {
	totalVP := math.LegacyZeroDec()
	results := createEmptyResults()

	// iterate over all votes, tally up the voting power of each validator
	rng := collections.NewPrefixedPairRange[uint64, sdk.AccAddress](proposalID)
	votesToRemove := []collections.Pair[uint64, sdk.AccAddress]{}
	if err := k.Votes.Walk(ctx, rng, func(key collections.Pair[uint64, sdk.AccAddress], vote v1.Vote) (bool, error) {
		// if validator, just record it in the map
		voter, err := k.authKeeper.AddressCodec().StringToBytes(vote.Voter)
		if err != nil {
			return false, err
		}

		valAddrStr, err := k.sk.ValidatorAddressCodec().BytesToString(voter)
		if err != nil {
			return false, err
		}

		if val, ok := validators[valAddrStr]; ok {
			val.Vote = vote.Options
			validators[valAddrStr] = val
		}

		// iterate over all delegations from voter, deduct from any delegated-to validators
		err = k.sk.IterateDelegations(ctx, voter, func(index int64, delegation sdk.DelegationI) (stop bool) {
			valAddrStr := delegation.GetValidatorAddr()

			if val, ok := validators[valAddrStr]; ok {
				// There is no need to handle the special case that validator address equal to voter address.
				// Because voter's voting power will tally again even if there will be deduction of voter's voting power from validator.
				val.DelegatorDeductions = val.DelegatorDeductions.Add(delegation.GetShares())
				validators[valAddrStr] = val

				// delegation shares * bonded / total shares
				votingPower := delegation.GetShares().MulInt(val.BondedTokens).Quo(val.DelegatorShares)

				for _, option := range vote.Options {
					weight, _ := math.LegacyNewDecFromStr(option.Weight)
					subPower := votingPower.Mul(weight)
					results[option.Option] = results[option.Option].Add(subPower)
				}

				totalVP = totalVP.Add(votingPower)
			}

			return false
		})
		if err != nil {
			return false, err
		}

		votesToRemove = append(votesToRemove, key)
		return false, nil
	}); err != nil {
		return math.LegacyDec{}, nil, err
	}

	// remove all votes from store
	for _, key := range votesToRemove {
		if err := k.Votes.Remove(ctx, key); err != nil {
			return math.LegacyDec{}, nil, err
		}
	}

	// iterate over the validators again to tally their voting power
	for _, val := range validators {
		if len(val.Vote) == 0 {
			continue
		}

		sharesAfterDeductions := val.DelegatorShares.Sub(val.DelegatorDeductions)
		votingPower := sharesAfterDeductions.MulInt(val.BondedTokens).Quo(val.DelegatorShares)

		for _, option := range val.Vote {
			weight, _ := math.LegacyNewDecFromStr(option.Weight)
			subPower := votingPower.Mul(weight)
			results[option.Option] = results[option.Option].Add(subPower)
		}
		totalVP = totalVP.Add(votingPower)
	}

	return totalVP, results, nil
}

func createEmptyResults() map[v1.VoteOption]math.LegacyDec {
	results := make(map[v1.VoteOption]math.LegacyDec)
	results[v1.OptionYes] = math.LegacyZeroDec()
	results[v1.OptionAbstain] = math.LegacyZeroDec()
	results[v1.OptionNo] = math.LegacyZeroDec()
	results[v1.OptionNoWithVeto] = math.LegacyZeroDec()
	results[v1.OptionSpam] = math.LegacyZeroDec()

	return results
}

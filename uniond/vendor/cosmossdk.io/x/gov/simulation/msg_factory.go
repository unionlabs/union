package simulation

import (
	"context"
	"math"
	"math/rand"
	"sync/atomic"
	"time"

	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/x/gov/keeper"
	v1 "cosmossdk.io/x/gov/types/v1"

	"github.com/cosmos/cosmos-sdk/simsx"
	sdk "github.com/cosmos/cosmos-sdk/types"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/simulation"
)

func MsgDepositFactory(k *keeper.Keeper, sharedState *SharedState) simsx.SimMsgFactoryFn[*v1.MsgDeposit] {
	return func(ctx context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, *v1.MsgDeposit) {
		r := testData.Rand()
		proposalID, ok := randomProposalID(r, k, ctx, v1.StatusDepositPeriod, sharedState)
		if !ok {
			reporter.Skip("no proposal in deposit state")
			return nil, nil
		}
		proposal, err := k.Proposals.Get(ctx, proposalID)
		if err != nil {
			reporter.Skip(err.Error())
			return nil, nil
		}
		// calculate deposit amount
		deposit := randDeposit(ctx, proposal, k, r, reporter)
		if reporter.IsSkipped() {
			return nil, nil
		}
		from := testData.AnyAccount(reporter, simsx.WithLiquidBalanceGTE(deposit))
		return []simsx.SimAccount{from}, v1.NewMsgDeposit(from.AddressBech32, proposalID, sdk.NewCoins(deposit))
	}
}

func MsgVoteFactory(k *keeper.Keeper, sharedState *SharedState) simsx.SimMsgFactoryFn[*v1.MsgVote] {
	return func(ctx context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, *v1.MsgVote) {
		r := testData.Rand()
		proposalID, ok := randomProposalID(r, k, ctx, v1.StatusVotingPeriod, sharedState)
		if !ok {
			reporter.Skip("no proposal in deposit state")
			return nil, nil
		}
		from := testData.AnyAccount(reporter, simsx.WithSpendableBalance())
		msg := v1.NewMsgVote(from.AddressBech32, proposalID, randomVotingOption(r.Rand), "")
		return []simsx.SimAccount{from}, msg
	}
}

func MsgWeightedVoteFactory(k *keeper.Keeper, sharedState *SharedState) simsx.SimMsgFactoryFn[*v1.MsgVoteWeighted] {
	return func(ctx context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, *v1.MsgVoteWeighted) {
		r := testData.Rand()
		proposalID, ok := randomProposalID(r, k, ctx, v1.StatusVotingPeriod, sharedState)
		if !ok {
			reporter.Skip("no proposal in deposit state")
			return nil, nil
		}
		from := testData.AnyAccount(reporter, simsx.WithSpendableBalance())
		msg := v1.NewMsgVoteWeighted(from.AddressBech32, proposalID, randomWeightedVotingOptions(r.Rand), "")
		return []simsx.SimAccount{from}, msg
	}
}

func MsgCancelProposalFactory(k *keeper.Keeper, sharedState *SharedState) simsx.SimMsgFactoryFn[*v1.MsgCancelProposal] {
	return func(ctx context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, *v1.MsgCancelProposal) {
		r := testData.Rand()
		status := simsx.OneOf(r, []v1.ProposalStatus{v1.StatusDepositPeriod, v1.StatusVotingPeriod})
		proposalID, ok := randomProposalID(r, k, ctx, status, sharedState)
		if !ok {
			reporter.Skip("no proposal in deposit state")
			return nil, nil
		}
		proposal, err := k.Proposals.Get(ctx, proposalID)
		if err != nil {
			reporter.Skip(err.Error())
			return nil, nil
		}
		// is cancellable? copied from keeper
		maxCancelPeriodRate := sdkmath.LegacyMustNewDecFromStr(must(k.Params.Get(ctx)).ProposalCancelMaxPeriod)
		maxCancelPeriod := time.Duration(float64(proposal.VotingEndTime.Sub(*proposal.VotingStartTime)) * maxCancelPeriodRate.MustFloat64()).Round(time.Second)
		if proposal.VotingEndTime.Add(-maxCancelPeriod).Before(simsx.BlockTime(ctx)) {
			reporter.Skip("not cancellable anymore")
			return nil, nil
		}

		from := testData.GetAccount(reporter, proposal.Proposer)
		if from.LiquidBalance().Empty() {
			reporter.Skip("proposer is broke")
			return nil, nil
		}
		msg := v1.NewMsgCancelProposal(proposalID, from.AddressBech32)
		return []simsx.SimAccount{from}, msg
	}
}

func MsgSubmitLegacyProposalFactory(k *keeper.Keeper, contentSimFn simtypes.ContentSimulatorFn) simsx.SimMsgFactoryX { //nolint:staticcheck // used for legacy testing
	return simsx.NewSimMsgFactoryWithFutureOps[*v1.MsgSubmitProposal](func(ctx context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter, fOpsReg simsx.FutureOpsRegistry) ([]simsx.SimAccount, *v1.MsgSubmitProposal) {
		// 1) submit proposal now
		accs := testData.AllAccounts()
		content := contentSimFn(testData.Rand().Rand, ctx, accs)
		if content == nil {
			reporter.Skip("content is nil")
			return nil, nil
		}
		govacc := must(testData.AddressCodec().BytesToString(k.GetGovernanceAccount(ctx).GetAddress()))
		contentMsg := must(v1.NewLegacyContent(content, govacc))
		return submitProposalWithVotesScheduled(ctx, k, testData, reporter, fOpsReg, contentMsg)
	})
}

func MsgSubmitProposalFactory(k *keeper.Keeper, payloadFactory simsx.FactoryMethod) simsx.SimMsgFactoryX {
	return simsx.NewSimMsgFactoryWithFutureOps[*v1.MsgSubmitProposal](func(ctx context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter, fOpsReg simsx.FutureOpsRegistry) ([]simsx.SimAccount, *v1.MsgSubmitProposal) {
		_, proposalMsg := payloadFactory(ctx, testData, reporter)
		return submitProposalWithVotesScheduled(ctx, k, testData, reporter, fOpsReg, proposalMsg)
	})
}

func submitProposalWithVotesScheduled(
	ctx context.Context,
	k *keeper.Keeper,
	testData *simsx.ChainDataSource,
	reporter simsx.SimulationReporter,
	fOpsReg simsx.FutureOpsRegistry,
	proposalMsgs ...sdk.Msg,
) ([]simsx.SimAccount, *v1.MsgSubmitProposal) {
	r := testData.Rand()
	expedited := true
	// expedited := r.Bool()
	params := must(k.Params.Get(ctx))
	minDeposits := params.MinDeposit
	if expedited {
		minDeposits = params.ExpeditedMinDeposit
	}
	minDeposit := r.Coin(minDeposits)

	minDepositRatio := must(sdkmath.LegacyNewDecFromStr(params.GetMinDepositRatio()))
	threshold := minDeposit.Amount.ToLegacyDec().Mul(minDepositRatio).TruncateInt()

	minDepositPercent := must(sdkmath.LegacyNewDecFromStr(params.MinInitialDepositRatio))
	minAmount := sdkmath.LegacyNewDecFromInt(minDeposit.Amount).Mul(minDepositPercent).TruncateInt()
	amount, err := r.PositiveSDKIntn(minDeposit.Amount.Sub(minAmount))
	if err != nil {
		reporter.Skip(err.Error())
		return nil, nil
	}
	if amount.LT(threshold) {
		reporter.Skip("below threshold amount for proposal")
		return nil, nil
	}
	deposit := minDeposit
	// deposit := sdk.Coin{Amount: amount.Add(minAmount), Denom: minDeposit.Denom}

	proposer := testData.AnyAccount(reporter, simsx.WithLiquidBalanceGTE(deposit))
	if reporter.IsSkipped() || !proposer.LiquidBalance().BlockAmount(deposit) {
		return nil, nil
	}
	proposalType := v1.ProposalType_PROPOSAL_TYPE_STANDARD
	if expedited {
		proposalType = v1.ProposalType_PROPOSAL_TYPE_EXPEDITED
	}
	msg, err := v1.NewMsgSubmitProposal(
		proposalMsgs,
		sdk.Coins{deposit},
		proposer.AddressBech32,
		r.StringN(100),
		r.StringN(100),
		r.StringN(100),
		proposalType,
	)
	if err != nil {
		reporter.Skip("unable to generate a submit proposal msg")
		return nil, nil
	}
	// futureOps
	var (
		// The states are:
		// column 1: All validators vote
		// column 2: 90% vote
		// column 3: 75% vote
		// column 4: 40% vote
		// column 5: 15% vote
		// column 6: no one votes
		// All columns sum to 100 for simplicity, values chosen by @valardragon semi-arbitrarily,
		// feel free to change.
		numVotesTransitionMatrix = must(simulation.CreateTransitionMatrix([][]int{
			{20, 10, 0, 0, 0, 0},
			{55, 50, 20, 10, 0, 0},
			{25, 25, 30, 25, 30, 15},
			{0, 15, 30, 25, 30, 30},
			{0, 0, 20, 30, 30, 30},
			{0, 0, 0, 10, 10, 25},
		}))
		statePercentageArray = []float64{1, .9, .75, .4, .15, 0}
		curNumVotesState     = 1
	)

	// get the submitted proposal ID
	proposalID := must(k.ProposalID.Peek(ctx))

	// 2) Schedule operations for votes
	// 2.1) first pick a number of people to vote.
	curNumVotesState = numVotesTransitionMatrix.NextState(r.Rand, curNumVotesState)
	numVotes := int(math.Ceil(float64(testData.AccountsCount()) * statePercentageArray[curNumVotesState]))

	// 2.2) select who votes and when
	whoVotes := r.Perm(testData.AccountsCount())

	// didntVote := whoVotes[numVotes:]
	whoVotes = whoVotes[:numVotes]
	votingPeriod := params.VotingPeriod
	// future ops so that votes do not flood the sims.
	if r.Intn(100) == 1 { // 1% chance
		now := simsx.BlockTime(ctx)
		for i := 0; i < numVotes; i++ {
			var vF simsx.SimMsgFactoryFn[*v1.MsgVote] = func(ctx context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, *v1.MsgVote) {
				switch p, err := k.Proposals.Get(ctx, proposalID); {
				case err != nil:
					reporter.Skip(err.Error())
					return nil, nil
				case p.Status != v1.ProposalStatus_PROPOSAL_STATUS_VOTING_PERIOD:
					reporter.Skip("proposal not in voting period")
					return nil, nil
				}
				voter := testData.AccountAt(reporter, whoVotes[i])
				msg := v1.NewMsgVote(voter.AddressBech32, proposalID, randomVotingOption(r.Rand), "")
				return []simsx.SimAccount{voter}, msg
			}
			whenVote := now.Add(time.Duration(r.Int63n(int64(votingPeriod.Seconds()))) * time.Second)
			fOpsReg.Add(whenVote, vF)
		}
	}
	return []simsx.SimAccount{proposer}, msg
}

// TextProposalFactory returns a random text proposal content.
// A text proposal is a proposal that contains no msgs.
func TextProposalFactory() simsx.SimMsgFactoryFn[sdk.Msg] {
	return func(ctx context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, sdk.Msg) {
		return nil, nil
	}
}

func randDeposit(ctx context.Context, proposal v1.Proposal, k *keeper.Keeper, r *simsx.XRand, reporter simsx.SimulationReporter) sdk.Coin {
	params, err := k.Params.Get(ctx)
	if err != nil {
		reporter.Skipf("gov params: %s", err)
		return sdk.Coin{}
	}
	minDeposits := params.MinDeposit
	if proposal.ProposalType == v1.ProposalType_PROPOSAL_TYPE_EXPEDITED {
		minDeposits = params.ExpeditedMinDeposit
	}
	minDeposit := simsx.OneOf(r, minDeposits)
	minDepositRatio, err := sdkmath.LegacyNewDecFromStr(params.GetMinDepositRatio())
	if err != nil {
		reporter.Skip(err.Error())
		return sdk.Coin{}
	}

	threshold := minDeposit.Amount.ToLegacyDec().Mul(minDepositRatio).TruncateInt()
	depositAmount, err := r.PositiveSDKIntInRange(threshold, minDeposit.Amount)
	if err != nil {
		reporter.Skipf("deposit amount: %s", err)
		return sdk.Coin{}
	}
	return sdk.Coin{Denom: minDeposit.Denom, Amount: depositAmount}
}

// Pick a random proposal ID between the initial proposal ID
// (defined in gov GenesisState) and the latest proposal ID
// that matches a given Status.
// It does not provide a default ID.
func randomProposalID(r *simsx.XRand, k *keeper.Keeper, ctx context.Context, status v1.ProposalStatus, s *SharedState) (proposalID uint64, found bool) {
	proposalID, _ = k.ProposalID.Peek(ctx)
	if initialProposalID := s.getMinProposalID(); initialProposalID == unsetProposalID {
		s.setMinProposalID(proposalID)
	} else if initialProposalID < proposalID {
		proposalID = r.Uint64InRange(initialProposalID, proposalID)
	}
	proposal, err := k.Proposals.Get(ctx, proposalID)
	if err != nil || proposal.Status != status {
		return proposalID, false
	}

	return proposalID, true
}

// Pick a random weighted voting options
func randomWeightedVotingOptions(r *rand.Rand) v1.WeightedVoteOptions {
	w1 := r.Intn(100 + 1)
	w2 := r.Intn(100 - w1 + 1)
	w3 := r.Intn(100 - w1 - w2 + 1)
	w4 := 100 - w1 - w2 - w3
	weightedVoteOptions := v1.WeightedVoteOptions{}
	if w1 > 0 {
		weightedVoteOptions = append(weightedVoteOptions, &v1.WeightedVoteOption{
			Option: v1.OptionYes,
			Weight: sdkmath.LegacyNewDecWithPrec(int64(w1), 2).String(),
		})
	}
	if w2 > 0 {
		weightedVoteOptions = append(weightedVoteOptions, &v1.WeightedVoteOption{
			Option: v1.OptionAbstain,
			Weight: sdkmath.LegacyNewDecWithPrec(int64(w2), 2).String(),
		})
	}
	if w3 > 0 {
		weightedVoteOptions = append(weightedVoteOptions, &v1.WeightedVoteOption{
			Option: v1.OptionNo,
			Weight: sdkmath.LegacyNewDecWithPrec(int64(w3), 2).String(),
		})
	}
	if w4 > 0 {
		weightedVoteOptions = append(weightedVoteOptions, &v1.WeightedVoteOption{
			Option: v1.OptionNoWithVeto,
			Weight: sdkmath.LegacyNewDecWithPrec(int64(w4), 2).String(),
		})
	}
	return weightedVoteOptions
}

func randomVotingOption(r *rand.Rand) v1.VoteOption {
	switch r.Intn(4) {
	case 0:
		return v1.OptionYes
	case 1:
		return v1.OptionAbstain
	case 2:
		return v1.OptionNo
	case 3:
		return v1.OptionNoWithVeto
	default:
		panic("invalid vote option")
	}
}

func must[T any](r T, err error) T {
	if err != nil {
		panic(err)
	}
	return r
}

const unsetProposalID = 100000000000000

// SharedState shared state between message invocations
type SharedState struct {
	minProposalID atomic.Uint64
}

// NewSharedState constructor
func NewSharedState() *SharedState {
	r := &SharedState{}
	r.setMinProposalID(unsetProposalID)
	return r
}

func (s *SharedState) getMinProposalID() uint64 {
	return s.minProposalID.Load()
}

func (s *SharedState) setMinProposalID(id uint64) {
	s.minProposalID.Store(id)
}

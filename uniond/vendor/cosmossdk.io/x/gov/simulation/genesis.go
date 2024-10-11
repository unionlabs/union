package simulation

import (
	"math/rand"
	"time"

	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/x/gov/types"
	v1 "cosmossdk.io/x/gov/types/v1"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	"github.com/cosmos/cosmos-sdk/types/simulation"
)

// Simulation parameter constants
const (
	MinDeposit                    = "min_deposit"
	ExpeditedMinDeposit           = "expedited_min_deposit"
	DepositPeriod                 = "deposit_period"
	MinInitialRatio               = "min_initial_ratio"
	VotingPeriod                  = "voting_period"
	ExpeditedVotingPeriod         = "expedited_voting_period"
	Quorum                        = "quorum"
	ExpeditedQuorum               = "expedited_quorum"
	YesQuorum                     = "yes_quorum"
	Threshold                     = "threshold"
	ExpeditedThreshold            = "expedited_threshold"
	Veto                          = "veto"
	OptimisticRejectedThreshold   = "optimistic_rejected_threshold"
	ProposalCancelRate            = "proposal_cancel_rate"
	ProposalMaxCancelVotingPeriod = "proposal_max_cancel_voting_period"
	MinDepositRatio               = "min_deposit_ratio"

	// ExpeditedThreshold must be at least as large as the regular Threshold
	// Therefore, we use this break out point in randomization.
	tallyNonExpeditedMax = 500

	// Similarly, expedited voting period must be strictly less than the regular
	// voting period to be valid. Therefore, we use this break out point in randomization.
	expeditedMaxVotingPeriod = 60 * 60 * 24 * 2
)

// GenDepositPeriod returns randomized DepositPeriod
func GenDepositPeriod(r *rand.Rand) time.Duration {
	return time.Duration(simulation.RandIntBetween(r, 1, 2*60*60*24*2)) * time.Second
}

// GenMinDeposit returns randomized MinDeposit
func GenMinDeposit(r *rand.Rand, bondDenom string) sdk.Coins {
	return sdk.NewCoins(sdk.NewInt64Coin(bondDenom, int64(simulation.RandIntBetween(r, 1, 1e3/2))))
}

// GenExpeditedMinDeposit returns randomized ExpeditedMinDeposit
// It is always greater than GenMinDeposit
func GenExpeditedMinDeposit(r *rand.Rand, bondDenom string) sdk.Coins {
	return sdk.NewCoins(sdk.NewInt64Coin(bondDenom, int64(simulation.RandIntBetween(r, 1e3/2, 1e3))))
}

// GenDepositMinInitialRatio returns randomized DepositMinInitialRatio
func GenDepositMinInitialDepositRatio(r *rand.Rand) sdkmath.LegacyDec {
	return sdkmath.LegacyNewDec(int64(simulation.RandIntBetween(r, 0, 99))).Quo(sdkmath.LegacyNewDec(100))
}

// GenProposalCancelRate returns randomized ProposalCancelRate
func GenProposalCancelRate(r *rand.Rand) sdkmath.LegacyDec {
	return sdkmath.LegacyNewDec(int64(simulation.RandIntBetween(r, 0, 99))).Quo(sdkmath.LegacyNewDec(100))
}

func GenProposalMaxCancelVotingPeriod(r *rand.Rand) sdkmath.LegacyDec {
	return sdkmath.LegacyNewDec(int64(simulation.RandIntBetween(r, 0, 99))).Quo(sdkmath.LegacyNewDec(100))
}

// GenVotingPeriod returns randomized VotingPeriod
func GenVotingPeriod(r *rand.Rand) time.Duration {
	return time.Duration(simulation.RandIntBetween(r, expeditedMaxVotingPeriod, 2*expeditedMaxVotingPeriod)) * time.Second
}

// GenExpeditedVotingPeriod randomized ExpeditedVotingPeriod
func GenExpeditedVotingPeriod(r *rand.Rand) time.Duration {
	return time.Duration(simulation.RandIntBetween(r, 1, expeditedMaxVotingPeriod)) * time.Second
}

// GenQuorum returns randomized Quorum
func GenQuorum(r *rand.Rand) sdkmath.LegacyDec {
	return sdkmath.LegacyNewDecWithPrec(int64(simulation.RandIntBetween(r, 334, 500)), 3)
}

// GenYesQuorum returns randomized YesQuorum
func GenYesQuorum(r *rand.Rand) sdkmath.LegacyDec {
	return sdkmath.LegacyNewDecWithPrec(int64(simulation.RandIntBetween(r, 0, 500)), 3)
}

// GenThreshold returns randomized Threshold
func GenThreshold(r *rand.Rand) sdkmath.LegacyDec {
	return sdkmath.LegacyNewDecWithPrec(int64(simulation.RandIntBetween(r, 450, tallyNonExpeditedMax+1)), 3)
}

// GenExpeditedThreshold randomized ExpeditedThreshold
func GenExpeditedThreshold(r *rand.Rand) sdkmath.LegacyDec {
	return sdkmath.LegacyNewDecWithPrec(int64(simulation.RandIntBetween(r, tallyNonExpeditedMax, 550)), 3)
}

// GenOptimisticRejectedThreshold randomized OptimisticRejectedThreshold
func GenOptimisticRejectedThreshold(r *rand.Rand) sdkmath.LegacyDec {
	return sdkmath.LegacyNewDecWithPrec(int64(simulation.RandIntBetween(r, 0, 200)), 3)
}

// GenVeto returns randomized Veto
func GenVeto(r *rand.Rand) sdkmath.LegacyDec {
	return sdkmath.LegacyNewDecWithPrec(int64(simulation.RandIntBetween(r, 250, 334)), 3)
}

// GenMinDepositRatio returns randomized DepositMinRatio
func GenMinDepositRatio(r *rand.Rand) sdkmath.LegacyDec {
	return sdkmath.LegacyMustNewDecFromStr("0.01")
}

// RandomizedGenState generates a random GenesisState for gov
func RandomizedGenState(simState *module.SimulationState) {
	startingProposalID := uint64(simState.Rand.Intn(100))

	var minDeposit sdk.Coins
	simState.AppParams.GetOrGenerate(MinDeposit, &minDeposit, simState.Rand, func(r *rand.Rand) { minDeposit = GenMinDeposit(r, simState.BondDenom) })

	var expeditedMinDeposit sdk.Coins
	simState.AppParams.GetOrGenerate(ExpeditedMinDeposit, &expeditedMinDeposit, simState.Rand, func(r *rand.Rand) { expeditedMinDeposit = GenExpeditedMinDeposit(r, simState.BondDenom) })

	var depositPeriod time.Duration
	simState.AppParams.GetOrGenerate(DepositPeriod, &depositPeriod, simState.Rand, func(r *rand.Rand) { depositPeriod = GenDepositPeriod(r) })

	var minInitialDepositRatio sdkmath.LegacyDec
	simState.AppParams.GetOrGenerate(MinInitialRatio, &minInitialDepositRatio, simState.Rand, func(r *rand.Rand) { minInitialDepositRatio = GenDepositMinInitialDepositRatio(r) })

	var proposalCancelRate sdkmath.LegacyDec
	simState.AppParams.GetOrGenerate(ProposalCancelRate, &proposalCancelRate, simState.Rand, func(r *rand.Rand) { proposalCancelRate = GenProposalCancelRate(r) })

	var proposalMaxCancelVotingPeriod sdkmath.LegacyDec
	simState.AppParams.GetOrGenerate(ProposalMaxCancelVotingPeriod, &proposalMaxCancelVotingPeriod, simState.Rand, func(r *rand.Rand) { proposalMaxCancelVotingPeriod = GenProposalMaxCancelVotingPeriod(r) })

	var votingPeriod time.Duration
	simState.AppParams.GetOrGenerate(VotingPeriod, &votingPeriod, simState.Rand, func(r *rand.Rand) { votingPeriod = GenVotingPeriod(r) })

	var expeditedVotingPeriod time.Duration
	simState.AppParams.GetOrGenerate(ExpeditedVotingPeriod, &expeditedVotingPeriod, simState.Rand, func(r *rand.Rand) { expeditedVotingPeriod = GenExpeditedVotingPeriod(r) })

	var quorum sdkmath.LegacyDec
	simState.AppParams.GetOrGenerate(Quorum, &quorum, simState.Rand, func(r *rand.Rand) { quorum = GenQuorum(r) })

	var yesQuorum sdkmath.LegacyDec
	simState.AppParams.GetOrGenerate(YesQuorum, &yesQuorum, simState.Rand, func(r *rand.Rand) { yesQuorum = GenQuorum(r) })

	var expeditedQuorum sdkmath.LegacyDec
	simState.AppParams.GetOrGenerate(ExpeditedQuorum, &expeditedQuorum, simState.Rand, func(r *rand.Rand) { expeditedQuorum = GenQuorum(r) })

	var threshold sdkmath.LegacyDec
	simState.AppParams.GetOrGenerate(Threshold, &threshold, simState.Rand, func(r *rand.Rand) { threshold = GenThreshold(r) })

	var expeditedVotingThreshold sdkmath.LegacyDec
	simState.AppParams.GetOrGenerate(ExpeditedThreshold, &expeditedVotingThreshold, simState.Rand, func(r *rand.Rand) { expeditedVotingThreshold = GenExpeditedThreshold(r) })

	var veto sdkmath.LegacyDec
	simState.AppParams.GetOrGenerate(Veto, &veto, simState.Rand, func(r *rand.Rand) { veto = GenVeto(r) })

	var optimisticRejectedThreshold sdkmath.LegacyDec
	simState.AppParams.GetOrGenerate(OptimisticRejectedThreshold, &optimisticRejectedThreshold, simState.Rand, func(r *rand.Rand) { optimisticRejectedThreshold = GenOptimisticRejectedThreshold(r) })

	var minDepositRatio sdkmath.LegacyDec
	simState.AppParams.GetOrGenerate(MinDepositRatio, &minDepositRatio, simState.Rand, func(r *rand.Rand) { minDepositRatio = GenMinDepositRatio(r) })

	govGenesis := v1.NewGenesisState(
		startingProposalID,
		v1.NewParams(
			minDeposit,
			expeditedMinDeposit,
			depositPeriod,
			votingPeriod,
			expeditedVotingPeriod,
			quorum.String(),
			yesQuorum.String(),
			expeditedQuorum.String(),
			threshold.String(),
			expeditedVotingThreshold.String(),
			veto.String(),
			minInitialDepositRatio.String(),
			proposalCancelRate.String(),
			"",
			proposalMaxCancelVotingPeriod.String(),
			simState.Rand.Intn(2) == 0,
			simState.Rand.Intn(2) == 0,
			simState.Rand.Intn(2) == 0,
			minDepositRatio.String(),
			optimisticRejectedThreshold.String(),
			[]string{},
			10_000_000,
		),
	)

	simState.GenState[types.ModuleName] = simState.Cdc.MustMarshalJSON(govGenesis)
}

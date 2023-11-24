package v1beta1

import (
	"time"

	"cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// Default period for deposits & voting
const (
	DefaultPeriod time.Duration = time.Hour * 24 * 2 // 2 days
)

// Default governance params
var (
	DefaultMinDepositTokens = math.NewInt(10000000)
	DefaultQuorum           = math.LegacyNewDecWithPrec(334, 3)
	DefaultThreshold        = math.LegacyNewDecWithPrec(5, 1)
	DefaultVetoThreshold    = math.LegacyNewDecWithPrec(334, 3)
)

// NewDepositParams creates a new DepositParams object
func NewDepositParams(minDeposit sdk.Coins, maxDepositPeriod time.Duration) DepositParams {
	return DepositParams{
		MinDeposit:       minDeposit,
		MaxDepositPeriod: maxDepositPeriod,
	}
}

// DefaultDepositParams returns the default parameters for deposits
func DefaultDepositParams() DepositParams {
	return NewDepositParams(
		sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, DefaultMinDepositTokens)),
		DefaultPeriod,
	)
}

// Equal checks equality of DepositParams
func (dp DepositParams) Equal(dp2 DepositParams) bool {
	return dp.MinDeposit.Equal(dp2.MinDeposit) && dp.MaxDepositPeriod == dp2.MaxDepositPeriod
}

// NewTallyParams creates a new TallyParams object
func NewTallyParams(quorum, threshold, vetoThreshold math.LegacyDec) TallyParams {
	return TallyParams{
		Quorum:        quorum,
		Threshold:     threshold,
		VetoThreshold: vetoThreshold,
	}
}

// DefaultTallyParams returns default parameters for tallying
func DefaultTallyParams() TallyParams {
	return NewTallyParams(DefaultQuorum, DefaultThreshold, DefaultVetoThreshold)
}

// Equal checks equality of TallyParams
func (tp TallyParams) Equal(other TallyParams) bool {
	return tp.Quorum.Equal(other.Quorum) && tp.Threshold.Equal(other.Threshold) && tp.VetoThreshold.Equal(other.VetoThreshold)
}

// NewVotingParams creates a new VotingParams object
func NewVotingParams(votingPeriod time.Duration) VotingParams {
	return VotingParams{
		VotingPeriod: votingPeriod,
	}
}

// DefaultVotingParams default parameters for voting
func DefaultVotingParams() VotingParams {
	return NewVotingParams(DefaultPeriod)
}

// Equal checks equality of TallyParams
func (vp VotingParams) Equal(other VotingParams) bool {
	return vp.VotingPeriod == other.VotingPeriod
}

// Params returns all of the governance params
type Params struct {
	VotingParams  VotingParams  `json:"voting_params" yaml:"voting_params"`
	TallyParams   TallyParams   `json:"tally_params" yaml:"tally_params"`
	DepositParams DepositParams `json:"deposit_params" yaml:"deposit_params"`
}

// String implements stringer interface
func (gp Params) String() string {
	return gp.VotingParams.String() + "\n" +
		gp.TallyParams.String() + "\n" + gp.DepositParams.String()
}

// NewParams creates a new gov Params instance
func NewParams(vp VotingParams, tp TallyParams, dp DepositParams) Params {
	return Params{
		VotingParams:  vp,
		DepositParams: dp,
		TallyParams:   tp,
	}
}

// DefaultParams returns the default governance params
func DefaultParams() Params {
	return NewParams(DefaultVotingParams(), DefaultTallyParams(), DefaultDepositParams())
}

package types

// Delay, in blocks, between when validator updates are returned to the
// consensus-engine and when they are applied. For example, if
// ValidatorUpdateDelay is set to X, and if a validator set update is
// returned with new validators at the end of block 10, then the new
// validators are expected to sign blocks beginning at block 11+X.
//
// This value is constant as this should not change without a hard fork.
// For Tendermint this should be set to 1 block, for more details see:
// https://tendermint.com/docs/spec/abci/apps.html#endblock
const ValidatorUpdateDelay int64 = 1

var (
	// DefaultBondDenom is the default bondable coin denomination (defaults to stake)
	// Overwriting this value has the side effect of changing the default denomination in genesis
	DefaultBondDenom = "stake"

	// DefaultPowerReduction is the default amount of staking tokens required for 1 unit of consensus-engine power
	DefaultPowerReduction = NewIntFromUint64(1000000)
)

// TokensToConsensusPower - convert input tokens to potential consensus-engine power
func TokensToConsensusPower(tokens Int, powerReduction Int) int64 {
	return (tokens.Quo(powerReduction)).Int64()
}

// TokensFromConsensusPower - convert input power to tokens
func TokensFromConsensusPower(power int64, powerReduction Int) Int {
	return NewInt(power).Mul(powerReduction)
}

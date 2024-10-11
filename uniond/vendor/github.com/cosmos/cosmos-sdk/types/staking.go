package types

import (
	"cosmossdk.io/math"

	cryptokeys "github.com/cosmos/cosmos-sdk/crypto/keys"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
)

// Delay, in blocks, between when validator updates are returned to the
// consensus-engine and when they are applied. For example, if
// ValidatorUpdateDelay is set to X, and if a validator set update is
// returned with new validators at the end of block 10, then the new
// validators are expected to sign blocks beginning at block 11+X.
//
// This value is constant as this should not change without a hard fork.
// For CometBFT this should be set to 1 block, for more details see:
// https://github.com/cometbft/cometbft/blob/main/spec/abci/abci%2B%2B_basic_concepts.md#consensusblock-execution-methods
const ValidatorUpdateDelay int64 = 1

var (
	// DefaultBondDenom is the default bondable coin denomination (defaults to stake)
	// Overwriting this value has the side effect of changing the default denomination in genesis
	DefaultBondDenom = "stake"

	// DefaultPowerReduction is the default amount of staking tokens required for 1 unit of consensus-engine power
	DefaultPowerReduction = math.NewIntFromUint64(1000000)

	// PubKeyEd25519Type is ed25519 for type the consensus params validator pub_key_types params
	PubKeyEd25519Type = "ed25519"
)

// TokensToConsensusPower - convert input tokens to potential consensus-engine power
func TokensToConsensusPower(tokens, powerReduction math.Int) int64 {
	return (tokens.Quo(powerReduction)).Int64()
}

// TokensFromConsensusPower - convert input power to tokens
func TokensFromConsensusPower(power int64, powerReduction math.Int) math.Int {
	return math.NewInt(power).Mul(powerReduction)
}

// ______________________________________________________________________
// Delegation & Validator Interfaces are moved here to avoid direct dependency on the staking module in expected keeper interfaces

// BondStatus is the status of a validator.
type BondStatus int32

const (
	// UNSPECIFIED defines an invalid validator status.
	Unspecified BondStatus = 0
	// UNBONDED defines a validator that is not bonded.
	Unbonded BondStatus = 1
	// UNBONDING defines a validator that is unbonding.
	Unbonding BondStatus = 2
	// BONDED defines a validator that is bonded.
	Bonded BondStatus = 3
)

// BondStatus_name is the string representation of BondStatus.
var bondStatus_name = map[int32]string{
	0: "BOND_STATUS_UNSPECIFIED",
	1: "BOND_STATUS_UNBONDED",
	2: "BOND_STATUS_UNBONDING",
	3: "BOND_STATUS_BONDED",
}

func (x BondStatus) String() string {
	return bondStatus_name[int32(x)]
}

// DelegationI delegation bond for a delegated proof of stake system
type DelegationI interface {
	GetDelegatorAddr() string  // delegator string for the bond
	GetValidatorAddr() string  // validator operator address
	GetShares() math.LegacyDec // amount of validator's shares held in this delegation
}

// ValidatorI expected validator functions
type ValidatorI interface {
	IsJailed() bool                                                 // whether the validator is jailed
	GetMoniker() string                                             // moniker of the validator
	GetStatus() BondStatus                                          // status of the validator
	IsBonded() bool                                                 // check if has a bonded status
	IsUnbonded() bool                                               // check if has status unbonded
	IsUnbonding() bool                                              // check if has status unbonding
	GetOperator() string                                            // operator address to receive/return validators coins
	ConsPubKey() (cryptotypes.PubKey, error)                        // validation consensus pubkey (cryptotypes.PubKey)
	GetConsAddr() ([]byte, error)                                   // validation consensus address
	GetTokens() math.Int                                            // validation tokens
	GetBondedTokens() math.Int                                      // validator bonded tokens
	GetConsensusPower(math.Int) int64                               // validation power in CometBFT
	GetCommission() math.LegacyDec                                  // validator commission rate
	GetMinSelfDelegation() math.Int                                 // validator minimum self delegation
	GetDelegatorShares() math.LegacyDec                             // total outstanding delegator shares
	TokensFromShares(math.LegacyDec) math.LegacyDec                 // token worth of provided delegator shares
	TokensFromSharesTruncated(math.LegacyDec) math.LegacyDec        // token worth of provided delegator shares, truncated
	TokensFromSharesRoundUp(math.LegacyDec) math.LegacyDec          // token worth of provided delegator shares, rounded up
	SharesFromTokens(amt math.Int) (math.LegacyDec, error)          // shares worth of delegator's bond
	SharesFromTokensTruncated(amt math.Int) (math.LegacyDec, error) // truncated shares worth of delegator's bond
}

// GenesisValidator is an initial validator.
type GenesisValidator struct {
	Address ConsAddress           `json:"address"`
	PubKey  cryptokeys.JSONPubkey `json:"pub_key"`
	Power   int64                 `json:"power"`
	Name    string                `json:"name"`
}

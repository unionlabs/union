package types

import (
	cmtprotocrypto "github.com/cometbft/cometbft/proto/tendermint/crypto"

	"cosmossdk.io/math"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
)

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
	TmConsPublicKey() (cmtprotocrypto.PublicKey, error)             // validation consensus pubkey (CometBFT)
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

package types

import (
	"cosmossdk.io/math"
	tmprotocrypto "github.com/cometbft/cometbft/proto/tendermint/crypto"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// DelegationI delegation bond for a delegated proof of stake system
type DelegationI interface {
	GetDelegatorAddr() sdk.AccAddress // delegator sdk.AccAddress for the bond
	GetValidatorAddr() sdk.ValAddress // validator operator address
	GetShares() math.LegacyDec        // amount of validator's shares held in this delegation
}

// ValidatorI expected validator functions
type ValidatorI interface {
	IsJailed() bool                                          // whether the validator is jailed
	GetMoniker() string                                      // moniker of the validator
	GetStatus() BondStatus                                   // status of the validator
	IsBonded() bool                                          // check if has a bonded status
	IsUnbonded() bool                                        // check if has status unbonded
	IsUnbonding() bool                                       // check if has status unbonding
	GetOperator() sdk.ValAddress                             // operator address to receive/return validators coins
	ConsPubKey() (cryptotypes.PubKey, error)                 // validation consensus pubkey (cryptotypes.PubKey)
	TmConsPublicKey() (tmprotocrypto.PublicKey, error)       // validation consensus pubkey (Tendermint)
	GetConsAddr() (sdk.ConsAddress, error)                   // validation consensus address
	GetTokens() math.Int                                     // validation tokens
	GetBondedTokens() math.Int                               // validator bonded tokens
	GetConsensusPower(math.Int) int64                        // validation power in tendermint
	GetCommission() math.LegacyDec                           // validator commission rate
	GetMinSelfDelegation() math.Int                          // validator minimum self delegation
	GetDelegatorShares() math.LegacyDec                      // total outstanding delegator shares
	TokensFromShares(sdk.Dec) math.LegacyDec                 // token worth of provided delegator shares
	TokensFromSharesTruncated(sdk.Dec) math.LegacyDec        // token worth of provided delegator shares, truncated
	TokensFromSharesRoundUp(sdk.Dec) math.LegacyDec          // token worth of provided delegator shares, rounded up
	SharesFromTokens(amt math.Int) (sdk.Dec, error)          // shares worth of delegator's bond
	SharesFromTokensTruncated(amt math.Int) (sdk.Dec, error) // truncated shares worth of delegator's bond
}

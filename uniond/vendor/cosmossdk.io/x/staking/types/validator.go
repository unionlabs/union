package types

import (
	"bytes"
	"fmt"
	"sort"
	"strings"
	"time"

	gogoprotoany "github.com/cosmos/gogoproto/types/any"

	"cosmossdk.io/core/address"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/errors"
	"cosmossdk.io/math"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

const (
	// TODO: Why can't we just have one string description which can be JSON by convention
	MaxMonikerLength         = 70
	MaxIdentityLength        = 3000
	MaxWebsiteLength         = 140
	MaxSecurityContactLength = 140
	MaxDetailsLength         = 280
)

var (
	BondStatusUnspecified = BondStatus_name[int32(Unspecified)]
	BondStatusUnbonded    = BondStatus_name[int32(Unbonded)]
	BondStatusUnbonding   = BondStatus_name[int32(Unbonding)]
	BondStatusBonded      = BondStatus_name[int32(Bonded)]
)

var _ sdk.ValidatorI = Validator{}

// NewValidator constructs a new Validator
func NewValidator(operator string, pubKey cryptotypes.PubKey, description Description) (Validator, error) {
	pkAny, err := codectypes.NewAnyWithValue(pubKey)
	if err != nil {
		return Validator{}, err
	}

	return Validator{
		OperatorAddress:         operator,
		ConsensusPubkey:         pkAny,
		Jailed:                  false,
		Status:                  Unbonded,
		Tokens:                  math.ZeroInt(),
		DelegatorShares:         math.LegacyZeroDec(),
		Description:             description,
		UnbondingHeight:         int64(0),
		UnbondingTime:           time.Unix(0, 0).UTC(),
		Commission:              NewCommission(math.LegacyZeroDec(), math.LegacyZeroDec(), math.LegacyZeroDec()),
		MinSelfDelegation:       math.OneInt(),
		UnbondingOnHoldRefCount: 0,
	}, nil
}

// Validators is a collection of Validator
type Validators struct {
	Validators     []Validator
	ValidatorCodec address.Codec
}

func (v Validators) String() (out string) {
	for _, val := range v.Validators {
		out += val.String() + "\n"
	}

	return strings.TrimSpace(out)
}

// ToSDKValidators -  convenience function convert []Validator to []sdk.ValidatorI
func (v Validators) ToSDKValidators() (validators []sdk.ValidatorI) {
	for _, val := range v.Validators {
		validators = append(validators, val)
	}

	return validators
}

// Sort Validators sorts validator array in ascending operator address order
func (v Validators) Sort() {
	sort.Sort(v)
}

// Implements sort interface
func (v Validators) Len() int {
	return len(v.Validators)
}

// Implements sort interface
func (v Validators) Less(i, j int) bool {
	vi, err := v.ValidatorCodec.StringToBytes(v.Validators[i].GetOperator())
	if err != nil {
		panic(err)
	}
	vj, err := v.ValidatorCodec.StringToBytes(v.Validators[j].GetOperator())
	if err != nil {
		panic(err)
	}

	return bytes.Compare(vi, vj) == -1
}

// Implements sort interface
func (v Validators) Swap(i, j int) {
	v.Validators[i], v.Validators[j] = v.Validators[j], v.Validators[i]
}

// ValidatorsByVotingPower implements sort.Interface for []Validator based on
// the VotingPower and Address fields.
// The validators are sorted first by their voting power (descending). Secondary index - Address (ascending).
// Copied from tendermint/types/validator_set.go
type ValidatorsByVotingPower []Validator

func (valz ValidatorsByVotingPower) Len() int { return len(valz) }

func (valz ValidatorsByVotingPower) Less(i, j int, r math.Int) bool {
	if valz[i].ConsensusPower(r) == valz[j].ConsensusPower(r) {
		addrI, errI := valz[i].GetConsAddr()
		addrJ, errJ := valz[j].GetConsAddr()
		// If either returns error, then return false
		if errI != nil || errJ != nil {
			return false
		}
		return bytes.Compare(addrI, addrJ) == -1
	}
	return valz[i].ConsensusPower(r) > valz[j].ConsensusPower(r)
}

func (valz ValidatorsByVotingPower) Swap(i, j int) {
	valz[i], valz[j] = valz[j], valz[i]
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (v Validators) UnpackInterfaces(c gogoprotoany.AnyUnpacker) error {
	for i := range v.Validators {
		if err := v.Validators[i].UnpackInterfaces(c); err != nil {
			return err
		}
	}
	return nil
}

// return the redelegation
func MustMarshalValidator(cdc codec.BinaryCodec, validator *Validator) []byte {
	return cdc.MustMarshal(validator)
}

// unmarshal a redelegation from a store value
func MustUnmarshalValidator(cdc codec.BinaryCodec, value []byte) Validator {
	validator, err := UnmarshalValidator(cdc, value)
	if err != nil {
		panic(err)
	}

	return validator
}

// unmarshal a redelegation from a store value
func UnmarshalValidator(cdc codec.BinaryCodec, value []byte) (v Validator, err error) {
	err = cdc.Unmarshal(value, &v)
	return v, err
}

// IsBonded checks if the validator status equals Bonded
func (v Validator) IsBonded() bool {
	return v.GetStatus() == sdk.Bonded
}

// IsUnbonded checks if the validator status equals Unbonded
func (v Validator) IsUnbonded() bool {
	return v.GetStatus() == sdk.Unbonded
}

// IsUnbonding checks if the validator status equals Unbonding
func (v Validator) IsUnbonding() bool {
	return v.GetStatus() == sdk.Unbonding
}

// constant used in flags to indicate that description field should not be updated
const DoNotModifyDesc = "[do-not-modify]"

func NewDescription(moniker, identity, website, securityContact, details string) Description {
	return Description{
		Moniker:         moniker,
		Identity:        identity,
		Website:         website,
		SecurityContact: securityContact,
		Details:         details,
	}
}

// UpdateDescription updates the fields of a given description. An error is
// returned if the resulting description contains an invalid length.
func (d Description) UpdateDescription(d2 Description) (Description, error) {
	if d2.Moniker == DoNotModifyDesc {
		d2.Moniker = d.Moniker
	}

	if d2.Identity == DoNotModifyDesc {
		d2.Identity = d.Identity
	}

	if d2.Website == DoNotModifyDesc {
		d2.Website = d.Website
	}

	if d2.SecurityContact == DoNotModifyDesc {
		d2.SecurityContact = d.SecurityContact
	}

	if d2.Details == DoNotModifyDesc {
		d2.Details = d.Details
	}

	return NewDescription(
		d2.Moniker,
		d2.Identity,
		d2.Website,
		d2.SecurityContact,
		d2.Details,
	).EnsureLength()
}

// EnsureLength ensures the length of a validator's description.
func (d Description) EnsureLength() (Description, error) {
	if len(d.Moniker) > MaxMonikerLength {
		return d, errors.Wrapf(sdkerrors.ErrInvalidRequest, "invalid moniker length; got: %d, max: %d", len(d.Moniker), MaxMonikerLength)
	}

	if len(d.Identity) > MaxIdentityLength {
		return d, errors.Wrapf(sdkerrors.ErrInvalidRequest, "invalid identity length; got: %d, max: %d", len(d.Identity), MaxIdentityLength)
	}

	if len(d.Website) > MaxWebsiteLength {
		return d, errors.Wrapf(sdkerrors.ErrInvalidRequest, "invalid website length; got: %d, max: %d", len(d.Website), MaxWebsiteLength)
	}

	if len(d.SecurityContact) > MaxSecurityContactLength {
		return d, errors.Wrapf(sdkerrors.ErrInvalidRequest, "invalid security contact length; got: %d, max: %d", len(d.SecurityContact), MaxSecurityContactLength)
	}

	if len(d.Details) > MaxDetailsLength {
		return d, errors.Wrapf(sdkerrors.ErrInvalidRequest, "invalid details length; got: %d, max: %d", len(d.Details), MaxDetailsLength)
	}

	return d, nil
}

// ModuleValidatorUpdate returns a appmodule.ValidatorUpdate from a staking validator type
// with the full validator power.
// It replaces the previous ABCIValidatorUpdate function.
func (v Validator) ModuleValidatorUpdate(r math.Int) appmodule.ValidatorUpdate {
	consPk, err := v.ConsPubKey()
	if err != nil {
		panic(err)
	}

	return appmodule.ValidatorUpdate{
		PubKey:     consPk.Bytes(),
		PubKeyType: consPk.Type(),
		Power:      v.ConsensusPower(r),
	}
}

// ModuleValidatorUpdateZero returns a appmodule.ValidatorUpdate from a staking validator type
// with zero power used for validator updates.
// It replaces the previous ABCIValidatorUpdateZero function.
func (v Validator) ModuleValidatorUpdateZero() appmodule.ValidatorUpdate {
	consPk, err := v.ConsPubKey()
	if err != nil {
		panic(err)
	}

	return appmodule.ValidatorUpdate{
		PubKey:     consPk.Bytes(),
		PubKeyType: consPk.Type(),
		Power:      0,
	}
}

// SetInitialCommission attempts to set a validator's initial commission. An
// error is returned if the commission is invalid.
func (v Validator) SetInitialCommission(commission Commission) (Validator, error) {
	if err := commission.Validate(); err != nil {
		return v, err
	}

	v.Commission = commission

	return v, nil
}

// In some situations, the exchange rate becomes invalid, e.g. if
// Validator loses all tokens due to slashing. In this case,
// make all future delegations invalid.
func (v Validator) InvalidExRate() bool {
	return v.Tokens.IsZero() && v.DelegatorShares.IsPositive()
}

// calculate the token worth of provided shares
func (v Validator) TokensFromShares(shares math.LegacyDec) math.LegacyDec {
	return (shares.MulInt(v.Tokens)).Quo(v.DelegatorShares)
}

// calculate the token worth of provided shares, truncated
func (v Validator) TokensFromSharesTruncated(shares math.LegacyDec) math.LegacyDec {
	return (shares.MulInt(v.Tokens)).QuoTruncate(v.DelegatorShares)
}

// TokensFromSharesRoundUp returns the token worth of provided shares, rounded
// up.
func (v Validator) TokensFromSharesRoundUp(shares math.LegacyDec) math.LegacyDec {
	return (shares.MulInt(v.Tokens)).QuoRoundUp(v.DelegatorShares)
}

// SharesFromTokens returns the shares of a delegation given a bond amount. It
// returns an error if the validator has no tokens.
func (v Validator) SharesFromTokens(amt math.Int) (math.LegacyDec, error) {
	if v.Tokens.IsZero() {
		return math.LegacyZeroDec(), ErrInsufficientShares
	}

	return v.GetDelegatorShares().MulInt(amt).QuoInt(v.GetTokens()), nil
}

// SharesFromTokensTruncated returns the truncated shares of a delegation given
// a bond amount. It returns an error if the validator has no tokens.
func (v Validator) SharesFromTokensTruncated(amt math.Int) (math.LegacyDec, error) {
	if v.Tokens.IsZero() {
		return math.LegacyZeroDec(), ErrInsufficientShares
	}

	return v.GetDelegatorShares().MulInt(amt).QuoTruncate(math.LegacyNewDecFromInt(v.GetTokens())), nil
}

// get the bonded tokens which the validator holds
func (v Validator) BondedTokens() math.Int {
	if v.IsBonded() {
		return v.Tokens
	}

	return math.ZeroInt()
}

// ConsensusPower gets the consensus-engine power. Aa reduction of 10^6 from
// validator tokens is applied
func (v Validator) ConsensusPower(r math.Int) int64 {
	if v.IsBonded() {
		return v.PotentialConsensusPower(r)
	}

	return 0
}

// PotentialConsensusPower returns the potential consensus-engine power.
func (v Validator) PotentialConsensusPower(r math.Int) int64 {
	return sdk.TokensToConsensusPower(v.Tokens, r)
}

// UpdateStatus updates the location of the shares within a validator
// to reflect the new status
func (v Validator) UpdateStatus(newStatus BondStatus) Validator {
	v.Status = newStatus
	return v
}

// AddTokensFromDel adds tokens to a validator
func (v Validator) AddTokensFromDel(amount math.Int) (Validator, math.LegacyDec) {
	// calculate the shares to issue
	var issuedShares math.LegacyDec
	if v.DelegatorShares.IsZero() {
		// the first delegation to a validator sets the exchange rate to one
		issuedShares = math.LegacyNewDecFromInt(amount)
	} else {
		shares, err := v.SharesFromTokens(amount)
		if err != nil {
			panic(err)
		}

		issuedShares = shares
	}

	v.Tokens = v.Tokens.Add(amount)
	v.DelegatorShares = v.DelegatorShares.Add(issuedShares)

	return v, issuedShares
}

// RemoveTokens removes tokens from a validator
func (v Validator) RemoveTokens(tokens math.Int) Validator {
	if tokens.IsNegative() {
		panic(fmt.Sprintf("should not happen: trying to remove negative tokens %v", tokens))
	}

	if v.Tokens.LT(tokens) {
		panic(fmt.Sprintf("should not happen: only have %v tokens, trying to remove %v", v.Tokens, tokens))
	}

	v.Tokens = v.Tokens.Sub(tokens)

	return v
}

// RemoveDelShares removes delegator shares from a validator.
// NOTE: because token fractions are left in the valiadator,
//
//	the exchange rate of future shares of this validator can increase.
func (v Validator) RemoveDelShares(delShares math.LegacyDec) (Validator, math.Int) {
	remainingShares := v.DelegatorShares.Sub(delShares)

	var issuedTokens math.Int
	if remainingShares.IsZero() {
		// last delegation share gets any trimmings
		issuedTokens = v.Tokens
		v.Tokens = math.ZeroInt()
	} else {
		// leave excess tokens in the validator
		// however fully use all the delegator shares
		issuedTokens = v.TokensFromShares(delShares).TruncateInt()
		v.Tokens = v.Tokens.Sub(issuedTokens)

		if v.Tokens.IsNegative() {
			panic("attempting to remove more tokens than available in validator")
		}
	}

	v.DelegatorShares = remainingShares

	return v, issuedTokens
}

// MinEqual defines a more minimum set of equality conditions when comparing two
// validators.
func (v *Validator) MinEqual(other *Validator) bool {
	return v.OperatorAddress == other.OperatorAddress &&
		v.Status == other.Status &&
		v.Tokens.Equal(other.Tokens) &&
		v.DelegatorShares.Equal(other.DelegatorShares) &&
		v.Description.Equal(other.Description) &&
		v.Commission.Equal(other.Commission) &&
		v.Jailed == other.Jailed &&
		v.MinSelfDelegation.Equal(other.MinSelfDelegation) &&
		v.ConsensusPubkey.Equal(other.ConsensusPubkey)
}

// Equal checks if the receiver equals the parameter
func (v *Validator) Equal(v2 *Validator) bool {
	return v.MinEqual(v2) &&
		v.UnbondingHeight == v2.UnbondingHeight &&
		v.UnbondingTime.Equal(v2.UnbondingTime)
}

func (v Validator) IsJailed() bool            { return v.Jailed }
func (v Validator) GetMoniker() string        { return v.Description.Moniker }
func (v Validator) GetStatus() sdk.BondStatus { return sdk.BondStatus(v.Status) }
func (v Validator) GetOperator() string {
	return v.OperatorAddress
}

// ConsPubKey returns the validator PubKey as a cryptotypes.PubKey.
func (v Validator) ConsPubKey() (cryptotypes.PubKey, error) {
	pk, ok := v.ConsensusPubkey.GetCachedValue().(cryptotypes.PubKey)
	if !ok {
		return nil, errors.Wrapf(sdkerrors.ErrInvalidType, "expecting cryptotypes.PubKey, got %T", pk)
	}

	return pk, nil
}

// GetConsAddr extracts Consensus key address
func (v Validator) GetConsAddr() ([]byte, error) {
	pk, ok := v.ConsensusPubkey.GetCachedValue().(cryptotypes.PubKey)
	if !ok {
		return nil, errors.Wrapf(sdkerrors.ErrInvalidType, "expecting cryptotypes.PubKey, got %T", pk)
	}

	return pk.Address().Bytes(), nil
}

func (v Validator) GetTokens() math.Int       { return v.Tokens }
func (v Validator) GetBondedTokens() math.Int { return v.BondedTokens() }
func (v Validator) GetConsensusPower(r math.Int) int64 {
	return v.ConsensusPower(r)
}
func (v Validator) GetCommission() math.LegacyDec      { return v.Commission.Rate }
func (v Validator) GetMinSelfDelegation() math.Int     { return v.MinSelfDelegation }
func (v Validator) GetDelegatorShares() math.LegacyDec { return v.DelegatorShares }

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (v Validator) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	var pk cryptotypes.PubKey
	return unpacker.UnpackAny(v.ConsensusPubkey, &pk)
}

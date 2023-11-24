package types

import (
	"encoding/json"
	"strings"
	"time"

	"cosmossdk.io/core/address"
	"cosmossdk.io/math"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// Implements Delegation interface
var _ DelegationI = Delegation{}

// NewDelegation creates a new delegation object
func NewDelegation(delegatorAddr, validatorAddr string, shares math.LegacyDec) Delegation {
	return Delegation{
		DelegatorAddress: delegatorAddr,
		ValidatorAddress: validatorAddr,
		Shares:           shares,
	}
}

// MustMarshalDelegation returns the delegation bytes. Panics if fails
func MustMarshalDelegation(cdc codec.BinaryCodec, delegation Delegation) []byte {
	return cdc.MustMarshal(&delegation)
}

// MustUnmarshalDelegation return the unmarshaled delegation from bytes.
// Panics if fails.
func MustUnmarshalDelegation(cdc codec.BinaryCodec, value []byte) Delegation {
	delegation, err := UnmarshalDelegation(cdc, value)
	if err != nil {
		panic(err)
	}

	return delegation
}

// return the delegation
func UnmarshalDelegation(cdc codec.BinaryCodec, value []byte) (delegation Delegation, err error) {
	err = cdc.Unmarshal(value, &delegation)
	return delegation, err
}

func (d Delegation) GetDelegatorAddr() string {
	return d.DelegatorAddress
}

func (d Delegation) GetValidatorAddr() string {
	return d.ValidatorAddress
}
func (d Delegation) GetShares() math.LegacyDec { return d.Shares }

// Delegations is a collection of delegations
type Delegations []Delegation

func (d Delegations) String() (out string) {
	for _, del := range d {
		out += del.String() + "\n"
	}

	return strings.TrimSpace(out)
}

func NewUnbondingDelegationEntry(creationHeight int64, completionTime time.Time, balance math.Int, unbondingID uint64) UnbondingDelegationEntry {
	return UnbondingDelegationEntry{
		CreationHeight:          creationHeight,
		CompletionTime:          completionTime,
		InitialBalance:          balance,
		Balance:                 balance,
		UnbondingId:             unbondingID,
		UnbondingOnHoldRefCount: 0,
	}
}

// IsMature - is the current entry mature
func (e UnbondingDelegationEntry) IsMature(currentTime time.Time) bool {
	return !e.CompletionTime.After(currentTime)
}

// OnHold - is the current entry on hold due to external modules
func (e UnbondingDelegationEntry) OnHold() bool {
	return e.UnbondingOnHoldRefCount > 0
}

// return the unbonding delegation entry
func MustMarshalUBDE(cdc codec.BinaryCodec, ubd UnbondingDelegationEntry) []byte {
	return cdc.MustMarshal(&ubd)
}

// unmarshal a unbonding delegation entry from a store value
func MustUnmarshalUBDE(cdc codec.BinaryCodec, value []byte) UnbondingDelegationEntry {
	ubd, err := UnmarshalUBDE(cdc, value)
	if err != nil {
		panic(err)
	}

	return ubd
}

// unmarshal a unbonding delegation entry from a store value
func UnmarshalUBDE(cdc codec.BinaryCodec, value []byte) (ubd UnbondingDelegationEntry, err error) {
	err = cdc.Unmarshal(value, &ubd)
	return ubd, err
}

// NewUnbondingDelegation - create a new unbonding delegation object
func NewUnbondingDelegation(
	delegatorAddr sdk.AccAddress, validatorAddr sdk.ValAddress,
	creationHeight int64, minTime time.Time, balance math.Int, id uint64,
	valAc, delAc address.Codec,
) UnbondingDelegation {
	valAddr, err := valAc.BytesToString(validatorAddr)
	if err != nil {
		panic(err)
	}
	delAddr, err := delAc.BytesToString(delegatorAddr)
	if err != nil {
		panic(err)
	}
	return UnbondingDelegation{
		DelegatorAddress: delAddr,
		ValidatorAddress: valAddr,
		Entries: []UnbondingDelegationEntry{
			NewUnbondingDelegationEntry(creationHeight, minTime, balance, id),
		},
	}
}

// AddEntry - append entry to the unbonding delegation
func (ubd *UnbondingDelegation) AddEntry(creationHeight int64, minTime time.Time, balance math.Int, unbondingID uint64) bool {
	// Check the entries exists with creation_height and complete_time
	entryIndex := -1
	for index, ubdEntry := range ubd.Entries {
		if ubdEntry.CreationHeight == creationHeight && ubdEntry.CompletionTime.Equal(minTime) {
			entryIndex = index
			break
		}
	}
	// entryIndex exists
	if entryIndex != -1 {
		ubdEntry := ubd.Entries[entryIndex]
		ubdEntry.Balance = ubdEntry.Balance.Add(balance)
		ubdEntry.InitialBalance = ubdEntry.InitialBalance.Add(balance)

		// update the entry
		ubd.Entries[entryIndex] = ubdEntry
		return false
	}
	// append the new unbond delegation entry
	entry := NewUnbondingDelegationEntry(creationHeight, minTime, balance, unbondingID)
	ubd.Entries = append(ubd.Entries, entry)
	return true
}

// RemoveEntry - remove entry at index i to the unbonding delegation
func (ubd *UnbondingDelegation) RemoveEntry(i int64) {
	ubd.Entries = append(ubd.Entries[:i], ubd.Entries[i+1:]...)
}

// return the unbonding delegation
func MustMarshalUBD(cdc codec.BinaryCodec, ubd UnbondingDelegation) []byte {
	return cdc.MustMarshal(&ubd)
}

// unmarshal a unbonding delegation from a store value
func MustUnmarshalUBD(cdc codec.BinaryCodec, value []byte) UnbondingDelegation {
	ubd, err := UnmarshalUBD(cdc, value)
	if err != nil {
		panic(err)
	}

	return ubd
}

// unmarshal a unbonding delegation from a store value
func UnmarshalUBD(cdc codec.BinaryCodec, value []byte) (ubd UnbondingDelegation, err error) {
	err = cdc.Unmarshal(value, &ubd)
	return ubd, err
}

// UnbondingDelegations is a collection of UnbondingDelegation
type UnbondingDelegations []UnbondingDelegation

func (ubds UnbondingDelegations) String() (out string) {
	for _, u := range ubds {
		out += u.String() + "\n"
	}

	return strings.TrimSpace(out)
}

func NewRedelegationEntry(creationHeight int64, completionTime time.Time, balance math.Int, sharesDst math.LegacyDec, id uint64) RedelegationEntry {
	return RedelegationEntry{
		CreationHeight:          creationHeight,
		CompletionTime:          completionTime,
		InitialBalance:          balance,
		SharesDst:               sharesDst,
		UnbondingId:             id,
		UnbondingOnHoldRefCount: 0,
	}
}

// IsMature - is the current entry mature
func (e RedelegationEntry) IsMature(currentTime time.Time) bool {
	return !e.CompletionTime.After(currentTime)
}

// OnHold - is the current entry on hold due to external modules
func (e RedelegationEntry) OnHold() bool {
	return e.UnbondingOnHoldRefCount > 0
}

func NewRedelegation(
	delegatorAddr sdk.AccAddress, validatorSrcAddr, validatorDstAddr sdk.ValAddress,
	creationHeight int64, minTime time.Time, balance math.Int, sharesDst math.LegacyDec, id uint64,
	valAc, delAc address.Codec,
) Redelegation {
	valSrcAddr, err := valAc.BytesToString(validatorSrcAddr)
	if err != nil {
		panic(err)
	}
	valDstAddr, err := valAc.BytesToString(validatorDstAddr)
	if err != nil {
		panic(err)
	}
	delAddr, err := delAc.BytesToString(delegatorAddr)
	if err != nil {
		panic(err)
	}

	return Redelegation{
		DelegatorAddress:    delAddr,
		ValidatorSrcAddress: valSrcAddr,
		ValidatorDstAddress: valDstAddr,
		Entries: []RedelegationEntry{
			NewRedelegationEntry(creationHeight, minTime, balance, sharesDst, id),
		},
	}
}

// AddEntry - append entry to the unbonding delegation
func (red *Redelegation) AddEntry(creationHeight int64, minTime time.Time, balance math.Int, sharesDst math.LegacyDec, id uint64) {
	entry := NewRedelegationEntry(creationHeight, minTime, balance, sharesDst, id)
	red.Entries = append(red.Entries, entry)
}

// RemoveEntry - remove entry at index i to the unbonding delegation
func (red *Redelegation) RemoveEntry(i int64) {
	red.Entries = append(red.Entries[:i], red.Entries[i+1:]...)
}

// MustMarshalRED returns the Redelegation bytes. Panics if fails.
func MustMarshalRED(cdc codec.BinaryCodec, red Redelegation) []byte {
	return cdc.MustMarshal(&red)
}

// MustUnmarshalRED unmarshals a redelegation from a store value. Panics if fails.
func MustUnmarshalRED(cdc codec.BinaryCodec, value []byte) Redelegation {
	red, err := UnmarshalRED(cdc, value)
	if err != nil {
		panic(err)
	}

	return red
}

// UnmarshalRED unmarshals a redelegation from a store value
func UnmarshalRED(cdc codec.BinaryCodec, value []byte) (red Redelegation, err error) {
	err = cdc.Unmarshal(value, &red)
	return red, err
}

// Redelegations are a collection of Redelegation
type Redelegations []Redelegation

func (d Redelegations) String() (out string) {
	for _, red := range d {
		out += red.String() + "\n"
	}

	return strings.TrimSpace(out)
}

// ----------------------------------------------------------------------------
// Client Types

// NewDelegationResp creates a new DelegationResponse instance
func NewDelegationResp(
	delegatorAddr, validatorAddr string, shares math.LegacyDec, balance sdk.Coin,
) DelegationResponse {
	return DelegationResponse{
		Delegation: NewDelegation(delegatorAddr, validatorAddr, shares),
		Balance:    balance,
	}
}

type delegationRespAlias DelegationResponse

// MarshalJSON implements the json.Marshaler interface. This is so we can
// achieve a flattened structure while embedding other types.
func (d DelegationResponse) MarshalJSON() ([]byte, error) {
	return json.Marshal((delegationRespAlias)(d))
}

// UnmarshalJSON implements the json.Unmarshaler interface. This is so we can
// achieve a flattened structure while embedding other types.
func (d *DelegationResponse) UnmarshalJSON(bz []byte) error {
	return json.Unmarshal(bz, (*delegationRespAlias)(d))
}

// DelegationResponses is a collection of DelegationResp
type DelegationResponses []DelegationResponse

// String implements the Stringer interface for DelegationResponses.
func (d DelegationResponses) String() (out string) {
	for _, del := range d {
		out += del.String() + "\n"
	}

	return strings.TrimSpace(out)
}

// NewRedelegationResponse crates a new RedelegationEntryResponse instance.
func NewRedelegationResponse(
	delegatorAddr, validatorSrc, validatorDst string, entries []RedelegationEntryResponse,
) RedelegationResponse {
	return RedelegationResponse{
		Redelegation: Redelegation{
			DelegatorAddress:    delegatorAddr,
			ValidatorSrcAddress: validatorSrc,
			ValidatorDstAddress: validatorDst,
		},
		Entries: entries,
	}
}

// NewRedelegationEntryResponse creates a new RedelegationEntryResponse instance.
func NewRedelegationEntryResponse(
	creationHeight int64, completionTime time.Time, sharesDst math.LegacyDec, initialBalance, balance math.Int, unbondingID uint64,
) RedelegationEntryResponse {
	return RedelegationEntryResponse{
		RedelegationEntry: NewRedelegationEntry(creationHeight, completionTime, initialBalance, sharesDst, unbondingID),
		Balance:           balance,
	}
}

type redelegationRespAlias RedelegationResponse

// MarshalJSON implements the json.Marshaler interface. This is so we can
// achieve a flattened structure while embedding other types.
func (r RedelegationResponse) MarshalJSON() ([]byte, error) {
	return json.Marshal((redelegationRespAlias)(r))
}

// UnmarshalJSON implements the json.Unmarshaler interface. This is so we can
// achieve a flattened structure while embedding other types.
func (r *RedelegationResponse) UnmarshalJSON(bz []byte) error {
	return json.Unmarshal(bz, (*redelegationRespAlias)(r))
}

// RedelegationResponses are a collection of RedelegationResp
type RedelegationResponses []RedelegationResponse

func (r RedelegationResponses) String() (out string) {
	for _, red := range r {
		out += red.String() + "\n"
	}

	return strings.TrimSpace(out)
}

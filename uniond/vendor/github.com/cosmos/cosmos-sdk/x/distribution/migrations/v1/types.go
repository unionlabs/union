package legacy

import (
	"encoding/binary"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/kv"
	v1auth "github.com/cosmos/cosmos-sdk/x/auth/migrations/v1"
)

const (
	// ModuleName is the module name constant used in many places
	ModuleName = "distribution"

	// StoreKey is the store key string for distribution
	StoreKey = ModuleName

	// RouterKey is the message route for distribution
	RouterKey = ModuleName

	// QuerierRoute is the querier route for distribution
	QuerierRoute = ModuleName
)

// Keys for distribution store
// Items are stored with the following key: values
//
// - 0x00<proposalID_Bytes>: FeePol
//
// - 0x01: sdk.ConsAddress
//
// - 0x02<valAddr_Bytes>: ValidatorOutstandingRewards
//
// - 0x03<accAddr_Bytes>: sdk.AccAddress
//
// - 0x04<valAddr_Bytes><accAddr_Bytes>: DelegatorStartingInfo
//
// - 0x05<valAddr_Bytes><period_Bytes>: ValidatorHistoricalRewards
//
// - 0x06<valAddr_Bytes>: ValidatorCurrentRewards
//
// - 0x07<valAddr_Bytes>: ValidatorCurrentRewards
//
// - 0x08<valAddr_Bytes><height>: ValidatorSlashEvent
var (
	FeePoolKey                        = []byte{0x00} // key for global distribution state
	ProposerKey                       = []byte{0x01} // key for the proposer operator address
	ValidatorOutstandingRewardsPrefix = []byte{0x02} // key for outstanding rewards

	DelegatorWithdrawAddrPrefix          = []byte{0x03} // key for delegator withdraw address
	DelegatorStartingInfoPrefix          = []byte{0x04} // key for delegator starting info
	ValidatorHistoricalRewardsPrefix     = []byte{0x05} // key for historical validators rewards / stake
	ValidatorCurrentRewardsPrefix        = []byte{0x06} // key for current validator rewards
	ValidatorAccumulatedCommissionPrefix = []byte{0x07} // key for accumulated validator commission
	ValidatorSlashEventPrefix            = []byte{0x08} // key for validator slash fraction
)

// gets an address from a validator's outstanding rewards key
func GetValidatorOutstandingRewardsAddress(key []byte) (valAddr sdk.ValAddress) {
	kv.AssertKeyAtLeastLength(key, 2)
	addr := key[1:]
	kv.AssertKeyLength(addr, v1auth.AddrLen)
	return sdk.ValAddress(addr)
}

// gets an address from a delegator's withdraw info key
func GetDelegatorWithdrawInfoAddress(key []byte) (delAddr sdk.AccAddress) {
	kv.AssertKeyAtLeastLength(key, 2)
	addr := key[1:]
	kv.AssertKeyLength(addr, v1auth.AddrLen)
	return sdk.AccAddress(addr)
}

// gets the addresses from a delegator starting info key
func GetDelegatorStartingInfoAddresses(key []byte) (valAddr sdk.ValAddress, delAddr sdk.AccAddress) {
	kv.AssertKeyAtLeastLength(key, 2+v1auth.AddrLen)
	addr := key[1 : 1+v1auth.AddrLen]
	kv.AssertKeyLength(addr, v1auth.AddrLen)
	valAddr = sdk.ValAddress(addr)
	addr = key[1+v1auth.AddrLen:]
	kv.AssertKeyLength(addr, v1auth.AddrLen)
	delAddr = sdk.AccAddress(addr)
	return
}

// gets the address & period from a validator's historical rewards key
func GetValidatorHistoricalRewardsAddressPeriod(key []byte) (valAddr sdk.ValAddress, period uint64) {
	kv.AssertKeyAtLeastLength(key, 2+v1auth.AddrLen)
	addr := key[1 : 1+v1auth.AddrLen]
	kv.AssertKeyLength(addr, v1auth.AddrLen)
	valAddr = sdk.ValAddress(addr)
	b := key[1+v1auth.AddrLen:]
	kv.AssertKeyLength(addr, 8)
	period = binary.LittleEndian.Uint64(b)
	return
}

// gets the address from a validator's current rewards key
func GetValidatorCurrentRewardsAddress(key []byte) (valAddr sdk.ValAddress) {
	kv.AssertKeyAtLeastLength(key, 2)
	addr := key[1:]
	kv.AssertKeyLength(addr, v1auth.AddrLen)
	return sdk.ValAddress(addr)
}

// gets the address from a validator's accumulated commission key
func GetValidatorAccumulatedCommissionAddress(key []byte) (valAddr sdk.ValAddress) {
	kv.AssertKeyAtLeastLength(key, 2)
	addr := key[1:]
	kv.AssertKeyLength(addr, v1auth.AddrLen)
	return sdk.ValAddress(addr)
}

// gets the height from a validator's slash event key
func GetValidatorSlashEventAddressHeight(key []byte) (valAddr sdk.ValAddress, height uint64) {
	kv.AssertKeyAtLeastLength(key, 2+v1auth.AddrLen)
	addr := key[1 : 1+v1auth.AddrLen]
	kv.AssertKeyLength(addr, v1auth.AddrLen)
	valAddr = sdk.ValAddress(addr)
	startB := 1 + v1auth.AddrLen
	kv.AssertKeyAtLeastLength(key, startB+9)
	b := key[startB : startB+8] // the next 8 bytes represent the height
	height = binary.BigEndian.Uint64(b)
	return
}

// gets the outstanding rewards key for a validator
func GetValidatorOutstandingRewardsKey(valAddr sdk.ValAddress) []byte {
	return append(ValidatorOutstandingRewardsPrefix, valAddr.Bytes()...)
}

// gets the key for a delegator's withdraw addr
func GetDelegatorWithdrawAddrKey(delAddr sdk.AccAddress) []byte {
	return append(DelegatorWithdrawAddrPrefix, delAddr.Bytes()...)
}

// gets the key for a delegator's starting info
func GetDelegatorStartingInfoKey(v sdk.ValAddress, d sdk.AccAddress) []byte {
	return append(append(DelegatorStartingInfoPrefix, v.Bytes()...), d.Bytes()...)
}

// gets the prefix key for a validator's historical rewards
func GetValidatorHistoricalRewardsPrefix(v sdk.ValAddress) []byte {
	return append(ValidatorHistoricalRewardsPrefix, v.Bytes()...)
}

// gets the key for a validator's historical rewards
func GetValidatorHistoricalRewardsKey(v sdk.ValAddress, k uint64) []byte {
	b := make([]byte, 8)
	binary.LittleEndian.PutUint64(b, k)
	return append(append(ValidatorHistoricalRewardsPrefix, v.Bytes()...), b...)
}

// gets the key for a validator's current rewards
func GetValidatorCurrentRewardsKey(v sdk.ValAddress) []byte {
	return append(ValidatorCurrentRewardsPrefix, v.Bytes()...)
}

// gets the key for a validator's current commission
func GetValidatorAccumulatedCommissionKey(v sdk.ValAddress) []byte {
	return append(ValidatorAccumulatedCommissionPrefix, v.Bytes()...)
}

// gets the prefix key for a validator's slash fractions
func GetValidatorSlashEventPrefix(v sdk.ValAddress) []byte {
	return append(ValidatorSlashEventPrefix, v.Bytes()...)
}

// gets the prefix key for a validator's slash fraction (ValidatorSlashEventPrefix + height)
func GetValidatorSlashEventKeyPrefix(v sdk.ValAddress, height uint64) []byte {
	heightBz := make([]byte, 8)
	binary.BigEndian.PutUint64(heightBz, height)
	return append(
		ValidatorSlashEventPrefix,
		append(v.Bytes(), heightBz...)...,
	)
}

// gets the key for a validator's slash fraction
func GetValidatorSlashEventKey(v sdk.ValAddress, height, period uint64) []byte {
	periodBz := make([]byte, 8)
	binary.BigEndian.PutUint64(periodBz, period)
	prefix := GetValidatorSlashEventKeyPrefix(v, height)
	return append(prefix, periodBz...)
}

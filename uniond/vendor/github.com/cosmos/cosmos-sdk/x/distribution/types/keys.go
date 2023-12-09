package types

import (
	"encoding/binary"

	"cosmossdk.io/collections"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	"github.com/cosmos/cosmos-sdk/types/kv"
)

const (
	// ModuleName is the module name constant used in many places
	ModuleName = "distribution"

	// StoreKey is the store key string for distribution
	StoreKey = ModuleName

	// RouterKey is the message route for distribution
	RouterKey = ModuleName
)

// Keys for distribution store
// Items are stored with the following key: values
//
// - 0x00<proposalID_Bytes>: FeePol
//
// - 0x01: sdk.ConsAddress
//
// - 0x02<valAddrLen (1 Byte)><valAddr_Bytes>: ValidatorOutstandingRewards
//
// - 0x03<accAddrLen (1 Byte)><accAddr_Bytes>: sdk.AccAddress
//
// - 0x04<valAddrLen (1 Byte)><valAddr_Bytes><accAddrLen (1 Byte)><accAddr_Bytes>: DelegatorStartingInfo
//
// - 0x05<valAddrLen (1 Byte)><valAddr_Bytes><period_Bytes>: ValidatorHistoricalRewards
//
// - 0x06<valAddrLen (1 Byte)><valAddr_Bytes>: ValidatorCurrentRewards
//
// - 0x07<valAddrLen (1 Byte)><valAddr_Bytes>: ValidatorCurrentCommission
//
// - 0x08<valAddrLen (1 Byte)><valAddr_Bytes><height>: ValidatorSlashEvent
//
// - 0x09: Params
var (
	FeePoolKey                        = collections.NewPrefix(0) // key for global distribution state
	ProposerKey                       = []byte{0x01}             // key for the proposer operator address
	ValidatorOutstandingRewardsPrefix = []byte{0x02}             // key for outstanding rewards

	DelegatorWithdrawAddrPrefix          = []byte{0x03} // key for delegator withdraw address
	DelegatorStartingInfoPrefix          = []byte{0x04} // key for delegator starting info
	ValidatorHistoricalRewardsPrefix     = []byte{0x05} // key for historical validators rewards / stake
	ValidatorCurrentRewardsPrefix        = []byte{0x06} // key for current validator rewards
	ValidatorAccumulatedCommissionPrefix = []byte{0x07} // key for accumulated validator commission
	ValidatorSlashEventPrefix            = []byte{0x08} // key for validator slash fraction

	ParamsKey = collections.NewPrefix(9) // key for distribution module params
)

// GetValidatorOutstandingRewardsAddress creates an address from a validator's outstanding rewards key.
func GetValidatorOutstandingRewardsAddress(key []byte) (valAddr sdk.ValAddress) {
	// key is in the format:
	// 0x02<valAddrLen (1 Byte)><valAddr_Bytes>

	// Remove prefix and address length.
	kv.AssertKeyAtLeastLength(key, 3)
	addr := key[2:]
	kv.AssertKeyLength(addr, int(key[1]))

	return sdk.ValAddress(addr)
}

// GetDelegatorWithdrawInfoAddress creates an address from a delegator's withdraw info key.
func GetDelegatorWithdrawInfoAddress(key []byte) (delAddr sdk.AccAddress) {
	// key is in the format:
	// 0x03<accAddrLen (1 Byte)><accAddr_Bytes>

	// Remove prefix and address length.
	kv.AssertKeyAtLeastLength(key, 3)
	addr := key[2:]
	kv.AssertKeyLength(addr, int(key[1]))

	return sdk.AccAddress(addr)
}

// GetDelegatorStartingInfoAddresses creates the addresses from a delegator starting info key.
func GetDelegatorStartingInfoAddresses(key []byte) (valAddr sdk.ValAddress, delAddr sdk.AccAddress) {
	// key is in the format:
	// 0x04<valAddrLen (1 Byte)><valAddr_Bytes><accAddrLen (1 Byte)><accAddr_Bytes>
	kv.AssertKeyAtLeastLength(key, 2)
	valAddrLen := int(key[1])
	kv.AssertKeyAtLeastLength(key, 3+valAddrLen)
	valAddr = sdk.ValAddress(key[2 : 2+valAddrLen])
	delAddrLen := int(key[2+valAddrLen])
	kv.AssertKeyAtLeastLength(key, 4+valAddrLen)
	delAddr = sdk.AccAddress(key[3+valAddrLen:])
	kv.AssertKeyLength(delAddr.Bytes(), delAddrLen)

	return
}

// GetValidatorHistoricalRewardsAddressPeriod creates the address & period from a validator's historical rewards key.
func GetValidatorHistoricalRewardsAddressPeriod(key []byte) (valAddr sdk.ValAddress, period uint64) {
	// key is in the format:
	// 0x05<valAddrLen (1 Byte)><valAddr_Bytes><period_Bytes>
	kv.AssertKeyAtLeastLength(key, 2)
	valAddrLen := int(key[1])
	kv.AssertKeyAtLeastLength(key, 3+valAddrLen)
	valAddr = sdk.ValAddress(key[2 : 2+valAddrLen])
	b := key[2+valAddrLen:]
	kv.AssertKeyLength(b, 8)
	period = binary.LittleEndian.Uint64(b)
	return
}

// GetValidatorCurrentRewardsAddress creates the address from a validator's current rewards key.
func GetValidatorCurrentRewardsAddress(key []byte) (valAddr sdk.ValAddress) {
	// key is in the format:
	// 0x06<valAddrLen (1 Byte)><valAddr_Bytes>: ValidatorCurrentRewards

	// Remove prefix and address length.
	kv.AssertKeyAtLeastLength(key, 3)
	addr := key[2:]
	kv.AssertKeyLength(addr, int(key[1]))

	return sdk.ValAddress(addr)
}

// GetValidatorAccumulatedCommissionAddress creates the address from a validator's accumulated commission key.
func GetValidatorAccumulatedCommissionAddress(key []byte) (valAddr sdk.ValAddress) {
	// key is in the format:
	// 0x07<valAddrLen (1 Byte)><valAddr_Bytes>: ValidatorCurrentRewards

	// Remove prefix and address length.
	kv.AssertKeyAtLeastLength(key, 3)
	addr := key[2:]
	kv.AssertKeyLength(addr, int(key[1]))

	return sdk.ValAddress(addr)
}

// GetValidatorSlashEventAddressHeight creates the height from a validator's slash event key.
func GetValidatorSlashEventAddressHeight(key []byte) (valAddr sdk.ValAddress, height uint64) {
	// key is in the format:
	// 0x08<valAddrLen (1 Byte)><valAddr_Bytes><height>: ValidatorSlashEvent
	kv.AssertKeyAtLeastLength(key, 2)
	valAddrLen := int(key[1])
	kv.AssertKeyAtLeastLength(key, 3+valAddrLen)
	valAddr = key[2 : 2+valAddrLen]
	startB := 2 + valAddrLen
	kv.AssertKeyAtLeastLength(key, startB+9)
	b := key[startB : startB+8] // the next 8 bytes represent the height
	height = binary.BigEndian.Uint64(b)
	return
}

// GetValidatorOutstandingRewardsKey creates the outstanding rewards key for a validator.
func GetValidatorOutstandingRewardsKey(valAddr sdk.ValAddress) []byte {
	return append(ValidatorOutstandingRewardsPrefix, address.MustLengthPrefix(valAddr.Bytes())...)
}

// GetDelegatorWithdrawAddrKey creates the key for a delegator's withdraw addr.
func GetDelegatorWithdrawAddrKey(delAddr sdk.AccAddress) []byte {
	return append(DelegatorWithdrawAddrPrefix, address.MustLengthPrefix(delAddr.Bytes())...)
}

// GetDelegatorStartingInfoKey creates the key for a delegator's starting info.
func GetDelegatorStartingInfoKey(v sdk.ValAddress, d sdk.AccAddress) []byte {
	return append(append(DelegatorStartingInfoPrefix, address.MustLengthPrefix(v.Bytes())...), address.MustLengthPrefix(d.Bytes())...)
}

// GetValidatorHistoricalRewardsPrefix creates the prefix key for a validator's historical rewards.
func GetValidatorHistoricalRewardsPrefix(v sdk.ValAddress) []byte {
	return append(ValidatorHistoricalRewardsPrefix, address.MustLengthPrefix(v.Bytes())...)
}

// GetValidatorHistoricalRewardsKey creates the key for a validator's historical rewards.
func GetValidatorHistoricalRewardsKey(v sdk.ValAddress, k uint64) []byte {
	b := make([]byte, 8)
	binary.LittleEndian.PutUint64(b, k)
	return append(append(ValidatorHistoricalRewardsPrefix, address.MustLengthPrefix(v.Bytes())...), b...)
}

// GetValidatorCurrentRewardsKey creates the key for a validator's current rewards.
func GetValidatorCurrentRewardsKey(v sdk.ValAddress) []byte {
	return append(ValidatorCurrentRewardsPrefix, address.MustLengthPrefix(v.Bytes())...)
}

// GetValidatorAccumulatedCommissionKey creates the key for a validator's current commission.
func GetValidatorAccumulatedCommissionKey(v sdk.ValAddress) []byte {
	return append(ValidatorAccumulatedCommissionPrefix, address.MustLengthPrefix(v.Bytes())...)
}

// GetValidatorSlashEventPrefix creates the prefix key for a validator's slash fractions.
func GetValidatorSlashEventPrefix(v sdk.ValAddress) []byte {
	return append(ValidatorSlashEventPrefix, address.MustLengthPrefix(v.Bytes())...)
}

// GetValidatorSlashEventKeyPrefix creates the prefix key for a validator's slash fraction (ValidatorSlashEventPrefix + height).
func GetValidatorSlashEventKeyPrefix(v sdk.ValAddress, height uint64) []byte {
	heightBz := make([]byte, 8)
	binary.BigEndian.PutUint64(heightBz, height)

	return append(
		ValidatorSlashEventPrefix,
		append(address.MustLengthPrefix(v.Bytes()), heightBz...)...,
	)
}

// GetValidatorSlashEventKey creates the key for a validator's slash fraction.
func GetValidatorSlashEventKey(v sdk.ValAddress, height, period uint64) []byte {
	periodBz := make([]byte, 8)
	binary.BigEndian.PutUint64(periodBz, period)
	prefix := GetValidatorSlashEventKeyPrefix(v, height)

	return append(prefix, periodBz...)
}

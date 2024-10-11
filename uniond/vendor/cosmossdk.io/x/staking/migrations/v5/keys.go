package v5

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"strconv"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
)

const (
	// ModuleName is the name of the module
	ModuleName = "staking"
)

var (
	DelegationKey           = []byte{0x31} // key for a delegation
	HistoricalInfoKey       = []byte{0x50} // prefix for the historical info
	DelegationByValIndexKey = []byte{0x71} // key for delegations by a validator

)

// ParseDelegationKey parses given key and returns delagator, validator address bytes
func ParseDelegationKey(bz []byte) (sdk.AccAddress, sdk.ValAddress, error) {
	prefixLength := len(DelegationKey)
	if prefix := bz[:prefixLength]; !bytes.Equal(prefix, DelegationKey) {
		return nil, nil, fmt.Errorf("invalid prefix; expected: %X, got: %x", DelegationKey, prefix)
	}

	bz = bz[prefixLength:] // remove the prefix byte
	if len(bz) == 0 {
		return nil, nil, fmt.Errorf("no bytes left to parse: %X", bz)
	}

	delAddrLen := bz[0]
	bz = bz[1:] // remove the length byte of delegator address.
	if len(bz) == 0 {
		return nil, nil, fmt.Errorf("no bytes left to parse delegator address: %X", bz)
	}

	del := bz[:int(delAddrLen)]
	bz = bz[int(delAddrLen):] // remove the length byte of a delegator address
	if len(bz) == 0 {
		return nil, nil, fmt.Errorf("no bytes left to parse delegator address: %X", bz)
	}

	bz = bz[1:] // remove the validator address bytes.
	if len(bz) == 0 {
		return nil, nil, fmt.Errorf("no bytes left to parse validator address: %X", bz)
	}

	val := bz

	return del, val, nil
}

// GetHistoricalInfoKey returns a key prefix for indexing HistoricalInfo objects.
func GetHistoricalInfoKey(height int64) []byte {
	heightBytes := make([]byte, 8)
	binary.BigEndian.PutUint64(heightBytes, uint64(height))
	return append(HistoricalInfoKey, heightBytes...)
}

// GetHistoricalInfoKey returns a key prefix for indexing HistoricalInfo objects.
func GetLegacyHistoricalInfoKey(height int64) []byte {
	return append(HistoricalInfoKey, []byte(strconv.FormatInt(height, 10))...)
}

// GetDelegationsByValPrefixKey builds a prefix key bytes with the given validator address bytes.
func GetDelegationsByValPrefixKey(valAddr sdk.ValAddress) []byte {
	return append(DelegationByValIndexKey, address.MustLengthPrefix(valAddr)...)
}

// GetDelegationsByValKey creates the key for delegations by validator address
// VALUE: staking/Delegation
func GetDelegationsByValKey(valAddr sdk.ValAddress, delAddr sdk.AccAddress) []byte {
	return append(GetDelegationsByValPrefixKey(valAddr), delAddr...)
}

// ParseDelegationsByValKey parses given key and returns validator, delegator address bytes
func ParseDelegationsByValKey(bz []byte) (sdk.ValAddress, sdk.AccAddress, error) {
	prefixLength := len(DelegationByValIndexKey)
	if prefix := bz[:prefixLength]; !bytes.Equal(prefix, DelegationByValIndexKey) {
		return nil, nil, fmt.Errorf("invalid prefix; expected: %X, got: %x", DelegationByValIndexKey, prefix)
	}

	bz = bz[prefixLength:] // remove the prefix byte
	if len(bz) == 0 {
		return nil, nil, fmt.Errorf("no bytes left to parse: %X", bz)
	}

	valAddrLen := bz[0]
	bz = bz[1:] // remove the length byte of validator address.
	if len(bz) == 0 {
		return nil, nil, fmt.Errorf("no bytes left to parse validator address: %X", bz)
	}

	val := bz[0:int(valAddrLen)]

	bz = bz[int(valAddrLen):] // remove the delegator bytes
	if len(bz) == 0 {
		return nil, nil, fmt.Errorf("no bytes left to parse delegator address: %X", bz)
	}

	del := bz

	return val, del, nil
}

// GetDelegationKey creates the key for delegator bond with validator
// VALUE: staking/Delegation
func GetDelegationKey(delAddr sdk.AccAddress, valAddr sdk.ValAddress) []byte {
	return append(GetDelegationsKey(delAddr), address.MustLengthPrefix(valAddr)...)
}

// GetDelegationsKey creates the prefix for a delegator for all validators
func GetDelegationsKey(delAddr sdk.AccAddress) []byte {
	return append(DelegationKey, address.MustLengthPrefix(delAddr)...)
}

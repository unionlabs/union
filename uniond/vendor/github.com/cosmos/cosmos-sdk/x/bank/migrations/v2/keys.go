package v2

import (
	"errors"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
)

const (
	ModuleName = "bank"
)

var (
	SupplyKey           = []byte{0x00}
	BalancesPrefix      = []byte{0x02}
	DenomMetadataPrefix = []byte{0x1}

	ErrInvalidKey = errors.New("invalid key")
)

func CreateAccountBalancesPrefix(addr []byte) []byte {
	return append(BalancesPrefix, address.MustLengthPrefix(addr)...)
}

func AddressFromBalancesStore(key []byte) (sdk.AccAddress, error) {
	if len(key) == 0 {
		return nil, ErrInvalidKey
	}

	addrLen := key[0]
	bound := int(addrLen)

	if len(key)-1 < bound {
		return nil, ErrInvalidKey
	}

	return key[1 : bound+1], nil
}

// DenomMetadataKey returns the denomination metadata key.
func DenomMetadataKey(denom string) []byte {
	d := []byte(denom)
	return append(DenomMetadataPrefix, d...)
}

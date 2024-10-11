package address

import (
	errorsmod "cosmossdk.io/errors"

	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// MaxAddrLen is the maximum allowed length (in bytes) for an address.
const MaxAddrLen = 255

// LengthPrefix prefixes the address bytes with its length, this is used
// for example for variable-length components in store keys.
func LengthPrefix(bz []byte) ([]byte, error) {
	bzLen := len(bz)
	if bzLen == 0 {
		return bz, nil
	}

	if bzLen > MaxAddrLen {
		return nil, errorsmod.Wrapf(sdkerrors.ErrUnknownAddress, "address length should be max %d bytes, got %d", MaxAddrLen, bzLen)
	}

	return append([]byte{byte(bzLen)}, bz...), nil
}

// MustLengthPrefix is LengthPrefix with panic on error.
func MustLengthPrefix(bz []byte) []byte {
	res, err := LengthPrefix(bz)
	if err != nil {
		panic(err)
	}

	return res
}

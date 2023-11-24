package codec

import (
	"errors"
	"strings"

	"cosmossdk.io/core/address"
	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/bech32"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

type bech32Codec struct {
	bech32Prefix string
}

var _ address.Codec = &bech32Codec{}

func NewBech32Codec(prefix string) address.Codec {
	return bech32Codec{prefix}
}

// StringToBytes encodes text to bytes
func (bc bech32Codec) StringToBytes(text string) ([]byte, error) {
	if len(strings.TrimSpace(text)) == 0 {
		return []byte{}, errors.New("empty address string is not allowed")
	}

	hrp, bz, err := bech32.DecodeAndConvert(text)
	if err != nil {
		return nil, err
	}

	if hrp != bc.bech32Prefix {
		return nil, errorsmod.Wrapf(sdkerrors.ErrLogic, "hrp does not match bech32 prefix: expected '%s' got '%s'", bc.bech32Prefix, hrp)
	}

	if err := sdk.VerifyAddressFormat(bz); err != nil {
		return nil, err
	}

	return bz, nil
}

// BytesToString decodes bytes to text
func (bc bech32Codec) BytesToString(bz []byte) (string, error) {
	text, err := bech32.ConvertAndEncode(bc.bech32Prefix, bz)
	if err != nil {
		return "", err
	}

	return text, nil
}

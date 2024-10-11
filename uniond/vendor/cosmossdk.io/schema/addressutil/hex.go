package addressutil

import (
	"encoding/hex"
	"fmt"
)

// HexAddressCodec is a basic address codec that encodes and decodes addresses as hex strings.
// It is intended to be used as a fallback codec when no other codec is provided.
type HexAddressCodec struct{}

func (h HexAddressCodec) StringToBytes(text string) ([]byte, error) {
	if len(text) < 2 || text[:2] != "0x" {
		return nil, fmt.Errorf("invalid hex address: %s", text)
	}

	return hex.DecodeString(text[2:])
}

func (h HexAddressCodec) BytesToString(bz []byte) (string, error) {
	return fmt.Sprintf("0x%x", bz), nil
}

var _ AddressCodec = HexAddressCodec{}

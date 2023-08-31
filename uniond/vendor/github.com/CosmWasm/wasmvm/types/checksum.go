package types

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
)

// Checksum represents a hash of the Wasm bytecode that serves as an ID. Must be generated from this library.
// The length of a checksum must always be ChecksumLen.
type Checksum []byte

func (cs Checksum) MarshalJSON() ([]byte, error) {
	return json.Marshal(hex.EncodeToString(cs))
}

func (cs *Checksum) UnmarshalJSON(input []byte) error {
	var hexString string
	err := json.Unmarshal(input, &hexString)
	if err != nil {
		return err
	}

	data, err := hex.DecodeString(hexString)
	if err != nil {
		return err
	}
	if len(data) != ChecksumLen {
		return fmt.Errorf("got wrong number of bytes for checksum")
	}
	*cs = Checksum(data)
	return nil
}

const ChecksumLen = 32

// ForceNewChecksum creates a Checksum instance from a hex string.
// It panics in case the input is invalid.
func ForceNewChecksum(input string) Checksum {
	data, err := hex.DecodeString(input)
	if err != nil {
		panic("could not decode hex bytes")
	}
	if len(data) != ChecksumLen {
		panic("got wrong number of bytes")
	}
	return Checksum(data)
}

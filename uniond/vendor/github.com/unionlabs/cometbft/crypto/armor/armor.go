package armor

import (
	"bytes"
	"fmt"
	"io"

	"golang.org/x/crypto/openpgp/armor" //nolint: staticcheck
)

// ErrEncode represents an error from calling [EncodeArmor].
type ErrEncode struct {
	Err error
}

func (e ErrEncode) Error() string {
	return fmt.Sprintf("armor: could not encode ASCII armor: %v", e.Err)
}

func (e ErrEncode) Unwrap() error {
	return e.Err
}

func EncodeArmor(blockType string, headers map[string]string, data []byte) (string, error) {
	buf := new(bytes.Buffer)
	w, err := armor.Encode(buf, blockType, headers)
	if err != nil {
		return "", ErrEncode{Err: err}
	}
	_, err = w.Write(data)
	if err != nil {
		return "", ErrEncode{Err: err}
	}
	err = w.Close()
	if err != nil {
		return "", ErrEncode{Err: err}
	}
	return buf.String(), nil
}

func DecodeArmor(armorStr string) (blockType string, headers map[string]string, data []byte, err error) {
	buf := bytes.NewBufferString(armorStr)
	block, err := armor.Decode(buf)
	if err != nil {
		return "", nil, nil, err
	}
	data, err = io.ReadAll(block.Body)
	if err != nil {
		return "", nil, nil, err
	}
	return block.Type, block.Header, data, nil
}

package offchain

import (
	"encoding/base64"
	"encoding/hex"
	"fmt"
)

const (
	noEncoder  = "no-encoding"
	b64Encoder = "base64"
	hexEncoder = "hex"
)

type encodingFunc = func([]byte) (string, error)

// noEncoding returns a byte slice as a string.
func noEncoding(digest []byte) (string, error) {
	return string(digest), nil
}

// base64Encoding returns a byte slice as a b64 encoded string.
func base64Encoding(digest []byte) (string, error) {
	return base64.StdEncoding.EncodeToString(digest), nil
}

// hexEncoding returns a byte slice as a hex encoded string.
func hexEncoding(digest []byte) (string, error) {
	return hex.EncodeToString(digest), nil
}

// getEncoder returns a encodingFunc bases on the encoder id provided.
func getEncoder(encoder string) (encodingFunc, error) {
	switch encoder {
	case noEncoder:
		return noEncoding, nil
	case b64Encoder:
		return base64Encoding, nil
	case hexEncoder:
		return hexEncoding, nil
	default:
		return nil, fmt.Errorf("unknown encoder: %s", encoder)
	}
}

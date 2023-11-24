package textual

import (
	"bytes"
	"context"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"strings"

	"google.golang.org/protobuf/reflect/protoreflect"
)

var (
	hashPrefix = "SHA-256="
	maxByteLen = 35 // Maximum byte length to be displayed as is. Longer than this, we hash.
)

// NewBytesValueRenderer returns a ValueRenderer for Protobuf bytes, which are
// encoded as capital-letter hexadecimal, without the '0x' prefix.
func NewBytesValueRenderer() ValueRenderer {
	return bytesValueRenderer{}
}

type bytesValueRenderer struct{}

func (vr bytesValueRenderer) Format(_ context.Context, v protoreflect.Value) ([]Screen, error) {
	bz := v.Bytes()

	if len(bz) <= maxByteLen {
		text := toHex(bz)
		return []Screen{{Content: text}}, nil
	}

	// For long bytes, we display the hash.
	hasher := sha256.New()
	_, err := hasher.Write(bz)
	if err != nil {
		return nil, err
	}
	h := hasher.Sum(nil)

	text := fmt.Sprintf("%s%s", hashPrefix, toHex(h))
	return []Screen{{Content: text}}, nil
}

func (vr bytesValueRenderer) Parse(_ context.Context, screens []Screen) (protoreflect.Value, error) {
	if len(screens) != 1 {
		return nilValue, fmt.Errorf("expected single screen: %v", screens)
	}
	formatted := screens[0].Content

	// If the formatted string starts with `SHA-256=`, then we can't actually
	// invert to get the original bytes. In this case, we return empty bytes.
	if strings.HasPrefix(formatted, hashPrefix) {
		return protoreflect.ValueOfBytes([]byte{}), nil
	}

	// Remove all spaces between every 4th char, then we can decode hex.
	data, err := hex.DecodeString(strings.ReplaceAll(formatted, " ", ""))
	if err != nil {
		return nilValue, err
	}

	return protoreflect.ValueOfBytes(data), nil
}

// toHex converts bytes to hex, and inserts a space every 4th character.
func toHex(bz []byte) string {
	text := strings.ToUpper(hex.EncodeToString(bz))

	var buffer bytes.Buffer
	for i, r := range text {
		buffer.WriteRune(r)
		if i < len(text)-1 && i%4 == 3 {
			buffer.WriteRune(' ')
		}
	}
	return buffer.String()
}

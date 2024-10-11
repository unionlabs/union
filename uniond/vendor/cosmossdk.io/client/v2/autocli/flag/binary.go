package flag

import (
	"context"
	"encoding/base64"
	"encoding/hex"
	"errors"
	"os"

	"google.golang.org/protobuf/reflect/protoreflect"
)

type binaryType struct{}

var _ Value = (*fileBinaryValue)(nil)

func (f binaryType) NewValue(*context.Context, *Builder) Value {
	return &fileBinaryValue{}
}

func (f binaryType) DefaultValue() string {
	return ""
}

// fileBinaryValue is a Value that holds a binary file.
type fileBinaryValue struct {
	value []byte
}

func (f *fileBinaryValue) Get(protoreflect.Value) (protoreflect.Value, error) {
	return protoreflect.ValueOfBytes(f.value), nil
}

func (f *fileBinaryValue) String() string {
	return string(f.value)
}

// Set implements the flag.Value interface for binary files, with exceptions.
// If the input string is a valid file path, the value will be the content of that file.
// If the input string is a valid hex or base64 string, the value will be the decoded form of that string.
// If the input string is not a valid file path, hex string, or base64 string, Set will return an error.
func (f *fileBinaryValue) Set(s string) error {
	if data, err := os.ReadFile(s); err == nil {
		f.value = data
		return nil
	}

	if data, err := hex.DecodeString(s); err == nil {
		f.value = data
		return nil
	}

	if data, err := base64.StdEncoding.DecodeString(s); err == nil {
		f.value = data
		return nil
	}

	return errors.New("input string is neither a valid file path, hex, or base64 encoded")
}

func (f *fileBinaryValue) Type() string {
	return "binary"
}

package orm

import (
	"fmt"

	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/x/group/errors"
)

// MaxBytesLen is the maximum allowed length for a key part of type []byte
const MaxBytesLen = 255

// buildKeyFromParts encodes and concatenates primary key and index parts.
// They can be []byte, string, and integer types. The function will return
// an error if there is a part of any other type.
// Key parts, except the last part, follow these rules:
//   - []byte is encoded with a single byte length prefix
//   - strings are null-terminated
//   - integers are encoded using 8 byte big endian.
func buildKeyFromParts(parts []interface{}) ([]byte, error) {
	bytesSlice := make([][]byte, len(parts))
	totalLen := 0
	var err error
	for i, part := range parts {
		bytesSlice[i], err = keyPartBytes(part, len(parts) > 1 && i == len(parts)-1)
		if err != nil {
			return nil, err
		}
		totalLen += len(bytesSlice[i])
	}
	key := make([]byte, 0, totalLen)
	for _, bs := range bytesSlice {
		key = append(key, bs...)
	}
	return key, nil
}

func keyPartBytes(part interface{}, last bool) ([]byte, error) {
	switch v := part.(type) {
	case []byte:
		if last || len(v) == 0 {
			return v, nil
		}
		return AddLengthPrefix(v), nil
	case string:
		if last || len(v) == 0 {
			return []byte(v), nil
		}
		return NullTerminatedBytes(v), nil
	case uint64:
		return EncodeSequence(v), nil
	default:
		return nil, fmt.Errorf("type %T not allowed as key part", v)
	}
}

// AddLengthPrefix prefixes the byte array with its length as 8 bytes. The function will panic
// if the bytes length is bigger than 255.
func AddLengthPrefix(bytes []byte) []byte {
	byteLen := len(bytes)
	if byteLen > MaxBytesLen {
		panic(errorsmod.Wrap(errors.ErrORMKeyMaxLength, "Cannot create key part with an []byte of length greater than 255 bytes. Try again with a smaller []byte."))
	}

	prefixedBytes := make([]byte, 1+len(bytes))
	copy(prefixedBytes, []byte{uint8(byteLen)})
	copy(prefixedBytes[1:], bytes)
	return prefixedBytes
}

// NullTerminatedBytes converts string to byte array and null terminate it
func NullTerminatedBytes(s string) []byte {
	bytes := make([]byte, len(s)+1)
	copy(bytes, s)
	return bytes
}

// stripRowID returns the RowID from the indexKey based on secondaryIndexKey type.
// It is the reverse operation to buildKeyFromParts for index keys
// where the first part is the encoded secondaryIndexKey and the second part is the RowID.
func stripRowID(indexKey []byte, secondaryIndexKey interface{}) (RowID, error) {
	switch v := secondaryIndexKey.(type) {
	case []byte:
		searchableKeyLen := indexKey[0]
		return indexKey[1+searchableKeyLen:], nil
	case string:
		searchableKeyLen := 0
		for i, b := range indexKey {
			if b == 0 {
				searchableKeyLen = i
				break
			}
		}
		return indexKey[1+searchableKeyLen:], nil
	case uint64:
		return indexKey[EncodedSeqLength:], nil
	default:
		return nil, fmt.Errorf("type %T not allowed as index key", v)
	}
}

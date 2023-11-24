package types

import (
	"encoding/binary"
	"encoding/json"
	"fmt"
	"time"

	"cosmossdk.io/log"

	"github.com/cosmos/cosmos-sdk/types/kv"
)

// SortedJSON takes any JSON and returns it sorted by keys. Also, all white-spaces
// are removed.
// This method can be used to canonicalize JSON to be returned by GetSignBytes,
// e.g. for the ledger integration.
// If the passed JSON isn't valid it will return an error.
// Deprecated: SortJSON was used for GetSignbytes, this is now automatic with amino signing
func SortJSON(toSortJSON []byte) ([]byte, error) {
	var c interface{}
	err := json.Unmarshal(toSortJSON, &c)
	if err != nil {
		return nil, err
	}
	js, err := json.Marshal(c)
	if err != nil {
		return nil, err
	}
	return js, nil
}

// MustSortJSON is like SortJSON but panic if an error occurs, e.g., if
// the passed JSON isn't valid.
// Deprecated: SortJSON was used for GetSignbytes, this is now automatic with amino signing
func MustSortJSON(toSortJSON []byte) []byte {
	js, err := SortJSON(toSortJSON)
	if err != nil {
		panic(err)
	}
	return js
}

// Uint64ToBigEndian - marshals uint64 to a bigendian byte slice so it can be sorted
func Uint64ToBigEndian(i uint64) []byte {
	b := make([]byte, 8)
	binary.BigEndian.PutUint64(b, i)
	return b
}

// BigEndianToUint64 returns an uint64 from big endian encoded bytes. If encoding
// is empty, zero is returned.
func BigEndianToUint64(bz []byte) uint64 {
	if len(bz) == 0 {
		return 0
	}

	return binary.BigEndian.Uint64(bz)
}

// Slight modification of the RFC3339Nano but it right pads all zeros and drops the time zone info
const SortableTimeFormat = "2006-01-02T15:04:05.000000000"

// Formats a time.Time into a []byte that can be sorted
func FormatTimeBytes(t time.Time) []byte {
	return []byte(FormatTimeString(t))
}

// Formats a time.Time into a string
func FormatTimeString(t time.Time) string {
	return t.UTC().Round(0).Format(SortableTimeFormat)
}

// Parses a []byte encoded using FormatTimeKey back into a time.Time
func ParseTimeBytes(bz []byte) (time.Time, error) {
	return ParseTime(bz)
}

// Parses an encoded type using FormatTimeKey back into a time.Time
func ParseTime(t any) (time.Time, error) {
	var (
		result time.Time
		err    error
	)

	switch t := t.(type) {
	case time.Time:
		result, err = t, nil
	case []byte:
		result, err = time.Parse(SortableTimeFormat, string(t))
	case string:
		result, err = time.Parse(SortableTimeFormat, t)
	default:
		return time.Time{}, fmt.Errorf("unexpected type %T", t)
	}

	if err != nil {
		return result, err
	}

	return result.UTC().Round(0), nil
}

// copy bytes
func CopyBytes(bz []byte) (ret []byte) {
	if bz == nil {
		return nil
	}
	ret = make([]byte, len(bz))
	copy(ret, bz)
	return ret
}

// AppendLengthPrefixedBytes combines the slices of bytes to one slice of bytes.
func AppendLengthPrefixedBytes(args ...[]byte) []byte {
	length := 0
	for _, v := range args {
		length += len(v)
	}
	res := make([]byte, length)

	length = 0
	for _, v := range args {
		copy(res[length:length+len(v)], v)
		length += len(v)
	}

	return res
}

// ParseLengthPrefixedBytes panics when store key length is not equal to the given length.
func ParseLengthPrefixedBytes(key []byte, startIndex, sliceLength int) ([]byte, int) {
	neededLength := startIndex + sliceLength
	endIndex := neededLength - 1
	kv.AssertKeyAtLeastLength(key, neededLength)
	byteSlice := key[startIndex:neededLength]

	return byteSlice, endIndex
}

// LogDeferred logs an error in a deferred function call if the returned error is non-nil.
func LogDeferred(logger log.Logger, f func() error) {
	if err := f(); err != nil {
		logger.Error(err.Error())
	}
}

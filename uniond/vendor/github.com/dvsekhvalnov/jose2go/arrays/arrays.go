// Package arrays provides various byte array utilities
package arrays

import (
	"bytes"
	"crypto/rand"
	"encoding/binary"
	"fmt"
	"github.com/dvsekhvalnov/jose2go/base64url"
)

// Xor is doing byte by byte exclusive or of 2 byte arrays
func Xor(left, right []byte) []byte {
	result := make([]byte, len(left))

	for i := 0; i < len(left); i++ {
		result[i] = left[i] ^ right[i]
	}

	return result
}

// Slice is splitting input byte array into slice of subarrays. Each of count length.
func Slice(arr []byte, count int) [][]byte {

	sliceCount := len(arr) / count
	result := make([][]byte, sliceCount)

	for i := 0; i < sliceCount; i++ {
		start := i * count
		end := i*count + count

		result[i] = arr[start:end]
	}

	return result
}

// Random generates byte array with random data of byteCount length
func Random(byteCount int) ([]byte, error) {
	data := make([]byte, byteCount)

	if _, err := rand.Read(data); err != nil {
		return nil, err
	}

	return data, nil
}

// Concat combine several arrays into single one, resulting slice = A1 | A2 | A3 | ... | An
func Concat(arrays ...[]byte) []byte {
	var result []byte = arrays[0]

	for _, arr := range arrays[1:] {
		result = append(result, arr...)
	}

	return result
}

// Unwrap same thing as Contact, just different interface, combines several array into single one
func Unwrap(arrays [][]byte) []byte {
	var result []byte = arrays[0]

	for _, arr := range arrays[1:] {
		result = append(result, arr...)
	}

	return result
}

// UInt64ToBytes unwrap uint64 value to byte array of length 8 using big endian
func UInt64ToBytes(value uint64) []byte {
	result := make([]byte, 8)
	binary.BigEndian.PutUint64(result, value)

	return result
}

// UInt32ToBytes unwrap uint32 value to byte array of length 4 using big endian
func UInt32ToBytes(value uint32) []byte {
	result := make([]byte, 4)
	binary.BigEndian.PutUint32(result, value)

	return result
}

// Dump produces printable debug representation of byte array as string
func Dump(arr []byte) string {
	var buf bytes.Buffer

	buf.WriteString("(")
	buf.WriteString(fmt.Sprintf("%v", len(arr)))
	buf.WriteString(" bytes)[")

	for idx, b := range arr {
		buf.WriteString(fmt.Sprintf("%v", b))
		if idx != len(arr)-1 {
			buf.WriteString(", ")
		}
	}

	buf.WriteString("], Hex: [")

	for idx, b := range arr {
		buf.WriteString(fmt.Sprintf("%X", b))
		if idx != len(arr)-1 {
			buf.WriteString(" ")
		}
	}

	buf.WriteString("], Base64Url:")
	buf.WriteString(base64url.Encode(arr))

	return buf.String()
}

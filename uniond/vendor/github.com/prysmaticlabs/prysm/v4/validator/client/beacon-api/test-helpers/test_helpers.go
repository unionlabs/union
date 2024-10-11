package test_helpers

import (
	"github.com/ethereum/go-ethereum/common/hexutil"
)

func FillByteSlice(sliceLength int, value byte) []byte {
	bytes := make([]byte, sliceLength)

	for index := range bytes {
		bytes[index] = value
	}

	return bytes
}

func FillByteArraySlice(sliceLength int, value []byte) [][]byte {
	bytes := make([][]byte, sliceLength)

	for index := range bytes {
		bytes[index] = value
	}

	return bytes
}

func FillEncodedByteSlice(sliceLength int, value byte) string {
	return hexutil.Encode(FillByteSlice(sliceLength, value))
}

func FillEncodedByteArraySlice(sliceLength int, value string) []string {
	encodedBytes := make([]string, sliceLength)
	for index := range encodedBytes {
		encodedBytes[index] = value
	}
	return encodedBytes
}

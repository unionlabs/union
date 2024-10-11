// Copyright 2017 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package bitutil

import "errors"

var (
	// errMissingData is returned from decompression if the byte referenced by
	// the bitset header overflows the input data.
	errMissingData = errors.New("missing bytes on input")

	// errUnreferencedData is returned from decompression if not all bytes were used
	// up from the input data after decompressing it.
	errUnreferencedData = errors.New("extra bytes on input")

	// errExceededTarget is returned from decompression if the bitset header has
	// more bits defined than the number of target buffer space available.
	errExceededTarget = errors.New("target data size exceeded")

	// errZeroContent is returned from decompression if a data byte referenced in
	// the bitset header is actually a zero byte.
	errZeroContent = errors.New("zero byte in input content")
)

// The compression algorithm implemented by CompressBytes and DecompressBytes is
// optimized for sparse input data which contains a lot of zero bytes. Decompression
// requires knowledge of the decompressed data length.
//
// Compression works as follows:
//
//   if data only contains zeroes,
//       CompressBytes(data) == nil
//   otherwise if len(data) <= 1,
//       CompressBytes(data) == data
//   otherwise:
//       CompressBytes(data) == append(CompressBytes(nonZeroBitset(data)), nonZeroBytes(data)...)
//       where
//         nonZeroBitset(data) is a bit vector with len(data) bits (MSB first):
//             nonZeroBitset(data)[i/8] && (1 << (7-i%8)) != 0  if data[i] != 0
//             len(nonZeroBitset(data)) == (len(data)+7)/8
//         nonZeroBytes(data) contains the non-zero bytes of data in the same order

// CompressBytes compresses the input byte slice according to the sparse bitset
// representation algorithm. If the result is bigger than the original input, no
// compression is done.
func CompressBytes(data []byte) []byte {
	if out := bitsetEncodeBytes(data); len(out) < len(data) {
		return out
	}
	cpy := make([]byte, len(data))
	copy(cpy, data)
	return cpy
}

// bitsetEncodeBytes compresses the input byte slice according to the sparse
// bitset representation algorithm.
func bitsetEncodeBytes(data []byte) []byte {
	// Empty slices get compressed to nil
	if len(data) == 0 {
		return nil
	}
	// One byte slices compress to nil or retain the single byte
	if len(data) == 1 {
		if data[0] == 0 {
			return nil
		}
		return data
	}
	// Calculate the bitset of set bytes, and gather the non-zero bytes
	nonZeroBitset := make([]byte, (len(data)+7)/8)
	nonZeroBytes := make([]byte, 0, len(data))

	for i, b := range data {
		if b != 0 {
			nonZeroBytes = append(nonZeroBytes, b)
			nonZeroBitset[i/8] |= 1 << byte(7-i%8)
		}
	}
	if len(nonZeroBytes) == 0 {
		return nil
	}
	return append(bitsetEncodeBytes(nonZeroBitset), nonZeroBytes...)
}

// DecompressBytes decompresses data with a known target size. If the input data
// matches the size of the target, it means no compression was done in the first
// place.
func DecompressBytes(data []byte, target int) ([]byte, error) {
	if len(data) > target {
		return nil, errExceededTarget
	}
	if len(data) == target {
		cpy := make([]byte, len(data))
		copy(cpy, data)
		return cpy, nil
	}
	return bitsetDecodeBytes(data, target)
}

// bitsetDecodeBytes decompresses data with a known target size.
func bitsetDecodeBytes(data []byte, target int) ([]byte, error) {
	out, size, err := bitsetDecodePartialBytes(data, target)
	if err != nil {
		return nil, err
	}
	if size != len(data) {
		return nil, errUnreferencedData
	}
	return out, nil
}

// bitsetDecodePartialBytes decompresses data with a known target size, but does
// not enforce consuming all the input bytes. In addition to the decompressed
// output, the function returns the length of compressed input data corresponding
// to the output as the input slice may be longer.
func bitsetDecodePartialBytes(data []byte, target int) ([]byte, int, error) {
	// Sanity check 0 targets to avoid infinite recursion
	if target == 0 {
		return nil, 0, nil
	}
	// Handle the zero and single byte corner cases
	decomp := make([]byte, target)
	if len(data) == 0 {
		return decomp, 0, nil
	}
	if target == 1 {
		decomp[0] = data[0] // copy to avoid referencing the input slice
		if data[0] != 0 {
			return decomp, 1, nil
		}
		return decomp, 0, nil
	}
	// Decompress the bitset of set bytes and distribute the non zero bytes
	nonZeroBitset, ptr, err := bitsetDecodePartialBytes(data, (target+7)/8)
	if err != nil {
		return nil, ptr, err
	}
	for i := 0; i < 8*len(nonZeroBitset); i++ {
		if nonZeroBitset[i/8]&(1<<byte(7-i%8)) != 0 {
			// Make sure we have enough data to push into the correct slot
			if ptr >= len(data) {
				return nil, 0, errMissingData
			}
			if i >= len(decomp) {
				return nil, 0, errExceededTarget
			}
			// Make sure the data is valid and push into the slot
			if data[ptr] == 0 {
				return nil, 0, errZeroContent
			}
			decomp[i] = data[ptr]
			ptr++
		}
	}
	return decomp, ptr, nil
}

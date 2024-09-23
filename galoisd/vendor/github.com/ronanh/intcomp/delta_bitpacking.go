package intcomp

//go:generate go run gen/gendeltapack.go

import "math/bits"

const (
	BitPackingBlockSize32 = 128
	BitPackingBlockSize64 = 256
)

// CompressDeltaBinPackInt32 compress blocks of 128 integers from `in`
// and append to `out`. `out` slice will be resized if necessary.
// The function returns the values from `in` that were not compressed (could
// not fit into a block), and the updated `out` slice.
//
// Compression logic:
//  1. Difference of consecutive inputs is computed (differential coding)
//  2. ZigZag encoding is applied if a block contains at least one negative delta value
//  3. The result is bit packed into the optimal number of bits for the block
func CompressDeltaBinPackInt32(in []int32, out []uint32) ([]int32, []uint32) {
	inlength := len(in) - len(in)%BitPackingBlockSize32
	if inlength == 0 {
		return in, out
	}

	if out == nil {
		out = make([]uint32, 0, len(in)/2)
	}

	inpos, outpos := 0, len(out)
	out = out[:cap(out)]

	// skip header (written at the end)
	headerpos := outpos
	outpos += 3

	firstValue := in[0]
	initoffset := firstValue
	endpos := inpos + inlength

	for inpos < endpos {
		// compute min/max bits for  4 groups of 32 inputs
		inblock1 := in[inpos : inpos+32]
		inblock2 := in[inpos+32 : inpos+64]
		inblock3 := in[inpos+64 : inpos+96]
		inblock4 := in[inpos+96 : inpos+128]
		bitlen1, sign1 := deltaBitLenAndSignInt32(initoffset, inblock1)
		bitlen2, sign2 := deltaBitLenAndSignInt32(inblock1[31], inblock2)
		bitlen3, sign3 := deltaBitLenAndSignInt32(inblock2[31], inblock3)
		bitlen4, sign4 := deltaBitLenAndSignInt32(inblock3[31], inblock4)

		if outsize := bitlen1 + bitlen2 + bitlen3 + bitlen4; outpos+outsize+1 >= len(out) {
			// no more space in out, realloc a bigger slice
			extrasize := outsize
			if extrasize < len(out)/4 {
				extrasize = len(out) / 4
			}
			tmpout := make([]uint32, outpos+extrasize+1)
			copy(tmpout, out)
			out = tmpout
		}

		// write block header
		out[outpos] = uint32((sign1 << 31) | (bitlen1 << 24) |
			(sign2 << 23) | (bitlen2 << 16) |
			(sign3 << 15) | (bitlen3 << 8) |
			(sign4 << 7) | bitlen4)
		outpos++

		// write block (4 x 32 packed inputs)
		if sign1 == 0 {
			deltaPack_int32(initoffset, inblock1, out[outpos:], bitlen1)
		} else {
			deltaPackZigzag_int32(initoffset, inblock1, out[outpos:], bitlen1)
		}
		outpos += bitlen1

		if sign2 == 0 {
			deltaPack_int32(inblock1[31], inblock2, out[outpos:], bitlen2)
		} else {
			deltaPackZigzag_int32(inblock1[31], inblock2, out[outpos:], bitlen2)
		}
		outpos += bitlen2

		if sign3 == 0 {
			deltaPack_int32(inblock2[31], inblock3, out[outpos:], bitlen3)
		} else {
			deltaPackZigzag_int32(inblock2[31], inblock3, out[outpos:], bitlen3)
		}
		outpos += bitlen3

		if sign4 == 0 {
			deltaPack_int32(inblock3[31], inblock4, out[outpos:], bitlen4)
		} else {
			deltaPackZigzag_int32(inblock3[31], inblock4, out[outpos:], bitlen4)
		}
		outpos += bitlen4

		inpos += 128
		initoffset = inblock4[31]
	}

	// write header
	out[headerpos] = uint32(inlength)
	out[headerpos+1] = uint32(outpos - headerpos)
	out[headerpos+2] = uint32(firstValue)
	return in[inpos:], out[:outpos]
}

// UncompressDeltaBinPackInt32 uncompress one ore more blocks of 128 integers from `in`
// and append the result to `out`. `out` slice will be resized if necessary.
// The function returns the values from `in` that were not uncompressed, and the updated `out` slice.
func UncompressDeltaBinPackInt32(in []uint32, out []int32) ([]uint32, []int32) {
	if len(in) == 0 {
		return in, out
	}
	outlen := int(in[0])

	// ensure enough space in out slice
	if len(out) == 0 && cap(out) < outlen {
		out = make([]int32, 0, outlen)
	} else if extrasize := outlen + len(out) - cap(out); extrasize > 0 {
		if extrasize < cap(out)/4 {
			extrasize = cap(out) / 4
		}
		tmpout := make([]int32, len(out), len(out)+extrasize)
		copy(tmpout, out)
		out = tmpout
	}

	inpos, outpos := 3, len(out)
	out = out[:cap(out)]

	// Read header
	endpos := outpos + outlen
	initoffset := int32(in[2])

	for outpos < endpos {
		tmp := uint32(in[inpos])
		sign1 := int(tmp>>31) & 0x1
		sign2 := int(tmp>>23) & 0x1
		sign3 := int(tmp>>15) & 0x1
		sign4 := int(tmp>>7) & 0x1
		bitlen1 := int(tmp>>24) & 0x7F
		bitlen2 := int(tmp>>16) & 0x7F
		bitlen3 := int(tmp>>8) & 0x7F
		bitlen4 := int(tmp & 0x7F)
		inpos++

		if sign1 == 0 {
			deltaUnpack_int32(initoffset, in[inpos:], out[outpos:], bitlen1)
		} else {
			deltaUnpackZigzag_int32(initoffset, in[inpos:], out[outpos:], bitlen1)
		}
		inpos += int(bitlen1)
		outpos += 32
		initoffset = out[outpos-1]

		if sign2 == 0 {
			deltaUnpack_int32(initoffset, in[inpos:], out[outpos:], bitlen2)
		} else {
			deltaUnpackZigzag_int32(initoffset, in[inpos:], out[outpos:], bitlen2)
		}
		inpos += int(bitlen2)
		outpos += 32
		initoffset = out[outpos-1]

		if sign3 == 0 {
			deltaUnpack_int32(initoffset, in[inpos:], out[outpos:], bitlen3)
		} else {
			deltaUnpackZigzag_int32(initoffset, in[inpos:], out[outpos:], bitlen3)
		}
		inpos += int(bitlen3)
		outpos += 32
		initoffset = out[outpos-1]

		if sign4 == 0 {
			deltaUnpack_int32(initoffset, in[inpos:], out[outpos:], bitlen4)
		} else {
			deltaUnpackZigzag_int32(initoffset, in[inpos:], out[outpos:], bitlen4)
		}
		inpos += int(bitlen4)
		outpos += 32
		initoffset = out[outpos-1]
	}
	return in[inpos:], out[:outpos]
}

func deltaBitLenAndSignInt32(initoffset int32, buf []int32) (int, int) {
	var mask uint32

	for _, v := range buf {
		diff := v - initoffset
		// zigzag encoding
		mask |= uint32((diff << 1) ^ (diff >> 31))
		initoffset = v
	}
	sign := int(mask & 1)
	// remove sign in zigzag encoding
	mask >>= 1

	return bits.Len32(uint32(mask)) + sign, sign
}

// CompressDeltaBinPackUInt32 compress blocks of 128 integers from `in`
// and append to `out`. `out` slice will be resized if necessary.
// The function returns the values from `in` that were not compressed (could
// not fit into a block), and the updated `out` slice.
//
// Compression logic:
//  1. Difference of consecutive inputs is computed (differential coding)
//  2. ZigZag encoding is applied if a block contains at least one negative delta value
//  3. The result is bit packed into the optimal number of bits for the block
func CompressDeltaBinPackUint32(in, out []uint32) ([]uint32, []uint32) {
	inlength := len(in) - len(in)%BitPackingBlockSize32
	if inlength == 0 {
		return in, out
	}

	if out == nil {
		out = make([]uint32, 0, len(in)/2)
	}

	inpos, outpos := 0, len(out)
	out = out[:cap(out)]

	// skip header (written at the end)
	headerpos := outpos
	outpos += 3

	firstValue := in[0]
	initoffset := firstValue
	endpos := inpos + inlength

	for inpos < endpos {
		// compute min/max bits for  4 groups of 32 inputs
		inblock1 := in[inpos : inpos+32]
		inblock2 := in[inpos+32 : inpos+64]
		inblock3 := in[inpos+64 : inpos+96]
		inblock4 := in[inpos+96 : inpos+128]
		bitlen1, sign1 := deltaBitLenAndSignUint32(initoffset, inblock1)
		bitlen2, sign2 := deltaBitLenAndSignUint32(inblock1[31], inblock2)
		bitlen3, sign3 := deltaBitLenAndSignUint32(inblock2[31], inblock3)
		bitlen4, sign4 := deltaBitLenAndSignUint32(inblock3[31], inblock4)

		if outsize := bitlen1 + bitlen2 + bitlen3 + bitlen4; outpos+outsize+1 >= len(out) {
			// no more space in out, realloc a bigger slice
			extrasize := outsize
			if extrasize < len(out)/4 {
				extrasize = len(out) / 4
			}
			tmpout := make([]uint32, outpos+extrasize+1)
			copy(tmpout, out)
			out = tmpout
		}

		// write block header
		out[outpos] = uint32((sign1 << 31) | (bitlen1 << 24) |
			(sign2 << 23) | (bitlen2 << 16) |
			(sign3 << 15) | (bitlen3 << 8) |
			(sign4 << 7) | bitlen4)
		outpos++

		// write block (4 x 32 packed inputs)
		if sign1 == 0 {
			deltaPack_uint32(initoffset, inblock1, out[outpos:], bitlen1)
		} else {
			deltaPackZigzag_uint32(initoffset, inblock1, out[outpos:], bitlen1)
		}
		outpos += bitlen1

		if sign2 == 0 {
			deltaPack_uint32(inblock1[31], inblock2, out[outpos:], bitlen2)
		} else {
			deltaPackZigzag_uint32(inblock1[31], inblock2, out[outpos:], bitlen2)
		}
		outpos += bitlen2

		if sign3 == 0 {
			deltaPack_uint32(inblock2[31], inblock3, out[outpos:], bitlen3)
		} else {
			deltaPackZigzag_uint32(inblock2[31], inblock3, out[outpos:], bitlen3)
		}
		outpos += bitlen3

		if sign4 == 0 {
			deltaPack_uint32(inblock3[31], inblock4, out[outpos:], bitlen4)
		} else {
			deltaPackZigzag_uint32(inblock3[31], inblock4, out[outpos:], bitlen4)
		}
		outpos += bitlen4

		inpos += 128
		initoffset = inblock4[31]
	}

	// write header
	out[headerpos] = uint32(inlength)
	out[headerpos+1] = uint32(outpos - headerpos)
	out[headerpos+2] = firstValue
	return in[inpos:], out[:outpos]
}

// UncompressDeltaBinPackUint32 uncompress one ore more blocks of 128 integers from `in`
// and append the result to `out`. `out` slice will be resized if necessary.
// The function returns the values from `in` that were not uncompressed, and the updated `out` slice.
func UncompressDeltaBinPackUint32(in, out []uint32) ([]uint32, []uint32) {
	if len(in) == 0 {
		return in, out
	}
	outlen := int(in[0])

	// ensure enough space in out slice
	if len(out) == 0 && cap(out) < outlen {
		out = make([]uint32, 0, outlen)
	} else if extrasize := outlen + len(out) - cap(out); extrasize > 0 {
		if extrasize < cap(out)/4 {
			extrasize = cap(out) / 4
		}
		tmpout := make([]uint32, len(out), len(out)+extrasize)
		copy(tmpout, out)
		out = tmpout
	}

	inpos, outpos := 3, len(out)
	out = out[:cap(out)]

	// Read header
	endpos := outpos + outlen
	initoffset := in[2]

	for outpos < endpos {
		tmp := uint32(in[inpos])
		sign1 := int(tmp>>31) & 0x1
		sign2 := int(tmp>>23) & 0x1
		sign3 := int(tmp>>15) & 0x1
		sign4 := int(tmp>>7) & 0x1
		bitlen1 := int(tmp>>24) & 0x7F
		bitlen2 := int(tmp>>16) & 0x7F
		bitlen3 := int(tmp>>8) & 0x7F
		bitlen4 := int(tmp & 0x7F)
		inpos++

		if sign1 == 0 {
			deltaUnpack_uint32(initoffset, in[inpos:], out[outpos:], bitlen1)
		} else {
			deltaUnpackZigzag_uint32(initoffset, in[inpos:], out[outpos:], bitlen1)
		}
		inpos += bitlen1
		outpos += 32
		initoffset = out[outpos-1]

		if sign2 == 0 {
			deltaUnpack_uint32(initoffset, in[inpos:], out[outpos:], bitlen2)
		} else {
			deltaUnpackZigzag_uint32(initoffset, in[inpos:], out[outpos:], bitlen2)
		}
		inpos += bitlen2
		outpos += 32
		initoffset = out[outpos-1]

		if sign3 == 0 {
			deltaUnpack_uint32(initoffset, in[inpos:], out[outpos:], bitlen3)
		} else {
			deltaUnpackZigzag_uint32(initoffset, in[inpos:], out[outpos:], bitlen3)
		}
		inpos += bitlen3
		outpos += 32
		initoffset = out[outpos-1]

		if sign4 == 0 {
			deltaUnpack_uint32(initoffset, in[inpos:], out[outpos:], bitlen4)
		} else {
			deltaUnpackZigzag_uint32(initoffset, in[inpos:], out[outpos:], bitlen4)
		}
		inpos += bitlen4
		outpos += 32
		initoffset = out[outpos-1]
	}
	return in[inpos:], out[:outpos]
}

func deltaBitLenAndSignUint32(initoffset uint32, buf []uint32) (int, int) {
	var mask uint32

	for _, v := range buf {
		diff := int32(v - initoffset)
		// zigzag encoding
		mask |= uint32((diff << 1) ^ (diff >> 31))
		initoffset = v
	}
	sign := int(mask & 1)
	// remove sign in zigzag encoding
	mask >>= 1

	return bits.Len32(uint32(mask)) + sign, sign
}

// CompressDeltaBinPackInt64 compress blocks of 256 integers from `in`
// and append to `out`. `out` slice will be resized if necessary.
// The function returns the values from `in` that were not compressed (could
// not fit into a block), and the updated `out` slice.
//
// Compression logic:
//  1. Difference of consecutive inputs is computed (differential coding)
//  2. ZigZag encoding is applied if a block contains at least one negative delta value
//  3. The result is bit packed into the optimal number of bits for the block
func CompressDeltaBinPackInt64(in []int64, out []uint64) ([]int64, []uint64) {
	inlength := len(in) - len(in)%BitPackingBlockSize64
	if inlength == 0 {
		return in, out
	}

	if out == nil {
		out = make([]uint64, 0, len(in)/2)
	}

	inpos, outpos := 0, len(out)
	out = out[:cap(out)]

	// skip header (written at the end)
	headerpos := outpos
	outpos += 2

	firstValue := in[0]
	initoffset := firstValue
	endpos := inpos + inlength

	for inpos < endpos {
		// compute min/max bits for  4 groups of 64 inputs
		inblock1 := in[inpos : inpos+64]
		inblock2 := in[inpos+64 : inpos+128]
		inblock3 := in[inpos+128 : inpos+192]
		inblock4 := in[inpos+192 : inpos+256]
		ntz1, bitlen1, sign1 := deltaBitTzAndLenAndSignInt64(initoffset, inblock1)
		ntz2, bitlen2, sign2 := deltaBitTzAndLenAndSignInt64(inblock1[63], inblock2)
		ntz3, bitlen3, sign3 := deltaBitTzAndLenAndSignInt64(inblock2[63], inblock3)
		ntz4, bitlen4, sign4 := deltaBitTzAndLenAndSignInt64(inblock3[63], inblock4)

		if outsize := bitlen1 + bitlen2 + bitlen3 + bitlen4 - ntz1 - ntz2 - ntz3 - ntz4; outpos+outsize+1 >= len(out) {
			// no more space in out, realloc a bigger slice
			extrasize := outsize
			if extrasize < len(out)/4 {
				extrasize = len(out) / 4
			}
			tmpout := make([]uint64, outpos+extrasize+1)
			copy(tmpout, out)
			out = tmpout
		}

		// write block header (min/max bits)
		out[outpos] = uint64((ntz1 << 56) | (ntz2 << 48) | (ntz3 << 40) | (ntz4 << 32) |
			(sign1 << 31) | (bitlen1 << 24) |
			(sign2 << 23) | (bitlen2 << 16) |
			(sign3 << 15) | (bitlen3 << 8) |
			(sign4 << 7) | bitlen4)
		outpos++

		// write block (4 x 64 packed inputs)
		if sign1 == 0 {
			deltaPack_int64(initoffset, inblock1, out[outpos:], ntz1, bitlen1)
		} else {
			deltaPackZigzag_int64(initoffset, inblock1, out[outpos:], ntz1, bitlen1)
		}
		outpos += int(bitlen1 - ntz1)

		if sign2 == 0 {
			deltaPack_int64(inblock1[63], inblock2, out[outpos:], ntz2, bitlen2)
		} else {
			deltaPackZigzag_int64(inblock1[63], inblock2, out[outpos:], ntz2, bitlen2)
		}
		outpos += int(bitlen2 - ntz2)

		if sign3 == 0 {
			deltaPack_int64(inblock2[63], inblock3, out[outpos:], ntz3, bitlen3)
		} else {
			deltaPackZigzag_int64(inblock2[63], inblock3, out[outpos:], ntz3, bitlen3)
		}
		outpos += int(bitlen3 - ntz3)

		if sign4 == 0 {
			deltaPack_int64(inblock3[63], inblock4, out[outpos:], ntz4, bitlen4)
		} else {
			deltaPackZigzag_int64(inblock3[63], inblock4, out[outpos:], ntz4, bitlen4)
		}
		outpos += int(bitlen4 - ntz4)

		inpos += 256
		initoffset = inblock4[63]
	}

	// write header
	out[headerpos] = uint64(inlength) + uint64(outpos-headerpos)<<32
	out[headerpos+1] = uint64(firstValue)
	return in[inpos:], out[:outpos]
}

// UncompressDeltaBinPackInt64 uncompress one ore more blocks of 256 integers from `in`
// and append the result to `out`. `out` slice will be resized if necessary.
// The function returns the values from `in` that were not uncompressed, and the updated `out` slice.
func UncompressDeltaBinPackInt64(in []uint64, out []int64) ([]uint64, []int64) {
	if len(in) == 0 {
		return in, out
	}
	// read header
	outlen := int(int32(in[0]))

	// ensure enough space in out slice
	if len(out) == 0 && cap(out) < outlen {
		out = make([]int64, 0, outlen)
	} else if extrasize := outlen + len(out) - cap(out); extrasize > 0 {
		if extrasize < cap(out)/4 {
			extrasize = cap(out) / 4
		}
		tmpout := make([]int64, len(out), len(out)+extrasize)
		copy(tmpout, out)
		out = tmpout
	}

	inpos, outpos := 2, len(out)
	out = out[:cap(out)]

	// Read header
	endpos := outpos + outlen
	initoffset := int64(in[1])

	for outpos < endpos {
		tmp := uint64(in[inpos])
		ntz1 := int(tmp>>56) & 0xFF
		ntz2 := int(tmp>>48) & 0xFF
		ntz3 := int(tmp>>40) & 0xFF
		ntz4 := int(tmp>>32) & 0xFF
		sign1 := int(tmp>>31) & 1
		sign2 := int(tmp>>23) & 1
		sign3 := int(tmp>>15) & 1
		sign4 := int(tmp>>7) & 1
		bitlen1 := int(tmp>>24) & 0x7F
		bitlen2 := int(tmp>>16) & 0x7F
		bitlen3 := int(tmp>>8) & 0x7F
		bitlen4 := int(tmp & 0x7F)
		inpos++

		if sign1 == 0 {
			deltaUnpack_int64(initoffset, in[inpos:], out[outpos:], ntz1, bitlen1)
		} else {
			deltaUnpackZigzag_int64(initoffset, in[inpos:], out[outpos:], ntz1, bitlen1)
		}
		inpos += int(bitlen1 - ntz1)
		outpos += 64
		initoffset = out[outpos-1]

		if sign2 == 0 {
			deltaUnpack_int64(initoffset, in[inpos:], out[outpos:], ntz2, bitlen2)
		} else {
			deltaUnpackZigzag_int64(initoffset, in[inpos:], out[outpos:], ntz2, bitlen2)
		}
		inpos += int(bitlen2 - ntz2)
		outpos += 64
		initoffset = out[outpos-1]

		if sign3 == 0 {
			deltaUnpack_int64(initoffset, in[inpos:], out[outpos:], ntz3, bitlen3)
		} else {
			deltaUnpackZigzag_int64(initoffset, in[inpos:], out[outpos:], ntz3, bitlen3)
		}
		inpos += int(bitlen3 - ntz3)
		outpos += 64
		initoffset = out[outpos-1]

		if sign4 == 0 {
			deltaUnpack_int64(initoffset, in[inpos:], out[outpos:], ntz4, bitlen4)
		} else {
			deltaUnpackZigzag_int64(initoffset, in[inpos:], out[outpos:], ntz4, bitlen4)
		}
		inpos += int(bitlen4 - ntz4)
		outpos += 64
		initoffset = out[outpos-1]
	}
	return in[inpos:], out[:outpos]
}

func deltaBitTzAndLenAndSignInt64(initoffset int64, buf []int64) (int, int, int) {
	var mask uint64

	for _, v := range buf {
		diff := v - initoffset
		// zigzag encoding
		mask |= uint64((diff << 1) ^ (diff >> 63))
		initoffset = v
	}
	sign := int(mask & 1)
	// remove sign in zigzag encoding
	mask >>= 1

	var ntz int
	if mask != 0 {
		ntz = bits.TrailingZeros32(uint32(mask))
	}

	return ntz, bits.Len64(uint64(mask)) + sign, sign
}

// CompressDeltaBinPackUint64 compress blocks of 256 integers from `in`
// and append to `out`. `out` slice will be resized if necessary.
// The function returns the values from `in` that were not compressed (could
// not fit into a block), and the updated `out` slice.
//
// Compression logic:
//  1. Difference of consecutive inputs is computed (differential coding)
//  2. ZigZag encoding is applied if a block contains at least one negative delta value
//  3. The result is bit packed into the optimal number of bits for the block
func CompressDeltaBinPackUint64(in, out []uint64) ([]uint64, []uint64) {
	inlength := len(in) - len(in)%BitPackingBlockSize64
	if inlength == 0 {
		return in, out
	}

	if out == nil {
		out = make([]uint64, 0, len(in)/2)
	}

	inpos, outpos := 0, len(out)
	out = out[:cap(out)]

	// skip header (written at the end)
	headerpos := outpos
	outpos += 2

	firstValue := in[0]
	initoffset := firstValue
	endpos := inpos + inlength

	for inpos < endpos {
		// compute min/max bits for  4 groups of 64 inputs
		inblock1 := in[inpos : inpos+64]
		inblock2 := in[inpos+64 : inpos+128]
		inblock3 := in[inpos+128 : inpos+192]
		inblock4 := in[inpos+192 : inpos+256]
		bitlen1, sign1 := deltaBitLenAndSignUint64(initoffset, inblock1)
		bitlen2, sign2 := deltaBitLenAndSignUint64(inblock1[63], inblock2)
		bitlen3, sign3 := deltaBitLenAndSignUint64(inblock2[63], inblock3)
		bitlen4, sign4 := deltaBitLenAndSignUint64(inblock3[63], inblock4)

		if outsize := bitlen1 + bitlen2 + bitlen3 + bitlen4; outpos+outsize+1 >= len(out) {
			// no more space in out, realloc a bigger slice
			extrasize := outsize
			if extrasize < len(out)/4 {
				extrasize = len(out) / 4
			}
			tmpout := make([]uint64, outpos+extrasize+1)
			copy(tmpout, out)
			out = tmpout
		}

		// write block header (min/max bits)
		out[outpos] = uint64(
			(sign1 << 31) | (bitlen1 << 24) |
				(sign2 << 23) | (bitlen2 << 16) |
				(sign3 << 15) | (bitlen3 << 8) |
				(sign4 << 7) | bitlen4)
		outpos++

		// write block (4 x 64 packed inputs)
		if sign1 == 0 {
			deltaPack_uint64(initoffset, inblock1, out[outpos:], bitlen1)
		} else {
			deltaPackZigzag_uint64(initoffset, inblock1, out[outpos:], bitlen1)
		}
		outpos += bitlen1

		if sign2 == 0 {
			deltaPack_uint64(inblock1[63], inblock2, out[outpos:], bitlen2)
		} else {
			deltaPackZigzag_uint64(inblock1[63], inblock2, out[outpos:], bitlen2)
		}
		outpos += bitlen2

		if sign3 == 0 {
			deltaPack_uint64(inblock2[63], inblock3, out[outpos:], bitlen3)
		} else {
			deltaPackZigzag_uint64(inblock2[63], inblock3, out[outpos:], bitlen3)
		}
		outpos += bitlen3

		if sign4 == 0 {
			deltaPack_uint64(inblock3[63], inblock4, out[outpos:], bitlen4)
		} else {
			deltaPackZigzag_uint64(inblock3[63], inblock4, out[outpos:], bitlen4)
		}
		outpos += bitlen4

		inpos += 256
		initoffset = inblock4[63]
	}

	// write header
	out[headerpos] = uint64(inlength) + uint64(outpos-headerpos)<<32
	out[headerpos+1] = firstValue
	return in[inpos:], out[:outpos]
}

// UncompressDeltaBinPackUint64 uncompress one ore more blocks of 256 integers from `in`
// and append the result to `out`. `out` slice will be resized if necessary.
// The function returns the values from `in` that were not uncompressed, and the updated `out` slice.
func UncompressDeltaBinPackUint64(in, out []uint64) ([]uint64, []uint64) {
	if len(in) == 0 {
		return in, out
	}
	// read header
	outlen := int(int32(in[0]))

	// ensure enough space in out slice
	if len(out) == 0 && cap(out) < outlen {
		out = make([]uint64, 0, outlen)
	} else if extrasize := outlen + len(out) - cap(out); extrasize > 0 {
		if extrasize < cap(out)/4 {
			extrasize = cap(out) / 4
		}
		tmpout := make([]uint64, len(out), len(out)+extrasize)
		copy(tmpout, out)
		out = tmpout
	}

	inpos, outpos := 2, len(out)
	out = out[:cap(out)]

	// Read header
	endpos := outpos + outlen
	initoffset := in[1]

	for outpos < endpos {
		tmp := uint64(in[inpos])
		sign1 := int(tmp>>31) & 1
		sign2 := int(tmp>>23) & 1
		sign3 := int(tmp>>15) & 1
		sign4 := int(tmp>>7) & 1
		bitlen1 := int(tmp>>24) & 0x7F
		bitlen2 := int(tmp>>16) & 0x7F
		bitlen3 := int(tmp>>8) & 0x7F
		bitlen4 := int(tmp & 0x7F)
		inpos++

		if sign1 == 0 {
			deltaUnpack_uint64(initoffset, in[inpos:], out[outpos:], bitlen1)
		} else {
			deltaUnpackZigzag_uint64(initoffset, in[inpos:], out[outpos:], bitlen1)
		}
		inpos += bitlen1
		outpos += 64
		initoffset = out[outpos-1]

		if sign2 == 0 {
			deltaUnpack_uint64(initoffset, in[inpos:], out[outpos:], bitlen2)
		} else {
			deltaUnpackZigzag_uint64(initoffset, in[inpos:], out[outpos:], bitlen2)
		}
		inpos += bitlen2
		outpos += 64
		initoffset = out[outpos-1]

		if sign3 == 0 {
			deltaUnpack_uint64(initoffset, in[inpos:], out[outpos:], bitlen3)
		} else {
			deltaUnpackZigzag_uint64(initoffset, in[inpos:], out[outpos:], bitlen3)
		}
		inpos += bitlen3
		outpos += 64
		initoffset = out[outpos-1]

		if sign4 == 0 {
			deltaUnpack_uint64(initoffset, in[inpos:], out[outpos:], bitlen4)
		} else {
			deltaUnpackZigzag_uint64(initoffset, in[inpos:], out[outpos:], bitlen4)
		}
		inpos += bitlen4
		outpos += 64
		initoffset = out[outpos-1]
	}
	return in[inpos:], out[:outpos]
}

func deltaBitLenAndSignUint64(initoffset uint64, buf []uint64) (int, int) {
	var mask uint64

	for _, v := range buf {
		diff := int64(v - initoffset)
		// zigzag encoding
		mask |= uint64((diff << 1) ^ (diff >> 63))
		initoffset = v
	}
	sign := int(mask & 1)
	// remove sign in zigzag encoding
	mask >>= 1

	return bits.Len64(uint64(mask)) + sign, sign
}

package intcomp

// CompressInt32 compress integers from `in`
// and append to `out`. `out` slice will be resized if necessary, the modified
// slice is returned.
//
// Compression logic:
//  1. Compress as many input as possible using `CompressDeltaBinPackInt32`
//  2. Compress the remaining input with `CompressDeltaBinPackInt32`
func CompressInt32(in []int32, out []uint32) []uint32 {
	if len(in) == 0 {
		return out
	}
	deltaVarByteHeaderPos := -1
	if len(out) > 0 {
		// truncate last value (lastBlockCompressedLen)
		lastBlockCompressedLen := int(out[len(out)-1])
		out = out[:len(out)-1]
		lastBlockHeaderPos := len(out) - lastBlockCompressedLen

		lastBlockUncompressedLen := int(out[lastBlockHeaderPos])
		if lastBlockUncompressedLen < BitPackingBlockSize32 && len(in) < BitPackingBlockSize32 {
			if lastBlockUncompressedLen+len(in) >= BitPackingBlockSize32 {
				// special case: both input and last compressed block
				// are smaller than bitPackingBlockSize32 but the concat is bigger
				// -> uncompress the last block and concat both buf
				// before compressing with bit packing
				_, tmpin := UncompressDeltaVarByteInt32(out[lastBlockHeaderPos:], make([]int32, 0, len(in)+lastBlockUncompressedLen+2))
				in = append(tmpin, in...)
				out = out[:lastBlockHeaderPos]
			} else {
				deltaVarByteHeaderPos = lastBlockHeaderPos
			}
		}
	}

	newBlockHeaderPos := len(out)
	if len(in) >= BitPackingBlockSize32 {
		in, out = CompressDeltaBinPackInt32(in, out)
	}
	if len(in) > 0 {
		if deltaVarByteHeaderPos >= 0 {
			newBlockHeaderPos = deltaVarByteHeaderPos
		} else {
			newBlockHeaderPos = len(out)
		}
		out = compressDeltaVarByteInt32(in, out, deltaVarByteHeaderPos)
	}
	return append(out, uint32(len(out)-newBlockHeaderPos))
}

// UncompressInt32 uncompress integers from `in` (compressed with `CompressInt32`)
// and append the result to `out`.
// `out` slice will be resized if necessary.
func UncompressInt32(in []uint32, out []int32) []int32 {
	// truncate last value (lastblock compressed length)
	if len(in) > 0 {
		in = in[:len(in)-1]
	}
	// compute output len, using block headers
	var iHeader, outlen int
	for iHeader < len(in) {
		outlen += int(int32(in[iHeader]))
		iHeader += int(int32(in[iHeader+1]))
	}
	// ensure enough space in out
	if cap(out) <= outlen {
		tmpout := make([]int32, len(out), outlen+1)
		copy(tmpout, out)
		out = tmpout
	}

	for len(in) > 0 {
		uncompressedSize := in[0]

		if uncompressedSize < BitPackingBlockSize32 {
			in, out = UncompressDeltaVarByteInt32(in, out)
		} else {
			in, out = UncompressDeltaBinPackInt32(in, out)
		}
	}

	return out
}

// CompressUint32 compress integers from `in`
// and append to `out`. `out` slice will be resized if necessary, the modified
// slice is returned.
//
// Compression logic:
//  1. Compress as many input as possible using `CompressDeltaBinPackUint32`
//  2. Compress the remaining input with `CompressDeltaBinPackUint32`
func CompressUint32(in, out []uint32) []uint32 {
	if len(in) == 0 {
		return out
	}
	deltaVarByteHeaderPos := -1
	if len(out) > 0 {
		// truncate last value (lastBlockCompressedLen)
		lastBlockCompressedLen := int(out[len(out)-1])
		out = out[:len(out)-1]
		lastBlockHeaderPos := len(out) - lastBlockCompressedLen

		lastBlockUncompressedLen := int(out[lastBlockHeaderPos])
		if lastBlockUncompressedLen < BitPackingBlockSize32 && len(in) < BitPackingBlockSize32 {
			if lastBlockUncompressedLen+len(in) >= BitPackingBlockSize32 {
				// special case: both input and last compressed block
				// are smaller than bitPackingBlockSize32 but the concat is bigger
				// -> uncompress the last block and concat both buf
				// before compressing with bit packing
				_, tmpin := UncompressDeltaVarByteUint32(out[lastBlockHeaderPos:], make([]uint32, 0, len(in)+lastBlockUncompressedLen+2))
				in = append(tmpin, in...)
				out = out[:lastBlockHeaderPos]
			} else {
				deltaVarByteHeaderPos = lastBlockHeaderPos
			}
		}
	}

	newBlockHeaderPos := len(out)
	if len(in) >= BitPackingBlockSize32 {
		in, out = CompressDeltaBinPackUint32(in, out)
	}
	if len(in) > 0 {
		if deltaVarByteHeaderPos >= 0 {
			newBlockHeaderPos = deltaVarByteHeaderPos
		} else {
			newBlockHeaderPos = len(out)
		}
		out = compressDeltaVarByteUint32(in, out, deltaVarByteHeaderPos)
	}
	return append(out, uint32(len(out)-newBlockHeaderPos))
}

// UncompressUint32 uncompress integers from `in` (compressed with `CompressUint32`)
// and append the result to `out`.
// `out` slice will be resized if necessary.
func UncompressUint32(in, out []uint32) []uint32 {
	// truncate last value (lastblock compressed length)
	if len(in) > 0 {
		in = in[:len(in)-1]
	}
	// compute output len, using block headers
	var iHeader, outlen int
	for iHeader < len(in) {
		outlen += int(in[iHeader])
		iHeader += int(in[iHeader+1])
	}
	// ensure enough space in out
	if cap(out) <= outlen {
		tmpout := make([]uint32, len(out), outlen+1)
		copy(tmpout, out)
		out = tmpout
	}

	for len(in) > 0 {
		uncompressedSize := in[0]

		if uncompressedSize < BitPackingBlockSize32 {
			in, out = UncompressDeltaVarByteUint32(in, out)
		} else {
			in, out = UncompressDeltaBinPackUint32(in, out)
		}
	}

	return out
}

// CompressInt64 compress integers from `in`
// and append to `out`. `out` slice will be resized if necessary, the modified
// slice is returned.
//
// Compression logic:
//  1. Compress as many input as possible using `CompressDeltaBinPackInt64`
//  2. Compress the remaining input with `CompressDeltaBinPackInt64`
func CompressInt64(in []int64, out []uint64) []uint64 {
	if len(in) == 0 {
		return out
	}
	deltaVarByteHeaderPos := -1
	if len(out) > 0 {
		// truncate last value (lastBlockCompressedLen)
		lastBlockCompressedLen := int(int64(out[len(out)-1]))
		out = out[:len(out)-1]
		lastBlockHeaderPos := len(out) - lastBlockCompressedLen

		lastBlockUncompressedLen := int(int32(int64(out[lastBlockHeaderPos])))
		if lastBlockUncompressedLen < BitPackingBlockSize64 && len(in) < BitPackingBlockSize64 {
			if lastBlockUncompressedLen+len(in) >= BitPackingBlockSize64 {
				// special case: both input and last compressed block
				// are smaller than bitPackingBlockSize64 but the concat is bigger
				// -> uncompress the last block and concat both buf
				// before compressing with bit packing
				_, tmpin := UncompressDeltaVarByteInt64(out[lastBlockHeaderPos:], make([]int64, 0, len(in)+lastBlockUncompressedLen+2))
				in = append(tmpin, in...)
				out = out[:lastBlockHeaderPos]
			} else {
				deltaVarByteHeaderPos = lastBlockHeaderPos
			}
		}
	}

	newBlockHeaderPos := len(out)
	if len(in) >= BitPackingBlockSize64 {
		in, out = CompressDeltaBinPackInt64(in, out)
	}
	if len(in) > 0 {
		if deltaVarByteHeaderPos >= 0 {
			newBlockHeaderPos = deltaVarByteHeaderPos
		} else {
			newBlockHeaderPos = len(out)
		}
		out = compressDeltaVarByteInt64(in, out, deltaVarByteHeaderPos)
	}
	return append(out, uint64(len(out)-newBlockHeaderPos))
}

// UncompressInt64 uncompress integers from `in` (compressed with `CompressInt64`)
// and append the result to `out`.
// `out` slice will be resized if necessary.
func UncompressInt64(in []uint64, out []int64) []int64 {
	// truncate last value (lastblock compressed length)
	if len(in) > 0 {
		in = in[:len(in)-1]
	}
	// compute output len, using block headers
	var iHeader, outlen int
	for iHeader < len(in) {
		outlen += int(int32(int64(in[iHeader])))
		iHeader += int(in[iHeader] >> 32)
	}
	// ensure enough space in out
	if cap(out) <= outlen {
		tmpout := make([]int64, len(out), outlen+1)
		copy(tmpout, out)
		out = tmpout
	}

	for len(in) > 0 {
		uncompressedSize := int32(in[0])

		if uncompressedSize < BitPackingBlockSize64 {
			in, out = UncompressDeltaVarByteInt64(in, out)
		} else {
			in, out = UncompressDeltaBinPackInt64(in, out)
		}
	}

	return out
}

// CompressUint64 compress integers from `in`
// and append to `out`. `out` slice will be resized if necessary, the modified
// slice is returned.
//
// Compression logic:
//  1. Compress as many input as possible using `CompressDeltaBinPackUint64`
//  2. Compress the remaining input with `CompressDeltaBinPackUint64`
func CompressUint64(in, out []uint64) []uint64 {
	if len(in) == 0 {
		return out
	}
	deltaVarByteHeaderPos := -1
	if len(out) > 0 {
		// truncate last value (lastBlockCompressedLen)
		lastBlockCompressedLen := int(out[len(out)-1])
		out = out[:len(out)-1]
		lastBlockHeaderPos := len(out) - lastBlockCompressedLen

		lastBlockUncompressedLen := int(int32(out[lastBlockHeaderPos]))
		if lastBlockUncompressedLen < BitPackingBlockSize64 && len(in) < BitPackingBlockSize64 {
			if lastBlockUncompressedLen+len(in) >= BitPackingBlockSize64 {
				// special case: both input and last compressed block
				// are smaller than bitPackingBlockSize64 but the concat is bigger
				// -> uncompress the last block and concat both buf
				// before compressing with bit packing
				_, tmpin := UncompressDeltaVarByteUint64(out[lastBlockHeaderPos:], make([]uint64, 0, len(in)+lastBlockUncompressedLen+2))
				in = append(tmpin, in...)
				out = out[:lastBlockHeaderPos]
			} else {
				deltaVarByteHeaderPos = lastBlockHeaderPos
			}
		}
	}

	newBlockHeaderPos := len(out)
	if len(in) >= BitPackingBlockSize64 {
		in, out = CompressDeltaBinPackUint64(in, out)
	}
	if len(in) > 0 {
		if deltaVarByteHeaderPos >= 0 {
			newBlockHeaderPos = deltaVarByteHeaderPos
		} else {
			newBlockHeaderPos = len(out)
		}
		out = compressDeltaVarByteUint64(in, out, deltaVarByteHeaderPos)
	}
	return append(out, uint64(len(out)-newBlockHeaderPos))
}

// UncompressUint64 uncompress integers from `in` (compressed with `CompressUint64`)
// and append the result to `out`.
// `out` slice will be resized if necessary.
func UncompressUint64(in, out []uint64) []uint64 {
	// truncate last value (lastblock compressed length)
	if len(in) > 0 {
		in = in[:len(in)-1]
	}
	// compute output len, using block headers
	var iHeader, outlen int
	for iHeader < len(in) {
		outlen += int(int32(in[iHeader]))
		iHeader += int(in[iHeader] >> 32)
	}
	// ensure enough space in out
	if cap(out) <= outlen {
		tmpout := make([]uint64, len(out), outlen+1)
		copy(tmpout, out)
		out = tmpout
	}

	for len(in) > 0 {
		uncompressedSize := int32(in[0])

		if uncompressedSize < BitPackingBlockSize64 {
			in, out = UncompressDeltaVarByteUint64(in, out)
		} else {
			in, out = UncompressDeltaBinPackUint64(in, out)
		}
	}

	return out
}

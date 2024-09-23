package intcomp

// CompressDeltaVarByteInt32 compress integers from `in`
// and append to `out`. `out` slice will be resized if necessary, the modified
// slice is returned.
//
// Compression logic:
//  1. Difference of consecutive inputs is computed (differential coding)
//  2. Variable byte encoding is applied
func CompressDeltaVarByteInt32(in []int32, out []uint32) []uint32 {
	return compressDeltaVarByteInt32(in, out, -1)
}

func compressDeltaVarByteInt32(in []int32, out []uint32, lastBlockHeaderPos int) []uint32 {
	if len(in) == 0 {
		return out
	}

	if out == nil {
		out = make([]uint32, 0, 6+len(in)/2)
	}

	var (
		prevInput  int32
		ivout      int
		inputDelta uint32
		vout       int32
		headerpos  = len(out)
		inpos      int
		outpos     = headerpos + 2
	)

	if lastBlockHeaderPos >= 0 {
		// append to last block
		headerpos = lastBlockHeaderPos

		outpos, ivout, prevInput, vout = getEncoderStateInt32(out, lastBlockHeaderPos)
	}
	out = out[:cap(out)]

	for {
		if outpos+2 >= len(out) { // we may not have enough buffer to write the next bytes
			extrasize := len(in) - inpos
			if extrasize < len(out)/4 {
				extrasize = len(out) / 4
			}
			if extrasize < 8 {
				extrasize = 8
			}
			tmpout := make([]uint32, outpos+extrasize)
			copy(tmpout, out)
			out = tmpout
		}

		if inpos >= len(in) {
			break
		}

		prevInput, inputDelta = in[inpos], uint32(in[inpos]-prevInput)
		for inputDelta >= 0x80 {
			vout <<= 8
			vout |= int32(byte(inputDelta) | 0x80)
			inputDelta >>= 7
			ivout++
			if ivout%4 == 0 { // vout is full
				out[outpos] = uint32(vout)
				outpos++
				vout = 0
			}
		}
		vout <<= 8
		vout |= int32(byte(inputDelta))
		ivout++
		if ivout%4 == 0 { // vout is full
			out[outpos] = uint32(vout)
			outpos++
			vout = 0
		}
		inpos++
	}

	if ivout%4 != 0 { // vout is  partially written
		// pad vout
		for ivout%4 != 0 {
			vout <<= 8
			vout |= 0x80
			ivout++
		}
		out[outpos] = uint32(vout)
		outpos++
	}
	// write header
	if lastBlockHeaderPos < 0 {
		out[headerpos] = 0
	}
	out[headerpos] += uint32(len(in))
	out[headerpos+1] = uint32(outpos - headerpos)
	return out[:outpos]
}

func getEncoderStateInt32(out []uint32, lastBlockHeaderPos int) (outpos, ivout int, prevInput int32, vout int32) {
	outpos = lastBlockHeaderPos + 2

	var (
		prevBlockLen = int(out[lastBlockHeaderPos])
		shiftIn      = 24
		shiftOut     int
		delta        int32
		i            int
	)
	for i < prevBlockLen {
		vout = int32(out[outpos]) >> shiftIn

		ivout++
		shiftIn -= 8
		if shiftIn < 0 {
			shiftIn = 24
			outpos++
		}

		delta += ((vout & 0x7F) << shiftOut)
		shiftOut += 7
		if vout&0x80 == 0 {
			shiftOut = 0
			prevInput += delta
			i++
			delta = 0
		}
	}
	return
}

// UncompressDeltaVarByteInt32 uncompress one block of integers from `in` (compressed with `CompressDeltaVarByteInt32`)
// and append the result to `out`. `out` slice will be resized if necessary.
//
// The function returns the values from `in` that were not uncompressed, and the updated `out` slice.
func UncompressDeltaVarByteInt32(in []uint32, out []int32) ([]uint32, []int32) {
	if len(in) == 0 {
		return in, out
	}

	// read header
	outlen := int(in[0])
	inlen := int(in[1])
	resin := in[inlen:]
	in = in[:inlen]

	if cap(out)-len(out) < outlen {
		tmpout := make([]int32, len(out), len(out)+outlen)
		copy(tmpout, out)
		out = tmpout
	}

	inpos, outpos := 2, len(out)
	out = out[:cap(out)]

	var (
		shiftIn    int = 24
		inend          = len(in)
		initoffset int32
		delta      int32
		shiftOut   int
	)

	for inpos < inend {
		v := int32(in[inpos]) >> shiftIn

		shiftIn -= 8
		if shiftIn < 0 {
			shiftIn = 24
			inpos++
		}

		delta += ((v & 0x7F) << shiftOut)
		shiftOut += 7
		if v&0x80 == 0 {
			shiftOut = 0

			out[outpos] = delta + initoffset
			initoffset = out[outpos]
			outpos++
			delta = 0
		}
	}
	return resin, out[:outpos]
}

// CompressDeltaVarByteUint32 compress integers from `in`
// and append to `out`. `out` slice will be resized if necessary, the modified
// slice is returned.
//
// Compression logic:
//  1. Difference of consecutive inputs is computed (differential coding)
//  2. Variable byte encoding is applied
func CompressDeltaVarByteUint32(in, out []uint32) []uint32 {
	return compressDeltaVarByteUint32(in, out, -1)
}

func compressDeltaVarByteUint32(in, out []uint32, lastBlockHeaderPos int) []uint32 {
	if len(in) == 0 {
		return out
	}

	if out == nil {
		out = make([]uint32, 0, 6+len(in)/2)
	}

	var (
		prevInput  uint32
		ivout      int
		inputDelta uint32
		vout       uint32
		headerpos  = len(out)
		inpos      int
		outpos     = headerpos + 2
	)

	if lastBlockHeaderPos >= 0 {
		// append to last block
		headerpos = lastBlockHeaderPos

		outpos, ivout, prevInput, vout = getEncoderStateUint32(out, lastBlockHeaderPos)
	}
	out = out[:cap(out)]

	for {
		if outpos+2 >= len(out) { // we may not have enough buffer to write the next bytes
			extrasize := len(in) - inpos
			if extrasize < len(out)/4 {
				extrasize = len(out) / 4
			}
			if extrasize < 8 {
				extrasize = 8
			}
			tmpout := make([]uint32, outpos+extrasize)
			copy(tmpout, out)
			out = tmpout
		}

		if inpos >= len(in) {
			break
		}

		prevInput, inputDelta = in[inpos], uint32(in[inpos]-prevInput)
		for inputDelta >= 0x80 {
			vout <<= 8
			vout |= uint32(byte(inputDelta) | 0x80)
			inputDelta >>= 7
			ivout++
			if ivout%4 == 0 { // vout is full
				out[outpos] = vout
				outpos++
				vout = 0
			}
		}
		vout <<= 8
		vout |= inputDelta & 0xFF
		ivout++
		if ivout%4 == 0 { // vout is full
			out[outpos] = vout
			outpos++
			vout = 0
		}
		inpos++
	}

	if ivout%4 != 0 { // vout is  partially written
		// pad vout
		for ivout%4 != 0 {
			vout <<= 8
			vout |= 0x80
			ivout++
		}
		out[outpos] = vout
		outpos++
	}
	// write header
	if lastBlockHeaderPos < 0 {
		out[headerpos] = 0
	}
	out[headerpos] += uint32(len(in))
	out[headerpos+1] = uint32(outpos - headerpos)
	return out[:outpos]
}

func getEncoderStateUint32(out []uint32, lastBlockHeaderPos int) (outpos, ivout int, prevInput, vout uint32) {
	outpos = lastBlockHeaderPos + 2

	var (
		prevBlockLen = int(out[lastBlockHeaderPos])
		shiftIn      = 24
		shiftOut     int
		delta        uint32
		i            int
	)
	for i < prevBlockLen {
		vout = out[outpos] >> shiftIn

		ivout++
		shiftIn -= 8
		if shiftIn < 0 {
			shiftIn = 24
			outpos++
		}

		delta += ((vout & 0x7F) << shiftOut)
		shiftOut += 7
		if vout&0x80 == 0 {
			shiftOut = 0
			prevInput += delta
			i++
			delta = 0
		}
	}
	return
}

// UncompressDeltaVarByteUint32 uncompress one block of integers from `in` (compressed with `CompressDeltaVarByteInt32`)
// and append the result to `out`. `out` slice will be resized if necessary.
//
// The function returns the values from `in` that were not uncompressed, and the updated `out` slice.
func UncompressDeltaVarByteUint32(in, out []uint32) ([]uint32, []uint32) {
	if len(in) == 0 {
		return in, out
	}
	// read header
	outlen := int(in[0])
	inlen := int(in[1])
	resin := in[inlen:]
	in = in[:inlen]

	if cap(out)-len(out) < outlen {
		tmpout := make([]uint32, len(out), len(out)+outlen)
		copy(tmpout, out)
		out = tmpout
	}

	inpos, outpos := 2, len(out)
	out = out[:cap(out)]

	var (
		shiftIn    int = 24
		inend          = len(in)
		initoffset uint32
		delta      uint32
		shiftOut   int
	)

	for inpos < inend {
		c := in[inpos] >> shiftIn

		shiftIn -= 8
		if shiftIn < 0 {
			shiftIn = 24
			inpos++
		}

		delta += ((c & 0x7F) << shiftOut)
		shiftOut += 7
		if c&0x80 == 0 {
			shiftOut = 0

			out[outpos] = delta + initoffset
			initoffset = out[outpos]
			outpos++
			delta = 0
		}
	}
	return resin, out[:outpos]
}

// CompressDeltaVarByteInt64 compress integers from `in`
// and append to `out`. `out` slice will be resized if necessary, the modified
// slice is returned.
//
// Compression logic:
//  1. Difference of consecutive inputs is computed (differential coding)
//  2. Variable byte encoding is applied
func CompressDeltaVarByteInt64(in []int64, out []uint64) []uint64 {
	return compressDeltaVarByteInt64(in, out, -1)
}

func compressDeltaVarByteInt64(in []int64, out []uint64, lastBlockHeaderPos int) []uint64 {
	if len(in) == 0 {
		return out
	}

	if out == nil {
		out = make([]uint64, 0, 6+len(in)/2)
	}

	var (
		prevInput  int64
		ivout      int
		inputDelta uint64
		vout       int64
		headerpos  = len(out)
		inpos      int
		outpos     = headerpos + 1
	)

	if lastBlockHeaderPos >= 0 {
		// append to last block
		headerpos = lastBlockHeaderPos

		outpos, ivout, prevInput, vout = getEncoderStateInt64(out, lastBlockHeaderPos)
	}
	out = out[:cap(out)]

	for {
		if outpos+2 >= len(out) { // we may not have enough buffer to write the next bytes
			extrasize := len(in) - inpos
			if extrasize < len(out)/4 {
				extrasize = len(out) / 4
			}
			if extrasize < 8 {
				extrasize = 8
			}
			tmpout := make([]uint64, outpos+extrasize)
			copy(tmpout, out)
			out = tmpout
		}

		if inpos >= len(in) {
			break
		}

		prevInput, inputDelta = in[inpos], uint64(in[inpos]-prevInput)
		for inputDelta >= 0x80 {
			vout <<= 8
			vout |= int64(byte(inputDelta) | 0x80)
			inputDelta >>= 7
			ivout++
			if ivout%8 == 0 { // vout is full
				out[outpos] = uint64(vout)
				outpos++
				vout = 0
			}
		}
		vout <<= 8
		vout |= int64(byte(inputDelta))
		ivout++
		if ivout%8 == 0 { // vout is full
			out[outpos] = uint64(vout)
			outpos++
			vout = 0
		}
		inpos++
	}

	if ivout%8 != 0 { // vout is  partially written
		// pad vout
		for ivout%8 != 0 {
			vout <<= 8
			vout |= 0x80
			ivout++
		}
		out[outpos] = uint64(vout)
		outpos++
	}
	// write header
	var prevLen int
	if lastBlockHeaderPos >= 0 {
		prevLen = int(int32(out[headerpos]))
	}
	out[headerpos] = uint64(len(in)+prevLen) + uint64(outpos-headerpos)<<32
	return out[:outpos]
}

func getEncoderStateInt64(out []uint64, lastBlockHeaderPos int) (outpos, ivout int, prevInput, vout int64) {
	outpos = lastBlockHeaderPos + 1

	var (
		prevBlockLen = int(int32(out[lastBlockHeaderPos]))
		shiftIn      = 56
		shiftOut     int
		delta        int64
		i            int
	)
	for i < prevBlockLen {
		vout = int64(out[outpos]) >> shiftIn

		ivout++
		shiftIn -= 8
		if shiftIn < 0 {
			shiftIn = 56
			outpos++
		}

		delta += ((vout & 0x7F) << shiftOut)
		shiftOut += 7
		if vout&0x80 == 0 {
			shiftOut = 0
			prevInput += delta
			i++
			delta = 0
		}
	}
	return
}

// UncompressDeltaVarByteInt64 uncompress one block of integers from `in` (compressed with `CompressDeltaVarByteInt32`)
// and append the result to `out`. `out` slice will be resized if necessary.
//
// The function returns the values from `in` that were not uncompressed, and the updated `out` slice.
func UncompressDeltaVarByteInt64(in []uint64, out []int64) ([]uint64, []int64) {
	if len(in) == 0 {
		return in, out
	}

	// read header
	outlen := int(int32(in[0]))
	inlen := int(in[0] >> 32)
	resin := in[inlen:]
	in = in[:inlen]

	if cap(out)-len(out) < outlen {
		tmpout := make([]int64, len(out), len(out)+outlen)
		copy(tmpout, out)
		out = tmpout
	}

	inpos, outpos := 1, len(out)
	out = out[:cap(out)]

	var (
		shiftIn    int = 56
		inend          = len(in)
		initoffset int64
		delta      int64
		shiftOut   int
	)

	for inpos < inend {
		c := int64(in[inpos]) >> shiftIn

		shiftIn -= 8
		if shiftIn < 0 {
			shiftIn = 56
			inpos++
		}

		delta += ((c & 0x7F) << shiftOut)
		shiftOut += 7
		if c&0x80 == 0 {
			shiftOut = 0

			out[outpos] = delta + initoffset
			initoffset = out[outpos]
			outpos++
			delta = 0
		}
	}
	return resin, out[:outpos]
}

// CompressDeltaVarByteUint64 compress integers from `in`
// and append to `out`. `out` slice will be resized if necessary, the modified
// slice is returned.
//
// Compression logic:
//  1. Difference of consecutive inputs is computed (differential coding)
//  2. Variable byte encoding is applied
func CompressDeltaVarByteUint64(in, out []uint64) []uint64 {
	return compressDeltaVarByteUint64(in, out, -1)
}

func compressDeltaVarByteUint64(in, out []uint64, lastBlockHeaderPos int) []uint64 {
	if len(in) == 0 {
		return out
	}

	if out == nil {
		out = make([]uint64, 0, 6+len(in)/2)
	}

	var (
		prevInput  uint64
		ivout      int
		inputDelta uint64
		vout       uint64
		headerpos  = len(out)
		inpos      int
		outpos     = headerpos + 1
	)

	if lastBlockHeaderPos >= 0 {
		// append to last block
		headerpos = lastBlockHeaderPos

		outpos, ivout, prevInput, vout = getEncoderStateUint64(out, lastBlockHeaderPos)
	}
	out = out[:cap(out)]

	for {
		if outpos+2 >= len(out) { // we may not have enough buffer to write the next bytes
			extrasize := len(in) - inpos
			if extrasize < len(out)/4 {
				extrasize = len(out) / 4
			}
			if extrasize < 8 {
				extrasize = 8
			}
			tmpout := make([]uint64, outpos+extrasize)
			copy(tmpout, out)
			out = tmpout
		}

		if inpos >= len(in) {
			break
		}

		prevInput, inputDelta = in[inpos], uint64(in[inpos]-prevInput)
		for inputDelta >= 0x80 {
			vout <<= 8
			vout |= uint64(byte(inputDelta) | 0x80)
			inputDelta >>= 7
			ivout++
			if ivout%8 == 0 { // vout is full
				out[outpos] = vout
				outpos++
				vout = 0
			}
		}
		vout <<= 8
		vout |= uint64(byte(inputDelta))
		ivout++
		if ivout%8 == 0 { // vout is full
			out[outpos] = vout
			outpos++
			vout = 0
		}
		inpos++
	}

	if ivout%8 != 0 { // vout is  partially written
		// pad vout
		for ivout%8 != 0 {
			vout <<= 8
			vout |= 0x80
			ivout++
		}
		out[outpos] = vout
		outpos++
	}
	// write header
	var prevLen int
	if lastBlockHeaderPos >= 0 {
		prevLen = int(int32(out[headerpos]))
	}
	out[headerpos] = uint64(len(in)+prevLen) + uint64(outpos-headerpos)<<32

	return out[:outpos]
}

func getEncoderStateUint64(out []uint64, lastBlockHeaderPos int) (outpos, ivout int, prevInput, vout uint64) {
	outpos = lastBlockHeaderPos + 1

	var (
		prevBlockLen = int(int32(out[lastBlockHeaderPos]))
		shiftIn      = 56
		shiftOut     int
		delta        uint64
		i            int
	)
	for i < prevBlockLen {
		vout = out[outpos] >> shiftIn

		ivout++
		shiftIn -= 8
		if shiftIn < 0 {
			shiftIn = 56
			outpos++
		}

		delta += ((vout & 0x7F) << shiftOut)
		shiftOut += 7
		if vout&0x80 == 0 {
			shiftOut = 0
			prevInput += delta
			i++
			delta = 0
		}
	}
	return
}

// UncompressDeltaVarByteUint64 uncompress one block of integers from `in` (compressed with `CompressDeltaVarByteInt32`)
// and append the result to `out`. `out` slice will be resized if necessary.
//
// The function returns the values from `in` that were not uncompressed, and the updated `out` slice.
func UncompressDeltaVarByteUint64(in, out []uint64) ([]uint64, []uint64) {
	if len(in) == 0 {
		return in, out
	}

	// read header
	outlen := int(int32(in[0]))
	inlen := int(in[0] >> 32)
	resin := in[inlen:]
	in = in[:inlen]

	if cap(out)-len(out) < outlen {
		tmpout := make([]uint64, len(out), len(out)+outlen)
		copy(tmpout, out)
		out = tmpout
	}

	inpos, outpos := 1, len(out)
	out = out[:cap(out)]

	var (
		shiftIn    int = 56
		inend          = len(in)
		initoffset uint64
		delta      uint64
		shiftOut   int
	)

	for inpos < inend {
		c := in[inpos] >> shiftIn

		shiftIn -= 8
		if shiftIn < 0 {
			shiftIn = 56
			inpos++
		}

		delta += ((c & 0x7F) << shiftOut)
		shiftOut += 7
		if c&0x80 == 0 {
			shiftOut = 0

			out[outpos] = delta + initoffset
			initoffset = out[outpos]
			outpos++
			delta = 0
		}
	}
	return resin, out[:outpos]
}

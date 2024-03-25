// Package sha2 implements SHA2 hash computation.
//
// This package extends the SHA2 permutation function [sha2] into a full SHA2
// hash.
package sha2

import (
	"encoding/binary"

	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/hash"
	"github.com/consensys/gnark/std/math/uints"
	"github.com/consensys/gnark/std/permutation/sha2"
)

var _seed = uints.NewU32Array([]uint32{
	0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
})

type digest struct {
	uapi *uints.BinaryField[uints.U32]
	in   []uints.U8
}

func New(api frontend.API) (hash.BinaryHasher, error) {
	uapi, err := uints.New[uints.U32](api)
	if err != nil {
		return nil, err
	}
	return &digest{uapi: uapi}, nil
}

func (d *digest) Write(data []uints.U8) {
	d.in = append(d.in, data...)
}

func (d *digest) padded(bytesLen int) []uints.U8 {
	zeroPadLen := 55 - bytesLen%64
	if zeroPadLen < 0 {
		zeroPadLen += 64
	}
	if cap(d.in) < len(d.in)+9+zeroPadLen {
		// in case this is the first time this method is called increase the
		// capacity of the slice to fit the padding.
		d.in = append(d.in, make([]uints.U8, 9+zeroPadLen)...)
		d.in = d.in[:len(d.in)-9-zeroPadLen]
	}
	buf := d.in
	buf = append(buf, uints.NewU8(0x80))
	buf = append(buf, uints.NewU8Array(make([]uint8, zeroPadLen))...)
	lenbuf := make([]uint8, 8)
	binary.BigEndian.PutUint64(lenbuf, uint64(8*bytesLen))
	buf = append(buf, uints.NewU8Array(lenbuf)...)
	return buf
}

func (d *digest) Sum() []uints.U8 {
	var runningDigest [8]uints.U32
	var buf [64]uints.U8
	copy(runningDigest[:], _seed)
	padded := d.padded(len(d.in))
	for i := 0; i < len(padded)/64; i++ {
		copy(buf[:], padded[i*64:(i+1)*64])
		runningDigest = sha2.Permute(d.uapi, runningDigest, buf)
	}
	var ret []uints.U8
	for i := range runningDigest {
		ret = append(ret, d.uapi.UnpackMSB(runningDigest[i])...)
	}
	return ret
}

func (d *digest) FixedLengthSum(length frontend.Variable) []uints.U8 {
	panic("TODO")
	// we need to do two things here -- first the padding has to be put to the
	// right place. For that we need to know how many blocks we have used. We
	// need to fit at least 9 more bytes (padding byte and 8 bytes for input
	// length). Knowing the block, we have to keep running track if the current
	// block is the expected one.
	//
	// idea - have a mask for blocks where 1 is only for the block we want to
	// use.
}

func (d *digest) Reset() {
	d.in = nil
}

func (d *digest) Size() int { return 32 }

package ioutils

import (
	"encoding/binary"
	"io"

	"github.com/ronanh/intcomp"
)

// CompressAndWriteUints32 compresses a slice of uint32 and writes it to w.
// It returns the input buffer (possibly extended) for future use.
func CompressAndWriteUints32(w io.Writer, input []uint32, buffer []uint32) ([]uint32, error) {
	buffer = buffer[:0]
	buffer = intcomp.CompressUint32(input, buffer)
	if err := binary.Write(w, binary.LittleEndian, uint64(len(buffer))); err != nil {
		return nil, err
	}
	if err := binary.Write(w, binary.LittleEndian, buffer); err != nil {
		return nil, err
	}
	return buffer, nil
}

// CompressAndWriteUints64 compresses a slice of uint64 and writes it to w.
// It returns the input buffer (possibly extended) for future use.
func CompressAndWriteUints64(w io.Writer, input []uint64) error {
	buffer := intcomp.CompressUint64(input, nil)
	if err := binary.Write(w, binary.LittleEndian, uint64(len(buffer))); err != nil {
		return err
	}
	return binary.Write(w, binary.LittleEndian, buffer)
}

// ReadAndDecompressUints32 reads a compressed slice of uint32 from r and decompresses it.
// It returns the number of bytes read, the decompressed slice and an error.
func ReadAndDecompressUints32(in []byte, buf32 []uint32) (outbuf32 []uint32, read int, out []uint32, err error) {
	if len(in) < 8 {
		return buf32, 0, nil, io.ErrUnexpectedEOF
	}
	length := binary.LittleEndian.Uint64(in[:8])
	if uint64(len(in)) < 8+4*length {
		return buf32, 0, nil, io.ErrUnexpectedEOF
	}
	in = in[8 : 8+4*length]
	if cap(buf32) < int(length) {
		buf32 = make([]uint32, length)
	} else {
		buf32 = buf32[:length]
	}

	for i := 0; i < int(length); i++ {
		buf32[i] = binary.LittleEndian.Uint32(in[4*i : 4*(i+1)])
	}

	return buf32, 8 + 4*int(length), intcomp.UncompressUint32(buf32, nil), nil
}

// ReadAndDecompressUints64 reads a compressed slice of uint64 from r and decompresses it.
// It returns the number of bytes read, the decompressed slice and an error.
func ReadAndDecompressUints64(in []byte) (int, []uint64, error) {
	if len(in) < 8 {
		return 0, nil, io.ErrUnexpectedEOF
	}
	length := binary.LittleEndian.Uint64(in[:8])
	if uint64(len(in)) < 8+8*length {
		return 0, nil, io.ErrUnexpectedEOF
	}
	in = in[8 : 8+8*length]
	buffer := make([]uint64, length)
	for i := 0; i < int(length); i++ {
		buffer[i] = binary.LittleEndian.Uint64(in[8*i : 8*(i+1)])
	}
	return 8 + 8*int(length), intcomp.UncompressUint64(buffer, nil), nil
}

package unsafe

import (
	"bytes"
	"encoding/binary"
	"errors"
	"io"
	"unsafe"
)

// WriteSlice writes a slice of arbitrary objects to the writer.
// Use with caution, as it writes the raw memory representation of the slice;
// In particular you do not want to use this with slices that contain pointers.
// This architecture dependent and will not work across different architectures
// (e.g. 32 vs 64 bit, big endian vs little endian).
func WriteSlice[S ~[]E, E any](w io.Writer, s S) error {
	var e E
	size := int(unsafe.Sizeof(e))
	if err := binary.Write(w, binary.LittleEndian, uint64(len(s))); err != nil {
		return err
	}

	if len(s) == 0 {
		return nil
	}

	data := unsafe.Slice((*byte)(unsafe.Pointer(&s[0])), size*len(s))
	if _, err := w.Write(data); err != nil {
		return err
	}
	return nil
}

// ReadSlice reads a slice of arbitrary objects from the reader, written by WriteSlice.
func ReadSlice[S ~[]E, E any](r io.Reader, maxElements ...int) (s S, read int, err error) {
	var buf [8]byte
	if _, err := io.ReadFull(r, buf[:]); err != nil {
		return nil, 0, err
	}
	read += 8

	// decode length of the slice
	length := binary.LittleEndian.Uint64(buf[:])

	var e E
	size := int(unsafe.Sizeof(e))
	limit := length
	if len(maxElements) == 1 && maxElements[0] > 0 && int(length) > maxElements[0] {
		limit = uint64(maxElements[0])
	}

	if limit == 0 {
		return make(S, 0), read, nil
	}

	toReturn := make(S, limit)

	// directly read the bytes from reader into the target memory area
	// (slice data)
	data := unsafe.Slice((*byte)(unsafe.Pointer(&toReturn[0])), size*int(limit))
	if _, err := io.ReadFull(r, data); err != nil {
		return nil, read, err
	}

	read += size * int(limit)

	// advance the reader if we had more elements than we wanted
	if length > limit {
		advance := int(length-limit) * size
		if _, err := io.CopyN(io.Discard, r, int64(advance)); err != nil {
			return nil, read, err
		}
		read += advance
	}

	return toReturn, read, nil
}

const marker uint64 = 0xdeadbeef

// WriteMarker writes the raw memory representation of a fixed marker to the writer.
// This is used to ensure that the dump was written on the same architecture.
func WriteMarker(w io.Writer) error {
	marker := marker
	_, err := w.Write(unsafe.Slice((*byte)(unsafe.Pointer(&marker)), 8))
	return err
}

// ReadMarker reads the raw memory representation of a fixed marker from the reader.
// This is used to ensure that the dump was written on the same architecture.
func ReadMarker(r io.Reader) error {
	var buf [8]byte
	if _, err := io.ReadFull(r, buf[:]); err != nil {
		return err
	}
	marker := marker
	d := unsafe.Slice((*byte)(unsafe.Pointer(&marker)), 8)
	if !bytes.Equal(d, buf[:]) {
		return errors.New("marker mismatch: dump was not written on the same architecture")
	}
	return nil
}

package websocket

import (
	"bufio"
	"encoding/binary"
	"fmt"
	"io"
	"math"
	"math/bits"

	"nhooyr.io/websocket/internal/errd"
)

// opcode represents a WebSocket opcode.
type opcode int

// https://tools.ietf.org/html/rfc6455#section-11.8.
const (
	opContinuation opcode = iota
	opText
	opBinary
	// 3 - 7 are reserved for further non-control frames.
	_
	_
	_
	_
	_
	opClose
	opPing
	opPong
	// 11-16 are reserved for further control frames.
)

// header represents a WebSocket frame header.
// See https://tools.ietf.org/html/rfc6455#section-5.2.
type header struct {
	fin    bool
	rsv1   bool
	rsv2   bool
	rsv3   bool
	opcode opcode

	payloadLength int64

	masked  bool
	maskKey uint32
}

// readFrameHeader reads a header from the reader.
// See https://tools.ietf.org/html/rfc6455#section-5.2.
func readFrameHeader(r *bufio.Reader, readBuf []byte) (h header, err error) {
	defer errd.Wrap(&err, "failed to read frame header")

	b, err := r.ReadByte()
	if err != nil {
		return header{}, err
	}

	h.fin = b&(1<<7) != 0
	h.rsv1 = b&(1<<6) != 0
	h.rsv2 = b&(1<<5) != 0
	h.rsv3 = b&(1<<4) != 0

	h.opcode = opcode(b & 0xf)

	b, err = r.ReadByte()
	if err != nil {
		return header{}, err
	}

	h.masked = b&(1<<7) != 0

	payloadLength := b &^ (1 << 7)
	switch {
	case payloadLength < 126:
		h.payloadLength = int64(payloadLength)
	case payloadLength == 126:
		_, err = io.ReadFull(r, readBuf[:2])
		h.payloadLength = int64(binary.BigEndian.Uint16(readBuf))
	case payloadLength == 127:
		_, err = io.ReadFull(r, readBuf)
		h.payloadLength = int64(binary.BigEndian.Uint64(readBuf))
	}
	if err != nil {
		return header{}, err
	}

	if h.payloadLength < 0 {
		return header{}, fmt.Errorf("received negative payload length: %v", h.payloadLength)
	}

	if h.masked {
		_, err = io.ReadFull(r, readBuf[:4])
		if err != nil {
			return header{}, err
		}
		h.maskKey = binary.LittleEndian.Uint32(readBuf)
	}

	return h, nil
}

// maxControlPayload is the maximum length of a control frame payload.
// See https://tools.ietf.org/html/rfc6455#section-5.5.
const maxControlPayload = 125

// writeFrameHeader writes the bytes of the header to w.
// See https://tools.ietf.org/html/rfc6455#section-5.2
func writeFrameHeader(h header, w *bufio.Writer, buf []byte) (err error) {
	defer errd.Wrap(&err, "failed to write frame header")

	var b byte
	if h.fin {
		b |= 1 << 7
	}
	if h.rsv1 {
		b |= 1 << 6
	}
	if h.rsv2 {
		b |= 1 << 5
	}
	if h.rsv3 {
		b |= 1 << 4
	}

	b |= byte(h.opcode)

	err = w.WriteByte(b)
	if err != nil {
		return err
	}

	lengthByte := byte(0)
	if h.masked {
		lengthByte |= 1 << 7
	}

	switch {
	case h.payloadLength > math.MaxUint16:
		lengthByte |= 127
	case h.payloadLength > 125:
		lengthByte |= 126
	case h.payloadLength >= 0:
		lengthByte |= byte(h.payloadLength)
	}
	err = w.WriteByte(lengthByte)
	if err != nil {
		return err
	}

	switch {
	case h.payloadLength > math.MaxUint16:
		binary.BigEndian.PutUint64(buf, uint64(h.payloadLength))
		_, err = w.Write(buf)
	case h.payloadLength > 125:
		binary.BigEndian.PutUint16(buf, uint16(h.payloadLength))
		_, err = w.Write(buf[:2])
	}
	if err != nil {
		return err
	}

	if h.masked {
		binary.LittleEndian.PutUint32(buf, h.maskKey)
		_, err = w.Write(buf[:4])
		if err != nil {
			return err
		}
	}

	return nil
}

// mask applies the WebSocket masking algorithm to p
// with the given key.
// See https://tools.ietf.org/html/rfc6455#section-5.3
//
// The returned value is the correctly rotated key to
// to continue to mask/unmask the message.
//
// It is optimized for LittleEndian and expects the key
// to be in little endian.
//
// See https://github.com/golang/go/issues/31586
func mask(key uint32, b []byte) uint32 {
	if len(b) >= 8 {
		key64 := uint64(key)<<32 | uint64(key)

		// At some point in the future we can clean these unrolled loops up.
		// See https://github.com/golang/go/issues/31586#issuecomment-487436401

		// Then we xor until b is less than 128 bytes.
		for len(b) >= 128 {
			v := binary.LittleEndian.Uint64(b)
			binary.LittleEndian.PutUint64(b, v^key64)
			v = binary.LittleEndian.Uint64(b[8:16])
			binary.LittleEndian.PutUint64(b[8:16], v^key64)
			v = binary.LittleEndian.Uint64(b[16:24])
			binary.LittleEndian.PutUint64(b[16:24], v^key64)
			v = binary.LittleEndian.Uint64(b[24:32])
			binary.LittleEndian.PutUint64(b[24:32], v^key64)
			v = binary.LittleEndian.Uint64(b[32:40])
			binary.LittleEndian.PutUint64(b[32:40], v^key64)
			v = binary.LittleEndian.Uint64(b[40:48])
			binary.LittleEndian.PutUint64(b[40:48], v^key64)
			v = binary.LittleEndian.Uint64(b[48:56])
			binary.LittleEndian.PutUint64(b[48:56], v^key64)
			v = binary.LittleEndian.Uint64(b[56:64])
			binary.LittleEndian.PutUint64(b[56:64], v^key64)
			v = binary.LittleEndian.Uint64(b[64:72])
			binary.LittleEndian.PutUint64(b[64:72], v^key64)
			v = binary.LittleEndian.Uint64(b[72:80])
			binary.LittleEndian.PutUint64(b[72:80], v^key64)
			v = binary.LittleEndian.Uint64(b[80:88])
			binary.LittleEndian.PutUint64(b[80:88], v^key64)
			v = binary.LittleEndian.Uint64(b[88:96])
			binary.LittleEndian.PutUint64(b[88:96], v^key64)
			v = binary.LittleEndian.Uint64(b[96:104])
			binary.LittleEndian.PutUint64(b[96:104], v^key64)
			v = binary.LittleEndian.Uint64(b[104:112])
			binary.LittleEndian.PutUint64(b[104:112], v^key64)
			v = binary.LittleEndian.Uint64(b[112:120])
			binary.LittleEndian.PutUint64(b[112:120], v^key64)
			v = binary.LittleEndian.Uint64(b[120:128])
			binary.LittleEndian.PutUint64(b[120:128], v^key64)
			b = b[128:]
		}

		// Then we xor until b is less than 64 bytes.
		for len(b) >= 64 {
			v := binary.LittleEndian.Uint64(b)
			binary.LittleEndian.PutUint64(b, v^key64)
			v = binary.LittleEndian.Uint64(b[8:16])
			binary.LittleEndian.PutUint64(b[8:16], v^key64)
			v = binary.LittleEndian.Uint64(b[16:24])
			binary.LittleEndian.PutUint64(b[16:24], v^key64)
			v = binary.LittleEndian.Uint64(b[24:32])
			binary.LittleEndian.PutUint64(b[24:32], v^key64)
			v = binary.LittleEndian.Uint64(b[32:40])
			binary.LittleEndian.PutUint64(b[32:40], v^key64)
			v = binary.LittleEndian.Uint64(b[40:48])
			binary.LittleEndian.PutUint64(b[40:48], v^key64)
			v = binary.LittleEndian.Uint64(b[48:56])
			binary.LittleEndian.PutUint64(b[48:56], v^key64)
			v = binary.LittleEndian.Uint64(b[56:64])
			binary.LittleEndian.PutUint64(b[56:64], v^key64)
			b = b[64:]
		}

		// Then we xor until b is less than 32 bytes.
		for len(b) >= 32 {
			v := binary.LittleEndian.Uint64(b)
			binary.LittleEndian.PutUint64(b, v^key64)
			v = binary.LittleEndian.Uint64(b[8:16])
			binary.LittleEndian.PutUint64(b[8:16], v^key64)
			v = binary.LittleEndian.Uint64(b[16:24])
			binary.LittleEndian.PutUint64(b[16:24], v^key64)
			v = binary.LittleEndian.Uint64(b[24:32])
			binary.LittleEndian.PutUint64(b[24:32], v^key64)
			b = b[32:]
		}

		// Then we xor until b is less than 16 bytes.
		for len(b) >= 16 {
			v := binary.LittleEndian.Uint64(b)
			binary.LittleEndian.PutUint64(b, v^key64)
			v = binary.LittleEndian.Uint64(b[8:16])
			binary.LittleEndian.PutUint64(b[8:16], v^key64)
			b = b[16:]
		}

		// Then we xor until b is less than 8 bytes.
		for len(b) >= 8 {
			v := binary.LittleEndian.Uint64(b)
			binary.LittleEndian.PutUint64(b, v^key64)
			b = b[8:]
		}
	}

	// Then we xor until b is less than 4 bytes.
	for len(b) >= 4 {
		v := binary.LittleEndian.Uint32(b)
		binary.LittleEndian.PutUint32(b, v^key)
		b = b[4:]
	}

	// xor remaining bytes.
	for i := range b {
		b[i] ^= byte(key)
		key = bits.RotateLeft32(key, -8)
	}

	return key
}

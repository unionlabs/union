package zstd

import (
	"encoding/binary"
	"errors"
	"fmt"
	"io"

	"github.com/klauspost/compress/huff0"
)

type dict struct {
	id uint32

	litEnc              *huff0.Scratch
	llDec, ofDec, mlDec sequenceDec
	//llEnc, ofEnc, mlEnc []*fseEncoder
	offsets [3]int
	content []byte
}

const dictMagic = "\x37\xa4\x30\xec"

// Maximum dictionary size for the reference implementation (1.5.3) is 2 GiB.
const dictMaxLength = 1 << 31

// ID returns the dictionary id or 0 if d is nil.
func (d *dict) ID() uint32 {
	if d == nil {
		return 0
	}
	return d.id
}

// ContentSize returns the dictionary content size or 0 if d is nil.
func (d *dict) ContentSize() int {
	if d == nil {
		return 0
	}
	return len(d.content)
}

// Content returns the dictionary content.
func (d *dict) Content() []byte {
	if d == nil {
		return nil
	}
	return d.content
}

// Offsets returns the initial offsets.
func (d *dict) Offsets() [3]int {
	if d == nil {
		return [3]int{}
	}
	return d.offsets
}

// LitEncoder returns the literal encoder.
func (d *dict) LitEncoder() *huff0.Scratch {
	if d == nil {
		return nil
	}
	return d.litEnc
}

// Load a dictionary as described in
// https://github.com/facebook/zstd/blob/master/doc/zstd_compression_format.md#dictionary-format
func loadDict(b []byte) (*dict, error) {
	// Check static field size.
	if len(b) <= 8+(3*4) {
		return nil, io.ErrUnexpectedEOF
	}
	d := dict{
		llDec: sequenceDec{fse: &fseDecoder{}},
		ofDec: sequenceDec{fse: &fseDecoder{}},
		mlDec: sequenceDec{fse: &fseDecoder{}},
	}
	if string(b[:4]) != dictMagic {
		return nil, ErrMagicMismatch
	}
	d.id = binary.LittleEndian.Uint32(b[4:8])
	if d.id == 0 {
		return nil, errors.New("dictionaries cannot have ID 0")
	}

	// Read literal table
	var err error
	d.litEnc, b, err = huff0.ReadTable(b[8:], nil)
	if err != nil {
		return nil, fmt.Errorf("loading literal table: %w", err)
	}
	d.litEnc.Reuse = huff0.ReusePolicyMust

	br := byteReader{
		b:   b,
		off: 0,
	}
	readDec := func(i tableIndex, dec *fseDecoder) error {
		if err := dec.readNCount(&br, uint16(maxTableSymbol[i])); err != nil {
			return err
		}
		if br.overread() {
			return io.ErrUnexpectedEOF
		}
		err = dec.transform(symbolTableX[i])
		if err != nil {
			println("Transform table error:", err)
			return err
		}
		if debugDecoder || debugEncoder {
			println("Read table ok", "symbolLen:", dec.symbolLen)
		}
		// Set decoders as predefined so they aren't reused.
		dec.preDefined = true
		return nil
	}

	if err := readDec(tableOffsets, d.ofDec.fse); err != nil {
		return nil, err
	}
	if err := readDec(tableMatchLengths, d.mlDec.fse); err != nil {
		return nil, err
	}
	if err := readDec(tableLiteralLengths, d.llDec.fse); err != nil {
		return nil, err
	}
	if br.remain() < 12 {
		return nil, io.ErrUnexpectedEOF
	}

	d.offsets[0] = int(br.Uint32())
	br.advance(4)
	d.offsets[1] = int(br.Uint32())
	br.advance(4)
	d.offsets[2] = int(br.Uint32())
	br.advance(4)
	if d.offsets[0] <= 0 || d.offsets[1] <= 0 || d.offsets[2] <= 0 {
		return nil, errors.New("invalid offset in dictionary")
	}
	d.content = make([]byte, br.remain())
	copy(d.content, br.unread())
	if d.offsets[0] > len(d.content) || d.offsets[1] > len(d.content) || d.offsets[2] > len(d.content) {
		return nil, fmt.Errorf("initial offset bigger than dictionary content size %d, offsets: %v", len(d.content), d.offsets)
	}

	return &d, nil
}

// InspectDictionary loads a zstd dictionary and provides functions to inspect the content.
func InspectDictionary(b []byte) (interface {
	ID() uint32
	ContentSize() int
	Content() []byte
	Offsets() [3]int
	LitEncoder() *huff0.Scratch
}, error) {
	initPredefined()
	d, err := loadDict(b)
	return d, err
}

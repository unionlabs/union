// Copyright 2011 The Go Authors.  All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Package libqrencode wraps the C libqrencode library.
// The qr package (in this package's parent directory)
// does not use any C wrapping.  This code is here only
// for use during that package's tests.
package libqrencode

/*
#cgo LDFLAGS: -lqrencode
#include <qrencode.h>
*/
import "C"

import (
	"fmt"
	"image"
	"image/color"
	"unsafe"
)

type Version int

type Mode int

const (
	Numeric      Mode = C.QR_MODE_NUM
	Alphanumeric Mode = C.QR_MODE_AN
	EightBit     Mode = C.QR_MODE_8
)

type Level int

const (
	L Level = C.QR_ECLEVEL_L
	M Level = C.QR_ECLEVEL_M
	Q Level = C.QR_ECLEVEL_Q
	H Level = C.QR_ECLEVEL_H
)

type Pixel int

const (
	Black Pixel = 1 << iota
	DataECC
	Format
	PVersion
	Timing
	Alignment
	Finder
	NonData
)

type Code struct {
	Version int
	Width   int
	Pixel   [][]Pixel
	Scale   int
}

func (*Code) ColorModel() color.Model {
	return color.RGBAModel
}

func (c *Code) Bounds() image.Rectangle {
	d := (c.Width + 8) * c.Scale
	return image.Rect(0, 0, d, d)
}

var (
	white  color.Color = color.RGBA{0xFF, 0xFF, 0xFF, 0xFF}
	black  color.Color = color.RGBA{0x00, 0x00, 0x00, 0xFF}
	blue   color.Color = color.RGBA{0x00, 0x00, 0x80, 0xFF}
	red    color.Color = color.RGBA{0xFF, 0x40, 0x40, 0xFF}
	yellow color.Color = color.RGBA{0xFF, 0xFF, 0x00, 0xFF}
	gray   color.Color = color.RGBA{0x80, 0x80, 0x80, 0xFF}
	green  color.Color = color.RGBA{0x22, 0x8B, 0x22, 0xFF}
)

func (c *Code) At(x, y int) color.Color {
	x = x/c.Scale - 4
	y = y/c.Scale - 4
	if 0 <= x && x < c.Width && 0 <= y && y < c.Width {
		switch p := c.Pixel[y][x]; {
		case p&Black == 0:
			// nothing
		case p&DataECC != 0:
			return black
		case p&Format != 0:
			return blue
		case p&PVersion != 0:
			return red
		case p&Timing != 0:
			return yellow
		case p&Alignment != 0:
			return gray
		case p&Finder != 0:
			return green
		}
	}
	return white
}

type Chunk struct {
	Mode Mode
	Text string
}

func Encode(version Version, level Level, mode Mode, text string) (*Code, error) {
	return EncodeChunk(version, level, Chunk{mode, text})
}

func EncodeChunk(version Version, level Level, chunk ...Chunk) (*Code, error) {
	qi, err := C.QRinput_new2(C.int(version), C.QRecLevel(level))
	if qi == nil {
		return nil, fmt.Errorf("QRinput_new2: %v", err)
	}
	defer C.QRinput_free(qi)
	for _, ch := range chunk {
		data := []byte(ch.Text)
		n, err := C.QRinput_append(qi, C.QRencodeMode(ch.Mode), C.int(len(data)), (*C.uchar)(&data[0]))
		if n < 0 {
			return nil, fmt.Errorf("QRinput_append %q: %v", data, err)
		}
	}

	qc, err := C.QRcode_encodeInput(qi)
	if qc == nil {
		return nil, fmt.Errorf("QRinput_encodeInput: %v", err)
	}

	c := &Code{
		Version: int(qc.version),
		Width:   int(qc.width),
		Scale:   16,
	}
	pix := make([]Pixel, c.Width*c.Width)
	cdat := (*[1000 * 1000]byte)(unsafe.Pointer(qc.data))[:len(pix)]
	for i := range pix {
		pix[i] = Pixel(cdat[i])
	}
	c.Pixel = make([][]Pixel, c.Width)
	for i := range c.Pixel {
		c.Pixel[i] = pix[i*c.Width : (i+1)*c.Width]
	}
	return c, nil
}

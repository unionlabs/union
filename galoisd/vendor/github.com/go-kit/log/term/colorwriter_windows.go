// The code in this file is adapted from github.com/mattn/go-colorable.

package term

import (
	"bytes"
	"fmt"
	"io"
	"strconv"
	"strings"
	"syscall"
	"unsafe"
)

type colorWriter struct {
	out     io.Writer
	handle  syscall.Handle
	lastbuf bytes.Buffer
	oldattr word
}

// NewColorWriter returns an io.Writer that writes to w and provides cross
// platform support for ANSI color codes. If w is not a terminal it is
// returned unmodified.
func NewColorWriter(w io.Writer) io.Writer {
	if !IsConsole(w) {
		return w
	}

	var csbi consoleScreenBufferInfo
	handle := syscall.Handle(w.(fder).Fd())
	procGetConsoleScreenBufferInfo.Call(uintptr(handle), uintptr(unsafe.Pointer(&csbi)))

	return &colorWriter{
		out:     w,
		handle:  handle,
		oldattr: csbi.attributes,
	}
}

func (w *colorWriter) Write(data []byte) (n int, err error) {
	var csbi consoleScreenBufferInfo
	procGetConsoleScreenBufferInfo.Call(uintptr(w.handle), uintptr(unsafe.Pointer(&csbi)))

	er := bytes.NewBuffer(data)
loop:
	for {
		r1, _, _ := procGetConsoleScreenBufferInfo.Call(uintptr(w.handle), uintptr(unsafe.Pointer(&csbi)))
		if r1 == 0 {
			break loop
		}

		c1, _, err := er.ReadRune()
		if err != nil {
			break loop
		}
		if c1 != 0x1b {
			fmt.Fprint(w.out, string(c1))
			continue
		}
		c2, _, err := er.ReadRune()
		if err != nil {
			w.lastbuf.WriteRune(c1)
			break loop
		}
		if c2 != 0x5b {
			w.lastbuf.WriteRune(c1)
			w.lastbuf.WriteRune(c2)
			continue
		}

		var buf bytes.Buffer
		var m rune
		for {
			c, _, err := er.ReadRune()
			if err != nil {
				w.lastbuf.WriteRune(c1)
				w.lastbuf.WriteRune(c2)
				w.lastbuf.Write(buf.Bytes())
				break loop
			}
			if ('a' <= c && c <= 'z') || ('A' <= c && c <= 'Z') || c == '@' {
				m = c
				break
			}
			buf.Write([]byte(string(c)))
		}

		switch m {
		case 'm':
			attr := csbi.attributes
			cs := buf.String()
			if cs == "" {
				procSetConsoleTextAttribute.Call(uintptr(w.handle), uintptr(w.oldattr))
				continue
			}
			token := strings.Split(cs, ";")
			intensityMode := word(0)
			for _, ns := range token {
				if n, err = strconv.Atoi(ns); err == nil {
					switch {
					case n == 0:
						attr = w.oldattr
					case n == 1:
						attr |= intensityMode
					case 30 <= n && n <= 37:
						attr = (attr & backgroundMask)
						if (n-30)&1 != 0 {
							attr |= foregroundRed
						}
						if (n-30)&2 != 0 {
							attr |= foregroundGreen
						}
						if (n-30)&4 != 0 {
							attr |= foregroundBlue
						}
						intensityMode = foregroundIntensity
					case n == 39: // reset foreground color
						attr &= backgroundMask
						attr |= w.oldattr & foregroundMask
					case 40 <= n && n <= 47:
						attr = (attr & foregroundMask)
						if (n-40)&1 != 0 {
							attr |= backgroundRed
						}
						if (n-40)&2 != 0 {
							attr |= backgroundGreen
						}
						if (n-40)&4 != 0 {
							attr |= backgroundBlue
						}
						intensityMode = backgroundIntensity
					case n == 49: // reset background color
						attr &= foregroundMask
						attr |= w.oldattr & backgroundMask
					}
					procSetConsoleTextAttribute.Call(uintptr(w.handle), uintptr(attr))
				}
			}
		}
	}
	return len(data) - w.lastbuf.Len(), nil
}

var (
	procGetConsoleScreenBufferInfo = kernel32.NewProc("GetConsoleScreenBufferInfo")
	procSetConsoleTextAttribute    = kernel32.NewProc("SetConsoleTextAttribute")
)

const (
	foregroundBlue      = 0x1
	foregroundGreen     = 0x2
	foregroundRed       = 0x4
	foregroundIntensity = 0x8
	foregroundMask      = (foregroundRed | foregroundBlue | foregroundGreen | foregroundIntensity)
	backgroundBlue      = 0x10
	backgroundGreen     = 0x20
	backgroundRed       = 0x40
	backgroundIntensity = 0x80
	backgroundMask      = (backgroundRed | backgroundBlue | backgroundGreen | backgroundIntensity)
)

type (
	wchar uint16 //lint:ignore U1000 unused
	short int16  //lint:ignore U1000 unused
	dword uint32 //lint:ignore U1000 unused
	word  uint16 //lint:ignore U1000 unused
)

type coord struct {
	x short //lint:ignore U1000 unused
	y short //lint:ignore U1000 unused
}

type smallRect struct {
	left   short //lint:ignore U1000 unused
	top    short //lint:ignore U1000 unused
	right  short //lint:ignore U1000 unused
	bottom short //lint:ignore U1000 unused
}

type consoleScreenBufferInfo struct {
	size              coord
	cursorPosition    coord
	attributes        word
	window            smallRect
	maximumWindowSize coord
}

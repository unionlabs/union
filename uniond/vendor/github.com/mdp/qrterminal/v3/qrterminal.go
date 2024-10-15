package qrterminal

import (
	"bytes"
	"fmt"
	"io"
	"os"
	"strings"

	"golang.org/x/term"
	"rsc.io/qr"
)

const WHITE = "\033[47m  \033[0m"
const BLACK = "\033[40m  \033[0m"

// Use ascii blocks to form the QR Code
const BLACK_WHITE = "▄"
const BLACK_BLACK = " "
const WHITE_BLACK = "▀"
const WHITE_WHITE = "█"

// Level - the QR Code's redundancy level
const H = qr.H
const M = qr.M
const L = qr.L

// default is 4-pixel-wide white quiet zone
const QUIET_ZONE = 4

// Sixel Support Control Sequence
// Color 0: Black Color 1: White
const SIXEL_BEGIN = "\x1bPq\n#0;2;0;0;0#1;2;100;100;100\n"
const SIXEL_END = "\x1b\\"

// Sixel Block Size, should be always greater than 6.
const SIXEL_BLOCK_SIZE = 12

// Config for generating a barcode
type Config struct {
	Level          qr.Level
	Writer         io.Writer
	HalfBlocks     bool
	BlackChar      string
	BlackWhiteChar string
	WhiteChar      string
	WhiteBlackChar string
	QuietZone      int
	WithSixel      bool
}

func IsSixelSupported(w io.Writer) bool {
	if w != os.Stdout {
		return false
	}
	stdout := os.Stdout
	if !term.IsTerminal(int(stdout.Fd())) {
		return false
	}
	_, err := stdout.Write([]byte("\x1B[c"))
	if err != nil {
		return false
	}
	buf := make([]byte, 1024)
	//set echo off
	raw, err := term.MakeRaw(int(stdout.Fd()))
	defer term.Restore(int(stdout.Fd()), raw)
	_, err = stdout.Read(buf)
	if err != nil {
		return false
	}
	for _, b := range string(buf) {
		if b == '4' {
			//Found Sixel Support
			return true
		}
	}
	return false
}

func (c *Config) writeSixel(w io.Writer, code *qr.Code) {
	line := SIXEL_BLOCK_SIZE / 6
	// Frame the barcode in a 1 pixel border
	w.Write([]byte(SIXEL_BEGIN))
	w.Write([]byte(stringRepeat(fmt.Sprintf("#1!%d~-\n", SIXEL_BLOCK_SIZE*(code.Size+c.QuietZone*2)), c.QuietZone*line))) // top border
	for i := 0; i <= code.Size; i++ {
		flag := -1
		repeat := 0
		content := bytes.NewBufferString("")
		if c.QuietZone > 0 {
			content.WriteString(fmt.Sprintf("#1!%d~", SIXEL_BLOCK_SIZE*c.QuietZone)) // left border
		}
		for j := 0; j <= code.Size; j++ {
			if code.Black(j, i) {
				if flag == 1 {
					content.WriteString(fmt.Sprintf("#1!%d~", SIXEL_BLOCK_SIZE*repeat))
					repeat = 0
				}
				flag = 0
				repeat++
			} else {
				if flag == 0 {
					content.WriteString(fmt.Sprintf("#0!%d~", SIXEL_BLOCK_SIZE*repeat))
					repeat = 0
				}
				flag = 1
				repeat++
			}
		}
		if repeat > 0 {
			content.WriteString(fmt.Sprintf("#%d!%d~", flag, SIXEL_BLOCK_SIZE*repeat))
		}
		if c.QuietZone > 1 {
			content.WriteString(fmt.Sprintf("#1!%d~", SIXEL_BLOCK_SIZE*(c.QuietZone-1))) // right border
		}
		content.WriteString("-\n")
		for i := 0; i < line; i++ {
			w.Write(content.Bytes())
		}
	}
	w.Write([]byte(stringRepeat(fmt.Sprintf("#1!%d~-\n", SIXEL_BLOCK_SIZE*(code.Size+c.QuietZone*2)), (c.QuietZone-1)*line))) // bottom border
	if c.QuietZone > 1 {
		w.Write([]byte(fmt.Sprintf("#1!%d~-", SIXEL_BLOCK_SIZE*(code.Size+c.QuietZone*2)))) // bottom border last line, Fix on iTerm2
	}
	defer w.Write([]byte(SIXEL_END))
}

func (c *Config) writeFullBlocks(w io.Writer, code *qr.Code) {
	white := c.WhiteChar
	black := c.BlackChar

	// Frame the barcode in a 1 pixel border
	w.Write([]byte(stringRepeat(stringRepeat(white,
		code.Size+c.QuietZone*2)+"\n", c.QuietZone))) // top border
	for i := 0; i <= code.Size; i++ {
		w.Write([]byte(stringRepeat(white, c.QuietZone))) // left border
		for j := 0; j <= code.Size; j++ {
			if code.Black(j, i) {
				w.Write([]byte(black))
			} else {
				w.Write([]byte(white))
			}
		}
		w.Write([]byte(stringRepeat(white, c.QuietZone-1) + "\n")) // right border
	}
	w.Write([]byte(stringRepeat(stringRepeat(white,
		code.Size+c.QuietZone*2)+"\n", c.QuietZone-1))) // bottom border
}

func (c *Config) writeHalfBlocks(w io.Writer, code *qr.Code) {
	ww := c.WhiteChar
	bb := c.BlackChar
	wb := c.WhiteBlackChar
	bw := c.BlackWhiteChar
	// Frame the barcode in a 4 pixel border
	// top border
	if c.QuietZone%2 != 0 {
		w.Write([]byte(stringRepeat(bw, code.Size+c.QuietZone*2) + "\n"))
		w.Write([]byte(stringRepeat(stringRepeat(ww,
			code.Size+c.QuietZone*2)+"\n", c.QuietZone/2)))
	} else {
		w.Write([]byte(stringRepeat(stringRepeat(ww,
			code.Size+c.QuietZone*2)+"\n", c.QuietZone/2)))
	}
	for i := 0; i <= code.Size; i += 2 {
		w.Write([]byte(stringRepeat(ww, c.QuietZone))) // left border
		for j := 0; j <= code.Size; j++ {
			next_black := false
			if i+1 < code.Size {
				next_black = code.Black(j, i+1)
			}
			curr_black := code.Black(j, i)
			if curr_black && next_black {
				w.Write([]byte(bb))
			} else if curr_black && !next_black {
				w.Write([]byte(bw))
			} else if !curr_black && !next_black {
				w.Write([]byte(ww))
			} else {
				w.Write([]byte(wb))
			}
		}
		w.Write([]byte(stringRepeat(ww, c.QuietZone-1) + "\n")) // right border
	}
	// bottom border
	if c.QuietZone%2 == 0 {
		w.Write([]byte(stringRepeat(stringRepeat(ww,
			code.Size+c.QuietZone*2)+"\n", c.QuietZone/2-1)))
		w.Write([]byte(stringRepeat(wb, code.Size+c.QuietZone*2) + "\n"))
	} else {
		w.Write([]byte(stringRepeat(stringRepeat(ww,
			code.Size+c.QuietZone*2)+"\n", c.QuietZone/2)))
	}
}

func stringRepeat(s string, count int) string {
	if count <= 0 {
		return ""
	}
	return strings.Repeat(s, count)
}

// GenerateWithConfig expects a string to encode and a config
func GenerateWithConfig(text string, config Config) {
	if config.QuietZone < 1 {
		config.QuietZone = 1 // at least 1-pixel-wide white quiet zone
	}
	w := config.Writer
	code, _ := qr.Encode(text, config.Level)
	if config.HalfBlocks {
		config.writeHalfBlocks(w, code)
	} else {
		if config.WithSixel {
			config.writeSixel(w, code)
		} else {
			config.writeFullBlocks(w, code)
		}
	}
}

// Generate a QR Code and write it out to io.Writer
func Generate(text string, l qr.Level, w io.Writer) {
	config := Config{
		Level:     l,
		Writer:    w,
		BlackChar: BLACK,
		WhiteChar: WHITE,
		QuietZone: QUIET_ZONE,
	}
	config.WithSixel = IsSixelSupported(w)
	GenerateWithConfig(text, config)
}

// Generate a QR Code with half blocks and write it out to io.Writer
func GenerateHalfBlock(text string, l qr.Level, w io.Writer) {
	config := Config{
		Level:          l,
		Writer:         w,
		HalfBlocks:     true,
		BlackChar:      BLACK_BLACK,
		WhiteBlackChar: WHITE_BLACK,
		WhiteChar:      WHITE_WHITE,
		BlackWhiteChar: BLACK_WHITE,
		QuietZone:      QUIET_ZONE,
	}
	GenerateWithConfig(text, config)
}

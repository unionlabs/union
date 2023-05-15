package term

import (
	"bytes"
	"fmt"
	"io"
	"sync"

	"github.com/go-kit/log"
)

// Color represents an ANSI color. The zero value is Default.
type Color uint8

// ANSI colors.
const (
	Default = Color(iota)

	Black
	DarkRed
	DarkGreen
	Brown
	DarkBlue
	DarkMagenta
	DarkCyan
	Gray

	DarkGray
	Red
	Green
	Yellow
	Blue
	Magenta
	Cyan
	White

	numColors
)

// For more on ANSI escape codes see
// https://en.wikipedia.org/wiki/ANSI_escape_code. See in particular
// https://en.wikipedia.org/wiki/ANSI_escape_code#Colors.

var (
	resetColorBytes = []byte("\x1b[39;49;22m")
	fgColorBytes    [][]byte
	bgColorBytes    [][]byte
)

func init() {
	// Default
	fgColorBytes = append(fgColorBytes, []byte("\x1b[39m"))
	bgColorBytes = append(bgColorBytes, []byte("\x1b[49m"))

	// dark colors
	for color := Black; color < DarkGray; color++ {
		fgColorBytes = append(fgColorBytes, []byte(fmt.Sprintf("\x1b[%dm", 30+color-Black)))
		bgColorBytes = append(bgColorBytes, []byte(fmt.Sprintf("\x1b[%dm", 40+color-Black)))
	}

	// bright colors
	for color := DarkGray; color < numColors; color++ {
		fgColorBytes = append(fgColorBytes, []byte(fmt.Sprintf("\x1b[%d;1m", 30+color-DarkGray)))
		bgColorBytes = append(bgColorBytes, []byte(fmt.Sprintf("\x1b[%d;1m", 40+color-DarkGray)))
	}
}

// FgBgColor represents a foreground and background color.
type FgBgColor struct {
	Fg, Bg Color
}

func (c FgBgColor) isZero() bool {
	return c.Fg == Default && c.Bg == Default
}

// NewColorLogger returns a Logger which writes colored logs to w. ANSI color
// codes for the colors returned by color are added to the formatted output
// from the Logger returned by newLogger and the combined result written to w.
func NewColorLogger(w io.Writer, newLogger func(io.Writer) log.Logger, color func(keyvals ...interface{}) FgBgColor) log.Logger {
	if color == nil {
		panic("color func nil")
	}
	return &colorLogger{
		w:             w,
		newLogger:     newLogger,
		color:         color,
		bufPool:       sync.Pool{New: func() interface{} { return &loggerBuf{} }},
		noColorLogger: newLogger(w),
	}
}

type colorLogger struct {
	w             io.Writer
	newLogger     func(io.Writer) log.Logger
	color         func(keyvals ...interface{}) FgBgColor
	bufPool       sync.Pool
	noColorLogger log.Logger
}

func (l *colorLogger) Log(keyvals ...interface{}) error {
	color := l.color(keyvals...)
	if color.isZero() {
		return l.noColorLogger.Log(keyvals...)
	}

	lb := l.getLoggerBuf()
	defer l.putLoggerBuf(lb)
	if color.Fg != Default {
		lb.buf.Write(fgColorBytes[color.Fg])
	}
	if color.Bg != Default {
		lb.buf.Write(bgColorBytes[color.Bg])
	}
	err := lb.logger.Log(keyvals...)
	if err != nil {
		return err
	}
	if color.Fg != Default || color.Bg != Default {
		lb.buf.Write(resetColorBytes)
	}
	_, err = io.Copy(l.w, lb.buf)
	return err
}

type loggerBuf struct {
	buf    *bytes.Buffer
	logger log.Logger
}

func (l *colorLogger) getLoggerBuf() *loggerBuf {
	lb := l.bufPool.Get().(*loggerBuf)
	if lb.buf == nil {
		lb.buf = &bytes.Buffer{}
		lb.logger = l.newLogger(lb.buf)
	} else {
		lb.buf.Reset()
	}
	return lb
}

func (l *colorLogger) putLoggerBuf(cb *loggerBuf) {
	l.bufPool.Put(cb)
}

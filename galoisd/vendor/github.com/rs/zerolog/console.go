package zerolog

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/mattn/go-colorable"
)

const (
	colorBlack = iota + 30
	colorRed
	colorGreen
	colorYellow
	colorBlue
	colorMagenta
	colorCyan
	colorWhite

	colorBold     = 1
	colorDarkGray = 90

	unknownLevel = "???"
)

var (
	consoleBufPool = sync.Pool{
		New: func() interface{} {
			return bytes.NewBuffer(make([]byte, 0, 100))
		},
	}
)

const (
	consoleDefaultTimeFormat = time.Kitchen
)

// Formatter transforms the input into a formatted string.
type Formatter func(interface{}) string

// ConsoleWriter parses the JSON input and writes it in an
// (optionally) colorized, human-friendly format to Out.
type ConsoleWriter struct {
	// Out is the output destination.
	Out io.Writer

	// NoColor disables the colorized output.
	NoColor bool

	// TimeFormat specifies the format for timestamp in output.
	TimeFormat string

	// TimeLocation tells ConsoleWriter’s default FormatTimestamp
	// how to localize the time.
	TimeLocation *time.Location

	// PartsOrder defines the order of parts in output.
	PartsOrder []string

	// PartsExclude defines parts to not display in output.
	PartsExclude []string

	// FieldsOrder defines the order of contextual fields in output.
	FieldsOrder []string

	fieldIsOrdered map[string]int

	// FieldsExclude defines contextual fields to not display in output.
	FieldsExclude []string

	FormatTimestamp     Formatter
	FormatLevel         Formatter
	FormatCaller        Formatter
	FormatMessage       Formatter
	FormatFieldName     Formatter
	FormatFieldValue    Formatter
	FormatErrFieldName  Formatter
	FormatErrFieldValue Formatter

	FormatExtra func(map[string]interface{}, *bytes.Buffer) error

	FormatPrepare func(map[string]interface{}) error
}

// NewConsoleWriter creates and initializes a new ConsoleWriter.
func NewConsoleWriter(options ...func(w *ConsoleWriter)) ConsoleWriter {
	w := ConsoleWriter{
		Out:          os.Stdout,
		TimeFormat:   consoleDefaultTimeFormat,
		PartsOrder:   consoleDefaultPartsOrder(),
	}

	for _, opt := range options {
		opt(&w)
	}

	// Fix color on Windows
	if w.Out == os.Stdout || w.Out == os.Stderr {
		w.Out = colorable.NewColorable(w.Out.(*os.File))
	}

	return w
}

// Write transforms the JSON input with formatters and appends to w.Out.
func (w ConsoleWriter) Write(p []byte) (n int, err error) {
	// Fix color on Windows
	if w.Out == os.Stdout || w.Out == os.Stderr {
		w.Out = colorable.NewColorable(w.Out.(*os.File))
	}

	if w.PartsOrder == nil {
		w.PartsOrder = consoleDefaultPartsOrder()
	}

	var buf = consoleBufPool.Get().(*bytes.Buffer)
	defer func() {
		buf.Reset()
		consoleBufPool.Put(buf)
	}()

	var evt map[string]interface{}
	p = decodeIfBinaryToBytes(p)
	d := json.NewDecoder(bytes.NewReader(p))
	d.UseNumber()
	err = d.Decode(&evt)
	if err != nil {
		return n, fmt.Errorf("cannot decode event: %s", err)
	}

	if w.FormatPrepare != nil {
		err = w.FormatPrepare(evt)
		if err != nil {
			return n, err
		}
	}

	for _, p := range w.PartsOrder {
		w.writePart(buf, evt, p)
	}

	w.writeFields(evt, buf)

	if w.FormatExtra != nil {
		err = w.FormatExtra(evt, buf)
		if err != nil {
			return n, err
		}
	}

	err = buf.WriteByte('\n')
	if err != nil {
		return n, err
	}

	_, err = buf.WriteTo(w.Out)
	return len(p), err
}

// Call the underlying writer's Close method if it is an io.Closer. Otherwise
// does nothing.
func (w ConsoleWriter) Close() error {
	if closer, ok := w.Out.(io.Closer); ok {
		return closer.Close()
	}
	return nil
}

// writeFields appends formatted key-value pairs to buf.
func (w ConsoleWriter) writeFields(evt map[string]interface{}, buf *bytes.Buffer) {
	var fields = make([]string, 0, len(evt))
	for field := range evt {
		var isExcluded bool
		for _, excluded := range w.FieldsExclude {
			if field == excluded {
				isExcluded = true
				break
			}
		}
		if isExcluded {
			continue
		}

		switch field {
		case LevelFieldName, TimestampFieldName, MessageFieldName, CallerFieldName:
			continue
		}
		fields = append(fields, field)
	}

	if len(w.FieldsOrder) > 0 {
		w.orderFields(fields)
	} else {
		sort.Strings(fields)
	}

	// Write space only if something has already been written to the buffer, and if there are fields.
	if buf.Len() > 0 && len(fields) > 0 {
		buf.WriteByte(' ')
	}

	// Move the "error" field to the front
	ei := sort.Search(len(fields), func(i int) bool { return fields[i] >= ErrorFieldName })
	if ei < len(fields) && fields[ei] == ErrorFieldName {
		fields[ei] = ""
		fields = append([]string{ErrorFieldName}, fields...)
		var xfields = make([]string, 0, len(fields))
		for _, field := range fields {
			if field == "" { // Skip empty fields
				continue
			}
			xfields = append(xfields, field)
		}
		fields = xfields
	}

	for i, field := range fields {
		var fn Formatter
		var fv Formatter

		if field == ErrorFieldName {
			if w.FormatErrFieldName == nil {
				fn = consoleDefaultFormatErrFieldName(w.NoColor)
			} else {
				fn = w.FormatErrFieldName
			}

			if w.FormatErrFieldValue == nil {
				fv = consoleDefaultFormatErrFieldValue(w.NoColor)
			} else {
				fv = w.FormatErrFieldValue
			}
		} else {
			if w.FormatFieldName == nil {
				fn = consoleDefaultFormatFieldName(w.NoColor)
			} else {
				fn = w.FormatFieldName
			}

			if w.FormatFieldValue == nil {
				fv = consoleDefaultFormatFieldValue
			} else {
				fv = w.FormatFieldValue
			}
		}

		buf.WriteString(fn(field))

		switch fValue := evt[field].(type) {
		case string:
			if needsQuote(fValue) {
				buf.WriteString(fv(strconv.Quote(fValue)))
			} else {
				buf.WriteString(fv(fValue))
			}
		case json.Number:
			buf.WriteString(fv(fValue))
		default:
			b, err := InterfaceMarshalFunc(fValue)
			if err != nil {
				fmt.Fprintf(buf, colorize("[error: %v]", colorRed, w.NoColor), err)
			} else {
				fmt.Fprint(buf, fv(b))
			}
		}

		if i < len(fields)-1 { // Skip space for last field
			buf.WriteByte(' ')
		}
	}
}

// writePart appends a formatted part to buf.
func (w ConsoleWriter) writePart(buf *bytes.Buffer, evt map[string]interface{}, p string) {
	var f Formatter

	if w.PartsExclude != nil && len(w.PartsExclude) > 0 {
		for _, exclude := range w.PartsExclude {
			if exclude == p {
				return
			}
		}
	}

	switch p {
	case LevelFieldName:
		if w.FormatLevel == nil {
			f = consoleDefaultFormatLevel(w.NoColor)
		} else {
			f = w.FormatLevel
		}
	case TimestampFieldName:
		if w.FormatTimestamp == nil {
			f = consoleDefaultFormatTimestamp(w.TimeFormat, w.TimeLocation, w.NoColor)
		} else {
			f = w.FormatTimestamp
		}
	case MessageFieldName:
		if w.FormatMessage == nil {
			f = consoleDefaultFormatMessage(w.NoColor, evt[LevelFieldName])
		} else {
			f = w.FormatMessage
		}
	case CallerFieldName:
		if w.FormatCaller == nil {
			f = consoleDefaultFormatCaller(w.NoColor)
		} else {
			f = w.FormatCaller
		}
	default:
		if w.FormatFieldValue == nil {
			f = consoleDefaultFormatFieldValue
		} else {
			f = w.FormatFieldValue
		}
	}

	var s = f(evt[p])

	if len(s) > 0 {
		if buf.Len() > 0 {
			buf.WriteByte(' ') // Write space only if not the first part
		}
		buf.WriteString(s)
	}
}

// orderFields takes an array of field names and an array representing field order
// and returns an array with any ordered fields at the beginning, in order,
// and the remaining fields after in their original order.
func (w ConsoleWriter) orderFields(fields []string) {
	if w.fieldIsOrdered == nil {
		w.fieldIsOrdered = make(map[string]int)
		for i, fieldName := range w.FieldsOrder {
			w.fieldIsOrdered[fieldName] = i
		}
	}
	sort.Slice(fields, func(i, j int) bool {
		ii, iOrdered := w.fieldIsOrdered[fields[i]]
		jj, jOrdered := w.fieldIsOrdered[fields[j]]
		if iOrdered && jOrdered {
			return ii < jj
		}
		if iOrdered {
			return true
		}
		if jOrdered {
			return false
		}
		return fields[i] < fields[j]
	})
}

// needsQuote returns true when the string s should be quoted in output.
func needsQuote(s string) bool {
	for i := range s {
		if s[i] < 0x20 || s[i] > 0x7e || s[i] == ' ' || s[i] == '\\' || s[i] == '"' {
			return true
		}
	}
	return false
}

// colorize returns the string s wrapped in ANSI code c, unless disabled is true or c is 0.
func colorize(s interface{}, c int, disabled bool) string {
	e := os.Getenv("NO_COLOR")
	if e != "" || c == 0 {
		disabled = true
	}

	if disabled {
		return fmt.Sprintf("%s", s)
	}
	return fmt.Sprintf("\x1b[%dm%v\x1b[0m", c, s)
}

// ----- DEFAULT FORMATTERS ---------------------------------------------------

func consoleDefaultPartsOrder() []string {
	return []string{
		TimestampFieldName,
		LevelFieldName,
		CallerFieldName,
		MessageFieldName,
	}
}

func consoleDefaultFormatTimestamp(timeFormat string, location *time.Location, noColor bool) Formatter {
	if timeFormat == "" {
		timeFormat = consoleDefaultTimeFormat
	}
	if location == nil {
		location = time.Local
	}

	return func(i interface{}) string {
		t := "<nil>"
		switch tt := i.(type) {
		case string:
			ts, err := time.ParseInLocation(TimeFieldFormat, tt, location)
			if err != nil {
				t = tt
			} else {
				t = ts.In(location).Format(timeFormat)
			}
		case json.Number:
			i, err := tt.Int64()
			if err != nil {
				t = tt.String()
			} else {
				var sec, nsec int64

				switch TimeFieldFormat {
				case TimeFormatUnixNano:
					sec, nsec = 0, i
				case TimeFormatUnixMicro:
					sec, nsec = 0, int64(time.Duration(i)*time.Microsecond)
				case TimeFormatUnixMs:
					sec, nsec = 0, int64(time.Duration(i)*time.Millisecond)
				default:
					sec, nsec = i, 0
				}

				ts := time.Unix(sec, nsec)
				t = ts.In(location).Format(timeFormat)
			}
		}
		return colorize(t, colorDarkGray, noColor)
	}
}

func stripLevel(ll string) string {
	if len(ll) == 0 {
		return unknownLevel
	}
	if len(ll) > 3 {
		ll = ll[:3]
	}
	return strings.ToUpper(ll)
}

func consoleDefaultFormatLevel(noColor bool) Formatter {
	return func(i interface{}) string {
		if ll, ok := i.(string); ok {
			level, _ := ParseLevel(ll)
			fl, ok := FormattedLevels[level]
			if ok {
				return colorize(fl, LevelColors[level], noColor)
			}
			return stripLevel(ll)
		}
		if i == nil {
			return unknownLevel
		}
		return stripLevel(fmt.Sprintf("%s", i))
	}
}

func consoleDefaultFormatCaller(noColor bool) Formatter {
	return func(i interface{}) string {
		var c string
		if cc, ok := i.(string); ok {
			c = cc
		}
		if len(c) > 0 {
			if cwd, err := os.Getwd(); err == nil {
				if rel, err := filepath.Rel(cwd, c); err == nil {
					c = rel
				}
			}
			c = colorize(c, colorBold, noColor) + colorize(" >", colorCyan, noColor)
		}
		return c
	}
}

func consoleDefaultFormatMessage(noColor bool, level interface{}) Formatter {
	return func(i interface{}) string {
		if i == nil || i == "" {
			return ""
		}
		switch level {
		case LevelInfoValue, LevelWarnValue, LevelErrorValue, LevelFatalValue, LevelPanicValue:
			return colorize(fmt.Sprintf("%s", i), colorBold, noColor)
		default:
			return fmt.Sprintf("%s", i)
		}
	}
}

func consoleDefaultFormatFieldName(noColor bool) Formatter {
	return func(i interface{}) string {
		return colorize(fmt.Sprintf("%s=", i), colorCyan, noColor)
	}
}

func consoleDefaultFormatFieldValue(i interface{}) string {
	return fmt.Sprintf("%s", i)
}

func consoleDefaultFormatErrFieldName(noColor bool) Formatter {
	return func(i interface{}) string {
		return colorize(fmt.Sprintf("%s=", i), colorCyan, noColor)
	}
}

func consoleDefaultFormatErrFieldValue(noColor bool) Formatter {
	return func(i interface{}) string {
		return colorize(colorize(fmt.Sprintf("%s", i), colorBold, noColor), colorRed, noColor)
	}
}

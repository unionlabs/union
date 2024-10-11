package log

import (
	"bytes"
	"encoding/json"
	"fmt"
	"math/big"
	"reflect"
	"strconv"
	"strings"
	"sync"
	"sync/atomic"
	"time"
	"unicode/utf8"

	"github.com/holiman/uint256"
)

const (
	timeFormat        = "2006-01-02T15:04:05-0700"
	termTimeFormat    = "01-02|15:04:05.000"
	floatFormat       = 'f'
	termMsgJust       = 40
	termCtxMaxPadding = 40
)

// ResetGlobalState resets the fieldPadding, which is useful for producing
// predictable output.
func ResetGlobalState() {
	fieldPaddingLock.Lock()
	fieldPadding = make(map[string]int)
	fieldPaddingLock.Unlock()
}

// locationTrims are trimmed for display to avoid unwieldy log lines.
var locationTrims = []string{
	"github.com/ethereum/go-ethereum/",
}

// PrintOrigins sets or unsets log location (file:line) printing for terminal
// format output.
func PrintOrigins(print bool) {
	locationEnabled.Store(print)
	if print {
		stackEnabled.Store(true)
	}
}

// stackEnabled is an atomic flag controlling whether the log handler needs
// to store the callsite stack. This is needed in case any handler wants to
// print locations (locationEnabled), use vmodule, or print full stacks (BacktraceAt).
var stackEnabled atomic.Bool

// locationEnabled is an atomic flag controlling whether the terminal formatter
// should append the log locations too when printing entries.
var locationEnabled atomic.Bool

// locationLength is the maxmimum path length encountered, which all logs are
// padded to to aid in alignment.
var locationLength atomic.Uint32

// fieldPadding is a global map with maximum field value lengths seen until now
// to allow padding log contexts in a bit smarter way.
var fieldPadding = make(map[string]int)

// fieldPaddingLock is a global mutex protecting the field padding map.
var fieldPaddingLock sync.RWMutex

type Format interface {
	Format(r *Record) []byte
}

// FormatFunc returns a new Format object which uses
// the given function to perform record formatting.
func FormatFunc(f func(*Record) []byte) Format {
	return formatFunc(f)
}

type formatFunc func(*Record) []byte

func (f formatFunc) Format(r *Record) []byte {
	return f(r)
}

// TerminalStringer is an analogous interface to the stdlib stringer, allowing
// own types to have custom shortened serialization formats when printed to the
// screen.
type TerminalStringer interface {
	TerminalString() string
}

// TerminalFormat formats log records optimized for human readability on
// a terminal with color-coded level output and terser human friendly timestamp.
// This format should only be used for interactive programs or while developing.
//
//	[LEVEL] [TIME] MESSAGE key=value key=value ...
//
// Example:
//
//	[DBUG] [May 16 20:58:45] remove route ns=haproxy addr=127.0.0.1:50002
func TerminalFormat(usecolor bool) Format {
	return FormatFunc(func(r *Record) []byte {
		msg := escapeMessage(r.Msg)
		var color = 0
		if usecolor {
			switch r.Lvl {
			case LvlCrit:
				color = 35
			case LvlError:
				color = 31
			case LvlWarn:
				color = 33
			case LvlInfo:
				color = 32
			case LvlDebug:
				color = 36
			case LvlTrace:
				color = 34
			}
		}

		b := &bytes.Buffer{}
		lvl := r.Lvl.AlignedString()
		if locationEnabled.Load() {
			// Log origin printing was requested, format the location path and line number
			location := fmt.Sprintf("%+v", r.Call)
			for _, prefix := range locationTrims {
				location = strings.TrimPrefix(location, prefix)
			}
			// Maintain the maximum location length for fancyer alignment
			align := int(locationLength.Load())
			if align < len(location) {
				align = len(location)
				locationLength.Store(uint32(align))
			}
			padding := strings.Repeat(" ", align-len(location))

			// Assemble and print the log heading
			if color > 0 {
				fmt.Fprintf(b, "\x1b[%dm%s\x1b[0m[%s|%s]%s %s ", color, lvl, r.Time.Format(termTimeFormat), location, padding, msg)
			} else {
				fmt.Fprintf(b, "%s[%s|%s]%s %s ", lvl, r.Time.Format(termTimeFormat), location, padding, msg)
			}
		} else {
			if color > 0 {
				fmt.Fprintf(b, "\x1b[%dm%s\x1b[0m[%s] %s ", color, lvl, r.Time.Format(termTimeFormat), msg)
			} else {
				fmt.Fprintf(b, "%s[%s] %s ", lvl, r.Time.Format(termTimeFormat), msg)
			}
		}
		// try to justify the log output for short messages
		length := utf8.RuneCountInString(msg)
		if len(r.Ctx) > 0 && length < termMsgJust {
			b.Write(bytes.Repeat([]byte{' '}, termMsgJust-length))
		}
		// print the keys logfmt style
		logfmt(b, r.Ctx, color, true)
		return b.Bytes()
	})
}

// LogfmtFormat prints records in logfmt format, an easy machine-parseable but human-readable
// format for key/value pairs.
//
// For more details see: http://godoc.org/github.com/kr/logfmt
func LogfmtFormat() Format {
	return FormatFunc(func(r *Record) []byte {
		common := []interface{}{r.KeyNames.Time, r.Time, r.KeyNames.Lvl, r.Lvl, r.KeyNames.Msg, r.Msg}
		buf := &bytes.Buffer{}
		logfmt(buf, append(common, r.Ctx...), 0, false)
		return buf.Bytes()
	})
}

func logfmt(buf *bytes.Buffer, ctx []interface{}, color int, term bool) {
	for i := 0; i < len(ctx); i += 2 {
		if i != 0 {
			buf.WriteByte(' ')
		}

		k, ok := ctx[i].(string)
		v := formatLogfmtValue(ctx[i+1], term)
		if !ok {
			k, v = errorKey, fmt.Sprintf("%+T is not a string key", ctx[i])
		} else {
			k = escapeString(k)
		}

		// XXX: we should probably check that all of your key bytes aren't invalid
		fieldPaddingLock.RLock()
		padding := fieldPadding[k]
		fieldPaddingLock.RUnlock()

		length := utf8.RuneCountInString(v)
		if padding < length && length <= termCtxMaxPadding {
			padding = length

			fieldPaddingLock.Lock()
			fieldPadding[k] = padding
			fieldPaddingLock.Unlock()
		}
		if color > 0 {
			fmt.Fprintf(buf, "\x1b[%dm%s\x1b[0m=", color, k)
		} else {
			buf.WriteString(k)
			buf.WriteByte('=')
		}
		buf.WriteString(v)
		if i < len(ctx)-2 && padding > length {
			buf.Write(bytes.Repeat([]byte{' '}, padding-length))
		}
	}
	buf.WriteByte('\n')
}

// JSONFormat formats log records as JSON objects separated by newlines.
// It is the equivalent of JSONFormatEx(false, true).
func JSONFormat() Format {
	return JSONFormatEx(false, true)
}

// JSONFormatOrderedEx formats log records as JSON arrays. If pretty is true,
// records will be pretty-printed. If lineSeparated is true, records
// will be logged with a new line between each record.
func JSONFormatOrderedEx(pretty, lineSeparated bool) Format {
	jsonMarshal := json.Marshal
	if pretty {
		jsonMarshal = func(v interface{}) ([]byte, error) {
			return json.MarshalIndent(v, "", "    ")
		}
	}
	return FormatFunc(func(r *Record) []byte {
		props := map[string]interface{}{
			r.KeyNames.Time: r.Time,
			r.KeyNames.Lvl:  r.Lvl.String(),
			r.KeyNames.Msg:  r.Msg,
		}

		ctx := make([]string, len(r.Ctx))
		for i := 0; i < len(r.Ctx); i += 2 {
			if k, ok := r.Ctx[i].(string); ok {
				ctx[i] = k
				ctx[i+1] = formatLogfmtValue(r.Ctx[i+1], true)
			} else {
				props[errorKey] = fmt.Sprintf("%+T is not a string key,", r.Ctx[i])
			}
		}
		props[r.KeyNames.Ctx] = ctx

		b, err := jsonMarshal(props)
		if err != nil {
			b, _ = jsonMarshal(map[string]string{
				errorKey: err.Error(),
			})
			return b
		}
		if lineSeparated {
			b = append(b, '\n')
		}
		return b
	})
}

// JSONFormatEx formats log records as JSON objects. If pretty is true,
// records will be pretty-printed. If lineSeparated is true, records
// will be logged with a new line between each record.
func JSONFormatEx(pretty, lineSeparated bool) Format {
	jsonMarshal := json.Marshal
	if pretty {
		jsonMarshal = func(v interface{}) ([]byte, error) {
			return json.MarshalIndent(v, "", "    ")
		}
	}

	return FormatFunc(func(r *Record) []byte {
		props := map[string]interface{}{
			r.KeyNames.Time: r.Time,
			r.KeyNames.Lvl:  r.Lvl.String(),
			r.KeyNames.Msg:  r.Msg,
		}

		for i := 0; i < len(r.Ctx); i += 2 {
			k, ok := r.Ctx[i].(string)
			if !ok {
				props[errorKey] = fmt.Sprintf("%+T is not a string key", r.Ctx[i])
			} else {
				props[k] = formatJSONValue(r.Ctx[i+1])
			}
		}

		b, err := jsonMarshal(props)
		if err != nil {
			b, _ = jsonMarshal(map[string]string{
				errorKey: err.Error(),
			})
			return b
		}

		if lineSeparated {
			b = append(b, '\n')
		}

		return b
	})
}

func formatShared(value interface{}) (result interface{}) {
	defer func() {
		if err := recover(); err != nil {
			if v := reflect.ValueOf(value); v.Kind() == reflect.Ptr && v.IsNil() {
				result = "nil"
			} else {
				panic(err)
			}
		}
	}()

	switch v := value.(type) {
	case time.Time:
		return v.Format(timeFormat)

	case error:
		return v.Error()

	case fmt.Stringer:
		return v.String()

	default:
		return v
	}
}

func formatJSONValue(value interface{}) interface{} {
	value = formatShared(value)
	switch value.(type) {
	case int, int8, int16, int32, int64, float32, float64, uint, uint8, uint16, uint32, uint64, string:
		return value
	default:
		return fmt.Sprintf("%+v", value)
	}
}

// formatValue formats a value for serialization
func formatLogfmtValue(value interface{}, term bool) string {
	if value == nil {
		return "nil"
	}

	switch v := value.(type) {
	case time.Time:
		// Performance optimization: No need for escaping since the provided
		// timeFormat doesn't have any escape characters, and escaping is
		// expensive.
		return v.Format(timeFormat)

	case *big.Int:
		// Big ints get consumed by the Stringer clause, so we need to handle
		// them earlier on.
		if v == nil {
			return "<nil>"
		}
		return formatLogfmtBigInt(v)

	case *uint256.Int:
		// Uint256s get consumed by the Stringer clause, so we need to handle
		// them earlier on.
		if v == nil {
			return "<nil>"
		}
		return formatLogfmtUint256(v)
	}
	if term {
		if s, ok := value.(TerminalStringer); ok {
			// Custom terminal stringer provided, use that
			return escapeString(s.TerminalString())
		}
	}
	value = formatShared(value)
	switch v := value.(type) {
	case bool:
		return strconv.FormatBool(v)
	case float32:
		return strconv.FormatFloat(float64(v), floatFormat, 3, 64)
	case float64:
		return strconv.FormatFloat(v, floatFormat, 3, 64)
	case int8:
		return strconv.FormatInt(int64(v), 10)
	case uint8:
		return strconv.FormatInt(int64(v), 10)
	case int16:
		return strconv.FormatInt(int64(v), 10)
	case uint16:
		return strconv.FormatInt(int64(v), 10)
	// Larger integers get thousands separators.
	case int:
		return FormatLogfmtInt64(int64(v))
	case int32:
		return FormatLogfmtInt64(int64(v))
	case int64:
		return FormatLogfmtInt64(v)
	case uint:
		return FormatLogfmtUint64(uint64(v))
	case uint32:
		return FormatLogfmtUint64(uint64(v))
	case uint64:
		return FormatLogfmtUint64(v)
	case string:
		return escapeString(v)
	default:
		return escapeString(fmt.Sprintf("%+v", value))
	}
}

// FormatLogfmtInt64 formats n with thousand separators.
func FormatLogfmtInt64(n int64) string {
	if n < 0 {
		return formatLogfmtUint64(uint64(-n), true)
	}
	return formatLogfmtUint64(uint64(n), false)
}

// FormatLogfmtUint64 formats n with thousand separators.
func FormatLogfmtUint64(n uint64) string {
	return formatLogfmtUint64(n, false)
}

func formatLogfmtUint64(n uint64, neg bool) string {
	// Small numbers are fine as is
	if n < 100000 {
		if neg {
			return strconv.Itoa(-int(n))
		} else {
			return strconv.Itoa(int(n))
		}
	}
	// Large numbers should be split
	const maxLength = 26

	var (
		out   = make([]byte, maxLength)
		i     = maxLength - 1
		comma = 0
	)
	for ; n > 0; i-- {
		if comma == 3 {
			comma = 0
			out[i] = ','
		} else {
			comma++
			out[i] = '0' + byte(n%10)
			n /= 10
		}
	}
	if neg {
		out[i] = '-'
		i--
	}
	return string(out[i+1:])
}

// formatLogfmtBigInt formats n with thousand separators.
func formatLogfmtBigInt(n *big.Int) string {
	if n.IsUint64() {
		return FormatLogfmtUint64(n.Uint64())
	}
	if n.IsInt64() {
		return FormatLogfmtInt64(n.Int64())
	}

	var (
		text  = n.String()
		buf   = make([]byte, len(text)+len(text)/3)
		comma = 0
		i     = len(buf) - 1
	)
	for j := len(text) - 1; j >= 0; j, i = j-1, i-1 {
		c := text[j]

		switch {
		case c == '-':
			buf[i] = c
		case comma == 3:
			buf[i] = ','
			i--
			comma = 0
			fallthrough
		default:
			buf[i] = c
			comma++
		}
	}
	return string(buf[i+1:])
}

// formatLogfmtUint256 formats n with thousand separators.
func formatLogfmtUint256(n *uint256.Int) string {
	if n.IsUint64() {
		return FormatLogfmtUint64(n.Uint64())
	}
	var (
		text  = n.Dec()
		buf   = make([]byte, len(text)+len(text)/3)
		comma = 0
		i     = len(buf) - 1
	)
	for j := len(text) - 1; j >= 0; j, i = j-1, i-1 {
		c := text[j]

		switch {
		case c == '-':
			buf[i] = c
		case comma == 3:
			buf[i] = ','
			i--
			comma = 0
			fallthrough
		default:
			buf[i] = c
			comma++
		}
	}
	return string(buf[i+1:])
}

// escapeString checks if the provided string needs escaping/quoting, and
// calls strconv.Quote if needed
func escapeString(s string) string {
	needsQuoting := false
	for _, r := range s {
		// We quote everything below " (0x22) and above~ (0x7E), plus equal-sign
		if r <= '"' || r > '~' || r == '=' {
			needsQuoting = true
			break
		}
	}
	if !needsQuoting {
		return s
	}
	return strconv.Quote(s)
}

// escapeMessage checks if the provided string needs escaping/quoting, similarly
// to escapeString. The difference is that this method is more lenient: it allows
// for spaces and linebreaks to occur without needing quoting.
func escapeMessage(s string) string {
	needsQuoting := false
	for _, r := range s {
		// Allow CR/LF/TAB. This is to make multi-line messages work.
		if r == '\r' || r == '\n' || r == '\t' {
			continue
		}
		// We quote everything below <space> (0x20) and above~ (0x7E),
		// plus equal-sign
		if r < ' ' || r > '~' || r == '=' {
			needsQuoting = true
			break
		}
	}
	if !needsQuoting {
		return s
	}
	return strconv.Quote(s)
}

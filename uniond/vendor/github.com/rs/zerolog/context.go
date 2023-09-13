package zerolog

import (
	"context"
	"fmt"
	"io/ioutil"
	"math"
	"net"
	"time"
)

// Context configures a new sub-logger with contextual fields.
type Context struct {
	l Logger
}

// Logger returns the logger with the context previously set.
func (c Context) Logger() Logger {
	return c.l
}

// Fields is a helper function to use a map or slice to set fields using type assertion.
// Only map[string]interface{} and []interface{} are accepted. []interface{} must
// alternate string keys and arbitrary values, and extraneous ones are ignored.
func (c Context) Fields(fields interface{}) Context {
	c.l.context = appendFields(c.l.context, fields)
	return c
}

// Dict adds the field key with the dict to the logger context.
func (c Context) Dict(key string, dict *Event) Context {
	dict.buf = enc.AppendEndMarker(dict.buf)
	c.l.context = append(enc.AppendKey(c.l.context, key), dict.buf...)
	putEvent(dict)
	return c
}

// Array adds the field key with an array to the event context.
// Use zerolog.Arr() to create the array or pass a type that
// implement the LogArrayMarshaler interface.
func (c Context) Array(key string, arr LogArrayMarshaler) Context {
	c.l.context = enc.AppendKey(c.l.context, key)
	if arr, ok := arr.(*Array); ok {
		c.l.context = arr.write(c.l.context)
		return c
	}
	var a *Array
	if aa, ok := arr.(*Array); ok {
		a = aa
	} else {
		a = Arr()
		arr.MarshalZerologArray(a)
	}
	c.l.context = a.write(c.l.context)
	return c
}

// Object marshals an object that implement the LogObjectMarshaler interface.
func (c Context) Object(key string, obj LogObjectMarshaler) Context {
	e := newEvent(levelWriterAdapter{ioutil.Discard}, 0)
	e.Object(key, obj)
	c.l.context = enc.AppendObjectData(c.l.context, e.buf)
	putEvent(e)
	return c
}

// EmbedObject marshals and Embeds an object that implement the LogObjectMarshaler interface.
func (c Context) EmbedObject(obj LogObjectMarshaler) Context {
	e := newEvent(levelWriterAdapter{ioutil.Discard}, 0)
	e.EmbedObject(obj)
	c.l.context = enc.AppendObjectData(c.l.context, e.buf)
	putEvent(e)
	return c
}

// Str adds the field key with val as a string to the logger context.
func (c Context) Str(key, val string) Context {
	c.l.context = enc.AppendString(enc.AppendKey(c.l.context, key), val)
	return c
}

// Strs adds the field key with val as a string to the logger context.
func (c Context) Strs(key string, vals []string) Context {
	c.l.context = enc.AppendStrings(enc.AppendKey(c.l.context, key), vals)
	return c
}

// Stringer adds the field key with val.String() (or null if val is nil) to the logger context.
func (c Context) Stringer(key string, val fmt.Stringer) Context {
	if val != nil {
		c.l.context = enc.AppendString(enc.AppendKey(c.l.context, key), val.String())
		return c
	}

	c.l.context = enc.AppendInterface(enc.AppendKey(c.l.context, key), nil)
	return c
}

// Bytes adds the field key with val as a []byte to the logger context.
func (c Context) Bytes(key string, val []byte) Context {
	c.l.context = enc.AppendBytes(enc.AppendKey(c.l.context, key), val)
	return c
}

// Hex adds the field key with val as a hex string to the logger context.
func (c Context) Hex(key string, val []byte) Context {
	c.l.context = enc.AppendHex(enc.AppendKey(c.l.context, key), val)
	return c
}

// RawJSON adds already encoded JSON to context.
//
// No sanity check is performed on b; it must not contain carriage returns and
// be valid JSON.
func (c Context) RawJSON(key string, b []byte) Context {
	c.l.context = appendJSON(enc.AppendKey(c.l.context, key), b)
	return c
}

// AnErr adds the field key with serialized err to the logger context.
func (c Context) AnErr(key string, err error) Context {
	switch m := ErrorMarshalFunc(err).(type) {
	case nil:
		return c
	case LogObjectMarshaler:
		return c.Object(key, m)
	case error:
		if m == nil || isNilValue(m) {
			return c
		} else {
			return c.Str(key, m.Error())
		}
	case string:
		return c.Str(key, m)
	default:
		return c.Interface(key, m)
	}
}

// Errs adds the field key with errs as an array of serialized errors to the
// logger context.
func (c Context) Errs(key string, errs []error) Context {
	arr := Arr()
	for _, err := range errs {
		switch m := ErrorMarshalFunc(err).(type) {
		case LogObjectMarshaler:
			arr = arr.Object(m)
		case error:
			if m == nil || isNilValue(m) {
				arr = arr.Interface(nil)
			} else {
				arr = arr.Str(m.Error())
			}
		case string:
			arr = arr.Str(m)
		default:
			arr = arr.Interface(m)
		}
	}

	return c.Array(key, arr)
}

// Err adds the field "error" with serialized err to the logger context.
func (c Context) Err(err error) Context {
	return c.AnErr(ErrorFieldName, err)
}

// Ctx adds the context.Context to the logger context. The context.Context is
// not rendered in the error message, but is made available for hooks to use.
// A typical use case is to extract tracing information from the
// context.Context.
func (c Context) Ctx(ctx context.Context) Context {
	c.l.ctx = ctx
	return c
}

// Bool adds the field key with val as a bool to the logger context.
func (c Context) Bool(key string, b bool) Context {
	c.l.context = enc.AppendBool(enc.AppendKey(c.l.context, key), b)
	return c
}

// Bools adds the field key with val as a []bool to the logger context.
func (c Context) Bools(key string, b []bool) Context {
	c.l.context = enc.AppendBools(enc.AppendKey(c.l.context, key), b)
	return c
}

// Int adds the field key with i as a int to the logger context.
func (c Context) Int(key string, i int) Context {
	c.l.context = enc.AppendInt(enc.AppendKey(c.l.context, key), i)
	return c
}

// Ints adds the field key with i as a []int to the logger context.
func (c Context) Ints(key string, i []int) Context {
	c.l.context = enc.AppendInts(enc.AppendKey(c.l.context, key), i)
	return c
}

// Int8 adds the field key with i as a int8 to the logger context.
func (c Context) Int8(key string, i int8) Context {
	c.l.context = enc.AppendInt8(enc.AppendKey(c.l.context, key), i)
	return c
}

// Ints8 adds the field key with i as a []int8 to the logger context.
func (c Context) Ints8(key string, i []int8) Context {
	c.l.context = enc.AppendInts8(enc.AppendKey(c.l.context, key), i)
	return c
}

// Int16 adds the field key with i as a int16 to the logger context.
func (c Context) Int16(key string, i int16) Context {
	c.l.context = enc.AppendInt16(enc.AppendKey(c.l.context, key), i)
	return c
}

// Ints16 adds the field key with i as a []int16 to the logger context.
func (c Context) Ints16(key string, i []int16) Context {
	c.l.context = enc.AppendInts16(enc.AppendKey(c.l.context, key), i)
	return c
}

// Int32 adds the field key with i as a int32 to the logger context.
func (c Context) Int32(key string, i int32) Context {
	c.l.context = enc.AppendInt32(enc.AppendKey(c.l.context, key), i)
	return c
}

// Ints32 adds the field key with i as a []int32 to the logger context.
func (c Context) Ints32(key string, i []int32) Context {
	c.l.context = enc.AppendInts32(enc.AppendKey(c.l.context, key), i)
	return c
}

// Int64 adds the field key with i as a int64 to the logger context.
func (c Context) Int64(key string, i int64) Context {
	c.l.context = enc.AppendInt64(enc.AppendKey(c.l.context, key), i)
	return c
}

// Ints64 adds the field key with i as a []int64 to the logger context.
func (c Context) Ints64(key string, i []int64) Context {
	c.l.context = enc.AppendInts64(enc.AppendKey(c.l.context, key), i)
	return c
}

// Uint adds the field key with i as a uint to the logger context.
func (c Context) Uint(key string, i uint) Context {
	c.l.context = enc.AppendUint(enc.AppendKey(c.l.context, key), i)
	return c
}

// Uints adds the field key with i as a []uint to the logger context.
func (c Context) Uints(key string, i []uint) Context {
	c.l.context = enc.AppendUints(enc.AppendKey(c.l.context, key), i)
	return c
}

// Uint8 adds the field key with i as a uint8 to the logger context.
func (c Context) Uint8(key string, i uint8) Context {
	c.l.context = enc.AppendUint8(enc.AppendKey(c.l.context, key), i)
	return c
}

// Uints8 adds the field key with i as a []uint8 to the logger context.
func (c Context) Uints8(key string, i []uint8) Context {
	c.l.context = enc.AppendUints8(enc.AppendKey(c.l.context, key), i)
	return c
}

// Uint16 adds the field key with i as a uint16 to the logger context.
func (c Context) Uint16(key string, i uint16) Context {
	c.l.context = enc.AppendUint16(enc.AppendKey(c.l.context, key), i)
	return c
}

// Uints16 adds the field key with i as a []uint16 to the logger context.
func (c Context) Uints16(key string, i []uint16) Context {
	c.l.context = enc.AppendUints16(enc.AppendKey(c.l.context, key), i)
	return c
}

// Uint32 adds the field key with i as a uint32 to the logger context.
func (c Context) Uint32(key string, i uint32) Context {
	c.l.context = enc.AppendUint32(enc.AppendKey(c.l.context, key), i)
	return c
}

// Uints32 adds the field key with i as a []uint32 to the logger context.
func (c Context) Uints32(key string, i []uint32) Context {
	c.l.context = enc.AppendUints32(enc.AppendKey(c.l.context, key), i)
	return c
}

// Uint64 adds the field key with i as a uint64 to the logger context.
func (c Context) Uint64(key string, i uint64) Context {
	c.l.context = enc.AppendUint64(enc.AppendKey(c.l.context, key), i)
	return c
}

// Uints64 adds the field key with i as a []uint64 to the logger context.
func (c Context) Uints64(key string, i []uint64) Context {
	c.l.context = enc.AppendUints64(enc.AppendKey(c.l.context, key), i)
	return c
}

// Float32 adds the field key with f as a float32 to the logger context.
func (c Context) Float32(key string, f float32) Context {
	c.l.context = enc.AppendFloat32(enc.AppendKey(c.l.context, key), f)
	return c
}

// Floats32 adds the field key with f as a []float32 to the logger context.
func (c Context) Floats32(key string, f []float32) Context {
	c.l.context = enc.AppendFloats32(enc.AppendKey(c.l.context, key), f)
	return c
}

// Float64 adds the field key with f as a float64 to the logger context.
func (c Context) Float64(key string, f float64) Context {
	c.l.context = enc.AppendFloat64(enc.AppendKey(c.l.context, key), f)
	return c
}

// Floats64 adds the field key with f as a []float64 to the logger context.
func (c Context) Floats64(key string, f []float64) Context {
	c.l.context = enc.AppendFloats64(enc.AppendKey(c.l.context, key), f)
	return c
}

type timestampHook struct{}

func (ts timestampHook) Run(e *Event, level Level, msg string) {
	e.Timestamp()
}

var th = timestampHook{}

// Timestamp adds the current local time to the logger context with the "time" key, formatted using zerolog.TimeFieldFormat.
// To customize the key name, change zerolog.TimestampFieldName.
// To customize the time format, change zerolog.TimeFieldFormat.
//
// NOTE: It won't dedupe the "time" key if the *Context has one already.
func (c Context) Timestamp() Context {
	c.l = c.l.Hook(th)
	return c
}

// Time adds the field key with t formated as string using zerolog.TimeFieldFormat.
func (c Context) Time(key string, t time.Time) Context {
	c.l.context = enc.AppendTime(enc.AppendKey(c.l.context, key), t, TimeFieldFormat)
	return c
}

// Times adds the field key with t formated as string using zerolog.TimeFieldFormat.
func (c Context) Times(key string, t []time.Time) Context {
	c.l.context = enc.AppendTimes(enc.AppendKey(c.l.context, key), t, TimeFieldFormat)
	return c
}

// Dur adds the fields key with d divided by unit and stored as a float.
func (c Context) Dur(key string, d time.Duration) Context {
	c.l.context = enc.AppendDuration(enc.AppendKey(c.l.context, key), d, DurationFieldUnit, DurationFieldInteger)
	return c
}

// Durs adds the fields key with d divided by unit and stored as a float.
func (c Context) Durs(key string, d []time.Duration) Context {
	c.l.context = enc.AppendDurations(enc.AppendKey(c.l.context, key), d, DurationFieldUnit, DurationFieldInteger)
	return c
}

// Interface adds the field key with obj marshaled using reflection.
func (c Context) Interface(key string, i interface{}) Context {
	c.l.context = enc.AppendInterface(enc.AppendKey(c.l.context, key), i)
	return c
}

type callerHook struct {
	callerSkipFrameCount int
}

func newCallerHook(skipFrameCount int) callerHook {
	return callerHook{callerSkipFrameCount: skipFrameCount}
}

func (ch callerHook) Run(e *Event, level Level, msg string) {
	switch ch.callerSkipFrameCount {
	case useGlobalSkipFrameCount:
		// Extra frames to skip (added by hook infra).
		e.caller(CallerSkipFrameCount + contextCallerSkipFrameCount)
	default:
		// Extra frames to skip (added by hook infra).
		e.caller(ch.callerSkipFrameCount + contextCallerSkipFrameCount)
	}
}

// useGlobalSkipFrameCount acts as a flag to informat callerHook.Run
// to use the global CallerSkipFrameCount.
const useGlobalSkipFrameCount = math.MinInt32

// ch is the default caller hook using the global CallerSkipFrameCount.
var ch = newCallerHook(useGlobalSkipFrameCount)

// Caller adds the file:line of the caller with the zerolog.CallerFieldName key.
func (c Context) Caller() Context {
	c.l = c.l.Hook(ch)
	return c
}

// CallerWithSkipFrameCount adds the file:line of the caller with the zerolog.CallerFieldName key.
// The specified skipFrameCount int will override the global CallerSkipFrameCount for this context's respective logger.
// If set to -1 the global CallerSkipFrameCount will be used.
func (c Context) CallerWithSkipFrameCount(skipFrameCount int) Context {
	c.l = c.l.Hook(newCallerHook(skipFrameCount))
	return c
}

// Stack enables stack trace printing for the error passed to Err().
func (c Context) Stack() Context {
	c.l.stack = true
	return c
}

// IPAddr adds IPv4 or IPv6 Address to the context
func (c Context) IPAddr(key string, ip net.IP) Context {
	c.l.context = enc.AppendIPAddr(enc.AppendKey(c.l.context, key), ip)
	return c
}

// IPPrefix adds IPv4 or IPv6 Prefix (address and mask) to the context
func (c Context) IPPrefix(key string, pfx net.IPNet) Context {
	c.l.context = enc.AppendIPPrefix(enc.AppendKey(c.l.context, key), pfx)
	return c
}

// MACAddr adds MAC address to the context
func (c Context) MACAddr(key string, ha net.HardwareAddr) Context {
	c.l.context = enc.AppendMACAddr(enc.AppendKey(c.l.context, key), ha)
	return c
}

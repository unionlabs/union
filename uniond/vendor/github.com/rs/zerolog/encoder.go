package zerolog

import (
	"net"
	"time"
)

type encoder interface {
	AppendArrayDelim(dst []byte) []byte
	AppendArrayEnd(dst []byte) []byte
	AppendArrayStart(dst []byte) []byte
	AppendBeginMarker(dst []byte) []byte
	AppendBool(dst []byte, val bool) []byte
	AppendBools(dst []byte, vals []bool) []byte
	AppendBytes(dst, s []byte) []byte
	AppendDuration(dst []byte, d time.Duration, unit time.Duration, useInt bool, precision int) []byte
	AppendDurations(dst []byte, vals []time.Duration, unit time.Duration, useInt bool, precision int) []byte
	AppendEndMarker(dst []byte) []byte
	AppendFloat32(dst []byte, val float32, precision int) []byte
	AppendFloat64(dst []byte, val float64, precision int) []byte
	AppendFloats32(dst []byte, vals []float32, precision int) []byte
	AppendFloats64(dst []byte, vals []float64, precision int) []byte
	AppendHex(dst, s []byte) []byte
	AppendIPAddr(dst []byte, ip net.IP) []byte
	AppendIPPrefix(dst []byte, pfx net.IPNet) []byte
	AppendInt(dst []byte, val int) []byte
	AppendInt16(dst []byte, val int16) []byte
	AppendInt32(dst []byte, val int32) []byte
	AppendInt64(dst []byte, val int64) []byte
	AppendInt8(dst []byte, val int8) []byte
	AppendInterface(dst []byte, i interface{}) []byte
	AppendInts(dst []byte, vals []int) []byte
	AppendInts16(dst []byte, vals []int16) []byte
	AppendInts32(dst []byte, vals []int32) []byte
	AppendInts64(dst []byte, vals []int64) []byte
	AppendInts8(dst []byte, vals []int8) []byte
	AppendKey(dst []byte, key string) []byte
	AppendLineBreak(dst []byte) []byte
	AppendMACAddr(dst []byte, ha net.HardwareAddr) []byte
	AppendNil(dst []byte) []byte
	AppendObjectData(dst []byte, o []byte) []byte
	AppendString(dst []byte, s string) []byte
	AppendStrings(dst []byte, vals []string) []byte
	AppendTime(dst []byte, t time.Time, format string) []byte
	AppendTimes(dst []byte, vals []time.Time, format string) []byte
	AppendUint(dst []byte, val uint) []byte
	AppendUint16(dst []byte, val uint16) []byte
	AppendUint32(dst []byte, val uint32) []byte
	AppendUint64(dst []byte, val uint64) []byte
	AppendUint8(dst []byte, val uint8) []byte
	AppendUints(dst []byte, vals []uint) []byte
	AppendUints16(dst []byte, vals []uint16) []byte
	AppendUints32(dst []byte, vals []uint32) []byte
	AppendUints64(dst []byte, vals []uint64) []byte
	AppendUints8(dst []byte, vals []uint8) []byte
}

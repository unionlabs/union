//go:build windows

package etw

import (
	"bytes"
	"encoding/binary"
)

// inType indicates the type of data contained in the ETW event.
type inType byte

// Various inType definitions for TraceLogging. These must match the definitions
// found in TraceLoggingProvider.h in the Windows SDK.
//
//nolint:deadcode,varcheck // keep unused constants for potential future use
const (
	inTypeNull inType = iota
	inTypeUnicodeString
	inTypeANSIString
	inTypeInt8
	inTypeUint8
	inTypeInt16
	inTypeUint16
	inTypeInt32
	inTypeUint32
	inTypeInt64
	inTypeUint64
	inTypeFloat
	inTypeDouble
	inTypeBool32
	inTypeBinary
	inTypeGUID
	inTypePointerUnsupported
	inTypeFileTime
	inTypeSystemTime
	inTypeSID
	inTypeHexInt32
	inTypeHexInt64
	inTypeCountedString
	inTypeCountedANSIString
	inTypeStruct
	inTypeCountedBinary
	inTypeCountedArray inType = 32
	inTypeArray        inType = 64
)

// outType specifies a hint to the event decoder for how the value should be
// formatted.
type outType byte

// Various outType definitions for TraceLogging. These must match the
// definitions found in TraceLoggingProvider.h in the Windows SDK.
//
//nolint:deadcode,varcheck // keep unused constants for potential future use
const (
	// outTypeDefault indicates that the default formatting for the inType will
	// be used by the event decoder.
	outTypeDefault outType = iota
	outTypeNoPrint
	outTypeString
	outTypeBoolean
	outTypeHex
	outTypePID
	outTypeTID
	outTypePort
	outTypeIPv4
	outTypeIPv6
	outTypeSocketAddress
	outTypeXML
	outTypeJSON
	outTypeWin32Error
	outTypeNTStatus
	outTypeHResult
	outTypeFileTime
	outTypeSigned
	outTypeUnsigned
	outTypeUTF8              outType = 35
	outTypePKCS7WithTypeInfo outType = 36
	outTypeCodePointer       outType = 37
	outTypeDateTimeUTC       outType = 38
)

// eventMetadata maintains a buffer which builds up the metadata for an ETW
// event. It needs to be paired with EventData which describes the event.
type eventMetadata struct {
	buffer bytes.Buffer
}

// toBytes returns the raw binary data containing the event metadata. Before being
// returned, the current size of the buffer is written to the start of the
// buffer. The returned value is not copied from the internal buffer, so it can
// be mutated by the eventMetadata object after it is returned.
func (em *eventMetadata) toBytes() []byte {
	// Finalize the event metadata buffer by filling in the buffer length at the
	// beginning.
	binary.LittleEndian.PutUint16(em.buffer.Bytes(), uint16(em.buffer.Len()))
	return em.buffer.Bytes()
}

// writeEventHeader writes the metadata for the start of an event to the buffer.
// This specifies the event name and tags.
func (em *eventMetadata) writeEventHeader(name string, tags uint32) {
	_ = binary.Write(&em.buffer, binary.LittleEndian, uint16(0)) // Length placeholder
	em.writeTags(tags)
	em.buffer.WriteString(name)
	em.buffer.WriteByte(0) // Null terminator for name
}

func (em *eventMetadata) writeFieldInner(name string, inType inType, outType outType, tags uint32, arrSize uint16) {
	em.buffer.WriteString(name)
	em.buffer.WriteByte(0) // Null terminator for name

	if outType == outTypeDefault && tags == 0 {
		em.buffer.WriteByte(byte(inType))
	} else {
		em.buffer.WriteByte(byte(inType | 128))
		if tags == 0 {
			em.buffer.WriteByte(byte(outType))
		} else {
			em.buffer.WriteByte(byte(outType | 128))
			em.writeTags(tags)
		}
	}

	if arrSize != 0 {
		_ = binary.Write(&em.buffer, binary.LittleEndian, arrSize)
	}
}

// writeTags writes out the tags value to the event metadata. Tags is a 28-bit
// value, interpreted as bit flags, which are only relevant to the event
// consumer. The event consumer may choose to attribute special meaning to tags
// (e.g. 0x4 could mean the field contains PII). Tags are written as a series of
// bytes, each containing 7 bits of tag value, with the high bit set if there is
// more tag data in the following byte. This allows for a more compact
// representation when not all of the tag bits are needed.
func (em *eventMetadata) writeTags(tags uint32) {
	// Only use the top 28 bits of the tags value.
	tags &= 0xfffffff

	for {
		// Tags are written with the most significant bits (e.g. 21-27) first.
		val := tags >> 21

		if tags&0x1fffff == 0 {
			// If there is no more data to write after this, write this value
			// without the high bit set, and return.
			em.buffer.WriteByte(byte(val & 0x7f))
			return
		}

		em.buffer.WriteByte(byte(val | 0x80))

		tags <<= 7
	}
}

// writeField writes the metadata for a simple field to the buffer.
//
//nolint:unparam // tags is currently always 0, may change in the future
func (em *eventMetadata) writeField(name string, inType inType, outType outType, tags uint32) {
	em.writeFieldInner(name, inType, outType, tags, 0)
}

// writeArray writes the metadata for an array field to the buffer. The number
// of elements in the array must be written as a uint16 in the event data,
// immediately preceding the event data.
//
//nolint:unparam // tags is currently always 0, may change in the future
func (em *eventMetadata) writeArray(name string, inType inType, outType outType, tags uint32) {
	em.writeFieldInner(name, inType|inTypeArray, outType, tags, 0)
}

// writeCountedArray writes the metadata for an array field to the buffer. The
// size of a counted array is fixed, and the size is written into the metadata
// directly.
//
//nolint:unused // keep for future use
func (em *eventMetadata) writeCountedArray(name string, count uint16, inType inType, outType outType, tags uint32) {
	em.writeFieldInner(name, inType|inTypeCountedArray, outType, tags, count)
}

// writeStruct writes the metadata for a nested struct to the buffer. The struct
// contains the next N fields in the metadata, where N is specified by the
// fieldCount argument.
func (em *eventMetadata) writeStruct(name string, fieldCount uint8, tags uint32) {
	em.writeFieldInner(name, inTypeStruct, outType(fieldCount), tags, 0)
}

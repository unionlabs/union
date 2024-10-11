//go:build windows

package etw

// Channel represents the ETW logging channel that is used. It can be used by
// event consumers to give an event special treatment.
type Channel uint8

const (
	// ChannelTraceLogging is the default channel for TraceLogging events. It is
	// not required to be used for TraceLogging, but will prevent decoding
	// issues for these events on older operating systems.
	ChannelTraceLogging Channel = 11
)

// Level represents the ETW logging level. There are several predefined levels
// that are commonly used, but technically anything from 0-255 is allowed.
// Lower levels indicate more important events, and 0 indicates an event that
// will always be collected.
type Level uint8

// Predefined ETW log levels from winmeta.xml in the Windows SDK.
const (
	LevelAlways Level = iota
	LevelCritical
	LevelError
	LevelWarning
	LevelInfo
	LevelVerbose
)

// Opcode represents the operation that the event indicates is being performed.
type Opcode uint8

// Predefined ETW opcodes from winmeta.xml in the Windows SDK.
const (
	// OpcodeInfo indicates an informational event.
	OpcodeInfo Opcode = iota
	// OpcodeStart indicates the start of an operation.
	OpcodeStart
	// OpcodeStop indicates the end of an operation.
	OpcodeStop
	// OpcodeDCStart indicates the start of a provider capture state operation.
	OpcodeDCStart
	// OpcodeDCStop indicates the end of a provider capture state operation.
	OpcodeDCStop
)

// EventDescriptor represents various metadata for an ETW event.
type eventDescriptor struct {
	id      uint16
	version uint8
	channel Channel
	level   Level
	opcode  Opcode
	task    uint16
	keyword uint64
}

// NewEventDescriptor returns an EventDescriptor initialized for use with
// TraceLogging.
func newEventDescriptor() *eventDescriptor {
	// Standard TraceLogging events default to the TraceLogging channel, and
	// verbose level.
	return &eventDescriptor{
		channel: ChannelTraceLogging,
		level:   LevelVerbose,
	}
}

// Identity returns the identity of the event. If the identity is not 0, it
// should uniquely identify the other event metadata (contained in
// EventDescriptor, and field metadata). Only the lower 24 bits of this value
// are relevant.
//
//nolint:unused // keep for future use
func (ed *eventDescriptor) identity() uint32 {
	return (uint32(ed.version) << 16) | uint32(ed.id)
}

// SetIdentity sets the identity of the event. If the identity is not 0, it
// should uniquely identify the other event metadata (contained in
// EventDescriptor, and field metadata). Only the lower 24 bits of this value
// are relevant.
//
//nolint:unused // keep for future use
func (ed *eventDescriptor) setIdentity(identity uint32) {
	ed.id = uint16(identity)
	ed.version = uint8(identity >> 16)
}

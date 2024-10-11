//go:build windows
// +build windows

package etw

import (
	"github.com/Microsoft/go-winio/pkg/guid"
)

type eventOptions struct {
	descriptor        *eventDescriptor
	activityID        guid.GUID
	relatedActivityID guid.GUID
	tags              uint32
}

// EventOpt defines the option function type that can be passed to
// Provider.WriteEvent to specify general event options, such as level and
// keyword.
type EventOpt func(options *eventOptions)

// WithEventOpts returns the variadic arguments as a single slice.
func WithEventOpts(opts ...EventOpt) []EventOpt {
	return opts
}

// WithLevel specifies the level of the event to be written.
func WithLevel(level Level) EventOpt {
	return func(options *eventOptions) {
		options.descriptor.level = level
	}
}

// WithKeyword specifies the keywords of the event to be written. Multiple uses
// of this option are OR'd together.
func WithKeyword(keyword uint64) EventOpt {
	return func(options *eventOptions) {
		options.descriptor.keyword |= keyword
	}
}

// WithChannel specifies the channel of the event to be written.
func WithChannel(channel Channel) EventOpt {
	return func(options *eventOptions) {
		options.descriptor.channel = channel
	}
}

// WithOpcode specifies the opcode of the event to be written.
func WithOpcode(opcode Opcode) EventOpt {
	return func(options *eventOptions) {
		options.descriptor.opcode = opcode
	}
}

// WithTags specifies the tags of the event to be written. Tags is a 28-bit
// value (top 4 bits are ignored) which are interpreted by the event consumer.
func WithTags(newTags uint32) EventOpt {
	return func(options *eventOptions) {
		options.tags |= newTags
	}
}

// WithActivityID specifies the activity ID of the event to be written.
func WithActivityID(activityID guid.GUID) EventOpt {
	return func(options *eventOptions) {
		options.activityID = activityID
	}
}

// WithRelatedActivityID specifies the parent activity ID of the event to be written.
func WithRelatedActivityID(activityID guid.GUID) EventOpt {
	return func(options *eventOptions) {
		options.relatedActivityID = activityID
	}
}

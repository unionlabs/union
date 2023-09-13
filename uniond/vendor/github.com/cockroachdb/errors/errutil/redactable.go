package errutil

import (
	"context"
	"fmt"

	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/errorspb"
	"github.com/cockroachdb/redact"
	"github.com/gogo/protobuf/proto"
)

// leafError is like the basic error string in the stdlib except the
// message can contain redactable and non-redactable parts.
type leafError struct {
	msg redact.RedactableString
}

var _ error = (*leafError)(nil)
var _ fmt.Formatter = (*leafError)(nil)
var _ errbase.SafeFormatter = (*leafError)(nil)
var _ errbase.SafeDetailer = (*leafError)(nil)

func (l *leafError) Error() string                 { return l.msg.StripMarkers() }
func (l *leafError) Format(s fmt.State, verb rune) { errbase.FormatError(l, s, verb) }
func (l *leafError) SafeFormatError(p errbase.Printer) (next error) {
	p.Print(l.msg)
	return nil
}
func (l *leafError) SafeDetails() []string {
	return []string{l.msg.Redact().StripMarkers()}
}

func encodeLeaf(_ context.Context, err error) (string, []string, proto.Message) {
	l := err.(*leafError)
	return l.Error(), l.SafeDetails(), &errorspb.StringPayload{Msg: string(l.msg)}
}

func decodeLeaf(_ context.Context, _ string, _ []string, payload proto.Message) error {
	m, ok := payload.(*errorspb.StringPayload)
	if !ok {
		// If this ever happens, this means some version of the library
		// (presumably future) changed the payload type, and we're
		// receiving this here. In this case, give up and let
		// DecodeError use the opaque type.
		return nil
	}
	return &leafError{msg: redact.RedactableString(m.Msg)}
}

func init() {
	errbase.RegisterLeafEncoder(errbase.GetTypeKey((*leafError)(nil)), encodeLeaf)
	errbase.RegisterLeafDecoder(errbase.GetTypeKey((*leafError)(nil)), decodeLeaf)
}

// withPrefix is like withMessage but the
// message can contain redactable and non-redactable parts.
type withPrefix struct {
	cause  error
	prefix redact.RedactableString
}

var _ error = (*withPrefix)(nil)
var _ fmt.Formatter = (*withPrefix)(nil)
var _ errbase.SafeFormatter = (*withPrefix)(nil)
var _ errbase.SafeDetailer = (*withPrefix)(nil)

func (l *withPrefix) Error() string {
	if l.prefix == "" {
		return l.cause.Error()
	}
	return fmt.Sprintf("%s: %v", l.prefix.StripMarkers(), l.cause)
}

func (l *withPrefix) Cause() error  { return l.cause }
func (l *withPrefix) Unwrap() error { return l.cause }

func (l *withPrefix) Format(s fmt.State, verb rune) { errbase.FormatError(l, s, verb) }
func (l *withPrefix) SafeFormatError(p errbase.Printer) (next error) {
	p.Print(l.prefix)
	return l.cause
}

func (l *withPrefix) SafeDetails() []string {
	return []string{l.prefix.Redact().StripMarkers()}
}

func encodeWithPrefix(_ context.Context, err error) (string, []string, proto.Message) {
	l := err.(*withPrefix)
	return l.Error(), l.SafeDetails(), &errorspb.StringPayload{Msg: string(l.prefix)}
}

func decodeWithPrefix(
	_ context.Context, cause error, _ string, _ []string, payload proto.Message,
) error {
	m, ok := payload.(*errorspb.StringPayload)
	if !ok {
		// If this ever happens, this means some version of the library
		// (presumably future) changed the payload type, and we're
		// receiving this here. In this case, give up and let
		// DecodeError use the opaque type.
		return nil
	}
	return &withPrefix{cause: cause, prefix: redact.RedactableString(m.Msg)}
}

func init() {
	errbase.RegisterWrapperEncoder(errbase.GetTypeKey((*withPrefix)(nil)), encodeWithPrefix)
	errbase.RegisterWrapperDecoder(errbase.GetTypeKey((*withPrefix)(nil)), decodeWithPrefix)
}

// withNewMessage is like withPrefix but the message completely
// overrides that of the underlying error.
type withNewMessage struct {
	cause   error
	message redact.RedactableString
}

var _ error = (*withNewMessage)(nil)
var _ fmt.Formatter = (*withNewMessage)(nil)
var _ errbase.SafeFormatter = (*withNewMessage)(nil)
var _ errbase.SafeDetailer = (*withNewMessage)(nil)

func (l *withNewMessage) Error() string {
	return l.message.StripMarkers()
}

func (l *withNewMessage) Cause() error  { return l.cause }
func (l *withNewMessage) Unwrap() error { return l.cause }

func (l *withNewMessage) Format(s fmt.State, verb rune) { errbase.FormatError(l, s, verb) }
func (l *withNewMessage) SafeFormatError(p errbase.Printer) (next error) {
	p.Print(l.message)
	return nil /* nil here overrides the cause's message */
}

func (l *withNewMessage) SafeDetails() []string {
	return []string{l.message.Redact().StripMarkers()}
}

func encodeWithNewMessage(_ context.Context, err error) (string, []string, proto.Message) {
	l := err.(*withNewMessage)
	return l.Error(), l.SafeDetails(), &errorspb.StringPayload{Msg: string(l.message)}
}

func decodeWithNewMessage(
	_ context.Context, cause error, _ string, _ []string, payload proto.Message,
) error {
	m, ok := payload.(*errorspb.StringPayload)
	if !ok {
		// If this ever happens, this means some version of the library
		// (presumably future) changed the payload type, and we're
		// receiving this here. In this case, give up and let
		// DecodeError use the opaque type.
		return nil
	}
	return &withNewMessage{cause: cause, message: redact.RedactableString(m.Msg)}
}

func init() {
	errbase.RegisterWrapperEncoder(errbase.GetTypeKey((*withNewMessage)(nil)), encodeWithNewMessage)
	errbase.RegisterWrapperDecoder(errbase.GetTypeKey((*withNewMessage)(nil)), decodeWithNewMessage)
}

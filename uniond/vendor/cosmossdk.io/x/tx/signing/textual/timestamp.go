package textual

import (
	"context"
	"fmt"
	"time"

	"google.golang.org/protobuf/reflect/protoreflect"
	tspb "google.golang.org/protobuf/types/known/timestamppb"
)

type timestampValueRenderer struct{}

// NewTimestampValueRenderer returns a ValueRenderer for protocol buffer Timestamp messages.
// It renders timestamps using the RFC 3339 format, always using UTC as the timezone.
// Fractional seconds are only rendered if nonzero.
func NewTimestampValueRenderer() ValueRenderer {
	return timestampValueRenderer{}
}

// Format implements the ValueRenderer interface.
func (vr timestampValueRenderer) Format(_ context.Context, v protoreflect.Value) ([]Screen, error) {
	ts := &tspb.Timestamp{}
	err := coerceToMessage(v.Message().Interface(), ts)
	if err != nil {
		return nil, err
	}
	t := ts.AsTime()

	// Format the Go Time as RFC 3339.
	s := t.Format(time.RFC3339Nano)
	return []Screen{{Content: s}}, nil
}

// Parse implements the ValueRenderer interface.
func (vr timestampValueRenderer) Parse(_ context.Context, screens []Screen) (protoreflect.Value, error) {
	// Parse the RFC 3339 input as a Go Time.
	if len(screens) != 1 {
		return nilValue, fmt.Errorf("expected single screen: %v", screens)
	}
	t, err := time.Parse(time.RFC3339Nano, screens[0].Content)
	if err != nil {
		return nilValue, err
	}

	// Convert Go Time to a proto Timestamp.
	timestamp := tspb.New(t)

	// Reflect the proto Timestamp.
	msg := timestamp.ProtoReflect()
	return protoreflect.ValueOfMessage(msg), nil
}

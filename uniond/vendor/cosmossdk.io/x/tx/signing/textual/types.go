package textual

import (
	"context"

	"google.golang.org/protobuf/reflect/protoreflect"
)

// Screen is the abstract unit of Textual rendering.
type Screen struct {
	// Title is the text (sequence of Unicode code points) to display first,
	// generally on the device's title section. It can be empty.
	Title string

	// Content is the text (sequence of Unicode code points) to display after
	// the Title, generally on the device's content section. It must be
	// non-empty.
	Content string

	// Indent is the indentation level of the screen.
	// Zero indicates top-level. Should be less than 16.
	Indent int

	// Expert indicates that the screen should only be displayed
	// via an opt-in from the user.
	Expert bool
}

// ValueRenderer defines an interface to produce formatted output for all
// protobuf types as well as parse a string into those protobuf types.
//
// The notion of "value renderer" is defined in ADR-050, and that ADR provides
// a default spec for value renderers. However, we define it as an interface
// here, so that optionally more value renderers could be built, for example, a
// separate one for a different language.
type ValueRenderer interface {
	// Format renders the Protobuf value to a list of Screens.
	Format(context.Context, protoreflect.Value) ([]Screen, error)

	// Parse is the inverse of Format. It must be able to parse all valid
	// screens, meaning only those generated using this renderer's Format method.
	// However the behavior of Parse on invalid screens is not specified,
	// and does not necessarily error.
	Parse(context.Context, []Screen) (protoreflect.Value, error)
}

// RepeatedValueRenderer defines an interface to produce formatted output for
// protobuf message fields that are repeated.
type RepeatedValueRenderer interface {
	ValueRenderer

	// FormatRepeated renders the Protobuf list value to a list of Screens.
	FormatRepeated(context.Context, protoreflect.Value) ([]Screen, error)

	// ParseRepeated is the inverse of FormatRepeated. It must parse all
	// valid screens, meaning only those generated using this renderer's
	// FormatRepeated method. However the behavior on invalid screens is not
	// specified, and does not necessarily error. The `protoreflect.List`
	// argument will be mutated and populated with the repeated values.
	ParseRepeated(context.Context, []Screen, protoreflect.List) error
}

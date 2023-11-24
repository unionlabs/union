package textual

import (
	"context"
	"fmt"

	"google.golang.org/protobuf/reflect/protoreflect"
)

type boolValueRenderer struct{}

// NewBoolValueRenderer returns a ValueRenderer for protocol buffer bool values.
// It renders the boolean as YES or NO.
func NewBoolValueRenderer() ValueRenderer {
	return boolValueRenderer{}
}

func (sr boolValueRenderer) Format(_ context.Context, v protoreflect.Value) ([]Screen, error) {
	str := "False"
	if v.Bool() {
		str = "True"
	}
	return []Screen{{Content: str}}, nil
}

func (sr boolValueRenderer) Parse(_ context.Context, screens []Screen) (protoreflect.Value, error) {
	if len(screens) != 1 {
		return protoreflect.Value{}, fmt.Errorf("expected single screen: %v", screens)
	}

	res := false
	if screens[0].Content == "True" {
		res = true
	}

	return protoreflect.ValueOfBool(res), nil
}

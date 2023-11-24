package textual

import (
	"context"
	"fmt"

	"google.golang.org/protobuf/reflect/protoreflect"
)

type enumValueRenderer struct {
	ed protoreflect.EnumDescriptor
}

func NewEnumValueRenderer(fd protoreflect.FieldDescriptor) ValueRenderer {
	ed := fd.Enum()
	if ed == nil {
		panic(fmt.Errorf("expected enum field, got %s", fd.Kind()))
	}

	return enumValueRenderer{ed: ed}
}

func (er enumValueRenderer) Format(_ context.Context, v protoreflect.Value) ([]Screen, error) {
	// Get the full name of the enum variant.
	evd := er.ed.Values().ByNumber(v.Enum())
	if evd == nil {
		return nil, fmt.Errorf("cannot get enum %s variant of number %d", er.ed.FullName(), v.Enum())
	}

	return []Screen{{Content: string(evd.Name())}}, nil
}

func (er enumValueRenderer) Parse(_ context.Context, screens []Screen) (protoreflect.Value, error) {
	if len(screens) != 1 {
		return nilValue, fmt.Errorf("expected single screen: %v", screens)
	}

	formatted := screens[0].Content

	// Loop through all enum variants until we find the one matching the
	// formatted screen's one.
	values := er.ed.Values()
	for i := 0; i < values.Len(); i++ {
		evd := values.Get(i)
		if string(evd.Name()) == formatted {
			return protoreflect.ValueOfEnum(evd.Number()), nil
		}
	}

	return nilValue, fmt.Errorf("cannot parse %s as enum on field %s", formatted, er.ed.FullName())
}

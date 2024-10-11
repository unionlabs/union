package schema

import (
	"errors"
	"fmt"
)

// EnumType represents the definition of an enum type.
type EnumType struct {
	// Name is the name of the enum type.
	// It must conform to the NameFormat regular expression.
	// Its name must be unique between all enum types and object types in the module.
	// The same enum, however, can be used in multiple object types and fields as long as the
	// definition is identical each time.
	Name string `json:"name,omitempty"`

	// Values is a list of distinct, non-empty values that are part of the enum type.
	// Each value must conform to the NameFormat regular expression.
	Values []EnumValueDefinition `json:"values"`

	// NumericKind is the numeric kind used to represent the enum values numerically.
	// If it is left empty, Int32Kind is used by default.
	// Valid values are Uint8Kind, Int8Kind, Uint16Kind, Int16Kind, and Int32Kind.
	NumericKind Kind `json:"numeric_kind,omitempty"`
}

// EnumValueDefinition represents a value in an enum type.
type EnumValueDefinition struct {
	// Name is the name of the enum value.
	// It must conform to the NameFormat regular expression.
	// Its name must be unique between all values in the enum.
	Name string `json:"name"`

	// Value is the numeric value of the enum.
	// It must be unique between all values in the enum.
	Value int32 `json:"value"`
}

// TypeName implements the Type interface.
func (e EnumType) TypeName() string {
	return e.Name
}

func (EnumType) isType()          {}
func (EnumType) isReferenceType() {}

// Validate validates the enum definition.
func (e EnumType) Validate(TypeSet) error {
	if !ValidateName(e.Name) {
		return fmt.Errorf("invalid enum definition name %q", e.Name)
	}

	if len(e.Values) == 0 {
		return errors.New("enum definition values cannot be empty")
	}
	names := make(map[string]bool, len(e.Values))
	values := make(map[int32]bool, len(e.Values))
	for i, v := range e.Values {
		if !ValidateName(v.Name) {
			return fmt.Errorf("invalid enum definition value %q at index %d for enum %s", v, i, e.Name)
		}

		if names[v.Name] {
			return fmt.Errorf("duplicate enum value name %q for enum %s", v.Name, e.Name)
		}
		names[v.Name] = true

		if values[v.Value] {
			return fmt.Errorf("duplicate enum numeric value %d for enum %s", v.Value, e.Name)
		}
		values[v.Value] = true

		switch e.GetNumericKind() {
		case Int8Kind:
			if v.Value < -128 || v.Value > 127 {
				return fmt.Errorf("enum value %q for enum %s is out of range for Int8Kind", v.Name, e.Name)
			}
		case Uint8Kind:
			if v.Value < 0 || v.Value > 255 {
				return fmt.Errorf("enum value %q for enum %s is out of range for Uint8Kind", v.Name, e.Name)
			}
		case Int16Kind:
			if v.Value < -32768 || v.Value > 32767 {
				return fmt.Errorf("enum value %q for enum %s is out of range for Int16Kind", v.Name, e.Name)
			}
		case Uint16Kind:
			if v.Value < 0 || v.Value > 65535 {
				return fmt.Errorf("enum value %q for enum %s is out of range for Uint16Kind", v.Name, e.Name)
			}
		case Int32Kind:
			// no range check needed
		default:
			return fmt.Errorf("invalid numeric kind %s for enum %s", e.NumericKind, e.Name)
		}
	}
	return nil
}

// ValidateValue validates that the value is a valid enum value.
func (e EnumType) ValidateValue(value string) error {
	for _, v := range e.Values {
		if v.Name == value {
			return nil
		}
	}
	return fmt.Errorf("value %q is not a valid enum value for %s", value, e.Name)
}

// GetNumericKind returns the numeric kind used to represent the enum values numerically.
// When EnumType.NumericKind is not set, the default value of Int32Kind is returned here.
func (e EnumType) GetNumericKind() Kind {
	if e.NumericKind == InvalidKind {
		return Int32Kind
	}
	return e.NumericKind
}

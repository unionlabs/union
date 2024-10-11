package schema

import (
	"fmt"
)

// Field represents a field in an object type.
type Field struct {
	// Name is the name of the field. It must conform to the NameFormat regular expression.
	Name string `json:"name"`

	// Kind is the basic type of the field.
	Kind Kind `json:"kind"`

	// Nullable indicates whether null values are accepted for the field. Key fields CANNOT be nullable.
	Nullable bool `json:"nullable,omitempty"`

	// ReferencedType is the referenced type name when Kind is EnumKind.
	ReferencedType string `json:"referenced_type,omitempty"`
}

// Validate validates the field.
func (c Field) Validate(typeSet TypeSet) error {
	// valid name
	if !ValidateName(c.Name) {
		return fmt.Errorf("invalid field name %q", c.Name)
	}

	// valid kind
	if err := c.Kind.Validate(); err != nil {
		return fmt.Errorf("invalid field kind for %q: %v", c.Name, err) //nolint:errorlint // false positive due to using go1.12
	}

	// enum definition only valid with EnumKind
	switch c.Kind {
	case EnumKind:
		if c.ReferencedType == "" {
			return fmt.Errorf("enum field %q must have a referenced type", c.Name)
		}

		_, ok := typeSet.LookupEnumType(c.ReferencedType)
		if !ok {
			return fmt.Errorf("can't find enum type %q referenced by field %q", c.ReferencedType, c.Name)
		}

	default:
		if c.ReferencedType != "" {
			return fmt.Errorf("field %q with kind %q cannot have a referenced type", c.Name, c.Kind)
		}
	}

	return nil
}

// ValidateValue validates that the value conforms to the field's kind and nullability.
// Unlike Kind.ValidateValue, it also checks that the value conforms to the EnumType
// if the field is an EnumKind.
func (c Field) ValidateValue(value interface{}, typeSet TypeSet) error {
	if value == nil {
		if !c.Nullable {
			return fmt.Errorf("field %q cannot be null", c.Name)
		}
		return nil
	}
	err := c.Kind.ValidateValueType(value)
	if err != nil {
		return fmt.Errorf("invalid value for field %q: %v", c.Name, err) //nolint:errorlint // false positive due to using go1.12
	}

	switch c.Kind {
	case EnumKind:
		enumType, ok := typeSet.LookupEnumType(c.ReferencedType)
		if !ok {
			return fmt.Errorf("enum field %q references unknown type %q", c.Name, c.ReferencedType)
		}
		err := enumType.ValidateValue(value.(string))
		if err != nil {
			return fmt.Errorf("invalid value for enum field %q: %v", c.Name, err) //nolint:errorlint // false positive due to using go1.12
		}
	default:
	}

	return nil
}

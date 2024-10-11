package schema

import (
	"strings"
	"testing"
)

func TestEnumDefinition_Validate(t *testing.T) {
	tests := []struct {
		name        string
		enum        EnumType
		errContains string
	}{
		{
			name: "valid enum",
			enum: EnumType{
				Name:   "test",
				Values: []EnumValueDefinition{{Name: "a", Value: 1}, {Name: "b", Value: 2}, {Name: "c", Value: 3}},
			},
			errContains: "",
		},
		{
			name: "empty name",
			enum: EnumType{
				Name:   "",
				Values: []EnumValueDefinition{{Name: "a", Value: 1}, {Name: "b", Value: 2}, {Name: "c", Value: 3}},
			},
			errContains: "invalid enum definition name",
		},
		{
			name: "empty values",
			enum: EnumType{
				Name:   "test",
				Values: []EnumValueDefinition{},
			},
			errContains: "enum definition values cannot be empty",
		},
		{
			name: "empty value name",
			enum: EnumType{
				Name:   "test",
				Values: []EnumValueDefinition{{Name: "a", Value: 1}, {Name: "", Value: 2}, {Name: "c", Value: 3}},
			},
			errContains: "invalid enum definition value",
		},
		{
			name: "duplicate value name",
			enum: EnumType{
				Name:   "test",
				Values: []EnumValueDefinition{{Name: "a", Value: 1}, {Name: "a", Value: 2}, {Name: "c", Value: 3}},
			},
			errContains: `duplicate enum value name "a" for enum test`,
		},
		{
			name: "duplicate value numeric",
			enum: EnumType{
				Name:   "test",
				Values: []EnumValueDefinition{{Name: "a", Value: 1}, {Name: "b", Value: 1}, {Name: "c", Value: 3}},
			},
			errContains: `duplicate enum numeric value 1 for enum test`,
		},
		{
			name: "invalid numeric kind",
			enum: EnumType{
				Name:        "test",
				NumericKind: StringKind,
				Values:      []EnumValueDefinition{{Name: "a", Value: 1}, {Name: "b", Value: 2}, {Name: "c", Value: 3}},
			},
			errContains: "invalid numeric kind",
		},
		{
			name: "out of range value for Int8Kind",
			enum: EnumType{
				Name:        "test",
				NumericKind: Int8Kind,
				Values:      []EnumValueDefinition{{Name: "a", Value: -129}},
			},
			errContains: "out of range",
		},
		{
			name: "out of range value for Uint8Kind",
			enum: EnumType{
				Name:        "test",
				NumericKind: Uint8Kind,
				Values:      []EnumValueDefinition{{Name: "a", Value: -1}},
			},
			errContains: "out of range",
		},
		{
			name: "out of range value for Int16Kind",
			enum: EnumType{
				Name:        "test",
				NumericKind: Int16Kind,
				Values:      []EnumValueDefinition{{Name: "a", Value: -32769}},
			},
			errContains: "out of range",
		},
		{
			name: "out of range value for Uint16Kind",
			enum: EnumType{
				Name:        "test",
				NumericKind: Uint16Kind,
				Values:      []EnumValueDefinition{{Name: "a", Value: -1}},
			},
			errContains: "out of range",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.enum.Validate(EmptyTypeSet())
			if tt.errContains == "" {
				if err != nil {
					t.Errorf("expected valid enum definition to pass validation, got: %v", err)
				}
			} else {
				if err == nil {
					t.Errorf("expected invalid enum definition to fail validation, got nil error")
				} else if !strings.Contains(err.Error(), tt.errContains) {
					t.Errorf("expected error to contain %s, got: %v", tt.errContains, err)
				}
			}
		})
	}
}

func TestEnumDefinition_ValidateValue(t *testing.T) {
	enum := EnumType{
		Name:   "test",
		Values: []EnumValueDefinition{{Name: "a", Value: 1}, {Name: "b", Value: 2}, {Name: "c", Value: 3}},
	}

	tests := []struct {
		value       string
		errContains string
	}{
		{"a", ""},
		{"b", ""},
		{"c", ""},
		{"d", "value \"d\" is not a valid enum value for test"},
	}

	for _, tt := range tests {
		t.Run(tt.value, func(t *testing.T) {
			err := enum.ValidateValue(tt.value)
			if tt.errContains == "" {
				if err != nil {
					t.Errorf("expected valid enum value to pass validation, got: %v", err)
				}
			} else {
				if err == nil {
					t.Errorf("expected invalid enum value to fail validation, got nil error")
				} else if !strings.Contains(err.Error(), tt.errContains) {
					t.Errorf("expected error to contain %s, got: %v", tt.errContains, err)
				}
			}
		})
	}
}

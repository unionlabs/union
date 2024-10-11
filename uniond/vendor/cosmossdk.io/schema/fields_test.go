package schema

import (
	"strings"
	"testing"
)

func TestValidateForKeyFields(t *testing.T) {
	tests := []struct {
		name        string
		keyFields   []Field
		key         interface{}
		errContains string
	}{
		{
			name:      "no key fields",
			keyFields: nil,
			key:       nil,
		},
		{
			name:        "single key field, valid",
			keyFields:   object1Type.KeyFields,
			key:         "hello",
			errContains: "",
		},
		{
			name:        "single key field, invalid",
			keyFields:   object1Type.KeyFields,
			key:         []interface{}{"value"},
			errContains: "invalid value",
		},
		{
			name:      "multiple key fields, valid",
			keyFields: object2Type.KeyFields,
			key:       []interface{}{"hello", int32(42)},
		},
		{
			name:        "multiple key fields, not a slice",
			keyFields:   object2Type.KeyFields,
			key:         map[string]interface{}{"field1": "hello", "field2": "42"},
			errContains: "expected slice of values",
		},
		{
			name:        "multiple key fields, wrong number of values",
			keyFields:   object2Type.KeyFields,
			key:         []interface{}{"hello"},
			errContains: "expected 2 key fields",
		},
		{
			name:        "multiple key fields, invalid value",
			keyFields:   object2Type.KeyFields,
			key:         []interface{}{"hello", "abc"},
			errContains: "invalid value",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := ValidateObjectKey(tt.keyFields, tt.key, EmptyTypeSet())
			if tt.errContains == "" {
				if err != nil {
					t.Fatalf("unexpected error: %v", err)
				}
			} else {
				if err == nil || !strings.Contains(err.Error(), tt.errContains) {
					t.Fatalf("expected error to contain %q, got: %v", tt.errContains, err)
				}
			}
		})
	}
}

func TestValidateForValueFields(t *testing.T) {
	tests := []struct {
		name        string
		valueFields []Field
		value       interface{}
		errContains string
	}{
		{
			name:        "no value fields",
			valueFields: nil,
			value:       nil,
		},
		{
			name: "single value field, valid",
			valueFields: []Field{
				{
					Name: "field1",
					Kind: StringKind,
				},
			},
			value:       "hello",
			errContains: "",
		},
		{
			name:        "value updates, empty",
			valueFields: object3Type.ValueFields,
			value:       MapValueUpdates(map[string]interface{}{}),
		},
		{
			name:        "value updates, 1 field valid",
			valueFields: object3Type.ValueFields,
			value: MapValueUpdates(map[string]interface{}{
				"field1": "hello",
			}),
		},
		{
			name:        "value updates, 2 fields, 1 invalid",
			valueFields: object3Type.ValueFields,
			value: MapValueUpdates(map[string]interface{}{
				"field1": "hello",
				"field2": "abc",
			}),
			errContains: "expected int32",
		},
		{
			name:        "value updates, extra value",
			valueFields: object3Type.ValueFields,
			value: MapValueUpdates(map[string]interface{}{
				"field1": "hello",
				"field2": int32(42),
				"field3": "extra",
			}),
			errContains: "unexpected values",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := ValidateObjectValue(tt.valueFields, tt.value, EmptyTypeSet())
			if tt.errContains == "" {
				if err != nil {
					t.Fatalf("unexpected error: %v", err)
				}
			} else {
				if err == nil || !strings.Contains(err.Error(), tt.errContains) {
					t.Fatalf("expected error to contain %q, got: %v", tt.errContains, err)
				}
			}
		})
	}
}

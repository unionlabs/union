package schema

import (
	"strings"
	"testing"
)

var object1Type = StateObjectType{
	Name: "object1",
	KeyFields: []Field{
		{
			Name: "field1",
			Kind: StringKind,
		},
	},
}

var object2Type = StateObjectType{
	KeyFields: []Field{
		{
			Name: "field1",
			Kind: StringKind,
		},
		{
			Name: "field2",
			Kind: Int32Kind,
		},
	},
}

var object3Type = StateObjectType{
	Name: "object3",
	ValueFields: []Field{
		{
			Name: "field1",
			Kind: StringKind,
		},
		{
			Name: "field2",
			Kind: Int32Kind,
		},
	},
}

var object4Type = StateObjectType{
	Name: "object4",
	KeyFields: []Field{
		{
			Name: "field1",
			Kind: Int32Kind,
		},
	},
	ValueFields: []Field{
		{
			Name: "field2",
			Kind: StringKind,
		},
	},
}

func TestObjectType_Validate(t *testing.T) {
	tests := []struct {
		name        string
		objectType  StateObjectType
		errContains string
	}{
		{
			name:        "valid object type",
			objectType:  object1Type,
			errContains: "",
		},
		{
			name: "empty object type name",
			objectType: StateObjectType{
				Name: "",
				KeyFields: []Field{
					{
						Name: "field1",
						Kind: StringKind,
					},
				},
			},
			errContains: "invalid object type name",
		},
		{
			name: "invalid key field",
			objectType: StateObjectType{
				Name: "object1",
				KeyFields: []Field{
					{
						Name: "",
						Kind: StringKind,
					},
				},
			},
			errContains: "invalid field name",
		},
		{
			name: "invalid value field",
			objectType: StateObjectType{
				Name: "object1",
				ValueFields: []Field{
					{
						Kind: StringKind,
					},
				},
			},
			errContains: "invalid field name",
		},
		{
			name:        "no fields",
			objectType:  StateObjectType{Name: "object0"},
			errContains: "has no key or value fields",
		},
		{
			name: "duplicate field",
			objectType: StateObjectType{
				Name: "object1",
				KeyFields: []Field{
					{
						Name: "field1",
						Kind: StringKind,
					},
				},
				ValueFields: []Field{
					{
						Name: "field1",
						Kind: StringKind,
					},
				},
			},
			errContains: "duplicate field name",
		},
		{
			name: "duplicate field 22",
			objectType: StateObjectType{
				Name: "object1",
				KeyFields: []Field{
					{
						Name: "field1",
						Kind: StringKind,
					},
					{
						Name: "field1",
						Kind: StringKind,
					},
				},
			},
			errContains: "duplicate field name",
		},
		{
			name: "nullable key field",
			objectType: StateObjectType{
				Name: "objectNullKey",
				KeyFields: []Field{
					{
						Name:     "field1",
						Kind:     StringKind,
						Nullable: true,
					},
				},
			},
			errContains: "key field \"field1\" cannot be nullable",
		},
		{
			name: "float32 key field",
			objectType: StateObjectType{
				Name: "o1",
				KeyFields: []Field{
					{
						Name: "field1",
						Kind: Float32Kind,
					},
				},
			},
			errContains: "invalid key field kind",
		},
		{
			name: "float64 key field",
			objectType: StateObjectType{
				Name: "o1",
				KeyFields: []Field{
					{
						Name: "field1",
						Kind: Float64Kind,
					},
				},
			},
			errContains: "invalid key field kind",
		},
		{
			name: "json key field",
			objectType: StateObjectType{
				Name: "o1",
				KeyFields: []Field{
					{
						Name: "field1",
						Kind: JSONKind,
					},
				},
			},
			errContains: "invalid key field kind",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.objectType.Validate(EmptyTypeSet())
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

func TestObjectType_ValidateObjectUpdate(t *testing.T) {
	tests := []struct {
		name        string
		objectType  StateObjectType
		object      StateObjectUpdate
		errContains string
	}{
		{
			name:       "wrong name",
			objectType: object1Type,
			object: StateObjectUpdate{
				TypeName: "object2",
				Key:      "hello",
			},
			errContains: "does not match update type name",
		},
		{
			name:       "invalid value",
			objectType: object1Type,
			object: StateObjectUpdate{
				TypeName: "object1",
				Key:      123,
			},
			errContains: "invalid value",
		},
		{
			name:       "valid update",
			objectType: object4Type,
			object: StateObjectUpdate{
				TypeName: "object4",
				Key:      int32(123),
				Value:    "hello",
			},
		},
		{
			name:       "valid deletion",
			objectType: object4Type,
			object: StateObjectUpdate{
				TypeName: "object4",
				Key:      int32(123),
				Value:    "ignored!",
				Delete:   true,
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.objectType.ValidateObjectUpdate(tt.object, EmptyTypeSet())
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

package schema

import (
	"reflect"
	"strings"
	"testing"
)

func TestModuleSchema_Validate(t *testing.T) {
	tests := []struct {
		name        string
		types       []Type
		errContains string
	}{
		{
			name: "valid module schema",
			types: []Type{
				StateObjectType{
					Name: "object1",
					KeyFields: []Field{
						{
							Name: "field1",
							Kind: StringKind,
						},
					},
				},
			},
			errContains: "",
		},
		{
			name: "invalid object type",
			types: []Type{
				StateObjectType{
					Name: "",
					KeyFields: []Field{
						{
							Name: "field1",
							Kind: StringKind,
						},
					},
				},
			},
			errContains: "invalid object type name",
		},
		{
			name: "duplicate type name",
			types: []Type{
				StateObjectType{
					Name: "type1",
					ValueFields: []Field{
						{
							Name:           "field1",
							Kind:           EnumKind,
							ReferencedType: "type1",
						},
					},
				},
				EnumType{
					Name:   "type1",
					Values: []EnumValueDefinition{{Name: "a", Value: 1}},
				},
			},
			errContains: `duplicate type "type1"`,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// because validate is called when calling CompileModuleSchema, we just call CompileModuleSchema
			_, err := CompileModuleSchema(tt.types...)
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

func TestModuleSchema_ValidateObjectUpdate(t *testing.T) {
	tests := []struct {
		name         string
		moduleSchema ModuleSchema
		objectUpdate StateObjectUpdate
		errContains  string
	}{
		{
			name: "valid object update",
			moduleSchema: requireModuleSchema(t,
				StateObjectType{
					Name: "object1",
					KeyFields: []Field{
						{
							Name: "field1",
							Kind: StringKind,
						},
					},
				},
			),
			objectUpdate: StateObjectUpdate{
				TypeName: "object1",
				Key:      "abc",
			},
			errContains: "",
		},
		{
			name: "object type not found",
			moduleSchema: requireModuleSchema(t,
				StateObjectType{
					Name: "object1",
					KeyFields: []Field{
						{
							Name: "field1",
							Kind: StringKind,
						},
					},
				},
			),
			objectUpdate: StateObjectUpdate{
				TypeName: "object2",
				Key:      "abc",
			},
			errContains: "object type \"object2\" not found in module schema",
		},
		{
			name: "type name refers to an enum",
			moduleSchema: requireModuleSchema(t, StateObjectType{
				Name: "obj1",
				KeyFields: []Field{
					{
						Name:           "field1",
						Kind:           EnumKind,
						ReferencedType: "enum1",
					},
				},
			},
				EnumType{
					Name:   "enum1",
					Values: []EnumValueDefinition{{Name: "a", Value: 1}, {Name: "b", Value: 2}},
				},
			),
			objectUpdate: StateObjectUpdate{
				TypeName: "enum1",
				Key:      "a",
			},
			errContains: "type \"enum1\" is not an object type",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.moduleSchema.ValidateObjectUpdate(tt.objectUpdate)
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

func requireModuleSchema(t *testing.T, types ...Type) ModuleSchema {
	t.Helper()
	moduleSchema, err := CompileModuleSchema(types...)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	return moduleSchema
}

func TestModuleSchema_LookupType(t *testing.T) {
	moduleSchema := requireModuleSchema(t, StateObjectType{
		Name: "object1",
		KeyFields: []Field{
			{
				Name: "field1",
				Kind: StringKind,
			},
		},
	})

	objectType, ok := moduleSchema.LookupStateObjectType("object1")
	if !ok {
		t.Fatalf("expected to find object type \"object1\"")
	}

	if objectType.Name != "object1" {
		t.Fatalf("expected object type name \"object1\", got %q", objectType.Name)
	}
}

func exampleSchema(t *testing.T) ModuleSchema {
	t.Helper()
	return requireModuleSchema(t,
		StateObjectType{
			Name: "object1",
			KeyFields: []Field{
				{
					Name:           "field1",
					Kind:           EnumKind,
					ReferencedType: "enum2",
				},
			},
		},
		StateObjectType{
			Name: "object2",
			KeyFields: []Field{
				{
					Name:           "field1",
					Kind:           EnumKind,
					ReferencedType: "enum1",
				},
			},
		},
		EnumType{
			Name:   "enum1",
			Values: []EnumValueDefinition{{Name: "a", Value: 1}, {Name: "b", Value: 2}, {Name: "c", Value: 3}},
		},
		EnumType{
			Name:   "enum2",
			Values: []EnumValueDefinition{{Name: "d", Value: 4}, {Name: "e", Value: 5}, {Name: "f", Value: 6}},
		},
	)
}

func TestModuleSchema_Types(t *testing.T) {
	moduleSchema := exampleSchema(t)

	var typeNames []string
	moduleSchema.AllTypes(func(typ Type) bool {
		typeNames = append(typeNames, typ.TypeName())
		return true
	})

	expected := []string{"enum1", "enum2", "object1", "object2"}
	if !reflect.DeepEqual(typeNames, expected) {
		t.Fatalf("expected %v, got %v", expected, typeNames)
	}

	typeNames = nil
	// scan just the first type and return false
	moduleSchema.AllTypes(func(typ Type) bool {
		typeNames = append(typeNames, typ.TypeName())
		return false
	})

	expected = []string{"enum1"}
	if !reflect.DeepEqual(typeNames, expected) {
		t.Fatalf("expected %v, got %v", expected, typeNames)
	}
}

func TestModuleSchema_ObjectTypes(t *testing.T) {
	moduleSchema := exampleSchema(t)

	var typeNames []string
	moduleSchema.StateObjectTypes(func(typ StateObjectType) bool {
		typeNames = append(typeNames, typ.Name)
		return true
	})

	expected := []string{"object1", "object2"}
	if !reflect.DeepEqual(typeNames, expected) {
		t.Fatalf("expected %v, got %v", expected, typeNames)
	}

	typeNames = nil
	// scan just the first type and return false
	moduleSchema.StateObjectTypes(func(typ StateObjectType) bool {
		typeNames = append(typeNames, typ.Name)
		return false
	})

	expected = []string{"object1"}
	if !reflect.DeepEqual(typeNames, expected) {
		t.Fatalf("expected %v, got %v", expected, typeNames)
	}
}

func TestModuleSchema_EnumTypes(t *testing.T) {
	moduleSchema := exampleSchema(t)

	var typeNames []string
	moduleSchema.EnumTypes(func(typ EnumType) bool {
		typeNames = append(typeNames, typ.Name)
		return true
	})

	expected := []string{"enum1", "enum2"}
	if !reflect.DeepEqual(typeNames, expected) {
		t.Fatalf("expected %v, got %v", expected, typeNames)
	}

	typeNames = nil
	// scan just the first type and return false
	moduleSchema.EnumTypes(func(typ EnumType) bool {
		typeNames = append(typeNames, typ.Name)
		return false
	})

	expected = []string{"enum1"}
	if !reflect.DeepEqual(typeNames, expected) {
		t.Fatalf("expected %v, got %v", expected, typeNames)
	}
}

func TestModuleSchemaJSON(t *testing.T) {
	moduleSchema := exampleSchema(t)

	b, err := moduleSchema.MarshalJSON()
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	const expectedJson = `{"object_types":[{"name":"object1","key_fields":[{"name":"field1","kind":"enum","referenced_type":"enum2"}]},{"name":"object2","key_fields":[{"name":"field1","kind":"enum","referenced_type":"enum1"}]}],"enum_types":[{"name":"enum1","values":[{"name":"a","value":1},{"name":"b","value":2},{"name":"c","value":3}]},{"name":"enum2","values":[{"name":"d","value":4},{"name":"e","value":5},{"name":"f","value":6}]}]}`
	if string(b) != expectedJson {
		t.Fatalf("expected %s\n, got %s", expectedJson, string(b))
	}

	var moduleSchema2 ModuleSchema
	err = moduleSchema2.UnmarshalJSON(b)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if !reflect.DeepEqual(moduleSchema, moduleSchema2) {
		t.Fatalf("expected %v, got %v", moduleSchema, moduleSchema2)
	}
}

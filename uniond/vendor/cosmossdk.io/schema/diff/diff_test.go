package diff

import (
	"reflect"
	"testing"

	"cosmossdk.io/schema"
)

func TestCompareModuleSchemas(t *testing.T) {
	tt := []struct {
		name                 string
		oldSchema            schema.ModuleSchema
		newSchema            schema.ModuleSchema
		diff                 ModuleSchemaDiff
		hasCompatibleChanges bool
		empty                bool
	}{
		{
			name: "no change",
			oldSchema: requireModuleSchema(t, schema.StateObjectType{
				Name:      "object1",
				KeyFields: []schema.Field{{Name: "key1", Kind: schema.StringKind}},
			}),
			newSchema: requireModuleSchema(t, schema.StateObjectType{
				Name:      "object1",
				KeyFields: []schema.Field{{Name: "key1", Kind: schema.StringKind}},
			}),
			diff:                 ModuleSchemaDiff{},
			hasCompatibleChanges: true,
			empty:                true,
		},
		{
			name:      "object type added",
			oldSchema: requireModuleSchema(t),
			newSchema: requireModuleSchema(t, schema.StateObjectType{
				Name:      "object1",
				KeyFields: []schema.Field{{Name: "key1", Kind: schema.StringKind}},
			}),
			diff: ModuleSchemaDiff{
				AddedStateObjectTypes: []schema.StateObjectType{
					{
						Name:      "object1",
						KeyFields: []schema.Field{{Name: "key1", Kind: schema.StringKind}},
					},
				},
			},
			hasCompatibleChanges: true,
		},
		{
			name: "object type removed",
			oldSchema: requireModuleSchema(t, schema.StateObjectType{
				Name:      "object1",
				KeyFields: []schema.Field{{Name: "key1", Kind: schema.StringKind}},
			}),
			newSchema: requireModuleSchema(t),
			diff: ModuleSchemaDiff{
				RemovedStateObjectTypes: []schema.StateObjectType{
					{
						Name:      "object1",
						KeyFields: []schema.Field{{Name: "key1", Kind: schema.StringKind}},
					},
				},
			},
			hasCompatibleChanges: false,
		},
		{
			name: "object type changed, key field added",
			oldSchema: requireModuleSchema(t, schema.StateObjectType{
				Name:      "object1",
				KeyFields: []schema.Field{{Name: "key1", Kind: schema.StringKind}},
			}),
			newSchema: requireModuleSchema(t, schema.StateObjectType{
				Name:      "object1",
				KeyFields: []schema.Field{{Name: "key1", Kind: schema.StringKind}, {Name: "key2", Kind: schema.StringKind}},
			}),
			diff: ModuleSchemaDiff{
				ChangedStateObjectTypes: []StateObjectTypeDiff{
					{
						Name: "object1",
						KeyFieldsDiff: FieldsDiff{
							Added: []schema.Field{
								{Name: "key2", Kind: schema.StringKind},
							},
						},
					},
				},
			},
			hasCompatibleChanges: false,
		},
		{
			name: "object type changed, nullable value field added",
			oldSchema: requireModuleSchema(t, schema.StateObjectType{
				Name:      "object1",
				KeyFields: []schema.Field{{Name: "key1", Kind: schema.StringKind}},
			}),
			newSchema: requireModuleSchema(t, schema.StateObjectType{
				Name:        "object1",
				KeyFields:   []schema.Field{{Name: "key1", Kind: schema.StringKind}},
				ValueFields: []schema.Field{{Name: "value1", Kind: schema.StringKind, Nullable: true}},
			}),
			diff: ModuleSchemaDiff{
				ChangedStateObjectTypes: []StateObjectTypeDiff{
					{
						Name: "object1",
						ValueFieldsDiff: FieldsDiff{
							Added: []schema.Field{{Name: "value1", Kind: schema.StringKind, Nullable: true}},
						},
					},
				},
			},
			hasCompatibleChanges: true,
		},
		{
			name: "object type changed, non-nullable value field added",
			oldSchema: requireModuleSchema(t, schema.StateObjectType{
				Name:      "object1",
				KeyFields: []schema.Field{{Name: "key1", Kind: schema.StringKind}},
			}),
			newSchema: requireModuleSchema(t, schema.StateObjectType{
				Name:        "object1",
				KeyFields:   []schema.Field{{Name: "key1", Kind: schema.StringKind}},
				ValueFields: []schema.Field{{Name: "value1", Kind: schema.StringKind}},
			}),
			diff: ModuleSchemaDiff{
				ChangedStateObjectTypes: []StateObjectTypeDiff{
					{
						Name: "object1",
						ValueFieldsDiff: FieldsDiff{
							Added: []schema.Field{{Name: "value1", Kind: schema.StringKind}},
						},
					},
				},
			},
			hasCompatibleChanges: false,
		},
		{
			name: "object type changed, fields reordered",
			oldSchema: requireModuleSchema(t, schema.StateObjectType{
				Name:      "object1",
				KeyFields: []schema.Field{{Name: "key1", Kind: schema.StringKind}, {Name: "key2", Kind: schema.StringKind}},
			}),
			newSchema: requireModuleSchema(t, schema.StateObjectType{
				Name:      "object1",
				KeyFields: []schema.Field{{Name: "key2", Kind: schema.StringKind}, {Name: "key1", Kind: schema.StringKind}},
			}),
			diff: ModuleSchemaDiff{
				ChangedStateObjectTypes: []StateObjectTypeDiff{
					{
						Name: "object1",
						KeyFieldsDiff: FieldsDiff{
							OldOrder: []string{"key1", "key2"},
							NewOrder: []string{"key2", "key1"},
						},
					},
				},
			},
			hasCompatibleChanges: false,
		},
		{
			name: "enum type added, nullable value field added",
			oldSchema: requireModuleSchema(t, schema.StateObjectType{
				Name:      "object1",
				KeyFields: []schema.Field{{Name: "key1", Kind: schema.Int32Kind}},
			}),
			newSchema: requireModuleSchema(t, schema.StateObjectType{
				Name:      "object1",
				KeyFields: []schema.Field{{Name: "key1", Kind: schema.Int32Kind}},
				ValueFields: []schema.Field{
					{
						Name:           "value1",
						Kind:           schema.EnumKind,
						ReferencedType: "enum1",
						Nullable:       true,
					},
				},
			},
				schema.EnumType{Name: "enum1", Values: []schema.EnumValueDefinition{{Name: "a", Value: 1}, {Name: "b", Value: 2}}}),
			diff: ModuleSchemaDiff{
				ChangedStateObjectTypes: []StateObjectTypeDiff{
					{
						Name: "object1",
						ValueFieldsDiff: FieldsDiff{
							Added: []schema.Field{
								{
									Name:           "value1",
									Kind:           schema.EnumKind,
									ReferencedType: "enum1",
									Nullable:       true,
								},
							},
						},
					},
				},
				AddedEnumTypes: []schema.EnumType{
					{Name: "enum1", Values: []schema.EnumValueDefinition{{Name: "a", Value: 1}, {Name: "b", Value: 2}}},
				},
			},
			hasCompatibleChanges: true,
		},
		{
			name: "enum type removed",
			oldSchema: requireModuleSchema(t,
				schema.StateObjectType{
					Name:      "object1",
					KeyFields: []schema.Field{{Name: "key1", Kind: schema.Int32Kind}},
					ValueFields: []schema.Field{
						{
							Name:           "value1",
							Kind:           schema.EnumKind,
							ReferencedType: "enum1",
						},
					},
				},
				schema.EnumType{Name: "enum1", Values: []schema.EnumValueDefinition{{Name: "a", Value: 1}, {Name: "b", Value: 2}}}),
			newSchema: requireModuleSchema(t, schema.StateObjectType{
				Name:      "object1",
				KeyFields: []schema.Field{{Name: "key1", Kind: schema.Int32Kind}},
			}),
			diff: ModuleSchemaDiff{
				ChangedStateObjectTypes: []StateObjectTypeDiff{
					{
						Name: "object1",
						ValueFieldsDiff: FieldsDiff{
							Removed: []schema.Field{
								{
									Name:           "value1",
									Kind:           schema.EnumKind,
									ReferencedType: "enum1",
								},
							},
						},
					},
				},
				RemovedEnumTypes: []schema.EnumType{
					{Name: "enum1", Values: []schema.EnumValueDefinition{{Name: "a", Value: 1}, {Name: "b", Value: 2}}},
				},
			},
			hasCompatibleChanges: false,
		},
		{
			name: "enum value added",
			oldSchema: requireModuleSchema(t,
				schema.EnumType{Name: "enum1", Values: []schema.EnumValueDefinition{{Name: "a", Value: 1}}},
			),
			newSchema: requireModuleSchema(t,
				schema.EnumType{Name: "enum1", Values: []schema.EnumValueDefinition{{Name: "a", Value: 1}, {Name: "b", Value: 2}}},
			),
			diff: ModuleSchemaDiff{
				ChangedEnumTypes: []EnumTypeDiff{
					{
						Name:        "enum1",
						AddedValues: []schema.EnumValueDefinition{{Name: "b", Value: 2}},
					},
				},
			},
			hasCompatibleChanges: true,
		},
		{
			name: "enum value removed",
			oldSchema: requireModuleSchema(t,
				schema.EnumType{Name: "enum1", Values: []schema.EnumValueDefinition{{Name: "a", Value: 1}, {Name: "b", Value: 2}, {Name: "c", Value: 3}}},
			),
			newSchema: requireModuleSchema(t,
				schema.EnumType{Name: "enum1", Values: []schema.EnumValueDefinition{{Name: "a", Value: 1}, {Name: "b", Value: 2}}},
			),
			diff: ModuleSchemaDiff{
				ChangedEnumTypes: []EnumTypeDiff{
					{
						Name:          "enum1",
						RemovedValues: []schema.EnumValueDefinition{{Name: "c", Value: 3}},
					},
				},
			},
			hasCompatibleChanges: false,
		},
		{
			name: "object type and enum type name switched",
			oldSchema: requireModuleSchema(t,
				schema.StateObjectType{
					Name:      "foo",
					KeyFields: []schema.Field{{Name: "key1", Kind: schema.EnumKind, ReferencedType: "bar"}},
				},
				schema.EnumType{Name: "bar", Values: []schema.EnumValueDefinition{{Name: "a", Value: 1}}},
			),
			newSchema: requireModuleSchema(t,
				schema.StateObjectType{
					Name:      "bar",
					KeyFields: []schema.Field{{Name: "key1", Kind: schema.EnumKind, ReferencedType: "foo"}},
				},
				schema.EnumType{Name: "foo", Values: []schema.EnumValueDefinition{{Name: "a", Value: 1}}},
			),
			diff: ModuleSchemaDiff{
				RemovedStateObjectTypes: []schema.StateObjectType{
					{
						Name:      "foo",
						KeyFields: []schema.Field{{Name: "key1", Kind: schema.EnumKind, ReferencedType: "bar"}},
					},
				},
				AddedStateObjectTypes: []schema.StateObjectType{
					{
						Name:      "bar",
						KeyFields: []schema.Field{{Name: "key1", Kind: schema.EnumKind, ReferencedType: "foo"}},
					},
				},
				RemovedEnumTypes: []schema.EnumType{
					{Name: "bar", Values: []schema.EnumValueDefinition{{Name: "a", Value: 1}}},
				},
				AddedEnumTypes: []schema.EnumType{
					{Name: "foo", Values: []schema.EnumValueDefinition{{Name: "a", Value: 1}}},
				},
			},
			hasCompatibleChanges: false,
		},
	}

	for _, tc := range tt {
		t.Run(tc.name, func(t *testing.T) {
			got := CompareModuleSchemas(tc.oldSchema, tc.newSchema)
			if !reflect.DeepEqual(got, tc.diff) {
				t.Errorf("CompareModuleSchemas() = %v, want %v", got, tc.diff)
			}
			hasCompatibleChanges := got.HasCompatibleChanges()
			if hasCompatibleChanges != tc.hasCompatibleChanges {
				t.Errorf("HasCompatibleChanges() = %v, want %v", hasCompatibleChanges, tc.hasCompatibleChanges)
			}
			if tc.empty != got.Empty() {
				t.Errorf("Empty() = %v, want %v", got.Empty(), tc.empty)
			}
		})
	}
}

func requireModuleSchema(t *testing.T, types ...schema.Type) schema.ModuleSchema {
	t.Helper()
	s, err := schema.CompileModuleSchema(types...)
	if err != nil {
		t.Fatal(err)
	}
	return s
}

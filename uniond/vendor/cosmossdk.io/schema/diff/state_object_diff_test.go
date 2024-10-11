package diff

import (
	"reflect"
	"testing"

	"cosmossdk.io/schema"
)

func Test_objectTypeDiff(t *testing.T) {
	tt := []struct {
		name                 string
		oldType              schema.StateObjectType
		newType              schema.StateObjectType
		diff                 StateObjectTypeDiff
		trueF                func(StateObjectTypeDiff) bool
		hasCompatibleChanges bool
	}{
		{
			name: "no change",
			oldType: schema.StateObjectType{
				KeyFields: []schema.Field{{Name: "id", Kind: schema.Int32Kind}},
			},
			newType: schema.StateObjectType{
				KeyFields: []schema.Field{{Name: "id", Kind: schema.Int32Kind}},
			},
			diff:                 StateObjectTypeDiff{},
			trueF:                StateObjectTypeDiff.Empty,
			hasCompatibleChanges: true,
		},
		{
			name: "key fields changed",
			oldType: schema.StateObjectType{
				KeyFields: []schema.Field{{Name: "id", Kind: schema.Int32Kind}},
			},
			newType: schema.StateObjectType{
				KeyFields: []schema.Field{{Name: "id", Kind: schema.StringKind}},
			},
			diff: StateObjectTypeDiff{
				KeyFieldsDiff: FieldsDiff{
					Changed: []FieldDiff{
						{
							Name:    "id",
							OldKind: schema.Int32Kind,
							NewKind: schema.StringKind,
						},
					},
				},
			},
			trueF:                func(d StateObjectTypeDiff) bool { return !d.KeyFieldsDiff.Empty() },
			hasCompatibleChanges: false,
		},
		{
			name: "value fields changed",
			oldType: schema.StateObjectType{
				ValueFields: []schema.Field{{Name: "name", Kind: schema.StringKind}},
			},
			newType: schema.StateObjectType{
				ValueFields: []schema.Field{{Name: "name", Kind: schema.Int32Kind}},
			},
			diff: StateObjectTypeDiff{
				ValueFieldsDiff: FieldsDiff{
					Changed: []FieldDiff{
						{
							Name:    "name",
							OldKind: schema.StringKind,
							NewKind: schema.Int32Kind,
						},
					},
				},
			},
			trueF:                func(d StateObjectTypeDiff) bool { return !d.ValueFieldsDiff.Empty() },
			hasCompatibleChanges: false,
		},
		{
			name: "nullable value field added",
			oldType: schema.StateObjectType{
				ValueFields: []schema.Field{{Name: "id", Kind: schema.Int32Kind}},
			},
			newType: schema.StateObjectType{
				ValueFields: []schema.Field{{Name: "id", Kind: schema.Int32Kind}, {Name: "name", Kind: schema.StringKind, Nullable: true}},
			},
			diff: StateObjectTypeDiff{
				ValueFieldsDiff: FieldsDiff{
					Added: []schema.Field{{Name: "name", Kind: schema.StringKind, Nullable: true}},
				},
			},
			trueF:                func(d StateObjectTypeDiff) bool { return !d.ValueFieldsDiff.Empty() },
			hasCompatibleChanges: true,
		},
		{
			name: "non-nullable value field added",
			oldType: schema.StateObjectType{
				ValueFields: []schema.Field{{Name: "id", Kind: schema.Int32Kind}},
			},
			newType: schema.StateObjectType{
				ValueFields: []schema.Field{{Name: "id", Kind: schema.Int32Kind}, {Name: "name", Kind: schema.StringKind}},
			},
			diff: StateObjectTypeDiff{
				ValueFieldsDiff: FieldsDiff{
					Added: []schema.Field{{Name: "name", Kind: schema.StringKind}},
				},
			},
			trueF:                func(d StateObjectTypeDiff) bool { return !d.ValueFieldsDiff.Empty() },
			hasCompatibleChanges: false,
		},
		{
			name: "fields reordered",
			oldType: schema.StateObjectType{
				KeyFields:   []schema.Field{{Name: "id", Kind: schema.Int32Kind}, {Name: "name", Kind: schema.StringKind}},
				ValueFields: []schema.Field{{Name: "x", Kind: schema.Int32Kind}, {Name: "y", Kind: schema.StringKind}},
			},
			newType: schema.StateObjectType{
				KeyFields:   []schema.Field{{Name: "name", Kind: schema.StringKind}, {Name: "id", Kind: schema.Int32Kind}},
				ValueFields: []schema.Field{{Name: "y", Kind: schema.StringKind}, {Name: "x", Kind: schema.Int32Kind}},
			},
			diff: StateObjectTypeDiff{
				KeyFieldsDiff: FieldsDiff{
					OldOrder: []string{"id", "name"},
					NewOrder: []string{"name", "id"},
				},
				ValueFieldsDiff: FieldsDiff{
					OldOrder: []string{"x", "y"},
					NewOrder: []string{"y", "x"},
				},
			},
			trueF:                func(d StateObjectTypeDiff) bool { return !d.KeyFieldsDiff.Empty() && !d.ValueFieldsDiff.Empty() },
			hasCompatibleChanges: false,
		},
	}

	for _, tc := range tt {
		t.Run(tc.name, func(t *testing.T) {
			got := compareObjectType(tc.oldType, tc.newType)
			if !reflect.DeepEqual(got, tc.diff) {
				t.Errorf("compareObjectType() = %v, want %v", got, tc.diff)
			}
			hasCompatibleChanges := got.HasCompatibleChanges()
			if hasCompatibleChanges != tc.hasCompatibleChanges {
				t.Errorf("HasCompatibleChanges() = %v, want %v", hasCompatibleChanges, tc.hasCompatibleChanges)
			}
		})
	}
}

func Test_fieldsDiff(t *testing.T) {
	tt := []struct {
		name      string
		oldFields []schema.Field
		newFields []schema.Field
		diff      FieldsDiff
	}{
		{
			name:      "no change",
			oldFields: []schema.Field{{Name: "id", Kind: schema.Int32Kind}},
			newFields: []schema.Field{{Name: "id", Kind: schema.Int32Kind}},
		},
		{
			name:      "field added",
			oldFields: []schema.Field{{Name: "id", Kind: schema.Int32Kind}},
			newFields: []schema.Field{{Name: "id", Kind: schema.Int32Kind}, {Name: "name", Kind: schema.StringKind}},
			diff: FieldsDiff{
				Added: []schema.Field{{Name: "name", Kind: schema.StringKind}},
			},
		},
		{
			name:      "field removed",
			oldFields: []schema.Field{{Name: "id", Kind: schema.Int32Kind}, {Name: "name", Kind: schema.StringKind}},
			newFields: []schema.Field{{Name: "id", Kind: schema.Int32Kind}},
			diff: FieldsDiff{
				Removed: []schema.Field{{Name: "name", Kind: schema.StringKind}},
			},
		},
		{
			name:      "field changed",
			oldFields: []schema.Field{{Name: "id", Kind: schema.Int32Kind}, {Name: "name", Kind: schema.StringKind}},
			newFields: []schema.Field{{Name: "id", Kind: schema.Int32Kind}, {Name: "name", Kind: schema.Int32Kind}},
			diff: FieldsDiff{
				Changed: []FieldDiff{
					{
						Name:    "name",
						OldKind: schema.StringKind,
						NewKind: schema.Int32Kind,
					},
				},
			},
		},
		{
			name:      "field order changed",
			oldFields: []schema.Field{{Name: "id", Kind: schema.Int32Kind}, {Name: "name", Kind: schema.StringKind}},
			newFields: []schema.Field{{Name: "name", Kind: schema.StringKind}, {Name: "id", Kind: schema.Int32Kind}},
			diff: FieldsDiff{
				OldOrder: []string{"id", "name"},
				NewOrder: []string{"name", "id"},
			},
		},
		{
			name:      "field order changed with added fields",
			oldFields: []schema.Field{{Name: "id", Kind: schema.Int32Kind}},
			newFields: []schema.Field{{Name: "name", Kind: schema.StringKind}, {Name: "id", Kind: schema.Int32Kind}},
			diff: FieldsDiff{
				Added:    []schema.Field{{Name: "name", Kind: schema.StringKind}},
				OldOrder: []string{"id"},
				NewOrder: []string{"name", "id"},
			},
		},
		{
			name:      "field order changed with removed fields",
			oldFields: []schema.Field{{Name: "name", Kind: schema.StringKind}, {Name: "id", Kind: schema.Int32Kind}},
			newFields: []schema.Field{{Name: "id", Kind: schema.Int32Kind}},
			diff: FieldsDiff{
				Removed:  []schema.Field{{Name: "name", Kind: schema.StringKind}},
				OldOrder: []string{"name", "id"},
				NewOrder: []string{"id"},
			},
		},
		{
			name:      "field order changed with changed fields",
			oldFields: []schema.Field{{Name: "name", Kind: schema.StringKind}, {Name: "id", Kind: schema.Int32Kind}},
			newFields: []schema.Field{{Name: "id", Kind: schema.Int32Kind}, {Name: "name", Kind: schema.Int32Kind}},
			diff: FieldsDiff{
				Changed: []FieldDiff{
					{
						Name:    "name",
						OldKind: schema.StringKind,
						NewKind: schema.Int32Kind,
					},
				},
				OldOrder: []string{"name", "id"},
				NewOrder: []string{"id", "name"},
			},
		},
		{
			name: "added, removed, changed and reordered fields",
			oldFields: []schema.Field{
				{Name: "id", Kind: schema.Int32Kind},
				{Name: "name", Kind: schema.StringKind},
				{Name: "age", Kind: schema.Int32Kind},
			},
			newFields: []schema.Field{
				{Name: "name", Kind: schema.Int32Kind},
				{Name: "age", Kind: schema.Int32Kind},
				{Name: "email", Kind: schema.StringKind},
			},
			diff: FieldsDiff{
				Added:   []schema.Field{{Name: "email", Kind: schema.StringKind}},
				Removed: []schema.Field{{Name: "id", Kind: schema.Int32Kind}},
				Changed: []FieldDiff{
					{
						Name:    "name",
						OldKind: schema.StringKind,
						NewKind: schema.Int32Kind,
					},
				},
				OldOrder: []string{"id", "name", "age"},
				NewOrder: []string{"name", "age", "email"},
			},
		},
	}

	for _, tc := range tt {
		t.Run(tc.name, func(t *testing.T) {
			got := compareFields(tc.oldFields, tc.newFields)
			if !reflect.DeepEqual(got, tc.diff) {
				t.Errorf("compareFields() = %v, want %v", got, tc.diff)
			}
		})
	}
}

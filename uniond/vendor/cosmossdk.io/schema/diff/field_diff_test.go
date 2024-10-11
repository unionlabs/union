package diff

import (
	"fmt"
	"reflect"
	"testing"

	"cosmossdk.io/schema"
)

func Test_compareField(t *testing.T) {
	tests := []struct {
		oldField schema.Field
		newField schema.Field
		wantDiff FieldDiff
		trueF    func(FieldDiff) bool
	}{
		{
			oldField: schema.Field{Kind: schema.Int32Kind},
			newField: schema.Field{Kind: schema.Int32Kind},
			wantDiff: FieldDiff{},
			trueF:    FieldDiff.Empty,
		},
		{
			oldField: schema.Field{Kind: schema.StringKind},
			newField: schema.Field{Kind: schema.Int32Kind},
			wantDiff: FieldDiff{
				OldKind: schema.StringKind,
				NewKind: schema.Int32Kind,
			},
			trueF: FieldDiff.KindChanged,
		},
		{
			oldField: schema.Field{Kind: schema.StringKind},
			newField: schema.Field{Kind: schema.StringKind, Nullable: true},
			wantDiff: FieldDiff{
				NewNullable: true,
			},
			trueF: FieldDiff.NullableChanged,
		},
		{
			oldField: schema.Field{Kind: schema.EnumKind, ReferencedType: "old"},
			newField: schema.Field{Kind: schema.EnumKind, ReferencedType: "new"},
			wantDiff: FieldDiff{
				OldReferencedType: "old",
				NewReferencedType: "new",
			},
			trueF: FieldDiff.ReferenceTypeChanged,
		},
	}

	for i, tt := range tests {
		t.Run(fmt.Sprintf("%d", i), func(t *testing.T) {
			gotDiff := compareField(tt.oldField, tt.newField)
			if !reflect.DeepEqual(gotDiff, tt.wantDiff) {
				t.Errorf("compareField() = %v, want %v", gotDiff, tt.wantDiff)
			}
			if tt.trueF != nil && !tt.trueF(gotDiff) {
				t.Errorf("trueF() = false, want true")
			}
		})
	}
}

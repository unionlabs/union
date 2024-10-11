package diff

import "cosmossdk.io/schema"

// FieldDiff represents the difference between two fields.
// The KindChanged, NullableChanged, and ReferenceTypeChanged methods can be used to determine
// what specific changes were made to the field.
type FieldDiff struct {
	// Name is the name of the field.
	Name string

	// OldKind is the old kind of the field. It will be InvalidKind if there was no change.
	OldKind schema.Kind

	// NewKind is the new kind of the field. It will be InvalidKind if there was no change.
	NewKind schema.Kind

	// OldNullable is the old nullable property of the field.
	OldNullable bool

	// NewNullable is the new nullable property of the field.
	NewNullable bool

	// OldReferencedType is the name of the old referenced type.
	// It will be empty if the field is not a reference type or if there was no change.
	OldReferencedType string

	// NewReferencedType is the name of the new referenced type.
	// It will be empty if the field is not a reference type or if there was no change.
	NewReferencedType string
}

func compareField(oldField, newField schema.Field) FieldDiff {
	diff := FieldDiff{
		Name: oldField.Name,
	}
	if oldField.Kind != newField.Kind {
		diff.OldKind = oldField.Kind
		diff.NewKind = newField.Kind
	}

	diff.OldNullable = oldField.Nullable
	diff.NewNullable = newField.Nullable

	if oldField.ReferencedType != newField.ReferencedType {
		diff.OldReferencedType = oldField.ReferencedType
		diff.NewReferencedType = newField.ReferencedType
	}

	return diff
}

// Empty returns true if the field diff has no changes.
func (d FieldDiff) Empty() bool {
	return !d.KindChanged() && !d.NullableChanged() && !d.ReferenceTypeChanged()
}

// KindChanged returns true if the field kind changed.
func (d FieldDiff) KindChanged() bool {
	return d.OldKind != d.NewKind
}

// NullableChanged returns true if the field nullable property changed.
func (d FieldDiff) NullableChanged() bool {
	return d.OldNullable != d.NewNullable
}

// ReferenceTypeChanged returns true if the referenced type changed.
func (d FieldDiff) ReferenceTypeChanged() bool {
	return d.OldReferencedType != d.NewReferencedType
}

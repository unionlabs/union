package diff

import "cosmossdk.io/schema"

// EnumTypeDiff represents the difference between two enum types.
type EnumTypeDiff struct {
	// Name is the name of the enum type.
	Name string

	// AddedValues is a list of values that were added.
	AddedValues []schema.EnumValueDefinition

	// RemovedValues is a list of values that were removed.
	RemovedValues []schema.EnumValueDefinition

	// ChangedValues is a list of values whose numeric values were changed.
	ChangedValues []EnumValueDefinitionDiff

	// OldNumericKind is the numeric kind used to represent the enum values numerically in the old enum type.
	// It will be empty if the kind did not change.
	OldNumericKind schema.Kind

	// NewNumericKind is the numeric kind used to represent the enum values numerically in the new enum type.
	// It will be empty if the kind did not change.
	NewNumericKind schema.Kind
}

// EnumValueDefinitionDiff represents the difference between two enum values definitions.
type EnumValueDefinitionDiff struct {
	// Name is the name of the enum value.
	Name string

	// OldValue is the old numeric value of the enum value.
	OldValue int32

	// NewValue is the new numeric value of the enum value.
	NewValue int32
}

func compareEnumType(oldEnum, newEnum schema.EnumType) EnumTypeDiff {
	diff := EnumTypeDiff{
		Name: oldEnum.TypeName(),
	}

	if oldEnum.GetNumericKind() != newEnum.GetNumericKind() {
		diff.OldNumericKind = oldEnum.GetNumericKind()
		diff.NewNumericKind = newEnum.GetNumericKind()
	}

	newValues := make(map[string]schema.EnumValueDefinition)
	for _, v := range newEnum.Values {
		newValues[v.Name] = v
	}

	oldValues := make(map[string]schema.EnumValueDefinition)
	for _, v := range oldEnum.Values {
		oldValues[v.Name] = v
		newV, ok := newValues[v.Name]
		if !ok {
			diff.RemovedValues = append(diff.RemovedValues, v)
		} else if newV.Value != v.Value {
			diff.ChangedValues = append(diff.ChangedValues, EnumValueDefinitionDiff{
				Name:     v.Name,
				OldValue: v.Value,
				NewValue: newV.Value,
			})
		}
	}

	for _, v := range newEnum.Values {
		if _, ok := oldValues[v.Name]; !ok {
			diff.AddedValues = append(diff.AddedValues, v)
		}
	}

	return diff
}

// Empty returns true if the enum type diff has no changes.
func (e EnumTypeDiff) Empty() bool {
	return len(e.AddedValues) == 0 &&
		e.HasCompatibleChanges()
}

// HasCompatibleChanges returns true if the diff contains only compatible changes.
// The only supported compatible change is adding values.
func (e EnumTypeDiff) HasCompatibleChanges() bool {
	return len(e.RemovedValues) == 0 &&
		len(e.ChangedValues) == 0 &&
		!e.KindChanged()
}

func (e EnumTypeDiff) KindChanged() bool {
	return e.OldNumericKind != e.NewNumericKind
}

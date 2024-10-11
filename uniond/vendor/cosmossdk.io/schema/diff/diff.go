package diff

import "cosmossdk.io/schema"

// ModuleSchemaDiff represents the difference between two module schemas.
type ModuleSchemaDiff struct {
	// AddedStateObjectTypes is a list of object types that were added.
	AddedStateObjectTypes []schema.StateObjectType

	// ChangedStateObjectTypes is a list of object types that were changed.
	ChangedStateObjectTypes []StateObjectTypeDiff

	// RemovedStateObjectTypes is a list of object types that were removed.
	RemovedStateObjectTypes []schema.StateObjectType

	// AddedEnumTypes is a list of enum types that were added.
	AddedEnumTypes []schema.EnumType

	// ChangedEnumTypes is a list of enum types that were changed.
	ChangedEnumTypes []EnumTypeDiff

	// RemovedEnumTypes is a list of enum types that were removed.
	RemovedEnumTypes []schema.EnumType
}

// CompareModuleSchemas compares an old and a new module schemas and returns the difference between them.
// If the schemas are equivalent, the Empty method of the returned ModuleSchemaDiff will return true.
//
// Indexer implementations can use these diffs to perform automatic schema migration.
// The specific supported changes that a specific indexer supports are defined by that indexer implementation.
// However, as a general rule, it is suggested that indexers support the following changes to module schemas:
// - Adding object types
// - Adding enum types
// - Adding nullable value fields to object types
// - Adding enum values to enum types
//
// These changes are officially considered "compatible" changes, and the HasCompatibleChanges method of the returned
// ModuleSchemaDiff will return true if only compatible changes are present.
// Module authors can use the above guidelines as a reference point for what changes are generally
// considered safe to make to a module schema without breaking existing indexers.
func CompareModuleSchemas(oldSchema, newSchema schema.ModuleSchema) ModuleSchemaDiff {
	diff := ModuleSchemaDiff{}

	oldSchema.StateObjectTypes(func(oldObj schema.StateObjectType) bool {
		newObj, found := newSchema.LookupStateObjectType(oldObj.Name)
		if !found {
			diff.RemovedStateObjectTypes = append(diff.RemovedStateObjectTypes, oldObj)
			return true
		}
		objDiff := compareObjectType(oldObj, newObj)
		if !objDiff.Empty() {
			diff.ChangedStateObjectTypes = append(diff.ChangedStateObjectTypes, objDiff)
		}
		return true
	})

	newSchema.StateObjectTypes(func(newObj schema.StateObjectType) bool {
		_, found := oldSchema.LookupStateObjectType(newObj.TypeName())
		if !found {
			diff.AddedStateObjectTypes = append(diff.AddedStateObjectTypes, newObj)
		}
		return true
	})

	oldSchema.EnumTypes(func(oldEnum schema.EnumType) bool {
		newEnum, found := newSchema.LookupEnumType(oldEnum.Name)
		if !found {
			diff.RemovedEnumTypes = append(diff.RemovedEnumTypes, oldEnum)
			return true
		}
		enumDiff := compareEnumType(oldEnum, newEnum)
		if !enumDiff.Empty() {
			diff.ChangedEnumTypes = append(diff.ChangedEnumTypes, enumDiff)
		}
		return true
	})

	newSchema.EnumTypes(func(newEnum schema.EnumType) bool {
		_, found := oldSchema.LookupEnumType(newEnum.TypeName())
		if !found {
			diff.AddedEnumTypes = append(diff.AddedEnumTypes, newEnum)
		}
		return true
	})

	return diff
}

func (m ModuleSchemaDiff) Empty() bool {
	return len(m.AddedStateObjectTypes) == 0 &&
		len(m.ChangedStateObjectTypes) == 0 &&
		len(m.RemovedStateObjectTypes) == 0 &&
		len(m.AddedEnumTypes) == 0 &&
		len(m.ChangedEnumTypes) == 0 &&
		len(m.RemovedEnumTypes) == 0
}

// HasCompatibleChanges returns true if the diff contains only compatible changes.
// Compatible changes are changes that are generally safe to make to a module schema without breaking existing indexers
// and indexers should aim to automatically migrate to such changes.
// See the CompareModuleSchemas function for a list of changes that are considered compatible.
func (m ModuleSchemaDiff) HasCompatibleChanges() bool {
	// object and enum types can be added but not removed
	// changed object and enum types must have compatible changes
	if len(m.RemovedStateObjectTypes) != 0 || len(m.RemovedEnumTypes) != 0 {
		return false
	}

	for _, objectType := range m.ChangedStateObjectTypes {
		if !objectType.HasCompatibleChanges() {
			return false
		}
	}

	for _, enumType := range m.ChangedEnumTypes {
		if !enumType.HasCompatibleChanges() {
			return false
		}
	}

	return true
}

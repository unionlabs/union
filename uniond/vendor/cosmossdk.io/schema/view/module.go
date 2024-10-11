package view

import "cosmossdk.io/schema"

// ModuleState defines an interface for things that represent module state in schema format.
type ModuleState interface {
	// ModuleName returns the name of the module.
	ModuleName() string

	// ModuleSchema returns the schema for the module.
	ModuleSchema() schema.ModuleSchema

	// GetObjectCollection returns the object collection for the given object type. If the object collection
	// does not exist, nil and no error should be returned
	GetObjectCollection(objectType string) (ObjectCollection, error)

	// ObjectCollections iterates over all the object collections in the module. If there is an error getting an object
	// collection, objColl may be nil and err will be non-nil.
	ObjectCollections(f func(value ObjectCollection, err error) bool)

	// NumObjectCollections returns the number of object collections in the module.
	NumObjectCollections() (int, error)
}

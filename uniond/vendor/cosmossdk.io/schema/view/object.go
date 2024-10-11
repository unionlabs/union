package view

import "cosmossdk.io/schema"

// ObjectCollection is the interface for viewing the state of a collection of objects in a module
// represented by StateObjectUpdate's for an ObjectType. ObjectUpdates must not include
// ValueUpdates in the Value field. When ValueUpdates are applied they must be
// converted to individual value or array format depending on the number of fields in
// the value. For collections which retain deletions, StateObjectUpdate's with the Delete
// field set to true should be returned with the latest Value still intact.
type ObjectCollection interface {
	// ObjectType returns the object type for the collection.
	ObjectType() schema.StateObjectType

	// GetObject returns the object update for the given key if it exists. And error should only be returned
	// if there was an error getting the object update. If the object does not exist but there was no error,
	// then found should be false and the error should be nil.
	GetObject(key interface{}) (update schema.StateObjectUpdate, found bool, err error)

	// AllState iterates over the state of the collection by calling the given function with each item in
	// state represented as an object update. If there is an error getting an object update, the error will be
	// non-nil and the object update should be empty.
	AllState(f func(schema.StateObjectUpdate, error) bool)

	// Len returns the number of objects in the collection.
	Len() (int, error)
}

package orm

import storetypes "cosmossdk.io/store/types"

// TableExportable defines the methods to import and export a table.
type TableExportable interface {
	// Export stores all the values in the table in the passed
	// ModelSlicePtr. If the table has an associated sequence, then its
	// current value is returned, otherwise 0 is returned by default.
	Export(store storetypes.KVStore, dest ModelSlicePtr) (uint64, error)

	// Import clears the table and initializes it from the given data
	// interface{}. data should be a slice of structs that implement
	// PrimaryKeyed. The seqValue is optional and only
	// used with tables that have an associated sequence.
	Import(store storetypes.KVStore, data interface{}, seqValue uint64) error
}

package schema

// HasModuleCodec is an interface that modules can implement to provide a ModuleCodec.
// Usually these modules would also implement appmodule.AppModule, but that is not included
// to keep this package free of any dependencies.
type HasModuleCodec interface {
	// ModuleCodec returns a ModuleCodec for the module.
	ModuleCodec() (ModuleCodec, error)
}

// ModuleCodec is a struct that contains the schema and a KVDecoder for a module.
type ModuleCodec struct {
	// Schema is the schema for the module. It is required.
	Schema ModuleSchema

	// KVDecoder is a function that decodes a key-value pair into an StateObjectUpdate.
	// If it is nil, the module doesn't support state decoding directly.
	KVDecoder KVDecoder
}

// KVDecoder is a function that decodes a key-value pair into one or more StateObjectUpdate's.
// If the KV-pair doesn't represent object updates, the function should return nil as the first
// and no error. The error result  should only be non-nil when the decoder expected
// to parse a valid update and was unable to. In the case of an error, the decoder may return
// a non-nil value for the first return value, which can indicate which parts of the update
// were decodable to aid debugging.
type KVDecoder = func(KVPairUpdate) ([]StateObjectUpdate, error)

// KVPairUpdate represents a key-value pair set or delete.
type KVPairUpdate = struct {
	// Key is the key of the key-value pair.
	Key []byte

	// Value is the value of the key-value pair. It should be ignored when Remove is true.
	Value []byte

	// Remove is a flag that indicates that the key-value pair was deleted. If it is false,
	// then it is assumed that this has been a set operation.
	Remove bool
}

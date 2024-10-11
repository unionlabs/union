package registry

// AminoRegistrar is an interface that allow to register concrete types and interfaces with the Amino codec.
type AminoRegistrar interface {
	// RegisterInterface registers an interface and its concrete type with the Amino codec.
	RegisterInterface(interfacePtr any, iopts *AminoInterfaceOptions)

	// RegisterConcrete registers a concrete type with the Amino codec.
	RegisterConcrete(cdcType interface{}, name string)
}

// AminoInterfaceOptions defines options for registering an interface with the Amino codec.
type AminoInterfaceOptions struct {
	Priority           []string // Disamb priority.
	AlwaysDisambiguate bool     // If true, include disamb for all types.
}

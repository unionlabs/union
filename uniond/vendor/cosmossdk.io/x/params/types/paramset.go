package types

type (
	ValueValidatorFn func(value interface{}) error

	// ParamSetPair is used for associating paramsubspace key and field of param
	// structs.
	ParamSetPair struct {
		Key         []byte
		Value       interface{}
		ValidatorFn ValueValidatorFn
	}
)

// NewParamSetPair creates a new ParamSetPair instance.
func NewParamSetPair(key []byte, value interface{}, vfn ValueValidatorFn) ParamSetPair {
	return ParamSetPair{key, value, vfn}
}

// ParamSetPairs Slice of KeyFieldPair
type ParamSetPairs []ParamSetPair

// ParamSet defines an interface for structs containing parameters for a module
type ParamSet interface {
	ParamSetPairs() ParamSetPairs
}

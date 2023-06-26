package ssz

// Marshaler is the interface implemented by types that can marshal themselves into valid SZZ.
type Marshaler interface {
	MarshalSSZTo(dst []byte) ([]byte, error)
	MarshalSSZ() ([]byte, error)
	SizeSSZ() int
}

// Unmarshaler is the interface implemented by types that can unmarshal a SSZ description of themselves
type Unmarshaler interface {
	UnmarshalSSZ(buf []byte) error
}

type HashRoot interface {
	HashTreeRoot() ([32]byte, error)
	HashTreeRootWith(hh *Hasher) error
}

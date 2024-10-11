package types

type PubKey interface {
	Address() Address
	Bytes() []byte
	VerifySignature(msg []byte, sig []byte) bool
	Equals(other PubKey) bool
	Type() string
}

// PrivKey interface with generics
type PrivKey[T PubKey] interface {
	Bytes() []byte
	Sign(msg []byte) ([]byte, error)
	PubKey() T
	Equals(other PrivKey[T]) bool
	Type() string
}

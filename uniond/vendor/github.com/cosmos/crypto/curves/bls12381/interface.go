package blst

type PubKey interface {
	Marshal() []byte
	Copy() PubKey
	Equals(p2 PubKey) bool
}

// SignatureI represents a BLS signature.
type SignatureI interface {
	Verify(pubKey PubKey, msg []byte) bool
	Marshal() []byte
	Copy() SignatureI
}

// SecretKey represents a BLS secret or private key.
type SecretKey interface {
	PublicKey() PubKey
	Sign(msg []byte) SignatureI
	Marshal() []byte
}

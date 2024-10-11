package blst

import (
	"errors"
	"fmt"
)

var dst = []byte("BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_")

// Signature used in the BLS signature scheme.
type Signature struct {
	s *blstSignature
}

// Marshal a signature into a LittleEndian byte slice.
func (s *Signature) Marshal() []byte {
	return s.s.Compress()
}

// Copy returns a full deep copy of a signature.
func (s *Signature) Copy() SignatureI {
	sign := *s.s
	return &Signature{s: &sign}
}

func (s *Signature) Verify(pubKey PubKey, msg []byte) bool {
	// Signature and PKs are assumed to have been validated upon decompression!
	return s.s.Verify(false, pubKey.(*PublicKey).p, false, msg, dst)
}

// VerifySignature verifies a single signature using public key and message.
func VerifySignature(sig []byte, msg [32]byte, pubKey PubKey) (bool, error) {
	rSig, err := SignatureFromBytes(sig)
	if err != nil {
		return false, err
	}
	return rSig.Verify(pubKey, msg[:]), nil
}

// signatureFromBytesNoValidation creates a BLS signature from a LittleEndian
// byte slice. It does not validate that the signature is in the BLS group
func signatureFromBytesNoValidation(sig []byte) (*blstSignature, error) {
	if len(sig) != SignatureLength {
		return nil, fmt.Errorf("signature must be %d bytes", SignatureLength)
	}
	signature := new(blstSignature).Uncompress(sig)
	if signature == nil {
		return nil, errors.New("could not unmarshal bytes into signature")
	}
	return signature, nil
}

// SignatureFromBytesNoValidation creates a BLS signature from a LittleEndian
// byte slice. It does not validate that the signature is in the BLS group
func SignatureFromBytesNoValidation(sig []byte) (SignatureI, error) {
	signature, err := signatureFromBytesNoValidation(sig)
	if err != nil {
		return nil, fmt.Errorf("could not create signature from byte slice: %w", err)
	}
	return &Signature{s: signature}, nil
}

// SignatureFromBytes creates a BLS signature from a LittleEndian byte slice.
func SignatureFromBytes(sig []byte) (SignatureI, error) {
	signature, err := signatureFromBytesNoValidation(sig)
	if err != nil {
		return nil, fmt.Errorf("could not create signature from byte slice: %w", err)
	}
	// Group check signature. Do not check for infinity since an aggregated signature
	// could be infinite.
	if !signature.SigValidate(false) {
		return nil, errors.New("signature not in group")
	}
	return &Signature{s: signature}, nil
}

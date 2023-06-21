//go:build ((linux && amd64) || (linux && arm64) || (darwin && amd64) || (darwin && arm64) || (windows && amd64)) && !blst_disabled

package blst

import (
	"bytes"
	"fmt"
	"sync"

	"github.com/pkg/errors"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls/common"
	"github.com/prysmaticlabs/prysm/v4/crypto/rand"
	blst "github.com/supranational/blst/bindings/go"
)

var dst = []byte("BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_")

const scalarBytes = 32
const randBitsEntropy = 64

// Signature used in the BLS signature scheme.
type Signature struct {
	s *blstSignature
}

// signatureFromBytesNoValidation creates a BLS signature from a LittleEndian
// byte slice. It does not validate that the signature is in the BLS group
func signatureFromBytesNoValidation(sig []byte) (*blstSignature, error) {
	if len(sig) != fieldparams.BLSSignatureLength {
		return nil, fmt.Errorf("signature must be %d bytes", fieldparams.BLSSignatureLength)
	}
	signature := new(blstSignature).Uncompress(sig)
	if signature == nil {
		return nil, errors.New("could not unmarshal bytes into signature")
	}
	return signature, nil
}

// SignatureFromBytesNoValidation creates a BLS signature from a LittleEndian
// byte slice. It does not validate that the signature is in the BLS group
func SignatureFromBytesNoValidation(sig []byte) (common.Signature, error) {
	signature, err := signatureFromBytesNoValidation(sig)
	if err != nil {
		return nil, errors.Wrap(err, "could not create signature from byte slice")
	}
	return &Signature{s: signature}, nil
}

// SignatureFromBytes creates a BLS signature from a LittleEndian byte slice.
func SignatureFromBytes(sig []byte) (common.Signature, error) {
	signature, err := signatureFromBytesNoValidation(sig)
	if err != nil {
		return nil, errors.Wrap(err, "could not create signature from byte slice")
	}
	// Group check signature. Do not check for infinity since an aggregated signature
	// could be infinite.
	if !signature.SigValidate(false) {
		return nil, errors.New("signature not in group")
	}
	return &Signature{s: signature}, nil
}

// AggregateCompressedSignatures converts a list of compressed signatures into a single, aggregated sig.
func AggregateCompressedSignatures(multiSigs [][]byte) (common.Signature, error) {
	signature := new(blstAggregateSignature)
	valid := signature.AggregateCompressed(multiSigs, true)
	if !valid {
		return nil, errors.New("provided signatures fail the group check and cannot be compressed")
	}
	return &Signature{s: signature.ToAffine()}, nil
}

// MultipleSignaturesFromBytes creates a group of BLS signatures from a LittleEndian 2d-byte slice.
func MultipleSignaturesFromBytes(multiSigs [][]byte) ([]common.Signature, error) {
	if len(multiSigs) == 0 {
		return nil, fmt.Errorf("0 signatures provided to the method")
	}
	for _, s := range multiSigs {
		if len(s) != fieldparams.BLSSignatureLength {
			return nil, fmt.Errorf("signature must be %d bytes", fieldparams.BLSSignatureLength)
		}
	}
	multiSignatures := new(blstSignature).BatchUncompress(multiSigs)
	if len(multiSignatures) == 0 {
		return nil, errors.New("could not unmarshal bytes into signature")
	}
	if len(multiSignatures) != len(multiSigs) {
		return nil, errors.Errorf("wanted %d decompressed signatures but got %d", len(multiSigs), len(multiSignatures))
	}
	wrappedSigs := make([]common.Signature, len(multiSignatures))
	for i, signature := range multiSignatures {
		// Group check signature. Do not check for infinity since an aggregated signature
		// could be infinite.
		if !signature.SigValidate(false) {
			return nil, errors.New("signature not in group")
		}
		copiedSig := signature
		wrappedSigs[i] = &Signature{s: copiedSig}
	}
	return wrappedSigs, nil
}

// Verify a bls signature given a public key, a message.
//
// In IETF draft BLS specification:
// Verify(PK, message, signature) -> VALID or INVALID: a verification
//
//	algorithm that outputs VALID if signature is a valid signature of
//	message under public key PK, and INVALID otherwise.
//
// In the Ethereum proof of stake specification:
// def Verify(PK: BLSPubkey, message: Bytes, signature: BLSSignature) -> bool
func (s *Signature) Verify(pubKey common.PublicKey, msg []byte) bool {
	// Signature and PKs are assumed to have been validated upon decompression!
	return s.s.Verify(false, pubKey.(*PublicKey).p, false, msg, dst)
}

// AggregateVerify verifies each public key against its respective message. This is vulnerable to
// rogue public-key attack. Each user must provide a proof-of-knowledge of the public key.
//
// Note: The msgs must be distinct. For maximum performance, this method does not ensure distinct
// messages.
//
// In IETF draft BLS specification:
// AggregateVerify((PK_1, message_1), ..., (PK_n, message_n),
//
//	signature) -> VALID or INVALID: an aggregate verification
//	algorithm that outputs VALID if signature is a valid aggregated
//	signature for a collection of public keys and messages, and
//	outputs INVALID otherwise.
//
// In the Ethereum proof of stake specification:
// def AggregateVerify(pairs: Sequence[PK: BLSPubkey, message: Bytes], signature: BLSSignature) -> bool
//
// Deprecated: Use FastAggregateVerify or use this method in spectests only.
func (s *Signature) AggregateVerify(pubKeys []common.PublicKey, msgs [][32]byte) bool {
	size := len(pubKeys)
	if size == 0 {
		return false
	}
	if size != len(msgs) {
		return false
	}
	msgSlices := make([][]byte, len(msgs))
	rawKeys := make([]*blstPublicKey, len(msgs))
	for i := 0; i < size; i++ {
		msgSlices[i] = msgs[i][:]
		rawKeys[i] = pubKeys[i].(*PublicKey).p
	}
	// Signature and PKs are assumed to have been validated upon decompression!
	return s.s.AggregateVerify(false, rawKeys, false, msgSlices, dst)
}

// FastAggregateVerify verifies all the provided public keys with their aggregated signature.
//
// In IETF draft BLS specification:
// FastAggregateVerify(PK_1, ..., PK_n, message, signature) -> VALID
//
//	or INVALID: a verification algorithm for the aggregate of multiple
//	signatures on the same message.  This function is faster than
//	AggregateVerify.
//
// In the Ethereum proof of stake specification:
// def FastAggregateVerify(PKs: Sequence[BLSPubkey], message: Bytes, signature: BLSSignature) -> bool
func (s *Signature) FastAggregateVerify(pubKeys []common.PublicKey, msg [32]byte) bool {
	if len(pubKeys) == 0 {
		return false
	}
	rawKeys := make([]*blstPublicKey, len(pubKeys))
	for i := 0; i < len(pubKeys); i++ {
		rawKeys[i] = pubKeys[i].(*PublicKey).p
	}
	return s.s.FastAggregateVerify(true, rawKeys, msg[:], dst)
}

// Eth2FastAggregateVerify implements a wrapper on top of bls's FastAggregateVerify. It accepts G2_POINT_AT_INFINITY signature
// when pubkeys empty.
//
// Spec code:
// def eth2_fast_aggregate_verify(pubkeys: Sequence[BLSPubkey], message: Bytes32, signature: BLSSignature) -> bool:
//
//	"""
//	Wrapper to ``bls.FastAggregateVerify`` accepting the ``G2_POINT_AT_INFINITY`` signature when ``pubkeys`` is empty.
//	"""
//	if len(pubkeys) == 0 and signature == G2_POINT_AT_INFINITY:
//	    return True
//	return bls.FastAggregateVerify(pubkeys, message, signature)
func (s *Signature) Eth2FastAggregateVerify(pubKeys []common.PublicKey, msg [32]byte) bool {
	if len(pubKeys) == 0 && bytes.Equal(s.Marshal(), common.InfiniteSignature[:]) {
		return true
	}
	return s.FastAggregateVerify(pubKeys, msg)
}

// NewAggregateSignature creates a blank aggregate signature.
func NewAggregateSignature() common.Signature {
	sig := blst.HashToG2([]byte{'m', 'o', 'c', 'k'}, dst).ToAffine()
	return &Signature{s: sig}
}

// AggregateSignatures converts a list of signatures into a single, aggregated sig.
func AggregateSignatures(sigs []common.Signature) common.Signature {
	if len(sigs) == 0 {
		return nil
	}

	rawSigs := make([]*blstSignature, len(sigs))
	for i := 0; i < len(sigs); i++ {
		rawSigs[i] = sigs[i].(*Signature).s
	}

	// Signature and PKs are assumed to have been validated upon decompression!
	signature := new(blstAggregateSignature)
	signature.Aggregate(rawSigs, false)
	return &Signature{s: signature.ToAffine()}
}

// VerifySignature verifies a single signature using public key and message.
func VerifySignature(sig []byte, msg [32]byte, pubKey common.PublicKey) (bool, error) {
	rSig, err := SignatureFromBytes(sig)
	if err != nil {
		return false, err
	}
	return rSig.Verify(pubKey, msg[:]), nil
}

// VerifyMultipleSignatures verifies a non-singular set of signatures and its respective pubkeys and messages.
// This method provides a safe way to verify multiple signatures at once. We pick a number randomly from 1 to max
// uint64 and then multiply the signature by it. We continue doing this for all signatures and its respective pubkeys.
// S* = S_1 * r_1 + S_2 * r_2 + ... + S_n * r_n
// P'_{i,j} = P_{i,j} * r_i
// e(S*, G) = \prod_{i=1}^n \prod_{j=1}^{m_i} e(P'_{i,j}, M_{i,j})
// Using this we can verify multiple signatures safely.
func VerifyMultipleSignatures(sigs [][]byte, msgs [][32]byte, pubKeys []common.PublicKey) (bool, error) {
	if len(sigs) == 0 || len(pubKeys) == 0 {
		return false, nil
	}
	rawSigs := new(blstSignature).BatchUncompress(sigs)

	length := len(sigs)
	if length != len(pubKeys) || length != len(msgs) {
		return false, errors.Errorf("provided signatures, pubkeys and messages have differing lengths. S: %d, P: %d,M %d",
			length, len(pubKeys), len(msgs))
	}
	mulP1Aff := make([]*blstPublicKey, length)
	rawMsgs := make([]blst.Message, length)

	for i := 0; i < length; i++ {
		mulP1Aff[i] = pubKeys[i].(*PublicKey).p
		rawMsgs[i] = msgs[i][:]
	}
	// Secure source of RNG
	randGen := rand.NewGenerator()
	randLock := new(sync.Mutex)

	randFunc := func(scalar *blst.Scalar) {
		var rbytes [scalarBytes]byte
		randLock.Lock()
		randGen.Read(rbytes[:]) // #nosec G104 -- Error will always be nil in `read` in math/rand
		randLock.Unlock()
		// Protect against the generator returning 0. Since the scalar value is
		// derived from a big endian byte slice, we take the last byte.
		rbytes[len(rbytes)-1] |= 0x01
		scalar.FromBEndian(rbytes[:])
	}
	dummySig := new(blstSignature)

	// Validate signatures since we uncompress them here. Public keys should already be validated.
	return dummySig.MultipleAggregateVerify(rawSigs, true, mulP1Aff, false, rawMsgs, dst, randFunc, randBitsEntropy), nil
}

// Marshal a signature into a LittleEndian byte slice.
func (s *Signature) Marshal() []byte {
	return s.s.Compress()
}

// Copy returns a full deep copy of a signature.
func (s *Signature) Copy() common.Signature {
	sign := *s.s
	return &Signature{s: &sign}
}

// VerifyCompressed verifies that the compressed signature and pubkey
// are valid from the message provided.
func VerifyCompressed(signature, pub, msg []byte) bool {
	// Validate signature and PKs since we will uncompress them here
	return new(blstSignature).VerifyCompressed(signature, true, pub, true, msg, dst)
}

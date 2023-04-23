package schnorrkel

import (
	"crypto/rand"
	"errors"

	"github.com/gtank/merlin"
	r255 "github.com/gtank/ristretto255"
)

const ChainCodeLength = 32

// DerivableKey implements DeriveKey
type DerivableKey interface {
	Encode() [32]byte
	Decode([32]byte) error
	DeriveKey(*merlin.Transcript, [ChainCodeLength]byte) (*ExtendedKey, error)
}

// ExtendedKey consists of a DerivableKey which can be a schnorrkel public or private key
// as well as chain code
type ExtendedKey struct {
	key       DerivableKey
	chaincode [ChainCodeLength]byte
}

// NewExtendedKey creates an ExtendedKey given a DerivableKey and chain code
func NewExtendedKey(k DerivableKey, cc [ChainCodeLength]byte) *ExtendedKey {
	return &ExtendedKey{
		key:       k,
		chaincode: cc,
	}
}

// Key returns the schnorrkel key underlying the ExtendedKey
func (ek *ExtendedKey) Key() DerivableKey {
	return ek.key
}

// ChainCode returns the chain code underlying the ExtendedKey
func (ek *ExtendedKey) ChainCode() [ChainCodeLength]byte {
	return ek.chaincode
}

// Secret returns the SecretKey underlying the ExtendedKey
// if it's not a secret key, it returns an error
func (ek *ExtendedKey) Secret() (*SecretKey, error) {
	if priv, ok := ek.key.(*SecretKey); ok {
		return priv, nil
	}

	return nil, errors.New("extended key is not a secret key")
}

// Public returns the PublicKey underlying the ExtendedKey
func (ek *ExtendedKey) Public() (*PublicKey, error) {
	if pub, ok := ek.key.(*PublicKey); ok {
		return pub, nil
	}

	if priv, ok := ek.key.(*SecretKey); ok {
		return priv.Public()
	}

	return nil, errors.New("extended key is not a valid public or private key")
}

// DeriveKey derives an extended key from an extended key
func (ek *ExtendedKey) DeriveKey(t *merlin.Transcript) (*ExtendedKey, error) {
	return ek.key.DeriveKey(t, ek.chaincode)
}

// DeriveKeySimple derives a subkey identified by byte array i and chain code.
func DeriveKeySimple(key DerivableKey, i []byte, cc [ChainCodeLength]byte) (*ExtendedKey, error) {
	t := merlin.NewTranscript("SchnorrRistrettoHDKD")
	t.AppendMessage([]byte("sign-bytes"), i)
	return key.DeriveKey(t, cc)
}

// DeriveKey derives a new secret key and chain code from an existing secret key and chain code
func (sk *SecretKey) DeriveKey(t *merlin.Transcript, cc [ChainCodeLength]byte) (*ExtendedKey, error) {
	pub, err := sk.Public()
	if err != nil {
		return nil, err
	}

	sc, dcc := pub.DeriveScalarAndChaincode(t, cc)

	// TODO: need transcript RNG to match rust-schnorrkel
	// see: https://github.com/w3f/schnorrkel/blob/798ab3e0813aa478b520c5cf6dc6e02fd4e07f0a/src/derive.rs#L186
	nonce := [32]byte{}
	_, err = rand.Read(nonce[:])
	if err != nil {
		return nil, err
	}

	dsk, err := ScalarFromBytes(sk.key)
	if err != nil {
		return nil, err
	}

	dsk.Add(dsk, sc)

	dskBytes := [32]byte{}
	copy(dskBytes[:], dsk.Encode([]byte{}))

	skNew := &SecretKey{
		key:   dskBytes,
		nonce: nonce,
	}

	return &ExtendedKey{
		key:       skNew,
		chaincode: dcc,
	}, nil
}

func (pk *PublicKey) DeriveKey(t *merlin.Transcript, cc [ChainCodeLength]byte) (*ExtendedKey, error) {
	sc, dcc := pk.DeriveScalarAndChaincode(t, cc)

	// derivedPk = pk + (sc * g)
	p1 := r255.NewElement().ScalarBaseMult(sc)
	p2 := r255.NewElement()
	p2.Add(pk.key, p1)

	pub := &PublicKey{
		key: p2,
	}

	return &ExtendedKey{
		key:       pub,
		chaincode: dcc,
	}, nil
}

// DeriveScalarAndChaincode derives a new scalar and chain code from an existing public key and chain code
func (pk *PublicKey) DeriveScalarAndChaincode(t *merlin.Transcript, cc [ChainCodeLength]byte) (*r255.Scalar, [ChainCodeLength]byte) {
	t.AppendMessage([]byte("chain-code"), cc[:])
	pkenc := pk.Encode()
	t.AppendMessage([]byte("public-key"), pkenc[:])

	scBytes := t.ExtractBytes([]byte("HDKD-scalar"), 64)
	sc := r255.NewScalar()
	sc.FromUniformBytes(scBytes)

	ccBytes := t.ExtractBytes([]byte("HDKD-chaincode"), ChainCodeLength)
	ccRes := [ChainCodeLength]byte{}
	copy(ccRes[:], ccBytes)
	return sc, ccRes
}

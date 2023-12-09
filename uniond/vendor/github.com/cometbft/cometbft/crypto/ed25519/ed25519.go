package ed25519

import (
	"bytes"
	"crypto/subtle"
	"errors"
	"fmt"
	"io"

	"github.com/oasisprotocol/curve25519-voi/primitives/ed25519"
	"github.com/oasisprotocol/curve25519-voi/primitives/ed25519/extra/cache"

	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/tmhash"
	cmtjson "github.com/cometbft/cometbft/libs/json"
)

//-------------------------------------

var (
	_ crypto.PrivKey       = PrivKey{}
	_ crypto.BatchVerifier = &BatchVerifier{}

	// curve25519-voi's Ed25519 implementation supports configurable
	// verification behavior, and CometBFT uses the ZIP-215 verification
	// semantics.
	verifyOptions = &ed25519.Options{
		Verify: ed25519.VerifyOptionsZIP_215,
	}

	cachingVerifier = cache.NewVerifier(cache.NewLRUCache(cacheSize))
)

const (
	PrivKeyName = "tendermint/PrivKeyEd25519"
	PubKeyName  = "tendermint/PubKeyEd25519"
	// PubKeySize is is the size, in bytes, of public keys as used in this package.
	PubKeySize = 32
	// PrivateKeySize is the size, in bytes, of private keys as used in this package.
	PrivateKeySize = 64
	// Size of an Edwards25519 signature. Namely the size of a compressed
	// Edwards25519 point, and a field element. Both of which are 32 bytes.
	SignatureSize = 64
	// SeedSize is the size, in bytes, of private key seeds. These are the
	// private key representations used by RFC 8032.
	SeedSize = 32

	KeyType = "ed25519"

	// cacheSize is the number of public keys that will be cached in
	// an expanded format for repeated signature verification.
	//
	// TODO/perf: Either this should exclude single verification, or be
	// tuned to `> validatorSize + maxTxnsPerBlock` to avoid cache
	// thrashing.
	cacheSize = 4096
)

func init() {
	cmtjson.RegisterType(PubKey{}, PubKeyName)
	cmtjson.RegisterType(PrivKey{}, PrivKeyName)
}

// PrivKey implements crypto.PrivKey.
type PrivKey []byte

// Bytes returns the privkey byte format.
func (privKey PrivKey) Bytes() []byte {
	return []byte(privKey)
}

// Sign produces a signature on the provided message.
// This assumes the privkey is wellformed in the golang format.
// The first 32 bytes should be random,
// corresponding to the normal ed25519 private key.
// The latter 32 bytes should be the compressed public key.
// If these conditions aren't met, Sign will panic or produce an
// incorrect signature.
func (privKey PrivKey) Sign(msg []byte) ([]byte, error) {
	signatureBytes := ed25519.Sign(ed25519.PrivateKey(privKey), msg)
	return signatureBytes, nil
}

// PubKey gets the corresponding public key from the private key.
//
// Panics if the private key is not initialized.
func (privKey PrivKey) PubKey() crypto.PubKey {
	// If the latter 32 bytes of the privkey are all zero, privkey is not
	// initialized.
	initialized := false
	for _, v := range privKey[32:] {
		if v != 0 {
			initialized = true
			break
		}
	}

	if !initialized {
		panic("Expected ed25519 PrivKey to include concatenated pubkey bytes")
	}

	pubkeyBytes := make([]byte, PubKeySize)
	copy(pubkeyBytes, privKey[32:])
	return PubKey(pubkeyBytes)
}

// Equals - you probably don't need to use this.
// Runs in constant time based on length of the keys.
func (privKey PrivKey) Equals(other crypto.PrivKey) bool {
	if otherEd, ok := other.(PrivKey); ok {
		return subtle.ConstantTimeCompare(privKey[:], otherEd[:]) == 1
	}

	return false
}

func (privKey PrivKey) Type() string {
	return KeyType
}

// GenPrivKey generates a new ed25519 private key.
// It uses OS randomness in conjunction with the current global random seed
// in cometbft/libs/rand to generate the private key.
func GenPrivKey() PrivKey {
	return genPrivKey(crypto.CReader())
}

// genPrivKey generates a new ed25519 private key using the provided reader.
func genPrivKey(rand io.Reader) PrivKey {
	_, priv, err := ed25519.GenerateKey(rand)
	if err != nil {
		panic(err)
	}

	return PrivKey(priv)
}

// GenPrivKeyFromSecret hashes the secret with SHA2, and uses
// that 32 byte output to create the private key.
// NOTE: secret should be the output of a KDF like bcrypt,
// if it's derived from user input.
func GenPrivKeyFromSecret(secret []byte) PrivKey {
	seed := crypto.Sha256(secret) // Not Ripemd160 because we want 32 bytes.

	return PrivKey(ed25519.NewKeyFromSeed(seed))
}

//-------------------------------------

var _ crypto.PubKey = PubKey{}

// PubKey implements crypto.PubKey for the Ed25519 signature scheme.
type PubKey []byte

// Address is the SHA256-20 of the raw pubkey bytes.
func (pubKey PubKey) Address() crypto.Address {
	if len(pubKey) != PubKeySize {
		panic("pubkey is incorrect size")
	}
	return crypto.Address(tmhash.SumTruncated(pubKey))
}

// Bytes returns the PubKey byte format.
func (pubKey PubKey) Bytes() []byte {
	return []byte(pubKey)
}

func (pubKey PubKey) VerifySignature(msg []byte, sig []byte) bool {
	// make sure we use the same algorithm to sign
	if len(sig) != SignatureSize {
		return false
	}

	return cachingVerifier.VerifyWithOptions(ed25519.PublicKey(pubKey), msg, sig, verifyOptions)
}

func (pubKey PubKey) String() string {
	return fmt.Sprintf("PubKeyEd25519{%X}", []byte(pubKey))
}

func (pubKey PubKey) Type() string {
	return KeyType
}

func (pubKey PubKey) Equals(other crypto.PubKey) bool {
	if otherEd, ok := other.(PubKey); ok {
		return bytes.Equal(pubKey[:], otherEd[:])
	}

	return false
}

//-------------------------------------

// BatchVerifier implements batch verification for ed25519.
type BatchVerifier struct {
	*ed25519.BatchVerifier
}

func NewBatchVerifier() crypto.BatchVerifier {
	return &BatchVerifier{ed25519.NewBatchVerifier()}
}

func (b *BatchVerifier) Add(key crypto.PubKey, msg, signature []byte) error {
	pkEd, ok := key.(PubKey)
	if !ok {
		return fmt.Errorf("pubkey is not Ed25519")
	}

	pkBytes := pkEd.Bytes()

	if l := len(pkBytes); l != PubKeySize {
		return fmt.Errorf("pubkey size is incorrect; expected: %d, got %d", PubKeySize, l)
	}

	// check that the signature is the correct length
	if len(signature) != SignatureSize {
		return errors.New("invalid signature")
	}

	cachingVerifier.AddWithOptions(b.BatchVerifier, ed25519.PublicKey(pkBytes), msg, signature, verifyOptions)

	return nil
}

func (b *BatchVerifier) Verify() (bool, []bool) {
	return b.BatchVerifier.Verify(crypto.CReader())
}

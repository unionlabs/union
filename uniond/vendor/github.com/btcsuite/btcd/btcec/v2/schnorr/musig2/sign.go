// Copyright 2013-2022 The btcsuite developers

package musig2

import (
	"bytes"
	"fmt"
	"io"

	secp "github.com/decred/dcrd/dcrec/secp256k1/v4"

	"github.com/btcsuite/btcd/btcec/v2"
	"github.com/btcsuite/btcd/btcec/v2/schnorr"
	"github.com/btcsuite/btcd/chaincfg/chainhash"
)

var (
	// NonceBlindTag is that tag used to construct the value b, which
	// blinds the second public nonce of each party.
	NonceBlindTag = []byte("MuSig/noncecoef")

	// ChallengeHashTag is the tag used to construct the challenge hash
	ChallengeHashTag = []byte("BIP0340/challenge")

	// ErrNoncePointAtInfinity is returned if during signing, the fully
	// combined public nonce is the point at infinity.
	ErrNoncePointAtInfinity = fmt.Errorf("signing nonce is the infinity " +
		"point")

	// ErrPrivKeyZero is returned when the private key for signing is
	// actually zero.
	ErrPrivKeyZero = fmt.Errorf("priv key is zero")

	// ErrPartialSigInvalid is returned when a partial is found to be
	// invalid.
	ErrPartialSigInvalid = fmt.Errorf("partial signature is invalid")

	// ErrSecretNonceZero is returned when a secret nonce is passed in a
	// zero.
	ErrSecretNonceZero = fmt.Errorf("secret nonce is blank")

	// ErrSecNoncePubkey is returned when the signing key does not match the
	// sec nonce pubkey
	ErrSecNoncePubkey = fmt.Errorf("public key does not match secnonce")

	// ErrPubkeyNotIncluded is returned when the signers pubkey is not included
	// in the list of pubkeys.
	ErrPubkeyNotIncluded = fmt.Errorf("signer's pubkey must be included" +
		" in the list of pubkeys")
)

// infinityPoint is the jacobian representation of the point at infinity.
var infinityPoint btcec.JacobianPoint

// PartialSignature reprints a partial (s-only) musig2 multi-signature. This
// isn't a valid schnorr signature by itself, as it needs to be aggregated
// along with the other partial signatures to be completed.
type PartialSignature struct {
	S *btcec.ModNScalar

	R *btcec.PublicKey
}

// NewPartialSignature returns a new instances of the partial sig struct.
func NewPartialSignature(s *btcec.ModNScalar,
	r *btcec.PublicKey) PartialSignature {

	return PartialSignature{
		S: s,
		R: r,
	}
}

// Encode writes a serialized version of the partial signature to the passed
// io.Writer
func (p *PartialSignature) Encode(w io.Writer) error {
	var sBytes [32]byte
	p.S.PutBytes(&sBytes)

	if _, err := w.Write(sBytes[:]); err != nil {
		return err
	}

	return nil
}

// Decode attempts to parse a serialized PartialSignature stored in the passed
// io reader.
func (p *PartialSignature) Decode(r io.Reader) error {
	p.S = new(btcec.ModNScalar)

	var sBytes [32]byte
	if _, err := io.ReadFull(r, sBytes[:]); err != nil {
		return nil
	}

	overflows := p.S.SetBytes(&sBytes)
	if overflows == 1 {
		return ErrPartialSigInvalid
	}

	return nil
}

// SignOption is a functional option argument that allows callers to modify the
// way we generate musig2 schnorr signatures.
type SignOption func(*signOptions)

// signOptions houses the set of functional options that can be used to modify
// the method used to generate the musig2 partial signature.
type signOptions struct {
	// fastSign determines if we'll skip the check at the end of the
	// routine where we attempt to verify the produced signature.
	fastSign bool

	// sortKeys determines if the set of keys should be sorted before doing
	// key aggregation.
	sortKeys bool

	// tweaks specifies a series of tweaks to be applied to the aggregated
	// public key, which also partially carries over into the signing
	// process.
	tweaks []KeyTweakDesc

	// taprootTweak specifies a taproot specific tweak.  of the tweaks
	// specified above. Normally we'd just apply the raw 32 byte tweak, but
	// for taproot, we first need to compute the aggregated key before
	// tweaking, and then use it as the internal key. This is required as
	// the taproot tweak also commits to the public key, which in this case
	// is the aggregated key before the tweak.
	taprootTweak []byte

	// bip86Tweak specifies that the taproot tweak should be done in a BIP
	// 86 style, where we don't expect an actual tweak and instead just
	// commit to the public key itself.
	bip86Tweak bool
}

// defaultSignOptions returns the default set of signing operations.
func defaultSignOptions() *signOptions {
	return &signOptions{}
}

// WithFastSign forces signing to skip the extra verification step at the end.
// Performance sensitive applications may opt to use this option to speed up
// the signing operation.
func WithFastSign() SignOption {
	return func(o *signOptions) {
		o.fastSign = true
	}
}

// WithSortedKeys determines if the set of signing public keys are to be sorted
// or not before doing key aggregation.
func WithSortedKeys() SignOption {
	return func(o *signOptions) {
		o.sortKeys = true
	}
}

// WithTweaks determines if the aggregated public key used should apply a
// series of tweaks before key aggregation.
func WithTweaks(tweaks ...KeyTweakDesc) SignOption {
	return func(o *signOptions) {
		o.tweaks = tweaks
	}
}

// WithTaprootSignTweak allows a caller to specify a tweak that should be used
// in a bip 340 manner when signing. This differs from WithTweaks as the tweak
// will be assumed to always be x-only and the intermediate aggregate key
// before tweaking will be used to generate part of the tweak (as the taproot
// tweak also commits to the internal key).
//
// This option should be used in the taproot context to create a valid
// signature for the keypath spend for taproot, when the output key is actually
// committing to a script path, or some other data.
func WithTaprootSignTweak(scriptRoot []byte) SignOption {
	return func(o *signOptions) {
		o.taprootTweak = scriptRoot
	}
}

// WithBip86SignTweak allows a caller to specify a tweak that should be used in
// a bip 340 manner when signing, factoring in BIP 86 as well. This differs
// from WithTaprootSignTweak as no true script root will be committed to,
// instead we just commit to the internal key.
//
// This option should be used in the taproot context to create a valid
// signature for the keypath spend for taproot, when the output key was
// generated using BIP 86.
func WithBip86SignTweak() SignOption {
	return func(o *signOptions) {
		o.bip86Tweak = true
	}
}

// computeSigningNonce calculates the final nonce used for signing. This will
// be the R value used in the final signature.
func computeSigningNonce(combinedNonce [PubNonceSize]byte,
	combinedKey *btcec.PublicKey, msg [32]byte) (
	*btcec.JacobianPoint, *btcec.ModNScalar, error) {

	// Next we'll compute the value b, that blinds our second public
	// nonce:
	//  * b = h(tag=NonceBlindTag, combinedNonce || combinedKey || m).
	var (
		nonceMsgBuf  bytes.Buffer
		nonceBlinder btcec.ModNScalar
	)
	nonceMsgBuf.Write(combinedNonce[:])
	nonceMsgBuf.Write(schnorr.SerializePubKey(combinedKey))
	nonceMsgBuf.Write(msg[:])
	nonceBlindHash := chainhash.TaggedHash(
		NonceBlindTag, nonceMsgBuf.Bytes(),
	)
	nonceBlinder.SetByteSlice(nonceBlindHash[:])

	// Next, we'll parse the public nonces into R1 and R2.
	r1J, err := btcec.ParseJacobian(
		combinedNonce[:btcec.PubKeyBytesLenCompressed],
	)
	if err != nil {
		return nil, nil, err
	}
	r2J, err := btcec.ParseJacobian(
		combinedNonce[btcec.PubKeyBytesLenCompressed:],
	)
	if err != nil {
		return nil, nil, err
	}

	// With our nonce blinding value, we'll now combine both the public
	// nonces, using the blinding factor to tweak the second nonce:
	//  * R = R_1 + b*R_2
	var nonce btcec.JacobianPoint
	btcec.ScalarMultNonConst(&nonceBlinder, &r2J, &r2J)
	btcec.AddNonConst(&r1J, &r2J, &nonce)

	// If the combined nonce is the point at infinity, we'll use the
	// generator point instead.
	if nonce == infinityPoint {
		G := btcec.Generator()
		G.AsJacobian(&nonce)
	}

	return &nonce, &nonceBlinder, nil
}

// Sign generates a musig2 partial signature given the passed key set, secret
// nonce, public nonce, and private keys. This method returns an error if the
// generated nonces are either too large, or end up mapping to the point at
// infinity.
func Sign(secNonce [SecNonceSize]byte, privKey *btcec.PrivateKey,
	combinedNonce [PubNonceSize]byte, pubKeys []*btcec.PublicKey,
	msg [32]byte, signOpts ...SignOption) (*PartialSignature, error) {

	// First, parse the set of optional signing options.
	opts := defaultSignOptions()
	for _, option := range signOpts {
		option(opts)
	}

	// Check that our signing key belongs to the secNonce
	if !bytes.Equal(secNonce[btcec.PrivKeyBytesLen*2:],
		privKey.PubKey().SerializeCompressed()) {

		return nil, ErrSecNoncePubkey
	}

	// Check that the key set contains the public key to our private key.
	var containsPrivKey bool
	for _, pk := range pubKeys {
		if privKey.PubKey().IsEqual(pk) {
			containsPrivKey = true
		}
	}

	if !containsPrivKey {
		return nil, ErrPubkeyNotIncluded
	}

	// Compute the hash of all the keys here as we'll need it do aggregate
	// the keys and also at the final step of signing.
	keysHash := keyHashFingerprint(pubKeys, opts.sortKeys)
	uniqueKeyIndex := secondUniqueKeyIndex(pubKeys, opts.sortKeys)

	keyAggOpts := []KeyAggOption{
		WithKeysHash(keysHash), WithUniqueKeyIndex(uniqueKeyIndex),
	}
	switch {
	case opts.bip86Tweak:
		keyAggOpts = append(
			keyAggOpts, WithBIP86KeyTweak(),
		)
	case opts.taprootTweak != nil:
		keyAggOpts = append(
			keyAggOpts, WithTaprootKeyTweak(opts.taprootTweak),
		)
	case len(opts.tweaks) != 0:
		keyAggOpts = append(keyAggOpts, WithKeyTweaks(opts.tweaks...))
	}

	// Next we'll construct the aggregated public key based on the set of
	// signers.
	combinedKey, parityAcc, _, err := AggregateKeys(
		pubKeys, opts.sortKeys, keyAggOpts...,
	)
	if err != nil {
		return nil, err
	}

	// We'll now combine both the public nonces, using the blinding factor
	// to tweak the second nonce:
	//  * R = R_1 + b*R_2
	nonce, nonceBlinder, err := computeSigningNonce(
		combinedNonce, combinedKey.FinalKey, msg,
	)
	if err != nil {
		return nil, err
	}

	// Next we'll parse out our two secret nonces, which we'll be using in
	// the core signing process below.
	var k1, k2 btcec.ModNScalar
	k1.SetByteSlice(secNonce[:btcec.PrivKeyBytesLen])
	k2.SetByteSlice(secNonce[btcec.PrivKeyBytesLen:])

	if k1.IsZero() || k2.IsZero() {
		return nil, ErrSecretNonceZero
	}

	nonce.ToAffine()

	nonceKey := btcec.NewPublicKey(&nonce.X, &nonce.Y)

	// If the nonce R has an odd y coordinate, then we'll negate both our
	// secret nonces.
	if nonce.Y.IsOdd() {
		k1.Negate()
		k2.Negate()
	}

	privKeyScalar := privKey.Key
	if privKeyScalar.IsZero() {
		return nil, ErrPrivKeyZero
	}

	pubKey := privKey.PubKey()
	combinedKeyYIsOdd := func() bool {
		combinedKeyBytes := combinedKey.FinalKey.SerializeCompressed()
		return combinedKeyBytes[0] == secp.PubKeyFormatCompressedOdd
	}()

	// Next we'll compute the two parity factors for Q, the combined key.
	// If the key is odd, then we'll negate it.
	parityCombinedKey := new(btcec.ModNScalar).SetInt(1)
	if combinedKeyYIsOdd {
		parityCombinedKey.Negate()
	}

	// Before we sign below, we'll multiply by our various parity factors
	// to ensure that the signing key is properly negated (if necessary):
	//  * d = g⋅gacc⋅d'
	privKeyScalar.Mul(parityCombinedKey).Mul(parityAcc)

	// Next we'll create the challenge hash that commits to the combined
	// nonce, combined public key and also the message:
	// * e = H(tag=ChallengeHashTag, R || Q || m) mod n
	var challengeMsg bytes.Buffer
	challengeMsg.Write(schnorr.SerializePubKey(nonceKey))
	challengeMsg.Write(schnorr.SerializePubKey(combinedKey.FinalKey))
	challengeMsg.Write(msg[:])
	challengeBytes := chainhash.TaggedHash(
		ChallengeHashTag, challengeMsg.Bytes(),
	)
	var e btcec.ModNScalar
	e.SetByteSlice(challengeBytes[:])

	// Next, we'll compute a, our aggregation coefficient for the key that
	// we're signing with.
	a := aggregationCoefficient(pubKeys, pubKey, keysHash, uniqueKeyIndex)

	// With mu constructed, we can finally generate our partial signature
	// as: s = (k1_1 + b*k_2 + e*a*d) mod n.
	s := new(btcec.ModNScalar)
	s.Add(&k1).Add(k2.Mul(nonceBlinder)).Add(e.Mul(a).Mul(&privKeyScalar))

	sig := NewPartialSignature(s, nonceKey)

	// If we're not in fast sign mode, then we'll also validate our partial
	// signature.
	if !opts.fastSign {
		pubNonce := secNonceToPubNonce(secNonce)
		sigValid := sig.Verify(
			pubNonce, combinedNonce, pubKeys, pubKey, msg,
			signOpts...,
		)
		if !sigValid {
			return nil, fmt.Errorf("sig is invalid!")
		}
	}

	return &sig, nil
}

// Verify implements partial signature verification given the public nonce for
// the signer, aggregate nonce, signer set and finally the message being
// signed.
func (p *PartialSignature) Verify(pubNonce [PubNonceSize]byte,
	combinedNonce [PubNonceSize]byte, keySet []*btcec.PublicKey,
	signingKey *btcec.PublicKey, msg [32]byte, signOpts ...SignOption) bool {

	pubKey := signingKey.SerializeCompressed()

	return verifyPartialSig(
		p, pubNonce, combinedNonce, keySet, pubKey, msg, signOpts...,
	) == nil
}

// verifyPartialSig attempts to verify a partial schnorr signature given the
// necessary parameters. This is the internal version of Verify that returns
// detailed errors.  signed.
func verifyPartialSig(partialSig *PartialSignature, pubNonce [PubNonceSize]byte,
	combinedNonce [PubNonceSize]byte, keySet []*btcec.PublicKey,
	pubKey []byte, msg [32]byte, signOpts ...SignOption) error {

	opts := defaultSignOptions()
	for _, option := range signOpts {
		option(opts)
	}

	// First we'll map the internal partial signature back into something
	// we can manipulate.
	s := partialSig.S

	// Next we'll parse out the two public nonces into something we can
	// use.
	//
	// Compute the hash of all the keys here as we'll need it do aggregate
	// the keys and also at the final step of verification.
	keysHash := keyHashFingerprint(keySet, opts.sortKeys)
	uniqueKeyIndex := secondUniqueKeyIndex(keySet, opts.sortKeys)

	keyAggOpts := []KeyAggOption{
		WithKeysHash(keysHash), WithUniqueKeyIndex(uniqueKeyIndex),
	}
	switch {
	case opts.bip86Tweak:
		keyAggOpts = append(
			keyAggOpts, WithBIP86KeyTweak(),
		)
	case opts.taprootTweak != nil:
		keyAggOpts = append(
			keyAggOpts, WithTaprootKeyTweak(opts.taprootTweak),
		)
	case len(opts.tweaks) != 0:
		keyAggOpts = append(keyAggOpts, WithKeyTweaks(opts.tweaks...))
	}

	// Next we'll construct the aggregated public key based on the set of
	// signers.
	combinedKey, parityAcc, _, err := AggregateKeys(
		keySet, opts.sortKeys, keyAggOpts...,
	)
	if err != nil {
		return err
	}

	// Next we'll compute the value b, that blinds our second public
	// nonce:
	//  * b = h(tag=NonceBlindTag, combinedNonce || combinedKey || m).
	var (
		nonceMsgBuf  bytes.Buffer
		nonceBlinder btcec.ModNScalar
	)
	nonceMsgBuf.Write(combinedNonce[:])
	nonceMsgBuf.Write(schnorr.SerializePubKey(combinedKey.FinalKey))
	nonceMsgBuf.Write(msg[:])
	nonceBlindHash := chainhash.TaggedHash(NonceBlindTag, nonceMsgBuf.Bytes())
	nonceBlinder.SetByteSlice(nonceBlindHash[:])

	r1J, err := btcec.ParseJacobian(
		combinedNonce[:btcec.PubKeyBytesLenCompressed],
	)
	if err != nil {
		return err
	}
	r2J, err := btcec.ParseJacobian(
		combinedNonce[btcec.PubKeyBytesLenCompressed:],
	)
	if err != nil {
		return err
	}

	// With our nonce blinding value, we'll now combine both the public
	// nonces, using the blinding factor to tweak the second nonce:
	//  * R = R_1 + b*R_2
	var nonce btcec.JacobianPoint
	btcec.ScalarMultNonConst(&nonceBlinder, &r2J, &r2J)
	btcec.AddNonConst(&r1J, &r2J, &nonce)

	// Next, we'll parse out the set of public nonces this signer used to
	// generate the signature.
	pubNonce1J, err := btcec.ParseJacobian(
		pubNonce[:btcec.PubKeyBytesLenCompressed],
	)
	if err != nil {
		return err
	}
	pubNonce2J, err := btcec.ParseJacobian(
		pubNonce[btcec.PubKeyBytesLenCompressed:],
	)
	if err != nil {
		return err
	}

	// If the nonce is the infinity point we set it to the Generator.
	if nonce == infinityPoint {
		btcec.GeneratorJacobian(&nonce)
	} else {
		nonce.ToAffine()
	}

	// We'll perform a similar aggregation and blinding operator as we did
	// above for the combined nonces: R' = R_1' + b*R_2'.
	var pubNonceJ btcec.JacobianPoint

	btcec.ScalarMultNonConst(&nonceBlinder, &pubNonce2J, &pubNonce2J)
	btcec.AddNonConst(&pubNonce1J, &pubNonce2J, &pubNonceJ)

	pubNonceJ.ToAffine()

	// If the combined nonce used in the challenge hash has an odd y
	// coordinate, then we'll negate our final public nonce.
	if nonce.Y.IsOdd() {
		pubNonceJ.Y.Negate(1)
		pubNonceJ.Y.Normalize()
	}

	// Next we'll create the challenge hash that commits to the combined
	// nonce, combined public key and also the message:
	//  * e = H(tag=ChallengeHashTag, R || Q || m) mod n
	var challengeMsg bytes.Buffer
	challengeMsg.Write(schnorr.SerializePubKey(btcec.NewPublicKey(
		&nonce.X, &nonce.Y,
	)))
	challengeMsg.Write(schnorr.SerializePubKey(combinedKey.FinalKey))
	challengeMsg.Write(msg[:])
	challengeBytes := chainhash.TaggedHash(
		ChallengeHashTag, challengeMsg.Bytes(),
	)
	var e btcec.ModNScalar
	e.SetByteSlice(challengeBytes[:])

	signingKey, err := btcec.ParsePubKey(pubKey)
	if err != nil {
		return err
	}

	// Next, we'll compute a, our aggregation coefficient for the key that
	// we're signing with.
	a := aggregationCoefficient(keySet, signingKey, keysHash, uniqueKeyIndex)

	// If the combined key has an odd y coordinate, then we'll negate
	// parity factor for the signing key.
	parityCombinedKey := new(btcec.ModNScalar).SetInt(1)
	combinedKeyBytes := combinedKey.FinalKey.SerializeCompressed()
	if combinedKeyBytes[0] == secp.PubKeyFormatCompressedOdd {
		parityCombinedKey.Negate()
	}

	// Next, we'll construct the final parity factor by multiplying the
	// sign key parity factor with the accumulated parity factor for all
	// the keys.
	finalParityFactor := parityCombinedKey.Mul(parityAcc)

	var signKeyJ btcec.JacobianPoint
	signingKey.AsJacobian(&signKeyJ)

	// In the final set, we'll check that: s*G == R' + e*a*g*P.
	var sG, rP btcec.JacobianPoint
	btcec.ScalarBaseMultNonConst(s, &sG)
	btcec.ScalarMultNonConst(e.Mul(a).Mul(finalParityFactor), &signKeyJ, &rP)
	btcec.AddNonConst(&rP, &pubNonceJ, &rP)

	sG.ToAffine()
	rP.ToAffine()

	if sG != rP {
		return ErrPartialSigInvalid
	}

	return nil
}

// CombineOption is a functional option argument that allows callers to modify the
// way we combine musig2 schnorr signatures.
type CombineOption func(*combineOptions)

// combineOptions houses the set of functional options that can be used to
// modify the method used to combine the musig2 partial signatures.
type combineOptions struct {
	msg [32]byte

	combinedKey *btcec.PublicKey

	tweakAcc *btcec.ModNScalar
}

// defaultCombineOptions returns the default set of signing operations.
func defaultCombineOptions() *combineOptions {
	return &combineOptions{}
}

// WithTweakedCombine is a functional option that allows callers to specify
// that the signature was produced using a tweaked aggregated public key. In
// order to properly aggregate the partial signatures, the caller must specify
// enough information to reconstruct the challenge, and also the final
// accumulated tweak value.
func WithTweakedCombine(msg [32]byte, keys []*btcec.PublicKey,
	tweaks []KeyTweakDesc, sort bool) CombineOption {

	return func(o *combineOptions) {
		combinedKey, _, tweakAcc, _ := AggregateKeys(
			keys, sort, WithKeyTweaks(tweaks...),
		)

		o.msg = msg
		o.combinedKey = combinedKey.FinalKey
		o.tweakAcc = tweakAcc
	}
}

// WithTaprootTweakedCombine is similar to the WithTweakedCombine option, but
// assumes a BIP 341 context where the final tweaked key is to be used as the
// output key, where the internal key is the aggregated key pre-tweak.
//
// This option should be used over WithTweakedCombine when attempting to
// aggregate signatures for a top-level taproot keyspend, where the output key
// commits to a script root.
func WithTaprootTweakedCombine(msg [32]byte, keys []*btcec.PublicKey,
	scriptRoot []byte, sort bool) CombineOption {

	return func(o *combineOptions) {
		combinedKey, _, tweakAcc, _ := AggregateKeys(
			keys, sort, WithTaprootKeyTweak(scriptRoot),
		)

		o.msg = msg
		o.combinedKey = combinedKey.FinalKey
		o.tweakAcc = tweakAcc
	}
}

// WithBip86TweakedCombine is similar to the WithTaprootTweakedCombine option,
// but assumes a BIP 341 + BIP 86 context where the final tweaked key is to be
// used as the output key, where the internal key is the aggregated key
// pre-tweak.
//
// This option should be used over WithTaprootTweakedCombine when attempting to
// aggregate signatures for a top-level taproot keyspend, where the output key
// was generated using BIP 86.
func WithBip86TweakedCombine(msg [32]byte, keys []*btcec.PublicKey,
	sort bool) CombineOption {

	return func(o *combineOptions) {
		combinedKey, _, tweakAcc, _ := AggregateKeys(
			keys, sort, WithBIP86KeyTweak(),
		)

		o.msg = msg
		o.combinedKey = combinedKey.FinalKey
		o.tweakAcc = tweakAcc
	}
}

// CombineSigs combines the set of public keys given the final aggregated
// nonce, and the series of partial signatures for each nonce.
func CombineSigs(combinedNonce *btcec.PublicKey,
	partialSigs []*PartialSignature,
	combineOpts ...CombineOption) *schnorr.Signature {

	// First, parse the set of optional combine options.
	opts := defaultCombineOptions()
	for _, option := range combineOpts {
		option(opts)
	}

	// If signer keys and tweaks are specified, then we need to carry out
	// some intermediate steps before we can combine the signature.
	var tweakProduct *btcec.ModNScalar
	if opts.combinedKey != nil && opts.tweakAcc != nil {
		// Next, we'll construct the parity factor of the combined key,
		// negating it if the combined key has an even y coordinate.
		parityFactor := new(btcec.ModNScalar).SetInt(1)
		combinedKeyBytes := opts.combinedKey.SerializeCompressed()
		if combinedKeyBytes[0] == secp.PubKeyFormatCompressedOdd {
			parityFactor.Negate()
		}

		// Next we'll reconstruct e the challenge has based on the
		// nonce and combined public key.
		//  * e = H(tag=ChallengeHashTag, R || Q || m) mod n
		var challengeMsg bytes.Buffer
		challengeMsg.Write(schnorr.SerializePubKey(combinedNonce))
		challengeMsg.Write(schnorr.SerializePubKey(opts.combinedKey))
		challengeMsg.Write(opts.msg[:])
		challengeBytes := chainhash.TaggedHash(
			ChallengeHashTag, challengeMsg.Bytes(),
		)
		var e btcec.ModNScalar
		e.SetByteSlice(challengeBytes[:])

		tweakProduct = new(btcec.ModNScalar).Set(&e)
		tweakProduct.Mul(opts.tweakAcc).Mul(parityFactor)
	}

	// Finally, the tweak factor also needs to be re-computed as well.
	var combinedSig btcec.ModNScalar
	for _, partialSig := range partialSigs {
		combinedSig.Add(partialSig.S)
	}

	// If the tweak product was set above, then we'll need to add the value
	// at the very end in order to produce a valid signature under the
	// final tweaked key.
	if tweakProduct != nil {
		combinedSig.Add(tweakProduct)
	}

	// TODO(roasbeef): less verbose way to get the x coord...
	var nonceJ btcec.JacobianPoint
	combinedNonce.AsJacobian(&nonceJ)
	nonceJ.ToAffine()

	return schnorr.NewSignature(&nonceJ.X, &combinedSig)
}

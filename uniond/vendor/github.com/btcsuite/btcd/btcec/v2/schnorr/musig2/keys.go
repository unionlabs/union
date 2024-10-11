// Copyright 2013-2022 The btcsuite developers

package musig2

import (
	"bytes"
	"fmt"
	"sort"

	secp "github.com/decred/dcrd/dcrec/secp256k1/v4"

	"github.com/btcsuite/btcd/btcec/v2"
	"github.com/btcsuite/btcd/btcec/v2/schnorr"
	"github.com/btcsuite/btcd/chaincfg/chainhash"
)

var (
	// KeyAggTagList is the tagged hash tag used to compute the hash of the
	// list of sorted public keys.
	KeyAggTagList = []byte("KeyAgg list")

	// KeyAggTagCoeff is the tagged hash tag used to compute the key
	// aggregation coefficient for each key.
	KeyAggTagCoeff = []byte("KeyAgg coefficient")

	// ErrTweakedKeyIsInfinity is returned if while tweaking a key, we end
	// up with the point at infinity.
	ErrTweakedKeyIsInfinity = fmt.Errorf("tweaked key is infinity point")

	// ErrTweakedKeyOverflows is returned if a tweaking key is larger than
	// 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141.
	ErrTweakedKeyOverflows = fmt.Errorf("tweaked key is too large")
)

// sortableKeys defines a type of slice of public keys that implements the sort
// interface for BIP 340 keys.
type sortableKeys []*btcec.PublicKey

// Less reports whether the element with index i must sort before the element
// with index j.
func (s sortableKeys) Less(i, j int) bool {
	// TODO(roasbeef): more efficient way to compare...
	keyIBytes := s[i].SerializeCompressed()
	keyJBytes := s[j].SerializeCompressed()

	return bytes.Compare(keyIBytes, keyJBytes) == -1
}

// Swap swaps the elements with indexes i and j.
func (s sortableKeys) Swap(i, j int) {
	s[i], s[j] = s[j], s[i]
}

// Len is the number of elements in the collection.
func (s sortableKeys) Len() int {
	return len(s)
}

// sortKeys takes a set of public keys and returns a new slice that is a copy
// of the keys sorted in lexicographical order bytes on the x-only pubkey
// serialization.
func sortKeys(keys []*btcec.PublicKey) []*btcec.PublicKey {
	keySet := sortableKeys(keys)
	if sort.IsSorted(keySet) {
		return keys
	}

	sort.Sort(keySet)
	return keySet
}

// keyHashFingerprint computes the tagged hash of the series of (sorted) public
// keys passed as input. This is used to compute the aggregation coefficient
// for each key. The final computation is:
//   - H(tag=KeyAgg list, pk1 || pk2..)
func keyHashFingerprint(keys []*btcec.PublicKey, sort bool) []byte {
	if sort {
		keys = sortKeys(keys)
	}

	// We'll create a single buffer and slice into that so the bytes buffer
	// doesn't continually need to grow the underlying buffer.
	keyAggBuf := make([]byte, 33*len(keys))
	keyBytes := bytes.NewBuffer(keyAggBuf[0:0])
	for _, key := range keys {
		keyBytes.Write(key.SerializeCompressed())
	}

	h := chainhash.TaggedHash(KeyAggTagList, keyBytes.Bytes())
	return h[:]
}

// keyBytesEqual returns true if two keys are the same based on the compressed
// serialization of each key.
func keyBytesEqual(a, b *btcec.PublicKey) bool {
	return bytes.Equal(a.SerializeCompressed(), b.SerializeCompressed())
}

// aggregationCoefficient computes the key aggregation coefficient for the
// specified target key. The coefficient is computed as:
//   - H(tag=KeyAgg coefficient, keyHashFingerprint(pks) || pk)
func aggregationCoefficient(keySet []*btcec.PublicKey,
	targetKey *btcec.PublicKey, keysHash []byte,
	secondKeyIdx int) *btcec.ModNScalar {

	var mu btcec.ModNScalar

	// If this is the second key, then this coefficient is just one.
	if secondKeyIdx != -1 && keyBytesEqual(keySet[secondKeyIdx], targetKey) {
		return mu.SetInt(1)
	}

	// Otherwise, we'll compute the full finger print hash for this given
	// key and then use that to compute the coefficient tagged hash:
	//  * H(tag=KeyAgg coefficient, keyHashFingerprint(pks, pk) || pk)
	var coefficientBytes [65]byte
	copy(coefficientBytes[:], keysHash[:])
	copy(coefficientBytes[32:], targetKey.SerializeCompressed())

	muHash := chainhash.TaggedHash(KeyAggTagCoeff, coefficientBytes[:])

	mu.SetByteSlice(muHash[:])

	return &mu
}

// secondUniqueKeyIndex returns the index of the second unique key. If all keys
// are the same, then a value of -1 is returned.
func secondUniqueKeyIndex(keySet []*btcec.PublicKey, sort bool) int {
	if sort {
		keySet = sortKeys(keySet)
	}

	// Find the first key that isn't the same as the very first key (second
	// unique key).
	for i := range keySet {
		if !keyBytesEqual(keySet[i], keySet[0]) {
			return i
		}
	}

	// A value of negative one is used to indicate that all the keys in the
	// sign set are actually equal, which in practice actually makes musig2
	// useless, but we need a value to distinguish this case.
	return -1
}

// KeyTweakDesc describes a tweak to be applied to the aggregated public key
// generation and signing process. The IsXOnly specifies if the target key
// should be converted to an x-only public key before tweaking.
type KeyTweakDesc struct {
	// Tweak is the 32-byte value that will modify the public key.
	Tweak [32]byte

	// IsXOnly if true, then the public key will be mapped to an x-only key
	// before the tweaking operation is applied.
	IsXOnly bool
}

// KeyAggOption is a functional option argument that allows callers to specify
// more or less information that has been pre-computed to the main routine.
type KeyAggOption func(*keyAggOption)

// keyAggOption houses the set of functional options that modify key
// aggregation.
type keyAggOption struct {
	// keyHash is the output of keyHashFingerprint for a given set of keys.
	keyHash []byte

	// uniqueKeyIndex is the pre-computed index of the second unique key.
	uniqueKeyIndex *int

	// tweaks specifies a series of tweaks to be applied to the aggregated
	// public key.
	tweaks []KeyTweakDesc

	// taprootTweak controls if the tweaks above should be applied in a BIP
	// 340 style.
	taprootTweak bool

	// bip86Tweak specifies that the taproot tweak should be done in a BIP
	// 86 style, where we don't expect an actual tweak and instead just
	// commit to the public key itself.
	bip86Tweak bool
}

// WithKeysHash allows key aggregation to be optimize, by allowing the caller
// to specify the hash of all the keys.
func WithKeysHash(keyHash []byte) KeyAggOption {
	return func(o *keyAggOption) {
		o.keyHash = keyHash
	}
}

// WithUniqueKeyIndex allows the caller to specify the index of the second
// unique key.
func WithUniqueKeyIndex(idx int) KeyAggOption {
	return func(o *keyAggOption) {
		i := idx
		o.uniqueKeyIndex = &i
	}
}

// WithKeyTweaks allows a caller to specify a series of 32-byte tweaks that
// should be applied to the final aggregated public key.
func WithKeyTweaks(tweaks ...KeyTweakDesc) KeyAggOption {
	return func(o *keyAggOption) {
		o.tweaks = tweaks
	}
}

// WithTaprootKeyTweak specifies that within this context, the final key should
// use the taproot tweak as defined in BIP 341: outputKey = internalKey +
// h_tapTweak(internalKey || scriptRoot). In this case, the aggregated key
// before the tweak will be used as the internal key.
//
// This option should be used instead of WithKeyTweaks when the aggregated key
// is intended to be used as a taproot output key that commits to a script
// root.
func WithTaprootKeyTweak(scriptRoot []byte) KeyAggOption {
	return func(o *keyAggOption) {
		var tweak [32]byte
		copy(tweak[:], scriptRoot[:])

		o.tweaks = []KeyTweakDesc{
			{
				Tweak:   tweak,
				IsXOnly: true,
			},
		}
		o.taprootTweak = true
	}
}

// WithBIP86KeyTweak specifies that then during key aggregation, the BIP 86
// tweak which just commits to the hash of the serialized public key should be
// used. This option should be used when signing with a key that was derived
// using BIP 86.
func WithBIP86KeyTweak() KeyAggOption {
	return func(o *keyAggOption) {
		o.tweaks = []KeyTweakDesc{
			{
				IsXOnly: true,
			},
		}
		o.taprootTweak = true
		o.bip86Tweak = true
	}
}

// defaultKeyAggOptions returns the set of default arguments for key
// aggregation.
func defaultKeyAggOptions() *keyAggOption {
	return &keyAggOption{}
}

// hasEvenY returns true if the affine representation of the passed jacobian
// point has an even y coordinate.
//
// TODO(roasbeef): double check, can just check the y coord even not jacobian?
func hasEvenY(pJ btcec.JacobianPoint) bool {
	pJ.ToAffine()
	p := btcec.NewPublicKey(&pJ.X, &pJ.Y)
	keyBytes := p.SerializeCompressed()
	return keyBytes[0] == secp.PubKeyFormatCompressedEven
}

// tweakKey applies a tweaks to the passed public key using the specified
// tweak. The parityAcc and tweakAcc are returned (in that order) which
// includes the accumulate ration of the parity factor and the tweak multiplied
// by the parity factor. The xOnly bool specifies if this is to be an x-only
// tweak or not.
func tweakKey(keyJ btcec.JacobianPoint, parityAcc btcec.ModNScalar, tweak [32]byte,
	tweakAcc btcec.ModNScalar,
	xOnly bool) (btcec.JacobianPoint, btcec.ModNScalar, btcec.ModNScalar, error) {

	// First we'll compute the new parity factor for this key. If the key has
	// an odd y coordinate (not even), then we'll need to negate it (multiply
	// by -1 mod n, in this case).
	var parityFactor btcec.ModNScalar
	if xOnly && !hasEvenY(keyJ) {
		parityFactor.SetInt(1).Negate()
	} else {
		parityFactor.SetInt(1)
	}

	// Next, map the tweak into a mod n integer so we can use it for
	// manipulations below.
	tweakInt := new(btcec.ModNScalar)
	overflows := tweakInt.SetBytes(&tweak)
	if overflows == 1 {
		return keyJ, parityAcc, tweakAcc, ErrTweakedKeyOverflows
	}

	// Next, we'll compute: Q_i = g*Q + t*G, where g is our parityFactor and t
	// is the tweakInt above. We'll space things out a bit to make it easier to
	// follow.
	//
	// First compute t*G:
	var tweakedGenerator btcec.JacobianPoint
	btcec.ScalarBaseMultNonConst(tweakInt, &tweakedGenerator)

	// Next compute g*Q:
	btcec.ScalarMultNonConst(&parityFactor, &keyJ, &keyJ)

	// Finally add both of them together to get our final
	// tweaked point.
	btcec.AddNonConst(&tweakedGenerator, &keyJ, &keyJ)

	// As a sanity check, make sure that we didn't just end up with the
	// point at infinity.
	if keyJ == infinityPoint {
		return keyJ, parityAcc, tweakAcc, ErrTweakedKeyIsInfinity
	}

	// As a final wrap up step, we'll accumulate the parity
	// factor and also this tweak into the final set of accumulators.
	parityAcc.Mul(&parityFactor)
	tweakAcc.Mul(&parityFactor).Add(tweakInt)

	return keyJ, parityAcc, tweakAcc, nil
}

// AggregateKey is a final aggregated key along with a possible version of the
// key without any tweaks applied.
type AggregateKey struct {
	// FinalKey is the final aggregated key which may include one or more
	// tweaks applied to it.
	FinalKey *btcec.PublicKey

	// PreTweakedKey is the aggregated *before* any tweaks have been
	// applied.  This should be used as the internal key in taproot
	// contexts.
	PreTweakedKey *btcec.PublicKey
}

// AggregateKeys takes a list of possibly unsorted keys and returns a single
// aggregated key as specified by the musig2 key aggregation algorithm. A nil
// value can be passed for keyHash, which causes this function to re-derive it.
// In addition to the combined public key, the parity accumulator and the tweak
// accumulator are returned as well.
func AggregateKeys(keys []*btcec.PublicKey, sort bool,
	keyOpts ...KeyAggOption) (
	*AggregateKey, *btcec.ModNScalar, *btcec.ModNScalar, error) {

	// First, parse the set of optional signing options.
	opts := defaultKeyAggOptions()
	for _, option := range keyOpts {
		option(opts)
	}

	// Sort the set of public key so we know we're working with them in
	// sorted order for all the routines below.
	if sort {
		keys = sortKeys(keys)
	}

	// The caller may provide the hash of all the keys as an optimization
	// during signing, as it already needs to be computed.
	if opts.keyHash == nil {
		opts.keyHash = keyHashFingerprint(keys, sort)
	}

	// A caller may also specify the unique key index themselves so we
	// don't need to re-compute it.
	if opts.uniqueKeyIndex == nil {
		idx := secondUniqueKeyIndex(keys, sort)
		opts.uniqueKeyIndex = &idx
	}

	// For each key, we'll compute the intermediate blinded key: a_i*P_i,
	// where a_i is the aggregation coefficient for that key, and P_i is
	// the key itself, then accumulate that (addition) into the main final
	// key: P = P_1 + P_2 ... P_N.
	var finalKeyJ btcec.JacobianPoint
	for _, key := range keys {
		// Port the key over to Jacobian coordinates as we need it in
		// this format for the routines below.
		var keyJ btcec.JacobianPoint
		key.AsJacobian(&keyJ)

		// Compute the aggregation coefficient for the key, then
		// multiply it by the key itself: P_i' = a_i*P_i.
		var tweakedKeyJ btcec.JacobianPoint
		a := aggregationCoefficient(
			keys, key, opts.keyHash, *opts.uniqueKeyIndex,
		)
		btcec.ScalarMultNonConst(a, &keyJ, &tweakedKeyJ)

		// Finally accumulate this into the final key in an incremental
		// fashion.
		btcec.AddNonConst(&finalKeyJ, &tweakedKeyJ, &finalKeyJ)
	}

	// We'll copy over the key at this point, since this represents the
	// aggregated key before any tweaks have been applied. This'll be used
	// as the internal key for script path proofs.
	finalKeyJ.ToAffine()
	combinedKey := btcec.NewPublicKey(&finalKeyJ.X, &finalKeyJ.Y)

	// At this point, if this is a taproot tweak, then we'll modify the
	// base tweak value to use the BIP 341 tweak value.
	if opts.taprootTweak {
		// Emulate the same behavior as txscript.ComputeTaprootOutputKey
		// which only operates on the x-only public key.
		key, _ := schnorr.ParsePubKey(schnorr.SerializePubKey(
			combinedKey,
		))

		// We only use the actual tweak bytes if we're not committing
		// to a BIP-0086 key only spend output. Otherwise, we just
		// commit to the internal key and an empty byte slice as the
		// root hash.
		tweakBytes := []byte{}
		if !opts.bip86Tweak {
			tweakBytes = opts.tweaks[0].Tweak[:]
		}

		// Compute the taproot key tagged hash of:
		// h_tapTweak(internalKey || scriptRoot). We only do this for
		// the first one, as you can only specify a single tweak when
		// using the taproot mode with this API.
		tapTweakHash := chainhash.TaggedHash(
			chainhash.TagTapTweak, schnorr.SerializePubKey(key),
			tweakBytes,
		)
		opts.tweaks[0].Tweak = *tapTweakHash
	}

	var (
		err       error
		tweakAcc  btcec.ModNScalar
		parityAcc btcec.ModNScalar
	)
	parityAcc.SetInt(1)

	// In this case we have a set of tweaks, so we'll incrementally apply
	// each one, until we have our final tweaked key, and the related
	// accumulators.
	for i := 1; i <= len(opts.tweaks); i++ {
		finalKeyJ, parityAcc, tweakAcc, err = tweakKey(
			finalKeyJ, parityAcc, opts.tweaks[i-1].Tweak, tweakAcc,
			opts.tweaks[i-1].IsXOnly,
		)
		if err != nil {
			return nil, nil, nil, err
		}
	}

	finalKeyJ.ToAffine()
	finalKey := btcec.NewPublicKey(&finalKeyJ.X, &finalKeyJ.Y)

	return &AggregateKey{
		PreTweakedKey: combinedKey,
		FinalKey:      finalKey,
	}, &parityAcc, &tweakAcc, nil
}

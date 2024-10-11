// Copyright (c) 2013-2022 The btcsuite developers

package musig2

import (
	"fmt"

	"github.com/btcsuite/btcd/btcec/v2"
	"github.com/btcsuite/btcd/btcec/v2/schnorr"
)

var (
	// ErrSignersNotSpecified is returned when a caller attempts to create
	// a context without specifying either the total number of signers, or
	// the complete set of singers.
	ErrSignersNotSpecified = fmt.Errorf("total number of signers or all " +
		"signers must be known")

	// ErrSignerNotInKeySet is returned when a the private key for a signer
	// isn't included in the set of signing public keys.
	ErrSignerNotInKeySet = fmt.Errorf("signing key is not found in key" +
		" set")

	// ErrAlredyHaveAllNonces is called when RegisterPubNonce is called too
	// many times for a given signing session.
	ErrAlredyHaveAllNonces = fmt.Errorf("already have all nonces")

	// ErrNotEnoughSigners is returned when a caller attempts to create a
	// session from a context, but before all the required signers are
	// known.
	ErrNotEnoughSigners = fmt.Errorf("not enough signers")

	// ErrAlredyHaveAllNonces is returned when a caller attempts to
	// register a signer, once we already have the total set of known
	// signers.
	ErrAlreadyHaveAllSigners = fmt.Errorf("all signers registered")

	// ErrAlredyHaveAllSigs is called when CombineSig is called too many
	// times for a given signing session.
	ErrAlredyHaveAllSigs = fmt.Errorf("already have all sigs")

	// ErrSigningContextReuse is returned if a user attempts to sign using
	// the same signing context more than once.
	ErrSigningContextReuse = fmt.Errorf("nonce already used")

	// ErrFinalSigInvalid is returned when the combined signature turns out
	// to be invalid.
	ErrFinalSigInvalid = fmt.Errorf("final signature is invalid")

	// ErrCombinedNonceUnavailable is returned when a caller attempts to
	// sign a partial signature, without first having collected all the
	// required combined nonces.
	ErrCombinedNonceUnavailable = fmt.Errorf("missing combined nonce")

	// ErrTaprootInternalKeyUnavailable is returned when a user attempts to
	// obtain the
	ErrTaprootInternalKeyUnavailable = fmt.Errorf("taproot tweak not used")

	// ErrNotEnoughSigners is returned if a caller attempts to obtain an
	// early nonce when it wasn't specified
	ErrNoEarlyNonce = fmt.Errorf("no early nonce available")
)

// Context is a managed signing context for musig2. It takes care of things
// like securely generating secret nonces, aggregating keys and nonces, etc.
type Context struct {
	// signingKey is the key we'll use for signing.
	signingKey *btcec.PrivateKey

	// pubKey is our even-y coordinate public  key.
	pubKey *btcec.PublicKey

	// combinedKey is the aggregated public key.
	combinedKey *AggregateKey

	// uniqueKeyIndex is the index of the second unique key in the keySet.
	// This is used to speed up signing and verification computations.
	uniqueKeyIndex int

	// keysHash is the hash of all the keys as defined in musig2.
	keysHash []byte

	// opts is the set of options for the context.
	opts *contextOptions

	// shouldSort keeps track of if the public keys should be sorted before
	// any operations.
	shouldSort bool

	// sessionNonce will be populated if the earlyNonce option is true.
	// After the first session is created, this nonce will be blanked out.
	sessionNonce *Nonces
}

// ContextOption is a functional option argument that allows callers to modify
// the musig2 signing is done within a context.
type ContextOption func(*contextOptions)

// contextOptions houses the set of functional options that can be used to
// musig2 signing protocol.
type contextOptions struct {
	// tweaks is the set of optinoal tweaks to apply to the combined public
	// key.
	tweaks []KeyTweakDesc

	// taprootTweak specifies the taproot tweak. If specified, then we'll
	// use this as the script root for the BIP 341 taproot (x-only) tweak.
	// Normally we'd just apply the raw 32 byte tweak, but for taproot, we
	// first need to compute the aggregated key before tweaking, and then
	// use it as the internal key. This is required as the taproot tweak
	// also commits to the public key, which in this case is the aggregated
	// key before the tweak.
	taprootTweak []byte

	// bip86Tweak if true, then the weak will just be
	// h_tapTweak(internalKey) as there is no true script root.
	bip86Tweak bool

	// keySet is the complete set of signers for this context.
	keySet []*btcec.PublicKey

	// numSigners is the total number of signers that will eventually be a
	// part of the context.
	numSigners int

	// earlyNonce determines if a nonce should be generated during context
	// creation, to be automatically passed to the created session.
	earlyNonce bool
}

// defaultContextOptions returns the default context options.
func defaultContextOptions() *contextOptions {
	return &contextOptions{}
}

// WithTweakedContext specifies that within the context, the aggregated public
// key should be tweaked with the specified tweaks.
func WithTweakedContext(tweaks ...KeyTweakDesc) ContextOption {
	return func(o *contextOptions) {
		o.tweaks = tweaks
	}
}

// WithTaprootTweakCtx specifies that within this context, the final key should
// use the taproot tweak as defined in BIP 341: outputKey = internalKey +
// h_tapTweak(internalKey || scriptRoot). In this case, the aggreaged key
// before the tweak will be used as the internal key.
func WithTaprootTweakCtx(scriptRoot []byte) ContextOption {
	return func(o *contextOptions) {
		o.taprootTweak = scriptRoot
	}
}

// WithBip86TweakCtx specifies that within this context, the final key should
// use the taproot tweak as defined in BIP 341, with the BIP 86 modification:
// outputKey = internalKey + h_tapTweak(internalKey)*G. In this case, the
// aggreaged key before the tweak will be used as the internal key.
func WithBip86TweakCtx() ContextOption {
	return func(o *contextOptions) {
		o.bip86Tweak = true
	}
}

// WithKnownSigners is an optional parameter that should be used if a session
// can be created as soon as all the singers are known.
func WithKnownSigners(signers []*btcec.PublicKey) ContextOption {
	return func(o *contextOptions) {
		o.keySet = signers
		o.numSigners = len(signers)
	}
}

// WithNumSigners is a functional option used to specify that a context should
// be created without knowing all the signers. Instead the total number of
// signers is specified to ensure that a session can only be created once all
// the signers are known.
//
// NOTE: Either WithKnownSigners or WithNumSigners MUST be specified.
func WithNumSigners(n int) ContextOption {
	return func(o *contextOptions) {
		o.numSigners = n
	}
}

// WithEarlyNonceGen allow a caller to specify that a nonce should be generated
// early, before the session is created. This should be used in protocols that
// require some partial nonce exchange before all the signers are known.
//
// NOTE: This option must only be specified with the WithNumSigners option.
func WithEarlyNonceGen() ContextOption {
	return func(o *contextOptions) {
		o.earlyNonce = true
	}
}

// NewContext creates a new signing context with the passed singing key and set
// of public keys for each of the other signers.
//
// NOTE: This struct should be used over the raw Sign API whenever possible.
func NewContext(signingKey *btcec.PrivateKey, shouldSort bool,
	ctxOpts ...ContextOption) (*Context, error) {

	// First, parse the set of optional context options.
	opts := defaultContextOptions()
	for _, option := range ctxOpts {
		option(opts)
	}

	pubKey := signingKey.PubKey()

	ctx := &Context{
		signingKey: signingKey,
		pubKey:     pubKey,
		opts:       opts,
		shouldSort: shouldSort,
	}

	switch {

	// We know all the signers, so we can compute the aggregated key, along
	// with all the other intermediate state we need to do signing and
	// verification.
	case opts.keySet != nil:
		if err := ctx.combineSignerKeys(); err != nil {
			return nil, err
		}

	// The total signers are known, so we add ourselves, and skip key
	// aggregation.
	case opts.numSigners != 0:
		// Otherwise, we'll add ourselves as the only known signer, and
		// await further calls to RegisterSigner before a session can
		// be created.
		opts.keySet = make([]*btcec.PublicKey, 0, opts.numSigners)
		opts.keySet = append(opts.keySet, pubKey)

	default:
		return nil, ErrSignersNotSpecified
	}

	// If early nonce generation is specified, then we'll generate the
	// nonce now to pass in to the session once all the callers are known.
	if opts.earlyNonce {
		var err error
		ctx.sessionNonce, err = GenNonces(
			WithPublicKey(ctx.pubKey),
			WithNonceSecretKeyAux(signingKey),
		)
		if err != nil {
			return nil, err
		}
	}

	return ctx, nil
}

// combineSignerKeys is used to compute the aggregated signer key once all the
// signers are known.
func (c *Context) combineSignerKeys() error {
	// As a sanity check, make sure the signing key is actually
	// amongst the sit of signers.
	var keyFound bool
	for _, key := range c.opts.keySet {
		if key.IsEqual(c.pubKey) {
			keyFound = true
			break
		}
	}
	if !keyFound {
		return ErrSignerNotInKeySet
	}

	// Now that we know that we're actually a signer, we'll
	// generate the key hash finger print and second unique key
	// index so we can speed up signing later.
	c.keysHash = keyHashFingerprint(c.opts.keySet, c.shouldSort)
	c.uniqueKeyIndex = secondUniqueKeyIndex(
		c.opts.keySet, c.shouldSort,
	)

	keyAggOpts := []KeyAggOption{
		WithKeysHash(c.keysHash),
		WithUniqueKeyIndex(c.uniqueKeyIndex),
	}
	switch {
	case c.opts.bip86Tweak:
		keyAggOpts = append(
			keyAggOpts, WithBIP86KeyTweak(),
		)
	case c.opts.taprootTweak != nil:
		keyAggOpts = append(
			keyAggOpts, WithTaprootKeyTweak(c.opts.taprootTweak),
		)
	case len(c.opts.tweaks) != 0:
		keyAggOpts = append(keyAggOpts, WithKeyTweaks(c.opts.tweaks...))
	}

	// Next, we'll use this information to compute the aggregated
	// public key that'll be used for signing in practice.
	var err error
	c.combinedKey, _, _, err = AggregateKeys(
		c.opts.keySet, c.shouldSort, keyAggOpts...,
	)
	if err != nil {
		return err
	}

	return nil
}

// EarlySessionNonce returns the early session nonce, if available.
func (c *Context) EarlySessionNonce() (*Nonces, error) {
	if c.sessionNonce == nil {
		return nil, ErrNoEarlyNonce
	}

	return c.sessionNonce, nil
}

// RegisterSigner allows a caller to register a signer after the context has
// been created. This will be used in scenarios where the total number of
// signers is known, but nonce exchange needs to happen before all the signers
// are known.
//
// A bool is returned which indicates if all the signers have been registered.
//
// NOTE: If the set of keys are not to be sorted during signing, then the
// ordering each key is registered with MUST match the desired ordering.
func (c *Context) RegisterSigner(pub *btcec.PublicKey) (bool, error) {
	haveAllSigners := len(c.opts.keySet) == c.opts.numSigners
	if haveAllSigners {
		return false, ErrAlreadyHaveAllSigners
	}

	c.opts.keySet = append(c.opts.keySet, pub)

	// If we have the expected number of signers at this point, then we can
	// generate the aggregated key and other necessary information.
	haveAllSigners = len(c.opts.keySet) == c.opts.numSigners
	if haveAllSigners {
		if err := c.combineSignerKeys(); err != nil {
			return false, err
		}
	}

	return haveAllSigners, nil
}

// NumRegisteredSigners returns the total number of registered signers.
func (c *Context) NumRegisteredSigners() int {
	return len(c.opts.keySet)
}

// CombinedKey returns the combined public key that will be used to generate
// multi-signatures  against.
func (c *Context) CombinedKey() (*btcec.PublicKey, error) {
	// If the caller hasn't registered all the signers at this point, then
	// the combined key won't be available.
	if c.combinedKey == nil {
		return nil, ErrNotEnoughSigners
	}

	return c.combinedKey.FinalKey, nil
}

// PubKey returns the public key of the signer of this session.
func (c *Context) PubKey() btcec.PublicKey {
	return *c.pubKey
}

// SigningKeys returns the set of keys used for signing.
func (c *Context) SigningKeys() []*btcec.PublicKey {
	keys := make([]*btcec.PublicKey, len(c.opts.keySet))
	copy(keys, c.opts.keySet)

	return keys
}

// TaprootInternalKey returns the internal taproot key, which is the aggregated
// key _before_ the tweak is applied. If a taproot tweak was specified, then
// CombinedKey() will return the fully tweaked output key, with this method
// returning the internal key. If a taproot tweak wasn't specified, then this
// method will return an error.
func (c *Context) TaprootInternalKey() (*btcec.PublicKey, error) {
	// If the caller hasn't registered all the signers at this point, then
	// the combined key won't be available.
	if c.combinedKey == nil {
		return nil, ErrNotEnoughSigners
	}

	if c.opts.taprootTweak == nil && !c.opts.bip86Tweak {
		return nil, ErrTaprootInternalKeyUnavailable
	}

	return c.combinedKey.PreTweakedKey, nil
}

// SessionOption is a functional option argument that allows callers to modify
// the musig2 signing is done within a session.
type SessionOption func(*sessionOptions)

// sessionOptions houses the set of functional options that can be used to
// modify the musig2 signing protocol.
type sessionOptions struct {
	externalNonce *Nonces
}

// defaultSessionOptions returns the default session options.
func defaultSessionOptions() *sessionOptions {
	return &sessionOptions{}
}

// WithPreGeneratedNonce allows a caller to start a session using a nonce
// they've generated themselves. This may be useful in protocols where all the
// signer keys may not be known before nonce exchange needs to occur.
func WithPreGeneratedNonce(nonce *Nonces) SessionOption {
	return func(o *sessionOptions) {
		o.externalNonce = nonce
	}
}

// Session represents a musig2 signing session. A new instance should be
// created each time a multi-signature is needed. The session struct handles
// nonces management, incremental partial sig vitrifaction, as well as final
// signature combination. Errors are returned when unsafe behavior such as
// nonce re-use is attempted.
//
// NOTE: This struct should be used over the raw Sign API whenever possible.
type Session struct {
	opts *sessionOptions

	ctx *Context

	localNonces *Nonces

	pubNonces [][PubNonceSize]byte

	combinedNonce *[PubNonceSize]byte

	msg [32]byte

	ourSig *PartialSignature
	sigs   []*PartialSignature

	finalSig *schnorr.Signature
}

// NewSession creates a new musig2 signing session.
func (c *Context) NewSession(options ...SessionOption) (*Session, error) {
	opts := defaultSessionOptions()
	for _, opt := range options {
		opt(opts)
	}

	// At this point we verify that we know of all the signers, as
	// otherwise we can't proceed with the session. This check is intended
	// to catch misuse of the API wherein a caller forgets to register the
	// remaining signers if they're doing nonce generation ahead of time.
	if len(c.opts.keySet) != c.opts.numSigners {
		return nil, ErrNotEnoughSigners
	}

	// If an early nonce was specified, then we'll automatically add the
	// corresponding session option for the caller.
	var localNonces *Nonces
	if c.sessionNonce != nil {
		// Apply the early nonce to the session, and also blank out the
		// session nonce on the context to ensure it isn't ever re-used
		// for another session.
		localNonces = c.sessionNonce
		c.sessionNonce = nil
	} else if opts.externalNonce != nil {
		// Otherwise if there's a custom nonce passed in via the
		// session options, then use that instead.
		localNonces = opts.externalNonce
	}

	// Now that we know we have enough signers, we'll either use the caller
	// specified nonce, or generate a fresh set.
	var err error
	if localNonces == nil {
		// At this point we need to generate a fresh nonce. We'll pass
		// in some auxiliary information to strengthen the nonce
		// generated.
		localNonces, err = GenNonces(
			WithPublicKey(c.pubKey),
			WithNonceSecretKeyAux(c.signingKey),
			WithNonceCombinedKeyAux(c.combinedKey.FinalKey),
		)
		if err != nil {
			return nil, err
		}
	}

	s := &Session{
		opts:        opts,
		ctx:         c,
		localNonces: localNonces,
		pubNonces:   make([][PubNonceSize]byte, 0, c.opts.numSigners),
		sigs:        make([]*PartialSignature, 0, c.opts.numSigners),
	}

	s.pubNonces = append(s.pubNonces, localNonces.PubNonce)

	return s, nil
}

// PublicNonce returns the public nonce for a signer. This should be sent to
// other parties before signing begins, so they can compute the aggregated
// public nonce.
func (s *Session) PublicNonce() [PubNonceSize]byte {
	return s.localNonces.PubNonce
}

// NumRegisteredNonces returns the total number of nonces that have been
// registered so far.
func (s *Session) NumRegisteredNonces() int {
	return len(s.pubNonces)
}

// RegisterPubNonce should be called for each public nonce from the set of
// signers. This method returns true once all the public nonces have been
// accounted for.
func (s *Session) RegisterPubNonce(nonce [PubNonceSize]byte) (bool, error) {
	// If we already have all the nonces, then this method was called too
	// many times.
	haveAllNonces := len(s.pubNonces) == s.ctx.opts.numSigners
	if haveAllNonces {
		return false, ErrAlredyHaveAllNonces
	}

	// Add this nonce and check again if we already have tall the nonces we
	// need.
	s.pubNonces = append(s.pubNonces, nonce)
	haveAllNonces = len(s.pubNonces) == s.ctx.opts.numSigners

	// If we have all the nonces, then we can go ahead and combine them
	// now.
	if haveAllNonces {
		combinedNonce, err := AggregateNonces(s.pubNonces)
		if err != nil {
			return false, err
		}

		s.combinedNonce = &combinedNonce
	}

	return haveAllNonces, nil
}

// Sign generates a partial signature for the target message, using the target
// context. If this method is called more than once per context, then an error
// is returned, as that means a nonce was re-used.
func (s *Session) Sign(msg [32]byte,
	signOpts ...SignOption) (*PartialSignature, error) {

	switch {
	// If no local nonce is present, then this means we already signed, so
	// we'll return an error to prevent nonce re-use.
	case s.localNonces == nil:
		return nil, ErrSigningContextReuse

	// We also need to make sure we have the combined nonce, otherwise this
	// function was called too early.
	case s.combinedNonce == nil:
		return nil, ErrCombinedNonceUnavailable
	}

	switch {
	case s.ctx.opts.bip86Tweak:
		signOpts = append(
			signOpts, WithBip86SignTweak(),
		)
	case s.ctx.opts.taprootTweak != nil:
		signOpts = append(
			signOpts, WithTaprootSignTweak(s.ctx.opts.taprootTweak),
		)
	case len(s.ctx.opts.tweaks) != 0:
		signOpts = append(signOpts, WithTweaks(s.ctx.opts.tweaks...))
	}

	partialSig, err := Sign(
		s.localNonces.SecNonce, s.ctx.signingKey, *s.combinedNonce,
		s.ctx.opts.keySet, msg, signOpts...,
	)

	// Now that we've generated our signature, we'll make sure to blank out
	// our signing nonce.
	s.localNonces = nil

	if err != nil {
		return nil, err
	}

	s.msg = msg

	s.ourSig = partialSig
	s.sigs = append(s.sigs, partialSig)

	return partialSig, nil
}

// CombineSig buffers a partial signature received from a signing party. The
// method returns true once all the signatures are available, and can be
// combined into the final signature.
func (s *Session) CombineSig(sig *PartialSignature) (bool, error) {
	// First check if we already have all the signatures we need. We
	// already accumulated our own signature when we generated the sig.
	haveAllSigs := len(s.sigs) == len(s.ctx.opts.keySet)
	if haveAllSigs {
		return false, ErrAlredyHaveAllSigs
	}

	// TODO(roasbeef): incremental check for invalid sig, or just detect at
	// the very end?

	// Accumulate this sig, and check again if we have all the sigs we
	// need.
	s.sigs = append(s.sigs, sig)
	haveAllSigs = len(s.sigs) == len(s.ctx.opts.keySet)

	// If we have all the signatures, then we can combine them all into the
	// final signature.
	if haveAllSigs {
		var combineOpts []CombineOption
		switch {
		case s.ctx.opts.bip86Tweak:
			combineOpts = append(
				combineOpts, WithBip86TweakedCombine(
					s.msg, s.ctx.opts.keySet,
					s.ctx.shouldSort,
				),
			)
		case s.ctx.opts.taprootTweak != nil:
			combineOpts = append(
				combineOpts, WithTaprootTweakedCombine(
					s.msg, s.ctx.opts.keySet,
					s.ctx.opts.taprootTweak, s.ctx.shouldSort,
				),
			)
		case len(s.ctx.opts.tweaks) != 0:
			combineOpts = append(
				combineOpts, WithTweakedCombine(
					s.msg, s.ctx.opts.keySet,
					s.ctx.opts.tweaks, s.ctx.shouldSort,
				),
			)
		}

		finalSig := CombineSigs(s.ourSig.R, s.sigs, combineOpts...)

		// We'll also verify the signature at this point to ensure it's
		// valid.
		//
		// TODO(roasbef): allow skipping?
		if !finalSig.Verify(s.msg[:], s.ctx.combinedKey.FinalKey) {
			return false, ErrFinalSigInvalid
		}

		s.finalSig = finalSig
	}

	return haveAllSigs, nil
}

// FinalSig returns the final combined multi-signature, if present.
func (s *Session) FinalSig() *schnorr.Signature {
	return s.finalSig
}

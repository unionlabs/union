package ledger

import (
	"errors"
	"fmt"
	"math/big"
	"os"

	secp "github.com/decred/dcrd/dcrec/secp256k1/v4"
	"github.com/decred/dcrd/dcrec/secp256k1/v4/ecdsa"
	"gitlab.com/yawning/secp256k1-voi/secec"

	"github.com/cosmos/cosmos-sdk/crypto/hd"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/crypto/types"
)

// options stores the Ledger Options that can be used to customize Ledger usage
var options Options

type (
	// discoverLedgerFn defines a Ledger discovery function that returns a
	// connected device or an error upon failure. Its allows a method to avoid CGO
	// dependencies when Ledger support is potentially not enabled.
	discoverLedgerFn func() (SECP256K1, error)

	// createPubkeyFn supports returning different public key types that implement
	// types.PubKey
	createPubkeyFn func([]byte) types.PubKey

	// SECP256K1 reflects an interface a Ledger API must implement for SECP256K1
	SECP256K1 interface {
		Close() error
		// GetPublicKeySECP256K1 returns an uncompressed pubkey
		GetPublicKeySECP256K1([]uint32) ([]byte, error)
		// GetAddressPubKeySECP256K1 returns a compressed pubkey and bech32 address (requires user confirmation)
		GetAddressPubKeySECP256K1([]uint32, string) ([]byte, string, error)
		// SignSECP256K1 signs a message (requires user confirmation)
		// The last byte denotes the SIGN_MODE to be used by Ledger: 0 for
		// LEGACY_AMINO_JSON, 1 for TEXTUAL. It corresponds to the P2 value
		// in https://github.com/cosmos/ledger-cosmos/blob/main/docs/APDUSPEC.md
		SignSECP256K1([]uint32, []byte, byte) ([]byte, error)
	}

	// Options hosts customization options to account for differences in Ledger
	// signing and usage across chains.
	Options struct {
		discoverLedger    discoverLedgerFn
		createPubkey      createPubkeyFn
		appName           string
		skipDERConversion bool
	}

	// PrivKeyLedgerSecp256k1 implements PrivKey, calling the ledger nano we
	// cache the PubKey from the first call to use it later.
	PrivKeyLedgerSecp256k1 struct {
		// CachedPubKey should be private, but we want to encode it via
		// go-amino so we can view the address later, even without having the
		// ledger attached.
		CachedPubKey types.PubKey
		Path         hd.BIP44Params
	}
)

// Initialize the default options values for the Cosmos Ledger
func initOptionsDefault() {
	options.createPubkey = func(key []byte) types.PubKey {
		return &secp256k1.PubKey{Key: key}
	}
	options.appName = "Cosmos"
	options.skipDERConversion = false
}

// SetDiscoverLedger set the discoverLedger function to use a different Ledger derivation
func SetDiscoverLedger(fn discoverLedgerFn) {
	options.discoverLedger = fn
}

// SetCreatePubkey set the createPubkey function to use a different public key
func SetCreatePubkey(fn createPubkeyFn) {
	options.createPubkey = fn
}

// SetAppName set the Ledger app name to use a different app name
func SetAppName(appName string) {
	options.appName = appName
}

// SetSkipDERConversion set the DER Conversion requirement to true (false by default)
func SetSkipDERConversion() {
	options.skipDERConversion = true
}

// NewPrivKeySecp256k1Unsafe will generate a new key and store the public key for later use.
//
// This function is marked as unsafe as it will retrieve a pubkey without user verification.
// It can only be used to verify a pubkey but never to create new accounts/keys. In that case,
// please refer to NewPrivKeySecp256k1
func NewPrivKeySecp256k1Unsafe(path hd.BIP44Params) (types.LedgerPrivKeyAminoJSON, error) {
	device, err := getDevice()
	if err != nil {
		return nil, err
	}
	defer warnIfErrors(device.Close)

	pubKey, err := getPubKeyUnsafe(device, path)
	if err != nil {
		return nil, err
	}

	return PrivKeyLedgerSecp256k1{pubKey, path}, nil
}

// NewPrivKeySecp256k1 will generate a new key and store the public key for later use.
// The request will require user confirmation and will show account and index in the device
func NewPrivKeySecp256k1(path hd.BIP44Params, hrp string) (types.LedgerPrivKey, string, error) {
	device, err := getDevice()
	if err != nil {
		return nil, "", fmt.Errorf("failed to retrieve device: %w", err)
	}
	defer warnIfErrors(device.Close)

	pubKey, addr, err := getPubKeyAddrSafe(device, path, hrp)
	if err != nil {
		return nil, "", fmt.Errorf("failed to recover pubkey: %w", err)
	}

	return PrivKeyLedgerSecp256k1{pubKey, path}, addr, nil
}

// PubKey returns the cached public key.
func (pkl PrivKeyLedgerSecp256k1) PubKey() types.PubKey {
	return pkl.CachedPubKey
}

// Sign returns a secp256k1 signature for the corresponding message using
// SIGN_MODE_TEXTUAL.
func (pkl PrivKeyLedgerSecp256k1) Sign(message []byte) ([]byte, error) {
	device, err := getDevice()
	if err != nil {
		return nil, err
	}
	defer warnIfErrors(device.Close)

	return sign(device, pkl, message, 1)
}

// SignLedgerAminoJSON returns a secp256k1 signature for the corresponding message using
// SIGN_MODE_LEGACY_AMINO_JSON.
func (pkl PrivKeyLedgerSecp256k1) SignLedgerAminoJSON(message []byte) ([]byte, error) {
	device, err := getDevice()
	if err != nil {
		return nil, err
	}
	defer warnIfErrors(device.Close)

	return sign(device, pkl, message, 0)
}

// ShowAddress triggers a ledger device to show the corresponding address.
func ShowAddress(path hd.BIP44Params, expectedPubKey types.PubKey, accountAddressPrefix string) error {
	device, err := getDevice()
	if err != nil {
		return err
	}
	defer warnIfErrors(device.Close)

	pubKey, err := getPubKeyUnsafe(device, path)
	if err != nil {
		return err
	}

	if !pubKey.Equals(expectedPubKey) {
		return errors.New("the key's pubkey does not match with the one retrieved from Ledger. Check that the HD path and device are the correct ones")
	}

	pubKey2, _, err := getPubKeyAddrSafe(device, path, accountAddressPrefix)
	if err != nil {
		return err
	}

	if !pubKey2.Equals(expectedPubKey) {
		return errors.New("the key's pubkey does not match with the one retrieved from Ledger. Check that the HD path and device are the correct ones")
	}

	return nil
}

// ValidateKey allows us to verify the sanity of a public key after loading it
// from disk.
func (pkl PrivKeyLedgerSecp256k1) ValidateKey() error {
	device, err := getDevice()
	if err != nil {
		return err
	}
	defer warnIfErrors(device.Close)

	return validateKey(device, pkl)
}

// AssertIsPrivKeyInner implements the PrivKey interface. It performs a no-op.
func (pkl *PrivKeyLedgerSecp256k1) AssertIsPrivKeyInner() {}

// Bytes implements the PrivKey interface. It stores the cached public key so
// we can verify the same key when we reconnect to a ledger.
func (pkl PrivKeyLedgerSecp256k1) Bytes() []byte {
	return cdc.MustMarshal(pkl)
}

// Equals implements the PrivKey interface. It makes sure two private keys
// refer to the same public key.
func (pkl PrivKeyLedgerSecp256k1) Equals(other types.LedgerPrivKey) bool {
	if otherKey, ok := other.(PrivKeyLedgerSecp256k1); ok {
		return pkl.CachedPubKey.Equals(otherKey.CachedPubKey)
	}
	return false
}

func (pkl PrivKeyLedgerSecp256k1) Type() string { return "PrivKeyLedgerSecp256k1" }

// warnIfErrors wraps a function and writes a warning to stderr. This is required
// to avoid ignoring errors when defer is used. Using defer may result in linter warnings.
func warnIfErrors(f func() error) {
	if err := f(); err != nil {
		_, _ = fmt.Fprint(os.Stderr, "received error when closing ledger connection", err)
	}
}

func convertDERtoBER(signatureDER []byte) ([]byte, error) {
	sigDER, err := ecdsa.ParseDERSignature(signatureDER)
	if err != nil {
		return nil, err
	}

	sigStr := sigDER.Serialize()
	// The format of a DER encoded signature is as follows:
	// 0x30 <total length> 0x02 <length of R> <R> 0x02 <length of S> <S>
	r, s := new(big.Int), new(big.Int)
	r.SetBytes(sigStr[4 : 4+sigStr[3]])
	s.SetBytes(sigStr[4+sigStr[3]+2:])

	sModNScalar := new(secp.ModNScalar)
	sModNScalar.SetByteSlice(s.Bytes())
	// based on https://github.com/tendermint/btcd/blob/ec996c5/btcec/signature.go#L33-L50
	if sModNScalar.IsOverHalfOrder() {
		s = new(big.Int).Sub(secp.S256().N, s)
	}

	sigBytes := make([]byte, 64)
	// 0 pad the byte arrays from the left if they aren't big enough.
	copy(sigBytes[32-len(r.Bytes()):32], r.Bytes())
	copy(sigBytes[64-len(s.Bytes()):64], s.Bytes())

	return sigBytes, nil
}

func getDevice() (SECP256K1, error) {
	if options.discoverLedger == nil {
		return nil, errors.New("no Ledger discovery function defined")
	}

	device, err := options.discoverLedger()
	if err != nil {
		return nil, fmt.Errorf("ledger nano S: %w", err)
	}

	return device, nil
}

func validateKey(device SECP256K1, pkl PrivKeyLedgerSecp256k1) error {
	pub, err := getPubKeyUnsafe(device, pkl.Path)
	if err != nil {
		return err
	}

	// verify this matches cached address
	if !pub.Equals(pkl.CachedPubKey) {
		return errors.New("cached key does not match retrieved key")
	}

	return nil
}

// Sign calls the ledger and stores the PubKey for future use.
//
// Communication is checked on NewPrivKeyLedger and PrivKeyFromBytes, returning
// an error, so this should only trigger if the private key is held in memory
// for a while before use.
//
// Last byte P2 is 0 for LEGACY_AMINO_JSON, and 1 for TEXTUAL.
func sign(device SECP256K1, pkl PrivKeyLedgerSecp256k1, msg []byte, p2 byte) ([]byte, error) {
	err := validateKey(device, pkl)
	if err != nil {
		return nil, err
	}

	sig, err := device.SignSECP256K1(pkl.Path.DerivationPath(), msg, p2)
	if err != nil {
		return nil, err
	}

	if options.skipDERConversion {
		return sig, nil
	}

	return convertDERtoBER(sig)
}

// getPubKeyUnsafe reads the pubkey from a ledger device
//
// This function is marked as unsafe as it will retrieve a pubkey without user verification
// It can only be used to verify a pubkey but never to create new accounts/keys. In that case,
// please refer to getPubKeyAddrSafe
//
// since this involves IO, it may return an error, which is not exposed
// in the PubKey interface, so this function allows better error handling
func getPubKeyUnsafe(device SECP256K1, path hd.BIP44Params) (types.PubKey, error) {
	publicKey, err := device.GetPublicKeySECP256K1(path.DerivationPath())
	if err != nil {
		return nil, fmt.Errorf("please open the %v app on the Ledger device - error: %w", options.appName, err)
	}

	// re-serialize in the 33-byte compressed format
	cmp, err := secec.NewPublicKey(publicKey)
	if err != nil {
		return nil, fmt.Errorf("error parsing public key: %w", err)
	}

	compressedPublicKey := make([]byte, secp256k1.PubKeySize)
	copy(compressedPublicKey, cmp.CompressedBytes())

	return options.createPubkey(compressedPublicKey), nil
}

// getPubKeyAddr reads the pubkey and the address from a ledger device.
// This function is marked as Safe as it will require user confirmation and
// account and index will be shown in the device.
//
// Since this involves IO, it may return an error, which is not exposed
// in the PubKey interface, so this function allows better error handling.
func getPubKeyAddrSafe(device SECP256K1, path hd.BIP44Params, hrp string) (types.PubKey, string, error) {
	publicKey, addr, err := device.GetAddressPubKeySECP256K1(path.DerivationPath(), hrp)
	if err != nil {
		// Check special case if user is trying to use an index > 100
		if path.AddressIndex > 100 {
			return nil, "", fmt.Errorf("%w: cannot derive paths where index > 100: %s "+
				"This is a security measure to avoid very hard to find derivation paths introduced by a possible attacker. "+
				"You can disable this by setting expert mode in your ledger device. Do this at your own risk", err, path)
		}
		return nil, "", fmt.Errorf("%w: address rejected for path %s", err, path)
	}

	// re-serialize in the 33-byte compressed format
	cmp, err := secec.NewPublicKey(publicKey)
	if err != nil {
		return nil, "", fmt.Errorf("error parsing public key: %w", err)
	}

	compressedPublicKey := make([]byte, secp256k1.PubKeySize)
	copy(compressedPublicKey, cmp.CompressedBytes())

	return options.createPubkey(compressedPublicKey), addr, nil
}

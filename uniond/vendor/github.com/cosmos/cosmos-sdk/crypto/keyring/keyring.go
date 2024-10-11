package keyring

import (
	"bufio"
	"encoding/hex"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"github.com/99designs/keyring"
	"github.com/cosmos/go-bip39"
	"golang.org/x/crypto/bcrypt"

	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/client/input"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/crypto"
	"github.com/cosmos/cosmos-sdk/crypto/hd"
	"github.com/cosmos/cosmos-sdk/crypto/ledger"
	"github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
)

// Backend options for Keyring
const (
	BackendFile    = "file"
	BackendOS      = "os"
	BackendKWallet = "kwallet"
	BackendPass    = "pass"
	BackendTest    = "test"
	BackendMemory  = "memory"
)

const (
	keyringFileDirName = "keyring-file"
	keyringTestDirName = "keyring-test"
	passKeyringPrefix  = "keyring-%s"

	// temporary pass phrase for exporting a key during a key rename
	passPhrase = "temp"
	// prefix for exported hex private keys
	hexPrefix = "0x"
)

var (
	_                          Keyring = &keystore{}
	maxPassphraseEntryAttempts         = 3
)

// Keyring exposes operations over a backend supported by github.com/99designs/keyring.
type Keyring interface {
	// Backend get the backend type used in the keyring config: "file", "os", "kwallet", "pass", "test", "memory".
	Backend() string

	// DB get the db keyring used in the keystore.
	DB() keyring.Keyring

	// List all keys.
	List() ([]*Record, error)

	// SupportedAlgorithms supported signing algorithms for Keyring and Ledger respectively.
	SupportedAlgorithms() (SigningAlgoList, SigningAlgoList)

	// Key and KeyByAddress return keys by uid and address respectively.
	Key(uid string) (*Record, error)
	KeyByAddress(address []byte) (*Record, error)

	// Delete and DeleteByAddress remove keys from the keyring.
	Delete(uid string) error
	DeleteByAddress(address []byte) error

	// Rename an existing key from the Keyring
	Rename(from, to string) error

	// NewMnemonic generates a new mnemonic, derives a hierarchical deterministic key from it, and
	// persists the key to storage. Returns the generated mnemonic and the key Info.
	// It returns an error if it fails to generate a key for the given algo type, or if
	// another key is already stored under the same name or address.
	//
	// A passphrase set to the empty string will set the passphrase to the DefaultBIP39Passphrase value.
	NewMnemonic(uid string, language Language, hdPath, bip39Passphrase string, algo SignatureAlgo) (*Record, string, error)

	// NewAccount converts a mnemonic to a private key and BIP-39 HD Path and persists it.
	// It fails if there is an existing key Info with the same address.
	NewAccount(uid, mnemonic, bip39Passphrase, hdPath string, algo SignatureAlgo) (*Record, error)

	// SaveLedgerKey retrieves a public key reference from a Ledger device and persists it.
	SaveLedgerKey(uid string, algo SignatureAlgo, hrp string, coinType, account, index uint32) (*Record, error)

	// SaveOfflineKey stores a public key and returns the persisted Info structure.
	SaveOfflineKey(uid string, pubkey types.PubKey) (*Record, error)

	// SaveMultisig stores and returns a new multsig (offline) key reference.
	SaveMultisig(uid string, pubkey types.PubKey) (*Record, error)

	Signer

	Importer
	Exporter

	Migrator
}

// Signer is implemented by key stores that want to provide signing capabilities.
type Signer interface {
	// Sign sign byte messages with a user key.
	Sign(uid string, msg []byte, signMode signing.SignMode) ([]byte, types.PubKey, error)

	// SignByAddress sign byte messages with a user key providing the address.
	SignByAddress(address, msg []byte, signMode signing.SignMode) ([]byte, types.PubKey, error)
}

// Importer is implemented by key stores that support import of public and private keys.
type Importer interface {
	// ImportPrivKey imports ASCII armored passphrase-encrypted private keys.
	ImportPrivKey(uid, armor, passphrase string) error
	// ImportPrivKeyHex imports hex encoded keys.
	ImportPrivKeyHex(uid, privKey, algoStr string) error
	// ImportPubKey imports ASCII armored public keys.
	ImportPubKey(uid, armor string) error
}

// Migrator is implemented by key stores and enables migration of keys from amino to proto
type Migrator interface {
	MigrateAll() ([]*Record, error)
}

// Exporter is implemented by key stores that support export of public and private keys.
type Exporter interface {
	// ExportPubKeyArmor export public key
	ExportPubKeyArmor(uid string) (string, error)
	ExportPubKeyArmorByAddress(address []byte) (string, error)

	// ExportPrivKeyArmor returns a private key in ASCII armored format.
	// It returns an error if the key does not exist or a wrong encryption passphrase is supplied.
	ExportPrivKeyArmor(uid, encryptPassphrase string) (armor string, err error)
	ExportPrivKeyArmorByAddress(address []byte, encryptPassphrase string) (armor string, err error)
}

// Option overrides keyring configuration options.
type Option func(options *Options)

// NewInMemory creates a transient keyring useful for testing
// purposes and on-the-fly key generation.
// Keybase options can be applied when generating this new Keybase.
func NewInMemory(cdc codec.Codec, opts ...Option) Keyring {
	return NewInMemoryWithKeyring(keyring.NewArrayKeyring(nil), cdc, opts...)
}

// NewInMemoryWithKeyring returns an in memory keyring using the specified keyring.Keyring
// as the backing keyring.
func NewInMemoryWithKeyring(kr keyring.Keyring, cdc codec.Codec, opts ...Option) Keyring {
	return newKeystore(kr, cdc, BackendMemory, opts...)
}

// New creates a new instance of a keyring.
// Keyring options can be applied when generating the new instance.
// Available backends are "os", "file", "kwallet", "memory", "pass", "test".
func newKeyringGeneric(
	appName, backend, rootDir string, userInput io.Reader, cdc codec.Codec, opts ...Option,
) (Keyring, error) {
	var (
		db  keyring.Keyring
		err error
	)

	switch backend {
	case BackendMemory:
		return NewInMemory(cdc, opts...), err
	case BackendTest:
		db, err = keyring.Open(newTestBackendKeyringConfig(appName, rootDir))
	case BackendFile:
		db, err = keyring.Open(newFileBackendKeyringConfig(appName, rootDir, userInput))
	case BackendOS:
		db, err = keyring.Open(newOSBackendKeyringConfig(appName, rootDir, userInput))
	case BackendKWallet:
		db, err = keyring.Open(newKWalletBackendKeyringConfig(appName, rootDir, userInput))
	case BackendPass:
		db, err = keyring.Open(newPassBackendKeyringConfig(appName, rootDir, userInput))
	default:
		return nil, errorsmod.Wrap(ErrUnknownBacked, backend)
	}

	if err != nil {
		return nil, err
	}

	return newKeystore(db, cdc, backend, opts...), nil
}

type keystore struct {
	db      keyring.Keyring
	cdc     codec.Codec
	backend string
	options Options
}

func newKeystore(kr keyring.Keyring, cdc codec.Codec, backend string, opts ...Option) keystore {
	// Default options for keybase, these can be overwritten using the
	// Option function
	options := Options{
		SupportedAlgos:       SigningAlgoList{hd.Secp256k1},
		SupportedAlgosLedger: SigningAlgoList{hd.Secp256k1},
	}

	for _, optionFn := range opts {
		optionFn(&options)
	}

	if options.LedgerDerivation != nil {
		ledger.SetDiscoverLedger(options.LedgerDerivation)
	}

	if options.LedgerCreateKey != nil {
		ledger.SetCreatePubkey(options.LedgerCreateKey)
	}

	if options.LedgerAppName != "" {
		ledger.SetAppName(options.LedgerAppName)
	}

	if options.LedgerSigSkipDERConv {
		ledger.SetSkipDERConversion()
	}

	return keystore{
		db:      kr,
		cdc:     cdc,
		backend: backend,
		options: options,
	}
}

// Backend returns the keyring backend option used in the config
func (ks keystore) Backend() string {
	return ks.backend
}

// DB returns the db keyring used in the keystore
func (ks keystore) DB() keyring.Keyring {
	return ks.db
}

func (ks keystore) ExportPubKeyArmor(uid string) (string, error) {
	k, err := ks.Key(uid)
	if err != nil {
		return "", err
	}

	key, err := k.GetPubKey()
	if err != nil {
		return "", err
	}

	bz, err := ks.cdc.MarshalInterface(key)
	if err != nil {
		return "", err
	}

	return crypto.ArmorPubKeyBytes(bz, key.Type()), nil
}

func (ks keystore) ExportPubKeyArmorByAddress(address []byte) (string, error) {
	k, err := ks.KeyByAddress(address)
	if err != nil {
		return "", err
	}

	return ks.ExportPubKeyArmor(k.Name)
}

// ExportPrivKeyArmor exports encrypted privKey
func (ks keystore) ExportPrivKeyArmor(uid, encryptPassphrase string) (armor string, err error) {
	priv, err := ks.ExportPrivateKeyObject(uid)
	if err != nil {
		return "", err
	}

	return crypto.EncryptArmorPrivKey(priv, encryptPassphrase, priv.Type()), nil
}

// ExportPrivateKeyObject exports an armored private key object.
func (ks keystore) ExportPrivateKeyObject(uid string) (types.PrivKey, error) {
	k, err := ks.Key(uid)
	if err != nil {
		return nil, err
	}

	priv, err := extractPrivKeyFromRecord(k)
	if err != nil {
		return nil, err
	}

	return priv, err
}

func (ks keystore) ExportPrivKeyArmorByAddress(address []byte, encryptPassphrase string) (armor string, err error) {
	k, err := ks.KeyByAddress(address)
	if err != nil {
		return "", err
	}

	return ks.ExportPrivKeyArmor(k.Name, encryptPassphrase)
}

func (ks keystore) ImportPrivKey(uid, armor, passphrase string) error {
	if k, err := ks.Key(uid); err == nil {
		if uid == k.Name {
			return errorsmod.Wrap(ErrOverwriteKey, uid)
		}
	}

	privKey, _, err := crypto.UnarmorDecryptPrivKey(armor, passphrase)
	if err != nil {
		return errorsmod.Wrap(err, "failed to decrypt private key")
	}

	_, err = ks.writeLocalKey(uid, privKey)
	if err != nil {
		return err
	}

	return nil
}

func (ks keystore) ImportPrivKeyHex(uid, privKey, algoStr string) error {
	if _, err := ks.Key(uid); err == nil {
		return errorsmod.Wrap(ErrOverwriteKey, uid)
	}
	if privKey[:2] == hexPrefix {
		privKey = privKey[2:]
	}
	decodedPriv, err := hex.DecodeString(privKey)
	if err != nil {
		return err
	}
	algo, err := NewSigningAlgoFromString(algoStr, ks.options.SupportedAlgos)
	if err != nil {
		return err
	}
	priv := algo.Generate()(decodedPriv)
	_, err = ks.writeLocalKey(uid, priv)
	if err != nil {
		return err
	}
	return nil
}

func (ks keystore) ImportPubKey(uid, armor string) error {
	if _, err := ks.Key(uid); err == nil {
		return errorsmod.Wrap(ErrOverwriteKey, uid)
	}

	pubBytes, _, err := crypto.UnarmorPubKeyBytes(armor)
	if err != nil {
		return err
	}

	var pubKey types.PubKey
	if err := ks.cdc.UnmarshalInterface(pubBytes, &pubKey); err != nil {
		return err
	}

	_, err = ks.writeOfflineKey(uid, pubKey)
	if err != nil {
		return err
	}

	return nil
}

func (ks keystore) Sign(uid string, msg []byte, signMode signing.SignMode) ([]byte, types.PubKey, error) {
	k, err := ks.Key(uid)
	if err != nil {
		return nil, nil, err
	}

	switch {
	case k.GetLocal() != nil:
		priv, err := extractPrivKeyFromLocal(k.GetLocal())
		if err != nil {
			return nil, nil, err
		}

		sig, err := priv.Sign(msg)
		if err != nil {
			return nil, nil, err
		}

		return sig, priv.PubKey(), nil

	case k.GetLedger() != nil:
		return SignWithLedger(k, msg, signMode)

		// multi or offline record
	default:
		pub, err := k.GetPubKey()
		if err != nil {
			return nil, nil, err
		}
		return nil, pub, ErrOfflineSign
	}
}

func (ks keystore) SignByAddress(address, msg []byte, signMode signing.SignMode) ([]byte, types.PubKey, error) {
	k, err := ks.KeyByAddress(address)
	if err != nil {
		return nil, nil, err
	}

	return ks.Sign(k.Name, msg, signMode)
}

func (ks keystore) SaveLedgerKey(uid string, algo SignatureAlgo, hrp string, coinType, account, index uint32) (*Record, error) {
	if !ks.options.SupportedAlgosLedger.Contains(algo) {
		return nil, errorsmod.Wrap(ErrUnsupportedSigningAlgo, fmt.Sprintf("signature algo %s is not defined in the keyring options", algo.Name()))
	}

	hdPath := hd.NewFundraiserParams(account, coinType, index)

	priv, _, err := ledger.NewPrivKeySecp256k1(*hdPath, hrp)
	if err != nil {
		return nil, errorsmod.Wrap(ErrLedgerGenerateKey, err.Error())
	}

	return ks.writeLedgerKey(uid, priv.PubKey(), hdPath)
}

func (ks keystore) writeLedgerKey(name string, pk types.PubKey, path *hd.BIP44Params) (*Record, error) {
	k, err := NewLedgerRecord(name, pk, path)
	if err != nil {
		return nil, err
	}

	return k, ks.writeRecord(k)
}

func (ks keystore) SaveMultisig(uid string, pubkey types.PubKey) (*Record, error) {
	return ks.writeMultisigKey(uid, pubkey)
}

func (ks keystore) SaveOfflineKey(uid string, pubkey types.PubKey) (*Record, error) {
	return ks.writeOfflineKey(uid, pubkey)
}

func (ks keystore) DeleteByAddress(address []byte) error {
	k, err := ks.KeyByAddress(address)
	if err != nil {
		return err
	}

	err = ks.Delete(k.Name)
	if err != nil {
		return err
	}

	return nil
}

func (ks keystore) Rename(oldName, newName string) error {
	_, err := ks.Key(newName)
	if err == nil {
		return errorsmod.Wrap(ErrKeyAlreadyExists, fmt.Sprintf("rename failed, %s", newName))
	}

	armor, err := ks.ExportPrivKeyArmor(oldName, passPhrase)
	if err != nil {
		return err
	}

	if err := ks.Delete(oldName); err != nil {
		return err
	}

	if err := ks.ImportPrivKey(newName, armor, passPhrase); err != nil {
		return err
	}

	return nil
}

// Delete deletes a key in the keyring. `uid` represents the key name, without
// the `.info` suffix.
func (ks keystore) Delete(uid string) error {
	k, err := ks.Key(uid)
	if err != nil {
		return err
	}

	addr, err := k.GetAddress()
	if err != nil {
		return err
	}

	err = ks.db.Remove(addrHexKeyAsString(addr))
	if err != nil {
		return err
	}

	err = ks.db.Remove(infoKey(uid))
	if err != nil {
		return err
	}

	return nil
}

func (ks keystore) KeyByAddress(address []byte) (*Record, error) {
	ik, err := ks.db.Get(addrHexKeyAsString(address))
	if err != nil {
		return nil, wrapKeyNotFound(err, "key with given address not found") // we do not print the address for not needing an address codec
	}

	if len(ik.Data) == 0 {
		return nil, wrapKeyNotFound(err, "key with given address not found") // we do not print the address for not needing an address codec
	}

	return ks.Key(string(ik.Data))
}

func wrapKeyNotFound(err error, msg string) error {
	if errors.Is(err, keyring.ErrKeyNotFound) {
		return errorsmod.Wrap(sdkerrors.ErrKeyNotFound, msg)
	}
	return err
}

func (ks keystore) List() ([]*Record, error) {
	return ks.MigrateAll()
}

func (ks keystore) NewMnemonic(uid string, language Language, hdPath, bip39Passphrase string, algo SignatureAlgo) (*Record, string, error) {
	if language != English {
		return nil, "", ErrUnsupportedLanguage
	}

	if !ks.isSupportedSigningAlgo(algo) {
		return nil, "", ErrUnsupportedSigningAlgo
	}

	// Default number of words (24): This generates a mnemonic directly from the
	// number of words by reading system entropy.
	entropy, err := bip39.NewEntropy(defaultEntropySize)
	if err != nil {
		return nil, "", err
	}

	mnemonic, err := bip39.NewMnemonic(entropy)
	if err != nil {
		return nil, "", err
	}

	if bip39Passphrase == "" {
		bip39Passphrase = DefaultBIP39Passphrase
	}

	k, err := ks.NewAccount(uid, mnemonic, bip39Passphrase, hdPath, algo)
	if err != nil {
		return nil, "", err
	}

	return k, mnemonic, nil
}

func (ks keystore) NewAccount(name, mnemonic, bip39Passphrase, hdPath string, algo SignatureAlgo) (*Record, error) {
	if !ks.isSupportedSigningAlgo(algo) {
		return nil, ErrUnsupportedSigningAlgo
	}

	// create master key and derive first key for keyring
	derivedPriv, err := algo.Derive()(mnemonic, bip39Passphrase, hdPath)
	if err != nil {
		return nil, err
	}

	privKey := algo.Generate()(derivedPriv)

	// check if the key already exists with the same address and return an error
	// if found
	address := sdk.AccAddress(privKey.PubKey().Address())
	if _, err := ks.KeyByAddress(address); err == nil {
		return nil, ErrDuplicatedAddress
	}

	return ks.writeLocalKey(name, privKey)
}

func (ks keystore) isSupportedSigningAlgo(algo SignatureAlgo) bool {
	return ks.options.SupportedAlgos.Contains(algo)
}

func (ks keystore) Key(uid string) (*Record, error) {
	k, err := ks.migrate(uid)
	if err != nil {
		return nil, err
	}

	return k, nil
}

// SupportedAlgorithms returns the keystore Options' supported signing algorithm.
// for the keyring and Ledger.
func (ks keystore) SupportedAlgorithms() (SigningAlgoList, SigningAlgoList) {
	return ks.options.SupportedAlgos, ks.options.SupportedAlgosLedger
}

// SignWithLedger signs a binary message with the ledger device referenced by an Info object
// and returns the signed bytes and the public key. It returns an error if the device could
// not be queried or it returned an error.
func SignWithLedger(k *Record, msg []byte, signMode signing.SignMode) (sig []byte, pub types.PubKey, err error) {
	ledgerInfo := k.GetLedger()
	if ledgerInfo == nil {
		return nil, nil, ErrNotLedgerObj
	}

	path := ledgerInfo.GetPath()

	priv, err := ledger.NewPrivKeySecp256k1Unsafe(*path)
	if err != nil {
		return nil, nil, err
	}
	ledgerPubKey := priv.PubKey()
	pubKey, err := k.GetPubKey()
	if err != nil {
		return nil, nil, err
	}
	if !pubKey.Equals(ledgerPubKey) {
		return nil, nil, fmt.Errorf("the public key that the user attempted to sign with does not match the public key on the ledger device. %v does not match %v", pubKey.String(), ledgerPubKey.String())
	}

	switch signMode {
	case signing.SignMode_SIGN_MODE_TEXTUAL:
		sig, err = priv.Sign(msg)
		if err != nil {
			return nil, nil, err
		}
	case signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON:
		sig, err = priv.SignLedgerAminoJSON(msg)
		if err != nil {
			return nil, nil, err
		}
	default:
		return nil, nil, errorsmod.Wrap(ErrInvalidSignMode, fmt.Sprintf("%v", signMode))
	}

	if !priv.PubKey().VerifySignature(msg, sig) {
		return nil, nil, ErrLedgerInvalidSignature
	}

	return sig, priv.PubKey(), nil
}

func newOSBackendKeyringConfig(appName, dir string, buf io.Reader) keyring.Config {
	return keyring.Config{
		ServiceName:              appName,
		FileDir:                  dir,
		KeychainTrustApplication: true,
		FilePasswordFunc:         newRealPrompt(dir, buf),
	}
}

func newTestBackendKeyringConfig(appName, dir string) keyring.Config {
	return keyring.Config{
		AllowedBackends: []keyring.BackendType{keyring.FileBackend},
		ServiceName:     appName,
		FileDir:         filepath.Join(dir, keyringTestDirName),
		FilePasswordFunc: func(_ string) (string, error) {
			return "test", nil
		},
	}
}

func newKWalletBackendKeyringConfig(appName, _ string, _ io.Reader) keyring.Config {
	return keyring.Config{
		AllowedBackends: []keyring.BackendType{keyring.KWalletBackend},
		ServiceName:     "kdewallet",
		KWalletAppID:    appName,
		KWalletFolder:   "",
	}
}

func newPassBackendKeyringConfig(appName, _ string, _ io.Reader) keyring.Config {
	prefix := fmt.Sprintf(passKeyringPrefix, appName)

	return keyring.Config{
		AllowedBackends: []keyring.BackendType{keyring.PassBackend},
		ServiceName:     appName,
		PassPrefix:      prefix,
	}
}

func newFileBackendKeyringConfig(name, dir string, buf io.Reader) keyring.Config {
	fileDir := filepath.Join(dir, keyringFileDirName)

	return keyring.Config{
		AllowedBackends:  []keyring.BackendType{keyring.FileBackend},
		ServiceName:      name,
		FileDir:          fileDir,
		FilePasswordFunc: newRealPrompt(fileDir, buf),
	}
}

func newRealPrompt(dir string, buf io.Reader) func(string) (string, error) {
	return func(prompt string) (string, error) {
		keyhashStored := false
		keyhashFilePath := filepath.Join(dir, "keyhash")

		var keyhash []byte

		_, err := os.Stat(keyhashFilePath)

		switch {
		case err == nil:
			keyhash, err = os.ReadFile(keyhashFilePath)
			if err != nil {
				return "", errorsmod.Wrap(err, fmt.Sprintf("failed to read %s", keyhashFilePath))
			}

			keyhashStored = true

		case os.IsNotExist(err):
			keyhashStored = false

		default:
			return "", errorsmod.Wrap(err, fmt.Sprintf("failed to open %s", keyhashFilePath))
		}

		failureCounter := 0

		for {
			failureCounter++
			if failureCounter > maxPassphraseEntryAttempts {
				return "", ErrMaxPassPhraseAttempts
			}

			buf := bufio.NewReader(buf)
			pass, err := input.GetPassword(fmt.Sprintf("Enter keyring passphrase (attempt %d/%d):", failureCounter, maxPassphraseEntryAttempts), buf)
			if err != nil {
				// NOTE: LGTM.io reports a false positive alert that states we are printing the password,
				// but we only log the error.
				//
				// lgtm [go/clear-text-logging]
				fmt.Fprintln(os.Stderr, err)
				continue
			}

			if keyhashStored {
				if err := bcrypt.CompareHashAndPassword(keyhash, []byte(pass)); err != nil {
					fmt.Fprintln(os.Stderr, "incorrect passphrase")
					continue
				}

				return pass, nil
			}

			reEnteredPass, err := input.GetPassword("Re-enter keyring passphrase:", buf)
			if err != nil {
				// NOTE: LGTM.io reports a false positive alert that states we are printing the password,
				// but we only log the error.
				//
				// lgtm [go/clear-text-logging]
				fmt.Fprintln(os.Stderr, err)
				continue
			}

			if pass != reEnteredPass {
				fmt.Fprintln(os.Stderr, "passphrase do not match")
				continue
			}

			passwordHash, err := bcrypt.GenerateFromPassword([]byte(pass), 2)
			if err != nil {
				fmt.Fprintln(os.Stderr, err)
				continue
			}

			if err := os.WriteFile(keyhashFilePath, passwordHash, 0o600); err != nil {
				return "", err
			}

			return pass, nil
		}
	}
}

func (ks keystore) writeLocalKey(name string, privKey types.PrivKey) (*Record, error) {
	k, err := NewLocalRecord(name, privKey, privKey.PubKey())
	if err != nil {
		return nil, err
	}

	return k, ks.writeRecord(k)
}

// writeRecord persists a keyring item in keystore if it does not exist there.
// For each key record, we actually write 2 items:
// - one with key `<uid>.info`, with Data = the serialized protobuf key
// - another with key `<addr_as_hex>.address`, with Data = the uid (i.e. the key name)
// This is to be able to query keys both by name and by address.
func (ks keystore) writeRecord(k *Record) error {
	addr, err := k.GetAddress()
	if err != nil {
		return err
	}

	key := infoKey(k.Name)

	exists, err := ks.existsInDb(addr, key)
	if err != nil {
		return err
	}
	if exists {
		return errorsmod.Wrap(ErrKeyAlreadyExists, key)
	}

	serializedRecord, err := ks.cdc.Marshal(k)
	if err != nil {
		return errorsmod.Wrap(ErrUnableToSerialize, err.Error())
	}

	item := keyring.Item{
		Key:  key,
		Data: serializedRecord,
	}

	if err := ks.SetItem(item); err != nil {
		return err
	}

	item = keyring.Item{
		Key:  addrHexKeyAsString(addr),
		Data: []byte(key),
	}

	if err := ks.SetItem(item); err != nil {
		return err
	}

	return nil
}

// existsInDb returns (true, nil) if either addr or name exist is in keystore DB.
// On the other hand, it returns (false, error) if Get method returns error different from keyring.ErrKeyNotFound
// In case of inconsistent keyring, it recovers it automatically.
func (ks keystore) existsInDb(addr []byte, name string) (bool, error) {
	_, errAddr := ks.db.Get(addrHexKeyAsString(addr))
	if errAddr != nil && !errors.Is(errAddr, keyring.ErrKeyNotFound) {
		return false, errAddr
	}

	_, errInfo := ks.db.Get(infoKey(name))
	if errInfo == nil {
		return true, nil // uid lookup succeeds - info exists
	} else if !errors.Is(errInfo, keyring.ErrKeyNotFound) {
		return false, errInfo // received unexpected error - returns
	}

	// looking for an issue, record with meta (getByAddress) exists, but record with public key itself does not
	if errAddr == nil && errors.Is(errInfo, keyring.ErrKeyNotFound) {
		fmt.Fprintf(os.Stderr, "address \"%s\" exists but pubkey itself does not\n", hex.EncodeToString(addr))
		fmt.Fprintln(os.Stderr, "recreating pubkey record")
		err := ks.db.Remove(addrHexKeyAsString(addr))
		if err != nil {
			return true, err
		}
		return false, nil
	}

	// both lookups failed, info does not exist
	return false, nil
}

func (ks keystore) writeOfflineKey(name string, pk types.PubKey) (*Record, error) {
	k, err := NewOfflineRecord(name, pk)
	if err != nil {
		return nil, err
	}

	return k, ks.writeRecord(k)
}

// writeMultisigKey investigate where thisf function is called maybe remove it
func (ks keystore) writeMultisigKey(name string, pk types.PubKey) (*Record, error) {
	k, err := NewMultiRecord(name, pk)
	if err != nil {
		return nil, err
	}

	return k, ks.writeRecord(k)
}

func (ks keystore) MigrateAll() ([]*Record, error) {
	keys, err := ks.db.Keys()
	if err != nil {
		return nil, err
	}

	if len(keys) == 0 {
		return nil, nil
	}

	sort.Strings(keys)
	var recs []*Record
	for _, key := range keys {
		// The keyring items only with `.info` consists the key info.
		if !strings.HasSuffix(key, infoSuffix) {
			continue
		}

		rec, err := ks.migrate(key)
		if err != nil {
			fmt.Fprintf(os.Stderr, "migrate err for key %s: %q\n", key, err)
			continue
		}

		recs = append(recs, rec)
	}

	return recs, nil
}

// migrate converts keyring.Item from amino to proto serialization format.
// the `key` argument can be a key uid (e.g. "alice") or with the '.info'
// suffix (e.g. "alice.info").
//
// It operates as follows:
// 1. retrieve any key
// 2. try to decode it using protobuf
// 3. if ok, then return the key, do nothing else
// 4. if it fails, then try to decode it using amino
// 5. convert from the amino struct to the protobuf struct
// 6. write the proto-encoded key back to the keyring
func (ks keystore) migrate(key string) (*Record, error) {
	if !strings.HasSuffix(key, infoSuffix) {
		key = infoKey(key)
	}

	// 1. get the key.
	item, err := ks.db.Get(key)
	if err != nil {
		if key == fmt.Sprintf(".%s", infoSuffix) {
			return nil, errors.New("no key name or address provided; have you forgotten the --from flag?")
		}

		return nil, wrapKeyNotFound(err, key)
	}

	if len(item.Data) == 0 {
		return nil, errorsmod.Wrap(sdkerrors.ErrKeyNotFound, key)
	}

	// 2. Try to deserialize using proto
	k, err := ks.protoUnmarshalRecord(item.Data)
	// 3. If ok then return the key
	if err == nil {
		return k, nil
	}

	// 4. Try to decode with amino
	legacyInfo, err := unMarshalLegacyInfo(item.Data)
	if err != nil {
		return nil, errorsmod.Wrap(err, "unable to unmarshal item.Data")
	}

	// 5. Convert and serialize info using proto
	k, err = ks.convertFromLegacyInfo(legacyInfo)
	if err != nil {
		return nil, errorsmod.Wrap(err, "convertFromLegacyInfo")
	}

	serializedRecord, err := ks.cdc.Marshal(k)
	if err != nil {
		return nil, errorsmod.Wrap(ErrUnableToSerialize, err.Error())
	}

	item = keyring.Item{
		Key:  key,
		Data: serializedRecord,
	}

	// 6. Overwrite the keyring entry with the new proto-encoded key.
	if err := ks.SetItem(item); err != nil {
		return nil, errorsmod.Wrap(err, "unable to set keyring.Item")
	}

	fmt.Fprintf(os.Stderr, "Successfully migrated key %s.\n", key)

	return k, nil
}

func (ks keystore) protoUnmarshalRecord(bz []byte) (*Record, error) {
	k := new(Record)
	if err := ks.cdc.Unmarshal(bz, k); err != nil {
		return nil, err
	}

	return k, nil
}

func (ks keystore) SetItem(item keyring.Item) error {
	return ks.db.Set(item)
}

func (ks keystore) convertFromLegacyInfo(info LegacyInfo) (*Record, error) {
	if info == nil {
		return nil, errorsmod.Wrap(ErrLegacyToRecord, "info is nil")
	}

	name := info.GetName()
	pk := info.GetPubKey()

	switch info.GetType() {
	case TypeLocal:
		priv, err := privKeyFromLegacyInfo(info)
		if err != nil {
			return nil, err
		}

		return NewLocalRecord(name, priv, pk)
	case TypeOffline:
		return NewOfflineRecord(name, pk)
	case TypeMulti:
		return NewMultiRecord(name, pk)
	case TypeLedger:
		path, err := info.GetPath()
		if err != nil {
			return nil, err
		}

		return NewLedgerRecord(name, pk, path)
	default:
		return nil, ErrUnknownLegacyType

	}
}

func addrHexKeyAsString(address []byte) string {
	return fmt.Sprintf("%s.%s", hex.EncodeToString(address), addressSuffix)
}

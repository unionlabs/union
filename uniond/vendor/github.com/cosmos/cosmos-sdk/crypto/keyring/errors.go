package keyring

import "github.com/cockroachdb/errors"

var (
	// ErrUnsupportedSigningAlgo is raised when the caller tries to use a
	// different signing scheme than secp256k1.
	ErrUnsupportedSigningAlgo = errors.New("unsupported signing algo")
	// ErrUnsupportedLanguage is raised when the caller tries to use a
	// different language than english for creating a mnemonic sentence.
	ErrUnsupportedLanguage = errors.New("unsupported language: only english is supported")
	// ErrUnknownBacked is raised when the keyring backend is unknown
	ErrUnknownBacked = errors.New("unknown keyring backend")
	// ErrOverwriteKey is raised when a key cannot be overwritten
	ErrOverwriteKey = errors.New("cannot overwrite key")
	// ErrKeyAlreadyExists is raised when creating a key that already exists
	ErrKeyAlreadyExists = errors.Newf("key already exists")
	// ErrInvalidSignMode is raised when trying to sign with an invaled method
	ErrInvalidSignMode = errors.New("invalid sign mode, expected LEGACY_AMINO_JSON or TEXTUAL")
	// ErrMaxPassPhraseAttempts is raised when the maxPassphraseEntryAttempts is reached
	ErrMaxPassPhraseAttempts = errors.New("too many failed passphrase attempts")
	// ErrUnableToSerialize is raised when codec fails to serialize
	ErrUnableToSerialize = errors.New("unable to serialize record")
	// ErrOfflineSign is raised when trying to sign offline record.
	ErrOfflineSign = errors.New("cannot sign with offline keys")
	// ErrDuplicatedAddress is raised when creating a key with the same address as a key that already exists.
	ErrDuplicatedAddress = errors.New("duplicated address created")
	// ErrLedgerGenerateKey is raised when a ledger can't generate a key
	ErrLedgerGenerateKey = errors.New("failed to generate ledger key")
	// ErrNotLedgerObj is raised when record.GetLedger() returns nil.
	ErrNotLedgerObj = errors.New("not a ledger object")
	// ErrLedgerInvalidSignature is raised when ledger generates an invalid signature.
	ErrLedgerInvalidSignature = errors.New("Ledger generated an invalid signature. Perhaps you have multiple ledgers and need to try another one")
	// ErrLegacyToRecord is raised when cannot be converted to a Record
	ErrLegacyToRecord = errors.New("unable to convert LegacyInfo to Record")
	// ErrUnknownLegacyType is raised when a LegacyInfo type is unknown.
	ErrUnknownLegacyType = errors.New("unknown LegacyInfo type")
)

// Package keyring provides a uniform API over a range of desktop credential storage engines.
package keyring

import (
	"errors"
	"log"
	"time"
)

// BackendType is an identifier for a credential storage service.
type BackendType string

// All currently supported secure storage backends.
const (
	InvalidBackend       BackendType = ""
	SecretServiceBackend BackendType = "secret-service"
	KeychainBackend      BackendType = "keychain"
	KeyCtlBackend        BackendType = "keyctl"
	KWalletBackend       BackendType = "kwallet"
	WinCredBackend       BackendType = "wincred"
	FileBackend          BackendType = "file"
	PassBackend          BackendType = "pass"
)

// This order makes sure the OS-specific backends
// are picked over the more generic backends.
var backendOrder = []BackendType{
	// Windows
	WinCredBackend,
	// MacOS
	KeychainBackend,
	// Linux
	SecretServiceBackend,
	KWalletBackend,
	KeyCtlBackend,
	// General
	PassBackend,
	FileBackend,
}

var supportedBackends = map[BackendType]opener{}

// AvailableBackends provides a slice of all available backend keys on the current OS.
func AvailableBackends() []BackendType {
	b := []BackendType{}
	for _, k := range backendOrder {
		_, ok := supportedBackends[k]
		if ok {
			b = append(b, k)
		}
	}
	return b
}

type opener func(cfg Config) (Keyring, error)

// Open will open a specific keyring backend.
func Open(cfg Config) (Keyring, error) {
	if cfg.AllowedBackends == nil {
		cfg.AllowedBackends = AvailableBackends()
	}
	debugf("Considering backends: %v", cfg.AllowedBackends)
	for _, backend := range cfg.AllowedBackends {
		if opener, ok := supportedBackends[backend]; ok {
			openBackend, err := opener(cfg)
			if err != nil {
				debugf("Failed backend %s: %s", backend, err)
				continue
			}
			return openBackend, nil
		}
	}
	return nil, ErrNoAvailImpl
}

// Item is a thing stored on the keyring.
type Item struct {
	Key         string
	Data        []byte
	Label       string
	Description string

	// Backend specific config
	KeychainNotTrustApplication bool
	KeychainNotSynchronizable   bool
}

// Metadata is information about a thing stored on the keyring; retrieving
// metadata must not require authentication.  The embedded Item should be
// filled in with an empty Data field.
// It's allowed for Item to be a nil pointer, indicating that all we
// have is the timestamps.
type Metadata struct {
	*Item
	ModificationTime time.Time
}

// Keyring provides the uniform interface over the underlying backends.
type Keyring interface {
	// Returns an Item matching the key or ErrKeyNotFound
	Get(key string) (Item, error)
	// Returns the non-secret parts of an Item
	GetMetadata(key string) (Metadata, error)
	// Stores an Item on the keyring
	Set(item Item) error
	// Removes the item with matching key
	Remove(key string) error
	// Provides a slice of all keys stored on the keyring
	Keys() ([]string, error)
}

// ErrNoAvailImpl is returned by Open when a backend cannot be found.
var ErrNoAvailImpl = errors.New("Specified keyring backend not available")

// ErrKeyNotFound is returned by Keyring Get when the item is not on the keyring.
var ErrKeyNotFound = errors.New("The specified item could not be found in the keyring")

// ErrMetadataNeedsCredentials is returned when Metadata is called against a
// backend which requires credentials even to see metadata.
var ErrMetadataNeedsCredentials = errors.New("The keyring backend requires credentials for metadata access")

// ErrMetadataNotSupported is returned when Metadata is not available for the backend.
var ErrMetadataNotSupported = errors.New("The keyring backend does not support metadata access")

var (
	// Debug specifies whether to print debugging output.
	Debug bool
)

func debugf(pattern string, args ...interface{}) {
	if Debug {
		log.Printf("[keyring] "+pattern, args...)
	}
}

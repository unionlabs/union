//go:build windows
// +build windows

package keyring

import (
	"strings"
	"syscall"

	"github.com/danieljoos/wincred"
)

// ERROR_NOT_FOUND from https://docs.microsoft.com/en-us/windows/win32/debug/system-error-codes--1000-1299-
const elementNotFoundError = syscall.Errno(1168)

type windowsKeyring struct {
	name   string
	prefix string
}

func init() {
	supportedBackends[WinCredBackend] = opener(func(cfg Config) (Keyring, error) {
		name := cfg.ServiceName
		if name == "" {
			name = "default"
		}

		prefix := cfg.WinCredPrefix
		if prefix == "" {
			prefix = "keyring"
		}

		return &windowsKeyring{
			name:   name,
			prefix: prefix,
		}, nil
	})
}

func (k *windowsKeyring) Get(key string) (Item, error) {
	cred, err := wincred.GetGenericCredential(k.credentialName(key))
	if err != nil {
		if err == elementNotFoundError {
			return Item{}, ErrKeyNotFound
		}
		return Item{}, err
	}

	item := Item{
		Key:  key,
		Data: cred.CredentialBlob,
	}

	return item, nil
}

// GetMetadata for pass returns an error indicating that it's unsupported
// for this backend.
// TODO: This is a stub. Look into whether pass would support metadata in a usable way for keyring.
func (k *windowsKeyring) GetMetadata(_ string) (Metadata, error) {
	return Metadata{}, ErrMetadataNotSupported
}

func (k *windowsKeyring) Set(item Item) error {
	cred := wincred.NewGenericCredential(k.credentialName(item.Key))
	cred.CredentialBlob = item.Data
	return cred.Write()
}

func (k *windowsKeyring) Remove(key string) error {
	cred, err := wincred.GetGenericCredential(k.credentialName(key))
	if err != nil {
		if err == elementNotFoundError {
			return ErrKeyNotFound
		}
		return err
	}
	return cred.Delete()
}

func (k *windowsKeyring) Keys() ([]string, error) {
	results := []string{}

	if creds, err := wincred.List(); err == nil {
		for _, cred := range creds {
			prefix := k.credentialName("")
			if strings.HasPrefix(cred.TargetName, prefix) {
				results = append(results, strings.TrimPrefix(cred.TargetName, prefix))
			}
		}
	}

	return results, nil
}

func (k *windowsKeyring) credentialName(key string) string {
	return k.prefix + ":" + k.name + ":" + key
}

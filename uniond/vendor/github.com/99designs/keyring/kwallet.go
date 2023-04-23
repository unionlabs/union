//go:build linux
// +build linux

package keyring

import (
	"encoding/json"
	"os"

	"github.com/godbus/dbus"
)

const (
	dbusServiceName = "org.kde.kwalletd5"
	dbusPath        = "/modules/kwalletd5"
)

func init() {
	if os.Getenv("DISABLE_KWALLET") == "1" {
		return
	}

	// silently fail if dbus isn't available
	_, err := dbus.SessionBus()
	if err != nil {
		return
	}

	supportedBackends[KWalletBackend] = opener(func(cfg Config) (Keyring, error) {
		if cfg.ServiceName == "" {
			cfg.ServiceName = "kdewallet"
		}

		if cfg.KWalletAppID == "" {
			cfg.KWalletAppID = "keyring"
		}

		if cfg.KWalletFolder == "" {
			cfg.KWalletFolder = "keyring"
		}

		wallet, err := newKwallet()
		if err != nil {
			return nil, err
		}

		ring := &kwalletKeyring{
			wallet: *wallet,
			name:   cfg.ServiceName,
			appID:  cfg.KWalletAppID,
			folder: cfg.KWalletFolder,
		}

		return ring, ring.openWallet()
	})
}

type kwalletKeyring struct {
	wallet kwalletBinding
	name   string
	handle int32
	appID  string
	folder string
}

func (k *kwalletKeyring) openWallet() error {
	isOpen, err := k.wallet.IsOpen(k.handle)
	if err != nil {
		return err
	}

	if !isOpen {
		handle, err := k.wallet.Open(k.name, 0, k.appID)
		if err != nil {
			return err
		}
		k.handle = handle
	}

	return nil
}

func (k *kwalletKeyring) Get(key string) (Item, error) {
	err := k.openWallet()
	if err != nil {
		return Item{}, err
	}

	data, err := k.wallet.ReadEntry(k.handle, k.folder, key, k.appID)
	if err != nil {
		return Item{}, err
	}
	if len(data) == 0 {
		return Item{}, ErrKeyNotFound
	}

	item := Item{}
	err = json.Unmarshal(data, &item)
	if err != nil {
		return Item{}, err
	}

	return item, nil
}

// GetMetadata for kwallet returns an error indicating that it's unsupported
// for this backend.
//
// The only APIs found around KWallet are for retrieving content, no indication
// found in docs for methods to use to retrieve metadata without needing unlock
// credentials.
func (k *kwalletKeyring) GetMetadata(_ string) (Metadata, error) {
	return Metadata{}, ErrMetadataNeedsCredentials
}

func (k *kwalletKeyring) Set(item Item) error {
	err := k.openWallet()
	if err != nil {
		return err
	}

	data, err := json.Marshal(item)
	if err != nil {
		return err
	}

	err = k.wallet.WriteEntry(k.handle, k.folder, item.Key, data, k.appID)
	if err != nil {
		return err
	}

	return nil
}

func (k *kwalletKeyring) Remove(key string) error {
	err := k.openWallet()
	if err != nil {
		return err
	}

	err = k.wallet.RemoveEntry(k.handle, k.folder, key, k.appID)
	if err != nil {
		return err
	}

	return nil
}

func (k *kwalletKeyring) Keys() ([]string, error) {
	err := k.openWallet()
	if err != nil {
		return []string{}, err
	}

	entries, err := k.wallet.EntryList(k.handle, k.folder, k.appID)
	if err != nil {
		return []string{}, err
	}

	return entries, nil
}

func newKwallet() (*kwalletBinding, error) {
	conn, err := dbus.SessionBus()
	if err != nil {
		return nil, err
	}

	return &kwalletBinding{
		conn.Object(dbusServiceName, dbusPath),
	}, nil
}

// Dumb Dbus bindings for kwallet bindings with types.
type kwalletBinding struct {
	dbus dbus.BusObject
}

// method bool org.kde.KWallet.isOpen(int handle)
func (k *kwalletBinding) IsOpen(handle int32) (bool, error) {
	call := k.dbus.Call("org.kde.KWallet.isOpen", 0, handle)
	if call.Err != nil {
		return false, call.Err
	}

	return call.Body[0].(bool), call.Err
}

// method int org.kde.KWallet.open(QString wallet, qlonglong wId, QString appid)
func (k *kwalletBinding) Open(name string, wID int64, appid string) (int32, error) {
	call := k.dbus.Call("org.kde.KWallet.open", 0, name, wID, appid)
	if call.Err != nil {
		return 0, call.Err
	}

	return call.Body[0].(int32), call.Err
}

// method QStringList org.kde.KWallet.entryList(int handle, QString folder, QString appid)
func (k *kwalletBinding) EntryList(handle int32, folder string, appid string) ([]string, error) {
	call := k.dbus.Call("org.kde.KWallet.entryList", 0, handle, folder, appid)
	if call.Err != nil {
		return []string{}, call.Err
	}

	return call.Body[0].([]string), call.Err
}

// method int org.kde.KWallet.writeEntry(int handle, QString folder, QString key, QByteArray value, QString appid)
func (k *kwalletBinding) WriteEntry(handle int32, folder string, key string, value []byte, appid string) error {
	call := k.dbus.Call("org.kde.KWallet.writeEntry", 0, handle, folder, key, value, appid)
	if call.Err != nil {
		return call.Err
	}

	return call.Err
}

// method int org.kde.KWallet.removeEntry(int handle, QString folder, QString key, QString appid)
func (k *kwalletBinding) RemoveEntry(handle int32, folder string, key string, appid string) error {
	call := k.dbus.Call("org.kde.KWallet.removeEntry", 0, handle, folder, key, appid)
	if call.Err != nil {
		return call.Err
	}

	return call.Err
}

// method QByteArray org.kde.KWallet.readEntry(int handle, QString folder, QString key, QString appid)
func (k *kwalletBinding) ReadEntry(handle int32, folder string, key string, appid string) ([]byte, error) {
	call := k.dbus.Call("org.kde.KWallet.readEntry", 0, handle, folder, key, appid)
	if call.Err != nil {
		return []byte{}, call.Err
	}

	return call.Body[0].([]byte), call.Err
}

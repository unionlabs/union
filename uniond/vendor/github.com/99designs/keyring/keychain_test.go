//go:build darwin
// +build darwin

package keyring

import (
	"fmt"
	"os"
	"path/filepath"
	"reflect"
	"testing"
	"time"
)

func TestOSXKeychainKeyringSet(t *testing.T) {
	path := tempPath()
	defer deleteKeychain(t, path)

	k := &keychain{
		path:         path,
		passwordFunc: FixedStringPrompt("test password"),
		service:      "test",
		isTrusted:    true,
	}

	item := Item{
		Key:         "llamas",
		Label:       "Arbitrary label",
		Description: "A freetext description",
		Data:        []byte("llamas are great"),
	}

	if err := k.Set(item); err != nil {
		t.Fatal(err)
	}

	v, err := k.Get("llamas")
	if err != nil {
		t.Fatal(err)
	}

	if string(v.Data) != string(item.Data) {
		t.Fatalf("Data stored was not the data retrieved: %q vs %q", v.Data, item.Data)
	}

	if v.Key != item.Key {
		t.Fatalf("Key stored was not the data retrieved: %q vs %q", v.Key, item.Key)
	}

	if v.Description != item.Description {
		t.Fatalf("Description stored was not the data retrieved: %q vs %q", v.Description, item.Description)
	}
}

func TestOSXKeychainKeyringOverwrite(t *testing.T) {
	path := tempPath()
	defer deleteKeychain(t, path)

	k := &keychain{
		path:         path,
		passwordFunc: FixedStringPrompt("test password"),
		service:      "test",
		isTrusted:    true,
	}

	item1 := Item{
		Key:         "llamas",
		Label:       "Arbitrary label",
		Description: "A freetext description",
		Data:        []byte("llamas are ok"),
	}

	if err := k.Set(item1); err != nil {
		t.Fatal(err)
	}

	v1, err := k.Get("llamas")
	if err != nil {
		t.Fatal(err)
	}

	if string(v1.Data) != string(item1.Data) {
		t.Fatalf("Data stored was not the data retrieved: %q vs %q", v1.Data, item1.Data)
	}

	item2 := Item{
		Key:         "llamas",
		Label:       "Arbitrary label",
		Description: "A freetext description",
		Data:        []byte("llamas are great"),
	}

	if err := k.Set(item2); err != nil {
		t.Fatal(err)
	}

	v2, err := k.Get("llamas")
	if err != nil {
		t.Fatal(err)
	}

	if string(v2.Data) != string(item2.Data) {
		t.Fatalf("Data stored was not the data retrieved: %q vs %q", v2.Data, item2.Data)
	}
}

func TestOSXKeychainKeyringListKeysWhenEmpty(t *testing.T) {
	path := tempPath()
	defer deleteKeychain(t, path)

	k := &keychain{
		path:         path,
		service:      "test",
		passwordFunc: FixedStringPrompt("test password"),
		isTrusted:    true,
	}

	keys, err := k.Keys()
	if err != nil {
		t.Fatal(err)
	}
	if len(keys) != 0 {
		t.Fatalf("Expected 0 keys, got %d", len(keys))
	}
}

func TestOSXKeychainKeyringListKeysWhenNotEmpty(t *testing.T) {
	path := tempPath()
	defer deleteKeychain(t, path)

	k := &keychain{
		path:         path,
		service:      "test",
		passwordFunc: FixedStringPrompt("test password"),
		isTrusted:    true,
	}

	keys := []string{"key1", "key2", "key3"}

	for _, key := range keys {
		item := Item{
			Key:  key,
			Data: []byte("llamas are great"),
		}

		if err := k.Set(item); err != nil {
			t.Fatal(err)
		}
	}

	keys2, err := k.Keys()
	if err != nil {
		t.Fatal(err)
	}

	if !reflect.DeepEqual(keys, keys2) {
		t.Fatalf("Retrieved keys weren't the same: %q vs %q", keys, keys2)
	}
}

func deleteKeychain(t *testing.T, path string) {
	t.Helper()

	if _, err := os.Stat(path); os.IsExist(err) {
		_ = os.Remove(path)
	}

	// Sierra introduced a -db suffix
	dbPath := path + "-db"
	if _, err := os.Stat(dbPath); os.IsExist(err) {
		_ = os.Remove(dbPath)
	}
}

func TestOSXKeychainGetKeyWhenEmpty(t *testing.T) {
	path := tempPath()
	defer deleteKeychain(t, path)

	k := &keychain{
		path:         path,
		passwordFunc: FixedStringPrompt("test password"),
		service:      "test",
		isTrusted:    true,
	}

	_, err := k.Get("no-such-key")
	if err != ErrKeyNotFound {
		t.Fatal("expected ErrKeyNotFound")
	}
}

func TestOSXKeychainGetKeyWhenNotEmpty(t *testing.T) {
	path := tempPath()
	defer deleteKeychain(t, path)

	k := &keychain{
		path:         path,
		passwordFunc: FixedStringPrompt("test password"),
		service:      "test",
		isTrusted:    true,
	}
	item := Item{
		Key:         "llamas",
		Label:       "Arbitrary label",
		Description: "A freetext description",
		Data:        []byte("llamas are ok"),
	}

	if err := k.Set(item); err != nil {
		t.Fatal(err)
	}

	v1, err := k.Get("llamas")
	if err != nil {
		t.Fatal(err)
	}
	if string(v1.Data) != string(item.Data) {
		t.Fatalf("Data stored was not the data retrieved: %q vs %q", v1.Data, item.Data)
	}
}

func TestOSXKeychainRemoveKeyWhenEmpty(t *testing.T) {
	path := tempPath()
	defer deleteKeychain(t, path)

	k := &keychain{
		path:         path,
		passwordFunc: FixedStringPrompt("test password"),
		service:      "test",
		isTrusted:    true,
	}

	err := k.Remove("no-such-key")
	if err != ErrKeyNotFound {
		t.Fatalf("expected ErrKeyNotFound, got: %v", err)
	}
}

func TestOSXKeychainRemoveKeyWhenNotEmpty(t *testing.T) {
	path := tempPath()
	defer deleteKeychain(t, path)

	k := &keychain{
		path:         path,
		passwordFunc: FixedStringPrompt("test password"),
		service:      "test",
		isTrusted:    true,
	}
	item := Item{
		Key:         "llamas",
		Label:       "Arbitrary label",
		Description: "A freetext description",
		Data:        []byte("llamas are ok"),
	}

	if err := k.Set(item); err != nil {
		t.Fatal(err)
	}

	_, err := k.Get("llamas")
	if err != nil {
		t.Fatal(err)
	}

	err = k.Remove("llamas")
	if err != nil {
		t.Fatal(err)
	}
}

func tempPath() string {
	// TODO make filename configurable
	return filepath.Join(os.TempDir(), fmt.Sprintf("keyring-test-%d.keychain", time.Now().UnixNano()))
}

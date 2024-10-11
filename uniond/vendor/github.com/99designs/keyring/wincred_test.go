//go:build windows
// +build windows

package keyring_test

import (
	"reflect"
	"testing"

	"github.com/99designs/keyring"
)

func TestSavingCredentialsWithWinCred(t *testing.T) {
	kr, err := keyring.Open(keyring.Config{
		AllowedBackends: []keyring.BackendType{keyring.WinCredBackend},
	})
	if err != nil {
		t.Fatal(err)
	}

	item1 := keyring.Item{
		Key:  "test",
		Data: []byte("loose lips sink ships"),
	}

	err = kr.Set(item1)
	if err != nil {
		t.Fatal(err)
	}

	item2, err := kr.Get("test")
	if err != nil {
		t.Fatal(err)
	}

	if !reflect.DeepEqual(item1, item2) {
		t.Fatalf("Expected %#v, got %#v", item1, item2)
	}

	err = kr.Remove("test")
	if err != nil {
		t.Fatal(err)
	}

	_, err = kr.Get("test")
	if err != keyring.ErrKeyNotFound {
		t.Fatalf("Expected %v, got %v", keyring.ErrKeyNotFound, err)
	}
}

func TestListingCredentialsWithWinCred(t *testing.T) {
	kr, err := keyring.Open(keyring.Config{
		AllowedBackends: []keyring.BackendType{keyring.WinCredBackend},
	})
	if err != nil {
		t.Fatal(err)
	}

	item1 := keyring.Item{
		Key:  "test",
		Data: []byte("loose lips sink ships"),
	}

	err = kr.Set(item1)
	if err != nil {
		t.Fatal(err)
	}

	keys, err := kr.Keys()
	if err != nil {
		t.Fatal(err)
	}

	if expected := []string{"test"}; !reflect.DeepEqual(keys, expected) {
		t.Fatalf("Unexpected keys, got %#v, expected %#v", keys, expected)
	}

	err = kr.Remove("test")
	if err != nil {
		t.Fatal(err)
	}
}

func TestWinCredGetWhenEmpty(t *testing.T) {
	kr, err := keyring.Open(keyring.Config{
		AllowedBackends: []keyring.BackendType{keyring.WinCredBackend},
	})
	if err != nil {
		t.Fatal(err)
	}

	_, err = kr.Get("llamas")
	if err != keyring.ErrKeyNotFound {
		t.Fatal("Expected ErrKeyNotFound")
	}
}

func TestWinCredRemoveWhenEmpty(t *testing.T) {
	kr, err := keyring.Open(keyring.Config{
		AllowedBackends: []keyring.BackendType{keyring.WinCredBackend},
	})
	if err != nil {
		t.Fatal(err)
	}

	err = kr.Remove("no-such-key")
	if err != keyring.ErrKeyNotFound {
		t.Fatal("Expected ErrKeyNotFound")
	}
}

func TestWinCredKeysWhenEmpty(t *testing.T) {
	kr, err := keyring.Open(keyring.Config{
		AllowedBackends: []keyring.BackendType{keyring.WinCredBackend},
	})
	if err != nil {
		t.Fatal(err)
	}

	keys, err := kr.Keys()
	if err != nil {
		t.Fatal(err)
	}
	if len(keys) != 0 {
		t.Fatalf("Expected 0 keys, got %d", len(keys))
	}
}

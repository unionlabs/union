//go:build linux
// +build linux

package keyring

import (
	"os"
	"sort"
	"testing"

	"github.com/gsterjov/go-libsecret"
)

// NOTE: These tests are not runnable from a headless environment such as
// Docker or a CI pipeline due to the DBus "prompt" interface being called
// by the underlying go-libsecret when creating and unlocking a keychain.
//
// TODO: Investigate a way to automate the prompting. Some ideas:
//
//  1. I've looked extensively but have not found a headless CLI tool that
//     could be run in the background of eg: a docker container
//  2. It might be possible to make a mock prompter that connects to DBus
//     and provides the Prompt interface using the go-libsecret library.

func libSecretSetup(t *testing.T) (Keyring, func(t *testing.T)) {
	t.Helper()
	if os.Getenv("GITHUB_ACTIONS") != "" {
		t.Skip("Skipping testing in CI environment")
	}

	service, err := libsecret.NewService()
	if err != nil {
		t.Fatal(err)
	}
	kr := &secretsKeyring{
		name:    "keyring-test",
		service: service,
	}
	return kr, func(t *testing.T) {
		t.Helper()
		if err := kr.deleteCollection(); err != nil {
			t.Fatal(err)
		}
	}
}

func TestLibSecretKeysWhenEmpty(t *testing.T) {
	kr, _ := libSecretSetup(t)

	keys, err := kr.Keys()
	if err != nil {
		t.Fatal(err)
	}
	if len(keys) != 0 {
		t.Fatalf("Expected 0 keys, got %d", len(keys))
	}
}

func TestLibSecretKeysWhenNotEmpty(t *testing.T) {
	kr, teardown := libSecretSetup(t)
	defer teardown(t)

	item := Item{Key: "llamas", Data: []byte("llamas are great")}
	item2 := Item{Key: "alpacas", Data: []byte("alpacas are better")}

	if err := kr.Set(item); err != nil {
		t.Fatal(err)
	}

	if err := kr.Set(item2); err != nil {
		t.Fatal(err)
	}

	keys, err := kr.Keys()
	if err != nil {
		t.Fatal(err)
	}

	if len(keys) != 2 {
		t.Fatalf("Expected 2 keys, got %d", len(keys))
	}

	sort.Strings(keys)
	if keys[0] != "alpacas" {
		t.Fatalf("Expected alpacas")
	}
	if keys[1] != "llamas" {
		t.Fatalf("Expected llamas")
	}
}

func TestLibSecretGetWhenEmpty(t *testing.T) {
	kr, _ := libSecretSetup(t)

	_, err := kr.Get("llamas")
	if err != ErrKeyNotFound {
		t.Fatalf("Expected ErrKeyNotFound, got: %s", err)
	}
}

func TestLibSecretGetWhenNotEmpty(t *testing.T) {
	kr, teardown := libSecretSetup(t)
	defer teardown(t)

	item := Item{Key: "llamas", Data: []byte("llamas are great")}

	if err := kr.Set(item); err != nil {
		t.Fatal(err)
	}

	it, err := kr.Get(item.Key)
	if err != nil {
		t.Fatal(err)
	}
	if it.Key != item.Key {
		t.Fatal("Expected item not returned")
	}
}

func TestLibSecretRemoveWhenEmpty(t *testing.T) {
	kr, _ := libSecretSetup(t)

	err := kr.Remove("no-such-key")
	if err != ErrKeyNotFound {
		t.Fatalf("Expected ErrKeyNotFound, got: %s", err)
	}
}

func TestLibSecretRemoveWhenNotEmpty(t *testing.T) {
	kr, teardown := libSecretSetup(t)
	defer teardown(t)

	item := Item{Key: "llamas", Data: []byte("llamas are great")}

	if err := kr.Set(item); err != nil {
		t.Fatal(err)
	}

	if _, err := kr.Get("llamas"); err != nil {
		t.Fatal(err)
	}

	if err := kr.Remove("llamas"); err != nil {
		t.Fatal(err)
	}
}

func TestLibSpecialCharacters(t *testing.T) {
	decoded := decodeKeyringString("keyring_2dtest")
	if decoded != "keyring-test" {
		t.Fatal("incorrect decodeKeyringString")
	}

	decoded = decodeKeyringString("keyring_2d_2dtest")
	if decoded != "keyring--test" {
		t.Fatal("incorrect decodeKeyringString")
	}

	decoded = decodeKeyringString("keyring_2dtest_2d_2d")
	if decoded != "keyring-test--" {
		t.Fatal("incorrect decodeKeyringString")
	}

	decoded = decodeKeyringString("_2d_2dkeyring_2dtest_2d_2d")
	if decoded != "--keyring-test--" {
		t.Fatal("incorrect decodeKeyringString")
	}
}

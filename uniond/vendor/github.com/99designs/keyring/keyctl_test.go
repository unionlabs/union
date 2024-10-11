//go:build linux
// +build linux

package keyring_test

import (
	"errors"
	"math/rand"
	"syscall"
	"testing"
	"time"

	"github.com/99designs/keyring"
	"golang.org/x/sys/unix"

	"github.com/stretchr/testify/require"
)

var ringname = getRandomKeyringName(16)

const ringparent = "thread"

func getRandomKeyringName(length int) string {
	const charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"

	rand.Seed(time.Now().UnixNano())

	buf := make([]byte, length)
	for i := range buf {
		buf[i] = charset[rand.Intn(len(charset))]
	}
	return "keyctl_test_" + string(buf)
}

func doesNamedKeyringExist() (bool, error) {
	ringparentID, err := keyring.GetKeyringIDForScope(ringparent)
	if err != nil {
		return false, nil //nolint:nilerr
	}

	_, err = unix.KeyctlSearch(int(ringparentID), "keyring", ringname, 0)
	if errors.Is(err, syscall.ENOKEY) {
		return false, nil
	}
	return err == nil, err
}

func cleanupNamedKeyring() {
	ringparentID, err := keyring.GetKeyringIDForScope(ringparent)
	if err != nil {
		return
	}

	named, err := unix.KeyctlSearch(int(ringparentID), "keyring", ringname, 0)
	if err != nil {
		return
	}
	_, _, _ = syscall.Syscall(syscall.SYS_KEYCTL, uintptr(unix.KEYCTL_UNLINK), uintptr(named), uintptr(ringparentID))
}

func TestKeyCtlIsAvailable(t *testing.T) {
	backends := keyring.AvailableBackends()
	require.Containsf(t, backends, keyring.KeyCtlBackend, "keyctl backends not among %v", backends)
}

func TestKeyCtlOpenFailWrongScope(t *testing.T) {
	failingScopes := []string{"", "group", "invalid"}
	for _, scope := range failingScopes {
		_, err := keyring.Open(keyring.Config{
			AllowedBackends: []keyring.BackendType{keyring.KeyCtlBackend},
			KeyCtlScope:     scope,
		})
		require.Errorf(t, err, "scope %q should fail", scope)
	}
}

func TestKeyCtlOpen(t *testing.T) {
	scopes := []string{"user", "session", "process", "thread"}
	for _, scope := range scopes {
		_, err := keyring.Open(keyring.Config{
			AllowedBackends: []keyring.BackendType{keyring.KeyCtlBackend},
			KeyCtlScope:     scope,
		})
		require.NoError(t, err)
	}
}

func TestKeyCtlOpenNamed(t *testing.T) {
	exists, err := doesNamedKeyringExist()
	require.Falsef(t, exists, "ring %q already exists in scope %q", ringname, ringparent)
	require.NoErrorf(t, err, "checking for ring %q in scope %q failed: %v", ringname, ringparent, err)
	t.Cleanup(cleanupNamedKeyring)

	_, err = keyring.Open(keyring.Config{
		AllowedBackends: []keyring.BackendType{keyring.KeyCtlBackend},
		KeyCtlScope:     ringparent,
		ServiceName:     ringname,
	})
	require.NoError(t, err)
}

func TestKeyCtlSet(t *testing.T) {
	kr, err := keyring.Open(keyring.Config{
		AllowedBackends: []keyring.BackendType{keyring.KeyCtlBackend},
		KeyCtlScope:     "user",
		KeyCtlPerm:      0x3f3f0000, // "alswrvalswrv------------"
	})
	require.NoError(t, err)

	item1 := keyring.Item{
		Key:  "test",
		Data: []byte("loose lips sink ships"),
	}

	require.NoError(t, kr.Set(item1))

	item2, err := kr.Get("test")
	require.NoError(t, err)

	require.Equal(t, item1, item2)

	require.NoError(t, kr.Remove("test"))

	_, err = kr.Get("test")
	require.Error(t, err)
	require.ErrorIs(t, err, keyring.ErrKeyNotFound)
}

func TestKeyCtlSetNamed(t *testing.T) {
	exists, err := doesNamedKeyringExist()
	require.Falsef(t, exists, "ring %q already exists in scope %q", ringname, ringparent)
	require.NoErrorf(t, err, "checking for ring %q in scope %q failed: %v", ringname, ringparent, err)
	t.Cleanup(cleanupNamedKeyring)

	kr, err := keyring.Open(keyring.Config{
		AllowedBackends: []keyring.BackendType{keyring.KeyCtlBackend},
		KeyCtlScope:     ringparent,
		ServiceName:     ringname,
		KeyCtlPerm:      0x3f3f0000, // "alswrvalswrv------------"
	})
	require.NoError(t, err)

	item1 := keyring.Item{
		Key:  "test",
		Data: []byte("loose lips sink ships"),
	}

	require.NoError(t, kr.Set(item1))

	item2, err := kr.Get("test")
	require.NoError(t, err)

	require.Equal(t, item1, item2)

	require.NoError(t, kr.Remove("test"))

	_, err = kr.Get("test")
	require.Error(t, err)
	require.ErrorIs(t, err, keyring.ErrKeyNotFound)
}

func TestKeyCtlList(t *testing.T) {
	exists, err := doesNamedKeyringExist()
	require.Falsef(t, exists, "ring %q already exists in scope %q", ringname, ringparent)
	require.NoErrorf(t, err, "checking for ring %q in scope %q failed: %v", ringname, ringparent, err)
	t.Cleanup(cleanupNamedKeyring)

	kr, err := keyring.Open(keyring.Config{
		AllowedBackends: []keyring.BackendType{keyring.KeyCtlBackend},
		KeyCtlScope:     ringparent,
		ServiceName:     ringname,
		KeyCtlPerm:      0x3f3f0000, // "alswrvalswrv------------"
	})
	require.NoError(t, err)

	item1 := keyring.Item{
		Key:  "test",
		Data: []byte("loose lips sink ships"),
	}
	require.NoError(t, kr.Set(item1))

	item2 := keyring.Item{
		Key:  "foobar",
		Data: []byte("don't foo the bar"),
	}
	require.NoError(t, kr.Set(item2))

	keys, err := kr.Keys()
	require.NoError(t, err)

	expected := []string{"test", "foobar"}
	require.ElementsMatch(t, keys, expected)

	require.NoError(t, kr.Remove("test"))
	require.NoError(t, kr.Remove("foobar"))
}

func TestKeyCtlGetNonExisting(t *testing.T) {
	exists, err := doesNamedKeyringExist()
	require.Falsef(t, exists, "ring %q already exists in scope %q", ringname, ringparent)
	require.NoErrorf(t, err, "checking for ring %q in scope %q failed: %v", ringname, ringparent, err)
	t.Cleanup(cleanupNamedKeyring)

	kr, err := keyring.Open(keyring.Config{
		AllowedBackends: []keyring.BackendType{keyring.KeyCtlBackend},
		KeyCtlScope:     ringparent,
		ServiceName:     ringname,
		KeyCtlPerm:      0x3f3f0000, // "alswrvalswrv------------"
	})
	require.NoError(t, err)

	_, err = kr.Get("llamas")
	require.Error(t, err)
	require.ErrorIs(t, err, keyring.ErrKeyNotFound)
}

func TestKeyCtlRemoveNonExisting(t *testing.T) {
	exists, err := doesNamedKeyringExist()
	require.Falsef(t, exists, "ring %q already exists in scope %q", ringname, ringparent)
	require.NoErrorf(t, err, "checking for ring %q in scope %q failed: %v", ringname, ringparent, err)
	t.Cleanup(cleanupNamedKeyring)

	kr, err := keyring.Open(keyring.Config{
		AllowedBackends: []keyring.BackendType{keyring.KeyCtlBackend},
		KeyCtlScope:     ringparent,
		ServiceName:     ringname,
		KeyCtlPerm:      0x3f3f0000, // "alswrvalswrv------------"
	})
	require.NoError(t, err)

	err = kr.Remove("no-such-key")
	require.Error(t, err)
	require.ErrorIs(t, err, keyring.ErrKeyNotFound)
}

func TestKeyCtlListEmptyKeyring(t *testing.T) {
	exists, err := doesNamedKeyringExist()
	require.Falsef(t, exists, "ring %q already exists in scope %q", ringname, ringparent)
	require.NoErrorf(t, err, "checking for ring %q in scope %q failed: %v", ringname, ringparent, err)
	t.Cleanup(cleanupNamedKeyring)

	kr, err := keyring.Open(keyring.Config{
		AllowedBackends: []keyring.BackendType{keyring.KeyCtlBackend},
		KeyCtlScope:     ringparent,
		ServiceName:     ringname,
		KeyCtlPerm:      0x3f3f0000, // "alswrvalswrv------------"
	})
	require.NoError(t, err)

	keys, err := kr.Keys()
	require.NoError(t, err)
	require.Len(t, keys, 0)
}

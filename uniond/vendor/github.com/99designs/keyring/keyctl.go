//go:build linux
// +build linux

package keyring

import (
	"errors"
	"fmt"
	"strings"
	"syscall"
	"unsafe"

	"golang.org/x/sys/unix"
)

//nolint:revive
const (
	KEYCTL_PERM_VIEW    = uint32(1 << 0)
	KEYCTL_PERM_READ    = uint32(1 << 1)
	KEYCTL_PERM_WRITE   = uint32(1 << 2)
	KEYCTL_PERM_SEARCH  = uint32(1 << 3)
	KEYCTL_PERM_LINK    = uint32(1 << 4)
	KEYCTL_PERM_SETATTR = uint32(1 << 5)
	KEYCTL_PERM_ALL     = uint32((1 << 6) - 1)

	KEYCTL_PERM_OTHERS  = 0
	KEYCTL_PERM_GROUP   = 8
	KEYCTL_PERM_USER    = 16
	KEYCTL_PERM_PROCESS = 24
)

// GetPermissions constructs the permission mask from the elements.
func GetPermissions(process, user, group, others uint32) uint32 {
	perm := others << KEYCTL_PERM_OTHERS
	perm |= group << KEYCTL_PERM_GROUP
	perm |= user << KEYCTL_PERM_USER
	perm |= process << KEYCTL_PERM_PROCESS

	return perm
}

// GetKeyringIDForScope get the keyring ID for a given scope.
func GetKeyringIDForScope(scope string) (int32, error) {
	ringRef, err := getKeyringForScope(scope)
	if err != nil {
		return 0, err
	}
	id, err := unix.KeyctlGetKeyringID(int(ringRef), false)
	return int32(id), err
}

type keyctlKeyring struct {
	keyring int32
	perm    uint32
}

func init() {
	supportedBackends[KeyCtlBackend] = opener(func(cfg Config) (Keyring, error) {
		keyring := keyctlKeyring{}
		if cfg.KeyCtlPerm > 0 {
			keyring.perm = cfg.KeyCtlPerm
		}

		parent, err := getKeyringForScope(cfg.KeyCtlScope)
		if err != nil {
			return nil, fmt.Errorf("accessing %q keyring failed: %v", cfg.KeyCtlScope, err)
		}

		// Check for named keyrings
		keyring.keyring = parent
		if cfg.ServiceName != "" {
			namedKeyring, err := keyctlSearch(parent, "keyring", cfg.ServiceName)
			if err != nil {
				if !errors.Is(err, syscall.ENOKEY) {
					return nil, fmt.Errorf("opening named %q keyring failed: %v", cfg.KeyCtlScope, err)
				}

				// Keyring does not yet exist, create it
				namedKeyring, err = keyring.createNamedKeyring(parent, cfg.ServiceName)
				if err != nil {
					return nil, fmt.Errorf("creating named %q keyring failed: %v", cfg.KeyCtlScope, err)
				}
			}
			keyring.keyring = namedKeyring
		}

		return &keyring, nil
	})
}

func (k *keyctlKeyring) Get(name string) (Item, error) {
	key, err := keyctlSearch(k.keyring, "user", name)
	if err != nil {
		if errors.Is(err, syscall.ENOKEY) {
			return Item{}, ErrKeyNotFound
		}
		return Item{}, err
	}
	// data, err := key.Get()
	data, err := keyctlRead(key)
	if err != nil {
		return Item{}, err
	}

	item := Item{
		Key:  name,
		Data: data,
	}

	return item, nil
}

// GetMetadata for pass returns an error indicating that it's unsupported for this backend.
// TODO: We can deliver metadata different from the defined ones (e.g. permissions, expire-time, etc).
func (k *keyctlKeyring) GetMetadata(_ string) (Metadata, error) {
	return Metadata{}, ErrMetadataNotSupported
}

func (k *keyctlKeyring) Set(item Item) error {
	if k.perm == 0 {
		// Keep the default permissions (alswrv-----v------------)
		_, err := keyctlAdd(k.keyring, "user", item.Key, item.Data)
		return err
	}

	// By default we loose possession of the key in anything above the session keyring.
	// Together with the default permissions (which cannot be changed during creation) we
	// cannot change the permissions without possessing the key. Therefore, create the
	// key in the session keyring, change permissions and then link to the target
	// keyring and unlink from the intermediate keyring again.
	key, err := keyctlAdd(unix.KEY_SPEC_SESSION_KEYRING, "user", item.Key, item.Data)
	if err != nil {
		return fmt.Errorf("adding key to session failed: %v", err)
	}

	if err := keyctlSetperm(key, k.perm); err != nil {
		return fmt.Errorf("setting permission 0x%x failed: %v", k.perm, err)
	}

	if err := keyctlLink(k.keyring, key); err != nil {
		return fmt.Errorf("linking key to keyring failed: %v", err)
	}

	if err := keyctlUnlink(unix.KEY_SPEC_SESSION_KEYRING, key); err != nil {
		return fmt.Errorf("unlinking key from session failed: %v", err)
	}

	return nil
}

func (k *keyctlKeyring) Remove(name string) error {
	key, err := keyctlSearch(k.keyring, "user", name)
	if err != nil {
		return ErrKeyNotFound
	}

	return keyctlUnlink(k.keyring, key)
}

func (k *keyctlKeyring) Keys() ([]string, error) {
	results := []string{}

	data, err := keyctlRead(k.keyring)
	if err != nil {
		return nil, fmt.Errorf("reading keyring failed: %v", err)
	}
	ids, err := keyctlConvertKeyBuffer(data)
	if err != nil {
		return nil, fmt.Errorf("converting raw keylist failed: %v", err)
	}

	for _, id := range ids {
		info, err := keyctlDescribe(id)
		if err != nil {
			return nil, err
		}
		if info["type"] == "user" {
			results = append(results, info["description"])
		}
	}

	return results, nil
}

func (k *keyctlKeyring) createNamedKeyring(parent int32, name string) (int32, error) {
	if k.perm == 0 {
		// Keep the default permissions (alswrv-----v------------)
		return keyctlAdd(parent, "keyring", name, nil)
	}

	// By default we loose possession of the keyring in anything above the session keyring.
	// Together with the default permissions (which cannot be changed during creation) we
	// cannot change the permissions without possessing the keyring. Therefore, create the
	// keyring linked to the session keyring, change permissions and then link to the target
	// keyring and unlink from the intermediate keyring again.
	keyring, err := keyctlAdd(unix.KEY_SPEC_SESSION_KEYRING, "keyring", name, nil)
	if err != nil {
		return 0, fmt.Errorf("creating keyring failed: %v", err)
	}

	if err := keyctlSetperm(keyring, k.perm); err != nil {
		return 0, fmt.Errorf("setting permission 0x%x failed: %v", k.perm, err)
	}

	if err := keyctlLink(k.keyring, keyring); err != nil {
		return 0, fmt.Errorf("linking keyring failed: %v", err)
	}

	if err := keyctlUnlink(unix.KEY_SPEC_SESSION_KEYRING, keyring); err != nil {
		return 0, fmt.Errorf("unlinking keyring from session failed: %v", err)
	}

	return keyring, nil
}

func getKeyringForScope(scope string) (int32, error) {
	switch scope {
	case "user":
		return int32(unix.KEY_SPEC_USER_KEYRING), nil
	case "usersession":
		return int32(unix.KEY_SPEC_USER_SESSION_KEYRING), nil
	case "group":
		// Not yet implemented in the kernel
		// return int32(unix.KEY_SPEC_GROUP_KEYRING)
		return 0, fmt.Errorf("scope %q not yet implemented", scope)
	case "session":
		return int32(unix.KEY_SPEC_SESSION_KEYRING), nil
	case "process":
		return int32(unix.KEY_SPEC_PROCESS_KEYRING), nil
	case "thread":
		return int32(unix.KEY_SPEC_THREAD_KEYRING), nil
	}
	return 0, fmt.Errorf("unknown scope %q", scope)
}

func keyctlAdd(parent int32, keytype, key string, data []byte) (int32, error) {
	id, err := unix.AddKey(keytype, key, data, int(parent))
	if err != nil {
		return 0, err
	}
	return int32(id), nil
}

func keyctlSearch(id int32, idtype, name string) (int32, error) {
	key, err := unix.KeyctlSearch(int(id), idtype, name, 0)
	if err != nil {
		return 0, err
	}
	return int32(key), nil
}

func keyctlRead(id int32) ([]byte, error) {
	var buffer []byte

	for {
		length, err := unix.KeyctlBuffer(unix.KEYCTL_READ, int(id), buffer, 0)
		if err != nil {
			return nil, err
		}

		// Return the buffer if it was large enough
		if length <= len(buffer) {
			return buffer[:length], nil
		}

		// Next try with a larger buffer
		buffer = make([]byte, length)
	}
}

func keyctlDescribe(id int32) (map[string]string, error) {
	description, err := unix.KeyctlString(unix.KEYCTL_DESCRIBE, int(id))
	if err != nil {
		return nil, err
	}
	fields := strings.Split(description, ";")
	if len(fields) < 1 {
		return nil, fmt.Errorf("no data")
	}

	data := make(map[string]string)
	names := []string{"type", "uid", "gid", "perm"} // according to keyctlDescribe(3) new fields are added at the end
	data["description"] = fields[len(fields)-1]     // according to keyctlDescribe(3) description is always last
	for i, f := range fields[:len(fields)-1] {
		if i >= len(names) {
			// Do not stumble upon unknown fields
			break
		}
		data[names[i]] = f
	}

	return data, nil
}

func keyctlLink(parent, child int32) error {
	_, _, errno := syscall.Syscall(syscall.SYS_KEYCTL, uintptr(unix.KEYCTL_LINK), uintptr(child), uintptr(parent))
	if errno != 0 {
		return errno
	}
	return nil
}

func keyctlUnlink(parent, child int32) error {
	_, _, errno := syscall.Syscall(syscall.SYS_KEYCTL, uintptr(unix.KEYCTL_UNLINK), uintptr(child), uintptr(parent))
	if errno != 0 {
		return errno
	}
	return nil
}

func keyctlSetperm(id int32, perm uint32) error {
	return unix.KeyctlSetperm(int(id), perm)
}

func keyctlConvertKeyBuffer(buffer []byte) ([]int32, error) {
	if len(buffer)%4 != 0 {
		return nil, fmt.Errorf("buffer size %d not a multiple of 4", len(buffer))
	}

	results := make([]int32, 0, len(buffer)/4)
	for i := 0; i < len(buffer); i += 4 {
		// We need to case in host-native endianess here as this is what we get from the kernel.
		r := *((*int32)(unsafe.Pointer(&buffer[i])))
		results = append(results, r)
	}
	return results, nil
}

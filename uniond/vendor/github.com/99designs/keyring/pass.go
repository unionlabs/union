//go:build !windows
// +build !windows

package keyring

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

func init() {
	supportedBackends[PassBackend] = opener(func(cfg Config) (Keyring, error) {
		var err error

		pass := &passKeyring{
			passcmd: cfg.PassCmd,
			dir:     cfg.PassDir,
			prefix:  cfg.PassPrefix,
		}

		if pass.passcmd == "" {
			pass.passcmd = "pass"
		}

		if pass.dir == "" {
			if passDir, found := os.LookupEnv("PASSWORD_STORE_DIR"); found {
				pass.dir = passDir
			} else {
				homeDir, err := os.UserHomeDir()
				if err != nil {
					return nil, err
				}
				pass.dir = filepath.Join(homeDir, ".password-store")
			}
		}

		pass.dir, err = ExpandTilde(pass.dir)
		if err != nil {
			return nil, err
		}

		// fail if the pass program is not available
		_, err = exec.LookPath(pass.passcmd)
		if err != nil {
			return nil, errors.New("The pass program is not available")
		}

		return pass, nil
	})
}

type passKeyring struct {
	dir     string
	passcmd string
	prefix  string
}

func (k *passKeyring) pass(args ...string) *exec.Cmd {
	cmd := exec.Command(k.passcmd, args...)
	if k.dir != "" {
		cmd.Env = append(os.Environ(), fmt.Sprintf("PASSWORD_STORE_DIR=%s", k.dir))
	}
	cmd.Stderr = os.Stderr

	return cmd
}

func (k *passKeyring) Get(key string) (Item, error) {
	if !k.itemExists(key) {
		return Item{}, ErrKeyNotFound
	}

	name := filepath.Join(k.prefix, key)
	cmd := k.pass("show", name)
	output, err := cmd.Output()
	if err != nil {
		return Item{}, err
	}

	var decoded Item
	err = json.Unmarshal(output, &decoded)

	return decoded, err
}

func (k *passKeyring) GetMetadata(key string) (Metadata, error) {
	return Metadata{}, nil
}

func (k *passKeyring) Set(i Item) error {
	bytes, err := json.Marshal(i)
	if err != nil {
		return err
	}

	name := filepath.Join(k.prefix, i.Key)
	cmd := k.pass("insert", "-m", "-f", name)
	cmd.Stdin = strings.NewReader(string(bytes))

	err = cmd.Run()
	if err != nil {
		return err
	}

	return nil
}

func (k *passKeyring) Remove(key string) error {
	if !k.itemExists(key) {
		return ErrKeyNotFound
	}

	name := filepath.Join(k.prefix, key)
	cmd := k.pass("rm", "-f", name)
	err := cmd.Run()
	if err != nil {
		return err
	}

	return nil
}

func (k *passKeyring) itemExists(key string) bool {
	var path = filepath.Join(k.dir, k.prefix, key+".gpg")
	_, err := os.Stat(path)

	return err == nil
}

func (k *passKeyring) Keys() ([]string, error) {
	var keys = []string{}
	var path = filepath.Join(k.dir, k.prefix)

	info, err := os.Stat(path)
	if err != nil {
		if os.IsNotExist(err) {
			return keys, nil
		}
		return keys, err
	}
	if !info.IsDir() {
		return keys, fmt.Errorf("%s is not a directory", path)
	}

	err = filepath.Walk(path, func(p string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		if !info.IsDir() && filepath.Ext(p) == ".gpg" {
			name := strings.TrimPrefix(p, path)
			if name[0] == os.PathSeparator {
				name = name[1:]
			}
			keys = append(keys, name[:len(name)-4])
		}
		return nil
	})

	return keys, err
}

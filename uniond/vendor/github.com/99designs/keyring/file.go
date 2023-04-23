package keyring

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"os"
	"path/filepath"
	"time"

	jose "github.com/dvsekhvalnov/jose2go"
	"github.com/mtibben/percent"
)

func init() {
	supportedBackends[FileBackend] = opener(func(cfg Config) (Keyring, error) {
		return &fileKeyring{
			dir:          cfg.FileDir,
			passwordFunc: cfg.FilePasswordFunc,
		}, nil
	})
}

var filenameEscape = func(s string) string {
	return percent.Encode(s, "/")
}
var filenameUnescape = percent.Decode

type fileKeyring struct {
	dir          string
	passwordFunc PromptFunc
	password     string
}

func (k *fileKeyring) resolveDir() (string, error) {
	if k.dir == "" {
		return "", fmt.Errorf("No directory provided for file keyring")
	}

	dir, err := ExpandTilde(k.dir)
	if err != nil {
		return "", err
	}

	stat, err := os.Stat(dir)
	if os.IsNotExist(err) {
		err = os.MkdirAll(dir, 0700)
	} else if err != nil && !stat.IsDir() {
		err = fmt.Errorf("%s is a file, not a directory", dir)
	}

	return dir, err
}

func (k *fileKeyring) unlock() error {
	dir, err := k.resolveDir()
	if err != nil {
		return err
	}

	if k.password == "" {
		pwd, err := k.passwordFunc(fmt.Sprintf("Enter passphrase to unlock %q", dir))
		if err != nil {
			return err
		}
		k.password = pwd
	}

	return nil
}

func (k *fileKeyring) Get(key string) (Item, error) {
	filename, err := k.filename(key)
	if err != nil {
		return Item{}, err
	}

	bytes, err := ioutil.ReadFile(filename)
	if os.IsNotExist(err) {
		return Item{}, ErrKeyNotFound
	} else if err != nil {
		return Item{}, err
	}

	if err = k.unlock(); err != nil {
		return Item{}, err
	}

	payload, _, err := jose.Decode(string(bytes), k.password)
	if err != nil {
		return Item{}, err
	}

	var decoded Item
	err = json.Unmarshal([]byte(payload), &decoded)

	return decoded, err
}

func (k *fileKeyring) GetMetadata(key string) (Metadata, error) {
	filename, err := k.filename(key)
	if err != nil {
		return Metadata{}, err
	}

	stat, err := os.Stat(filename)
	if os.IsNotExist(err) {
		return Metadata{}, ErrKeyNotFound
	} else if err != nil {
		return Metadata{}, err
	}

	// For the File provider, all internal data is encrypted, not just the
	// credentials.  Thus we only have the timestamps.  Return a nil *Item.
	//
	// If we want to change this ... how portable are extended file attributes
	// these days?  Would it break user expectations of the security model to
	// leak data into those?  I'm hesitant to do so.

	return Metadata{
		ModificationTime: stat.ModTime(),
	}, nil
}

func (k *fileKeyring) Set(i Item) error {
	bytes, err := json.Marshal(i)
	if err != nil {
		return err
	}

	if err = k.unlock(); err != nil {
		return err
	}

	token, err := jose.Encrypt(string(bytes), jose.PBES2_HS256_A128KW, jose.A256GCM, k.password,
		jose.Headers(map[string]interface{}{
			"created": time.Now().String(),
		}))
	if err != nil {
		return err
	}

	filename, err := k.filename(i.Key)
	if err != nil {
		return err
	}
	return ioutil.WriteFile(filename, []byte(token), 0600)
}

func (k *fileKeyring) filename(key string) (string, error) {
	dir, err := k.resolveDir()
	if err != nil {
		return "", err
	}

	return filepath.Join(dir, filenameEscape(key)), nil
}

func (k *fileKeyring) Remove(key string) error {
	filename, err := k.filename(key)
	if err != nil {
		return err
	}

	return os.Remove(filename)
}

func (k *fileKeyring) Keys() ([]string, error) {
	dir, err := k.resolveDir()
	if err != nil {
		return nil, err
	}

	var keys = []string{}
	files, _ := ioutil.ReadDir(dir)
	for _, f := range files {
		keys = append(keys, filenameUnescape(f.Name()))
	}

	return keys, nil
}

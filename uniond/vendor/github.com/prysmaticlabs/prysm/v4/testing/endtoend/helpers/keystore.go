package helpers

import (
	"os"

	"github.com/ethereum/go-ethereum/accounts/keystore"
	"github.com/pkg/errors"
)

// KeyFromPath should only be used in endtoend tests. It is a simple helper to init a geth keystore.Key from a file.
func KeyFromPath(path, pw string) (*keystore.Key, error) {
	jsonb, err := os.ReadFile(path) // #nosec G304 -- for endtoend use only
	if err != nil {
		return nil, errors.Wrapf(err, "couldn't read keystore file %s", path)
	}
	return keystore.DecryptKey(jsonb, pw)
}

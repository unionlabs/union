package testdata

import (
	"crypto/rand"
	"fmt"
	"math/big"
	osAlias "os"
	"path/filepath"
)

// UseAliasedPackages --
func UseAliasedPackages() {
	randPath, _ := rand.Int(rand.Reader, big.NewInt(1000000))
	p := filepath.Join(tempDir(), fmt.Sprintf("/%d", randPath))
	_ = osAlias.MkdirAll(p, osAlias.ModePerm) // want "os and ioutil dir and file writing functions are not permissions-safe, use shared/file"
	someFile := filepath.Join(p, "some.txt")
	_ = osAlias.WriteFile(someFile, []byte("hello"), osAlias.ModePerm) // want "os and ioutil dir and file writing functions are not permissions-safe, use shared/file"
}

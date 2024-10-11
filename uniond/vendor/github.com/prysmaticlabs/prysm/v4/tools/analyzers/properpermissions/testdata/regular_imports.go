package testdata

import (
	"crypto/rand"
	"fmt"
	"math/big"
	"os"
	"path/filepath"
)

func tempDir() string {
	d := os.Getenv("TEST_TMPDIR")
	if d == "" {
		return os.TempDir()
	}
	return d
}

// UseOsMkdirAllAndWriteFile --
func UseOsMkdirAllAndWriteFile() {
	randPath, _ := rand.Int(rand.Reader, big.NewInt(1000000))
	p := filepath.Join(tempDir(), fmt.Sprintf("/%d", randPath))
	_ = os.MkdirAll(p, os.ModePerm) // want "os and ioutil dir and file writing functions are not permissions-safe, use shared/file"
	someFile := filepath.Join(p, "some.txt")
	_ = os.WriteFile(someFile, []byte("hello"), os.ModePerm) // want "os and ioutil dir and file writing functions are not permissions-safe, use shared/file"
}

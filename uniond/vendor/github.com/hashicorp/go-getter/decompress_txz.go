package getter

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/ulikunitz/xz"
)

// TarXzDecompressor is an implementation of Decompressor that can
// decompress tar.xz files.
type TarXzDecompressor struct {
	// FileSizeLimit limits the total size of all
	// decompressed files.
	//
	// The zero value means no limit.
	FileSizeLimit int64

	// FilesLimit limits the number of files that are
	// allowed to be decompressed.
	//
	// The zero value means no limit.
	FilesLimit int
}

func (d *TarXzDecompressor) Decompress(dst, src string, dir bool, umask os.FileMode) error {
	// If we're going into a directory we should make that first
	mkdir := dst
	if !dir {
		mkdir = filepath.Dir(dst)
	}
	if err := os.MkdirAll(mkdir, mode(0755, umask)); err != nil {
		return err
	}

	// File first
	f, err := os.Open(src)
	if err != nil {
		return err
	}
	defer f.Close()

	// xz compression is second
	txzR, err := xz.NewReader(f)
	if err != nil {
		return fmt.Errorf("Error opening an xz reader for %s: %s", src, err)
	}

	return untar(txzR, dst, src, dir, umask, d.FileSizeLimit, d.FilesLimit)
}

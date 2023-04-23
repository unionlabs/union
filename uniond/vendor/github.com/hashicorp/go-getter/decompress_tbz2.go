package getter

import (
	"compress/bzip2"
	"os"
	"path/filepath"
)

// TarBzip2Decompressor is an implementation of Decompressor that can
// decompress tar.bz2 files.
type TarBzip2Decompressor struct {
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

func (d *TarBzip2Decompressor) Decompress(dst, src string, dir bool, umask os.FileMode) error {
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

	// Bzip2 compression is second
	bzipR := bzip2.NewReader(f)
	return untar(bzipR, dst, src, dir, umask, d.FileSizeLimit, d.FilesLimit)
}

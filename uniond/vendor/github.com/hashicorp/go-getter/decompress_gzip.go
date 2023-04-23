package getter

import (
	"compress/gzip"
	"fmt"
	"os"
	"path/filepath"
)

// GzipDecompressor is an implementation of Decompressor that can
// decompress gzip files.
type GzipDecompressor struct {
	// FileSizeLimit limits the size of a decompressed file.
	//
	// The zero value means no limit.
	FileSizeLimit int64
}

func (d *GzipDecompressor) Decompress(dst, src string, dir bool, umask os.FileMode) error {
	// Directory isn't supported at all
	if dir {
		return fmt.Errorf("gzip-compressed files can only unarchive to a single file")
	}

	// If we're going into a directory we should make that first
	if err := os.MkdirAll(filepath.Dir(dst), mode(0755, umask)); err != nil {
		return err
	}

	// File first
	f, err := os.Open(src)
	if err != nil {
		return err
	}
	defer f.Close()

	// gzip compression is second
	gzipR, err := gzip.NewReader(f)
	if err != nil {
		return err
	}
	defer gzipR.Close()

	// Copy it out
	return copyReader(dst, gzipR, 0622, umask, d.FileSizeLimit)
}

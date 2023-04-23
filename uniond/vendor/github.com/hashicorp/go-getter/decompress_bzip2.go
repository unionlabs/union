package getter

import (
	"compress/bzip2"
	"fmt"
	"os"
	"path/filepath"
)

// Bzip2Decompressor is an implementation of Decompressor that can
// decompress bz2 files.
type Bzip2Decompressor struct {
	// FileSizeLimit limits the size of a decompressed file.
	//
	// The zero value means no limit.
	FileSizeLimit int64
}

func (d *Bzip2Decompressor) Decompress(dst, src string, dir bool, umask os.FileMode) error {
	// Directory isn't supported at all
	if dir {
		return fmt.Errorf("bzip2-compressed files can only unarchive to a single file")
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

	// Bzip2 compression is second
	bzipR := bzip2.NewReader(f)

	// Copy it out
	return copyReader(dst, bzipR, 0622, umask, d.FileSizeLimit)
}

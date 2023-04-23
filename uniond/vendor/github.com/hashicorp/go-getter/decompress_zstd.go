package getter

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/klauspost/compress/zstd"
)

// ZstdDecompressor is an implementation of Decompressor that
// can decompress .zst files.
type ZstdDecompressor struct {
	// FileSizeLimit limits the size of a decompressed file.
	//
	// The zero value means no limit.
	FileSizeLimit int64
}

func (d *ZstdDecompressor) Decompress(dst, src string, dir bool, umask os.FileMode) error {
	if dir {
		return fmt.Errorf("zstd-compressed files can only unarchive to a single file")
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

	// zstd compression is second
	zstdR, err := zstd.NewReader(f)
	if err != nil {
		return err
	}
	defer zstdR.Close()

	// Copy it out, potentially using a file size limit.
	return copyReader(dst, zstdR, 0622, umask, d.FileSizeLimit)
}

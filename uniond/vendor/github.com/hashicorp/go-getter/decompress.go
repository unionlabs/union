package getter

import (
	"os"
	"strings"
)

// Decompressor defines the interface that must be implemented to add
// support for decompressing a type.
//
// Important: if you're implementing a decompressor, please use the
// containsDotDot helper in this file to ensure that files can't be
// decompressed outside of the specified directory.
type Decompressor interface {
	// Decompress should decompress src to dst. dir specifies whether dst
	// is a directory or single file. src is guaranteed to be a single file
	// that exists. dst is not guaranteed to exist already.
	Decompress(dst, src string, dir bool, umask os.FileMode) error
}

// LimitedDecompressors creates the set of Decompressors, but with each compressor configured
// with the given filesLimit and/or fileSizeLimit where applicable.
func LimitedDecompressors(filesLimit int, fileSizeLimit int64) map[string]Decompressor {
	tarDecompressor := &TarDecompressor{FilesLimit: filesLimit, FileSizeLimit: fileSizeLimit}
	tbzDecompressor := &TarBzip2Decompressor{FilesLimit: filesLimit, FileSizeLimit: fileSizeLimit}
	tgzDecompressor := &TarGzipDecompressor{FilesLimit: filesLimit, FileSizeLimit: fileSizeLimit}
	txzDecompressor := &TarXzDecompressor{FilesLimit: filesLimit, FileSizeLimit: fileSizeLimit}
	tzstDecompressor := &TarZstdDecompressor{FilesLimit: filesLimit, FileSizeLimit: fileSizeLimit}
	bzipDecompressor := &Bzip2Decompressor{FileSizeLimit: fileSizeLimit}
	gzipDecompressor := &GzipDecompressor{FileSizeLimit: fileSizeLimit}
	xzDecompressor := &XzDecompressor{FileSizeLimit: fileSizeLimit}
	zipDecompressor := &ZipDecompressor{FilesLimit: filesLimit, FileSizeLimit: fileSizeLimit}
	zstDecompressor := &ZstdDecompressor{FileSizeLimit: fileSizeLimit}

	return map[string]Decompressor{
		"bz2":     bzipDecompressor,
		"gz":      gzipDecompressor,
		"xz":      xzDecompressor,
		"tar":     tarDecompressor,
		"tar.bz2": tbzDecompressor,
		"tar.gz":  tgzDecompressor,
		"tar.xz":  txzDecompressor,
		"tar.zst": tzstDecompressor,
		"tbz2":    tbzDecompressor,
		"tgz":     tgzDecompressor,
		"txz":     txzDecompressor,
		"tzst":    tzstDecompressor,
		"zip":     zipDecompressor,
		"zst":     zstDecompressor,
	}
}

const (
	noFilesLimit    = 0
	noFileSizeLimit = 0
)

// Decompressors is the mapping of extension to the Decompressor implementation
// configured with default settings that will decompress that extension/type.
//
// Note: these decompressors by default do not limit the number of files or the
// maximum file size created by the decompressed payload.
var Decompressors = LimitedDecompressors(noFilesLimit, noFileSizeLimit)

// containsDotDot checks if the filepath value v contains a ".." entry.
// This will check filepath components by splitting along / or \. This
// function is copied directly from the Go net/http implementation.
func containsDotDot(v string) bool {
	if !strings.Contains(v, "..") {
		return false
	}
	for _, ent := range strings.FieldsFunc(v, isSlashRune) {
		if ent == ".." {
			return true
		}
	}
	return false
}

func isSlashRune(r rune) bool { return r == '/' || r == '\\' }

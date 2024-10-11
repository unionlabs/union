package getter

import (
	"testing"
)

func TestLimitedDecompressors(t *testing.T) {
	const (
		maxFiles = 111
		maxSize  = 222
	)

	checkFileSizeLimit := func(limit int64) {
		if limit != maxSize {
			t.Fatalf("expected FileSizeLimit of %d, got %d", maxSize, limit)
		}
	}

	checkFilesLimit := func(limit int) {
		if limit != maxFiles {
			t.Fatalf("expected FilesLimit of %d, got %d", maxFiles, limit)
		}
	}

	decompressors := LimitedDecompressors(maxFiles, maxSize)

	checkFilesLimit(decompressors["tar"].(*TarDecompressor).FilesLimit)
	checkFileSizeLimit(decompressors["tar"].(*TarDecompressor).FileSizeLimit)

	checkFilesLimit(decompressors["tar.bz2"].(*TarBzip2Decompressor).FilesLimit)
	checkFileSizeLimit(decompressors["tar.bz2"].(*TarBzip2Decompressor).FileSizeLimit)

	checkFilesLimit(decompressors["tar.gz"].(*TarGzipDecompressor).FilesLimit)
	checkFileSizeLimit(decompressors["tar.gz"].(*TarGzipDecompressor).FileSizeLimit)

	checkFilesLimit(decompressors["tar.xz"].(*TarXzDecompressor).FilesLimit)
	checkFileSizeLimit(decompressors["tar.xz"].(*TarXzDecompressor).FileSizeLimit)

	checkFilesLimit(decompressors["tar.zst"].(*TarZstdDecompressor).FilesLimit)
	checkFileSizeLimit(decompressors["tar.zst"].(*TarZstdDecompressor).FileSizeLimit)

	checkFilesLimit(decompressors["zip"].(*ZipDecompressor).FilesLimit)
	checkFileSizeLimit(decompressors["zip"].(*ZipDecompressor).FileSizeLimit)

	// ones with file size limit only
	checkFileSizeLimit(decompressors["bz2"].(*Bzip2Decompressor).FileSizeLimit)
	checkFileSizeLimit(decompressors["gz"].(*GzipDecompressor).FileSizeLimit)
	checkFileSizeLimit(decompressors["xz"].(*XzDecompressor).FileSizeLimit)
	checkFileSizeLimit(decompressors["zst"].(*ZstdDecompressor).FileSizeLimit)
}

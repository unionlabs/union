package zipfs

import (
	"archive/zip"
	"io"
	"testing"
)

func TestFileRead(t *testing.T) {
	zrc, err := zip.OpenReader("testdata/small.zip")
	if err != nil {
		t.Fatal(err)
	}
	zfs := New(&zrc.Reader)
	f, err := zfs.Open("smallFile")
	if err != nil {
		t.Fatal(err)
	}
	info, err := f.Stat()
	if err != nil {
		t.Fatal(err)
	}
	chunkSize := info.Size() * 2 // read with extra large buffer

	buf := make([]byte, chunkSize)
	n, err := f.Read(buf)
	if err != io.EOF {
		t.Fatal("Failed to read file to completion:", err)
	}
	if n != int(info.Size()) {
		t.Errorf("Expected read length to be %d, found: %d", info.Size(), n)
	}

	// read a second time to check f.offset and f.buf are correct
	buf = make([]byte, chunkSize)
	n, err = f.Read(buf)
	if err != io.EOF {
		t.Fatal("Failed to read a fully read file:", err)
	}
	if n != 0 {
		t.Errorf("Expected read length to be 0, found: %d", n)
	}
}

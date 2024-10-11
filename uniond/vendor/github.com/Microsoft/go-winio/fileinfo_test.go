//go:build windows
// +build windows

package winio

import (
	"os"
	"testing"

	"golang.org/x/sys/windows"
)

// Checks if current matches expected. Note that AllocationSize is filesystem-specific,
// so we check that the current.AllocationSize is >= expected.AllocationSize.
// https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-fscc/5afa7f66-619c-48f3-955f-68c4ece704ae
func checkFileStandardInfo(t *testing.T, current, expected *FileStandardInfo) {
	if current.AllocationSize < expected.AllocationSize {
		t.Fatalf("FileStandardInfo unexpectedly had AllocationSize %d, expecting >=%d", current.AllocationSize, expected.AllocationSize)
	}

	if current.EndOfFile != expected.EndOfFile {
		t.Fatalf("FileStandardInfo unexpectedly had EndOfFile %d, expecting %d", current.EndOfFile, expected.EndOfFile)
	}

	if current.NumberOfLinks != expected.NumberOfLinks {
		t.Fatalf("FileStandardInfo unexpectedly had NumberOfLinks %d, expecting %d", current.NumberOfLinks, expected.NumberOfLinks)
	}

	if current.DeletePending != expected.DeletePending {
		if current.DeletePending {
			t.Fatalf("FileStandardInfo unexpectedly DeletePending")
		} else {
			t.Fatalf("FileStandardInfo unexpectedly not DeletePending")
		}
	}

	if current.Directory != expected.Directory {
		if current.Directory {
			t.Fatalf("FileStandardInfo unexpectedly Directory")
		} else {
			t.Fatalf("FileStandardInfo unexpectedly not Directory")
		}
	}
}

func TestGetFileStandardInfo_File(t *testing.T) {
	f, err := os.CreateTemp("", "tst")
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()
	defer os.Remove(f.Name())

	expectedFileInfo := &FileStandardInfo{
		AllocationSize: 0,
		EndOfFile:      0,
		NumberOfLinks:  1,
		DeletePending:  false,
		Directory:      false,
	}

	info, err := GetFileStandardInfo(f)
	if err != nil {
		t.Fatal(err)
	}
	checkFileStandardInfo(t, info, expectedFileInfo)

	bytesWritten, err := f.Write([]byte("0123456789"))
	if err != nil {
		t.Fatal(err)
	}

	expectedFileInfo.EndOfFile = int64(bytesWritten)
	expectedFileInfo.AllocationSize = int64(bytesWritten)

	info, err = GetFileStandardInfo(f)
	if err != nil {
		t.Fatal(err)
	}
	checkFileStandardInfo(t, info, expectedFileInfo)

	linkName := f.Name() + ".link"

	if err = os.Link(f.Name(), linkName); err != nil {
		t.Fatal(err)
	}
	defer os.Remove(linkName)

	expectedFileInfo.NumberOfLinks = 2

	info, err = GetFileStandardInfo(f)
	if err != nil {
		t.Fatal(err)
	}
	checkFileStandardInfo(t, info, expectedFileInfo)

	os.Remove(linkName)

	expectedFileInfo.NumberOfLinks = 1

	info, err = GetFileStandardInfo(f)
	if err != nil {
		t.Fatal(err)
	}
	checkFileStandardInfo(t, info, expectedFileInfo)
}

func TestGetFileStandardInfo_Directory(t *testing.T) {
	tempDir := t.TempDir()
	// os.Open returns the Search Handle, not the Directory Handle
	// See https://github.com/golang/go/issues/13738
	f, err := OpenForBackup(tempDir, windows.GENERIC_READ, 0, windows.OPEN_EXISTING)
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()

	expectedFileInfo := &FileStandardInfo{
		AllocationSize: 0,
		EndOfFile:      0,
		NumberOfLinks:  1,
		DeletePending:  false,
		Directory:      true,
	}

	info, err := GetFileStandardInfo(f)
	if err != nil {
		t.Fatal(err)
	}
	checkFileStandardInfo(t, info, expectedFileInfo)
}

package os

import (
	"bytes"
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestCopyFile(t *testing.T) {
	tmpfile, err := os.CreateTemp("", "example")
	if err != nil {
		t.Fatal(err)
	}
	defer os.Remove(tmpfile.Name())
	content := []byte("hello world")
	if _, err := tmpfile.Write(content); err != nil {
		t.Fatal(err)
	}

	copyfile := tmpfile.Name() + ".copy"
	if err := CopyFile(tmpfile.Name(), copyfile); err != nil {
		t.Fatal(err)
	}
	if _, err := os.Stat(copyfile); os.IsNotExist(err) {
		t.Fatal("copy should exist")
	}
	data, err := os.ReadFile(copyfile)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(data, content) {
		t.Fatalf("copy file content differs: expected %v, got %v", content, data)
	}
	os.Remove(copyfile)
}

func TestEnsureDir(t *testing.T) {
	tmp, err := os.MkdirTemp("", "ensure-dir")
	require.NoError(t, err)
	defer os.RemoveAll(tmp)

	// Should be possible to create a new directory.
	err = EnsureDir(filepath.Join(tmp, "dir"), 0o755)
	require.NoError(t, err)
	require.DirExists(t, filepath.Join(tmp, "dir"))

	// Should succeed on existing directory.
	err = EnsureDir(filepath.Join(tmp, "dir"), 0o755)
	require.NoError(t, err)

	// Should fail on file.
	err = os.WriteFile(filepath.Join(tmp, "file"), []byte{}, 0o644)
	require.NoError(t, err)
	err = EnsureDir(filepath.Join(tmp, "file"), 0o755)
	require.Error(t, err)

	// Should allow symlink to dir.
	err = os.Symlink(filepath.Join(tmp, "dir"), filepath.Join(tmp, "linkdir"))
	require.NoError(t, err)
	err = EnsureDir(filepath.Join(tmp, "linkdir"), 0o755)
	require.NoError(t, err)

	// Should error on symlink to file.
	err = os.Symlink(filepath.Join(tmp, "file"), filepath.Join(tmp, "linkfile"))
	require.NoError(t, err)
	err = EnsureDir(filepath.Join(tmp, "linkfile"), 0o755)
	require.Error(t, err)
}

// Ensure that using CopyFile does not truncate the destination file before
// the origin is positively a non-directory and that it is ready for copying.
// See https://github.com/tendermint/tendermint/issues/6427
func TestTrickedTruncation(t *testing.T) {
	tmpDir, err := os.MkdirTemp(os.TempDir(), "pwn_truncate")
	if err != nil {
		t.Fatal(err)
	}
	defer os.Remove(tmpDir)

	originalWALPath := filepath.Join(tmpDir, "wal")
	originalWALContent := []byte("I AM BECOME DEATH, DESTROYER OF ALL WORLDS!")
	if err := os.WriteFile(originalWALPath, originalWALContent, 0o755); err != nil {
		t.Fatal(err)
	}

	// 1. Sanity check.
	readWAL, err := os.ReadFile(originalWALPath)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(readWAL, originalWALContent) {
		t.Fatalf("Cannot proceed as the content does not match\nGot:  %q\nWant: %q", readWAL, originalWALContent)
	}

	// 2. Now cause the truncation of the original file.
	// It is absolutely legal to invoke os.Open on a directory.
	if err := CopyFile(tmpDir, originalWALPath); err == nil {
		t.Fatal("Expected an error")
	}

	// 3. Check the WAL's content
	reReadWAL, err := os.ReadFile(originalWALPath)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(reReadWAL, originalWALContent) {
		t.Fatalf("Oops, the WAL's content was changed :(\nGot:  %q\nWant: %q", reReadWAL, originalWALContent)
	}
}

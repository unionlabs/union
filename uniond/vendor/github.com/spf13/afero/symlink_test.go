package afero

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestSymlinkIfPossible(t *testing.T) {
	wd, _ := os.Getwd()
	defer func() {
		os.Chdir(wd)
	}()

	osFs := &OsFs{}

	workDir, err := TempDir(osFs, "", "afero-symlink")
	if err != nil {
		t.Fatal(err)
	}

	defer func() {
		osFs.RemoveAll(workDir)
	}()

	memWorkDir := "/sym"

	memFs := NewMemMapFs()
	overlayFs1 := &CopyOnWriteFs{base: osFs, layer: memFs}
	overlayFs2 := &CopyOnWriteFs{base: memFs, layer: osFs}
	overlayFsMemOnly := &CopyOnWriteFs{base: memFs, layer: NewMemMapFs()}
	basePathFs := &BasePathFs{source: osFs, path: workDir}
	basePathFsMem := &BasePathFs{source: memFs, path: memWorkDir}
	roFs := &ReadOnlyFs{source: osFs}
	roFsMem := &ReadOnlyFs{source: memFs}

	pathFileMem := filepath.Join(memWorkDir, "aferom.txt")
	osPath := filepath.Join(workDir, "afero.txt")

	WriteFile(osFs, osPath, []byte("Hi, Afero!"), 0o777)
	WriteFile(memFs, filepath.Join(pathFileMem), []byte("Hi, Afero!"), 0o777)

	testLink := func(l Linker, source, destination string, output *string) {
		if fs, ok := l.(Fs); ok {
			dir := filepath.Dir(destination)
			if dir != "" {
				fs.MkdirAll(dir, 0o777)
			}
		}

		err := l.SymlinkIfPossible(source, destination)
		if (err == nil) && (output != nil) {
			t.Fatalf("Error creating symlink, succeeded when expecting error %v", *output)
		} else if (err != nil) && (output == nil) {
			t.Fatalf("Error creating symlink, expected success, got %v", err)
		} else if err != nil && err.Error() != *output && !strings.HasSuffix(err.Error(), *output) {
			t.Fatalf("Error creating symlink, expected error '%v', instead got output '%v'", *output, err)
		} else {
			// test passed, if expecting a successful link, check the link with lstat if able
			if output == nil {
				if lst, ok := l.(Lstater); ok {
					_, ok, err := lst.LstatIfPossible(destination)
					if !ok {
						if err != nil {
							t.Fatalf("Error calling lstat on file after successful link, got: %v", err)
						} else {
							t.Fatalf("Error calling lstat on file after successful link, result didn't use lstat (not link)")
						}
						return
					}
				}
			}
		}
	}

	notSupported := ErrNoSymlink.Error()

	testLink(osFs, osPath, filepath.Join(workDir, "os/link.txt"), nil)
	testLink(overlayFs1, osPath, filepath.Join(workDir, "overlay/link1.txt"), &notSupported)
	testLink(overlayFs2, pathFileMem, filepath.Join(workDir, "overlay2/link2.txt"), nil)
	testLink(overlayFsMemOnly, pathFileMem, filepath.Join(memWorkDir, "overlay3/link.txt"), &notSupported)
	testLink(basePathFs, "afero.txt", "basepath/link.txt", nil)
	testLink(basePathFsMem, pathFileMem, "link/file.txt", &notSupported)
	testLink(roFs, osPath, filepath.Join(workDir, "ro/link.txt"), &notSupported)
	testLink(roFsMem, pathFileMem, filepath.Join(memWorkDir, "ro/link.txt"), &notSupported)
}

func TestReadlinkIfPossible(t *testing.T) {
	wd, _ := os.Getwd()
	defer func() {
		os.Chdir(wd)
	}()

	osFs := &OsFs{}

	workDir, err := TempDir(osFs, "", "afero-readlink")
	if err != nil {
		t.Fatal(err)
	}

	defer func() {
		osFs.RemoveAll(workDir)
	}()

	memWorkDir := "/read"

	memFs := NewMemMapFs()
	overlayFs1 := &CopyOnWriteFs{base: osFs, layer: memFs}
	overlayFs2 := &CopyOnWriteFs{base: memFs, layer: osFs}
	overlayFsMemOnly := &CopyOnWriteFs{base: memFs, layer: NewMemMapFs()}
	basePathFs := &BasePathFs{source: osFs, path: workDir}
	basePathFsMem := &BasePathFs{source: memFs, path: memWorkDir}
	roFs := &ReadOnlyFs{source: osFs}
	roFsMem := &ReadOnlyFs{source: memFs}

	pathFileMem := filepath.Join(memWorkDir, "aferom.txt")
	osPath := filepath.Join(workDir, "afero.txt")

	WriteFile(osFs, osPath, []byte("Hi, Afero!"), 0o777)
	WriteFile(memFs, filepath.Join(pathFileMem), []byte("Hi, Afero!"), 0o777)

	createLink := func(l Linker, source, destination string) error {
		if fs, ok := l.(Fs); ok {
			dir := filepath.Dir(destination)
			if dir != "" {
				fs.MkdirAll(dir, 0o777)
			}
		}

		return l.SymlinkIfPossible(source, destination)
	}

	testRead := func(r LinkReader, name string, output *string) {
		_, err := r.ReadlinkIfPossible(name)
		if (err != nil) && (output == nil) {
			t.Fatalf("Error reading link, expected success, got error: %v", err)
		} else if (err == nil) && (output != nil) {
			t.Fatalf("Error reading link, succeeded when expecting error: %v", *output)
		} else if err != nil && err.Error() != *output && !strings.HasSuffix(err.Error(), *output) {
			t.Fatalf("Error reading link, expected error '%v', instead received '%v'", *output, err)
		}
	}

	notSupported := ErrNoReadlink.Error()

	err = createLink(osFs, osPath, filepath.Join(workDir, "os/link.txt"))
	if err != nil {
		t.Fatal("Error creating test link: ", err)
	}

	testRead(osFs, filepath.Join(workDir, "os/link.txt"), nil)
	testRead(overlayFs1, filepath.Join(workDir, "os/link.txt"), nil)
	testRead(overlayFs2, filepath.Join(workDir, "os/link.txt"), nil)
	testRead(overlayFsMemOnly, pathFileMem, &notSupported)
	testRead(basePathFs, "os/link.txt", nil)
	testRead(basePathFsMem, pathFileMem, &notSupported)
	testRead(roFs, filepath.Join(workDir, "os/link.txt"), nil)
	testRead(roFsMem, pathFileMem, &notSupported)
}

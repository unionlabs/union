package afero

import (
	"fmt"
	"io"
	"io/fs"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"sync"
	"testing"
	"time"
)

func TestNormalizePath(t *testing.T) {
	type test struct {
		input    string
		expected string
	}

	data := []test{
		{".", FilePathSeparator},
		{"./", FilePathSeparator},
		{"..", FilePathSeparator},
		{"../", FilePathSeparator},
		{"./..", FilePathSeparator},
		{"./../", FilePathSeparator},
	}

	for i, d := range data {
		cpath := normalizePath(d.input)
		if d.expected != cpath {
			t.Errorf("Test %d failed. Expected %q got %q", i, d.expected, cpath)
		}
	}
}

func TestPathErrors(t *testing.T) {
	path := filepath.Join(".", "some", "path")
	path2 := filepath.Join(".", "different", "path")
	fs := NewMemMapFs()
	perm := os.FileMode(0o755)
	uid := 1000
	gid := 1000

	// relevant functions:
	// func (m *MemMapFs) Chmod(name string, mode os.FileMode) error
	// func (m *MemMapFs) Chtimes(name string, atime time.Time, mtime time.Time) error
	// func (m *MemMapFs) Create(name string) (File, error)
	// func (m *MemMapFs) Mkdir(name string, perm os.FileMode) error
	// func (m *MemMapFs) MkdirAll(path string, perm os.FileMode) error
	// func (m *MemMapFs) Open(name string) (File, error)
	// func (m *MemMapFs) OpenFile(name string, flag int, perm os.FileMode) (File, error)
	// func (m *MemMapFs) Remove(name string) error
	// func (m *MemMapFs) Rename(oldname, newname string) error
	// func (m *MemMapFs) Stat(name string) (os.FileInfo, error)

	err := fs.Chmod(path, perm)
	checkPathError(t, err, "Chmod")

	err = fs.Chown(path, uid, gid)
	checkPathError(t, err, "Chown")

	err = fs.Chtimes(path, time.Now(), time.Now())
	checkPathError(t, err, "Chtimes")

	// fs.Create doesn't return an error

	err = fs.Mkdir(path2, perm)
	if err != nil {
		t.Error(err)
	}
	err = fs.Mkdir(path2, perm)
	checkPathError(t, err, "Mkdir")

	err = fs.MkdirAll(path2, perm)
	if err != nil {
		t.Error("MkdirAll:", err)
	}

	_, err = fs.Open(path)
	checkPathError(t, err, "Open")

	_, err = fs.OpenFile(path, os.O_RDWR, perm)
	checkPathError(t, err, "OpenFile")

	err = fs.Remove(path)
	checkPathError(t, err, "Remove")

	err = fs.RemoveAll(path)
	if err != nil {
		t.Error("RemoveAll:", err)
	}

	err = fs.Rename(path, path2)
	checkPathError(t, err, "Rename")

	_, err = fs.Stat(path)
	checkPathError(t, err, "Stat")
}

func checkPathError(t *testing.T, err error, op string) {
	pathErr, ok := err.(*os.PathError)
	if !ok {
		t.Error(op+":", err, "is not a os.PathError")
		return
	}
	_, ok = pathErr.Err.(*os.PathError)
	if ok {
		t.Error(op+":", err, "contains another os.PathError")
	}
}

// Ensure os.O_EXCL is correctly handled.
func TestOpenFileExcl(t *testing.T) {
	const fileName = "/myFileTest"
	const fileMode = os.FileMode(0o765)

	fs := NewMemMapFs()

	// First creation should succeed.
	f, err := fs.OpenFile(fileName, os.O_CREATE|os.O_EXCL, fileMode)
	if err != nil {
		t.Errorf("OpenFile Create Excl failed: %s", err)
		return
	}
	f.Close()

	// Second creation should fail.
	_, err = fs.OpenFile(fileName, os.O_CREATE|os.O_EXCL, fileMode)
	if err == nil {
		t.Errorf("OpenFile Create Excl should have failed, but it didn't")
	}
	checkPathError(t, err, "Open")
}

// Ensure Permissions are set on OpenFile/Mkdir/MkdirAll
func TestPermSet(t *testing.T) {
	const fileName = "/myFileTest"
	const dirPath = "/myDirTest"
	const dirPathAll = "/my/path/to/dir"

	const fileMode = os.FileMode(0o765)
	// directories will also have the directory bit set
	const dirMode = fileMode | os.ModeDir

	fs := NewMemMapFs()

	// Test Openfile
	f, err := fs.OpenFile(fileName, os.O_CREATE, fileMode)
	if err != nil {
		t.Errorf("OpenFile Create failed: %s", err)
		return
	}
	f.Close()

	s, err := fs.Stat(fileName)
	if err != nil {
		t.Errorf("Stat failed: %s", err)
		return
	}
	if s.Mode().String() != fileMode.String() {
		t.Errorf("Permissions Incorrect: %s != %s", s.Mode().String(), fileMode.String())
		return
	}

	// Test Mkdir
	err = fs.Mkdir(dirPath, dirMode)
	if err != nil {
		t.Errorf("MkDir Create failed: %s", err)
		return
	}
	s, err = fs.Stat(dirPath)
	if err != nil {
		t.Errorf("Stat failed: %s", err)
		return
	}
	// sets File
	if s.Mode().String() != dirMode.String() {
		t.Errorf("Permissions Incorrect: %s != %s", s.Mode().String(), dirMode.String())
		return
	}

	// Test MkdirAll
	err = fs.MkdirAll(dirPathAll, dirMode)
	if err != nil {
		t.Errorf("MkDir Create failed: %s", err)
		return
	}
	s, err = fs.Stat(dirPathAll)
	if err != nil {
		t.Errorf("Stat failed: %s", err)
		return
	}
	if s.Mode().String() != dirMode.String() {
		t.Errorf("Permissions Incorrect: %s != %s", s.Mode().String(), dirMode.String())
		return
	}
}

// Fails if multiple file objects use the same file.at counter in MemMapFs
func TestMultipleOpenFiles(t *testing.T) {
	defer removeAllTestFiles(t)
	const fileName = "afero-demo2.txt"

	data := make([][]byte, len(Fss))

	for i, fs := range Fss {
		dir := testDir(fs)
		path := filepath.Join(dir, fileName)
		fh1, err := fs.Create(path)
		if err != nil {
			t.Error("fs.Create failed: " + err.Error())
		}
		_, err = fh1.Write([]byte("test"))
		if err != nil {
			t.Error("fh.Write failed: " + err.Error())
		}
		_, err = fh1.Seek(0, io.SeekStart)
		if err != nil {
			t.Error(err)
		}

		fh2, err := fs.OpenFile(path, os.O_RDWR, 0o777)
		if err != nil {
			t.Error("fs.OpenFile failed: " + err.Error())
		}
		_, err = fh2.Seek(0, io.SeekEnd)
		if err != nil {
			t.Error(err)
		}
		_, err = fh2.Write([]byte("data"))
		if err != nil {
			t.Error(err)
		}
		err = fh2.Close()
		if err != nil {
			t.Error(err)
		}

		_, err = fh1.Write([]byte("data"))
		if err != nil {
			t.Error(err)
		}
		err = fh1.Close()
		if err != nil {
			t.Error(err)
		}
		// the file now should contain "datadata"
		data[i], err = ReadFile(fs, path)
		if err != nil {
			t.Error(err)
		}
	}

	for i, fs := range Fss {
		if i == 0 {
			continue
		}
		if string(data[0]) != string(data[i]) {
			t.Errorf("%s and %s don't behave the same\n"+
				"%s: \"%s\"\n%s: \"%s\"\n",
				Fss[0].Name(), fs.Name(), Fss[0].Name(), data[0], fs.Name(), data[i])
		}
	}
}

// Test if file.Write() fails when opened as read only
func TestReadOnly(t *testing.T) {
	defer removeAllTestFiles(t)
	const fileName = "afero-demo.txt"

	for _, fs := range Fss {
		dir := testDir(fs)
		path := filepath.Join(dir, fileName)

		f, err := fs.Create(path)
		if err != nil {
			t.Error(fs.Name()+":", "fs.Create failed: "+err.Error())
		}
		_, err = f.Write([]byte("test"))
		if err != nil {
			t.Error(fs.Name()+":", "Write failed: "+err.Error())
		}
		f.Close()

		f, err = fs.Open(path)
		if err != nil {
			t.Error("fs.Open failed: " + err.Error())
		}
		_, err = f.Write([]byte("data"))
		if err == nil {
			t.Error(fs.Name()+":", "No write error")
		}
		f.Close()

		f, err = fs.OpenFile(path, os.O_RDONLY, 0o644)
		if err != nil {
			t.Error("fs.Open failed: " + err.Error())
		}
		_, err = f.Write([]byte("data"))
		if err == nil {
			t.Error(fs.Name()+":", "No write error")
		}
		f.Close()
	}
}

func TestWriteCloseTime(t *testing.T) {
	defer removeAllTestFiles(t)
	const fileName = "afero-demo.txt"

	for _, fs := range Fss {
		dir := testDir(fs)
		path := filepath.Join(dir, fileName)

		f, err := fs.Create(path)
		if err != nil {
			t.Error(fs.Name()+":", "fs.Create failed: "+err.Error())
		}
		f.Close()

		f, err = fs.Create(path)
		if err != nil {
			t.Error(fs.Name()+":", "fs.Create failed: "+err.Error())
		}
		fi, err := f.Stat()
		if err != nil {
			t.Error(fs.Name()+":", "Stat failed: "+err.Error())
		}
		timeBefore := fi.ModTime()

		// sorry for the delay, but we have to make sure time advances,
		// also on non Un*x systems...
		switch runtime.GOOS {
		case "windows":
			time.Sleep(2 * time.Second)
		case "darwin":
			time.Sleep(1 * time.Second)
		default: // depending on the FS, this may work with < 1 second, on my old ext3 it does not
			time.Sleep(1 * time.Second)
		}

		_, err = f.Write([]byte("test"))
		if err != nil {
			t.Error(fs.Name()+":", "Write failed: "+err.Error())
		}
		f.Close()
		fi, err = fs.Stat(path)
		if err != nil {
			t.Error(fs.Name()+":", "fs.Stat failed: "+err.Error())
		}
		if fi.ModTime().Equal(timeBefore) {
			t.Error(fs.Name()+":", "ModTime was not set on Close()")
		}
	}
}

// This test should be run with the race detector on:
// go test -race -v -timeout 10s -run TestRacingDeleteAndClose
func TestRacingDeleteAndClose(t *testing.T) {
	fs := NewMemMapFs()
	pathname := "testfile"
	f, err := fs.Create(pathname)
	if err != nil {
		t.Fatal(err)
	}

	in := make(chan bool)

	go func() {
		<-in
		f.Close()
	}()
	go func() {
		<-in
		fs.Remove(pathname)
	}()
	close(in)
}

// This test should be run with the race detector on:
// go test -run TestMemFsDataRace -race
func TestMemFsDataRace(t *testing.T) {
	const dir = "test_dir"
	fs := NewMemMapFs()

	if err := fs.MkdirAll(dir, 0o777); err != nil {
		t.Fatal(err)
	}

	const n = 1000
	done := make(chan struct{})

	go func() {
		defer close(done)
		for i := 0; i < n; i++ {
			fname := filepath.Join(dir, fmt.Sprintf("%d.txt", i))
			if err := WriteFile(fs, fname, []byte(""), 0o777); err != nil {
				panic(err)
			}
			if err := fs.Remove(fname); err != nil {
				panic(err)
			}
		}
	}()

loop:
	for {
		select {
		case <-done:
			break loop
		default:
			_, err := ReadDir(fs, dir)
			if err != nil {
				t.Fatal(err)
			}
		}
	}
}

// root is a directory
func TestMemFsRootDirMode(t *testing.T) {
	t.Parallel()

	fs := NewMemMapFs()
	info, err := fs.Stat("/")
	if err != nil {
		t.Fatal(err)
	}
	if !info.IsDir() {
		t.Error("should be a directory")
	}
	if !info.Mode().IsDir() {
		t.Errorf("FileMode is not directory, is %s", info.Mode().String())
	}
}

// MkdirAll creates intermediate directories with correct mode
func TestMemFsMkdirAllMode(t *testing.T) {
	t.Parallel()

	fs := NewMemMapFs()
	err := fs.MkdirAll("/a/b/c", 0o755)
	if err != nil {
		t.Fatal(err)
	}
	info, err := fs.Stat("/a")
	if err != nil {
		t.Fatal(err)
	}
	if !info.Mode().IsDir() {
		t.Error("/a: mode is not directory")
	}
	if !info.ModTime().After(time.Now().Add(-1 * time.Hour)) {
		t.Errorf("/a: mod time not set, got %s", info.ModTime())
	}
	if info.Mode() != os.FileMode(os.ModeDir|0o755) {
		t.Errorf("/a: wrong permissions, expected drwxr-xr-x, got %s", info.Mode())
	}
	info, err = fs.Stat("/a/b")
	if err != nil {
		t.Fatal(err)
	}
	if !info.Mode().IsDir() {
		t.Error("/a/b: mode is not directory")
	}
	if info.Mode() != os.FileMode(os.ModeDir|0o755) {
		t.Errorf("/a/b: wrong permissions, expected drwxr-xr-x, got %s", info.Mode())
	}
	if !info.ModTime().After(time.Now().Add(-1 * time.Hour)) {
		t.Errorf("/a/b: mod time not set, got %s", info.ModTime())
	}
	info, err = fs.Stat("/a/b/c")
	if err != nil {
		t.Fatal(err)
	}
	if !info.Mode().IsDir() {
		t.Error("/a/b/c: mode is not directory")
	}
	if info.Mode() != os.FileMode(os.ModeDir|0o755) {
		t.Errorf("/a/b/c: wrong permissions, expected drwxr-xr-x, got %s", info.Mode())
	}
	if !info.ModTime().After(time.Now().Add(-1 * time.Hour)) {
		t.Errorf("/a/b/c: mod time not set, got %s", info.ModTime())
	}
}

// MkdirAll does not change permissions of already-existing directories
func TestMemFsMkdirAllNoClobber(t *testing.T) {
	t.Parallel()

	fs := NewMemMapFs()
	err := fs.MkdirAll("/a/b/c", 0o755)
	if err != nil {
		t.Fatal(err)
	}
	info, err := fs.Stat("/a/b")
	if err != nil {
		t.Fatal(err)
	}
	if info.Mode() != os.FileMode(os.ModeDir|0o755) {
		t.Errorf("/a/b: wrong permissions, expected drwxr-xr-x, got %s", info.Mode())
	}
	err = fs.MkdirAll("/a/b/c/d/e/f", 0o710)
	// '/a/b' is unchanged
	if err != nil {
		t.Fatal(err)
	}
	info, err = fs.Stat("/a/b")
	if err != nil {
		t.Fatal(err)
	}
	if info.Mode() != os.FileMode(os.ModeDir|0o755) {
		t.Errorf("/a/b: wrong permissions, expected drwxr-xr-x, got %s", info.Mode())
	}
	// new directories created with proper permissions
	info, err = fs.Stat("/a/b/c/d")
	if err != nil {
		t.Fatal(err)
	}
	if info.Mode() != os.FileMode(os.ModeDir|0o710) {
		t.Errorf("/a/b/c/d: wrong permissions, expected drwx--x---, got %s", info.Mode())
	}
	info, err = fs.Stat("/a/b/c/d/e")
	if err != nil {
		t.Fatal(err)
	}
	if info.Mode() != os.FileMode(os.ModeDir|0o710) {
		t.Errorf("/a/b/c/d/e: wrong permissions, expected drwx--x---, got %s", info.Mode())
	}
	info, err = fs.Stat("/a/b/c/d/e/f")
	if err != nil {
		t.Fatal(err)
	}
	if info.Mode() != os.FileMode(os.ModeDir|0o710) {
		t.Errorf("/a/b/c/d/e/f: wrong permissions, expected drwx--x---, got %s", info.Mode())
	}
}

func TestMemFsDirMode(t *testing.T) {
	fs := NewMemMapFs()
	err := fs.Mkdir("/testDir1", 0o644)
	if err != nil {
		t.Error(err)
	}
	err = fs.MkdirAll("/sub/testDir2", 0o644)
	if err != nil {
		t.Error(err)
	}
	info, err := fs.Stat("/testDir1")
	if err != nil {
		t.Error(err)
	}
	if !info.IsDir() {
		t.Error("should be a directory")
	}
	if !info.Mode().IsDir() {
		t.Error("FileMode is not directory")
	}
	info, err = fs.Stat("/sub/testDir2")
	if err != nil {
		t.Error(err)
	}
	if !info.IsDir() {
		t.Error("should be a directory")
	}
	if !info.Mode().IsDir() {
		t.Error("FileMode is not directory")
	}
}

func TestMemFsUnexpectedEOF(t *testing.T) {
	t.Parallel()

	fs := NewMemMapFs()

	if err := WriteFile(fs, "file.txt", []byte("abc"), 0o777); err != nil {
		t.Fatal(err)
	}

	f, err := fs.Open("file.txt")
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()

	// Seek beyond the end.
	_, err = f.Seek(512, 0)
	if err != nil {
		t.Fatal(err)
	}

	buff := make([]byte, 256)
	_, err = io.ReadAtLeast(f, buff, 256)

	if err != io.ErrUnexpectedEOF {
		t.Fatal("Expected ErrUnexpectedEOF")
	}
}

func TestMemFsChmod(t *testing.T) {
	t.Parallel()

	fs := NewMemMapFs()
	const file = "hello"
	if err := fs.Mkdir(file, 0o700); err != nil {
		t.Fatal(err)
	}

	info, err := fs.Stat(file)
	if err != nil {
		t.Fatal(err)
	}
	if info.Mode().String() != "drwx------" {
		t.Fatal("mkdir failed to create a directory: mode =", info.Mode())
	}

	err = fs.Chmod(file, 0)
	if err != nil {
		t.Error("Failed to run chmod:", err)
	}

	info, err = fs.Stat(file)
	if err != nil {
		t.Fatal(err)
	}
	if info.Mode().String() != "d---------" {
		t.Error("chmod should not change file type. New mode =", info.Mode())
	}
}

// can't use Mkdir to get around which permissions we're allowed to set
func TestMemFsMkdirModeIllegal(t *testing.T) {
	t.Parallel()

	fs := NewMemMapFs()
	err := fs.Mkdir("/a", os.ModeSocket|0o755)
	if err != nil {
		t.Fatal(err)
	}
	info, err := fs.Stat("/a")
	if err != nil {
		t.Fatal(err)
	}
	if info.Mode() != os.FileMode(os.ModeDir|0o755) {
		t.Fatalf("should not be able to use Mkdir to set illegal mode: %s", info.Mode().String())
	}
}

// can't use OpenFile to get around which permissions we're allowed to set
func TestMemFsOpenFileModeIllegal(t *testing.T) {
	t.Parallel()

	fs := NewMemMapFs()
	file, err := fs.OpenFile("/a", os.O_CREATE, os.ModeSymlink|0o644)
	if err != nil {
		t.Fatal(err)
	}
	defer file.Close()
	info, err := fs.Stat("/a")
	if err != nil {
		t.Fatal(err)
	}
	if info.Mode() != os.FileMode(0o644) {
		t.Fatalf("should not be able to use OpenFile to set illegal mode: %s", info.Mode().String())
	}
}

// LstatIfPossible should always return false, since MemMapFs does not
// support symlinks.
func TestMemFsLstatIfPossible(t *testing.T) {
	t.Parallel()

	fs := NewMemMapFs()

	// We assert that fs implements Lstater
	fsAsserted, ok := fs.(Lstater)
	if !ok {
		t.Fatalf("The filesytem does not implement Lstater")
	}

	file, err := fs.OpenFile("/a.txt", os.O_CREATE, 0o644)
	if err != nil {
		t.Fatalf("Error when opening file: %v", err)
	}
	defer file.Close()

	_, lstatCalled, err := fsAsserted.LstatIfPossible("/a.txt")
	if err != nil {
		t.Fatalf("Function returned err: %v", err)
	}
	if lstatCalled {
		t.Fatalf("Function indicated lstat was called. This should never be true.")
	}
}

func TestMemMapFsConfurrentMkdir(t *testing.T) {
	const dir = "test_dir"
	const n = 1000
	mfs := NewMemMapFs().(*MemMapFs)

	allFilePaths := make([]string, 0, n)

	// run concurrency test
	var wg sync.WaitGroup
	for i := 0; i < n; i++ {
		fp := filepath.Join(
			dir,
			fmt.Sprintf("%02d", n%10),
			fmt.Sprintf("%d.txt", i),
		)
		allFilePaths = append(allFilePaths, fp)

		wg.Add(1)
		go func() {
			defer wg.Done()

			if err := mfs.MkdirAll(filepath.Dir(fp), 0o755); err != nil {
				t.Error(err)
			}

			wt, err := mfs.Create(fp)
			if err != nil {
				t.Error(err)
			}
			defer func() {
				if err := wt.Close(); err != nil {
					t.Error(err)
				}
			}()

			// write 30 bytes
			for j := 0; j < 10; j++ {
				_, err := wt.Write([]byte("000"))
				if err != nil {
					t.Error(err)
				}
			}
		}()
	}
	wg.Wait()

	// Test1: find all files by full path access
	for _, fp := range allFilePaths {
		info, err := mfs.Stat(fp)
		if err != nil {
			t.Error(err)
		}

		if info.Size() != 30 {
			t.Errorf("file size should be 30, but got %d", info.Size())
		}

	}

	// Test2: find all files by walk
	foundFiles := make([]string, 0, n)
	wErr := Walk(mfs, dir, func(path string, info fs.FileInfo, err error) error {
		if err != nil {
			t.Error(err)
		}
		if info.IsDir() {
			return nil // skip dir
		}
		if strings.HasSuffix(info.Name(), ".txt") {
			foundFiles = append(foundFiles, path)
		}
		return nil
	})
	if wErr != nil {
		t.Error(wErr)
	}
	if len(foundFiles) != n {
		t.Errorf("found %d files, but expect %d", len(foundFiles), n)
	}
}

func TestMemFsRenameDir(t *testing.T) {
	const srcPath = "/src"
	const dstPath = "/dst"
	const subDir = "dir"
	const subFile = "file.txt"

	fs := NewMemMapFs()

	err := fs.MkdirAll(srcPath+FilePathSeparator+subDir, 0o777)
	if err != nil {
		t.Fatalf("MkDirAll failed: %s", err)
	}

	f, err := fs.Create(srcPath + FilePathSeparator + subFile)
	if err != nil {
		t.Fatalf("Create failed: %s", err)
	}
	if err = f.Close(); err != nil {
		t.Fatalf("Close failed: %s", err)
	}

	err = fs.Rename(srcPath, dstPath)
	if err != nil {
		t.Fatalf("Rename failed: %s", err)
	}

	_, err = fs.Stat(srcPath + FilePathSeparator + subDir)
	if err == nil {
		t.Fatalf("SubDir still exists in the source dir")
	}

	_, err = fs.Stat(srcPath + FilePathSeparator + subFile)
	if err == nil {
		t.Fatalf("SubFile still exists in the source dir")
	}

	_, err = fs.Stat(dstPath + FilePathSeparator + subDir)
	if err != nil {
		t.Fatalf("SubDir stat in the destination dir: %s", err)
	}

	_, err = fs.Stat(dstPath + FilePathSeparator + subFile)
	if err != nil {
		t.Fatalf("SubFile stat in the destination dir: %s", err)
	}

	err = fs.Mkdir(srcPath, 0o777)
	if err != nil {
		t.Fatalf("Cannot recreate the source dir: %s", err)
	}

	err = fs.Mkdir(srcPath+FilePathSeparator+subDir, 0o777)
	if err != nil {
		t.Errorf("Cannot recreate the subdir in the source dir: %s", err)
	}
}

func TestMemMapFsRename(t *testing.T) {
	t.Parallel()

	fs := &MemMapFs{}
	tDir := testDir(fs)
	rFrom := "/renamefrom"
	rTo := "/renameto"
	rExists := "/renameexists"

	type test struct {
		dirs   []string
		from   string
		to     string
		exists string
	}

	parts := strings.Split(tDir, "/")
	root := "/"
	if len(parts) > 1 {
		root = filepath.Join("/", parts[1])
	}

	testData := make([]test, 0, len(parts))

	i := len(parts)
	for i > 0 {
		prefix := strings.Join(parts[:i], "/")
		suffix := strings.Join(parts[i:], "/")
		testData = append(testData, test{
			dirs: []string{
				filepath.Join(prefix, rFrom, suffix),
				filepath.Join(prefix, rExists, suffix),
			},
			from:   filepath.Join(prefix, rFrom),
			to:     filepath.Join(prefix, rTo),
			exists: filepath.Join(prefix, rExists),
		})
		i--
	}

	for _, data := range testData {
		err := fs.RemoveAll(root)
		if err != nil {
			t.Fatalf("%s: RemoveAll %q failed: %v", fs.Name(), root, err)
		}

		for _, dir := range data.dirs {
			err = fs.MkdirAll(dir, os.FileMode(0775))
			if err != nil {
				t.Fatalf("%s: MkdirAll %q failed: %v", fs.Name(), dir, err)
			}
		}

		dataCnt := len(fs.getData())
		err = fs.Rename(data.from, data.to)
		if err != nil {
			t.Fatalf("%s: rename %q, %q failed: %v", fs.Name(), data.from, data.to, err)
		}
		err = fs.Mkdir(data.from, os.FileMode(0775))
		if err != nil {
			t.Fatalf("%s: Mkdir %q failed: %v", fs.Name(), data.from, err)
		}

		err = fs.Rename(data.from, data.exists)
		if err != nil {
			t.Errorf("%s: rename %q, %q failed: %v", fs.Name(), data.from, data.exists, err)
		}

		for p := range fs.getData() {
			if strings.Contains(p, data.from) {
				t.Errorf("File was not renamed to renameto: %v", p)
			}
		}

		_, err = fs.Stat(data.to)
		if err != nil {
			t.Errorf("%s: stat %q failed: %v", fs.Name(), data.to, err)
		}

		if dataCnt != len(fs.getData()) {
			t.Errorf("invalid data len: expected %v, get %v", dataCnt, len(fs.getData()))
		}
	}
}

//go:build go1.16
// +build go1.16

package afero

import (
	"bytes"
	"errors"
	"fmt"
	"io"
	"io/fs"
	"math/rand"
	"os"
	"path/filepath"
	"runtime"
	"testing"
	"testing/fstest"
	"time"

	"github.com/spf13/afero/internal/common"
)

func TestIOFS(t *testing.T) {
	if runtime.GOOS == "windows" {
		// TODO(bep): some of the "bad path" tests in fstest.TestFS fail on Windows
		t.Skip("Skipping on Windows")
	}
	t.Parallel()

	t.Run("use MemMapFs", func(t *testing.T) {
		mmfs := NewMemMapFs()

		err := mmfs.MkdirAll("dir1/dir2", os.ModePerm)
		if err != nil {
			t.Fatal("MkdirAll failed:", err)
		}

		f, err := mmfs.OpenFile("dir1/dir2/test.txt", os.O_RDWR|os.O_CREATE, os.ModePerm)
		if err != nil {
			t.Fatal("OpenFile (O_CREATE) failed:", err)
		}

		f.Close()

		if err := fstest.TestFS(NewIOFS(mmfs), "dir1/dir2/test.txt"); err != nil {
			t.Error(err)
		}
	})

	t.Run("use OsFs", func(t *testing.T) {
		osfs := NewBasePathFs(NewOsFs(), t.TempDir())

		err := osfs.MkdirAll("dir1/dir2", os.ModePerm)
		if err != nil {
			t.Fatal("MkdirAll failed:", err)
		}

		f, err := osfs.OpenFile("dir1/dir2/test.txt", os.O_RDWR|os.O_CREATE, os.ModePerm)
		if err != nil {
			t.Fatal("OpenFile (O_CREATE) failed:", err)
		}

		f.Close()

		if err := fstest.TestFS(NewIOFS(osfs), "dir1/dir2/test.txt"); err != nil {
			t.Error(err)
		}
	})
}

func TestIOFSNativeDirEntryWhenPossible(t *testing.T) {
	t.Parallel()

	osfs := NewBasePathFs(NewOsFs(), t.TempDir())

	err := osfs.MkdirAll("dir1/dir2", os.ModePerm)
	if err != nil {
		t.Fatal(err)
	}

	const numFiles = 10

	var fileNumbers []int
	for i := 0; i < numFiles; i++ {
		fileNumbers = append(fileNumbers, i)
	}
	rand.Shuffle(len(fileNumbers), func(i, j int) {
		fileNumbers[i], fileNumbers[j] = fileNumbers[j], fileNumbers[i]
	})

	for _, i := range fileNumbers {
		f, err := osfs.Create(fmt.Sprintf("dir1/dir2/test%d.txt", i))
		if err != nil {
			t.Fatal(err)
		}
		f.Close()
	}

	dir2, err := osfs.Open("dir1/dir2")
	if err != nil {
		t.Fatal(err)
	}
	defer dir2.Close()

	assertDirEntries := func(entries []fs.DirEntry, ordered bool) {
		if len(entries) != numFiles {
			t.Fatalf("expected %d, got %d", numFiles, len(entries))
		}
		for i, entry := range entries {
			if _, ok := entry.(common.FileInfoDirEntry); ok {
				t.Fatal("DirEntry not native")
			}
			if ordered && entry.Name() != fmt.Sprintf("test%d.txt", i) {
				t.Fatalf("expected %s, got %s", fmt.Sprintf("test%d.txt", i), entry.Name())
			}
		}
	}

	dirEntries, err := dir2.(fs.ReadDirFile).ReadDir(-1)
	if err != nil {
		t.Fatal(err)
	}
	assertDirEntries(dirEntries, false)

	iofs := NewIOFS(osfs)

	dirEntries, err = iofs.ReadDir("dir1/dir2")
	if err != nil {
		t.Fatal(err)
	}
	assertDirEntries(dirEntries, true)

	fileCount := 0
	err = fs.WalkDir(iofs, "", func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}

		if !d.IsDir() {
			fileCount++
		}

		if _, ok := d.(common.FileInfoDirEntry); ok {
			t.Fatal("DirEntry not native")
		}

		return nil
	})

	if err != nil {
		t.Fatal(err)
	}

	if fileCount != numFiles {
		t.Fatalf("expected %d, got %d", numFiles, fileCount)
	}
}

func TestFromIOFS(t *testing.T) {
	t.Parallel()

	fsys := fstest.MapFS{
		"test.txt": {
			Data:    []byte("File in root"),
			Mode:    fs.ModePerm,
			ModTime: time.Now(),
		},
		"dir1": {
			Mode:    fs.ModeDir | fs.ModePerm,
			ModTime: time.Now(),
		},
		"dir1/dir2": {
			Mode:    fs.ModeDir | fs.ModePerm,
			ModTime: time.Now(),
		},
		"dir1/dir2/hello.txt": {
			Data:    []byte("Hello world"),
			Mode:    fs.ModePerm,
			ModTime: time.Now(),
		},
	}

	fromIOFS := FromIOFS{fsys}

	t.Run("Create", func(t *testing.T) {
		_, err := fromIOFS.Create("test")
		assertPermissionError(t, err)
	})

	t.Run("Mkdir", func(t *testing.T) {
		err := fromIOFS.Mkdir("test", 0)
		assertPermissionError(t, err)
	})

	t.Run("MkdirAll", func(t *testing.T) {
		err := fromIOFS.Mkdir("test", 0)
		assertPermissionError(t, err)
	})

	t.Run("Open", func(t *testing.T) {
		t.Run("non existing file", func(t *testing.T) {
			_, err := fromIOFS.Open("nonexisting")
			if !errors.Is(err, fs.ErrNotExist) {
				t.Errorf("Expected error to be fs.ErrNotExist, got %[1]T (%[1]v)", err)
			}
		})

		t.Run("directory", func(t *testing.T) {
			dirFile, err := fromIOFS.Open("dir1")
			if err != nil {
				t.Errorf("dir1 open failed: %v", err)
				return
			}

			defer dirFile.Close()

			dirStat, err := dirFile.Stat()
			if err != nil {
				t.Errorf("dir1 stat failed: %v", err)
				return
			}

			if !dirStat.IsDir() {
				t.Errorf("dir1 stat told that it is not a directory")
				return
			}
		})

		t.Run("simple file", func(t *testing.T) {
			file, err := fromIOFS.Open("test.txt")
			if err != nil {
				t.Errorf("test.txt open failed: %v", err)
				return
			}

			defer file.Close()

			fileStat, err := file.Stat()
			if err != nil {
				t.Errorf("test.txt stat failed: %v", err)
				return
			}

			if fileStat.IsDir() {
				t.Errorf("test.txt stat told that it is a directory")
				return
			}
		})
	})

	t.Run("Remove", func(t *testing.T) {
		err := fromIOFS.Remove("test")
		assertPermissionError(t, err)
	})

	t.Run("Rename", func(t *testing.T) {
		err := fromIOFS.Rename("test", "test2")
		assertPermissionError(t, err)
	})

	t.Run("Stat", func(t *testing.T) {
		t.Run("non existing file", func(t *testing.T) {
			_, err := fromIOFS.Stat("nonexisting")
			if !errors.Is(err, fs.ErrNotExist) {
				t.Errorf("Expected error to be fs.ErrNotExist, got %[1]T (%[1]v)", err)
			}
		})

		t.Run("directory", func(t *testing.T) {
			stat, err := fromIOFS.Stat("dir1/dir2")
			if err != nil {
				t.Errorf("dir1/dir2 stat failed: %v", err)
				return
			}

			if !stat.IsDir() {
				t.Errorf("dir1/dir2 stat told that it is not a directory")
				return
			}
		})

		t.Run("file", func(t *testing.T) {
			stat, err := fromIOFS.Stat("dir1/dir2/hello.txt")
			if err != nil {
				t.Errorf("dir1/dir2 stat failed: %v", err)
				return
			}

			if stat.IsDir() {
				t.Errorf("dir1/dir2/hello.txt stat told that it is a directory")
				return
			}

			if lenFile := len(fsys["dir1/dir2/hello.txt"].Data); int64(lenFile) != stat.Size() {
				t.Errorf("dir1/dir2/hello.txt stat told invalid size: expected %d, got %d", lenFile, stat.Size())
				return
			}
		})
	})

	t.Run("Chmod", func(t *testing.T) {
		err := fromIOFS.Chmod("test", os.ModePerm)
		assertPermissionError(t, err)
	})

	t.Run("Chown", func(t *testing.T) {
		err := fromIOFS.Chown("test", 0, 0)
		assertPermissionError(t, err)
	})

	t.Run("Chtimes", func(t *testing.T) {
		err := fromIOFS.Chtimes("test", time.Now(), time.Now())
		assertPermissionError(t, err)
	})
}

func TestFromIOFS_File(t *testing.T) {
	t.Parallel()

	fsys := fstest.MapFS{
		"test.txt": {
			Data:    []byte("File in root"),
			Mode:    fs.ModePerm,
			ModTime: time.Now(),
		},
		"dir1": {
			Mode:    fs.ModeDir | fs.ModePerm,
			ModTime: time.Now(),
		},
		"dir2": {
			Mode:    fs.ModeDir | fs.ModePerm,
			ModTime: time.Now(),
		},
	}

	fromIOFS := FromIOFS{fsys}

	file, err := fromIOFS.Open("test.txt")
	if err != nil {
		t.Errorf("test.txt open failed: %v", err)
		return
	}

	defer file.Close()

	fileStat, err := file.Stat()
	if err != nil {
		t.Errorf("test.txt stat failed: %v", err)
		return
	}

	if fileStat.IsDir() {
		t.Errorf("test.txt stat told that it is a directory")
		return
	}

	t.Run("ReadAt", func(t *testing.T) {
		// MapFS files implements io.ReaderAt
		b := make([]byte, 2)
		_, err := file.ReadAt(b, 2)
		if err != nil {
			t.Errorf("ReadAt failed: %v", err)
			return
		}

		if expectedData := fsys["test.txt"].Data[2:4]; !bytes.Equal(b, expectedData) {
			t.Errorf("Unexpected content read: %s, expected %s", b, expectedData)
		}
	})

	t.Run("Seek", func(t *testing.T) {
		n, err := file.Seek(2, io.SeekStart)
		if err != nil {
			t.Errorf("Seek failed: %v", err)
			return
		}

		if n != 2 {
			t.Errorf("Seek returned unexpected value: %d, expected 2", n)
		}
	})

	t.Run("Write", func(t *testing.T) {
		_, err := file.Write(nil)
		assertPermissionError(t, err)
	})

	t.Run("WriteAt", func(t *testing.T) {
		_, err := file.WriteAt(nil, 0)
		assertPermissionError(t, err)
	})

	t.Run("Name", func(t *testing.T) {
		if name := file.Name(); name != "test.txt" {
			t.Errorf("expected file.Name() == test.txt, got %s", name)
		}
	})

	t.Run("Readdir", func(t *testing.T) {
		t.Run("not directory", func(t *testing.T) {
			_, err := file.Readdir(-1)
			assertPermissionError(t, err)
		})

		t.Run("root directory", func(t *testing.T) {
			root, err := fromIOFS.Open(".")
			if err != nil {
				t.Errorf("root open failed: %v", err)
				return
			}

			defer root.Close()

			items, err := root.Readdir(-1)
			if err != nil {
				t.Errorf("Readdir error: %v", err)
				return
			}

			expectedItems := []struct {
				Name  string
				IsDir bool
				Size  int64
			}{
				{Name: "dir1", IsDir: true, Size: 0},
				{Name: "dir2", IsDir: true, Size: 0},
				{Name: "test.txt", IsDir: false, Size: int64(len(fsys["test.txt"].Data))},
			}

			if len(expectedItems) != len(items) {
				t.Errorf("Items count mismatch, expected %d, got %d", len(expectedItems), len(items))
				return
			}

			for i, item := range items {
				if item.Name() != expectedItems[i].Name {
					t.Errorf("Item %d: expected name %s, got %s", i, expectedItems[i].Name, item.Name())
				}

				if item.IsDir() != expectedItems[i].IsDir {
					t.Errorf("Item %d: expected IsDir %t, got %t", i, expectedItems[i].IsDir, item.IsDir())
				}

				if item.Size() != expectedItems[i].Size {
					t.Errorf("Item %d: expected IsDir %d, got %d", i, expectedItems[i].Size, item.Size())
				}
			}
		})
	})

	t.Run("Readdirnames", func(t *testing.T) {
		t.Run("not directory", func(t *testing.T) {
			_, err := file.Readdirnames(-1)
			assertPermissionError(t, err)
		})

		t.Run("root directory", func(t *testing.T) {
			root, err := fromIOFS.Open(".")
			if err != nil {
				t.Errorf("root open failed: %v", err)
				return
			}

			defer root.Close()

			items, err := root.Readdirnames(-1)
			if err != nil {
				t.Errorf("Readdirnames error: %v", err)
				return
			}

			expectedItems := []string{"dir1", "dir2", "test.txt"}

			if len(expectedItems) != len(items) {
				t.Errorf("Items count mismatch, expected %d, got %d", len(expectedItems), len(items))
				return
			}

			for i, item := range items {
				if item != expectedItems[i] {
					t.Errorf("Item %d: expected name %s, got %s", i, expectedItems[i], item)
				}
			}
		})
	})

	t.Run("Truncate", func(t *testing.T) {
		err := file.Truncate(1)
		assertPermissionError(t, err)
	})

	t.Run("WriteString", func(t *testing.T) {
		_, err := file.WriteString("a")
		assertPermissionError(t, err)
	})
}

func assertPermissionError(t *testing.T, err error) {
	t.Helper()

	var perr *fs.PathError
	if !errors.As(err, &perr) {
		t.Errorf("Expected *fs.PathError, got %[1]T (%[1]v)", err)
		return
	}

	if perr.Err != fs.ErrPermission {
		t.Errorf("Expected (*fs.PathError).Err == fs.ErrPermisson, got %[1]T (%[1]v)", err)
	}
}

func BenchmarkWalkDir(b *testing.B) {
	osfs := NewBasePathFs(NewOsFs(), b.TempDir())

	createSomeFiles := func(dirname string) {
		for i := 0; i < 10; i++ {
			f, err := osfs.Create(filepath.Join(dirname, fmt.Sprintf("test%d.txt", i)))
			if err != nil {
				b.Fatal(err)
			}
			f.Close()
		}
	}

	depth := 10
	for level := depth; level > 0; level-- {
		dirname := ""
		for i := 0; i < level; i++ {
			dirname = filepath.Join(dirname, fmt.Sprintf("dir%d", i))
			err := osfs.MkdirAll(dirname, 0o755)
			if err != nil && !os.IsExist(err) {
				b.Fatal(err)
			}
		}
		createSomeFiles(dirname)
	}

	iofs := NewIOFS(osfs)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		err := fs.WalkDir(iofs, "", func(path string, d fs.DirEntry, err error) error {
			if err != nil {
				return err
			}
			return nil
		})
		if err != nil {
			b.Fatal(err)
		}
	}
}

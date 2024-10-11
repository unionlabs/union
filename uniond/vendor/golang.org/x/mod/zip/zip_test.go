// Copyright 2019 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package zip_test

import (
	"archive/zip"
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path"
	"path/filepath"
	"runtime"
	"strings"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"golang.org/x/mod/module"
	"golang.org/x/mod/sumdb/dirhash"
	modzip "golang.org/x/mod/zip"
	"golang.org/x/tools/txtar"
)

const emptyHash = "h1:47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU="

var gitOnce struct {
	path string
	err  error
	sync.Once
}

func init() {
	if os.Getenv("GO_BUILDER_NAME") != "" || os.Getenv("GIT_TRACE_CURL") == "1" {
		// Enable extra Git logging to diagnose networking issues.
		// (These environment variables will be inherited by subprocesses.)
		os.Setenv("GIT_TRACE_CURL", "1")
		os.Setenv("GIT_TRACE_CURL_NO_DATA", "1")
		os.Setenv("GIT_REDACT_COOKIES", "o,SSO,GSSO_Uberproxy")
	}
}

// gitPath returns the path to a usable "git" command,
// or a non-nil error.
func gitPath() (string, error) {
	gitOnce.Do(func() {
		path, err := exec.LookPath("git")
		if err != nil {
			gitOnce.err = err
			return
		}
		if runtime.GOOS == "plan9" {
			gitOnce.err = errors.New("plan9 git does not support the full git command line")
		}
		gitOnce.path = path
	})

	return gitOnce.path, gitOnce.err
}

func mustHaveGit(t testing.TB) {
	if _, err := gitPath(); err != nil {
		t.Helper()
		t.Skipf("skipping: %v", err)
	}
}

type testParams struct {
	path, version, wantErr, hash string
	archive                      *txtar.Archive
}

// readTest loads a test from a txtar file. The comment section of the file
// should contain lines with key=value pairs. Valid keys are the field names
// from testParams.
func readTest(file string) (testParams, error) {
	var test testParams
	var err error
	test.archive, err = txtar.ParseFile(file)
	if err != nil {
		return testParams{}, err
	}

	lines := strings.Split(string(test.archive.Comment), "\n")
	for n, line := range lines {
		n++ // report line numbers starting with 1
		if i := strings.IndexByte(line, '#'); i >= 0 {
			line = line[:i]
		}
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}
		eq := strings.IndexByte(line, '=')
		if eq < 0 {
			return testParams{}, fmt.Errorf("%s:%d: missing = separator", file, n)
		}
		key, value := strings.TrimSpace(line[:eq]), strings.TrimSpace(line[eq+1:])
		switch key {
		case "path":
			test.path = value
		case "version":
			test.version = value
		case "wantErr":
			test.wantErr = value
		case "hash":
			test.hash = value
		default:
			return testParams{}, fmt.Errorf("%s:%d: unknown key %q", file, n, key)
		}
	}

	return test, nil
}

func extractTxtarToTempDir(t testing.TB, arc *txtar.Archive) (dir string, err error) {
	dir = t.TempDir()
	for _, f := range arc.Files {
		filePath := filepath.Join(dir, f.Name)
		if err := os.MkdirAll(filepath.Dir(filePath), 0777); err != nil {
			return "", err
		}
		if err := os.WriteFile(filePath, f.Data, 0666); err != nil {
			return "", err
		}
	}
	return dir, nil
}

func extractTxtarToTempZip(t *testing.T, arc *txtar.Archive) (zipPath string, err error) {
	zipPath = filepath.Join(t.TempDir(), "txtar.zip")

	zipFile, err := os.Create(zipPath)
	if err != nil {
		return "", err
	}
	defer func() {
		if cerr := zipFile.Close(); err == nil && cerr != nil {
			err = cerr
		}
	}()

	zw := zip.NewWriter(zipFile)
	for _, f := range arc.Files {
		zf, err := zw.Create(f.Name)
		if err != nil {
			return "", err
		}
		if _, err := zf.Write(f.Data); err != nil {
			return "", err
		}
	}
	if err := zw.Close(); err != nil {
		return "", err
	}
	return zipFile.Name(), nil
}

type fakeFile struct {
	name string
	size uint64
	data []byte // if nil, Open will access a sequence of 0-bytes
}

func (f fakeFile) Path() string                { return f.name }
func (f fakeFile) Lstat() (os.FileInfo, error) { return fakeFileInfo{f}, nil }
func (f fakeFile) Open() (io.ReadCloser, error) {
	if f.data != nil {
		return io.NopCloser(bytes.NewReader(f.data)), nil
	}
	if f.size >= uint64(modzip.MaxZipFile<<1) {
		return nil, fmt.Errorf("cannot open fakeFile of size %d", f.size)
	}
	return io.NopCloser(io.LimitReader(zeroReader{}, int64(f.size))), nil
}

type fakeFileInfo struct {
	f fakeFile
}

func (fi fakeFileInfo) Name() string       { return path.Base(fi.f.name) }
func (fi fakeFileInfo) Size() int64        { return int64(fi.f.size) }
func (fi fakeFileInfo) Mode() os.FileMode  { return 0644 }
func (fi fakeFileInfo) ModTime() time.Time { return time.Time{} }
func (fi fakeFileInfo) IsDir() bool        { return false }
func (fi fakeFileInfo) Sys() interface{}   { return nil }

type zeroReader struct{}

func (r zeroReader) Read(b []byte) (int, error) {
	for i := range b {
		b[i] = 0
	}
	return len(b), nil
}

func formatCheckedFiles(cf modzip.CheckedFiles) string {
	buf := &bytes.Buffer{}
	fmt.Fprintf(buf, "valid:\n")
	for _, f := range cf.Valid {
		fmt.Fprintln(buf, f)
	}
	fmt.Fprintf(buf, "\nomitted:\n")
	for _, f := range cf.Omitted {
		fmt.Fprintf(buf, "%s: %v\n", f.Path, f.Err)
	}
	fmt.Fprintf(buf, "\ninvalid:\n")
	for _, f := range cf.Invalid {
		fmt.Fprintf(buf, "%s: %v\n", f.Path, f.Err)
	}
	return buf.String()
}

// TestCheckFiles verifies behavior of CheckFiles. Note that CheckFiles is also
// covered by TestCreate, TestCreateDir, and TestCreateSizeLimits, so this test
// focuses on how multiple errors and omissions are reported, rather than trying
// to cover every case.
func TestCheckFiles(t *testing.T) {
	testPaths, err := filepath.Glob(filepath.FromSlash("testdata/check_files/*.txt"))
	if err != nil {
		t.Fatal(err)
	}
	for _, testPath := range testPaths {
		testPath := testPath
		name := strings.TrimSuffix(filepath.Base(testPath), ".txt")
		t.Run(name, func(t *testing.T) {
			t.Parallel()

			// Load the test.
			test, err := readTest(testPath)
			if err != nil {
				t.Fatal(err)
			}
			files := make([]modzip.File, 0, len(test.archive.Files))
			var want string
			for _, tf := range test.archive.Files {
				if tf.Name == "want" {
					want = string(tf.Data)
					continue
				}
				files = append(files, fakeFile{
					name: tf.Name,
					size: uint64(len(tf.Data)),
					data: tf.Data,
				})
			}

			// Check the files.
			cf, _ := modzip.CheckFiles(files)
			got := formatCheckedFiles(cf)
			if got != want {
				t.Errorf("got:\n%s\n\nwant:\n%s", got, want)
			}

			// Check that the error (if any) is just a list of invalid files.
			// SizeError is not covered in this test.
			var gotErr, wantErr string
			if len(cf.Invalid) > 0 {
				wantErr = modzip.FileErrorList(cf.Invalid).Error()
			}
			if err := cf.Err(); err != nil {
				gotErr = err.Error()
			}
			if gotErr != wantErr {
				t.Errorf("got error:\n%s\n\nwant error:\n%s", gotErr, wantErr)
			}
		})
	}
}

// TestCheckDir verifies behavior of the CheckDir function. Note that CheckDir
// relies on CheckFiles and listFilesInDir (called by CreateFromDir), so this
// test focuses on how multiple errors and omissions are reported, rather than
// trying to cover every case.
func TestCheckDir(t *testing.T) {
	testPaths, err := filepath.Glob(filepath.FromSlash("testdata/check_dir/*.txt"))
	if err != nil {
		t.Fatal(err)
	}
	for _, testPath := range testPaths {
		testPath := testPath
		name := strings.TrimSuffix(filepath.Base(testPath), ".txt")
		t.Run(name, func(t *testing.T) {
			t.Parallel()

			// Load the test and extract the files to a temporary directory.
			test, err := readTest(testPath)
			if err != nil {
				t.Fatal(err)
			}
			var want string
			for i, f := range test.archive.Files {
				if f.Name == "want" {
					want = string(f.Data)
					test.archive.Files = append(test.archive.Files[:i], test.archive.Files[i+1:]...)
					break
				}
			}
			tmpDir, err := extractTxtarToTempDir(t, test.archive)
			if err != nil {
				t.Fatal(err)
			}

			// Check the directory.
			cf, err := modzip.CheckDir(tmpDir)
			if err != nil && err.Error() != cf.Err().Error() {
				// I/O error
				t.Fatal(err)
			}
			rep := strings.NewReplacer(tmpDir, "$work", `'\''`, `'\''`, string(os.PathSeparator), "/")
			got := rep.Replace(formatCheckedFiles(cf))
			if got != want {
				t.Errorf("got:\n%s\n\nwant:\n%s", got, want)
			}

			// Check that the error (if any) is just a list of invalid files.
			// SizeError is not covered in this test.
			var gotErr, wantErr string
			if len(cf.Invalid) > 0 {
				wantErr = modzip.FileErrorList(cf.Invalid).Error()
			}
			if err := cf.Err(); err != nil {
				gotErr = err.Error()
			}
			if gotErr != wantErr {
				t.Errorf("got error:\n%s\n\nwant error:\n%s", gotErr, wantErr)
			}
		})
	}
}

// TestCheckZip verifies behavior of CheckZip. Note that CheckZip is also
// covered by TestUnzip, so this test focuses on how multiple errors are
// reported, rather than trying to cover every case.
func TestCheckZip(t *testing.T) {
	testPaths, err := filepath.Glob(filepath.FromSlash("testdata/check_zip/*.txt"))
	if err != nil {
		t.Fatal(err)
	}
	for _, testPath := range testPaths {
		testPath := testPath
		name := strings.TrimSuffix(filepath.Base(testPath), ".txt")
		t.Run(name, func(t *testing.T) {
			t.Parallel()

			// Load the test and extract the files to a temporary zip file.
			test, err := readTest(testPath)
			if err != nil {
				t.Fatal(err)
			}
			var want string
			for i, f := range test.archive.Files {
				if f.Name == "want" {
					want = string(f.Data)
					test.archive.Files = append(test.archive.Files[:i], test.archive.Files[i+1:]...)
					break
				}
			}
			tmpZipPath, err := extractTxtarToTempZip(t, test.archive)
			if err != nil {
				t.Fatal(err)
			}

			// Check the zip.
			m := module.Version{Path: test.path, Version: test.version}
			cf, err := modzip.CheckZip(m, tmpZipPath)
			if err != nil && err.Error() != cf.Err().Error() {
				// I/O error
				t.Fatal(err)
			}
			got := formatCheckedFiles(cf)
			if got != want {
				t.Errorf("got:\n%s\n\nwant:\n%s", got, want)
			}

			// Check that the error (if any) is just a list of invalid files.
			// SizeError is not covered in this test.
			var gotErr, wantErr string
			if len(cf.Invalid) > 0 {
				wantErr = modzip.FileErrorList(cf.Invalid).Error()
			}
			if err := cf.Err(); err != nil {
				gotErr = err.Error()
			}
			if gotErr != wantErr {
				t.Errorf("got error:\n%s\n\nwant error:\n%s", gotErr, wantErr)
			}
		})
	}
}

func TestCreate(t *testing.T) {
	testDir := filepath.FromSlash("testdata/create")
	testEntries, err := os.ReadDir(testDir)
	if err != nil {
		t.Fatal(err)
	}
	for _, testEntry := range testEntries {
		testEntry := testEntry
		base := filepath.Base(testEntry.Name())
		if filepath.Ext(base) != ".txt" {
			continue
		}
		t.Run(base[:len(base)-len(".txt")], func(t *testing.T) {
			t.Parallel()

			// Load the test.
			testPath := filepath.Join(testDir, testEntry.Name())
			test, err := readTest(testPath)
			if err != nil {
				t.Fatal(err)
			}

			// Write zip to temporary file.
			tmpZip, err := os.CreateTemp(t.TempDir(), "TestCreate-*.zip")
			if err != nil {
				t.Fatal(err)
			}
			tmpZipPath := tmpZip.Name()
			defer tmpZip.Close()
			m := module.Version{Path: test.path, Version: test.version}
			files := make([]modzip.File, len(test.archive.Files))
			for i, tf := range test.archive.Files {
				files[i] = fakeFile{
					name: tf.Name,
					size: uint64(len(tf.Data)),
					data: tf.Data,
				}
			}
			if err := modzip.Create(tmpZip, m, files); err != nil {
				if test.wantErr == "" {
					t.Fatalf("unexpected error: %v", err)
				} else if !strings.Contains(err.Error(), test.wantErr) {
					t.Fatalf("got error %q; want error containing %q", err.Error(), test.wantErr)
				} else {
					return
				}
			} else if test.wantErr != "" {
				t.Fatalf("unexpected success; wanted error containing %q", test.wantErr)
			}
			if err := tmpZip.Close(); err != nil {
				t.Fatal(err)
			}

			// Hash zip file, compare with known value.
			if hash, err := dirhash.HashZip(tmpZipPath, dirhash.Hash1); err != nil {
				t.Fatal(err)
			} else if hash != test.hash {
				t.Fatalf("got hash: %q\nwant: %q", hash, test.hash)
			}
		})
	}
}

func TestCreateFromDir(t *testing.T) {
	testDir := filepath.FromSlash("testdata/create_from_dir")
	testEntries, err := os.ReadDir(testDir)
	if err != nil {
		t.Fatal(err)
	}
	for _, testEntry := range testEntries {
		testEntry := testEntry
		base := filepath.Base(testEntry.Name())
		if filepath.Ext(base) != ".txt" {
			continue
		}
		t.Run(base[:len(base)-len(".txt")], func(t *testing.T) {
			t.Parallel()

			// Load the test.
			testPath := filepath.Join(testDir, testEntry.Name())
			test, err := readTest(testPath)
			if err != nil {
				t.Fatal(err)
			}

			// Write files to a temporary directory.
			tmpDir, err := extractTxtarToTempDir(t, test.archive)
			if err != nil {
				t.Fatal(err)
			}

			// Create zip from the directory.
			tmpZip, err := os.CreateTemp(t.TempDir(), "TestCreateFromDir-*.zip")
			if err != nil {
				t.Fatal(err)
			}
			tmpZipPath := tmpZip.Name()
			defer tmpZip.Close()
			m := module.Version{Path: test.path, Version: test.version}
			if err := modzip.CreateFromDir(tmpZip, m, tmpDir); err != nil {
				if test.wantErr == "" {
					t.Fatalf("unexpected error: %v", err)
				} else if !strings.Contains(err.Error(), test.wantErr) {
					t.Fatalf("got error %q; want error containing %q", err, test.wantErr)
				} else {
					return
				}
			} else if test.wantErr != "" {
				t.Fatalf("unexpected success; want error containing %q", test.wantErr)
			}

			// Hash zip file, compare with known value.
			if hash, err := dirhash.HashZip(tmpZipPath, dirhash.Hash1); err != nil {
				t.Fatal(err)
			} else if hash != test.hash {
				t.Fatalf("got hash: %q\nwant: %q", hash, test.hash)
			}
		})
	}
}

func TestCreateFromDirSpecial(t *testing.T) {
	for _, test := range []struct {
		desc     string
		setup    func(t *testing.T, tmpDir string) string
		wantHash string
	}{
		{
			desc: "ignore_empty_dir",
			setup: func(t *testing.T, tmpDir string) string {
				if err := os.Mkdir(filepath.Join(tmpDir, "empty"), 0777); err != nil {
					t.Fatal(err)
				}
				return tmpDir
			},
			wantHash: emptyHash,
		}, {
			desc: "ignore_symlink",
			setup: func(t *testing.T, tmpDir string) string {
				if err := os.Symlink(tmpDir, filepath.Join(tmpDir, "link")); err != nil {
					switch runtime.GOOS {
					case "aix", "android", "darwin", "dragonfly", "freebsd", "illumos", "ios", "js", "linux", "netbsd", "openbsd", "solaris":
						// Symlinks in tmpDir are always expected to work on these platforms.
						t.Fatal(err)
					default:
						t.Skipf("could not create symlink: %v", err)
					}
				}
				return tmpDir
			},
			wantHash: emptyHash,
		}, {
			desc: "dir_is_vendor",
			setup: func(t *testing.T, tmpDir string) string {
				vendorDir := filepath.Join(tmpDir, "vendor")
				if err := os.Mkdir(vendorDir, 0777); err != nil {
					t.Fatal(err)
				}
				goModData := []byte("module example.com/m\n\ngo 1.13\n")
				if err := os.WriteFile(filepath.Join(vendorDir, "go.mod"), goModData, 0666); err != nil {
					t.Fatal(err)
				}
				return vendorDir
			},
			wantHash: "h1:XduFAgX/GaspZa8Jv4pfzoGEzNaU/r88PiCunijw5ok=",
		},
	} {
		t.Run(test.desc, func(t *testing.T) {
			dir := test.setup(t, t.TempDir())

			tmpZipFile, err := os.CreateTemp(t.TempDir(), "TestCreateFromDir-*.zip")
			if err != nil {
				t.Fatal(err)
			}
			tmpZipPath := tmpZipFile.Name()
			defer tmpZipFile.Close()

			m := module.Version{Path: "example.com/m", Version: "v1.0.0"}
			if err := modzip.CreateFromDir(tmpZipFile, m, dir); err != nil {
				t.Fatal(err)
			}
			if err := tmpZipFile.Close(); err != nil {
				t.Fatal(err)
			}

			if hash, err := dirhash.HashZip(tmpZipPath, dirhash.Hash1); err != nil {
				t.Fatal(err)
			} else if hash != test.wantHash {
				t.Fatalf("got hash %q; want %q", hash, emptyHash)
			}
		})
	}
}

func TestUnzip(t *testing.T) {
	testDir := filepath.FromSlash("testdata/unzip")
	testEntries, err := os.ReadDir(testDir)
	if err != nil {
		t.Fatal(err)
	}
	for _, testEntry := range testEntries {
		base := filepath.Base(testEntry.Name())
		if filepath.Ext(base) != ".txt" {
			continue
		}
		t.Run(base[:len(base)-len(".txt")], func(t *testing.T) {
			// Load the test.
			testPath := filepath.Join(testDir, testEntry.Name())
			test, err := readTest(testPath)
			if err != nil {
				t.Fatal(err)
			}

			// Convert txtar to temporary zip file.
			tmpZipPath, err := extractTxtarToTempZip(t, test.archive)
			if err != nil {
				t.Fatal(err)
			}

			// Extract to a temporary directory.
			tmpDir := t.TempDir()
			m := module.Version{Path: test.path, Version: test.version}
			if err := modzip.Unzip(tmpDir, m, tmpZipPath); err != nil {
				if test.wantErr == "" {
					t.Fatalf("unexpected error: %v", err)
				} else if !strings.Contains(err.Error(), test.wantErr) {
					t.Fatalf("got error %q; want error containing %q", err.Error(), test.wantErr)
				} else {
					return
				}
			} else if test.wantErr != "" {
				t.Fatalf("unexpected success; wanted error containing %q", test.wantErr)
			}

			// Hash the directory, compare to known value.
			prefix := fmt.Sprintf("%s@%s/", test.path, test.version)
			if hash, err := dirhash.HashDir(tmpDir, prefix, dirhash.Hash1); err != nil {
				t.Fatal(err)
			} else if hash != test.hash {
				t.Fatalf("got hash %q\nwant: %q", hash, test.hash)
			}
		})
	}
}

type sizeLimitTest struct {
	desc              string
	files             []modzip.File
	wantErr           string
	wantCheckFilesErr string
	wantCreateErr     string
	wantCheckZipErr   string
	wantUnzipErr      string
}

// sizeLimitTests is shared by TestCreateSizeLimits and TestUnzipSizeLimits.
var sizeLimitTests = [...]sizeLimitTest{
	{
		desc: "one_large",
		files: []modzip.File{fakeFile{
			name: "large.go",
			size: modzip.MaxZipFile,
		}},
	}, {
		desc: "one_too_large",
		files: []modzip.File{fakeFile{
			name: "large.go",
			size: modzip.MaxZipFile + 1,
		}},
		wantCheckFilesErr: "module source tree too large",
		wantCreateErr:     "module source tree too large",
		wantCheckZipErr:   "total uncompressed size of module contents too large",
		wantUnzipErr:      "total uncompressed size of module contents too large",
	}, {
		desc: "total_large",
		files: []modzip.File{
			fakeFile{
				name: "small.go",
				size: 10,
			},
			fakeFile{
				name: "large.go",
				size: modzip.MaxZipFile - 10,
			},
		},
	}, {
		desc: "total_too_large",
		files: []modzip.File{
			fakeFile{
				name: "small.go",
				size: 10,
			},
			fakeFile{
				name: "large.go",
				size: modzip.MaxZipFile - 9,
			},
		},
		wantCheckFilesErr: "module source tree too large",
		wantCreateErr:     "module source tree too large",
		wantCheckZipErr:   "total uncompressed size of module contents too large",
		wantUnzipErr:      "total uncompressed size of module contents too large",
	}, {
		desc: "large_gomod",
		files: []modzip.File{fakeFile{
			name: "go.mod",
			size: modzip.MaxGoMod,
		}},
	}, {
		desc: "too_large_gomod",
		files: []modzip.File{fakeFile{
			name: "go.mod",
			size: modzip.MaxGoMod + 1,
		}},
		wantErr: "go.mod file too large",
	}, {
		desc: "large_license",
		files: []modzip.File{fakeFile{
			name: "LICENSE",
			size: modzip.MaxLICENSE,
		}},
	}, {
		desc: "too_large_license",
		files: []modzip.File{fakeFile{
			name: "LICENSE",
			size: modzip.MaxLICENSE + 1,
		}},
		wantErr: "LICENSE file too large",
	},
}

var sizeLimitVersion = module.Version{Path: "example.com/large", Version: "v1.0.0"}

func TestCreateSizeLimits(t *testing.T) {
	if testing.Short() {
		t.Skip("creating large files takes time")
	}
	tests := append(sizeLimitTests[:], sizeLimitTest{
		// negative file size may happen when size is represented as uint64
		// but is cast to int64, as is the case in zip files.
		desc: "negative",
		files: []modzip.File{fakeFile{
			name: "neg.go",
			size: 0x8000000000000000,
		}},
		wantErr: "module source tree too large",
	}, sizeLimitTest{
		desc: "size_is_a_lie",
		files: []modzip.File{fakeFile{
			name: "lie.go",
			size: 1,
			data: []byte(`package large`),
		}},
		wantCreateErr: "larger than declared size",
	})

	for _, test := range tests {
		test := test
		t.Run(test.desc, func(t *testing.T) {
			t.Parallel()

			wantCheckFilesErr := test.wantCheckFilesErr
			if wantCheckFilesErr == "" {
				wantCheckFilesErr = test.wantErr
			}
			if _, err := modzip.CheckFiles(test.files); err == nil && wantCheckFilesErr != "" {
				t.Fatalf("CheckFiles: unexpected success; want error containing %q", wantCheckFilesErr)
			} else if err != nil && wantCheckFilesErr == "" {
				t.Fatalf("CheckFiles: got error %q; want success", err)
			} else if err != nil && !strings.Contains(err.Error(), wantCheckFilesErr) {
				t.Fatalf("CheckFiles: got error %q; want error containing %q", err, wantCheckFilesErr)
			}

			wantCreateErr := test.wantCreateErr
			if wantCreateErr == "" {
				wantCreateErr = test.wantErr
			}
			if err := modzip.Create(io.Discard, sizeLimitVersion, test.files); err == nil && wantCreateErr != "" {
				t.Fatalf("Create: unexpected success; want error containing %q", wantCreateErr)
			} else if err != nil && wantCreateErr == "" {
				t.Fatalf("Create: got error %q; want success", err)
			} else if err != nil && !strings.Contains(err.Error(), wantCreateErr) {
				t.Fatalf("Create: got error %q; want error containing %q", err, wantCreateErr)
			}
		})
	}
}

func TestUnzipSizeLimits(t *testing.T) {
	if testing.Short() {
		t.Skip("creating large files takes time")
	}
	for _, test := range sizeLimitTests {
		test := test
		t.Run(test.desc, func(t *testing.T) {
			t.Parallel()
			tmpZipFile, err := os.CreateTemp(t.TempDir(), "TestUnzipSizeLimits-*.zip")
			if err != nil {
				t.Fatal(err)
			}
			tmpZipPath := tmpZipFile.Name()
			defer tmpZipFile.Close()

			zw := zip.NewWriter(tmpZipFile)
			prefix := fmt.Sprintf("%s@%s/", sizeLimitVersion.Path, sizeLimitVersion.Version)
			for _, tf := range test.files {
				zf, err := zw.Create(prefix + tf.Path())
				if err != nil {
					t.Fatal(err)
				}
				rc, err := tf.Open()
				if err != nil {
					t.Fatal(err)
				}
				_, err = io.Copy(zf, rc)
				rc.Close()
				if err != nil {
					t.Fatal(err)
				}
			}
			if err := zw.Close(); err != nil {
				t.Fatal(err)
			}
			if err := tmpZipFile.Close(); err != nil {
				t.Fatal(err)
			}

			wantCheckZipErr := test.wantCheckZipErr
			if wantCheckZipErr == "" {
				wantCheckZipErr = test.wantErr
			}
			cf, err := modzip.CheckZip(sizeLimitVersion, tmpZipPath)
			if err == nil {
				err = cf.Err()
			}
			if err == nil && wantCheckZipErr != "" {
				t.Fatalf("CheckZip: unexpected success; want error containing %q", wantCheckZipErr)
			} else if err != nil && wantCheckZipErr == "" {
				t.Fatalf("CheckZip: got error %q; want success", err)
			} else if err != nil && !strings.Contains(err.Error(), wantCheckZipErr) {
				t.Fatalf("CheckZip: got error %q; want error containing %q", err, wantCheckZipErr)
			}

			wantUnzipErr := test.wantUnzipErr
			if wantUnzipErr == "" {
				wantUnzipErr = test.wantErr
			}
			if err := modzip.Unzip(t.TempDir(), sizeLimitVersion, tmpZipPath); err == nil && wantUnzipErr != "" {
				t.Fatalf("Unzip: unexpected success; want error containing %q", wantUnzipErr)
			} else if err != nil && wantUnzipErr == "" {
				t.Fatalf("Unzip: got error %q; want success", err)
			} else if err != nil && !strings.Contains(err.Error(), wantUnzipErr) {
				t.Fatalf("Unzip: got error %q; want error containing %q", err, wantUnzipErr)
			}
		})
	}
}

func TestUnzipSizeLimitsSpecial(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping test; creating large files takes time")
	}

	for _, test := range []struct {
		desc               string
		wantErr1, wantErr2 string
		m                  module.Version
		writeZip           func(t *testing.T, zipFile *os.File)
	}{
		{
			desc: "large_zip",
			m:    module.Version{Path: "example.com/m", Version: "v1.0.0"},
			writeZip: func(t *testing.T, zipFile *os.File) {
				if err := zipFile.Truncate(modzip.MaxZipFile); err != nil {
					t.Fatal(err)
				}
			},
			// this is not an error we care about; we're just testing whether
			// Unzip checks the size of the file before opening.
			// It's harder to create a valid zip file of exactly the right size.
			wantErr1: "not a valid zip file",
		}, {
			desc: "too_large_zip",
			m:    module.Version{Path: "example.com/m", Version: "v1.0.0"},
			writeZip: func(t *testing.T, zipFile *os.File) {
				if err := zipFile.Truncate(modzip.MaxZipFile + 1); err != nil {
					t.Fatal(err)
				}
			},
			wantErr1: "module zip file is too large",
		}, {
			desc: "size_is_a_lie",
			m:    module.Version{Path: "example.com/m", Version: "v1.0.0"},
			writeZip: func(t *testing.T, zipFile *os.File) {
				// Create a normal zip file in memory containing one file full of zero
				// bytes. Use a distinctive size so we can find it later.
				zipBuf := &bytes.Buffer{}
				zw := zip.NewWriter(zipBuf)
				f, err := zw.Create("example.com/m@v1.0.0/go.mod")
				if err != nil {
					t.Fatal(err)
				}
				realSize := 0x0BAD
				buf := make([]byte, realSize)
				if _, err := f.Write(buf); err != nil {
					t.Fatal(err)
				}
				if err := zw.Close(); err != nil {
					t.Fatal(err)
				}

				// Replace the uncompressed size of the file. As a shortcut, we just
				// search-and-replace the byte sequence. It should occur twice because
				// the 32- and 64-byte sizes are stored separately. All multi-byte
				// values are little-endian.
				zipData := zipBuf.Bytes()
				realSizeData := []byte{0xAD, 0x0B}
				fakeSizeData := []byte{0xAC, 0x00}
				s := zipData
				n := 0
				for {
					if i := bytes.Index(s, realSizeData); i < 0 {
						break
					} else {
						s = s[i:]
					}
					copy(s[:len(fakeSizeData)], fakeSizeData)
					n++
				}
				if n != 2 {
					t.Fatalf("replaced size %d times; expected 2", n)
				}

				// Write the modified zip to the actual file.
				if _, err := zipFile.Write(zipData); err != nil {
					t.Fatal(err)
				}
			},
			// wantErr1 is for 1.18 and earlier,
			// wantErr2 is for 1.19 and later.
			wantErr1: "uncompressed size of file example.com/m@v1.0.0/go.mod is larger than declared size",
			wantErr2: "not a valid zip file",
		},
	} {
		test := test
		t.Run(test.desc, func(t *testing.T) {
			t.Parallel()
			tmpZipFile, err := os.CreateTemp(t.TempDir(), "TestUnzipSizeLimitsSpecial-*.zip")
			if err != nil {
				t.Fatal(err)
			}
			tmpZipPath := tmpZipFile.Name()
			defer tmpZipFile.Close()

			test.writeZip(t, tmpZipFile)
			if err := tmpZipFile.Close(); err != nil {
				t.Fatal(err)
			}

			want := func() string {
				s := fmt.Sprintf("%q", test.wantErr1)
				if test.wantErr2 != "" {
					s = fmt.Sprintf("%q or %q", test.wantErr1, test.wantErr2)
				}
				return s
			}

			if err := modzip.Unzip(t.TempDir(), test.m, tmpZipPath); err == nil && test.wantErr1 != "" {
				t.Fatalf("unexpected success; want error containing %s", want())
			} else if err != nil && test.wantErr1 == "" {
				t.Fatalf("got error %q; want success", err)
			} else if err != nil && !strings.Contains(err.Error(), test.wantErr1) && (test.wantErr2 == "" || !strings.Contains(err.Error(), test.wantErr2)) {
				t.Fatalf("got error %q; want error containing %s", err, want())
			}
		})
	}
}

// TestVCS clones a repository, creates a zip for a known version,
// and verifies the zip file itself has the same SHA-256 hash as the one
// 'go mod download' produces.
//
// This test is intended to build confidence that this implementation produces
// the same output as the go command, given the same VCS zip input. This is
// not intended to be a complete conformance test. The code that produces zip
// archives from VCS repos is based on the go command, but it's for testing
// only, and we don't export it.
//
// Note that we test the hash of the zip file itself. This is stricter than
// testing the hash of the content, which is what we've promised users.
// It's okay if the zip hash changes without changing the content hash, but
// we should not let that happen accidentally.
func TestVCS(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping VCS cloning in -short mode")
	}

	var downloadErrorCount int32
	const downloadErrorLimit = 3

	_, gitErr := gitPath()
	_, hgErr := exec.LookPath("hg")
	haveVCS := map[string]bool{
		"git": gitErr == nil,
		"hg":  hgErr == nil,
	}

	for _, test := range []struct {
		m                            module.Version
		vcs, url, subdir, rev        string
		wantContentHash, wantZipHash string
	}{
		// Simple tests: all versions of rsc.io/quote + newer major versions
		{
			m:               module.Version{Path: "rsc.io/quote", Version: "v1.0.0"},
			vcs:             "git",
			url:             "https://github.com/rsc/quote",
			rev:             "v1.0.0",
			wantContentHash: "h1:haUSojyo3j2M9g7CEUFG8Na09dtn7QKxvPGaPVQdGwM=",
			wantZipHash:     "5c08ba2c09a364f93704aaa780e7504346102c6ef4fe1333a11f09904a732078",
		},
		{
			m:               module.Version{Path: "rsc.io/quote", Version: "v1.1.0"},
			vcs:             "git",
			url:             "https://github.com/rsc/quote",
			rev:             "v1.1.0",
			wantContentHash: "h1:n/ElL9GOlVEwL0mVjzaYj0UxTI/TX9aQ7lR5LHqP/Rw=",
			wantZipHash:     "730a5ae6e5c4e216e4f84bb93aa9785a85630ad73f96954ebb5f9daa123dcaa9",
		},
		{
			m:               module.Version{Path: "rsc.io/quote", Version: "v1.2.0"},
			vcs:             "git",
			url:             "https://github.com/rsc/quote",
			rev:             "v1.2.0",
			wantContentHash: "h1:fFMCNi0A97hfNrtUZVQKETbuc3h7bmfFQHnjutpPYCg=",
			wantZipHash:     "fe1bd62652e9737a30d6b7fd396ea13e54ad13fb05f295669eb63d6d33290b06",
		},
		{
			m:               module.Version{Path: "rsc.io/quote", Version: "v1.2.1"},
			vcs:             "git",
			url:             "https://github.com/rsc/quote",
			rev:             "v1.2.1",
			wantContentHash: "h1:l+HtgC05eds8qgXNApuv6g1oK1q3B144BM5li1akqXY=",
			wantZipHash:     "9f0e74de55a6bd20c1567a81e707814dc221f07df176af2a0270392c6faf32fd",
		},
		{
			m:               module.Version{Path: "rsc.io/quote", Version: "v1.3.0"},
			vcs:             "git",
			url:             "https://github.com/rsc/quote",
			rev:             "v1.3.0",
			wantContentHash: "h1:aPUoHx/0Cd7BTZs4SAaknT4TaKryH766GcFTvJjVbHU=",
			wantZipHash:     "03872ee7d6747bc2ee0abadbd4eb09e60f6df17d0a6142264abe8a8a00af50e7",
		},
		{
			m:               module.Version{Path: "rsc.io/quote", Version: "v1.4.0"},
			vcs:             "git",
			url:             "https://github.com/rsc/quote",
			rev:             "v1.4.0",
			wantContentHash: "h1:tYuJspOzwTRMUOX6qmSDRTEKFVV80GM0/l89OLZuVNg=",
			wantZipHash:     "f60be8193c607bf197da01da4bedb3d683fe84c30de61040eb5d7afaf7869f2e",
		},
		{
			m:               module.Version{Path: "rsc.io/quote", Version: "v1.5.0"},
			vcs:             "git",
			url:             "https://github.com/rsc/quote",
			rev:             "v1.5.0",
			wantContentHash: "h1:mVjf/WMWxfIw299sOl/O3EXn5qEaaJPMDHMsv7DBDlw=",
			wantZipHash:     "a2d281834ce159703540da94425fa02c7aec73b88b560081ed0d3681bfe9cd1f",
		},
		{
			m:               module.Version{Path: "rsc.io/quote", Version: "v1.5.1"},
			vcs:             "git",
			url:             "https://github.com/rsc/quote",
			rev:             "v1.5.1",
			wantContentHash: "h1:ptSemFtffEBvMed43o25vSUpcTVcqxfXU8Jv0sfFVJs=",
			wantZipHash:     "4ecd78a6d9f571e84ed2baac1688fd150400db2c5b017b496c971af30aaece02",
		},
		{
			m:               module.Version{Path: "rsc.io/quote", Version: "v1.5.2"},
			vcs:             "git",
			url:             "https://github.com/rsc/quote",
			rev:             "v1.5.2",
			wantContentHash: "h1:w5fcysjrx7yqtD/aO+QwRjYZOKnaM9Uh2b40tElTs3Y=",
			wantZipHash:     "643fcf8ef4e4cbb8f910622c42df3f9a81f3efe8b158a05825a81622c121ca0a",
		},
		{
			m:               module.Version{Path: "rsc.io/quote", Version: "v1.5.3-pre1"},
			vcs:             "git",
			url:             "https://github.com/rsc/quote",
			rev:             "v1.5.3-pre1",
			wantContentHash: "h1:c3EJ21kn75/hyrOL/Dvj45+ifxGFSY8Wf4WBcoWTxF0=",
			wantZipHash:     "24106f0f15384949df51fae5d34191bf120c3b80c1c904721ca2872cf83126b2",
		},
		{
			m:               module.Version{Path: "rsc.io/quote/v2", Version: "v2.0.1"},
			vcs:             "git",
			url:             "https://github.com/rsc/quote",
			rev:             "v2.0.1",
			wantContentHash: "h1:DF8hmGbDhgiIa2tpqLjHLIKkJx6WjCtLEqZBAU+hACI=",
			wantZipHash:     "009ed42474a59526fe56a14a9dd02bd7f977d1bd3844398bd209d0da0484aade",
		},
		{
			m:               module.Version{Path: "rsc.io/quote/v3", Version: "v3.0.0"},
			vcs:             "git",
			url:             "https://github.com/rsc/quote",
			rev:             "v3.0.0",
			subdir:          "v3",
			wantContentHash: "h1:OEIXClZHFMyx5FdatYfxxpNEvxTqHlu5PNdla+vSYGg=",
			wantZipHash:     "cf3ff89056b785d7b3ef3a10e984efd83b47d9e65eabe8098b927b3370d5c3eb",
		},

		// Test cases from vcs-test.golang.org
		{
			m:               module.Version{Path: "vcs-test.golang.org/git/v3pkg.git/v3", Version: "v3.0.0"},
			vcs:             "git",
			url:             "https://vcs-test.golang.org/git/v3pkg",
			rev:             "v3.0.0",
			wantContentHash: "h1:mZhljS1BaiW8lODR6wqY5pDxbhXja04rWPFXPwRAtvA=",
			wantZipHash:     "9c65f0d235e531008dc04e977f6fa5d678febc68679bb63d4148dadb91d3fe57",
		},
		{
			m:               module.Version{Path: "vcs-test.golang.org/go/custom-hg-hello", Version: "v0.0.0-20171010233936-a8c8e7a40da9"},
			vcs:             "hg",
			url:             "https://vcs-test.golang.org/hg/custom-hg-hello",
			rev:             "a8c8e7a40da9",
			wantContentHash: "h1:LU6jFCbwn5VVgTcj+y4LspOpJHLZvl5TGPE+LwwpMw4=",
			wantZipHash:     "a1b12047da979d618c639ee98f370767a13d0507bd77785dc2f8dad66b40e2e6",
		},

		// Latest versions of selected golang.org/x repos
		{
			m:               module.Version{Path: "golang.org/x/arch", Version: "v0.0.0-20190927153633-4e8777c89be4"},
			vcs:             "git",
			url:             "https://go.googlesource.com/arch",
			rev:             "4e8777c89be4d9e61691fbe5d4e6c8838a7806f3",
			wantContentHash: "h1:QlVATYS7JBoZMVaf+cNjb90WD/beKVHnIxFKT4QaHVI=",
			wantZipHash:     "d17551a0c4957180ec1507065d13dcdd0f5cd8bfd7dd735fb81f64f3e2b31b68",
		},
		{
			m:               module.Version{Path: "golang.org/x/blog", Version: "v0.0.0-20191017104857-0cd0cdff05c2"},
			vcs:             "git",
			url:             "https://go.googlesource.com/blog",
			rev:             "0cd0cdff05c251ad0c796cc94d7059e013311fc6",
			wantContentHash: "h1:IKGICrORhR1aH2xG/WqrnpggSNolSj5urQxggCfmj28=",
			wantZipHash:     "0fed6b400de54da34b52b464ef2cdff45167236aaaf9a99ba8eba8855036faff",
		},
		{
			m:               module.Version{Path: "golang.org/x/crypto", Version: "v0.0.0-20191011191535-87dc89f01550"},
			vcs:             "git",
			url:             "https://go.googlesource.com/crypto",
			rev:             "87dc89f01550277dc22b74ffcf4cd89fa2f40f4c",
			wantContentHash: "h1:ObdrDkeb4kJdCP557AjRjq69pTHfNouLtWZG7j9rPN8=",
			wantZipHash:     "88e47aa05eb25c6abdad7387ccccfc39e74541896d87b7b1269e9dd2fa00100d",
		},
		{
			m:               module.Version{Path: "golang.org/x/net", Version: "v0.0.0-20191014212845-da9a3fd4c582"},
			vcs:             "git",
			url:             "https://go.googlesource.com/net",
			rev:             "da9a3fd4c5820e74b24a6cb7fb438dc9b0dd377c",
			wantContentHash: "h1:p9xBe/w/OzkeYVKm234g55gMdD1nSIooTir5kV11kfA=",
			wantZipHash:     "34901a85e6c15475a40457c2393ce66fb0999accaf2d6aa5b64b4863751ddbde",
		},
		{
			m:               module.Version{Path: "golang.org/x/sync", Version: "v0.0.0-20190911185100-cd5d95a43a6e"},
			vcs:             "git",
			url:             "https://go.googlesource.com/sync",
			rev:             "cd5d95a43a6e21273425c7ae415d3df9ea832eeb",
			wantContentHash: "h1:vcxGaoTs7kV8m5Np9uUNQin4BrLOthgV7252N8V+FwY=",
			wantZipHash:     "9c63fe51b0c533b258d3acc30d9319fe78679ce1a051109c9dea3105b93e2eef",
		},
		{
			m:               module.Version{Path: "golang.org/x/sys", Version: "v0.0.0-20191010194322-b09406accb47"},
			vcs:             "git",
			url:             "https://go.googlesource.com/sys",
			rev:             "b09406accb4736d857a32bf9444cd7edae2ffa79",
			wantContentHash: "h1:/XfQ9z7ib8eEJX2hdgFTZJ/ntt0swNk5oYBziWeTCvY=",
			wantZipHash:     "f26f2993757670b4d1fee3156d331513259757f17133a36966c158642c3f61df",
		},
		{
			m:               module.Version{Path: "golang.org/x/talks", Version: "v0.0.0-20191010201600-067e0d331fee"},
			vcs:             "git",
			url:             "https://go.googlesource.com/talks",
			rev:             "067e0d331feee4f8d0fa17d47444db533bd904e7",
			wantContentHash: "h1:8fnBMBUwliuiHuzfFw6kSSx79AzQpqkjZi3FSNIoqYs=",
			wantZipHash:     "fab2129f3005f970dbf2247378edb3220f6bd36726acdc7300ae3bb0f129e2f2",
		},
		{
			m:               module.Version{Path: "golang.org/x/tools", Version: "v0.0.0-20191017205301-920acffc3e65"},
			vcs:             "git",
			url:             "https://go.googlesource.com/tools",
			rev:             "920acffc3e65862cb002dae6b227b8d9695e3d29",
			wantContentHash: "h1:GwXwgmbrvlcHLDsENMqrQTTIC2C0kIPszsq929NruKI=",
			wantZipHash:     "7f0ab7466448190f8ad1b8cfb05787c3fb08f4a8f9953cd4b40a51c76ddebb28",
		},
		{
			m:               module.Version{Path: "golang.org/x/tour", Version: "v0.0.0-20191002171047-6bb846ce41cd"},
			vcs:             "git",
			url:             "https://go.googlesource.com/tour",
			rev:             "6bb846ce41cdca087b14c8e3560a679691c424b6",
			wantContentHash: "h1:EUlK3Rq8iTkQERnCnveD654NvRJ/ZCM9XCDne+S5cJ8=",
			wantZipHash:     "d6a7e03e02e5f7714bd12653d319a3b0f6e1099c01b1f9a17bc3613fb31c9170",
		},
	} {
		test := test
		testName := strings.ReplaceAll(test.m.String(), "/", "_")
		t.Run(testName, func(t *testing.T) {
			if have, ok := haveVCS[test.vcs]; !ok {
				t.Fatalf("unknown vcs: %s", test.vcs)
			} else if !have {
				t.Skipf("no %s executable in path", test.vcs)
			}
			t.Parallel()

			repo, dl, err := downloadVCSZip(t, test.vcs, test.url, test.rev, test.subdir)
			if err != nil {
				// This may fail if there's a problem with the network or upstream
				// repository. The package being tested doesn't directly interact with
				// VCS tools; the test just does this to simulate what the go command
				// does. So an error should cause a skip instead of a failure. But we
				// should fail after too many errors so we don't lose test coverage
				// when something changes permanently.
				n := atomic.AddInt32(&downloadErrorCount, 1)
				if n < downloadErrorLimit {
					t.Skipf("failed to download zip from repository: %v", err)
				} else {
					t.Fatalf("failed to download zip from repository (repeated failure): %v", err)
				}
			}

			// Create a module zip from that archive.
			// (adapted from cmd/go/internal/modfetch.codeRepo.Zip)
			info, err := dl.Stat()
			if err != nil {
				t.Fatal(err)
			}
			zr, err := zip.NewReader(dl, info.Size())
			if err != nil {
				t.Fatal(err)
			}

			var files []modzip.File
			topPrefix := ""
			subdir := test.subdir
			if subdir != "" && !strings.HasSuffix(subdir, "/") {
				subdir += "/"
			}
			haveLICENSE := false
			for _, f := range zr.File {
				if !f.FileInfo().Mode().IsRegular() {
					continue
				}
				if topPrefix == "" {
					i := strings.Index(f.Name, "/")
					if i < 0 {
						t.Fatal("missing top-level directory prefix")
					}
					topPrefix = f.Name[:i+1]
				}
				if strings.HasSuffix(f.Name, "/") { // drop directory dummy entries
					continue
				}
				if !strings.HasPrefix(f.Name, topPrefix) {
					t.Fatal("zip file contains more than one top-level directory")
				}
				name := strings.TrimPrefix(f.Name, topPrefix)
				if !strings.HasPrefix(name, subdir) {
					continue
				}
				name = strings.TrimPrefix(name, subdir)
				if name == ".hg_archival.txt" {
					// Inserted by hg archive.
					// Not correct to drop from other version control systems, but too bad.
					continue
				}
				if name == "LICENSE" {
					haveLICENSE = true
				}
				files = append(files, zipFile{name: name, f: f})
			}
			if !haveLICENSE && subdir != "" {
				license, err := downloadVCSFile(t, test.vcs, repo, test.rev, "LICENSE")
				if err != nil {
					t.Fatal(err)
				}
				files = append(files, fakeFile{
					name: "LICENSE",
					size: uint64(len(license)),
					data: license,
				})
			}

			tmpModZipFile, err := os.CreateTemp(t.TempDir(), "TestVCS-*.zip")
			if err != nil {
				t.Fatal(err)
			}
			tmpModZipPath := tmpModZipFile.Name()
			defer tmpModZipFile.Close()
			h := sha256.New()
			w := io.MultiWriter(tmpModZipFile, h)
			if err := modzip.Create(w, test.m, files); err != nil {
				t.Fatal(err)
			}
			if err := tmpModZipFile.Close(); err != nil {
				t.Fatal(err)
			}

			gotZipHash := hex.EncodeToString(h.Sum(nil))
			if test.wantZipHash != gotZipHash {
				// If the test fails because the hash of the zip file itself differs,
				// that may be okay as long as the hash of the data within the zip file
				// does not change. For example, we might change the compression,
				// order, or alignment of files without affecting the extracted output.
				// We shouldn't make such a change unintentionally though, so this
				// test will fail either way.
				if gotSum, err := dirhash.HashZip(tmpModZipPath, dirhash.Hash1); err == nil && test.wantContentHash != gotSum {
					t.Fatalf("zip content hash: got %s, want %s", gotSum, test.wantContentHash)
				} else {
					t.Fatalf("zip file hash: got %s, want %s", gotZipHash, test.wantZipHash)
				}
			}
		})
	}
}

func downloadVCSZip(t testing.TB, vcs, url, rev, subdir string) (repoDir string, dl *os.File, err error) {
	repoDir = t.TempDir()

	switch vcs {
	case "git":
		// Create a repository and download the revision we want.
		if _, err := runWithGitDir(t, repoDir, repoDir, "git", "init", "--bare"); err != nil {
			return "", nil, err
		}
		if err := os.MkdirAll(filepath.Join(repoDir, "info"), 0777); err != nil {
			return "", nil, err
		}
		attrFile, err := os.OpenFile(filepath.Join(repoDir, "info", "attributes"), os.O_CREATE|os.O_APPEND|os.O_RDWR, 0666)
		if err != nil {
			return "", nil, err
		}
		if _, err := attrFile.Write([]byte("\n* -export-subst -export-ignore\n")); err != nil {
			attrFile.Close()
			return "", nil, err
		}
		if err := attrFile.Close(); err != nil {
			return "", nil, err
		}
		if _, err := runWithGitDir(t, repoDir, repoDir, "git", "remote", "add", "origin", "--", url); err != nil {
			return "", nil, err
		}
		var refSpec string
		if strings.HasPrefix(rev, "v") {
			refSpec = fmt.Sprintf("refs/tags/%[1]s:refs/tags/%[1]s", rev)
		} else {
			refSpec = fmt.Sprintf("%s:refs/dummy", rev)
		}
		if _, err := runWithGitDir(t, repoDir, repoDir, "git", "fetch", "-f", "--depth=1", "origin", refSpec); err != nil {
			return "", nil, err
		}

		// Create an archive.
		tmpZipFile, err := os.CreateTemp(t.TempDir(), "downloadVCSZip-*.zip")
		if err != nil {
			return "", nil, err
		}
		t.Cleanup(func() { tmpZipFile.Close() })
		subdirArg := subdir
		if subdir == "" {
			subdirArg = "."
		}

		cmd := exec.Command("git", "-c", "core.autocrlf=input", "-c", "core.eol=lf", "archive", "--format=zip", "--prefix=prefix/", rev, "--", subdirArg)
		cmd.Dir = repoDir
		cmd.Env = append(cmd.Environ(), "GIT_DIR="+repoDir)
		cmd.Stdout = tmpZipFile
		stderr := new(strings.Builder)
		cmd.Stderr = stderr

		err = cmd.Run()
		if stderr.Len() > 0 && (err != nil || testing.Verbose()) {
			t.Logf("%v: %v\n%s", err, cmd, stderr)
		} else if err != nil {
			t.Logf("%v: %v", err, cmd)
		} else {
			t.Logf("%v", cmd)
		}
		if err != nil {
			return "", nil, err
		}

		if _, err := tmpZipFile.Seek(0, 0); err != nil {
			return "", nil, err
		}
		return repoDir, tmpZipFile, nil

	case "hg":
		// Clone the whole repository.
		if _, err := run(t, repoDir, "hg", "clone", "-U", "--", url, "."); err != nil {
			return "", nil, err
		}

		// Create an archive.
		tmpZipFile, err := os.CreateTemp(t.TempDir(), "downloadVCSZip-*.zip")
		if err != nil {
			return "", nil, err
		}
		tmpZipPath := tmpZipFile.Name()
		tmpZipFile.Close()
		args := []string{"archive", "-t", "zip", "--no-decode", "-r", rev, "--prefix=prefix/"}
		if subdir != "" {
			args = append(args, "-I", subdir+"/**")
		}
		args = append(args, "--", tmpZipPath)
		if _, err := run(t, repoDir, "hg", args...); err != nil {
			return "", nil, err
		}
		if tmpZipFile, err = os.Open(tmpZipPath); err != nil {
			return "", nil, err
		}
		t.Cleanup(func() { tmpZipFile.Close() })
		return repoDir, tmpZipFile, err

	default:
		return "", nil, fmt.Errorf("vcs %q not supported", vcs)
	}
}

func downloadVCSFile(t testing.TB, vcs, repo, rev, file string) ([]byte, error) {
	t.Helper()
	switch vcs {
	case "git":
		return runWithGitDir(t, repo, repo, "git", "cat-file", "blob", rev+":"+file)
	default:
		return nil, fmt.Errorf("vcs %q not supported", vcs)
	}
}

func runWithGitDir(t testing.TB, gitDir, dir string, name string, args ...string) ([]byte, error) {
	t.Helper()

	cmd := exec.Command(name, args...)
	cmd.Dir = dir
	if gitDir != "" {
		cmd.Env = append(cmd.Environ(), "GIT_DIR="+gitDir)
	}
	stderr := new(strings.Builder)
	cmd.Stderr = stderr

	out, err := cmd.Output()
	if stderr.Len() > 0 && (err != nil || testing.Verbose()) {
		t.Logf("%v: %v\n%s", err, cmd, stderr)
	} else if err != nil {
		t.Logf("%v: %v", err, cmd)
	} else {
		t.Logf("%v", cmd)
	}
	return out, err
}

func run(t testing.TB, dir string, name string, args ...string) ([]byte, error) {
	t.Helper()

	return runWithGitDir(t, "", dir, name, args...)
}

type zipFile struct {
	name string
	f    *zip.File
}

func (f zipFile) Path() string                 { return f.name }
func (f zipFile) Lstat() (os.FileInfo, error)  { return f.f.FileInfo(), nil }
func (f zipFile) Open() (io.ReadCloser, error) { return f.f.Open() }

func TestCreateFromVCS_basic(t *testing.T) {
	mustHaveGit(t)

	// Write files to a temporary directory.
	tmpDir, err := extractTxtarToTempDir(t, txtar.Parse([]byte(`-- go.mod --
module example.com/foo/bar

go 1.12
-- LICENSE --
root license
-- a.go --
package a

var A = 5
-- b.go --
package a

var B = 5
-- c/c.go --
package c

var C = 5
-- d/d.go --
package c

var D = 5
-- e/LICENSE --
e license
-- e/e.go --
package e

var E = 5
-- f/go.mod --
module example.com/foo/bar/f

go 1.12
-- f/f.go --
package f

var F = 5
-- .gitignore --
b.go
c/`)))
	if err != nil {
		t.Fatal(err)
	}

	gitInit(t, tmpDir)
	gitCommit(t, tmpDir)

	for _, tc := range []struct {
		desc      string
		version   module.Version
		subdir    string
		wantFiles []string
		wantData  map[string]string
	}{
		{
			desc:      "from root",
			version:   module.Version{Path: "example.com/foo/bar", Version: "v0.0.1"},
			subdir:    "",
			wantFiles: []string{"go.mod", "LICENSE", "a.go", "d/d.go", "e/LICENSE", "e/e.go", ".gitignore"},
			wantData:  map[string]string{"LICENSE": "root license\n"},
		},
		{
			desc:    "from subdir",
			version: module.Version{Path: "example.com/foo/bar", Version: "v0.0.1"},
			subdir:  "d/",
			// Note: File paths are zipped as if the subdir were the root. ie d.go instead of d/d.go.
			// subdirs without a license hoist the license from the root
			wantFiles: []string{"d.go", "LICENSE"},
			wantData:  map[string]string{"LICENSE": "root license\n"},
		},
		{
			desc:    "from subdir with license",
			version: module.Version{Path: "example.com/foo/bar", Version: "v0.0.1"},
			subdir:  "e/",
			// Note: File paths are zipped as if the subdir were the root. ie e.go instead of e/e.go.
			// subdirs with a license use their own
			wantFiles: []string{"LICENSE", "e.go"},
			wantData:  map[string]string{"LICENSE": "e license\n"},
		},
		{
			desc:    "from submodule subdir",
			version: module.Version{Path: "example.com/foo/bar/f", Version: "v0.0.1"},
			subdir:  "f/",
			// Note: File paths are zipped as if the subdir were the root. ie f.go instead of f/f.go.
			// subdirs without a license hoist the license from the root
			wantFiles: []string{"go.mod", "f.go", "LICENSE"},
			wantData:  map[string]string{"LICENSE": "root license\n"},
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			// Create zip from the directory.
			tmpZip := &bytes.Buffer{}

			if err := modzip.CreateFromVCS(tmpZip, tc.version, tmpDir, "HEAD", tc.subdir); err != nil {
				t.Fatal(err)
			}

			wantData := map[string]string{}
			for f, data := range tc.wantData {
				p := path.Join(tc.version.String(), f)
				wantData[p] = data
			}

			readerAt := bytes.NewReader(tmpZip.Bytes())
			r, err := zip.NewReader(readerAt, int64(tmpZip.Len()))
			if err != nil {
				t.Fatal(err)
			}
			var gotFiles []string
			gotMap := map[string]bool{}
			for _, f := range r.File {
				gotMap[f.Name] = true
				gotFiles = append(gotFiles, f.Name)

				if want, ok := wantData[f.Name]; ok {
					r, err := f.Open()
					if err != nil {
						t.Errorf("CreatedFromVCS: error opening %s: %v", f.Name, err)
						continue
					}
					defer r.Close()
					got, err := io.ReadAll(r)
					if err != nil {
						t.Errorf("CreatedFromVCS: error reading %s: %v", f.Name, err)
						continue
					}
					if want != string(got) {
						t.Errorf("CreatedFromVCS: zipped file %s contains %s, expected %s", f.Name, string(got), want)
						continue
					}
				}
			}
			wantMap := map[string]bool{}
			for _, f := range tc.wantFiles {
				p := path.Join(tc.version.String(), f)
				wantMap[p] = true
			}

			// The things that should be there.
			for f := range gotMap {
				if !wantMap[f] {
					t.Errorf("CreatedFromVCS: zipped file contains %s, but expected it not to", f)
				}
			}

			// The things that are missing.
			for f := range wantMap {
				if !gotMap[f] {
					t.Errorf("CreatedFromVCS: zipped file doesn't contain %s, but expected it to. all files: %v", f, gotFiles)
				}
			}
			for f := range wantData {
				if !gotMap[f] {
					t.Errorf("CreatedFromVCS: zipped file doesn't contain %s, but expected it to. all files: %v", f, gotFiles)
				}
			}
		})
	}
}

// Test what the experience of creating a zip from a v2 module is like.
func TestCreateFromVCS_v2(t *testing.T) {
	mustHaveGit(t)

	// Write files to a temporary directory.
	tmpDir, err := extractTxtarToTempDir(t, txtar.Parse([]byte(`-- go.mod --
module example.com/foo/bar

go 1.12
-- a.go --
package a

var A = 5
-- b.go --
package a

var B = 5
-- go.mod --
module example.com/foo/bar

go 1.12
-- gaz/v2/a_2.go --
package a

var C = 9
-- gaz/v2/b_2.go --
package a

var B = 11
-- gaz/v2/go.mod --
module example.com/foo/bar/v2

go 1.12
-- .gitignore --
`)))
	if err != nil {
		t.Fatal(err)
	}

	gitInit(t, tmpDir)
	gitCommit(t, tmpDir)

	// Create zip from the directory.
	tmpZip := &bytes.Buffer{}

	m := module.Version{Path: "example.com/foo/bar/v2", Version: "v2.0.0"}

	if err := modzip.CreateFromVCS(tmpZip, m, tmpDir, "HEAD", "gaz/v2"); err != nil {
		t.Fatal(err)
	}

	readerAt := bytes.NewReader(tmpZip.Bytes())
	r, err := zip.NewReader(readerAt, int64(tmpZip.Len()))
	if err != nil {
		t.Fatal(err)
	}
	var gotFiles []string
	gotMap := map[string]bool{}
	for _, f := range r.File {
		gotMap[f.Name] = true
		gotFiles = append(gotFiles, f.Name)
	}
	wantMap := map[string]bool{
		"example.com/foo/bar/v2@v2.0.0/a_2.go": true,
		"example.com/foo/bar/v2@v2.0.0/b_2.go": true,
		"example.com/foo/bar/v2@v2.0.0/go.mod": true,
	}

	// The things that should be there.
	for f := range gotMap {
		if !wantMap[f] {
			t.Errorf("CreatedFromVCS: zipped file contains %s, but expected it not to", f)
		}
	}

	// The things that are missing.
	for f := range wantMap {
		if !gotMap[f] {
			t.Errorf("CreatedFromVCS: zipped file doesn't contain %s, but expected it to. all files: %v", f, gotFiles)
		}
	}
}

func TestCreateFromVCS_nonGitDir(t *testing.T) {
	mustHaveGit(t)

	// Write files to a temporary directory.
	tmpDir, err := extractTxtarToTempDir(t, txtar.Parse([]byte(`-- go.mod --
module example.com/foo/bar

go 1.12
-- a.go --
package a

var A = 5
`)))
	if err != nil {
		t.Fatal(err)
	}

	// Create zip from the directory.
	tmpZip, err := os.CreateTemp(t.TempDir(), "TestCreateFromDir-*.zip")
	if err != nil {
		t.Fatal(err)
	}
	defer tmpZip.Close()

	m := module.Version{Path: "example.com/foo/bar", Version: "v0.0.1"}

	err = modzip.CreateFromVCS(tmpZip, m, tmpDir, "HEAD", "")
	if err == nil {
		t.Fatal("CreateFromVCS: expected error, got nil")
	}
	var gotErr *modzip.UnrecognizedVCSError
	if !errors.As(err, &gotErr) {
		t.Errorf("CreateFromVCS: returned error does not unwrap to modzip.UnrecognizedVCSError, but expected it to. returned error: %v", err)
	} else if gotErr.RepoRoot != tmpDir {
		t.Errorf("CreateFromVCS: returned error has RepoRoot %q, but want %q. returned error: %v", gotErr.RepoRoot, tmpDir, err)
	}
}

func TestCreateFromVCS_zeroCommitsGitDir(t *testing.T) {
	mustHaveGit(t)

	// Write files to a temporary directory.
	tmpDir, err := extractTxtarToTempDir(t, txtar.Parse([]byte(`-- go.mod --
module example.com/foo/bar

go 1.12
-- a.go --
package a

var A = 5
`)))
	if err != nil {
		t.Fatal(err)
	}

	gitInit(t, tmpDir)

	// Create zip from the directory.
	tmpZip, err := os.CreateTemp(t.TempDir(), "TestCreateFromDir-*.zip")
	if err != nil {
		t.Fatal(err)
	}
	defer tmpZip.Close()

	m := module.Version{Path: "example.com/foo/bar", Version: "v0.0.1"}

	if err := modzip.CreateFromVCS(tmpZip, m, tmpDir, "HEAD", ""); err == nil {
		t.Error("CreateFromVCS: expected error, got nil")
	}
}

// gitInit runs "git init" at the specified dir.
//
// Note: some environments - and trybots - don't have git installed. This
// function will cause the calling test to be skipped if that's the case.
func gitInit(t testing.TB, dir string) {
	t.Helper()
	mustHaveGit(t)

	if _, err := run(t, dir, "git", "init"); err != nil {
		t.Fatal(err)
	}
	if _, err := run(t, dir, "git", "config", "user.name", "Go Gopher"); err != nil {
		t.Fatal(err)
	}
	if _, err := run(t, dir, "git", "config", "user.email", "gopher@golang.org"); err != nil {
		t.Fatal(err)
	}
}

func gitCommit(t testing.TB, dir string) {
	t.Helper()
	mustHaveGit(t)

	if _, err := run(t, dir, "git", "add", "-A"); err != nil {
		t.Fatal(err)
	}
	if _, err := run(t, dir, "git", "commit", "-m", "some commit"); err != nil {
		t.Fatal(err)
	}
}

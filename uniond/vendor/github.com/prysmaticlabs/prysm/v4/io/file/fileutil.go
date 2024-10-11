package file

import (
	"crypto/sha256"
	"encoding/base64"
	"fmt"
	"io"
	"os"
	"os/user"
	"path"
	"path/filepath"
	"sort"
	"strings"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	log "github.com/sirupsen/logrus"
)

// ExpandPath given a string which may be a relative path.
// 1. replace tilde with users home dir
// 2. expands embedded environment variables
// 3. cleans the path, e.g. /a/b/../c -> /a/c
// Note, it has limitations, e.g. ~someuser/tmp will not be expanded
func ExpandPath(p string) (string, error) {
	if strings.HasPrefix(p, "~/") || strings.HasPrefix(p, "~\\") {
		if home := HomeDir(); home != "" {
			p = home + p[1:]
		}
	}
	return filepath.Abs(path.Clean(os.ExpandEnv(p)))
}

// HandleBackupDir takes an input directory path and either alters its permissions to be usable if it already exists, creates it if not
func HandleBackupDir(dirPath string, permissionOverride bool) error {
	expanded, err := ExpandPath(dirPath)
	if err != nil {
		return err
	}
	exists, err := HasDir(expanded)
	if err != nil {
		return err
	}
	if exists {
		info, err := os.Stat(expanded)
		if err != nil {
			return err
		}
		if info.Mode().Perm() != params.BeaconIoConfig().ReadWriteExecutePermissions {
			if permissionOverride {
				if err := os.Chmod(expanded, params.BeaconIoConfig().ReadWriteExecutePermissions); err != nil {
					return err
				}
			} else {
				return errors.New("dir already exists without proper 0700 permissions")
			}
		}
	}
	return os.MkdirAll(expanded, params.BeaconIoConfig().ReadWriteExecutePermissions)
}

// MkdirAll takes in a path, expands it if necessary, and creates the directory accordingly
// with standardized, Prysm project permissions. If a directory already exists as this path,
// then the method returns without making any changes. This is the static-analysis enforced
// method for creating a directory programmatically in Prysm.
func MkdirAll(dirPath string) error {
	expanded, err := ExpandPath(dirPath)
	if err != nil {
		return err
	}
	exists, err := HasDir(expanded)
	if err != nil {
		return err
	}
	if exists {
		return nil
	}
	return os.MkdirAll(expanded, params.BeaconIoConfig().ReadWriteExecutePermissions)
}

// WriteFile is the static-analysis enforced method for writing binary data to a file
// in Prysm, enforcing a single entrypoint with standardized permissions.
func WriteFile(file string, data []byte) error {
	expanded, err := ExpandPath(file)
	if err != nil {
		return err
	}
	if Exists(expanded) {
		info, err := os.Stat(expanded)
		if err != nil {
			return err
		}
		if info.Mode() != params.BeaconIoConfig().ReadWritePermissions {
			return errors.New("file already exists without proper 0600 permissions")
		}
	}
	return os.WriteFile(expanded, data, params.BeaconIoConfig().ReadWritePermissions)
}

// HomeDir for a user.
func HomeDir() string {
	if home := os.Getenv("HOME"); home != "" {
		return home
	}
	if usr, err := user.Current(); err == nil {
		return usr.HomeDir
	}
	return ""
}

// HasDir checks if a directory indeed exists at the specified path.
func HasDir(dirPath string) (bool, error) {
	fullPath, err := ExpandPath(dirPath)
	if err != nil {
		return false, err
	}
	info, err := os.Stat(fullPath)
	if os.IsNotExist(err) {
		return false, nil
	}
	if info == nil {
		return false, err
	}
	return info.IsDir(), err
}

// HasReadWritePermissions checks if file at a path has proper
// 0600 permissions set.
func HasReadWritePermissions(itemPath string) (bool, error) {
	info, err := os.Stat(itemPath)
	if err != nil {
		return false, err
	}
	return info.Mode() == params.BeaconIoConfig().ReadWritePermissions, nil
}

// Exists returns true if a file is not a directory and exists
// at the specified path.
func Exists(filename string) bool {
	filePath, err := ExpandPath(filename)
	if err != nil {
		return false
	}
	info, err := os.Stat(filePath)
	if err != nil {
		if !os.IsNotExist(err) {
			log.WithError(err).Info("Checking for file existence returned an error")
		}
		return false
	}
	return info != nil && !info.IsDir()
}

// RecursiveFileFind returns true, and the path,  if a file is not a directory and exists
// at  dir or any of its subdirectories.  Finds the first instant based on the Walk order and returns.
// Define non-fatal error to stop the recursive directory walk
var errStopWalk = errors.New("stop walking")

// RecursiveFileFind searches for file in a directory and its subdirectories.
func RecursiveFileFind(filename, dir string) (bool, string, error) {
	var found bool
	var fpath string
	dir = filepath.Clean(dir)
	found = false
	err := filepath.Walk(dir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		// checks if its a file  and has the exact name as the filename
		// need to break the walk function by using a non-fatal error
		if !info.IsDir() && filename == info.Name() {
			found = true
			fpath = path
			return errStopWalk
		}

		// no errors or file found
		return nil
	})
	if err != nil && err != errStopWalk {
		return false, "", err
	}
	return found, fpath, nil
}

// ReadFileAsBytes expands a file name's absolute path and reads it as bytes from disk.
func ReadFileAsBytes(filename string) ([]byte, error) {
	filePath, err := ExpandPath(filename)
	if err != nil {
		return nil, errors.Wrap(err, "could not determine absolute path of password file")
	}
	return os.ReadFile(filePath) // #nosec G304
}

// CopyFile copy a file from source to destination path.
func CopyFile(src, dst string) error {
	if !Exists(src) {
		return errors.New("source file does not exist at provided path")
	}
	f, err := os.Open(src) // #nosec G304
	if err != nil {
		return err
	}
	dstFile, err := os.OpenFile(dst, os.O_WRONLY|os.O_CREATE|os.O_TRUNC, params.BeaconIoConfig().ReadWritePermissions) // #nosec G304
	if err != nil {
		return err
	}
	_, err = io.Copy(dstFile, f)
	return err
}

// CopyDir copies contents of one directory into another, recursively.
func CopyDir(src, dst string) error {
	dstExists, err := HasDir(dst)
	if err != nil {
		return err
	}
	if dstExists {
		return errors.New("destination directory already exists")
	}
	fds, err := os.ReadDir(src)
	if err != nil {
		return err
	}
	if err := MkdirAll(dst); err != nil {
		return errors.Wrapf(err, "error creating directory: %s", dst)
	}
	for _, fd := range fds {
		srcPath := path.Join(src, fd.Name())
		dstPath := path.Join(dst, fd.Name())
		if fd.IsDir() {
			if err = CopyDir(srcPath, dstPath); err != nil {
				return errors.Wrapf(err, "error copying directory %s -> %s", srcPath, dstPath)
			}
		} else {
			if err = CopyFile(srcPath, dstPath); err != nil {
				return errors.Wrapf(err, "error copying file %s -> %s", srcPath, dstPath)
			}
		}
	}
	return nil
}

// DirsEqual checks whether two directories have the same content.
func DirsEqual(src, dst string) bool {
	hash1, err := HashDir(src)
	if err != nil {
		return false
	}

	hash2, err := HashDir(dst)
	if err != nil {
		return false
	}

	return hash1 == hash2
}

// HashDir calculates and returns hash of directory: each file's hash is calculated and saved along
// with the file name into the list, after which list is hashed to produce the final signature.
// Implementation is based on https://github.com/golang/mod/blob/release-branch.go1.15/sumdb/dirhash/hash.go
func HashDir(dir string) (string, error) {
	files, err := DirFiles(dir)
	if err != nil {
		return "", err
	}

	h := sha256.New()
	files = append([]string(nil), files...)
	sort.Strings(files)
	for _, file := range files {
		hf, err := HashFile(filepath.Join(dir, file))
		if err != nil {
			return "", err
		}
		if _, err := fmt.Fprintf(h, "%x  %s\n", hf, file); err != nil {
			return "", err
		}
	}
	return "hashdir:" + base64.StdEncoding.EncodeToString(h.Sum(nil)), nil
}

// HashFile calculates and returns the hash of a file.
func HashFile(filePath string) ([]byte, error) {
	f, err := os.Open(filepath.Clean(filePath))
	if err != nil {
		return nil, err
	}
	hf := sha256.New()
	if _, err := io.Copy(hf, f); err != nil {
		return nil, err
	}
	err = f.Close()
	if err != nil {
		return nil, err
	}
	return hf.Sum(nil), nil
}

// DirFiles returns list of files found within a given directory and its sub-directories.
// Directory prefix will not be included as a part of returned file string i.e. for a file located
// in "dir/foo/bar" only "foo/bar" part will be returned.
func DirFiles(dir string) ([]string, error) {
	var files []string
	dir = filepath.Clean(dir)
	err := filepath.Walk(dir, func(file string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if info.IsDir() {
			return nil
		}
		relFile := file
		if dir != "." {
			relFile = file[len(dir)+1:]
		}
		files = append(files, filepath.ToSlash(relFile))
		return nil
	})
	if err != nil {
		return nil, err
	}
	return files, nil
}

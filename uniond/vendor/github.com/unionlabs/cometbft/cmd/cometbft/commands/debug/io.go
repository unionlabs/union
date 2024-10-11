package debug

import (
	"archive/zip"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"path"
	"path/filepath"
	"strings"
)

// zipDir zips all the contents found in src, including both files and
// directories, into a destination file dest. It returns an error upon failure.
// It assumes src is a directory.
func zipDir(src, dest string) error {
	zipFile, err := os.Create(dest)
	if err != nil {
		return err
	}
	defer zipFile.Close()

	zipWriter := zip.NewWriter(zipFile)
	defer zipWriter.Close()

	dirName := filepath.Base(dest)
	baseDir := strings.TrimSuffix(dirName, filepath.Ext(dirName))

	return filepath.Walk(src, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		header, err := zip.FileInfoHeader(info)
		if err != nil {
			return err
		}

		// Each execution of this utility on a CometBFT process will result in a
		// unique file.
		header.Name = filepath.Join(baseDir, strings.TrimPrefix(path, src))

		// Handle cases where the content to be zipped is a file or a directory,
		// where a directory must have a '/' suffix.
		if info.IsDir() {
			header.Name += "/"
		} else {
			header.Method = zip.Deflate
		}

		headerWriter, err := zipWriter.CreateHeader(header)
		if err != nil {
			return err
		}

		if info.IsDir() {
			return nil
		}

		file, err := os.Open(path)
		if err != nil {
			return err
		}
		defer file.Close()

		_, err = io.Copy(headerWriter, file)
		return err
	})
}

// copyFile copies a file from src to dest and returns an error upon failure. The
// copied file retains the source file's permissions.
func copyFile(src, dest string) error {
	if _, err := os.Stat(src); os.IsNotExist(err) {
		return err
	}

	srcFile, err := os.Open(src)
	if err != nil {
		return err
	}
	defer srcFile.Close()

	destFile, err := os.Create(dest)
	if err != nil {
		return err
	}
	defer destFile.Close()

	if _, err = io.Copy(destFile, srcFile); err != nil {
		return err
	}

	srcInfo, err := os.Stat(src)
	if err != nil {
		return err
	}

	return os.Chmod(dest, srcInfo.Mode())
}

// writeStateToFile pretty JSON encodes an object and writes it to file composed
// of dir and filename. It returns an error upon failure to encode or write to
// file.
func writeStateJSONToFile(state any, dir, filename string) error {
	stateJSON, err := json.MarshalIndent(state, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to encode state dump: %w", err)
	}

	return os.WriteFile(path.Join(dir, filename), stateJSON, os.ModePerm)
}

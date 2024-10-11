// Package dirhash is a thin forwarding layer on top of
// [golang.org/x/mod/sumdb/dirhash]. See that package for documentation.
//
// Deprecated: use [golang.org/x/mod/sumdb/dirhash] instead.
package dirhash

import (
	"io"

	"golang.org/x/mod/sumdb/dirhash"
)

var DefaultHash = dirhash.Hash1

type Hash = dirhash.Hash

func Hash1(files []string, open func(string) (io.ReadCloser, error)) (string, error) {
	return dirhash.Hash1(files, open)
}

func HashDir(dir, prefix string, hash Hash) (string, error) {
	return dirhash.HashDir(dir, prefix, hash)
}

func DirFiles(dir, prefix string) ([]string, error) {
	return dirhash.DirFiles(dir, prefix)
}

func HashZip(zipfile string, hash Hash) (string, error) {
	return dirhash.HashZip(zipfile, hash)
}

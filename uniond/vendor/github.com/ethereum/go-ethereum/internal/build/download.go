// Copyright 2019 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package build

import (
	"bufio"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"path/filepath"
	"strings"
)

// ChecksumDB keeps file checksums.
type ChecksumDB struct {
	allChecksums []string
}

// MustLoadChecksums loads a file containing checksums.
func MustLoadChecksums(file string) *ChecksumDB {
	content, err := os.ReadFile(file)
	if err != nil {
		log.Fatal("can't load checksum file: " + err.Error())
	}
	return &ChecksumDB{strings.Split(string(content), "\n")}
}

// Verify checks whether the given file is valid according to the checksum database.
func (db *ChecksumDB) Verify(path string) error {
	fd, err := os.Open(path)
	if err != nil {
		return err
	}
	defer fd.Close()

	h := sha256.New()
	if _, err := io.Copy(h, bufio.NewReader(fd)); err != nil {
		return err
	}
	fileHash := hex.EncodeToString(h.Sum(nil))
	if !db.findHash(filepath.Base(path), fileHash) {
		return fmt.Errorf("invalid file hash %s for %s", fileHash, filepath.Base(path))
	}
	return nil
}

func (db *ChecksumDB) findHash(basename, hash string) bool {
	want := hash + "  " + basename
	for _, line := range db.allChecksums {
		if strings.TrimSpace(line) == want {
			return true
		}
	}
	return false
}

// DownloadFile downloads a file and verifies its checksum.
func (db *ChecksumDB) DownloadFile(url, dstPath string) error {
	if err := db.Verify(dstPath); err == nil {
		fmt.Printf("%s is up-to-date\n", dstPath)
		return nil
	}
	fmt.Printf("%s is stale\n", dstPath)
	fmt.Printf("downloading from %s\n", url)

	resp, err := http.Get(url)
	if err != nil {
		return fmt.Errorf("download error: %v", err)
	} else if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("download error: status %d", resp.StatusCode)
	}
	defer resp.Body.Close()
	if err := os.MkdirAll(filepath.Dir(dstPath), 0755); err != nil {
		return err
	}
	fd, err := os.OpenFile(dstPath, os.O_CREATE|os.O_TRUNC|os.O_WRONLY, 0644)
	if err != nil {
		return err
	}
	dst := newDownloadWriter(fd, resp.ContentLength)
	_, err = io.Copy(dst, resp.Body)
	dst.Close()
	if err != nil {
		return err
	}

	return db.Verify(dstPath)
}

type downloadWriter struct {
	file    *os.File
	dstBuf  *bufio.Writer
	size    int64
	written int64
	lastpct int64
}

func newDownloadWriter(dst *os.File, size int64) *downloadWriter {
	return &downloadWriter{
		file:   dst,
		dstBuf: bufio.NewWriter(dst),
		size:   size,
	}
}

func (w *downloadWriter) Write(buf []byte) (int, error) {
	n, err := w.dstBuf.Write(buf)

	// Report progress.
	w.written += int64(n)
	pct := w.written * 10 / w.size * 10
	if pct != w.lastpct {
		if w.lastpct != 0 {
			fmt.Print("...")
		}
		fmt.Print(pct, "%")
		w.lastpct = pct
	}
	return n, err
}

func (w *downloadWriter) Close() error {
	if w.lastpct > 0 {
		fmt.Println() // Finish the progress line.
	}
	flushErr := w.dstBuf.Flush()
	closeErr := w.file.Close()
	if flushErr != nil {
		return flushErr
	}
	return closeErr
}

// Copyright 2018 The go-ethereum Authors
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

//go:build none
// +build none

/*
This command generates GPL license headers on top of all source files.
You can run it once per month, before cutting a release or just
whenever you feel like it.

	go run update-license.go

All authors (people who have contributed code) are listed in the
AUTHORS file. The author names are mapped and deduplicated using the
.mailmap file. You can use .mailmap to set the canonical name and
address for each author. See git-shortlog(1) for an explanation of the
.mailmap format.

Please review the resulting diff to check whether the correct
copyright assignments are performed.
*/

package main

import (
	"bufio"
	"bytes"
	"fmt"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"runtime"
	"strconv"
	"strings"
	"sync"
	"text/template"
	"time"

	"golang.org/x/exp/slices"
)

var (
	// only files with these extensions will be considered
	extensions = []string{".go", ".js", ".qml"}

	// paths with any of these prefixes will be skipped
	skipPrefixes = []string{
		// boring stuff
		"vendor/", "tests/testdata/", "build/",

		// don't relicense vendored sources
		"cmd/internal/browser",
		"common/bitutil/bitutil",
		"common/prque/",
		"consensus/ethash/xor.go",
		"crypto/blake2b/",
		"crypto/bn256/",
		"crypto/bls12381/",
		"crypto/ecies/",
		"graphql/graphiql.go",
		"internal/jsre/deps",
		"log/",
		"metrics/",
		"signer/rules/deps",

		// skip special licenses
		"crypto/secp256k1", // Relicensed to BSD-3 via https://github.com/ethereum/go-ethereum/pull/17225
	}

	// paths with this prefix are licensed as GPL. all other files are LGPL.
	gplPrefixes = []string{"cmd/"}

	// this regexp must match the entire license comment at the
	// beginning of each file.
	licenseCommentRE = regexp.MustCompile(`^//\s*(Copyright|This file is part of).*?\n(?://.*?\n)*\n*`)

	// this text appears at the start of AUTHORS
	authorsFileHeader = "# This is the official list of go-ethereum authors for copyright purposes.\n\n"
)

// this template generates the license comment.
// its input is an info structure.
var licenseT = template.Must(template.New("").Parse(`
// Copyright {{.Year}} The go-ethereum Authors
// This file is part of {{.Whole false}}.
//
// {{.Whole true}} is free software: you can redistribute it and/or modify
// it under the terms of the GNU {{.License}} as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// {{.Whole true}} is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU {{.License}} for more details.
//
// You should have received a copy of the GNU {{.License}}
// along with {{.Whole false}}. If not, see <http://www.gnu.org/licenses/>.

`[1:]))

type info struct {
	file string
	Year int64
}

func (i info) License() string {
	if i.gpl() {
		return "General Public License"
	}
	return "Lesser General Public License"
}

func (i info) ShortLicense() string {
	if i.gpl() {
		return "GPL"
	}
	return "LGPL"
}

func (i info) Whole(startOfSentence bool) string {
	if i.gpl() {
		return "go-ethereum"
	}
	if startOfSentence {
		return "The go-ethereum library"
	}
	return "the go-ethereum library"
}

func (i info) gpl() bool {
	for _, p := range gplPrefixes {
		if strings.HasPrefix(i.file, p) {
			return true
		}
	}
	return false
}

func main() {
	var (
		files = getFiles()
		filec = make(chan string)
		infoc = make(chan *info, 20)
		wg    sync.WaitGroup
	)

	writeAuthors(files)

	go func() {
		for _, f := range files {
			filec <- f
		}
		close(filec)
	}()
	for i := runtime.NumCPU(); i >= 0; i-- {
		// getting file info is slow and needs to be parallel.
		// it traverses git history for each file.
		wg.Add(1)
		go getInfo(filec, infoc, &wg)
	}
	go func() {
		wg.Wait()
		close(infoc)
	}()
	writeLicenses(infoc)
}

func skipFile(path string) bool {
	if strings.Contains(path, "/testdata/") {
		return true
	}
	for _, p := range skipPrefixes {
		if strings.HasPrefix(path, p) {
			return true
		}
	}
	return false
}

func getFiles() []string {
	cmd := exec.Command("git", "ls-tree", "-r", "--name-only", "HEAD")
	var files []string
	err := doLines(cmd, func(line string) {
		if skipFile(line) {
			return
		}
		ext := filepath.Ext(line)
		for _, wantExt := range extensions {
			if ext == wantExt {
				goto keep
			}
		}
		return
	keep:
		files = append(files, line)
	})
	if err != nil {
		log.Fatal("error getting files:", err)
	}
	return files
}

var authorRegexp = regexp.MustCompile(`\s*[0-9]+\s*(.*)`)

func gitAuthors(files []string) []string {
	cmds := []string{"shortlog", "-s", "-n", "-e", "HEAD", "--"}
	cmds = append(cmds, files...)
	cmd := exec.Command("git", cmds...)
	var authors []string
	err := doLines(cmd, func(line string) {
		m := authorRegexp.FindStringSubmatch(line)
		if len(m) > 1 {
			authors = append(authors, m[1])
		}
	})
	if err != nil {
		log.Fatalln("error getting authors:", err)
	}
	return authors
}

func readAuthors() []string {
	content, err := os.ReadFile("AUTHORS")
	if err != nil && !os.IsNotExist(err) {
		log.Fatalln("error reading AUTHORS:", err)
	}
	var authors []string
	for _, a := range bytes.Split(content, []byte("\n")) {
		if len(a) > 0 && a[0] != '#' {
			authors = append(authors, string(a))
		}
	}
	// Retranslate existing authors through .mailmap.
	// This should catch email address changes.
	authors = mailmapLookup(authors)
	return authors
}

func mailmapLookup(authors []string) []string {
	if len(authors) == 0 {
		return nil
	}
	cmds := []string{"check-mailmap", "--"}
	cmds = append(cmds, authors...)
	cmd := exec.Command("git", cmds...)
	var translated []string
	err := doLines(cmd, func(line string) {
		translated = append(translated, line)
	})
	if err != nil {
		log.Fatalln("error translating authors:", err)
	}
	return translated
}

func writeAuthors(files []string) {
	var (
		dedup = make(map[string]bool)
		list  []string
	)
	// Add authors that Git reports as contributors.
	// This is the primary source of author information.
	for _, a := range gitAuthors(files) {
		if la := strings.ToLower(a); !dedup[la] {
			list = append(list, a)
			dedup[la] = true
		}
	}
	// Add existing authors from the file. This should ensure that we
	// never lose authors, even if Git stops listing them. We can also
	// add authors manually this way.
	for _, a := range readAuthors() {
		if la := strings.ToLower(a); !dedup[la] {
			list = append(list, a)
			dedup[la] = true
		}
	}
	// Write sorted list of authors back to the file.
	slices.SortFunc(list, func(a, b string) bool {
		return strings.ToLower(a) < strings.ToLower(b)
	})
	content := new(bytes.Buffer)
	content.WriteString(authorsFileHeader)
	for _, a := range list {
		content.WriteString(a)
		content.WriteString("\n")
	}
	fmt.Println("writing AUTHORS")
	if err := os.WriteFile("AUTHORS", content.Bytes(), 0644); err != nil {
		log.Fatalln(err)
	}
}

func getInfo(files <-chan string, out chan<- *info, wg *sync.WaitGroup) {
	for file := range files {
		stat, err := os.Lstat(file)
		if err != nil {
			fmt.Printf("ERROR %s: %v\n", file, err)
			continue
		}
		if !stat.Mode().IsRegular() {
			continue
		}
		if isGenerated(file) {
			continue
		}
		info, err := fileInfo(file)
		if err != nil {
			fmt.Printf("ERROR %s: %v\n", file, err)
			continue
		}
		out <- info
	}
	wg.Done()
}

func isGenerated(file string) bool {
	fd, err := os.Open(file)
	if err != nil {
		return false
	}
	defer fd.Close()
	buf := make([]byte, 2048)
	n, err := fd.Read(buf)
	if err != nil {
		return false
	}
	buf = buf[:n]
	for _, l := range bytes.Split(buf, []byte("\n")) {
		if bytes.HasPrefix(l, []byte("// Code generated")) {
			return true
		}
	}
	return false
}

// fileInfo finds the lowest year in which the given file was committed.
func fileInfo(file string) (*info, error) {
	info := &info{file: file, Year: int64(time.Now().Year())}
	cmd := exec.Command("git", "log", "--follow", "--find-renames=80", "--find-copies=80", "--pretty=format:%ai", "--", file)
	err := doLines(cmd, func(line string) {
		y, err := strconv.ParseInt(line[:4], 10, 64)
		if err != nil {
			fmt.Printf("cannot parse year: %q", line[:4])
		}
		if y < info.Year {
			info.Year = y
		}
	})
	return info, err
}

func writeLicenses(infos <-chan *info) {
	for i := range infos {
		writeLicense(i)
	}
}

func writeLicense(info *info) {
	fi, err := os.Stat(info.file)
	if os.IsNotExist(err) {
		fmt.Println("skipping (does not exist)", info.file)
		return
	}
	if err != nil {
		log.Fatalf("error stat'ing %s: %v\n", info.file, err)
	}
	content, err := os.ReadFile(info.file)
	if err != nil {
		log.Fatalf("error reading %s: %v\n", info.file, err)
	}
	// Construct new file content.
	buf := new(bytes.Buffer)
	licenseT.Execute(buf, info)
	if m := licenseCommentRE.FindIndex(content); m != nil && m[0] == 0 {
		buf.Write(content[:m[0]])
		buf.Write(content[m[1]:])
	} else {
		buf.Write(content)
	}
	// Write it to the file.
	if bytes.Equal(content, buf.Bytes()) {
		fmt.Println("skipping (no changes)", info.file)
		return
	}
	fmt.Println("writing", info.ShortLicense(), info.file)
	if err := os.WriteFile(info.file, buf.Bytes(), fi.Mode()); err != nil {
		log.Fatalf("error writing %s: %v", info.file, err)
	}
}

func doLines(cmd *exec.Cmd, f func(string)) error {
	stdout, err := cmd.StdoutPipe()
	if err != nil {
		return err
	}
	if err := cmd.Start(); err != nil {
		return err
	}
	s := bufio.NewScanner(stdout)
	for s.Scan() {
		f(s.Text())
	}
	if s.Err() != nil {
		return s.Err()
	}
	if err := cmd.Wait(); err != nil {
		return fmt.Errorf("%v (for %s)", err, strings.Join(cmd.Args, " "))
	}
	return nil
}

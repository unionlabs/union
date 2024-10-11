// Copyright 2021 The go-ethereum Authors
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
	"fmt"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
)

type GoToolchain struct {
	Root string // GOROOT

	// Cross-compilation variables. These are set when running the go tool.
	GOARCH string
	GOOS   string
	CC     string
}

// Go creates an invocation of the go command.
func (g *GoToolchain) Go(command string, args ...string) *exec.Cmd {
	tool := g.goTool(command, args...)

	// Configure environment for cross build.
	if g.GOARCH != "" && g.GOARCH != runtime.GOARCH {
		tool.Env = append(tool.Env, "CGO_ENABLED=1")
		tool.Env = append(tool.Env, "GOARCH="+g.GOARCH)
	}
	if g.GOOS != "" && g.GOOS != runtime.GOOS {
		tool.Env = append(tool.Env, "GOOS="+g.GOOS)
	}
	// Configure C compiler.
	if g.CC != "" {
		tool.Env = append(tool.Env, "CC="+g.CC)
	} else if os.Getenv("CC") != "" {
		tool.Env = append(tool.Env, "CC="+os.Getenv("CC"))
	}
	// CKZG by default is not portable, append the necessary build flags to make
	// it not rely on modern CPU instructions and enable linking against.
	tool.Env = append(tool.Env, "CGO_CFLAGS=-O2 -g -D__BLST_PORTABLE__")

	return tool
}

func (g *GoToolchain) goTool(command string, args ...string) *exec.Cmd {
	if g.Root == "" {
		g.Root = runtime.GOROOT()
	}
	tool := exec.Command(filepath.Join(g.Root, "bin", "go"), command) // nolint: gosec
	tool.Args = append(tool.Args, args...)
	tool.Env = append(tool.Env, "GOROOT="+g.Root)

	// Forward environment variables to the tool, but skip compiler target settings.
	// TODO: what about GOARM?
	skip := map[string]struct{}{"GOROOT": {}, "GOARCH": {}, "GOOS": {}, "GOBIN": {}, "CC": {}}
	for _, e := range os.Environ() {
		if i := strings.IndexByte(e, '='); i >= 0 {
			if _, ok := skip[e[:i]]; ok {
				continue
			}
		}
		tool.Env = append(tool.Env, e)
	}
	return tool
}

// DownloadGo downloads the Go binary distribution and unpacks it into a temporary
// directory. It returns the GOROOT of the unpacked toolchain.
func DownloadGo(csdb *ChecksumDB) string {
	version, err := Version(csdb, "golang")
	if err != nil {
		log.Fatal(err)
	}
	// Shortcut: if the Go version that runs this script matches the
	// requested version exactly, there is no need to download anything.
	activeGo := strings.TrimPrefix(runtime.Version(), "go")
	if activeGo == version {
		log.Printf("-dlgo version matches active Go version %s, skipping download.", activeGo)
		return runtime.GOROOT()
	}

	ucache, err := os.UserCacheDir()
	if err != nil {
		log.Fatal(err)
	}

	// For Arm architecture, GOARCH includes ISA version.
	os := runtime.GOOS
	arch := runtime.GOARCH
	if arch == "arm" {
		arch = "armv6l"
	}
	file := fmt.Sprintf("go%s.%s-%s", version, os, arch)
	if os == "windows" {
		file += ".zip"
	} else {
		file += ".tar.gz"
	}
	url := "https://golang.org/dl/" + file
	dst := filepath.Join(ucache, file)
	if err := csdb.DownloadFile(url, dst); err != nil {
		log.Fatal(err)
	}

	godir := filepath.Join(ucache, fmt.Sprintf("geth-go-%s-%s-%s", version, os, arch))
	if err := ExtractArchive(dst, godir); err != nil {
		log.Fatal(err)
	}
	goroot, err := filepath.Abs(filepath.Join(godir, "go"))
	if err != nil {
		log.Fatal(err)
	}
	return goroot
}

// Version returns the versions defined in the checksumdb.
func Version(csdb *ChecksumDB, version string) (string, error) {
	for _, l := range csdb.allChecksums {
		if !strings.HasPrefix(l, "# version:") {
			continue
		}
		v := strings.Split(l, ":")[1]
		parts := strings.Split(v, " ")
		if len(parts) != 2 {
			log.Print("Erroneous version-string", "v", l)
			continue
		}
		if parts[0] == version {
			log.Printf("Found version %q", parts[1])
			return parts[1], nil
		}
	}
	return "", fmt.Errorf("no version found for '%v'", version)
}

// DownloadAndVerifyChecksums downloads all files and checks that they match
// the checksum given in checksums.txt.
// This task can be used to sanity-check new checksums.
func DownloadAndVerifyChecksums(csdb *ChecksumDB) {
	var (
		base   = ""
		ucache = os.TempDir()
	)
	for _, l := range csdb.allChecksums {
		if strings.HasPrefix(l, "# https://") {
			base = l[2:]
			continue
		}
		if strings.HasPrefix(l, "#") {
			continue
		}
		hashFile := strings.Split(l, "  ")
		if len(hashFile) != 2 {
			continue
		}
		file := hashFile[1]
		url := base + file
		dst := filepath.Join(ucache, file)
		if err := csdb.DownloadFile(url, dst); err != nil {
			log.Print(err)
		}
	}
}

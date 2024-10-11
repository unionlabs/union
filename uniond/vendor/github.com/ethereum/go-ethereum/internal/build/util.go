// Copyright 2016 The go-ethereum Authors
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
	"bytes"
	"flag"
	"fmt"
	"go/parser"
	"go/token"
	"io"
	"log"
	"os"
	"os/exec"
	"path"
	"path/filepath"
	"strconv"
	"strings"
	"text/template"
	"time"
)

var DryRunFlag = flag.Bool("n", false, "dry run, don't execute commands")

// MustRun executes the given command and exits the host process for
// any error.
func MustRun(cmd *exec.Cmd) {
	fmt.Println(">>>", printArgs(cmd.Args))
	if !*DryRunFlag {
		cmd.Stderr = os.Stderr
		cmd.Stdout = os.Stdout
		if err := cmd.Run(); err != nil {
			log.Fatal(err)
		}
	}
}

func printArgs(args []string) string {
	var s strings.Builder
	for i, arg := range args {
		if i > 0 {
			s.WriteByte(' ')
		}
		if strings.IndexByte(arg, ' ') >= 0 {
			arg = strconv.QuoteToASCII(arg)
		}
		s.WriteString(arg)
	}
	return s.String()
}

func MustRunCommand(cmd string, args ...string) {
	MustRun(exec.Command(cmd, args...))
}

var warnedAboutGit bool

// RunGit runs a git subcommand and returns its output.
// The command must complete successfully.
func RunGit(args ...string) string {
	cmd := exec.Command("git", args...)
	var stdout, stderr bytes.Buffer
	cmd.Stdout, cmd.Stderr = &stdout, &stderr
	if err := cmd.Run(); err != nil {
		if e, ok := err.(*exec.Error); ok && e.Err == exec.ErrNotFound {
			if !warnedAboutGit {
				log.Println("Warning: can't find 'git' in PATH")
				warnedAboutGit = true
			}
			return ""
		}
		log.Fatal(strings.Join(cmd.Args, " "), ": ", err, "\n", stderr.String())
	}
	return strings.TrimSpace(stdout.String())
}

// readGitFile returns content of file in .git directory.
func readGitFile(file string) string {
	content, err := os.ReadFile(path.Join(".git", file))
	if err != nil {
		return ""
	}
	return strings.TrimSpace(string(content))
}

// Render renders the given template file into outputFile.
func Render(templateFile, outputFile string, outputPerm os.FileMode, x interface{}) {
	tpl := template.Must(template.ParseFiles(templateFile))
	render(tpl, outputFile, outputPerm, x)
}

// RenderString renders the given template string into outputFile.
func RenderString(templateContent, outputFile string, outputPerm os.FileMode, x interface{}) {
	tpl := template.Must(template.New("").Parse(templateContent))
	render(tpl, outputFile, outputPerm, x)
}

func render(tpl *template.Template, outputFile string, outputPerm os.FileMode, x interface{}) {
	if err := os.MkdirAll(filepath.Dir(outputFile), 0755); err != nil {
		log.Fatal(err)
	}
	out, err := os.OpenFile(outputFile, os.O_CREATE|os.O_WRONLY|os.O_EXCL, outputPerm)
	if err != nil {
		log.Fatal(err)
	}
	if err := tpl.Execute(out, x); err != nil {
		log.Fatal(err)
	}
	if err := out.Close(); err != nil {
		log.Fatal(err)
	}
}

// UploadSFTP uploads files to a remote host using the sftp command line tool.
// The destination host may be specified either as [user@]host: or as a URI in
// the form sftp://[user@]host[:port].
func UploadSFTP(identityFile, host, dir string, files []string) error {
	sftp := exec.Command("sftp")
	sftp.Stderr = os.Stderr
	if identityFile != "" {
		sftp.Args = append(sftp.Args, "-i", identityFile)
	}
	sftp.Args = append(sftp.Args, host)
	fmt.Println(">>>", printArgs(sftp.Args))
	if *DryRunFlag {
		return nil
	}

	stdin, err := sftp.StdinPipe()
	if err != nil {
		return fmt.Errorf("can't create stdin pipe for sftp: %v", err)
	}
	stdout, err := sftp.StdoutPipe()
	if err != nil {
		return fmt.Errorf("can't create stdout pipe for sftp: %v", err)
	}
	if err := sftp.Start(); err != nil {
		return err
	}
	in := io.MultiWriter(stdin, os.Stdout)
	for _, f := range files {
		fmt.Fprintln(in, "put", f, path.Join(dir, filepath.Base(f)))
	}
	fmt.Fprintln(in, "exit")
	// Some issue with the PPA sftp server makes it so the server does not
	// respond properly to a 'bye', 'exit' or 'quit' from the client.
	// To work around that, we check the output, and when we see the client
	// exit command, we do a hard exit.
	// See
	// https://github.com/kolban-google/sftp-gcs/issues/23
	// https://github.com/mscdex/ssh2/pull/1111
	aborted := false
	go func() {
		scanner := bufio.NewScanner(stdout)
		for scanner.Scan() {
			txt := scanner.Text()
			fmt.Println(txt)
			if txt == "sftp> exit" {
				// Give it .5 seconds to exit (server might be fixed), then
				// hard kill it from the outside
				time.Sleep(500 * time.Millisecond)
				aborted = true
				sftp.Process.Kill()
			}
		}
	}()
	stdin.Close()
	err = sftp.Wait()
	if aborted {
		return nil
	}
	return err
}

// FindMainPackages finds all 'main' packages in the given directory and returns their
// package paths.
func FindMainPackages(dir string) []string {
	var commands []string
	cmds, err := os.ReadDir(dir)
	if err != nil {
		log.Fatal(err)
	}
	for _, cmd := range cmds {
		pkgdir := filepath.Join(dir, cmd.Name())
		if !cmd.IsDir() {
			continue
		}
		pkgs, err := parser.ParseDir(token.NewFileSet(), pkgdir, nil, parser.PackageClauseOnly)
		if err != nil {
			log.Fatal(err)
		}
		for name := range pkgs {
			if name == "main" {
				path := "./" + filepath.ToSlash(pkgdir)
				commands = append(commands, path)
				break
			}
		}
	}
	return commands
}

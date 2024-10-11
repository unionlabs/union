// Program tomledit provides basic command-line support for reading and
// modifying TOML files.
package main

import (
	"errors"
	"flag"
	"fmt"
	"os"
	"path/filepath"

	"github.com/creachadair/atomicfile"
	"github.com/creachadair/command"
	"github.com/creachadair/tomledit"
)

func main() {
	var cfg settings
	root := &command.C{
		Name: filepath.Base(os.Args[0]),
		Usage: `[options] command [args...]
help [command/topic]`,
		Help: `Read or modify the contents of a TOML file.

For commands accepting a value, TOML syntax is required.
As a shorthand for bare string values, prefix arguments with "@":
The argument @foo is parsed as if it were a basic string "foo".`,

		SetFlags: func(_ *command.Env, fs *flag.FlagSet) {
			fs.StringVar(&cfg.Path, "path", "", "Path of TOML file to process")
		},

		Commands: []*command.C{
			cmdList,
			cmdPrint,
			cmdSet,
			cmdAdd,
			command.HelpCommand(nil),
		},
	}
	command.RunOrFail(root.NewEnv(&cfg), os.Args[1:])
}

type settings struct {
	Path    string
	Replace bool
	Text    string
}

func (s *settings) loadDocument() (*tomledit.Document, error) {
	if s.Path == "" {
		return nil, errors.New("no input -path is set")
	}
	f, err := os.Open(s.Path)
	if err != nil {
		return nil, err
	}
	defer f.Close()
	return tomledit.Parse(f)
}

func (s *settings) saveDocument(doc *tomledit.Document) error {
	if s.Path == "" {
		return errors.New("no output -path is set")
	}
	return atomicfile.Tx(s.Path, 0600, func(f *atomicfile.File) error {
		if err := tomledit.Format(f, doc); err != nil {
			return fmt.Errorf("formatting output: %w", err)
		}
		return nil
	})
}

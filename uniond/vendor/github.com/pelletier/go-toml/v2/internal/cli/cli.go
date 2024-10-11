package cli

import (
	"bytes"
	"errors"
	"flag"
	"fmt"
	"io"
	"io/ioutil"
	"os"

	"github.com/pelletier/go-toml/v2"
)

type ConvertFn func(r io.Reader, w io.Writer) error

type Program struct {
	Usage string
	Fn    ConvertFn
	// Inplace allows the command to take more than one file as argument and
	// perform conversion in place on each provided file.
	Inplace bool
}

func (p *Program) Execute() {
	flag.Usage = func() { fmt.Fprintf(os.Stderr, p.Usage) }
	flag.Parse()
	os.Exit(p.main(flag.Args(), os.Stdin, os.Stdout, os.Stderr))
}

func (p *Program) main(files []string, input io.Reader, output, error io.Writer) int {
	err := p.run(files, input, output)
	if err != nil {

		var derr *toml.DecodeError
		if errors.As(err, &derr) {
			fmt.Fprintln(error, derr.String())
			row, col := derr.Position()
			fmt.Fprintln(error, "error occurred at row", row, "column", col)
		} else {
			fmt.Fprintln(error, err.Error())
		}

		return -1
	}
	return 0
}

func (p *Program) run(files []string, input io.Reader, output io.Writer) error {
	if len(files) > 0 {
		if p.Inplace {
			return p.runAllFilesInPlace(files)
		}
		f, err := os.Open(files[0])
		if err != nil {
			return err
		}
		defer f.Close()
		input = f
	}
	return p.Fn(input, output)
}

func (p *Program) runAllFilesInPlace(files []string) error {
	for _, path := range files {
		err := p.runFileInPlace(path)
		if err != nil {
			return err
		}
	}
	return nil
}

func (p *Program) runFileInPlace(path string) error {
	in, err := ioutil.ReadFile(path)
	if err != nil {
		return err
	}

	out := new(bytes.Buffer)

	err = p.Fn(bytes.NewReader(in), out)
	if err != nil {
		return err
	}

	return ioutil.WriteFile(path, out.Bytes(), 0600)
}

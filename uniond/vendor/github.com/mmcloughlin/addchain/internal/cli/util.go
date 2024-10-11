package cli

import (
	"io"
	"io/ioutil"
	"os"
)

// OpenInput is a convenience for possibly opening an input file, or otherwise returning standard in.
func OpenInput(filename string) (string, io.ReadCloser, error) {
	if filename == "" {
		return "<stdin>", ioutil.NopCloser(os.Stdin), nil
	}
	f, err := os.Open(filename)
	return filename, f, err
}

// OpenOutput is a convenience for possibly opening an output file, or otherwise returning standard out.
func OpenOutput(filename string) (string, io.WriteCloser, error) {
	if filename == "" {
		return "<stdout>", nopwritercloser{os.Stdout}, nil
	}
	f, err := os.Create(filename)
	return filename, f, err
}

// nopwritercloser wraps an io.Writer and provides a no-op Close() method.
type nopwritercloser struct {
	io.Writer
}

func (nopwritercloser) Close() error { return nil }

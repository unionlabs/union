package cmd

import (
	"bufio"
	"io"
	"os"
)

func saveTo(file string, x io.WriterTo) error {
	f, err := os.Create(file)
	if err != nil {
		return err
	}
	defer f.Close()
	w := bufio.NewWriter(f)
	_, err = x.WriteTo(w)
	if err != nil {
		return err
	}
	return w.Flush()
}

func readFrom(file string, obj io.ReaderFrom) error {
	f, err := os.OpenFile(file, os.O_RDONLY, os.ModePerm)
	if err != nil {
		return err
	}
	defer f.Close()
	_, err = obj.ReadFrom(f)
	return err
}

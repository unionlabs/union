package main

import (
	"errors"
	"flag"
	"io/ioutil"
	"os"

	"github.com/golang/snappy"
)

var (
	decode = flag.Bool("d", false, "decode")
	encode = flag.Bool("e", false, "encode")
)

func run() error {
	flag.Parse()
	if *decode == *encode {
		return errors.New("exactly one of -d or -e must be given")
	}

	in, err := ioutil.ReadAll(os.Stdin)
	if err != nil {
		return err
	}

	out := []byte(nil)
	if *decode {
		out, err = snappy.Decode(nil, in)
		if err != nil {
			return err
		}
	} else {
		out = snappy.Encode(nil, in)
	}
	_, err = os.Stdout.Write(out)
	return err
}

func main() {
	if err := run(); err != nil {
		os.Stderr.WriteString(err.Error() + "\n")
		os.Exit(1)
	}
}

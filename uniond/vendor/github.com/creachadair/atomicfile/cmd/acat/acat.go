// Program acat copies its standard input to an output file.
package main

import (
	"flag"
	"fmt"
	"io"
	"log"
	"os"
	"path/filepath"
	"strconv"

	"github.com/creachadair/atomicfile"
)

var (
	fileMode = flag.String("mode", "0600", "Output file mode")
	nonEmpty = flag.Bool("nonempty", false, "Only write if the input is nonempty")
	doTee    = flag.Bool("tee", false, "Also copy stdin to stdout")
)

func init() {
	flag.Usage = func() {
		fmt.Fprintf(os.Stderr, `Usage: %[1]s <output-file>

Copy standard input to the specified file through a temporary file.
In case of error, the original contents of the file, if any, are not
modified; otherwise, the file is replaced in one step by renaming the
temporary file.

If -nonempty is true, %[1]s reports an error without writing the target
file if the input is empty.

If %[1]s is interrupted, the target file is not replaced, but the
temporary file is left intact with any output was written before the
interrupt occurred.

Options:
`, filepath.Base(os.Args[0]))
		flag.PrintDefaults()
	}
}

func main() {
	flag.Parse()
	if flag.NArg() != 1 {
		log.Fatalf("Usage: %s <output-file>", filepath.Base(os.Args[0]))
	}
	mode, err := strconv.ParseInt(*fileMode, 0, 32)
	if err != nil {
		log.Fatalf("Invalid mode %q: %v", *fileMode, err)
	}
	f, err := atomicfile.New(flag.Arg(0), os.FileMode(mode))
	if err != nil {
		log.Fatalf("New: %v", err)
	}
	defer f.Cancel()

	var w io.Writer = f
	if *doTee {
		w = io.MultiWriter(f, os.Stdout)
	}

	nw, err := io.Copy(w, os.Stdin)
	if err != nil {
		f.Cancel()
		log.Fatalf("Copy: %v", err)
	} else if nw == 0 && *nonEmpty {
		return
	} else if err := f.Close(); err != nil {
		log.Fatalf("Close: %v", err)
	}
}

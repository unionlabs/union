package atomicfile_test

import (
	"fmt"
	"io"
	"log"
	"os"
	"path/filepath"
	"strings"

	"github.com/creachadair/atomicfile"
)

var tempDir string

func init() {
	dir, err := os.MkdirTemp("", "example")
	if err != nil {
		panic(err)
	}
	tempDir = dir
}

func cat(path string) {
	f, err := os.Open(path)
	if err != nil {
		log.Fatalf("Open: %v", err)
	}
	defer f.Close()
	io.Copy(os.Stdout, f)
}

func ExampleNew() {
	path := filepath.Join(tempDir, "new.txt")
	f, err := atomicfile.New(path, 0600)
	if err != nil {
		log.Fatalf("New: %v", err)
	}
	defer f.Cancel()

	fmt.Fprintln(f, "Hello, world!")
	if err := f.Close(); err != nil {
		log.Fatalf("Close: %v", err)
	}

	cat(path)
	// Output:
	// Hello, world!
}

func ExampleWriteData() {
	path := filepath.Join(tempDir, "writedata.txt")
	if err := atomicfile.WriteData(path, []byte("99 Luftballons"), 0600); err != nil {
		log.Fatalf("WriteData: %v", err)
	}
	cat(path)
	// Output:
	// 99 Luftballons
}

func ExampleWriteAll() {
	path := filepath.Join(tempDir, "writeall.txt")
	nw, err := atomicfile.WriteAll(path, strings.NewReader("I knew you were trouble"), 0640)
	if err != nil {
		log.Fatalf("WriteAll: %v", err)
	}
	fmt.Println(nw)
	cat(path)
	// Output:
	// 23
	// I knew you were trouble
}

func ExampleFile_Cancel() {
	path := filepath.Join(tempDir, "cancel.txt")
	if err := os.WriteFile(path, []byte("left right\n"), 0600); err != nil {
		log.Fatalf("WriteFile: %v", err)
	}
	cat(path)

	f, err := atomicfile.New(path, 0640)
	if err != nil {
		log.Fatalf("New: %v", err)
	}
	fmt.Fprintln(f, "Hello, world!")
	f.Cancel()

	// After cancellation, Close reports an error.
	if err := f.Close(); err == nil {
		log.Fatal("Close should have reported an error")
	}

	// The target path should not have been modified.
	cat(path)
	// Output:
	// left right
	// left right
}

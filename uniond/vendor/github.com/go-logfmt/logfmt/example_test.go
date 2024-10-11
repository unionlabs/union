package logfmt_test

import (
	"errors"
	"fmt"
	"os"
	"strings"
	"time"

	"github.com/go-logfmt/logfmt"
)

func ExampleEncoder() {
	check := func(err error) {
		if err != nil {
			panic(err)
		}
	}

	e := logfmt.NewEncoder(os.Stdout)

	check(e.EncodeKeyval("id", 1))
	check(e.EncodeKeyval("dur", time.Second+time.Millisecond))
	check(e.EndRecord())

	check(e.EncodeKeyval("id", 1))
	check(e.EncodeKeyval("path", "/path/to/file"))
	check(e.EncodeKeyval("err", errors.New("file not found")))
	check(e.EndRecord())

	// Output:
	// id=1 dur=1.001s
	// id=1 path=/path/to/file err="file not found"
}

func ExampleDecoder() {
	in := `
id=1 dur=1.001s
id=1 path=/path/to/file err="file not found"
`

	d := logfmt.NewDecoder(strings.NewReader(in))
	for d.ScanRecord() {
		for d.ScanKeyval() {
			fmt.Printf("k: %s v: %s\n", d.Key(), d.Value())
		}
		fmt.Println()
	}
	if d.Err() != nil {
		panic(d.Err())
	}

	// Output:
	// k: id v: 1
	// k: dur v: 1.001s
	//
	// k: id v: 1
	// k: path v: /path/to/file
	// k: err v: file not found
}
